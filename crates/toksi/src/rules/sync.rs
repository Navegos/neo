// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rules/sync.rs
// created: 2026-11-05
// lastModified: 2026-11-14

//! Universal rules, skills, hooks, MCP & config synchronizer for user-configured agents.
//!
//! Enforces:
//! - Strictly configured/selected agent filtering across 18+ built-in harnesses & OpenAgents conventions.
//! - Granular component toggles (`hooks`, `mcp`, `skills`, `rules`, `instructions`, `configs`).
//! - First workspace root resolution (`.git` root via `find_project_dir`) + nested subfolder merge.
//! - Universal forward-slash (`/`) paths via `normalize_path()`.
//! - Zero external network fetches at runtime; all rules/skills/commands are embedded compile-time assets.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::commands::doctor::{consolidate_nested_agent_dirs, consolidate_nested_toksi_dirs};
use crate::rules::{
    patch_agents_md, patch_agents_rules_md, patch_claude_md, patch_cline_rules, patch_codex_md,
    patch_copilot_md, patch_cursor_rules, patch_cursorrules, patch_gemini_commands,
    patch_gemini_md, patch_markdown_file, patch_skills_dir, RuleKind, RulesError,
};
use crate::util::paths::{home, normalize_path};

pub type DynError = Box<dyn std::error::Error + Send + Sync>;

/// Granular toggles controlling which artifacts the synchronizer writes or repairs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncComponents {
    pub hooks: bool,
    pub mcp: bool,
    pub skills: bool,
    pub rules: bool,
    pub instructions: bool,
    pub configs: bool,
}

impl Default for SyncComponents {
    fn default() -> Self {
        Self::all()
    }
}

impl SyncComponents {
    pub const fn all() -> Self {
        Self {
            hooks: true,
            mcp: true,
            skills: true,
            rules: true,
            instructions: true,
            configs: true,
        }
    }

    pub const fn none() -> Self {
        Self {
            hooks: false,
            mcp: false,
            skills: false,
            rules: false,
            instructions: false,
            configs: false,
        }
    }

    /// Parses comma-separated components (`hooks,mcp,skills,rules,instructions,configs` or `all`).
    pub fn from_csv(spec: &str) -> Result<Self, RulesError> {
        let trimmed = spec.trim();
        if trimmed.eq_ignore_ascii_case("all") {
            return Ok(Self::all());
        }
        let mut out = Self::none();
        for part in trimmed.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            match part.to_ascii_lowercase().as_str() {
                "hooks" | "hook" => out.hooks = true,
                "mcp" => out.mcp = true,
                "skills" | "skill" => out.skills = true,
                "rules" | "rule" => out.rules = true,
                "instructions" | "instruction" => out.instructions = true,
                "configs" | "config" => out.configs = true,
                other => return Err(RulesError::InvalidComponent(other.to_string())),
            }
        }
        Ok(out)
    }
}

/// Target scope for synchronization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncScope {
    UserGlobal,
    Workspace,
}

/// Controls which agents, components, and placement policies apply during sync.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncOptions {
    /// Explicit subset of canonical agent IDs or directory aliases. When `None`, only configured/detected agents are synced.
    pub selected_agents: Option<Vec<String>>,
    /// Component-level gates (`hooks`, `mcp`, `skills`, `rules`, `instructions`, `configs`).
    pub components: SyncComponents,
    /// Policy #5 guard: workspace repository installs happen only when opted in by the user/caller.
    pub allow_workspace_install: bool,
    /// Auto-merge nested `<repo>/**/.toksi` and `.<agent>` dirs into workspace root before syncing.
    pub auto_merge_nested: bool,
    /// Managed MCP tool names to wire into `skills/<tool>`.
    pub mcp_tools: Vec<String>,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            selected_agents: None,
            components: SyncComponents::all(),
            allow_workspace_install: true,
            auto_merge_nested: true,
            mcp_tools: MANAGED_MCP_TOOLS.iter().map(|s| (*s).to_string()).collect(),
        }
    }
}

impl SyncOptions {
    /// Targets only the given selected agent IDs with all components enabled.
    pub fn for_agents<I, S>(agents: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            selected_agents: Some(agents.into_iter().map(Into::into).collect()),
            ..Self::default()
        }
    }

    pub fn with_components(mut self, components: SyncComponents) -> Self {
        self.components = components;
        self
    }

    pub fn with_workspace_opt_in(mut self, allow: bool) -> Self {
        self.allow_workspace_install = allow;
        self
    }
}

/// Synchronization outcome report.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SyncReport {
    pub workspace: PathBuf,
    pub updated_files: Vec<PathBuf>,
    pub up_to_date_files: Vec<PathBuf>,
    pub synced_agents: Vec<String>,
    pub merged_dirs: usize,
    pub errors: Vec<String>,
}

impl SyncReport {
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn total_touched(&self) -> usize {
        self.updated_files.len() + self.up_to_date_files.len()
    }

    /// Record a target path into the report using normalized forward-slash representation.
    pub fn record_updated(&mut self, path: impl AsRef<Path>) {
        let norm = normalize_path(&path.as_ref().to_string_lossy());
        let pb = PathBuf::from(norm);
        if !self.updated_files.contains(&pb) {
            self.updated_files.push(pb);
        }
    }

    /// Record an up-to-date target path into the report.
    pub fn record_up_to_date(&mut self, path: impl AsRef<Path>) {
        let norm = normalize_path(&path.as_ref().to_string_lossy());
        let pb = PathBuf::from(norm);
        if !self.up_to_date_files.contains(&pb) {
            self.up_to_date_files.push(pb);
        }
    }

    /// Record an error message into the report.
    pub fn record_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }
}

/// Known tool identifiers that get wired into `skills/<tool>`.
pub const MANAGED_MCP_TOOLS: &[&str] = &[
    "codegraph",
    "context",
    "projectmem",
    "rtk",
    "icm",
    "aimemory",
];

/// All supported agent configuration directories across `18+` harnesses & OpenAgents conventions.
pub const SUPPORTED_AGENT_DIRS: &[&str] = &[
    ".agents",
    ".gemini",
    ".copilot",
    ".github",
    ".claude",
    ".codex",
    ".cursor",
    ".grok",
    ".opencode",
    ".opencode2",
    ".cline",
    ".droid",
    ".factory",
    ".kilo",
    ".omp",
    ".pi",
    ".amp",
    ".kimi",
    ".zcode",
    ".dsh",
];

/// Data-driven harness descriptor achieving `agents-structures/` parity without duplicate code.
#[derive(Debug, Clone, Copy)]
struct HarnessSpec {
    id: &'static str,
    primary_dir: &'static str,
    secondary_dir: Option<&'static str>,
    root_instruction_file: Option<&'static str>,
    mcp_config_rel: Option<&'static str>,
    hook_agent_name: Option<&'static str>,
    settings_rel: Option<&'static str>,
}

const HARNESS_SPECS: &[HarnessSpec] = &[
    HarnessSpec {
        id: "openagents",
        primary_dir: ".agents",
        secondary_dir: None,
        root_instruction_file: Some("AGENTS.md"),
        mcp_config_rel: Some(".agents/mcp.json"),
        hook_agent_name: None,
        settings_rel: None,
    },
    HarnessSpec {
        id: "antigravity",
        primary_dir: ".gemini",
        secondary_dir: None,
        root_instruction_file: Some("GEMINI.md"),
        mcp_config_rel: Some(".gemini/mcp_config.json"),
        hook_agent_name: Some("antigravity"),
        settings_rel: Some(".gemini/settings.json"),
    },
    HarnessSpec {
        id: "copilot",
        primary_dir: ".copilot",
        secondary_dir: Some(".github"),
        root_instruction_file: None,
        mcp_config_rel: Some(".copilot/mcp-config.json"),
        hook_agent_name: Some("copilot"),
        settings_rel: Some(".copilot/settings.json"),
    },
    HarnessSpec {
        id: "claude",
        primary_dir: ".claude",
        secondary_dir: None,
        root_instruction_file: Some("CLAUDE.md"),
        mcp_config_rel: Some(".claude/.mcp.json"),
        hook_agent_name: Some("claude"),
        settings_rel: Some(".claude/settings.json"),
    },
    HarnessSpec {
        id: "codex",
        primary_dir: ".codex",
        secondary_dir: None,
        root_instruction_file: Some("CODEX.md"),
        mcp_config_rel: None,
        hook_agent_name: None,
        settings_rel: None,
    },
    HarnessSpec {
        id: "cursor",
        primary_dir: ".cursor",
        secondary_dir: None,
        root_instruction_file: Some(".cursorrules"),
        mcp_config_rel: Some(".cursor/mcp.json"),
        hook_agent_name: Some("cursor"),
        settings_rel: Some(".cursor/cli-config.json"),
    },
    HarnessSpec {
        id: "grok",
        primary_dir: ".grok",
        secondary_dir: None,
        root_instruction_file: None,
        mcp_config_rel: None,
        hook_agent_name: Some("grok"),
        settings_rel: None,
    },
    HarnessSpec {
        id: "opencode",
        primary_dir: ".opencode",
        secondary_dir: None,
        root_instruction_file: None,
        mcp_config_rel: None,
        hook_agent_name: None,
        settings_rel: None,
    },
    HarnessSpec {
        id: "opencode2",
        primary_dir: ".opencode2",
        secondary_dir: None,
        root_instruction_file: None,
        mcp_config_rel: None,
        hook_agent_name: None,
        settings_rel: None,
    },
    HarnessSpec {
        id: "cline",
        primary_dir: ".cline",
        secondary_dir: None,
        root_instruction_file: Some(".clinerules"),
        mcp_config_rel: None,
        hook_agent_name: None,
        settings_rel: None,
    },
    HarnessSpec {
        id: "droid",
        primary_dir: ".droid",
        secondary_dir: Some(".factory"),
        root_instruction_file: None,
        mcp_config_rel: None,
        hook_agent_name: None,
        settings_rel: None,
    },
    HarnessSpec {
        id: "kilo",
        primary_dir: ".kilo",
        secondary_dir: None,
        root_instruction_file: None,
        mcp_config_rel: None,
        hook_agent_name: None,
        settings_rel: None,
    },
    HarnessSpec {
        id: "omp",
        primary_dir: ".omp",
        secondary_dir: None,
        root_instruction_file: None,
        mcp_config_rel: None,
        hook_agent_name: None,
        settings_rel: None,
    },
    HarnessSpec {
        id: "pi",
        primary_dir: ".pi",
        secondary_dir: None,
        root_instruction_file: None,
        mcp_config_rel: None,
        hook_agent_name: None,
        settings_rel: None,
    },
    HarnessSpec {
        id: "amp",
        primary_dir: ".amp",
        secondary_dir: None,
        root_instruction_file: None,
        mcp_config_rel: None,
        hook_agent_name: None,
        settings_rel: None,
    },
    HarnessSpec {
        id: "kimi",
        primary_dir: ".kimi",
        secondary_dir: None,
        root_instruction_file: None,
        mcp_config_rel: None,
        hook_agent_name: None,
        settings_rel: None,
    },
    HarnessSpec {
        id: "zcode",
        primary_dir: ".zcode",
        secondary_dir: None,
        root_instruction_file: None,
        mcp_config_rel: None,
        hook_agent_name: None,
        settings_rel: None,
    },
    HarnessSpec {
        id: "dsh",
        primary_dir: ".dsh",
        secondary_dir: None,
        root_instruction_file: None,
        mcp_config_rel: None,
        hook_agent_name: None,
        settings_rel: None,
    },
];

/// Normalizes agent IDs, variants (`antigravity-cli`, `copilot-vscode`), and dir names (`.gemini`) to canonical harness IDs.
pub fn normalize_harness_id(raw: &str) -> Option<&'static str> {
    let key = raw.trim().trim_start_matches('.').to_ascii_lowercase();
    match key.as_str() {
        "openagents" | "agents" => Some("openagents"),
        "antigravity" | "antigravity-cli" | "antigravity-ide" | "gemini" => Some("antigravity"),
        "copilot" | "copilot-cli" | "copilot-vscode" | "vscode" | "github" => Some("copilot"),
        "claude" | "claude-code" => Some("claude"),
        "codex" => Some("codex"),
        "cursor" => Some("cursor"),
        "grok" => Some("grok"),
        "opencode" => Some("opencode"),
        "opencode2" => Some("opencode2"),
        "cline" => Some("cline"),
        "droid" | "factory" => Some("droid"),
        "kilo" => Some("kilo"),
        "omp" => Some("omp"),
        "pi" => Some("pi"),
        "amp" => Some("amp"),
        "kimi" => Some("kimi"),
        "zcode" => Some("zcode"),
        "dsh" => Some("dsh"),
        _ => None,
    }
}

fn find_harness(id: &str) -> Option<&'static HarnessSpec> {
    let canonical = normalize_harness_id(id)?;
    HARNESS_SPECS.iter().find(|h| h.id == canonical)
}

/// Resolves the first workspace root directory (walking ancestors strictly to `.git` root).
/// Falls back to normalized `workspace` itself if no `.git` ancestor is found before home/root.
pub fn resolve_workspace_root(workspace: impl AsRef<Path>) -> PathBuf {
    let raw = workspace.as_ref().to_path_buf();
    let current = fs::canonicalize(&raw)
        .or_else(|_| std::path::absolute(&raw))
        .unwrap_or(raw);
    let home_norm = normalize_path(&home());
    let mut iter = current.clone();
    loop {
        if iter.join(".git").exists() {
            return PathBuf::from(normalize_path(&iter.to_string_lossy()));
        }
        if !home_norm.is_empty() && normalize_path(&iter.to_string_lossy()) == home_norm {
            break;
        }
        if !iter.pop() {
            break;
        }
    }
    PathBuf::from(normalize_path(&current.to_string_lossy()))
}

/// Helper determining if a file name matches convention targets:
/// `rules*.md`, `skills*.md`, `instructions*.md`, `agents*.md`.
pub fn matches_sync_target_filename(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if !lower.ends_with(".md") && lower != ".cursorrules" && lower != ".clinerules" {
        return false;
    }

    lower.starts_with("rules")
        || lower.starts_with("skills")
        || lower.starts_with("instructions")
        || lower.contains("instructions.md")
        || lower.starts_with("agents")
        || lower.contains("agents.md")
        || lower == "gemini.md"
        || lower == "claude.md"
        || lower == "codex.md"
        || lower == "system-prompt.md"
        || lower == ".cursorrules"
        || lower == ".clinerules"
}

fn parse_agents_from_toml(path: &Path, out: &mut BTreeSet<&'static str>) {
    let Ok(raw) = fs::read_to_string(path) else {
        return;
    };
    let Ok(val) = raw.parse::<toml::Value>() else {
        return;
    };
    for key in ["agents", "selected_agents", "configured_agents"] {
        if let Some(arr) = val.get(key).and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    if let Some(id) = normalize_harness_id(s) {
                        out.insert(id);
                    }
                }
            }
        }
    }
    if let Some(tbl) = val.get("agents").and_then(|v| v.as_table()) {
        for (k, v) in tbl {
            let enabled = v.as_bool().unwrap_or(true);
            if enabled {
                if let Some(id) = normalize_harness_id(k) {
                    out.insert(id);
                }
            }
        }
    }
}

/// Resolves user-configured or already-present agent harnesses under `base_dir` (and optional `home_dir`).
pub fn resolve_configured_agents(base_dir: &Path, home_dir: Option<&Path>) -> Vec<String> {
    let mut ids: BTreeSet<&'static str> = BTreeSet::new();

    parse_agents_from_toml(&base_dir.join(".toksi").join("config.toml"), &mut ids);
    if let Some(h) = home_dir {
        parse_agents_from_toml(&h.join(".toksi").join("config.toml"), &mut ids);
    }

    let mut scan_dir = |dir: &Path| {
        for spec in HARNESS_SPECS {
            if dir.join(spec.primary_dir).exists()
                || spec.secondary_dir.is_some_and(|s| dir.join(s).exists())
                || spec
                    .root_instruction_file
                    .is_some_and(|f| dir.join(f).exists())
            {
                ids.insert(spec.id);
            }
        }
        if dir.join(".github").join("copilot-instructions.md").exists() {
            ids.insert("copilot");
        }
    };

    scan_dir(base_dir);
    if let Some(h) = home_dir {
        scan_dir(h);
    }

    ids.into_iter().map(str::to_string).collect()
}

fn resolve_target_harnesses(
    base_dir: &Path,
    home_for_detection: Option<&Path>,
    opts: &SyncOptions,
) -> Result<Vec<&'static HarnessSpec>, RulesError> {
    if let Some(explicit) = &opts.selected_agents {
        let mut seen = BTreeSet::new();
        let mut specs = Vec::new();
        for raw in explicit {
            let canonical = normalize_harness_id(raw)
                .ok_or_else(|| RulesError::InvalidAgent(raw.clone()))?;
            if seen.insert(canonical) {
                if let Some(spec) = find_harness(canonical) {
                    specs.push(spec);
                }
            }
        }
        return Ok(specs);
    }

    let configured = resolve_configured_agents(base_dir, home_for_detection);
    Ok(configured
        .into_iter()
        .filter_map(|id| find_harness(&id))
        .collect())
}

/// Applies in-tree rules patch to a single markdown target file and logs into the report.
pub fn sync_markdown_target(report: &mut SyncReport, file_path: &Path, rules: &[RuleKind]) {
    match patch_markdown_file(file_path, rules) {
        Ok(true) => report.record_updated(file_path),
        Ok(false) => report.record_up_to_date(file_path),
        Err(e) => report.record_error(format!("{}: {}", normalize_path(&file_path.to_string_lossy()), e)),
    }
}

fn ensure_json_file_merged(
    report: &mut SyncReport,
    file_path: &Path,
    defaults: &Value,
) {
    if let Some(parent) = file_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            report.record_error(format!("{}: {}", normalize_path(&parent.to_string_lossy()), e));
            return;
        }
    }

    let mut current = if file_path.exists() {
        match fs::read_to_string(file_path) {
            Ok(raw) => serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| json!({})),
            Err(e) => {
                report.record_error(format!("{}: {}", normalize_path(&file_path.to_string_lossy()), e));
                return;
            }
        }
    } else {
        json!({})
    };

    let before = current.clone();
    merge_json_missing(&mut current, defaults);

    if file_path.exists() && current == before {
        report.record_up_to_date(file_path);
        return;
    }

    match serde_json::to_string_pretty(&current) {
        Ok(rendered) => {
            let with_newline = format!("{}\n", rendered);
            if let Err(e) = fs::write(file_path, with_newline) {
                report.record_error(format!("{}: {}", normalize_path(&file_path.to_string_lossy()), e));
            } else {
                report.record_updated(file_path);
            }
        }
        Err(e) => report.record_error(format!("{}: {}", normalize_path(&file_path.to_string_lossy()), e)),
    }
}

fn merge_json_missing(dst: &mut Value, defaults: &Value) {
    match (dst, defaults) {
        (Value::Object(d_map), Value::Object(s_map)) => {
            for (k, v) in s_map {
                match d_map.get_mut(k) {
                    Some(existing) => merge_json_missing(existing, v),
                    None => {
                        d_map.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        (dst_val, default_val) if dst_val.is_null() => {
            *dst_val = default_val.clone();
        }
        _ => {}
    }
}

fn ensure_file_if_missing(report: &mut SyncReport, file_path: &Path, content: &str) {
    if let Some(parent) = file_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            report.record_error(format!("{}: {}", normalize_path(&parent.to_string_lossy()), e));
            return;
        }
    }
    if file_path.exists() {
        report.record_up_to_date(file_path);
        return;
    }
    if let Err(e) = fs::write(file_path, content) {
        report.record_error(format!("{}: {}", normalize_path(&file_path.to_string_lossy()), e));
    } else {
        report.record_updated(file_path);
    }
}

fn default_mcp_servers_json(tools: &[String]) -> Value {
    let mut servers = serde_json::Map::new();
    for t in tools {
        let (key, subcmd) = match t.as_str() {
            "codegraph" => ("codegraph", "graph"),
            "context" | "context-mode" => ("context", "ctx"),
            "projectmem" => ("projectmem", "mem"),
            "icm" => ("icm", "icm"),
            "aimemory" | "ai-memory" => ("aimemory", "aimem"),
            _ => continue,
        };
        servers.insert(
            key.to_string(),
            json!({
                "command": "toksi",
                "args": ["run-mcp", "--tool", key, "toksi", subcmd, "mcp"]
            }),
        );
    }
    json!({ "mcpServers": servers })
}

fn default_hook_json(agent_name: &str) -> Value {
    let cmd = format!("toksi rtk-hook {}", agent_name);
    if agent_name == "copilot" {
        json!({
            "version": 1,
            "hooks": {
                "preToolUse": [{ "matcher": "run_command", "hooks": [{ "type": "command", "command": cmd, "timeoutSec": 120 }] }],
                "postToolUse": [{ "matcher": "run_command", "hooks": [{ "type": "command", "command": cmd, "timeoutSec": 120 }] }]
            }
        })
    } else {
        json!({
            "hooks": {
                "PreToolUse": [{ "matcher": "Bash|run_command|bash|terminal|exec", "hooks": [{ "type": "command", "command": cmd, "timeout": 120 }] }],
                "PostToolUse": [{ "matcher": "Bash|run_command|bash|terminal|exec", "hooks": [{ "type": "command", "command": cmd, "timeout": 120 }] }]
            }
        })
    }
}

fn default_gemini_settings_json() -> Value {
    json!({
        "antigravity.experimental.terminalHook": "toksi",
        "permissions": {
            "allow": [
                "command(toksi)",
                "command(rtk)",
                "mcp(codegraph:*)",
                "mcp(context:*)",
                "mcp(projectmem:*)",
                "mcp(aimemory:*)",
                "mcp(icm:*)"
            ]
        }
    })
}

/// Wires tool settings/instructions from `<agent>/mcp` directories (including `.gemini/mcp`,
/// `.copilot/mcp`, `.claude/mcp`, `.agents/mcp`, `.gemini/antigravity/mcp`) and MCP JSON server keys
/// into `<agent>/skills/<tool>`.
pub fn wire_mcp_skills(base_dir: impl AsRef<Path>, tools: &[&str]) -> Result<Vec<PathBuf>, DynError> {
    let owned: Vec<String> = tools.iter().map(|s| (*s).to_string()).collect();
    wire_mcp_skills_for_dirs(base_dir.as_ref(), SUPPORTED_AGENT_DIRS, &owned)
        .map_err(|e| Box::new(e) as DynError)
}

pub fn wire_mcp_skills_for_dirs(
    base_dir: &Path,
    agent_dirs: &[&str],
    tools: &[String],
) -> Result<Vec<PathBuf>, RulesError> {
    let mut wired = Vec::new();
    let mut scan_roots: Vec<PathBuf> = agent_dirs.iter().map(|d| base_dir.join(d)).collect();
    let antigravity_sub = base_dir.join(".gemini").join("antigravity");
    if agent_dirs.contains(&".gemini") && antigravity_sub.exists() {
        scan_roots.push(antigravity_sub);
    }

    for agent_path in scan_roots {
        let mcp_dir = agent_path.join("mcp");
        let skills_dir = agent_path.join("skills");

        // Also discover tools declared inside agent's MCP JSON settings
        let mut effective_tools: BTreeSet<String> = tools.iter().cloned().collect();
        for cfg_name in [".mcp.json", "mcp.json", "mcp_config.json", "mcp-config.json"] {
            let cfg_path = agent_path.join(cfg_name);
            if let Ok(raw) = fs::read_to_string(&cfg_path) {
                if let Ok(Value::Object(root)) = serde_json::from_str::<Value>(&raw) {
                    if let Some(Value::Object(servers)) = root.get("mcpServers") {
                        for k in servers.keys() {
                            let canonical = crate::agents::normalize_mcp_tool_id(k);
                            effective_tools.insert(canonical.to_string());
                        }
                    }
                }
            }
        }

        for tool in &effective_tools {
            let tool_mcp_dir = mcp_dir.join(tool);
            let tool_mcp_file = mcp_dir.join(format!("{}.md", tool));
            let tool_mcp_json = mcp_dir.join(format!("{}.json", tool));
            let tool_skill = skills_dir.join(tool);

            if tool_mcp_dir.is_dir() {
                fs::create_dir_all(&tool_skill).map_err(|e| RulesError::io(&tool_skill, e))?;
                if let Ok(entries) = fs::read_dir(&tool_mcp_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            let target = tool_skill.join(entry.file_name());
                            if let Ok(content) = fs::read_to_string(&path) {
                                fs::write(&target, content).map_err(|e| RulesError::io(&target, e))?;
                                wired.push(PathBuf::from(normalize_path(&target.to_string_lossy())));
                            }
                        }
                    }
                }
            } else if tool_mcp_file.is_file() || tool_mcp_json.is_file() {
                fs::create_dir_all(&tool_skill).map_err(|e| RulesError::io(&tool_skill, e))?;
                for src in [&tool_mcp_file, &tool_mcp_json] {
                    if src.is_file() {
                        if let (Some(name), Ok(content)) = (src.file_name(), fs::read_to_string(src)) {
                            let target = tool_skill.join(name);
                            fs::write(&target, content).map_err(|e| RulesError::io(&target, e))?;
                            wired.push(PathBuf::from(normalize_path(&target.to_string_lossy())));
                        }
                    }
                }
            }

            if agent_path.exists() {
                let skill_file = tool_skill.join("SKILL.md");
                if !skill_file.exists() {
                    fs::create_dir_all(&tool_skill).map_err(|e| RulesError::io(&tool_skill, e))?;
                    let manifest = format!(
                        "---\nname: {}\ndescription: Toksi consolidated {} intelligence tool for AI coding agents.\nlicense: Apache-2.0\n---\n\n# {}\n\nToksi in-process tool wire for {}.\n",
                        tool, tool, tool, tool
                    );
                    fs::write(&skill_file, manifest).map_err(|e| RulesError::io(&skill_file, e))?;
                    wired.push(PathBuf::from(normalize_path(&skill_file.to_string_lossy())));
                }
            }
        }
    }

    Ok(wired)
}

fn sync_harness(
    base_dir: &Path,
    scope: SyncScope,
    spec: &HarnessSpec,
    rules: &[RuleKind],
    opts: &SyncOptions,
    report: &mut SyncReport,
) {
    let comp = opts.components;
    let agent_dir = base_dir.join(spec.primary_dir);

    // Rules & Instructions components
    if comp.rules || comp.instructions {
        sync_harness_rules_and_instructions(base_dir, &agent_dir, scope, spec, rules, comp, report);
    }

    // Existing markdown convention files directly inside base agent dir
    if (comp.rules || comp.instructions) && agent_dir.exists() {
        if let Ok(entries) = fs::read_dir(&agent_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if matches_sync_target_filename(&name) {
                        sync_markdown_target(report, &path, rules);
                    }
                }
            }
        }
    }

    // Skills component
    if comp.skills {
        sync_harness_skills(base_dir, &agent_dir, spec, rules, report);
    }

    // Configs component (`settings.json`, `commands/*.toml`, `.toml` stubs)
    if comp.configs {
        sync_harness_configs(base_dir, &agent_dir, spec, rules, report);
    }

    // Hooks component
    if comp.hooks {
        sync_harness_hooks(base_dir, &agent_dir, spec, report);
    }

    // MCP component
    if comp.mcp {
        sync_harness_mcp(base_dir, spec, &opts.mcp_tools, report);
    }
}

fn sync_harness_rules_and_instructions(
    base_dir: &Path,
    agent_dir: &Path,
    scope: SyncScope,
    spec: &HarnessSpec,
    rules: &[RuleKind],
    comp: SyncComponents,
    report: &mut SyncReport,
) {
    // Workspace root markdown files when workspace scope
    if scope == SyncScope::Workspace {
        if let Some(root_file) = spec.root_instruction_file {
            let target = base_dir.join(root_file);
            let res = match spec.id {
                "claude" => patch_claude_md(base_dir, rules),
                "openagents" => patch_agents_md(base_dir, rules),
                "antigravity" => patch_gemini_md(base_dir, rules),
                "codex" => patch_codex_md(base_dir, rules),
                "cursor" => patch_cursorrules(base_dir, rules),
                "cline" => patch_cline_rules(base_dir, rules),
                _ => patch_markdown_file(&target, rules),
            };
            match res {
                Ok(true) => report.record_updated(&target),
                Ok(false) => report.record_up_to_date(&target),
                Err(e) => report.record_error(format!("{}: {}", root_file, e)),
            }
        }
    }

    match spec.id {
        "claude" => {
            if comp.rules {
                let target = if scope == SyncScope::UserGlobal {
                    agent_dir.join("CLAUDE.md")
                } else {
                    agent_dir.join("rules").join("rules.md")
                };
                if let Some(p) = target.parent() {
                    let _ = fs::create_dir_all(p);
                }
                sync_markdown_target(report, &target, rules);
            }
            if comp.instructions {
                let inst = agent_dir.join("instructions.md");
                if scope == SyncScope::UserGlobal || inst.exists() {
                    sync_markdown_target(report, &inst, rules);
                }
            }
        }
        "openagents" => {
            let _ = fs::create_dir_all(agent_dir);
            if comp.rules {
                let agents_rules = agent_dir.join("rules.md");
                match patch_agents_rules_md(base_dir, rules) {
                    Ok(true) => report.record_updated(&agents_rules),
                    Ok(false) => report.record_up_to_date(&agents_rules),
                    Err(e) => report.record_error(format!(".agents/rules.md: {}", e)),
                }
            }
            if comp.instructions {
                sync_markdown_target(report, &agent_dir.join("agents.md"), rules);
                if scope == SyncScope::Workspace {
                    sync_markdown_target(report, &agent_dir.join("system-prompt.md"), rules);
                }
            }
        }
        "antigravity" => {
            if comp.rules {
                let rules_target = if scope == SyncScope::UserGlobal {
                    agent_dir.join("rules.md")
                } else {
                    agent_dir.join("rules").join("rules.md")
                };
                if let Some(p) = rules_target.parent() {
                    let _ = fs::create_dir_all(p);
                }
                match patch_gemini_md(&rules_target, rules) {
                    Ok(true) => report.record_updated(&rules_target),
                    Ok(false) => report.record_up_to_date(&rules_target),
                    Err(e) => report.record_error(format!(".gemini/rules: {}", e)),
                }
            }
            if comp.instructions {
                let target = if scope == SyncScope::UserGlobal {
                    agent_dir.join("GEMINI.md")
                } else {
                    agent_dir.join("instructions.md")
                };
                sync_markdown_target(report, &target, rules);
            }
        }
        "copilot" => {
            if comp.instructions {
                if scope == SyncScope::Workspace {
                    let gh_copilot = base_dir.join(".github").join("copilot-instructions.md");
                    match patch_copilot_md(base_dir, rules) {
                        Ok(true) => report.record_updated(&gh_copilot),
                        Ok(false) => report.record_up_to_date(&gh_copilot),
                        Err(e) => report.record_error(format!(".github/copilot-instructions.md: {}", e)),
                    }
                    sync_markdown_target(report, &base_dir.join(".github").join("instructions").join("instructions.md"), rules);
                }
                sync_markdown_target(report, &agent_dir.join("instructions").join("instructions.md"), rules);
            }
            if comp.rules {
                sync_markdown_target(report, &agent_dir.join("rules.md"), rules);
            }
        }
        "codex" => {
            if scope == SyncScope::Workspace && comp.instructions {
                sync_markdown_target(report, &base_dir.join("AGENTS.md"), rules);
            }
            if comp.rules {
                let target = agent_dir.join("rules").join("rules.md");
                sync_markdown_target(report, &target, rules);
            }
            if comp.instructions {
                sync_markdown_target(report, &agent_dir.join("instructions.md"), rules);
            }
        }
        "cursor" => {
            if comp.rules {
                match patch_cursor_rules(agent_dir, rules) {
                    Ok(paths) => {
                        for p in paths {
                            report.record_updated(p);
                        }
                    }
                    Err(e) => report.record_error(format!(".cursor/rules: {}", e)),
                }
            }
            if comp.instructions {
                sync_markdown_target(report, &agent_dir.join("instructions.md"), rules);
            }
        }
        "grok" => {
            if comp.rules {
                let grok_agents = agent_dir.join("rules").join("AGENTS.md");
                sync_markdown_target(report, &grok_agents, rules);
            }
            if comp.instructions {
                sync_markdown_target(report, &agent_dir.join("instructions.md"), rules);
            }
        }
        "cline" => {
            if comp.rules {
                let cline_rules = agent_dir.join("rules.md");
                match patch_cline_rules(&cline_rules, rules) {
                    Ok(true) => report.record_updated(&cline_rules),
                    Ok(false) => report.record_up_to_date(&cline_rules),
                    Err(e) => report.record_error(format!(".cline/rules.md: {}", e)),
                }
            }
            if comp.instructions {
                sync_markdown_target(report, &agent_dir.join("instructions.md"), rules);
            }
        }
        _ => {
            if comp.rules {
                sync_markdown_target(report, &agent_dir.join("rules.md"), rules);
            }
            if comp.instructions {
                sync_markdown_target(report, &agent_dir.join("instructions.md"), rules);
            }
        }
    }
}

fn sync_harness_skills(
    base_dir: &Path,
    agent_dir: &Path,
    spec: &HarnessSpec,
    rules: &[RuleKind],
    report: &mut SyncReport,
) {
    match patch_skills_dir(agent_dir, rules) {
        Ok(paths) => {
            for p in paths {
                report.record_updated(p);
            }
        }
        Err(e) => report.record_error(format!("{}/skills: {}", spec.primary_dir, e)),
    }

    if let Some(sec) = spec.secondary_dir {
        if spec.id == "copilot" {
            match patch_skills_dir(base_dir.join(sec), rules) {
                Ok(paths) => {
                    for p in paths {
                        report.record_updated(p);
                    }
                }
                Err(e) => report.record_error(format!("{}/skills: {}", sec, e)),
            }
        }
    }
}

fn sync_harness_configs(
    _base_dir: &Path,
    agent_dir: &Path,
    spec: &HarnessSpec,
    rules: &[RuleKind],
    report: &mut SyncReport,
) {
    if spec.id == "antigravity" {
        match patch_gemini_commands(agent_dir, rules) {
            Ok(paths) => {
                for p in paths {
                    report.record_updated(p);
                }
            }
            Err(e) => report.record_error(format!(".gemini/commands: {}", e)),
        }
        ensure_json_file_merged(report, &agent_dir.join("settings.json"), &default_gemini_settings_json());
    } else if spec.id == "claude" || spec.id == "copilot" || spec.id == "cursor" {
        if let Some(rel) = spec.settings_rel {
            let file_name = Path::new(rel).file_name().unwrap_or_default();
            ensure_json_file_merged(report, &agent_dir.join(file_name), &json!({}));
        }
    } else if spec.id == "codex" || spec.id == "grok" {
        ensure_file_if_missing(report, &agent_dir.join("config.toml"), "# Managed by toksi\n");
    } else if spec.id == "openagents" {
        for sub in ["agents", "knowledge", "layouts", "memories", "tasks"] {
            let dir = agent_dir.join(sub);
            if let Err(e) = fs::create_dir_all(&dir) {
                report.record_error(format!("{}: {}", normalize_path(&dir.to_string_lossy()), e));
            }
        }
    }
}

fn sync_harness_hooks(
    base_dir: &Path,
    agent_dir: &Path,
    spec: &HarnessSpec,
    report: &mut SyncReport,
) {
    let Some(agent_name) = spec.hook_agent_name else {
        return;
    };
    let payload = default_hook_json(agent_name);
    match spec.id {
        "copilot" => {
            ensure_json_file_merged(report, &agent_dir.join("hooks").join("rtk.json"), &payload);
            ensure_json_file_merged(
                report,
                &base_dir.join(".github").join("hooks").join("rtk.json"),
                &payload,
            );
        }
        "grok" => {
            ensure_json_file_merged(report, &agent_dir.join("hooks").join("rtk.json"), &payload);
        }
        "antigravity" | "cursor" => {
            ensure_json_file_merged(report, &agent_dir.join("hooks.json"), &payload);
        }
        "claude" => {
            ensure_json_file_merged(report, &agent_dir.join("settings.json"), &payload);
        }
        _ => {}
    }
}

fn sync_harness_mcp(
    base_dir: &Path,
    spec: &HarnessSpec,
    tools: &[String],
    report: &mut SyncReport,
) {
    let defaults = default_mcp_servers_json(tools);
    if let Some(rel) = spec.mcp_config_rel {
        ensure_json_file_merged(report, &base_dir.join(rel), &defaults);
    }
    if spec.id == "copilot" {
        ensure_json_file_merged(report, &base_dir.join(".github").join("mcp.json"), &defaults);
    }
}

/// Synchronizes rules, skills, hooks, MCP configs, and harness settings into the resolved first workspace root
/// strictly for user-configured/selected agents matching `opts`.
pub fn sync_workspace_rules_with_options(
    workspace: impl AsRef<Path>,
    rules: &[RuleKind],
    opts: &SyncOptions,
) -> Result<SyncReport, RulesError> {
    let root = resolve_workspace_root(workspace);
    let mut report = SyncReport {
        workspace: root.clone(),
        ..Default::default()
    };

    if !opts.allow_workspace_install {
        return Ok(report);
    }

    // 1. Auto-merge nested `.toksi/` and `.<agent>/` dirs into root workspace cleanly.
    if opts.auto_merge_nested {
        match consolidate_nested_toksi_dirs(&root) {
            Ok(n) => report.merged_dirs += n,
            Err(e) => report.record_error(format!("consolidate .toksi: {}", e)),
        }
        match consolidate_nested_agent_dirs(&root) {
            Ok(n) => report.merged_dirs += n,
            Err(e) => report.record_error(format!("consolidate agent dirs: {}", e)),
        }
    }

    // 2. Determine configured/selected agents ONLY (when unselected/empty -> no unsolicited workspace writes).
    let harnesses = resolve_target_harnesses(&root, None, opts)?;
    if harnesses.is_empty() {
        return Ok(report);
    }

    // 3. Scan existing matching convention files at root when rules/instructions enabled.
    if (opts.components.rules || opts.components.instructions) && root.exists() {
        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if matches_sync_target_filename(&name) {
                        sync_markdown_target(&mut report, &path, rules);
                    }
                }
            }
        }
    }

    // 4. Sync each configured/selected harness according to component gates.
    let mut wired_dirs: Vec<&'static str> = Vec::new();
    for spec in &harnesses {
        report.synced_agents.push(spec.id.to_string());
        sync_harness(&root, SyncScope::Workspace, spec, rules, opts, &mut report);
        wired_dirs.push(spec.primary_dir);
        if let Some(sec) = spec.secondary_dir {
            wired_dirs.push(sec);
        }
    }

    // 5. Wire `.{gemini,copilot,claude,agents}/mcp` -> `skills/<tool>`.
    if opts.components.mcp && opts.components.skills {
        match wire_mcp_skills_for_dirs(&root, &wired_dirs, &opts.mcp_tools) {
            Ok(paths) => {
                for p in paths {
                    report.record_updated(p);
                }
            }
            Err(e) => report.record_error(format!("wire_mcp_skills: {}", e)),
        }
    }

    Ok(report)
}

/// Synchronizes rules strictly for user-configured/selected agents in a given user home directory.
pub fn sync_user_dir_rules_with_options(
    home_dir: impl AsRef<Path>,
    rules: &[RuleKind],
    opts: &SyncOptions,
) -> Result<SyncReport, RulesError> {
    let home_path = PathBuf::from(normalize_path(&home_dir.as_ref().to_string_lossy()));
    let mut report = SyncReport {
        workspace: home_path.clone(),
        ..Default::default()
    };

    let harnesses = resolve_target_harnesses(&home_path, Some(&home_path), opts)?;
    if harnesses.is_empty() {
        return Ok(report);
    }

    let mut wired_dirs: Vec<&'static str> = Vec::new();
    for spec in &harnesses {
        report.synced_agents.push(spec.id.to_string());
        sync_harness(&home_path, SyncScope::UserGlobal, spec, rules, opts, &mut report);
        wired_dirs.push(spec.primary_dir);
        if let Some(sec) = spec.secondary_dir {
            wired_dirs.push(sec);
        }
    }

    if opts.components.mcp && opts.components.skills {
        match wire_mcp_skills_for_dirs(&home_path, &wired_dirs, &opts.mcp_tools) {
            Ok(paths) => {
                for p in paths {
                    report.record_updated(p);
                }
            }
            Err(e) => report.record_error(format!("wire_mcp_skills (global): {}", e)),
        }
    }

    Ok(report)
}

/// Synchronizes user-level configuration files across configured/selected agents in `home()`.
pub fn sync_global_rules_with_options(
    rules: &[RuleKind],
    opts: &SyncOptions,
) -> Result<SyncReport, RulesError> {
    let home_dir = home();
    if home_dir.is_empty() {
        return Err(RulesError::HomeUnavailable);
    }
    sync_user_dir_rules_with_options(PathBuf::from(home_dir), rules, opts)
}

/// Synchronizes both workspace (when opted in) and global user configs for configured/selected agents.
pub fn sync_all_rules_with_options(
    workspace: impl AsRef<Path>,
    include_global: bool,
    rules: &[RuleKind],
    opts: &SyncOptions,
) -> Result<SyncReport, RulesError> {
    let mut ws_report = sync_workspace_rules_with_options(workspace, rules, opts)?;
    if include_global {
        let global_report = sync_global_rules_with_options(rules, opts)?;
        ws_report.updated_files.extend(global_report.updated_files);
        ws_report.up_to_date_files.extend(global_report.up_to_date_files);
        for agent in global_report.synced_agents {
            if !ws_report.synced_agents.contains(&agent) {
                ws_report.synced_agents.push(agent);
            }
        }
        ws_report.errors.extend(global_report.errors);
    }
    Ok(ws_report)
}

/// Backward-compatible workspace sync wrapper using default options (configured/selected agents only).
pub fn sync_workspace_rules(
    workspace: impl AsRef<Path>,
    rules: &[RuleKind],
) -> Result<SyncReport, DynError> {
    sync_workspace_rules_with_options(workspace, rules, &SyncOptions::default())
        .map_err(|e| Box::new(e) as DynError)
}

/// Backward-compatible user-global sync wrapper using default options.
pub fn sync_global_rules(rules: &[RuleKind]) -> Result<SyncReport, DynError> {
    match sync_global_rules_with_options(rules, &SyncOptions::default()) {
        Ok(r) => Ok(r),
        Err(RulesError::HomeUnavailable) => {
            let mut report = SyncReport::default();
            report.record_error("Cannot resolve user home directory");
            Ok(report)
        }
        Err(e) => Err(Box::new(e) as DynError),
    }
}

/// Backward-compatible full sync wrapper using default options.
pub fn sync_all_rules(
    workspace: impl AsRef<Path>,
    include_global: bool,
    rules: &[RuleKind],
) -> Result<SyncReport, DynError> {
    sync_all_rules_with_options(workspace, include_global, rules, &SyncOptions::default())
        .map_err(|e| Box::new(e) as DynError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_sync_workspace_configured_only_skips_unconfigured_agents() -> Result<(), DynError> {
        let tmp = tempdir()?;
        // Pre-configure ONLY claude and antigravity (gemini)
        fs::create_dir_all(tmp.path().join(".claude"))?;
        fs::create_dir_all(tmp.path().join(".gemini"))?;

        let report = sync_workspace_rules(tmp.path(), RuleKind::all())?;
        assert!(report.is_success());
        assert!(tmp.path().join("CLAUDE.md").exists());
        assert!(tmp.path().join("GEMINI.md").exists());
        assert!(tmp.path().join(".gemini/settings.json").exists());
        assert!(tmp.path().join(".gemini/commands/ultracave.toml").exists());
        assert!(tmp.path().join(".gemini/commands/ponytail.toml").exists());
        assert!(tmp.path().join(".claude/.mcp.json").exists());
        assert!(tmp.path().join(".claude/skills/ultracave/SKILL.md").exists());
        assert!(tmp.path().join(".claude/skills/ponytail-debt/SKILL.md").exists());

        // Unconfigured agents MUST NOT be created
        assert!(!tmp.path().join(".cursor").exists());
        assert!(!tmp.path().join(".grok").exists());
        assert!(!tmp.path().join(".codex").exists());
        assert!(!tmp.path().join(".clinerules").exists());
        Ok(())
    }

    #[test]
    fn test_sync_selected_agents_and_component_toggles() -> Result<(), DynError> {
        let tmp = tempdir()?;
        let opts = SyncOptions::for_agents(["claude", "openagents", "antigravity", "copilot", "codex", "cursor", "grok", "cline"]);
        let report = sync_workspace_rules_with_options(tmp.path(), RuleKind::all(), &opts)?;

        assert!(report.is_success());
        assert!(tmp.path().join("CLAUDE.md").exists());
        assert!(tmp.path().join("AGENTS.md").exists());
        assert!(tmp.path().join("GEMINI.md").exists());
        assert!(tmp.path().join(".github/copilot-instructions.md").exists());
        assert!(tmp.path().join(".copilot/hooks/rtk.json").exists());
        assert!(tmp.path().join(".clinerules").exists());
        assert!(tmp.path().join("CODEX.md").exists());
        assert!(tmp.path().join(".cursor/rules/principles.mdc").exists());
        assert!(tmp.path().join(".agents/skills/principles/SKILL.md").exists());
        assert!(tmp.path().join(".agents/agents.md").exists());
        assert!(tmp.path().join(".agents/system-prompt.md").exists());
        assert!(tmp.path().join(".grok/rules/AGENTS.md").exists());
        assert!(tmp.path().join(".grok/hooks/rtk.json").exists());

        // Second sync reports up-to-date idempotency
        let report2 = sync_workspace_rules_with_options(tmp.path(), RuleKind::all(), &opts)?;
        assert!(report2.is_success());
        assert!(report2.up_to_date_files.len() >= 6);

        // Component toggles: skills-only in a clean dir does not write hooks/configs/rules
        let tmp_skills = tempdir()?;
        let skills_only = SyncOptions::for_agents(["claude"]).with_components(SyncComponents {
            skills: true,
            ..SyncComponents::none()
        });
        let r3 = sync_workspace_rules_with_options(tmp_skills.path(), RuleKind::all(), &skills_only)?;
        assert!(r3.is_success());
        assert!(tmp_skills.path().join(".claude/skills/caveman/SKILL.md").exists());
        assert!(!tmp_skills.path().join("CLAUDE.md").exists());
        assert!(!tmp_skills.path().join(".claude/.mcp.json").exists());
        Ok(())
    }

    #[test]
    fn test_sync_first_root_placement_and_nested_auto_merge() -> Result<(), DynError> {
        let repo = tempdir()?;
        fs::create_dir_all(repo.path().join(".git"))?;
        let nested_crate = repo.path().join("crates").join("sub");
        fs::create_dir_all(nested_crate.join(".claude").join("skills").join("custom"))?;
        fs::write(nested_crate.join(".claude").join("skills").join("custom").join("SKILL.md"), "# Custom\n")?;
        fs::create_dir_all(nested_crate.join(".toksi"))?;
        fs::write(nested_crate.join(".toksi").join("note.txt"), "hi")?;

        let opts = SyncOptions::for_agents(["claude"]);
        let report = sync_workspace_rules_with_options(&nested_crate, RuleKind::all(), &opts)?;

        assert!(report.is_success());
        assert_eq!(
            normalize_path(&report.workspace.to_string_lossy()),
            normalize_path(&repo.path().to_string_lossy())
        );
        // Nested dirs auto-merged into first repo root and removed from nested crate
        assert!(repo.path().join(".claude/skills/custom/SKILL.md").exists());
        assert!(repo.path().join("CLAUDE.md").exists());
        assert!(!nested_crate.join(".claude").exists());
        assert!(!nested_crate.join(".toksi").exists());
        Ok(())
    }

    #[test]
    fn test_wire_mcp_skills_all_supported_dirs() -> Result<(), DynError> {
        let tmp = tempdir()?;
        for dir in [".gemini", ".copilot", ".claude", ".agents"] {
            let mcp_dir = tmp.path().join(dir).join("mcp").join("codegraph");
            fs::create_dir_all(&mcp_dir)?;
            fs::write(mcp_dir.join("instructions.md"), "# CodeGraph Instructions\n")?;
        }

        let wired = wire_mcp_skills(tmp.path(), &["codegraph"])?;
        assert!(!wired.is_empty());
        for dir in [".gemini", ".copilot", ".claude", ".agents"] {
            assert!(tmp.path().join(dir).join("skills").join("codegraph").join("instructions.md").exists());
            assert!(tmp.path().join(dir).join("skills").join("codegraph").join("SKILL.md").exists());
        }
        Ok(())
    }

    #[test]
    fn test_sync_global_user_dir_respects_selected_only() -> Result<(), DynError> {
        let fake_home = tempdir()?;
        fs::create_dir_all(fake_home.path().join(".agents"))?;

        let opts = SyncOptions::for_agents(["openagents"]);
        let report = sync_user_dir_rules_with_options(fake_home.path(), RuleKind::all(), &opts)?;
        assert!(report.is_success());
        assert!(fake_home.path().join(".agents/agents.md").exists());
        assert!(fake_home.path().join(".agents/mcp.json").exists());
        assert!(fake_home.path().join(".agents/skills/ultracave/SKILL.md").exists());
        assert!(!fake_home.path().join(".claude").exists());
        assert!(!fake_home.path().join(".cursor").exists());
        Ok(())
    }

    #[test]
    fn test_matches_sync_target_filename_and_component_parse() -> Result<(), DynError> {
        assert!(matches_sync_target_filename("rules.md"));
        assert!(matches_sync_target_filename("rules_dev.md"));
        assert!(matches_sync_target_filename("skills.md"));
        assert!(matches_sync_target_filename("instructions.md"));
        assert!(matches_sync_target_filename("copilot-instructions.md"));
        assert!(matches_sync_target_filename("AGENTS.md"));
        assert!(matches_sync_target_filename("CLAUDE.md"));
        assert!(matches_sync_target_filename("GEMINI.md"));
        assert!(matches_sync_target_filename(".cursorrules"));
        assert!(matches_sync_target_filename(".clinerules"));
        assert!(!matches_sync_target_filename("Cargo.toml"));

        let comp = SyncComponents::from_csv("hooks,skills")?;
        assert!(comp.hooks && comp.skills && !comp.mcp && !comp.rules && !comp.configs);
        assert!(SyncComponents::from_csv("bogus").is_err());
        Ok(())
    }
}
