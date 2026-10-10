use std::path::Path;
use crate::util::paths::*;
use crate::util::instructions::{section_present, tokless_agent_body, tokenize_body};

pub fn instruction_path(agent: &str) -> String {
    match agent {
        "claude" => claude_code_paths().instructions,
        "opencode" => opencode_paths_resolved().instructions,
        "codex" => codex_paths_resolved().instructions,
        "antigravity" => antigravity_paths_resolved().instructions,
        "copilot" => copilot_paths_resolved().instructions,
        "droid" => Path::new(&home()).join(".factory").join("AGENTS.md").to_string_lossy().to_string(),
        "pi" => Path::new(&home()).join(".pi").join("AGENTS.md").to_string_lossy().to_string(),
        "omp" => Path::new(&home()).join(".omp").join("AGENTS.md").to_string_lossy().to_string(),
        "grok" => Path::new(&home()).join(".grok").join("AGENTS.md").to_string_lossy().to_string(),
        "kilo" => Path::new(&kilo_paths_resolved().dir).join("AGENTS.md").to_string_lossy().to_string(),
        "cline" => cline_paths_resolved().instructions,
        _ => String::new(),
    }
}

pub fn has_owner(agent: &str, owner: &str) -> bool {
    let path = instruction_path(agent);
    if path.is_empty() {
        return false;
    }
    if let Some(content) = read_file_safe(&path) {
        section_present(&content, owner)
    } else {
        false
    }
}

pub fn write_owner(agent: &str, owner: &str) {
    let path = instruction_path(agent);
    if path.is_empty() {
        return;
    }
    let existing = read_file_safe(&path).unwrap_or_default();
    let mut owners = tokenize_body(&existing);
    if !owners.contains(&owner.to_string()) {
        owners.push(owner.to_string());
    }
    let owners_refs: Vec<&str> = owners.iter().map(|s| s.as_str()).collect();
    let body = tokless_agent_body(&owners_refs);
    let _ = write_file(&path, &body);
}

pub fn remove_owner(agent: &str, owner: &str) {
    let path = instruction_path(agent);
    if path.is_empty() {
        return;
    }
    let existing = match read_file_safe(&path) {
        Some(e) => e,
        None => return,
    };
    let mut owners = tokenize_body(&existing);
    owners.retain(|o| o != owner);
    let owners_refs: Vec<&str> = owners.iter().map(|s| s.as_str()).collect();
    let body = tokless_agent_body(&owners_refs);
    let _ = write_file(&path, &body);
}

