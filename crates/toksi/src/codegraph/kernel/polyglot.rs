// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/kernel/polyglot.rs
// created: 2026-11-13
// lastModified: 2026-11-13

use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Parser};

use super::buffers::EmitOut;
use super::{decode_buffers, langs, tsjs, FileExtraction, SymbolEdge, SymbolNode, SymbolRef};
use crate::util::paths::normalize_path;

/// Extracts Vue Single File Components (.vue) script block with line padding (raw EmitOut helper).
pub fn extract_vue(file_path: &str, content: &str) -> Result<EmitOut, String> {
    let blocks = extract_sfc_scripts(content, "vue");
    if let Some(first) = blocks.first() {
        let padded = pad_block(content, first.body_start, first.body_end);
        return tsjs::extract(file_path, &padded, first.script_lang);
    }
    Err("no script block found in vue".to_string())
}

/// Extracts Svelte Single File Components (.svelte) script block with line padding (raw EmitOut helper).
pub fn extract_svelte(file_path: &str, content: &str) -> Result<EmitOut, String> {
    let blocks = extract_sfc_scripts(content, "svelte");
    if let Some(first) = blocks.first() {
        let padded = pad_block(content, first.body_start, first.body_end);
        return tsjs::extract(file_path, &padded, first.script_lang);
    }
    Err("no script block found in svelte".to_string())
}

/// Full Vue SFC extractor supporting multiple <script>/<script setup> blocks,
/// Options API members, template component usages, and template call expressions.
pub fn extract_vue_full(file_path: &str, content: &str) -> FileExtraction {
    extract_sfc_full(file_path, content, "vue")
}

/// Full Svelte SFC extractor supporting module + instance <script> blocks,
/// template `{expr}` call expressions (filtering Svelte runes), and component tags.
pub fn extract_svelte_full(file_path: &str, content: &str) -> FileExtraction {
    extract_sfc_full(file_path, content, "svelte")
}

struct SfcScriptBlock<'a> {
    attrs: &'a str,
    body_start: usize,
    body_end: usize,
    script_lang: &'static str,
    line_offset: u32,
    is_setup_or_instance: bool,
}

fn extract_sfc_scripts<'a>(content: &'a str, host_lang: &str) -> Vec<SfcScriptBlock<'a>> {
    let mut blocks = Vec::new();
    let lower = content.to_ascii_lowercase();
    let mut cursor = 0usize;

    while let Some(rel_start) = lower[cursor..].find("<script") {
        let start_idx = cursor + rel_start;
        let Some(rel_gt) = lower[start_idx..].find('>') else {
            break;
        };
        let open_end = start_idx + rel_gt;
        let attrs = &content[start_idx + 7..open_end];
        if content[..=open_end].trim_end().ends_with("/>") {
            cursor = open_end + 1;
            continue;
        }
        let body_start = open_end + 1;
        let body_end = lower[body_start..]
            .find("</script")
            .map(|e| body_start + e)
            .unwrap_or(content.len());
        let after_end = lower[body_end..]
            .find('>')
            .map(|e| body_end + e + 1)
            .unwrap_or(content.len());

        let attrs_lower = attrs.to_ascii_lowercase();
        let script_lang = if attrs_lower.contains("tsx") && host_lang == "vue" {
            "tsx"
        } else if attrs_lower.contains("lang=\"ts\"")
            || attrs_lower.contains("lang='ts'")
            || attrs_lower.contains("typescript")
        {
            "typescript"
        } else {
            "javascript"
        };

        let is_setup_or_instance = if host_lang == "vue" {
            attrs_lower.split_whitespace().any(|tok| tok == "setup")
                || attrs_lower.contains(" setup")
        } else {
            !(attrs_lower.contains("context=\"module\"")
                || attrs_lower.contains("context='module'")
                || attrs_lower.split_whitespace().any(|tok| tok == "module"))
        };

        let line_offset = content[..body_start].bytes().filter(|&b| b == b'\n').count() as u32;
        blocks.push(SfcScriptBlock {
            attrs,
            body_start,
            body_end,
            script_lang,
            line_offset,
            is_setup_or_instance,
        });
        cursor = after_end;
    }
    let _ = &blocks.first().map(|b| b.attrs);
    blocks
}

fn pad_block(content: &str, body_start: usize, body_end: usize) -> String {
    let mut padded = String::with_capacity(content.len());
    for (idx, ch) in content.char_indices() {
        if idx >= body_start && idx < body_end {
            padded.push(ch);
        } else if ch == '\n' {
            padded.push('\n');
        } else {
            padded.push(' ');
        }
    }
    padded
}

fn line_count(s: &str) -> u32 {
    s.lines().count().max(1) as u32
}

fn base_file_name(norm_path: &str) -> &str {
    norm_path.rsplit('/').next().unwrap_or(norm_path)
}

fn extract_sfc_full(file_path: &str, content: &str, host_lang: &str) -> FileExtraction {
    let norm_path = normalize_path(file_path);
    let file_id = format!("file:{}", norm_path);
    let file_name = base_file_name(&norm_path).to_string();
    let total_lines = line_count(content);
    let comp_name = file_name
        .strip_suffix(".vue")
        .or_else(|| file_name.strip_suffix(".svelte"))
        .unwrap_or(&file_name)
        .to_string();
    let comp_id = format!("component:sfc:{}", norm_path);

    let mut nodes = vec![
        SymbolNode {
            id: file_id.clone(),
            name: file_name,
            qualified_name: norm_path.clone(),
            kind: "file".to_string(),
            visibility: None,
            file_path: norm_path.clone(),
            start_line: 1,
            end_line: total_lines,
            start_column: 0,
            end_column: 0,
            docstring: None,
            signature: None,
            return_type: None,
        },
        SymbolNode {
            id: comp_id.clone(),
            name: comp_name.clone(),
            qualified_name: format!("{}::{}", norm_path, comp_name),
            kind: "component".to_string(),
            visibility: Some("public".to_string()),
            file_path: norm_path.clone(),
            start_line: 1,
            end_line: total_lines,
            start_column: 0,
            end_column: content.lines().last().map(|l| l.len() as u32).unwrap_or(0),
            docstring: None,
            signature: Some(format!("<{}> component {}", host_lang, comp_name)),
            return_type: None,
        },
    ];
    let mut edges = vec![SymbolEdge {
        source_id: file_id.clone(),
        target_id: comp_id.clone(),
        kind: "contains".to_string(),
        line: Some(1),
        column: Some(0),
    }];
    let mut refs = Vec::new();

    let blocks = extract_sfc_scripts(content, host_lang);
    for block in &blocks {
        let block_src = &content[block.body_start..block.body_end];
        if let Ok(emit) = tsjs::extract(&norm_path, block_src, block.script_lang) {
            if let Ok(mut ext) = decode_buffers(&norm_path, &emit) {
                if host_lang == "vue" && !block.is_setup_or_instance {
                    attach_vue_options_members(block_src, &comp_name, &norm_path, &mut ext);
                }

                let mut parented = HashSet::new();
                for e in &ext.edges {
                    if e.kind == "contains" && e.source_id != file_id {
                        parented.insert(e.target_id.clone());
                    }
                }
                let mut instance_values = HashSet::new();
                if block.is_setup_or_instance {
                    for n in &ext.nodes {
                        if (n.kind == "constant" || n.kind == "variable") && !parented.contains(&n.id) {
                            instance_values.insert(n.id.clone());
                        }
                    }
                }

                for mut n in ext.nodes {
                    if n.kind == "file" {
                        continue;
                    }
                    n.start_line += block.line_offset;
                    n.end_line += block.line_offset;
                    let is_top = !parented.contains(&n.id);
                    let id = n.id.clone();
                    nodes.push(n);
                    if is_top {
                        edges.push(SymbolEdge {
                            source_id: comp_id.clone(),
                            target_id: id,
                            kind: "contains".to_string(),
                            line: None,
                            column: None,
                        });
                    }
                }

                for mut e in ext.edges {
                    if e.kind == "contains" && e.source_id == file_id {
                        continue;
                    }
                    if let Some(ref mut l) = e.line {
                        *l += block.line_offset;
                    }
                    if block.is_setup_or_instance
                        && (e.source_id == file_id || instance_values.contains(&e.source_id))
                        && e.kind != "imports"
                        && e.kind != "contains"
                    {
                        e.source_id = comp_id.clone();
                    }
                    edges.push(e);
                }

                for mut r in ext.refs {
                    r.line += block.line_offset;
                    if block.is_setup_or_instance
                        && (r.from_node_id == file_id || instance_values.contains(&r.from_node_id))
                        && matches!(
                            r.kind.as_str(),
                            "calls" | "references" | "instantiates" | "function_ref"
                        )
                    {
                        r.from_node_id = comp_id.clone();
                    }
                    refs.push(r);
                }
            }
        }
    }

    // Collect own Vue Options API methods before scanning template calls
    let mut own_methods: HashMap<String, String> = HashMap::new();
    if host_lang == "vue" {
        let prefix = format!("{}::", comp_name);
        for n in &nodes {
            if n.kind == "method" && n.qualified_name.starts_with(&prefix) {
                own_methods.insert(n.name.clone(), n.id.clone());
            }
        }
    }

    let skip_ranges = sfc_script_style_line_ranges(content);
    scan_sfc_template(
        &norm_path,
        content,
        host_lang,
        &comp_id,
        &skip_ranges,
        &own_methods,
        &mut edges,
        &mut refs,
    );

    FileExtraction { nodes, edges, refs }
}

fn sfc_script_style_line_ranges(content: &str) -> Vec<(u32, u32)> {
    let mut ranges = Vec::new();
    let lower = content.to_ascii_lowercase();
    for tag in ["<script", "<style"] {
        let end_tag = if tag == "<script" { "</script" } else { "</style" };
        let mut cursor = 0usize;
        while let Some(rel) = lower[cursor..].find(tag) {
            let start = cursor + rel;
            let Some(gt_rel) = lower[start..].find('>') else {
                break;
            };
            let open_end = start + gt_rel;
            if content[..=open_end].trim_end().ends_with("/>") {
                cursor = open_end + 1;
                continue;
            }
            let close = lower[open_end + 1..]
                .find(end_tag)
                .map(|c| open_end + 1 + c)
                .unwrap_or(content.len());
            let start_line = content[..start].bytes().filter(|&b| b == b'\n').count() as u32 + 1;
            let end_line = content[..close].bytes().filter(|&b| b == b'\n').count() as u32 + 1;
            ranges.push((start_line, end_line));
            cursor = close;
        }
    }
    ranges
}

fn is_vue_builtin(name: &str) -> bool {
    matches!(
        name,
        "Transition" | "TransitionGroup" | "KeepAlive" | "Suspense" | "Teleport" | "Component" | "Slot"
    )
}

fn is_svelte_rune(name: &str) -> bool {
    matches!(
        name,
        "$props" | "$state" | "$derived" | "$effect" | "$bindable" | "$inspect" | "$host" | "$snippet"
    )
}

fn kebab_to_pascal(raw: &str) -> String {
    raw.split('-')
        .filter(|p| !p.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn attach_vue_options_members(
    block_src: &str,
    owner: &str,
    file_path: &str,
    ext: &mut FileExtraction,
) {
    // Detect object-literal method / hook definitions in Options API blocks
    let known_hooks = [
        "data", "setup", "render", "beforeCreate", "created", "beforeMount", "mounted",
        "beforeUpdate", "updated", "activated", "deactivated", "beforeDestroy", "destroyed",
        "beforeUnmount", "unmounted", "errorCaptured", "renderTracked", "renderTriggered",
        "serverPrefetch", "asyncData", "fetch", "head", "validate", "middleware",
    ];
    let mut created = Vec::new();
    let lines: Vec<&str> = block_src.lines().collect();
    let mut in_member_group = false;
    let mut group_brace_depth: i32 = 0;

    for (idx, raw_line) in lines.iter().enumerate() {
        let trimmed = raw_line.trim();
        let line_no = (idx + 1) as u32;
        if (trimmed.starts_with("methods:") || trimmed.starts_with("computed:") || trimmed.starts_with("watch:"))
            && trimmed.contains('{')
        {
            in_member_group = true;
            group_brace_depth = 1;
            continue;
        }
        if in_member_group {
            for ch in trimmed.chars() {
                match ch {
                    '{' => group_brace_depth += 1,
                    '}' => group_brace_depth -= 1,
                    _ => {}
                }
            }
            if group_brace_depth <= 0 {
                in_member_group = false;
                continue;
            }
            if group_brace_depth <= 2 {
                if let Some(name) = extract_js_obj_method_name(trimmed) {
                    let id = format!("method:vue:{}:{}:{}", file_path, name, line_no);
                    created.push(SymbolNode {
                        id,
                        name: name.clone(),
                        qualified_name: format!("{}::{}", owner, name),
                        kind: "method".to_string(),
                        visibility: None,
                        file_path: file_path.to_string(),
                        start_line: line_no,
                        end_line: find_brace_end_line(&lines, idx),
                        start_column: 0,
                        end_column: trimmed.len() as u32,
                        docstring: None,
                        signature: Some(trimmed.to_string()),
                        return_type: None,
                    });
                }
            }
        } else if let Some(name) = extract_js_obj_method_name(trimmed) {
            if known_hooks.contains(&name.as_str()) {
                let id = format!("method:vue:{}:{}:{}", file_path, name, line_no);
                created.push(SymbolNode {
                    id,
                    name: name.clone(),
                    qualified_name: format!("{}::{}", owner, name),
                    kind: "method".to_string(),
                    visibility: None,
                    file_path: file_path.to_string(),
                    start_line: line_no,
                    end_line: find_brace_end_line(&lines, idx),
                    start_column: 0,
                    end_column: trimmed.len() as u32,
                    docstring: None,
                    signature: Some(trimmed.to_string()),
                    return_type: None,
                });
            }
        }
    }

    if created.is_empty() {
        return;
    }

    let file_id = format!("file:{}", file_path);
    for r in &mut ext.refs {
        if r.from_node_id == file_id {
            if let Some(m) = created.iter().find(|n| n.start_line <= r.line && r.line <= n.end_line) {
                r.from_node_id = m.id.clone();
            }
        }
    }
    ext.nodes.extend(created);
}

fn extract_js_obj_method_name(trimmed: &str) -> Option<String> {
    let s = trimmed.strip_prefix("async ").unwrap_or(trimmed).trim();
    if let Some(paren_idx) = s.find('(') {
        let head = s[..paren_idx].trim();
        let ident = head.strip_suffix(':').map(|x| x.trim()).unwrap_or(head);
        let ident = ident.strip_prefix("function").map(|x| x.trim()).unwrap_or(ident);
        if !ident.is_empty()
            && !matches!(ident, "if" | "for" | "while" | "switch" | "catch" | "return")
            && ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
            && s.contains('{')
        {
            return Some(ident.to_string());
        }
    }
    None
}

fn find_brace_end_line(lines: &[&str], start_idx: usize) -> u32 {
    let mut depth = 0i32;
    let mut started = false;
    for (offset, line) in lines[start_idx..].iter().enumerate() {
        for ch in line.chars() {
            if ch == '{' {
                depth += 1;
                started = true;
            } else if ch == '}' {
                depth -= 1;
            }
        }
        if started && depth <= 0 {
            return (start_idx + offset + 1) as u32;
        }
    }
    (start_idx + 1) as u32
}

#[allow(clippy::too_many_arguments)]
fn scan_sfc_template(
    file_path: &str,
    content: &str,
    host_lang: &str,
    comp_id: &str,
    skip_ranges: &[(u32, u32)],
    own_methods: &HashMap<String, String>,
    edges: &mut Vec<SymbolEdge>,
    refs: &mut Vec<SymbolRef>,
) {
    let tag_re = regex::Regex::new(r"<([A-Za-z][A-Za-z0-9_$-]*)\b").ok();
    let call_re = regex::Regex::new(r"\b([A-Za-z_$][\w$.]*)\s*\(").ok();
    let brace_re = regex::Regex::new(r"\{\{([^}]+)\}\}|\{([^}#/:@][^}]*)\}").ok();
    let directive_re = regex::Regex::new(r#"(?:[:@]|v-[a-z-]+(?:[:.][a-z-]+)*)=["']([^"']+)["']"#).ok();

    for (idx, line) in content.lines().enumerate() {
        let line_no = (idx + 1) as u32;
        if skip_ranges.iter().any(|&(s, e)| line_no >= s && line_no <= e) {
            continue;
        }

        if let Some(ref tre) = tag_re {
            for cap in tre.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    let raw = m.as_str();
                    let comp_name = if raw.starts_with(|c: char| c.is_ascii_uppercase()) {
                        raw.to_string()
                    } else if host_lang == "vue" && raw.contains('-') {
                        kebab_to_pascal(raw)
                    } else {
                        continue;
                    };
                    if host_lang == "vue" && is_vue_builtin(&comp_name) {
                        continue;
                    }
                    refs.push(SymbolRef {
                        from_node_id: comp_id.to_string(),
                        name: comp_name,
                        kind: "references".to_string(),
                        line: line_no,
                        column: (m.start() + 1) as u32,
                        file_path: file_path.to_string(),
                    });
                }
            }
        }

        let mut scan_expr = |expr: &str, col_base: usize| {
            let Some(ref cre) = call_re else {
                return;
            };
            for cap in cre.captures_iter(expr) {
                let Some(m) = cap.get(1) else { continue };
                let callee = m.as_str();
                if matches!(callee, "if" | "else" | "each" | "await" | "for" | "while" | "return") {
                    continue;
                }
                if host_lang == "svelte" && is_svelte_rune(callee) {
                    continue;
                }
                if host_lang == "vue" && !callee.contains('.') {
                    if let Some(method_id) = own_methods.get(callee) {
                        edges.push(SymbolEdge {
                            source_id: comp_id.to_string(),
                            target_id: method_id.clone(),
                            kind: "calls".to_string(),
                            line: Some(line_no),
                            column: Some((col_base + m.start()) as u32),
                        });
                        continue;
                    }
                }
                refs.push(SymbolRef {
                    from_node_id: comp_id.to_string(),
                    name: callee.to_string(),
                    kind: "calls".to_string(),
                    line: line_no,
                    column: (col_base + m.start()) as u32,
                    file_path: file_path.to_string(),
                });
            }
        };

        if let Some(ref bre) = brace_re {
            for cap in bre.captures_iter(line) {
                if let Some(m) = cap.get(1).or_else(|| cap.get(2)) {
                    scan_expr(m.as_str(), m.start());
                }
            }
        }
        if host_lang == "vue" {
            if let Some(ref dre) = directive_re {
                for cap in dre.captures_iter(line) {
                    if let Some(m) = cap.get(1) {
                        scan_expr(m.as_str(), m.start());
                    }
                }
            }
        }
    }
}

/// Primary polyglot extractor: uses real Tree-sitter CST traversal for all supported
/// polyglot languages and structured outline extraction where no CST grammar exists.
pub fn extract_structural(file_path: &str, content: &str, language: &str) -> FileExtraction {
    let norm_path = normalize_path(file_path);
    let canon_lang = langs::canonical_language(language);
    if let Some(ts_lang) = langs::grammar_for(canon_lang) {
        if let Some(cst) = extract_polyglot_cst(&norm_path, content, canon_lang, ts_lang) {
            return cst;
        }
    }
    extract_fallback_outline(&norm_path, content, canon_lang)
}

fn extract_polyglot_cst(
    norm_path: &str,
    content: &str,
    language: &str,
    ts_lang: tree_sitter::Language,
) -> Option<FileExtraction> {
    let mut parser = Parser::new();
    parser.set_language(&ts_lang).ok()?;
    let tree = parser.parse(content, None)?;
    let root = tree.root_node();

    let file_id = format!("file:{}", norm_path);
    let file_name = base_file_name(norm_path).to_string();
    let total_lines = line_count(content);

    let mut nodes = vec![SymbolNode {
        id: file_id.clone(),
        name: file_name,
        qualified_name: norm_path.to_string(),
        kind: "file".to_string(),
        visibility: None,
        file_path: norm_path.to_string(),
        start_line: 1,
        end_line: total_lines,
        start_column: 0,
        end_column: 0,
        docstring: None,
        signature: None,
        return_type: None,
    }];
    let mut edges = Vec::new();
    let mut refs = Vec::new();
    let src = content.as_bytes();

    walk_cst_node(
        root,
        src,
        norm_path,
        language,
        &file_id,
        None,
        0,
        &mut nodes,
        &mut edges,
        &mut refs,
    );

    Some(FileExtraction { nodes, edges, refs })
}

#[allow(clippy::too_many_arguments)]
fn walk_cst_node(
    node: Node,
    src: &[u8],
    norm_path: &str,
    language: &str,
    parent_id: &str,
    parent_qn: Option<&str>,
    depth: usize,
    nodes: &mut Vec<SymbolNode>,
    edges: &mut Vec<SymbolEdge>,
    refs: &mut Vec<SymbolRef>,
) {
    if depth > 128 {
        return;
    }

    let kind = node.kind();
    let mut next_parent_id = parent_id.to_string();
    let mut next_parent_qn = parent_qn.map(str::to_string);

    if let Some((sym_kind, sym_name)) = classify_cst_symbol(node, src, language) {
        let start_pos = node.start_position();
        let end_pos = node.end_position();
        let start_line = (start_pos.row + 1) as u32;
        let end_line = (end_pos.row + 1) as u32;
        let start_column = start_pos.column as u32;
        let end_column = end_pos.column as u32;
        let qualified_name = match parent_qn {
            Some(pqn) => format!("{}::{}", pqn, sym_name),
            None => sym_name.clone(),
        };
        let sym_id = format!("{}:{}:{}:{}", sym_kind, norm_path, sym_name, start_line);
        let first_line = node
            .utf8_text(src)
            .ok()
            .and_then(|t| t.lines().next())
            .map(|l| l.trim().to_string());
        let doc = preceding_comment(node, src);

        nodes.push(SymbolNode {
            id: sym_id.clone(),
            name: sym_name,
            qualified_name: qualified_name.clone(),
            kind: sym_kind.to_string(),
            visibility: None,
            file_path: norm_path.to_string(),
            start_line,
            end_line,
            start_column,
            end_column,
            docstring: doc,
            signature: first_line,
            return_type: None,
        });
        edges.push(SymbolEdge {
            source_id: parent_id.to_string(),
            target_id: sym_id.clone(),
            kind: "contains".to_string(),
            line: Some(start_line),
            column: Some(start_column),
        });
        next_parent_id = sym_id;
        next_parent_qn = Some(qualified_name);
    } else if let Some(ref_pair) = classify_cst_ref(node, src, language) {
        let pos = node.start_position();
        refs.push(SymbolRef {
            from_node_id: parent_id.to_string(),
            name: ref_pair.1,
            kind: ref_pair.0.to_string(),
            line: (pos.row + 1) as u32,
            column: pos.column as u32,
            file_path: norm_path.to_string(),
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_cst_node(
            child,
            src,
            norm_path,
            language,
            &next_parent_id,
            next_parent_qn.as_deref(),
            depth + 1,
            nodes,
            edges,
            refs,
        );
    }
    let _ = kind;
}

fn node_text<'a>(node: Node<'a>, src: &'a [u8]) -> Option<&'a str> {
    node.utf8_text(src).ok().map(str::trim).filter(|s| !s.is_empty())
}

fn first_ident_child(node: Node, src: &[u8], kinds: &[&str]) -> Option<String> {
    for field in ["name", "declarator", "title", "key", "target", "identifier"] {
        if let Some(child) = node.child_by_field_name(field) {
            if let Some(txt) = node_text(child, src) {
                let clean = txt.trim_matches(|c| c == '"' || c == '\'').trim();
                if !clean.is_empty() && !clean.contains('\n') && clean.len() <= 120 {
                    return Some(clean.to_string());
                }
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if kinds.contains(&child.kind()) {
            if let Some(txt) = node_text(child, src) {
                let clean = txt.trim_matches(|c| c == '"' || c == '\'').trim();
                if !clean.is_empty() && !clean.contains('\n') && clean.len() <= 120 {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

fn classify_cst_symbol(node: Node, src: &[u8], language: &str) -> Option<(&'static str, String)> {
    let kind = node.kind();
    // Language-specific Elixir def/defmodule syntax represented as `call`
    if language == "elixir" && kind == "call" {
        let mut cursor = node.walk();
        let target = node.child_by_field_name("target").or_else(|| {
            node.named_children(&mut cursor)
                .find(|c| c.kind() == "identifier")
        });
        if let Some(target) = target {
            let kw = node_text(target, src).unwrap_or("");
            if matches!(kw, "def" | "defp" | "defmacro" | "defmodule" | "defprotocol" | "defimpl") {
                let mut acur = node.walk();
                let args = node.child_by_field_name("arguments").or_else(|| {
                    node.named_children(&mut acur)
                        .find(|c| c.kind() == "arguments")
                });
                if let Some(args) = args {
                    if let Some(arg0) = args.named_child(0) {
                        let name = first_ident_child(arg0, src, &["identifier", "alias", "atom"])
                            .or_else(|| node_text(arg0, src).map(|s| s.split(['(', ',']).next().unwrap_or(s).trim().to_string()));
                        if let Some(name) = name.filter(|s| !s.is_empty()) {
                            let sk = if kw == "defmodule" { "module" } else if kw == "defprotocol" { "protocol" } else { "function" };
                            return Some((sk, name));
                        }
                    }
                }
            }
        }
        return None;
    }

    // Markdown headings and LaTeX sections
    if matches!(kind, "atx_heading" | "setext_heading") {
        let text = node_text(node, src)?;
        let title = text.trim_start_matches('#').trim();
        if !title.is_empty() {
            return Some(("section", title.to_string()));
        }
        return None;
    }
    if matches!(kind, "chapter" | "section" | "subsection" | "subsubsection") {
        let name = first_ident_child(node, src, &["curly_group", "text"])?;
        return Some(("section", name.trim_matches(|c| c == '{' || c == '}').to_string()));
    }

    // CSS / SCSS rulesets & keyframes
    if matches!(kind, "rule_set" | "mixin_statement") {
        if let Some(sel) = first_ident_child(node, src, &["selectors", "class_selector", "id_selector", "tag_name", "identifier"]) {
            return Some(("rule", sel));
        }
    }

    // Structured data top-level keys (JSON / YAML / TOML)
    if matches!(kind, "pair" | "block_mapping_pair") {
        let is_top_level = node
            .parent()
            .and_then(|p| p.parent())
            .map(|gp| gp.parent().is_none() || matches!(gp.kind(), "document" | "stream"))
            .unwrap_or(false);
        if is_top_level {
            if let Some(k) = first_ident_child(node, src, &["string", "string_scalar", "plain_scalar", "bare_key", "quoted_key"]) {
                return Some(("key", k));
            }
        }
        return None;
    }

    // HCL blocks (resource / variable / output / module / data)
    if language == "hcl" && kind == "block" {
        if let Some(first_line) = node_text(node, src).and_then(|t| t.lines().next()) {
            let head = first_line.trim_end_matches('{').trim();
            if !head.is_empty() {
                return Some(("block", head.to_string()));
            }
        }
        return None;
    }

    // Make targets & CMake functions
    if language == "make" && kind == "rule" {
        if let Some(t) = first_ident_child(node, src, &["targets", "word"]) {
            return Some(("target", t));
        }
    }
    if language == "cmake" && kind == "function_def" {
        if let Some(t) = first_ident_child(node, src, &["argument", "identifier"]) {
            return Some(("function", t));
        }
    }

    // Standard polyglot AST symbol declarations
    let sym_kind = match kind {
        "function_declaration"
        | "function_definition"
        | "function_item"
        | "fn_proto"
        | "fun_declaration"
        | "fun_definition"
        | "function"
        | "signature"
        | "value_binding"
        | "normal_command" => {
            if language == "cmake" && kind == "normal_command" {
                let txt = node_text(node, src)?;
                let lower = txt.to_ascii_lowercase();
                if !lower.starts_with("function(") && !lower.starts_with("macro(") {
                    return None;
                }
            }
            "function"
        }
        "rpc" | "rpc_declaration" => "function",
        "method_declaration" | "method_definition" => "method",
        "struct_declaration" | "struct_definition" | "struct_item" => "struct",
        "class_declaration" | "class_definition" => "class",
        "interface_declaration" | "interface_type_definition" => "interface",
        "module_declaration" | "module_definition" | "module" => "module",
        "enum_declaration" | "enum_definition" | "enum_type_definition" => "enum",
        "service_declaration" | "service" => "service",
        "message" | "message_declaration" | "create_table_statement" | "create_table" => "struct",
        "type_declaration" | "type_definition" | "type_alias_declaration" | "type_binding" | "object_type_definition" => "type",
        "binding" if language == "nix" => "variable",
        "operation_definition" | "fragment_definition" if language == "graphql" => "function",
        _ => return None,
    };

    let name = first_ident_child(
        node,
        src,
        &[
            "identifier",
            "variable_name",
            "type_identifier",
            "name",
            "function_name",
            "word",
            "attrpath",
            "value_name",
            "type_constructor",
            "table_name",
            "object_reference",
            "message_name",
            "enum_name",
            "service_name",
        ],
    )?;

    Some((sym_kind, name))
}

fn classify_cst_ref(node: Node, src: &[u8], _language: &str) -> Option<(&'static str, String)> {
    match node.kind() {
        "call_expression" | "function_call" | "call" => {
            let target = node
                .child_by_field_name("function")
                .or_else(|| node.child_by_field_name("name"))
                .or_else(|| node.named_child(0))?;
            let txt = node_text(target, src)?;
            if !txt.contains('\n') && txt.len() <= 80 {
                return Some(("calls", txt.to_string()));
            }
        }
        "use_declaration" | "import_statement" | "import_declaration" | "include_directive" => {
            let txt = node_text(node, src)?;
            if let Some(mod_name) = txt.split_whitespace().last() {
                let clean = mod_name.trim_end_matches(';').trim_matches(|c| c == '"' || c == '\'');
                if !clean.is_empty() && clean.len() <= 120 {
                    return Some(("imports", clean.to_string()));
                }
            }
        }
        _ => {}
    }
    None
}

fn preceding_comment(node: Node, src: &[u8]) -> Option<String> {
    let mut cur = node.prev_named_sibling();
    let mut comments = Vec::new();
    while let Some(sib) = cur {
        let k = sib.kind();
        if k.contains("comment") {
            if let Some(t) = node_text(sib, src) {
                let cleaned = t
                    .trim_start_matches(['/', '#', '-', '*'])
                    .trim_end_matches("*/")
                    .trim();
                if !cleaned.is_empty() {
                    comments.push(cleaned.to_string());
                }
            }
            cur = sib.prev_named_sibling();
        } else {
            break;
        }
    }
    if comments.is_empty() {
        None
    } else {
        comments.reverse();
        Some(comments.join(" "))
    }
}

/// Fallback outline extractor for languages without CST grammars (e.g., Dockerfile) or parse failure.
fn extract_fallback_outline(norm_path: &str, content: &str, language: &str) -> FileExtraction {
    let mut nodes = Vec::new();
    let file_id = format!("file:{}", norm_path);
    let file_name = base_file_name(norm_path).to_string();
    let total_lines = line_count(content);

    nodes.push(SymbolNode {
        id: file_id.clone(),
        name: file_name,
        qualified_name: norm_path.to_string(),
        kind: "file".to_string(),
        visibility: None,
        file_path: norm_path.to_string(),
        start_line: 1,
        end_line: total_lines,
        start_column: 0,
        end_column: 0,
        docstring: None,
        signature: None,
        return_type: None,
    });

    let mut edges = Vec::new();
    let mut current_doc = String::new();
    for (idx, line) in content.lines().enumerate() {
        let line_num = (idx + 1) as u32;
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            let doc_line = trimmed.trim_start_matches(['#', '/']).trim();
            if !doc_line.is_empty() {
                if !current_doc.is_empty() {
                    current_doc.push(' ');
                }
                current_doc.push_str(doc_line);
            }
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }
        let docstring = if current_doc.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut current_doc))
        };

        if let Some((name, kind, signature)) = extract_line_symbol(trimmed, language) {
            let sym_id = format!("{}:{}:{}:{}", kind, norm_path, name, line_num);
            nodes.push(SymbolNode {
                id: sym_id.clone(),
                name: name.clone(),
                qualified_name: name,
                kind: kind.to_string(),
                visibility: None,
                file_path: norm_path.to_string(),
                start_line: line_num,
                end_line: line_num,
                start_column: 0,
                end_column: line.len() as u32,
                docstring,
                signature: Some(signature),
                return_type: None,
            });
            edges.push(SymbolEdge {
                source_id: file_id.clone(),
                target_id: sym_id,
                kind: "contains".to_string(),
                line: Some(line_num),
                column: Some(0),
            });
        }
    }

    FileExtraction {
        nodes,
        edges,
        refs: Vec::new(),
    }
}

fn extract_line_symbol(line: &str, language: &str) -> Option<(String, &'static str, String)> {
    if language == "dockerfile" {
        for kw in ["FROM ", "ARG ", "ENV ", "EXPOSE ", "WORKDIR ", "ENTRYPOINT ", "CMD "] {
            if let Some(rest) = line.strip_prefix(kw) {
                let name = rest.split_whitespace().next().unwrap_or(kw.trim()).to_string();
                let kind = if kw == "FROM " { "stage" } else { "instruction" };
                return Some((name, kind, line.to_string()));
            }
        }
        return None;
    }

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 && matches!(parts[0], "fn" | "def" | "func" | "function" | "sub" | "proc") {
        let name = parts[1].split('(').next().unwrap_or(parts[1]).trim();
        if !name.is_empty() {
            return Some((name.to_string(), "function", line.to_string()));
        }
    }
    if parts.len() >= 2 && matches!(parts[0], "class" | "struct" | "interface" | "type" | "enum") {
        let name = parts[1].trim_end_matches([':', '{', '(']);
        if !name.is_empty() {
            let kind = match parts[0] {
                "class" => "class",
                "struct" => "struct",
                "interface" => "interface",
                _ => "type",
            };
            return Some((name.to_string(), kind, line.to_string()));
        }
    }
    None
}
