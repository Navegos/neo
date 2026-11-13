// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/kernel/mod.rs
// created: 2026-10-28
// lastModified: 2026-11-13

#[macro_export]
macro_rules! stack_guard {
    () => {
        if $crate::codegraph::kernel::stack::exhausted() {
            return ::core::default::Default::default();
        }
    };
}

pub mod buffers;
pub mod ccpp;
pub mod cfnptr;
pub mod csharp;
pub mod dart;
pub mod docstring;
pub mod go;
pub mod ids;
pub mod java;
pub mod kotlin;
pub mod langs;
pub mod lua;
pub mod php;
pub mod polyglot;
pub mod python;
pub mod rlang;
pub mod ruby;
pub mod rustlang;
pub mod scala;
pub mod stack;
pub mod swift;
pub mod textutil;
pub mod tsjs;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolNode {
    pub id: String,
    pub name: String,
    pub qualified_name: String,
    pub kind: String,
    pub visibility: Option<String>,
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub start_column: u32,
    pub end_column: u32,
    pub docstring: Option<String>,
    pub signature: Option<String>,
    pub return_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolEdge {
    pub source_id: String,
    pub target_id: String,
    pub kind: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolRef {
    pub from_node_id: String,
    pub name: String,
    pub kind: String,
    pub line: u32,
    pub column: u32,
    pub file_path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileExtraction {
    pub nodes: Vec<SymbolNode>,
    pub edges: Vec<SymbolEdge>,
    pub refs: Vec<SymbolRef>,
}

fn get_arena_str<'a>(arena: &'a [u8], offset: u32, len: u32) -> Option<&'a str> {
    if offset == buffers::NONE || (offset as usize + len as usize) > arena.len() {
        None
    } else {
        std::str::from_utf8(&arena[offset as usize..(offset + len) as usize]).ok()
    }
}

pub fn extract_file(file_path: &str, content: &str, language: &str) -> Result<FileExtraction, String> {
    let norm_path = crate::util::paths::normalize_path(file_path);
    let canon = langs::canonical_language(language);

    if canon == "vue" {
        return Ok(polyglot::extract_vue_full(&norm_path, content));
    }
    if canon == "svelte" {
        return Ok(polyglot::extract_svelte_full(&norm_path, content));
    }

    let out = stack::run_guarded(|| match canon {
        "rust" => rustlang::extract(&norm_path, content),
        "python" => python::extract(&norm_path, content),
        "go" => go::extract(&norm_path, content),
        "c" | "cpp" => ccpp::extract(&norm_path, content, canon),
        "typescript" | "tsx" | "javascript" | "jsx" => tsjs::extract(&norm_path, content, canon),
        "lua" | "luau" => lua::extract(&norm_path, content, canon),
        "dart" => dart::extract(&norm_path, content),
        "kotlin" => kotlin::extract(&norm_path, content),
        "scala" => scala::extract(&norm_path, content),
        "swift" => swift::extract(&norm_path, content),
        "csharp" => csharp::extract(&norm_path, content),
        "java" => java::extract(&norm_path, content),
        "php" => php::extract(&norm_path, content),
        "ruby" => ruby::extract(&norm_path, content),
        "r" => rlang::extract(&norm_path, content),
        _ => Err(format!("polyglot CST extraction: {}", canon)),
    });

    match out {
        Ok(out_buffers) => {
            let mut ext = decode_buffers(&norm_path, &out_buffers)?;
            if matches!(canon, "c" | "cpp") {
                synthesize_cfnptr_dispatch(content, &norm_path, &mut ext);
            }
            Ok(ext)
        }
        Err(_) => Ok(polyglot::extract_structural(&norm_path, content, canon)),
    }
}

/// Synthesize C/C++ indirect function-pointer dispatch edges/refs using `cfnptr::scan_file`.
fn synthesize_cfnptr_dispatch(content: &str, norm_path: &str, ext: &mut FileExtraction) {
    let structs: Vec<cfnptr::StructExtent> = ext
        .nodes
        .iter()
        .filter(|n| n.kind == "struct" || n.kind == "class")
        .map(|n| cfnptr::StructExtent {
            id: n.id.clone(),
            start_line: n.start_line,
            end_line: n.end_line,
        })
        .collect();
    let facts = cfnptr::scan_file(content, &structs);
    if facts.dispatch_fields.is_empty() && facts.array_dispatch_names.is_empty() {
        return;
    }

    let fn_names: std::collections::HashSet<&str> = ext
        .nodes
        .iter()
        .filter(|n| n.kind == "function" || n.kind == "method")
        .map(|n| n.name.as_str())
        .collect();
    if fn_names.is_empty() {
        return;
    }

    // Collect candidate handler functions bound via designated/positional initializers or assignments
    let mut handlers: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for sf in &facts.structs {
        for raw in &sf.fields {
            if !raw.name.is_empty() && facts.dispatch_fields.contains(&raw.name) {
                let needle = format!(".{}", raw.name);
                for line in content.lines() {
                    if let Some(pos) = line.find(&needle) {
                        let after = line[pos + needle.len()..].trim_start();
                        if let Some(rhs) = after.strip_prefix('=') {
                            let cand = rhs
                                .trim_start()
                                .trim_start_matches('&')
                                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                                .next()
                                .unwrap_or("");
                            if fn_names.contains(cand) {
                                handlers.entry(raw.name.clone()).or_default().push(cand.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    for (field, cands) in handlers {
        let mut uniq = cands;
        uniq.sort();
        uniq.dedup();
        let needle = format!("{}(", field);
        for (idx, line) in content.lines().enumerate() {
            if line.contains(&needle) && (line.contains("->") || line.contains('.')) {
                let line_no = (idx + 1) as u32;
                let owner_id = ext
                    .nodes
                    .iter()
                    .find(|n| {
                        (n.kind == "function" || n.kind == "method")
                            && n.start_line <= line_no
                            && line_no <= n.end_line
                    })
                    .map(|n| n.id.clone())
                    .unwrap_or_else(|| format!("file:{}", norm_path));
                for handler in &uniq {
                    ext.refs.push(SymbolRef {
                        from_node_id: owner_id.clone(),
                        name: handler.clone(),
                        kind: "calls".to_string(),
                        line: line_no,
                        column: 0,
                        file_path: norm_path.to_string(),
                    });
                }
            }
        }
    }
}

#[inline(always)]
fn le_u16(slice: &[u8]) -> u16 {
    if slice.len() >= 2 {
        u16::from_le_bytes([slice[0], slice[1]])
    } else {
        0
    }
}

#[inline(always)]
fn le_u32(slice: &[u8]) -> u32 {
    if slice.len() >= 4 {
        u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]])
    } else {
        0
    }
}

pub fn decode_buffers(file_path: &str, out: &buffers::EmitOut) -> Result<FileExtraction, String> {
    if out.meta.len() < buffers::META_SIZE {
        return Err("corrupt extraction meta".into());
    }

    let node_count = le_u32(&out.meta[4..8]) as usize;
    let edge_count = le_u32(&out.meta[8..12]) as usize;
    let ref_count = le_u32(&out.meta[12..16]) as usize;
    let arena = &out.arena;

    let mut nodes = Vec::with_capacity(node_count);
    for i in 0..node_count {
        let off = i * buffers::NODE_ROW_SIZE;
        if off + buffers::NODE_ROW_SIZE > out.nodes.len() {
            break;
        }
        let row = &out.nodes[off..off + buffers::NODE_ROW_SIZE];
        let kind_idx = row[0] as usize;
        let vis_code = row[1];
        let flags_val = le_u16(&row[2..4]);
        let start_line = le_u32(&row[4..8]);
        let end_line = le_u32(&row[8..12]);
        let start_col = le_u32(&row[12..16]);
        let end_col = le_u32(&row[16..20]);

        let name_off = le_u32(&row[20..24]);
        let name_len = le_u32(&row[24..28]);
        let qn_off = le_u32(&row[28..32]);
        let qn_len = le_u32(&row[32..36]);
        let id_off = le_u32(&row[36..40]);
        let id_len = le_u32(&row[40..44]);
        let doc_off = le_u32(&row[44..48]);
        let doc_len = le_u32(&row[48..52]);
        let sig_off = le_u32(&row[52..56]);
        let sig_len = le_u32(&row[56..60]);
        let ret_off = le_u32(&row[76..80]);
        let ret_len = le_u32(&row[80..84]);

        let kind = buffers::NODE_KINDS.get(kind_idx).copied().unwrap_or("symbol").to_string();
        let visibility = match vis_code {
            1 => Some("public".to_string()),
            2 => Some("private".to_string()),
            3 => Some("protected".to_string()),
            4 => Some("internal".to_string()),
            _ => {
                if (flags_val & 0x01) != 0 {
                    if (flags_val & 0x02) != 0 {
                        Some("public".to_string())
                    } else {
                        Some("private".to_string())
                    }
                } else {
                    None
                }
            }
        };

        let name = get_arena_str(arena, name_off, name_len).unwrap_or("").to_string();
        let qualified_name = get_arena_str(arena, qn_off, qn_len).unwrap_or(&name).to_string();
        let id = get_arena_str(arena, id_off, id_len).unwrap_or("").to_string();
        let docstring = get_arena_str(arena, doc_off, doc_len).map(|s| s.to_string());
        let signature = get_arena_str(arena, sig_off, sig_len).map(|s| s.to_string());
        let return_type = get_arena_str(arena, ret_off, ret_len).map(|s| s.to_string());

        nodes.push(SymbolNode {
            id,
            name,
            qualified_name,
            kind,
            visibility,
            file_path: file_path.to_string(),
            start_line,
            end_line,
            start_column: start_col,
            end_column: end_col,
            docstring,
            signature,
            return_type,
        });
    }

    let mut edges = Vec::with_capacity(edge_count);
    for i in 0..edge_count {
        let off = i * buffers::EDGE_ROW_SIZE;
        if off + buffers::EDGE_ROW_SIZE > out.edges.len() {
            break;
        }
        let row = &out.edges[off..off + buffers::EDGE_ROW_SIZE];
        let src_idx = le_u32(&row[0..4]);
        let tgt_idx = le_u32(&row[4..8]);
        let kind_idx = row[8] as usize;
        let line = le_u32(&row[12..16]);
        let col = le_u32(&row[16..20]);

        let src_str_off = le_u32(&row[28..32]);
        let src_str_len = le_u32(&row[32..36]);
        let tgt_str_off = le_u32(&row[36..40]);
        let tgt_str_len = le_u32(&row[40..44]);

        let source_id = if src_idx != buffers::NONE && (src_idx as usize) < nodes.len() {
            nodes[src_idx as usize].id.clone()
        } else {
            get_arena_str(arena, src_str_off, src_str_len).unwrap_or("").to_string()
        };

        let target_id = if tgt_idx != buffers::NONE && (tgt_idx as usize) < nodes.len() {
            nodes[tgt_idx as usize].id.clone()
        } else {
            get_arena_str(arena, tgt_str_off, tgt_str_len).unwrap_or("").to_string()
        };

        let kind = buffers::EDGE_KINDS.get(kind_idx).copied().unwrap_or("references").to_string();

        if !source_id.is_empty() && !target_id.is_empty() {
            edges.push(SymbolEdge {
                source_id,
                target_id,
                kind,
                line: if line != buffers::NONE { Some(line) } else { None },
                column: if col != buffers::NONE { Some(col) } else { None },
            });
        }
    }

    let mut refs = Vec::with_capacity(ref_count);
    for i in 0..ref_count {
        let off = i * buffers::REF_ROW_SIZE;
        if off + buffers::REF_ROW_SIZE > out.refs.len() {
            break;
        }
        let row = &out.refs[off..off + buffers::REF_ROW_SIZE];
        let from_idx = le_u32(&row[0..4]);
        let kind_code = row[4];
        let line = le_u32(&row[8..12]);
        let col = le_u32(&row[12..16]);
        let name_off = le_u32(&row[16..20]);
        let name_len = le_u32(&row[20..24]);
        let from_str_off = le_u32(&row[32..36]);
        let from_str_len = le_u32(&row[36..40]);

        let from_node_id = if from_idx != buffers::NONE && (from_idx as usize) < nodes.len() {
            nodes[from_idx as usize].id.clone()
        } else {
            get_arena_str(arena, from_str_off, from_str_len).unwrap_or("").to_string()
        };

        let name = get_arena_str(arena, name_off, name_len).unwrap_or("").to_string();
        let kind = if kind_code == buffers::FUNCTION_REF_CODE {
            "function_ref".to_string()
        } else {
            buffers::EDGE_KINDS.get(kind_code as usize).copied().unwrap_or("references").to_string()
        };

        if !name.is_empty() && !from_node_id.is_empty() {
            refs.push(SymbolRef {
                from_node_id,
                name,
                kind,
                line,
                column: col,
                file_path: file_path.to_string(),
            });
        }
    }

    Ok(FileExtraction { nodes, edges, refs })
}

