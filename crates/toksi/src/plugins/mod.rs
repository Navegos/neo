// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/plugins/mod.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! Toksi Unified Plugin System (`src/plugins/`) with `<enforce>` pre-load security gate.

#![deny(unsafe_code)]

pub mod hooks;
pub mod loader;
pub mod manifest;
pub mod slash_router;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::db::{open_default_db, PluginRegistryRecord};
use crate::util::paths::{home, normalize_path};

pub use hooks::{global_hook_dispatcher, HookContext, HookEventName, HookOutcome, PluginHookDispatcher};
pub use slash_router::{global_plugin_slash_router, PluginSlashEntry, PluginSlashRouter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    AgentFeature,
    CliCmd,
    Tool,
    ChatCmd,
    WireHook,
    Protocol,
    A2a,
}

impl PluginCapability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AgentFeature => "agent_feature",
            Self::CliCmd => "cli_cmd",
            Self::Tool => "tool",
            Self::ChatCmd => "chat_cmd",
            Self::WireHook => "wire_hook",
            Self::Protocol => "protocol",
            Self::A2a => "a2a",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginSecurityStatus {
    Unscanned,
    Passed,
    BlockedFailClosed,
}

impl PluginSecurityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unscanned => "unscanned",
            Self::Passed => "passed",
            Self::BlockedFailClosed => "blocked_fail_closed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub entry_path: String,
    #[serde(default)]
    pub capabilities: Vec<PluginCapability>,
    #[serde(default)]
    pub protocols: Vec<String>,
    #[serde(default)]
    pub slash_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEntry {
    pub manifest: PluginManifest,
    pub manifest_path: String,
    pub digest_hex: String,
    pub security_status: PluginSecurityStatus,
    pub scan_findings: Vec<String>,
    pub is_loaded: bool,
    pub is_enabled: bool,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginScanReport {
    pub plugin_id: String,
    pub status: PluginSecurityStatus,
    pub findings: Vec<String>,
    pub digest_hex: String,
}

fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    out.iter().map(|b| format!("{:02x}", b)).collect()
}

/// `<enforce>` pre-load security scan for plugin manifests and entry payloads.
pub fn scan_plugin_content(plugin_id: &str, content: &str) -> PluginScanReport {
    let lower = content.to_ascii_lowercase();
    let mut findings = Vec::new();

    let blocked_patterns: &[(&str, &str)] = &[
        ("ignore previous instructions", "Prompt injection directive detected"),
        ("bypass <enforce>", "Attempt to bypass <enforce> boundary"),
        ("curl | sh", "Remote pipe-to-shell pattern detected"),
        ("wget | bash", "Remote pipe-to-shell pattern detected"),
        ("invoke-expression", "PowerShell dynamic code execution detected"),
        ("rm -rf /", "Destructive root filesystem command detected"),
        ("exfiltrate", "Potential data exfiltration directive detected"),
    ];

    for (pat, msg) in blocked_patterns {
        if lower.contains(pat) {
            findings.push(format!("FAIL-CLOSED: {msg} (matched '{pat}')"));
        }
    }

    let status = if findings.is_empty() {
        PluginSecurityStatus::Passed
    } else {
        PluginSecurityStatus::BlockedFailClosed
    };

    PluginScanReport {
        plugin_id: plugin_id.to_string(),
        status,
        findings,
        digest_hex: sha256_hex(content.as_bytes()),
    }
}

pub struct PluginRegistry {
    entries: RwLock<HashMap<String, PluginEntry>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        let reg = Self {
            entries: RwLock::new(HashMap::new()),
        };
        reg.hydrate_from_db();
        reg
    }

    fn hydrate_from_db(&self) {
        let Ok(db) = open_default_db() else {
            return;
        };
        let Ok(rows) = db.list_plugins() else {
            return;
        };
        let Ok(mut guard) = self.entries.write() else {
            return;
        };
        for row in rows {
            if let Ok(manifest) = serde_json::from_str::<PluginManifest>(&row.manifest_json) {
                let status = match row.security_scan_status.as_str() {
                    "passed" => PluginSecurityStatus::Passed,
                    "blocked" | "blocked_fail_closed" => PluginSecurityStatus::BlockedFailClosed,
                    _ => PluginSecurityStatus::Unscanned,
                };
                guard.insert(
                    row.id.clone(),
                    PluginEntry {
                        manifest,
                        manifest_path: row.id.clone(),
                        digest_hex: row.blake3_hash,
                        security_status: status,
                        scan_findings: Vec::new(),
                        is_loaded: row.is_enabled,
                        is_enabled: row.is_enabled,
                        updated_at: row.updated_at,
                    },
                );
            }
        }
    }

    fn persist_entry(&self, entry: &PluginEntry) {
        let Ok(mut db) = open_default_db() else {
            return;
        };
        let manifest_json = serde_json::to_string(&entry.manifest).unwrap_or_else(|_| "{}".to_string());
        let capabilities_json = serde_json::to_string(&entry.manifest.capabilities).unwrap_or_else(|_| "[]".to_string());
        let protocols_json = serde_json::to_string(&entry.manifest.protocols).unwrap_or_else(|_| "[]".to_string());
        let record = PluginRegistryRecord {
            id: entry.manifest.id.clone(),
            name: entry.manifest.name.clone(),
            version: entry.manifest.version.clone(),
            manifest_json,
            blake3_hash: entry.digest_hex.clone(),
            capabilities_json,
            protocols_json,
            security_scan_status: entry.security_status.as_str().to_string(),
            is_enabled: entry.is_enabled,
            installed_at: entry.updated_at,
            updated_at: entry.updated_at,
        };
        let _ = db.upsert_plugin(&record);
    }

    pub fn list(&self) -> Vec<PluginEntry> {
        let mut out = self
            .entries
            .read()
            .map(|g| g.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        out.sort_by(|a, b| a.manifest.id.cmp(&b.manifest.id));
        out
    }

    pub fn get(&self, id: &str) -> Option<PluginEntry> {
        self.entries
            .read()
            .ok()
            .and_then(|g| g.get(id).cloned())
    }

    pub fn scan_path(&self, path: &Path) -> Result<PluginScanReport, String> {
        let norm = normalize_path(&path.to_string_lossy());
        let manifest_file = if path.is_dir() {
            path.join("plugin.json")
        } else {
            path.to_path_buf()
        };
        let raw = std::fs::read_to_string(&manifest_file)
            .map_err(|e| format!("Failed to read plugin manifest '{}': {}", normalize_path(&manifest_file.to_string_lossy()), e))?;
        let manifest: PluginManifest = serde_json::from_str(&raw)
            .map_err(|e| format!("Invalid plugin manifest JSON in '{}': {}", norm, e))?;

        let mut combined = raw.clone();
        if !manifest.entry_path.is_empty() {
            let base = manifest_file.parent().unwrap_or_else(|| Path::new("."));
            let entry_full = base.join(&manifest.entry_path);
            if let Ok(entry_src) = std::fs::read_to_string(&entry_full) {
                combined.push('\n');
                combined.push_str(&entry_src);
            }
        }

        let report = scan_plugin_content(&manifest.id, &combined);
        let entry = PluginEntry {
            manifest: manifest.clone(),
            manifest_path: normalize_path(&manifest_file.to_string_lossy()),
            digest_hex: report.digest_hex.clone(),
            security_status: report.status,
            scan_findings: report.findings.clone(),
            is_loaded: false,
            is_enabled: false,
            updated_at: now_epoch(),
        };
        if let Ok(mut guard) = self.entries.write() {
            guard.insert(manifest.id.clone(), entry.clone());
        }
        self.persist_entry(&entry);
        Ok(report)
    }

    pub fn load(&self, id_or_path: &str) -> Result<PluginEntry, String> {
        let p = Path::new(id_or_path);
        if p.exists() {
            let report = self.scan_path(p)?;
            if report.status == PluginSecurityStatus::BlockedFailClosed {
                return Err(format!(
                    "Plugin '{}' blocked by <enforce> fail-closed security gate: {}",
                    report.plugin_id,
                    report.findings.join("; ")
                ));
            }
            return self.load(&report.plugin_id);
        }

        let mut entry = self
            .get(id_or_path)
            .ok_or_else(|| format!("Plugin '{}' not found in registry", id_or_path))?;

        if entry.security_status == PluginSecurityStatus::BlockedFailClosed {
            return Err(format!(
                "Plugin '{}' is blocked by <enforce> fail-closed security gate",
                entry.manifest.id
            ));
        }

        if entry.security_status == PluginSecurityStatus::Unscanned {
            let raw = serde_json::to_string(&entry.manifest).unwrap_or_default();
            let report = scan_plugin_content(&entry.manifest.id, &raw);
            entry.security_status = report.status;
            entry.scan_findings = report.findings;
            entry.digest_hex = report.digest_hex;
            if entry.security_status == PluginSecurityStatus::BlockedFailClosed {
                self.persist_entry(&entry);
                return Err(format!(
                    "Plugin '{}' blocked during pre-load <enforce> scan",
                    entry.manifest.id
                ));
            }
        }

        entry.is_loaded = true;
        entry.is_enabled = true;
        entry.updated_at = now_epoch();

        // Register any declared plugin slash commands onto the fast PluginSlashRouter
        let pid = entry.manifest.id.clone();
        let pname = entry.manifest.name.clone();
        for cmd in &entry.manifest.slash_commands {
            let cmd_label = cmd.clone();
            let p_clone = pid.clone();
            let name_clone = pname.clone();
            global_plugin_slash_router().register(
                cmd.clone(),
                pid.clone(),
                format!("Plugin '{}' command {}", pname, cmd),
                Arc::new(move |args| {
                    format!(
                        "[plugin:{} ({})] {} {}",
                        p_clone, name_clone, cmd_label, args
                    )
                    .trim()
                    .to_string()
                }),
            );
        }

        if let Ok(mut guard) = self.entries.write() {
            guard.insert(entry.manifest.id.clone(), entry.clone());
        }
        self.persist_entry(&entry);
        Ok(entry)
    }

    pub fn unload(&self, id: &str) -> Result<PluginEntry, String> {
        let mut entry = self
            .get(id)
            .ok_or_else(|| format!("Plugin '{}' not found", id))?;
        entry.is_loaded = false;
        entry.updated_at = now_epoch();
        global_hook_dispatcher().unregister_plugin_hooks(id);
        global_plugin_slash_router().unregister_plugin(id);
        if let Ok(mut guard) = self.entries.write() {
            guard.insert(id.to_string(), entry.clone());
        }
        self.persist_entry(&entry);
        Ok(entry)
    }

    pub fn reload(&self, id: &str) -> Result<PluginEntry, String> {
        let _ = self.unload(id);
        self.load(id)
    }

    pub fn enable(&self, id: &str) -> Result<PluginEntry, String> {
        self.load(id)
    }

    pub fn disable(&self, id: &str) -> Result<PluginEntry, String> {
        let mut entry = self.unload(id)?;
        entry.is_enabled = false;
        entry.updated_at = now_epoch();
        if let Ok(mut guard) = self.entries.write() {
            guard.insert(id.to_string(), entry.clone());
        }
        self.persist_entry(&entry);
        Ok(entry)
    }

    pub fn default_plugin_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        let h = home();
        if !h.is_empty() {
            dirs.push(PathBuf::from(format!("{}/.toksi/plugins", h)));
        }
        dirs.push(PathBuf::from(".toksi/plugins"));
        dirs
    }
}

static GLOBAL_PLUGIN_REGISTRY: OnceLock<Arc<PluginRegistry>> = OnceLock::new();

pub fn global_plugin_registry() -> Arc<PluginRegistry> {
    Arc::clone(GLOBAL_PLUGIN_REGISTRY.get_or_init(|| Arc::new(PluginRegistry::new())))
}
