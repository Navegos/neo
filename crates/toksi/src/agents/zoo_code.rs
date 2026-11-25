// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agents/zoo_code.rs
// created: 2026-11-25

//! `Zoo-Code` (`inspect-vault/Zoo-Code`, Roo Code continuation v3.98.2) connector:
//! - Semble on-demand semantic code search adapter (`semble_semantic_search`)
//! - `.roomodes` custom mode loader & per-mode MCP server allowlists
//! - `.rooignore` path filter rules
//! - Parent/child subtask recovery stack (`SubtaskStack`)
//! - Gemini 4 `xhigh` `thoughtSignature` preservation across multi-turn payloads

#![deny(unsafe_code)]

use std::fs;
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::core::{AgentManifest, Detection};
use crate::headroom::effort::EffortLevel;
use crate::util::paths::{self, normalize_path};

pub fn zoo_code_manifest() -> AgentManifest {
    AgentManifest {
        id: "zoo-code".to_string(),
        label: "Zoo-Code (Roo Code)".to_string(),
        homepage: "https://github.com/RooCodeInc/Roo-Code".to_string(),
        cli_bin: "roo".to_string(),
        config_dir: Some(Arc::new(|| {
            normalize_path(&dirs::home_dir().unwrap_or_default().join(".roo").to_string_lossy())
        })),
        detect: Some(Arc::new(|| {
            if !paths::find_binary::<&str>("roo", &[]).is_empty()
                || !paths::find_binary::<&str>("zoo-code", &[]).is_empty()
            {
                Detection {
                    installed: true,
                    source: "cli".to_string(),
                }
            } else {
                Detection::default()
            }
        })),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RooModeDefinition {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub role_definition: String,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub allowed_mcp_servers: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_regex: Option<String>,
}

impl RooModeDefinition {
    /// Empty `allowed_mcp_servers` permits any server; non-empty restricts strictly to listed IDs.
    pub fn is_mcp_server_allowed(&self, server_id: &str) -> bool {
        if self.allowed_mcp_servers.is_empty() {
            return true;
        }
        let target = server_id.trim().to_ascii_lowercase();
        self.allowed_mcp_servers
            .iter()
            .any(|s| s.trim().to_ascii_lowercase() == target)
    }
}

#[derive(Debug, Deserialize)]
struct RoomodesEnvelope {
    #[serde(default, rename = "customModes")]
    custom_modes: Vec<RoomodesRawEntry>,
}

#[derive(Debug, Deserialize)]
struct RoomodesRawEntry {
    slug: String,
    name: String,
    #[serde(default, rename = "roleDefinition")]
    role_definition: String,
    #[serde(default)]
    groups: Vec<Value>,
    #[serde(default, rename = "allowedMcpServers")]
    allowed_mcp_servers: Vec<String>,
    #[serde(default, rename = "fileRegex")]
    file_regex: Option<String>,
}

pub fn parse_roomodes_json(raw: &str) -> Result<Vec<RooModeDefinition>, String> {
    let env: RoomodesEnvelope =
        serde_json::from_str(raw).map_err(|e| format!("Invalid .roomodes JSON: {e}"))?;
    let mut out = Vec::with_capacity(env.custom_modes.len());
    for m in env.custom_modes {
        let mut groups = Vec::new();
        let mut file_regex = m.file_regex;
        for g in m.groups {
            match g {
                Value::String(s) => groups.push(s),
                Value::Array(arr) => {
                    if let Some(first) = arr.first().and_then(Value::as_str) {
                        groups.push(first.to_string());
                    }
                    if file_regex.is_none()
                        && let Some(obj) = arr.get(1).and_then(Value::as_object)
                        && let Some(rx) = obj.get("fileRegex").and_then(Value::as_str)
                    {
                        file_regex = Some(rx.to_string());
                    }
                }
                _ => {}
            }
        }
        out.push(RooModeDefinition {
            slug: m.slug,
            name: m.name,
            role_definition: m.role_definition,
            groups,
            allowed_mcp_servers: m.allowed_mcp_servers,
            file_regex,
        });
    }
    Ok(out)
}

pub fn load_roomodes_from_workspace(root: &Path) -> Vec<RooModeDefinition> {
    let path = root.join(".roomodes");
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    parse_roomodes_json(&raw).unwrap_or_default()
}

#[derive(Debug, Clone, Default)]
pub struct RooIgnoreRules {
    patterns: Vec<String>,
}

impl RooIgnoreRules {
    pub fn parse(raw: &str) -> Self {
        let patterns = raw
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(ToString::to_string)
            .collect();
        Self { patterns }
    }

    pub fn load_from_workspace(root: &Path) -> Self {
        let path = root.join(".rooignore");
        match fs::read_to_string(path) {
            Ok(raw) => Self::parse(&raw),
            Err(_) => Self::default(),
        }
    }

    pub fn is_ignored(&self, rel_path: &str) -> bool {
        let norm = normalize_path(rel_path).trim_start_matches("./").to_string();
        let mut ignored = false;
        for pat in &self.patterns {
            if let Some(neg) = pat.strip_prefix('!') {
                if wildcard_or_prefix_match(neg.trim(), &norm) {
                    ignored = false;
                }
            } else if wildcard_or_prefix_match(pat, &norm) {
                ignored = true;
            }
        }
        ignored
    }
}

fn wildcard_or_prefix_match(pattern: &str, path: &str) -> bool {
    let p = normalize_path(pattern).trim_start_matches('/').to_string();
    if p.is_empty() {
        return false;
    }
    if let Some(dir_prefix) = p.strip_suffix('/') {
        return path == dir_prefix || path.starts_with(&format!("{dir_prefix}/"));
    }
    if let Some(ext) = p.strip_prefix("*.") {
        return path.ends_with(&format!(".{ext}"));
    }
    if p.contains('*') {
        let parts: Vec<&str> = p.split('*').filter(|s| !s.is_empty()).collect();
        let mut rest = path;
        for part in parts {
            let Some(pos) = rest.find(part) else {
                return false;
            };
            rest = &rest[pos + part.len()..];
        }
        return true;
    }
    path == p || path.starts_with(&format!("{p}/")) || path.ends_with(&format!("/{p}"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SembleSearchQuery {
    pub query: String,
    pub max_results: usize,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SembleSearchHit {
    pub file_path: String,
    pub line: usize,
    pub score: f64,
    pub snippet: String,
}

pub fn semble_semantic_search(root: &Path, query: &SembleSearchQuery) -> Vec<SembleSearchHit> {
    let tokens: Vec<String> = query
        .query
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .filter(|t| t.len() >= 2)
        .map(|t| t.to_ascii_lowercase())
        .collect();
    if tokens.is_empty() {
        return Vec::new();
    }

    let ignore = RooIgnoreRules::load_from_workspace(root);
    let mut hits = Vec::new();
    collect_semble_hits(root, root, &tokens, &query.extensions, &ignore, &mut hits);
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.file_path.cmp(&b.file_path))
    });
    hits.truncate(query.max_results.max(1));
    hits
}

fn collect_semble_hits(
    root: &Path,
    dir: &Path,
    tokens: &[String],
    extensions: &[String],
    ignore: &RooIgnoreRules,
    out: &mut Vec<SembleSearchHit>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if matches!(name, ".git" | "target" | "node_modules" | ".toksi") {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .map(|r| normalize_path(&r.to_string_lossy()))
            .unwrap_or_else(|_| normalize_path(&path.to_string_lossy()));
        if ignore.is_ignored(&rel) {
            continue;
        }
        if path.is_dir() {
            collect_semble_hits(root, &path, tokens, extensions, ignore, out);
            continue;
        }
        if !extensions.is_empty() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !extensions.iter().any(|allowed| allowed.eq_ignore_ascii_case(&ext)) {
                continue;
            }
        }
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        for (idx, line) in raw.lines().enumerate() {
            let low = line.to_ascii_lowercase();
            let matched = tokens.iter().filter(|t| low.contains(t.as_str())).count();
            if matched > 0 {
                let score = matched as f64 / tokens.len() as f64;
                out.push(SembleSearchHit {
                    file_path: rel.clone(),
                    line: idx + 1,
                    score,
                    snippet: line.trim().chars().take(160).collect(),
                });
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtaskStatus {
    Running,
    Completed,
    Recovered,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubtaskFrame {
    pub task_id: String,
    pub parent_id: Option<String>,
    pub mode_slug: String,
    pub status: SubtaskStatus,
    pub summary: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubtaskStack {
    frames: Vec<SubtaskFrame>,
}

impl SubtaskStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_subtask(
        &mut self,
        task_id: impl Into<String>,
        parent_id: Option<String>,
        mode_slug: impl Into<String>,
        summary: impl Into<String>,
    ) {
        self.frames.push(SubtaskFrame {
            task_id: task_id.into(),
            parent_id,
            mode_slug: mode_slug.into(),
            status: SubtaskStatus::Running,
            summary: summary.into(),
        });
    }

    pub fn complete_subtask(&mut self, task_id: &str) -> bool {
        if let Some(f) = self.frames.iter_mut().find(|f| f.task_id == task_id) {
            f.status = SubtaskStatus::Completed;
            true
        } else {
            false
        }
    }

    /// Marks any interrupted `Running` child subtasks belonging to `parent_id` as `Recovered`.
    pub fn recover_orphaned_subtasks(&mut self, parent_id: &str) -> Vec<String> {
        let mut recovered = Vec::new();
        for f in &mut self.frames {
            if f.parent_id.as_deref() == Some(parent_id) && f.status == SubtaskStatus::Running {
                f.status = SubtaskStatus::Recovered;
                recovered.push(f.task_id.clone());
            }
        }
        recovered
    }

    pub fn frames(&self) -> &[SubtaskFrame] {
        &self.frames
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThoughtSignatureEnvelope {
    pub model: String,
    pub effort: EffortLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thought_signature: Option<String>,
    pub content: String,
}

pub fn preserve_thought_signature(payload: &mut Value, signature: &str) {
    if signature.trim().is_empty() {
        return;
    }
    if let Some(obj) = payload.as_object_mut() {
        obj.insert("thoughtSignature".to_string(), json!(signature));
    }
}

pub fn extract_thought_signature(payload: &Value) -> Option<String> {
    payload
        .get("thoughtSignature")
        .or_else(|| payload.get("thought_signature"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roomodes_rooignore_subtasks_and_thought_signature() {
        let modes = parse_roomodes_json(
            r#"{
              "customModes": [
                {
                  "slug": "rust-sec",
                  "name": "Rust Security",
                  "roleDefinition": "Audit Rust code",
                  "groups": ["read", ["edit", {"fileRegex": "\\.rs$"}]],
                  "allowedMcpServers": ["codegraph", "security"]
                }
              ]
            }"#,
        )
        .expect("parse roomodes");
        assert_eq!(modes.len(), 1);
        assert!(modes[0].is_mcp_server_allowed("codegraph"));
        assert!(!modes[0].is_mcp_server_allowed("untrusted-mcp"));
        assert_eq!(modes[0].file_regex.as_deref(), Some("\\.rs$"));

        let ignore = RooIgnoreRules::parse("target/\n*.secret\n!keep.secret\n");
        assert!(ignore.is_ignored("target/debug/toksi"));
        assert!(ignore.is_ignored("keys/api.secret"));
        assert!(!ignore.is_ignored("keep.secret"));

        let mut stack = SubtaskStack::new();
        stack.push_subtask("parent-1", None, "orchestrator", "Parent task");
        stack.push_subtask("child-1", Some("parent-1".into()), "code", "Child step");
        let rec = stack.recover_orphaned_subtasks("parent-1");
        assert_eq!(rec, vec!["child-1"]);

        let mut payload = json!({ "model": "gemini-4-neon", "effort": "xhigh" });
        preserve_thought_signature(&mut payload, "sig-g4-xhigh-xyz");
        assert_eq!(
            extract_thought_signature(&payload).as_deref(),
            Some("sig-g4-xhigh-xyz")
        );
    }
}
