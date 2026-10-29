// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/aimemory/handoff.rs

use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use serde::{Deserialize, Serialize};

pub const SUPPORTED_AGENTS: &[&str] = &[
    "claude",
    "cursor",
    "antigravity",
    "codex",
    "opencode",
    "copilot",
    "cline",
    "amp",
    "droid",
    "dsh",
    "grok",
    "kilo",
    "kimi",
    "omp",
    "pi",
    "vscode",
    "zcode",
];

pub fn normalize_agent_id(agent: &str) -> String {
    let lower = agent.trim().to_lowercase();
    match lower.as_str() {
        "claude-code" | "anthropic" => "claude".to_string(),
        "opencode2" | "open-code" => "opencode".to_string(),
        "factory-droid" => "droid".to_string(),
        "github-copilot" | "github" => "copilot".to_string(),
        "agy" | "google-antigravity" => "antigravity".to_string(),
        "kimi-code" => "kimi".to_string(),
        "kiro-cli" => "kilo".to_string(),
        "command-code" => "codex".to_string(),
        other => other.to_string(),
    }
}

pub fn is_supported_agent(agent: &str) -> bool {
    let norm = normalize_agent_id(agent);
    norm == "any" || SUPPORTED_AGENTS.contains(&norm.as_str())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HandoffState {
    Offered,
    Claimed,
    Resolved,
    Cancelled,
}

impl HandoffState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Offered => "offered",
            Self::Claimed => "claimed",
            Self::Resolved => "resolved",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handoff {
    pub id: String,
    pub title: String,
    pub task: String,
    pub from_agent: String,
    pub to_agent: Option<String>,
    pub claimed_by: Option<String>,
    pub state: HandoffState,
    pub active_files: Vec<String>,
    pub unresolved_issues: Vec<String>,
    pub next_steps: Vec<String>,
    pub context_notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub claimed_at: Option<String>,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewHandoff {
    pub title: String,
    pub task: String,
    pub from_agent: String,
    pub to_agent: Option<String>,
    pub active_files: Vec<String>,
    pub unresolved_issues: Vec<String>,
    pub next_steps: Vec<String>,
    pub context_notes: Option<String>,
}

pub struct HandoffStore {
    base_dir: PathBuf,
}

impl HandoffStore {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        let dir = base_dir.as_ref().to_path_buf();
        let _ = fs::create_dir_all(&dir);
        Self { base_dir: dir }
    }

    pub fn open_default() -> Self {
        Self::new(crate::aimemory::default_handoffs_dir())
    }

    fn handoff_path(&self, id: &str) -> PathBuf {
        self.base_dir.join(format!("{}.json", id))
    }

    pub fn offer(&self, input: NewHandoff) -> Result<Handoff, String> {
        let now = Utc::now().to_rfc3339();
        let id = format!("handoff_{:x}", rand_u64());

        let from_norm = normalize_agent_id(&input.from_agent);
        let to_norm = input.to_agent.map(|t| normalize_agent_id(&t));

        let handoff = Handoff {
            id: id.clone(),
            title: input.title,
            task: input.task,
            from_agent: from_norm,
            to_agent: to_norm,
            claimed_by: None,
            state: HandoffState::Offered,
            active_files: input.active_files,
            unresolved_issues: input.unresolved_issues,
            next_steps: input.next_steps,
            context_notes: input.context_notes,
            created_at: now.clone(),
            updated_at: now,
            claimed_at: None,
            resolved_at: None,
        };

        self.save(&handoff)?;
        Ok(handoff)
    }

    /// Claim-once state machine: Atomic transition from Offered -> Claimed
    pub fn claim(&self, id: &str, agent: &str) -> Result<Handoff, String> {
        let mut handoff = self.get(id)?.ok_or_else(|| format!("handoff '{}' not found", id))?;

        if handoff.state != HandoffState::Offered {
            return Err(format!(
                "cannot claim handoff '{}': state is already '{}' (claimed by {:?})",
                id,
                handoff.state.as_str(),
                handoff.claimed_by
            ));
        }

        let claim_agent = normalize_agent_id(agent);

        if let Some(ref target) = handoff.to_agent {
            let target_norm = normalize_agent_id(target);
            if target_norm != claim_agent && target_norm != "any" {
                return Err(format!(
                    "cannot claim handoff '{}': designated for agent '{}', but claimed by '{}'",
                    id, target, agent
                ));
            }
        }

        let now = Utc::now().to_rfc3339();
        handoff.state = HandoffState::Claimed;
        handoff.claimed_by = Some(claim_agent);
        handoff.claimed_at = Some(now.clone());
        handoff.updated_at = now;

        self.save(&handoff)?;
        Ok(handoff)
    }

    pub fn resolve(&self, id: &str, agent: &str, resolution_note: Option<&str>) -> Result<Handoff, String> {
        let mut handoff = self.get(id)?.ok_or_else(|| format!("handoff '{}' not found", id))?;

        if handoff.state != HandoffState::Claimed {
            return Err(format!(
                "cannot resolve handoff '{}': must be in 'claimed' state, currently '{}'",
                id,
                handoff.state.as_str()
            ));
        }

        let resolver_norm = normalize_agent_id(agent);

        if let Some(ref claimed) = handoff.claimed_by {
            let claimed_norm = normalize_agent_id(claimed);
            if claimed_norm != resolver_norm {
                return Err(format!(
                    "cannot resolve handoff '{}': claimed by '{}', but resolver is '{}'",
                    id, claimed, agent
                ));
            }
        }

        let now = Utc::now().to_rfc3339();
        handoff.state = HandoffState::Resolved;
        handoff.resolved_at = Some(now.clone());
        handoff.updated_at = now;
        if let Some(note) = resolution_note {
            let combined = match handoff.context_notes {
                Some(existing) => format!("{}\n\n[Resolution]: {}", existing, note),
                None => format!("[Resolution]: {}", note),
            };
            handoff.context_notes = Some(combined);
        }

        self.save(&handoff)?;
        Ok(handoff)
    }

    pub fn cancel(&self, id: &str, reason: Option<&str>) -> Result<Handoff, String> {
        let mut handoff = self.get(id)?.ok_or_else(|| format!("handoff '{}' not found", id))?;

        if handoff.state == HandoffState::Resolved {
            return Err(format!("cannot cancel already resolved handoff '{}'", id));
        }

        let now = Utc::now().to_rfc3339();
        handoff.state = HandoffState::Cancelled;
        handoff.updated_at = now;
        if let Some(r) = reason {
            let note = match handoff.context_notes {
                Some(existing) => format!("{}\n\n[Cancelled]: {}", existing, r),
                None => format!("[Cancelled]: {}", r),
            };
            handoff.context_notes = Some(note);
        }

        self.save(&handoff)?;
        Ok(handoff)
    }

    pub fn get(&self, id: &str) -> Result<Option<Handoff>, String> {
        let path = self.handoff_path(id);
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let handoff: Handoff = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(Some(handoff))
    }

    pub fn list_active(&self) -> Result<Vec<Handoff>, String> {
        let mut list = Vec::new();
        if !self.base_dir.exists() {
            return Ok(list);
        }

        let entries = fs::read_dir(&self.base_dir).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(handoff) = serde_json::from_str::<Handoff>(&content) {
                        if handoff.state == HandoffState::Offered || handoff.state == HandoffState::Claimed {
                            list.push(handoff);
                        }
                    }
                }
            }
        }

        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(list)
    }

    pub fn pending_for_agent(&self, agent: &str) -> Result<Vec<Handoff>, String> {
        let norm = normalize_agent_id(agent);
        let active = self.list_active()?;
        let filtered = active
            .into_iter()
            .filter(|h| {
                if h.state == HandoffState::Offered {
                    match h.to_agent.as_deref() {
                        None => true,
                        Some(t) => {
                            let tn = normalize_agent_id(t);
                            tn == "any" || tn == norm
                        }
                    }
                } else if h.state == HandoffState::Claimed {
                    h.claimed_by.as_deref().map(normalize_agent_id).as_deref() == Some(&norm)
                } else {
                    false
                }
            })
            .collect();
        Ok(filtered)
    }

    fn save(&self, handoff: &Handoff) -> Result<(), String> {
        let path = self.handoff_path(&handoff.id);
        let tmp_path = self.base_dir.join(format!("{}.tmp.{}", handoff.id, rand_u64()));
        let content = serde_json::to_string_pretty(handoff).map_err(|e| e.to_string())?;
        fs::write(&tmp_path, content).map_err(|e| e.to_string())?;
        let _ = fs::remove_file(&path);
        fs::rename(&tmp_path, &path).map_err(|e| e.to_string())
    }
}

fn rand_u64() -> u64 {
    use std::time::SystemTime;
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    d.as_nanos() as u64
}

