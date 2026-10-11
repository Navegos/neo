use std::path::Path;
use super::paths::ensure_dir;

pub const TOKLESS_OWNERS: &[&str] = &[
    "principles",
    "caveman",
    "ponytail",
    "codegraph",
    "context-mode",
    "projectmem",
];

pub fn section_header_for_owner(owner: &str) -> &'static str {
    match owner {
        "principles" => "## Principles",
        "caveman" => "## Response Style (caveman)",
        "ponytail" => "## Build Discipline (ponytail)",
        "codegraph" => "## Code Index (codegraph)",
        "context-mode" => "## Context Tools (context-mode)",
        "projectmem" => "## Project Memory (projectmem)",
        _ => "",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorProjectRuleSpec {
    pub filename: &'static str,
    pub description: &'static str,
    pub owner: &'static str,
}

pub const CURSOR_PROJECT_RULE_SPECS: &[CursorProjectRuleSpec] = &[
    CursorProjectRuleSpec {
        filename: "principles.mdc",
        description: "This rule guides software work toward simple, correct, and purposeful solutions.",
        owner: "principles",
    },
    CursorProjectRuleSpec {
        filename: "response-style.mdc",
        description: "This rule provides standards for concise, direct, and technically accurate communication.",
        owner: "caveman",
    },
    CursorProjectRuleSpec {
        filename: "build-discipline.mdc",
        description: "This rule guides simple, focused implementations that reuse existing solutions and avoid unnecessary work.",
        owner: "ponytail",
    },
    CursorProjectRuleSpec {
        filename: "code-index.mdc",
        description: "This rule guides codebase exploration through relevant symbols, flows, dependencies, and affected areas.",
        owner: "codegraph",
    },
    CursorProjectRuleSpec {
        filename: "context-tools.mdc",
        description: "This rule guides efficient context collection, analysis, and retrieval while keeping raw data focused.",
        owner: "context-mode",
    },
    CursorProjectRuleSpec {
        filename: "project-memory.mdc",
        description: "This rule guides recording project events and decisions in projectmem.",
        owner: "projectmem",
    },
];

pub fn cursor_project_rule_specs() -> Vec<CursorProjectRuleSpec> {
    CURSOR_PROJECT_RULE_SPECS.to_vec()
}

pub trait AsCursorRuleSpec {
    fn resolve_spec(&self) -> Option<CursorProjectRuleSpec>;
}

impl AsCursorRuleSpec for CursorProjectRuleSpec {
    fn resolve_spec(&self) -> Option<CursorProjectRuleSpec> {
        Some(*self)
    }
}

impl AsCursorRuleSpec for &CursorProjectRuleSpec {
    fn resolve_spec(&self) -> Option<CursorProjectRuleSpec> {
        Some(**self)
    }
}

impl AsCursorRuleSpec for &str {
    fn resolve_spec(&self) -> Option<CursorProjectRuleSpec> {
        CURSOR_PROJECT_RULE_SPECS
            .iter()
            .find(|s| s.owner == *self || s.filename == *self)
            .copied()
    }
}

pub fn cursor_project_rule_content(spec: impl AsCursorRuleSpec) -> String {
    let Some(spec) = spec.resolve_spec() else {
        return String::new();
    };
    format!(
        "---\ndescription: {}\nalwaysApply: true\n---\n{}\n",
        spec.description,
        instruction_section(spec.owner)
    )
}

pub fn install_cursor_project_rules(
    workspace: &str,
    dry_run: bool,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    if workspace.is_empty() {
        return Err("Cursor project rules require non-empty workspace".into());
    }
    let ws_path = Path::new(workspace);
    let meta = std::fs::metadata(ws_path)
        .map_err(|e| format!("stat Cursor workspace: {}", e))?;
    if !meta.is_dir() {
        return Err(format!("Cursor workspace is not a directory: {}", workspace).into());
    }
    if dry_run {
        return Ok(true);
    }
    let rules_dir = ws_path.join(".cursor").join("rules");
    ensure_dir(&rules_dir).map_err(|e| format!("create Cursor rules directory: {}", e))?;
    for spec in CURSOR_PROJECT_RULE_SPECS {
        let path = rules_dir.join(spec.filename);
        if path.exists() {
            continue;
        }
        std::fs::write(&path, cursor_project_rule_content(spec))
            .map_err(|e| format!("write Cursor rule {}: {}", spec.filename, e))?;
    }
    Ok(true)
}

const AGENT_INSTRUCTIONS_TEMPLATE: &str = include_str!("agent_instructions.md");

fn normalized_agent_instructions() -> String {
    AGENT_INSTRUCTIONS_TEMPLATE.replace("\r\n", "\n")
}

pub fn instruction_index_section() -> String {
    let body = normalized_agent_instructions();
    let trimmed = body.trim_end_matches('\n');
    if let Some(idx) = trimmed.find("\n## ") {
        trimmed[..idx].to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn instruction_section(owner: &str) -> String {
    let marker = section_header_for_owner(owner);
    if marker.is_empty() {
        return String::new();
    }
    let body = normalized_agent_instructions();
    let trimmed = body.trim_end_matches('\n');
    let start_pos = match trimmed.find(marker) {
        Some(pos) => pos,
        None => return String::new(),
    };
    let start = if start_pos > 0 {
        trimmed[..start_pos].rfind('\n').map(|p| p + 1).unwrap_or(0)
    } else {
        0
    };
    let rest = &trimmed[start..];
    if let Some(idx) = rest[1..].find("\n## ") {
        rest[..idx + 1].trim_end_matches('\n').to_string()
    } else {
        rest.trim_end_matches('\n').to_string()
    }
}

pub fn section_markers(owner: &str) -> Vec<&'static str> {
    let marker = section_header_for_owner(owner);
    if marker.is_empty() {
        return Vec::new();
    }
    let legacy: &[&str] = match owner {
        "principles" => &[
            "## 1. Principles",
            "## Principles (craft) →",
            "## Principles (craft)",
        ],
        "caveman" => &[
            "## 2. Response Style",
            "## Response Style",
            "## Style",
            "## Caveman Style",
            "## Caveman",
            "## Voice (caveman)",
            "## Response Style (caveman)",
        ],
        "ponytail" => &[
            "## 3. Build Discipline",
            "## Build Discipline",
            "## Build Less",
            "## Ponytail",
            "## Ponytail: Build Less",
            "## Reuse Ladder (ponytail)",
            "## Lazy Ladder (ponytail)",
            "## Build Discipline (ponytail)",
        ],
        "codegraph" => &[
            "## 4. Code Search",
            "## Codegraph",
            "## Codegraph — MUST USE FOR CODE",
            "## Code Index (codegraph)",
        ],
        "context-mode" => &[
            "## 5. Context Control",
            "## Context Tools",
            "## Context Tools — MUST USE FOR DATA",
            "## Context Tools (context-mode)",
        ],
        _ => &[],
    };
    let mut out = vec![marker];
    out.extend_from_slice(legacy);
    out
}

pub fn section_present(body: &str, owner: &str) -> bool {
    for m in section_markers(owner) {
        if body.contains(m) {
            return true;
        }
    }
    false
}

pub fn tokless_agent_body(owners: &[&str]) -> String {
    let mut out = String::new();
    if owners.len() >= 2 {
        out.push_str(&instruction_index_section());
        out.push('\n');
    }
    if !owners.is_empty() {
        out.push_str(&instruction_section("principles"));
        out.push_str("\n\n");
    }
    for &sec in &["caveman", "ponytail", "codegraph", "context-mode", "projectmem"] {
        if owners.contains(&sec) {
            out.push_str(&instruction_section(sec));
            out.push_str("\n\n");
        }
    }
    out.trim_end_matches('\n').to_string()
}

pub fn tokenize_body(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for &owner in TOKLESS_OWNERS {
        if section_present(body, owner) {
            out.push(owner.to_string());
        }
    }
    out
}
