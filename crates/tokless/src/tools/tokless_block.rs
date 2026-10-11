use std::collections::HashMap;
use std::fs;
use std::path::Path;
use regex::Regex;

use crate::agents::{omp_agent_dir_resolved, pi_agent_dir_resolved};
use crate::util::instructions::{
    instruction_index_section, instruction_section, section_header_for_owner, section_present,
    TOKLESS_OWNERS,
};
use crate::util::jsonc::{stringify_json, try_parse_jsonc, Value};
use crate::util::paths::{
    antigravity_paths_resolved, claude_code_paths, cline_paths_resolved, codex_paths_resolved,
    copilot_paths_resolved, ensure_dir, exists, home, kilo_paths_resolved,
    opencode_paths_resolved, read_file_safe, write_file_atomic,
};

pub fn instruction_path(agent: &str) -> String {
    match agent {
        "claude" => claude_code_paths().instructions,
        "opencode" => opencode_paths_resolved().instructions,
        "codex" => codex_paths_resolved().instructions,
        "antigravity" => antigravity_paths_resolved().instructions,
        "copilot" => copilot_paths_resolved().instructions,
        "droid" => Path::new(&home()).join(".factory").join("AGENTS.md").to_string_lossy().to_string(),
        "pi" => Path::new(&pi_agent_dir_resolved()).join("AGENTS.md").to_string_lossy().to_string(),
        "omp" => Path::new(&omp_agent_dir_resolved()).join("AGENTS.md").to_string_lossy().to_string(),
        "grok" => {
            if let Ok(custom) = std::env::var("GROK_HOME") {
                if !custom.is_empty() {
                    return Path::new(&custom).join("GROK.md").to_string_lossy().to_string();
                }
            }
            Path::new(&home()).join(".grok").join("GROK.md").to_string_lossy().to_string()
        }
        "kilo" => kilo_instructions_path(),
        "cline" => cline_paths_resolved().instructions,
        _ => String::new(),
    }
}

pub fn kilo_instructions_path() -> String {
    Path::new(&kilo_paths_resolved().dir).join("AGENTS.md").to_string_lossy().to_string()
}

/// Cleans up legacy per-tool instruction files/config entries from older Tokless versions.
pub fn configure_instruction_conflicts(agent: &str) {
    match agent {
        "claude" => {
            let p = claude_code_paths();
            for legacy in ["CAVEMAN.md", "PONYTAIL.md", "PRINCIPLES.md", "RTK.md", "TOKLESS.md"] {
                let f = Path::new(&p.dir).join(legacy);
                if f.exists() {
                    let _ = fs::remove_file(f);
                }
            }
        }
        "opencode" => {
            let p = opencode_paths_resolved();
            for legacy in ["CAVEMAN.md", "PONYTAIL.md", "PRINCIPLES.md"] {
                let f = Path::new(&p.dir).join(legacy);
                if f.exists() {
                    let _ = fs::remove_file(f);
                }
            }
            if let Some(raw) = read_file_safe(&p.config) {
                if let Some(Value::Object(mut cfg)) = try_parse_jsonc(&raw) {
                    let mut changed = false;
                    if let Some(Value::Array(arr)) = cfg.get_mut("instructions") {
                        let before = arr.len();
                        arr.retain(|item| {
                            if let Value::String(s) = item {
                                let lower = s.to_lowercase();
                                !(lower.ends_with("caveman.md")
                                    || lower.ends_with("ponytail.md")
                                    || lower.ends_with("principles.md"))
                            } else {
                                true
                            }
                        });
                        if arr.len() != before {
                            changed = true;
                        }
                        if arr.is_empty() {
                            cfg.remove("instructions");
                        }
                    }
                    if changed {
                        let _ = write_file_atomic(&p.config, &stringify_json(&Value::Object(cfg)));
                    }
                }
            }
        }
        "kilo" => {
            let p = kilo_paths_resolved();
            if let Some(raw) = read_file_safe(&p.config) {
                if let Some(Value::Object(mut cfg)) = try_parse_jsonc(&raw) {
                    if let Some(Value::Array(arr)) = cfg.get_mut("instructions") {
                        let target = kilo_instructions_path();
                        let before = arr.len();
                        arr.retain(|item| {
                            if let Value::String(s) = item {
                                s != &target && !s.ends_with("AGENTS.md")
                            } else {
                                true
                            }
                        });
                        if arr.len() != before {
                            if arr.is_empty() {
                                cfg.remove("instructions");
                            }
                            let _ = write_file_atomic(&p.config, &stringify_json(&Value::Object(cfg)));
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

/// Removes all legacy markers, comments, `@RTK.md` references, and old tokless headings.
pub fn strip_legacy(content: &str) -> String {
    let mut s = content.replace("\r\n", "\n");

    let block_patterns = [
        r"(?is)<!--\s*tokless:[a-z0-9_-]+:start\s*-->.*?<!--\s*tokless:[a-z0-9_-]+:end\s*-->\n?",
        r"(?is)<!--\s*tokless:start\s*-->.*?<!--\s*tokless:end\s*-->\n?",
        r"(?is)<!--\s*caveman:start\s*-->.*?<!--\s*caveman:end\s*-->\n?",
        r"(?is)<!--\s*ponytail:start\s*-->.*?<!--\s*ponytail:end\s*-->\n?",
        r"(?is)<!--\s*principles:start\s*-->.*?<!--\s*principles:end\s*-->\n?",
        r"(?is)<!--\s*codegraph:start\s*-->.*?<!--\s*codegraph:end\s*-->\n?",
        r"(?is)<!--\s*context-mode:start\s*-->.*?<!--\s*context-mode:end\s*-->\n?",
        r"(?is)<!--\s*projectmem:start\s*-->.*?<!--\s*projectmem:end\s*-->\n?",
        r"(?is)<!--\s*rtk:start\s*-->.*?<!--\s*rtk:end\s*-->\n?",
    ];
    for pat in block_patterns {
        if let Ok(re) = Regex::new(pat) {
            s = re.replace_all(&s, "").to_string();
        }
    }

    let line_patterns = [
        r"(?m)^@RTK\.md\s*\n?",
        r"(?m)^@CAVEMAN\.md\s*\n?",
        r"(?m)^@PONYTAIL\.md\s*\n?",
        r"(?m)^@PRINCIPLES\.md\s*\n?",
        r"(?m)^@TOKLESS\.md\s*\n?",
    ];
    for pat in line_patterns {
        if let Ok(re) = Regex::new(pat) {
            s = re.replace_all(&s, "").to_string();
        }
    }

    s = strip_legacy_heading(&s, "# Agent Instructions", 1);
    for owner in TOKLESS_OWNERS {
        let h = section_header_for_owner(owner);
        if !h.is_empty() {
            let level = if h.starts_with("## ") { 2 } else { 1 };
            s = strip_legacy_heading(&s, h, level);
        }
    }

    // Legacy alternative headings from older versions
    for (old_h, level) in [
        ("# RTK", 1),
        ("# Command output", 1),
        ("## Caveman", 2),
        ("## Ponytail", 2),
        ("## CodeGraph", 2),
        ("## Context Mode", 2),
        ("## ProjectMem", 2),
    ] {
        s = strip_legacy_heading(&s, old_h, level);
    }

    let multi_nl = Regex::new(r"\n{3,}").unwrap();
    multi_nl.replace_all(&s, "\n\n").trim().to_string()
}

pub fn strip_legacy_heading(content: &str, heading: &str, level: usize) -> String {
    let lines: Vec<&str> = content.split('\n').collect();
    let mut out = Vec::with_capacity(lines.len());
    let mut skipping = false;

    for line in lines {
        let trimmed = line.trim_end();
        if !skipping {
            if trimmed == heading {
                skipping = true;
                continue;
            }
            out.push(line);
        } else {
            if is_heading_at_or_above(trimmed, level) {
                skipping = false;
                out.push(line);
            }
        }
    }

    out.join("\n")
}

fn is_heading_at_or_above(line: &str, level: usize) -> bool {
    if !line.starts_with('#') {
        return false;
    }
    let hashes = line.bytes().take_while(|&b| b == b'#').count();
    if hashes == 0 || hashes > level {
        return false;
    }
    line.as_bytes().get(hashes) == Some(&b' ')
}

pub fn write_owner(agent: &str, owner: &str) -> bool {
    configure_instruction_conflicts(agent);
    let path = instruction_path(agent);
    if path.is_empty() {
        return false;
    }
    write_owner_at_path(&path, owner)
}

pub fn remove_owner(agent: &str, owner: &str) -> bool {
    remove_owner_safe(agent, owner).is_ok()
}

pub fn remove_owner_safe(agent: &str, owner: &str) -> Result<(), String> {
    configure_instruction_conflicts(agent);
    let path = instruction_path(agent);
    if path.is_empty() {
        return Err(format!("unknown instruction path for agent {}", agent));
    }
    remove_owner_at_path_safe(&path, owner)
}

pub fn has_owner(agent: &str, owner: &str) -> bool {
    let path = instruction_path(agent);
    if path.is_empty() {
        return false;
    }
    has_owner_at_path(&path, owner)
}

pub fn write_owner_at_path(path: &str, owner: &str) -> bool {
    if path.is_empty() || instruction_section(owner).is_empty() {
        return false;
    }
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            let _ = ensure_dir(parent);
        }
    }
    let existing = read_file_safe(path).unwrap_or_default();
    let mut owners = extract_present_owners(&existing);
    if !owners.iter().any(|o| o == owner) {
        owners.push(owner.to_string());
    }
    let cleaned_user = strip_legacy(&existing);
    let rendered = render_combined_instructions(&cleaned_user, &owners);
    if write_file_atomic(path, &rendered).is_err() {
        return false;
    }
    has_owner_at_path(path, owner)
}

pub fn has_owner_at_path(path: &str, owner: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    let content = match read_file_safe(path) {
        Some(c) => c,
        None => return false,
    };
    if !section_present(&content, owner) {
        return false;
    }
    if owner != "rtk" {
        let idx = instruction_index_section();
        let norm_c = content.replace("\r\n", "\n");
        let norm_i = idx.replace("\r\n", "\n").trim().to_string();
        if !norm_c.contains(&norm_i) {
            return false;
        }
    }
    true
}

pub fn remove_owner_at_path(path: &str, owner: &str) -> bool {
    remove_owner_at_path_safe(path, owner).is_ok()
}

pub fn remove_owner_at_path_safe(path: &str, owner: &str) -> Result<(), String> {
    if path.is_empty() || !exists(path) {
        return Ok(());
    }
    let existing = match read_file_safe(path) {
        Some(c) => c,
        None => return Err(format!("failed to read {}", path)),
    };
    let mut owners = extract_present_owners(&existing);
    owners.retain(|o| o != owner);
    let cleaned_user = strip_legacy(&existing);
    let rendered = render_combined_instructions(&cleaned_user, &owners);
    if rendered.trim().is_empty() {
        let _ = fs::remove_file(path);
        return Ok(());
    }
    write_file_atomic(path, &rendered).map_err(|e| e.to_string())
}

pub fn kilo_write_owner(owner: &str) -> bool {
    configure_instruction_conflicts("kilo");
    write_owner_at_path(&kilo_instructions_path(), owner)
}

pub fn kilo_remove_owner(owner: &str) -> bool {
    configure_instruction_conflicts("kilo");
    remove_owner_at_path(&kilo_instructions_path(), owner)
}

pub fn kilo_has_owner(owner: &str) -> bool {
    has_owner_at_path(&kilo_instructions_path(), owner)
}

fn extract_present_owners(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for &owner in TOKLESS_OWNERS {
        if section_present(content, owner) {
            out.push(owner.to_string());
        }
    }
    out
}

fn render_combined_instructions(user_content: &str, owners: &[String]) -> String {
    let order_map: HashMap<&str, usize> = TOKLESS_OWNERS
        .iter()
        .enumerate()
        .map(|(i, &o)| (o, i))
        .collect();

    let mut sorted: Vec<String> = owners.to_vec();
    sorted.sort_by_key(|o| order_map.get(o.as_str()).copied().unwrap_or(999));
    sorted.dedup();

    let mut blocks: Vec<String> = Vec::new();
    let has_non_rtk = sorted.iter().any(|o| o != "rtk");

    for o in &sorted {
        if o == "rtk" {
            let sec = instruction_section("rtk").trim().to_string();
            if !sec.is_empty() {
                blocks.push(sec);
            }
        }
    }

    if has_non_rtk {
        let idx = instruction_index_section().trim().to_string();
        if !idx.is_empty() {
            blocks.push(idx);
        }
        for o in &sorted {
            if o != "rtk" {
                let sec = instruction_section(o).trim().to_string();
                if !sec.is_empty() {
                    blocks.push(sec);
                }
            }
        }
    }

    let tokless_part = blocks.join("\n\n");
    let user_trimmed = user_content.trim();
    if user_trimmed.is_empty() {
        if tokless_part.is_empty() {
            String::new()
        } else {
            format!("{}\n", tokless_part)
        }
    } else if tokless_part.is_empty() {
        format!("{}\n", user_trimmed)
    } else {
        format!("{}\n\n{}\n", user_trimmed, tokless_part)
    }
}
