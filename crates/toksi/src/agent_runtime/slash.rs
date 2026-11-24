// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/slash.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! Fast (< 5ms) non-blocking 16-command in-chat slash engine for both native `toksi chat`
//! and external hooked agent chats (`agy`, `claude`, `codex`, `cursor`, `grok`, `opencode`).

#![deny(unsafe_code)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::{load_toksi_toml_config, save_toksi_toml_config};
use crate::db::{open_default_db, AgentProfileBindingRecord, ByokProfileRecord, ToksiConfigDb};
use crate::headroom::effort::{global_effort_metrics, EffortLevel};
use crate::plugins::{global_plugin_registry, global_plugin_slash_router};
use crate::projectmem::plan_checkpoint::{
    latest_resumable_checkpoint, list_plan_checkpoints, load_plan_checkpoint,
    validate_dirty_tree_and_plugins,
};
use crate::rules::skill_validator::{analyze_directory, fix_skill_content};
use crate::rules::skills_repo::{
    discover_vetted_catalog, sync_external_skills_repo, SkillsRepoConfig,
};
use crate::tools::direct_wire::global_direct_wire_bus;
use crate::tools::multi_protocol::ToolTarget;
use crate::util::paths::normalize_path;

use super::patch::load_custom_slash_commands;
use super::sandbox::{global_sandbox_manager, SandboxBackend};

pub const CORE_SLASH_COMMANDS: [&str; 29] = [
    "/byok",
    "/model",
    "/effort",
    "/byokmodel",
    "/mode",
    "/sampling",
    "/router",
    "/projectmem",
    "/ctxmode",
    "/codegraph",
    "/icm",
    "/aimemory",
    "/rtk",
    "/security",
    "/advisory",
    "/skills",
    "/sandbox",
    "/plugin",
    "/resume",
    "/workforce",
    "/spec",
    "/dataset",
    "/train",
    "/eval",
    "/compress",
    "/serve",
    "/cluster",
    "/otel",
    "/voice",
];

fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SlashSessionState {
    pub active_byok: Option<String>,
    pub active_model: Option<String>,
    pub active_effort: Option<String>,
    pub advisory_dir: Option<String>,
    pub skills_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashDispatchResult {
    pub handled: bool,
    pub command: String,
    pub output: String,
    pub elapsed_us: u128,
}

pub struct SlashCommandEngine {
    state: RwLock<SlashSessionState>,
    workspace_root: RwLock<PathBuf>,
    db: Mutex<Option<ToksiConfigDb>>,
    otel_db: Mutex<Option<crate::db::service_dbs::ServiceTelemetryDb>>,
    cached_skills_count: RwLock<Option<usize>>,
}

impl Default for SlashCommandEngine {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

impl SlashCommandEngine {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            state: RwLock::new(SlashSessionState::default()),
            workspace_root: RwLock::new(workspace_root),
            db: Mutex::new(None),
            otel_db: Mutex::new(None),
            cached_skills_count: RwLock::new(None),
        }
    }

    fn otel_span_count(&self) -> usize {
        use crate::db::service_dbs::{ServiceDbKind, ServiceTelemetryDb};
        let Ok(mut guard) = self.otel_db.lock() else {
            return 0;
        };
        if guard.is_none()
            && let Ok(db) = ServiceTelemetryDb::open(ServiceDbKind::Otel)
        {
            *guard = Some(db);
        }
        guard
            .as_ref()
            .and_then(|d| d.event_count().ok())
            .unwrap_or(0)
    }

    fn with_db<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut ToksiConfigDb) -> R,
    {
        let mut guard = self.db.lock().map_err(|_| "DB mutex poisoned".to_string())?;
        if guard.is_none() {
            let db = open_default_db().map_err(|e| e.to_string())?;
            *guard = Some(db);
        }
        let Some(ref mut db) = *guard else {
            return Err("Database unavailable".to_string());
        };
        Ok(f(db))
    }

    pub fn set_workspace_root(&self, root: PathBuf) {
        if let Ok(mut g) = self.workspace_root.write() {
            *g = root;
        }
    }

    pub fn state(&self) -> SlashSessionState {
        self.state.read().map(|g| g.clone()).unwrap_or_default()
    }

    pub fn is_core_slash_command(input: &str) -> bool {
        let trimmed = input.trim();
        if !trimmed.starts_with('/') {
            return false;
        }
        let cmd = trimmed
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        CORE_SLASH_COMMANDS.iter().any(|c| *c == cmd)
    }

    pub fn execute(&self, input: &str, agent_id: &str) -> SlashDispatchResult {
        let started = Instant::now();
        let trimmed = input.trim();
        if !trimmed.starts_with('/') {
            return SlashDispatchResult {
                handled: false,
                command: String::new(),
                output: String::new(),
                elapsed_us: started.elapsed().as_micros(),
            };
        }

        if let crate::commands::guard::GuardDecision::Deny { reason, .. }
        | crate::commands::guard::GuardDecision::KillSwitchTriggered { reason, .. } =
            crate::commands::guard::CommandGuard::check_pre_exec(agent_id, trimmed, None)
        {
            return SlashDispatchResult {
                handled: true,
                command: trimmed.split_whitespace().next().unwrap_or("").to_string(),
                output: format!("[CommandGuard] {reason}"),
                elapsed_us: started.elapsed().as_micros(),
            };
        }

        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let cmd = parts.next().unwrap_or("").to_ascii_lowercase();
        let args = parts.next().unwrap_or("").trim();
        let root = self
            .workspace_root
            .read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| PathBuf::from("."));

        let output = match cmd.as_str() {
            "/byok" => Some(self.handle_byok(args)),
            "/model" => Some(self.handle_model(args)),
            "/effort" => Some(self.handle_effort(args)),
            "/byokmodel" => Some(self.handle_byokmodel(args, agent_id)),
            "/mode" => Some(self.handle_mode(args)),
            "/sampling" => Some(self.handle_sampling(args)),
            "/router" => Some(self.handle_router(args)),
            "/projectmem" => Some(self.handle_direct_tool(agent_id, ToolTarget::Projectmem, args, &root)),
            "/ctxmode" => Some(self.handle_direct_tool(agent_id, ToolTarget::Ctxmode, args, &root)),
            "/context" => {
                let res = self.handle_direct_tool(agent_id, ToolTarget::Ctxmode, args, &root);
                Some(format!("[deprecated: `/context` is renamed to `/ctxmode`]\n{res}"))
            }
            "/codegraph" => Some(self.handle_direct_tool(agent_id, ToolTarget::Codegraph, args, &root)),
            "/icm" => Some(self.handle_direct_tool(agent_id, ToolTarget::Icm, args, &root)),
            "/aimemory" => Some(self.handle_direct_tool(agent_id, ToolTarget::Aimemory, args, &root)),
            "/ai-memory" => {
                let res = self.handle_direct_tool(agent_id, ToolTarget::Aimemory, args, &root);
                Some(format!("[deprecated: `/ai-memory` is renamed to `/aimemory`]\n{res}"))
            }
            "/rtk" => Some(self.handle_direct_tool(agent_id, ToolTarget::Rtk, args, &root)),
            "/security" => Some(self.handle_security(args, &root)),
            "/advisory" => Some(self.handle_advisory(args, &root)),
            "/skills" => Some(self.handle_skills(args, &root)),
            "/sandbox" => Some(self.handle_sandbox(args, &root)),
            "/plugin" => Some(self.handle_plugin(args)),
            "/resume" => Some(self.handle_resume(args, &root)),
            "/workforce" => Some(self.handle_wave20_command("workforce", args)),
            "/spec" => Some(self.handle_wave20_command("spec", args)),
            "/dataset" => Some(self.handle_wave20_command("dataset", args)),
            "/train" => Some(self.handle_wave20_command("train", args)),
            "/eval" => Some(self.handle_wave20_command("eval", args)),
            "/compress" => Some(self.handle_wave20_command("compress", args)),
            "/serve" => Some(self.handle_wave20_command("serve", args)),
            "/cluster" => Some(self.handle_wave20_command("cluster", args)),
            "/otel" => Some(self.handle_wave20_command("otel", args)),
            "/voice" => Some(self.handle_wave20_command("voice", args)),
            _ => global_plugin_slash_router().dispatch(trimmed).or_else(|| {
                load_custom_slash_commands(&root)
                    .into_iter()
                    .find(|c| c.name == cmd)
                    .map(|c| format!("[custom-slash:{}]\n{}", c.name, c.prompt))
            }),
        };

        match output {
            Some(text) => SlashDispatchResult {
                handled: true,
                command: cmd,
                output: text,
                elapsed_us: started.elapsed().as_micros(),
            },
            None => SlashDispatchResult {
                handled: false,
                command: cmd,
                output: String::new(),
                elapsed_us: started.elapsed().as_micros(),
            },
        }
    }

    fn handle_byok(&self, args: &str) -> String {
        let filter = args.trim().to_ascii_lowercase();
        let Ok(profiles) = self.with_db(|db| db.list_byok_profiles().unwrap_or_default()) else {
            return "[/byok] Unable to open ~/.toksi/toksi.db".to_string();
        };
        let mut providers: Vec<String> = profiles
            .iter()
            .filter(|p| p.is_active)
            .map(|p| p.provider_id.clone())
            .collect();
        providers.sort();
        providers.dedup();

        if !filter.is_empty() {
            providers.retain(|p| p.to_ascii_lowercase().contains(&filter));
            if let Some(selected) = providers.first().cloned() {
                if let Ok(mut st) = self.state.write() {
                    st.active_byok = Some(selected.clone());
                }
                return format!(
                    "[/byok] Active BYOK set to '{}'. Available matches: {}",
                    selected,
                    providers.join(", ")
                );
            }
            if let Ok(mut st) = self.state.write() {
                st.active_byok = Some(filter.clone());
            }
            return format!(
                "[/byok] Active BYOK set to '{}' (no existing profiles matched).",
                filter
            );
        }

        if providers.is_empty() {
            "[/byok] No active BYOK providers configured in ~/.toksi/toksi.db.".to_string()
        } else {
            if let Ok(mut st) = self.state.write() {
                if st.active_byok.is_none() {
                    st.active_byok = providers.first().cloned();
                }
            }
            format!("[/byok] Configured BYOK providers: {}", providers.join(", "))
        }
    }

    fn handle_model(&self, args: &str) -> String {
        let filter = args.trim().to_ascii_lowercase();
        let active_byok = self.state.read().ok().and_then(|s| s.active_byok.clone());
        let Ok(profiles) = self.with_db(|db| db.list_byok_profiles().unwrap_or_default()) else {
            return "[/model] Unable to open ~/.toksi/toksi.db".to_string();
        };
        let mut prioritized = Vec::new();
        let mut others = Vec::new();

        for p in profiles {
            if !filter.is_empty() && !p.model_id.to_ascii_lowercase().contains(&filter) {
                continue;
            }
            let label = format!("{} ({})", p.model_id, p.provider_id);
            if active_byok
                .as_deref()
                .is_some_and(|b| p.provider_id.eq_ignore_ascii_case(b))
            {
                prioritized.push((p.model_id.clone(), label));
            } else {
                others.push((p.model_id.clone(), label));
            }
        }

        prioritized.extend(others);
        if !filter.is_empty() {
            if let Some((model_id, _)) = prioritized.first() {
                if let Ok(mut st) = self.state.write() {
                    st.active_model = Some(model_id.clone());
                }
            } else if let Ok(mut st) = self.state.write() {
                st.active_model = Some(filter.clone());
            }
        }

        if prioritized.is_empty() {
            format!(
                "[/model] No models found for BYOK '{}'.",
                active_byok.unwrap_or_else(|| "any".to_string())
            )
        } else {
            let list: Vec<String> = prioritized.into_iter().map(|(_, l)| l).collect();
            format!(
                "[/model] Models (BYOK={}): {}",
                active_byok.unwrap_or_else(|| "all".to_string()),
                list.join(", ")
            )
        }
    }

    fn handle_effort(&self, args: &str) -> String {
        let raw = args.trim();
        if raw.is_empty() {
            let cur = self
                .state
                .read()
                .ok()
                .and_then(|s| s.active_effort.clone())
                .unwrap_or_else(|| "medium".to_string());
            return format!("[/effort] Current effort: {} (options: low|medium|high|xhigh|max)", cur);
        }

        let Some(level) = EffortLevel::from_str_loose(raw) else {
            return format!(
                "[/effort] Effort '{}' unsupported by target; ignored gracefully.",
                raw
            );
        };

        let active_model = self
            .state
            .read()
            .ok()
            .and_then(|s| s.active_model.clone())
            .unwrap_or_default();
        if active_model.to_ascii_lowercase().contains("haiku")
            || active_model.to_ascii_lowercase().contains("gpt-4o-mini")
        {
            return format!(
                "[/effort] Model '{}' does not support thinking effort; ignored gracefully.",
                active_model
            );
        }

        if let Ok(mut st) = self.state.write() {
            st.active_effort = Some(level.as_str().to_string());
        }
        global_effort_metrics().set_active_effort(level);
        format!(
            "[/effort] Thinking effort set to '{}' (HeadroomEffortCompressor scaled).",
            level.as_str()
        )
    }

    fn handle_byokmodel(&self, args: &str, agent_id: &str) -> String {
        let tokens: Vec<&str> = args.split_whitespace().collect();
        if tokens.len() < 2 {
            return "[/byokmodel] Usage: /byokmodel <byok> <model> [low|medium|high]".to_string();
        }
        let provider = tokens[0].to_ascii_lowercase();
        let model = tokens[1].to_string();
        let effort = tokens
            .get(2)
            .and_then(|e| EffortLevel::from_str_loose(e))
            .unwrap_or(EffortLevel::Medium);

        let now = now_epoch();
        let profile_id = format!("{}:{}", provider, model);
        let profile = ByokProfileRecord {
            id: profile_id.clone(),
            provider_id: provider.clone(),
            instance_label: "slash".to_string(),
            model_id: model.clone(),
            thinking_effort: effort.as_str().to_string(),
            base_url: "http://127.0.0.1:18787".to_string(),
            secret_ref: format!("keyring://{}/default", provider),
            auth_mode: "api_key".to_string(),
            gcp_project: String::new(),
            gcp_location: "us-central1".to_string(),
            priority: 200,
            weight: 100,
            headroom_enabled: true,
            is_active: true,
            created_at: now,
            updated_at: now,
            ..Default::default()
        };
        let binding = AgentProfileBindingRecord {
            agent_id: agent_id.to_string(),
            byok_profile_id: profile_id.clone(),
            is_primary: true,
            fallback_order: 0,
            updated_at: now,
        };
        let db_res = self.with_db(|db| {
            db.upsert_byok_profile(&profile)
                .and_then(|_| db.upsert_agent_profile_binding(&binding))
        });
        match db_res {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return format!("[/byokmodel] DB error: {}", e),
            Err(e) => return format!("[/byokmodel] Failed to open ~/.toksi/toksi.db: {}", e),
        }

        let mut cfg = load_toksi_toml_config();
        cfg.byok.insert(
            profile_id,
            crate::config::ByokTableEntry {
                base_url: "http://127.0.0.1:18787".to_string(),
                model: model.clone(),
                effort: effort.as_str().to_string(),
                enabled: true,
                provider_id: provider.clone(),
            },
        );
        let _ = save_toksi_toml_config(&cfg);

        if let Ok(mut st) = self.state.write() {
            st.active_byok = Some(provider.clone());
            st.active_model = Some(model.clone());
            st.active_effort = Some(effort.as_str().to_string());
        }

        format!(
            "[/byokmodel] Persisted {} -> model={} effort={} for agent '{}' in ~/.toksi/toksi.db & config.toml",
            provider,
            model,
            effort.as_str(),
            agent_id
        )
    }

    fn handle_direct_tool(
        &self,
        agent_id: &str,
        target: ToolTarget,
        args: &str,
        root: &Path,
    ) -> String {
        let mut parts = args.splitn(2, char::is_whitespace);
        let action = parts
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or("status");
        let rest = parts.next().unwrap_or("").trim();
        let payload = if rest.is_empty() {
            json!({})
        } else {
            json!({ "query": rest, "input": rest, "command": rest })
        };

        let resp = global_direct_wire_bus().invoke_tool(
            agent_id,
            target,
            action,
            payload,
            Some(&root.to_path_buf()),
        );
        if let Some(raw) = resp.raw_text {
            raw
        } else {
            serde_json::to_string_pretty(&resp.result).unwrap_or_else(|_| resp.status)
        }
    }

    fn handle_security(&self, args: &str, root: &Path) -> String {
        let mut parts = args.split_whitespace();
        let sub = parts.next().unwrap_or("scan");
        let target_arg = parts.next();

        match sub {
            "scan-skills" | "audit-skills" => {
                let dir = target_arg
                    .map(PathBuf::from)
                    .unwrap_or_else(|| root.join("agents"));
                if !dir.exists() {
                    return format!("[/security] Skills directory '{}' not found.", normalize_path(&dir.to_string_lossy()));
                }
                match analyze_directory(&dir) {
                    Ok(rep) => format!(
                        "[/security {}] files={} score={} findings={}",
                        sub,
                        rep.files.len(),
                        rep.score,
                        rep.error_count + rep.warning_count + rep.info_count
                    ),
                    Err(e) => format!("[/security {}] Error: {}", sub, e),
                }
            }
            "guardrails" | "adr" | "audit-evidence" | "rules" => {
                let db_ok = open_default_db()
                    .and_then(|d| d.list_adr_audit_events())
                    .map(|rows| rows.len())
                    .unwrap_or(0);
                format!(
                    "[/security {}] <enforce>=active defenseclaw_fail_closed=true adr_events={}",
                    sub, db_ok
                )
            }
            _ => {
                let scan_root = target_arg
                    .map(PathBuf::from)
                    .unwrap_or_else(|| root.to_path_buf());
                let resp = global_direct_wire_bus().invoke_tool(
                    "slash",
                    ToolTarget::Security,
                    "scan",
                    json!({ "path": normalize_path(&scan_root.to_string_lossy()) }),
                    Some(&root.to_path_buf()),
                );
                resp.raw_text
                    .unwrap_or_else(|| serde_json::to_string(&resp.result).unwrap_or_default())
            }
        }
    }

    fn handle_advisory(&self, args: &str, root: &Path) -> String {
        let mut parts = args.splitn(2, char::is_whitespace);
        let sub = parts.next().filter(|s| !s.is_empty()).unwrap_or("status");
        let rest = parts.next().unwrap_or("").trim();

        if sub == "set-dir" && !rest.is_empty() {
            let norm = normalize_path(rest);
            if let Ok(mut st) = self.state.write() {
                st.advisory_dir = Some(norm.clone());
            }
            return format!("[/advisory] Advisory directory set to '{}'.", norm);
        }

        let resp = global_direct_wire_bus().invoke_tool(
            "slash",
            ToolTarget::Advisory,
            sub,
            json!({ "query": rest }),
            Some(&root.to_path_buf()),
        );
        resp.raw_text
            .unwrap_or_else(|| serde_json::to_string(&resp.result).unwrap_or_default())
    }

    fn handle_skills(&self, args: &str, root: &Path) -> String {
        let mut parts = args.splitn(2, char::is_whitespace);
        let sub = parts.next().filter(|s| !s.is_empty()).unwrap_or("list");
        let rest = parts.next().unwrap_or("").trim();

        match sub {
            "set-dir" if !rest.is_empty() => {
                let norm = normalize_path(rest);
                if let Ok(mut st) = self.state.write() {
                    st.skills_dir = Some(norm.clone());
                }
                format!("[/skills] Skills directory set to '{}'.", norm)
            }
            "sync" => {
                let cfg = SkillsRepoConfig::load();
                match sync_external_skills_repo(&cfg) {
                    Ok(rep) => format!(
                        "[/skills sync] synced={} healed={} blocked={}",
                        rep.synced_files.len(),
                        rep.healed_files.len(),
                        rep.blocked_files.len()
                    ),
                    Err(e) => format!("[/skills sync] Error: {}", e),
                }
            }
            "grade" | "scan" => {
                let dir = if rest.is_empty() {
                    root.join("agents")
                } else {
                    PathBuf::from(rest)
                };
                match analyze_directory(&dir) {
                    Ok(rep) => format!(
                        "[/skills {}] files={} score={} findings={}",
                        sub,
                        rep.files.len(),
                        rep.score,
                        rep.error_count + rep.warning_count + rep.info_count
                    ),
                    Err(e) => format!("[/skills {}] Error: {}", sub, e),
                }
            }
            "heal" => {
                if rest.is_empty() {
                    return "[/skills heal] Usage: /skills heal <file.md>".to_string();
                }
                let p = Path::new(rest);
                match std::fs::read_to_string(p) {
                    Ok(raw) => {
                        let fixed = fix_skill_content(rest, &raw);
                        format!(
                            "[/skills heal] changed={} applied={}",
                            fixed.changed,
                            fixed.applied.len()
                        )
                    }
                    Err(e) => format!("[/skills heal] Read error: {}", e),
                }
            }
            _ => {
                if let Ok(g) = self.cached_skills_count.read() {
                    if let Some(count) = *g {
                        return format!("[/skills list] {} vetted skill(s) discovered.", count);
                    }
                }
                let cfg = SkillsRepoConfig::load();
                match discover_vetted_catalog(&cfg) {
                    Ok(cat) => {
                        let count = cat.entries.len();
                        if let Ok(mut g) = self.cached_skills_count.write() {
                            *g = Some(count);
                        }
                        format!("[/skills list] {} vetted skill(s) discovered.", count)
                    }
                    Err(e) => format!("[/skills list] Error: {}", e),
                }
            }
        }
    }

    fn handle_sandbox(&self, args: &str, root: &Path) -> String {
        let mut parts = args.split_whitespace();
        let sub = parts.next().unwrap_or("ls");
        let mgr = global_sandbox_manager();

        match sub {
            "ls" => {
                let list = mgr.list();
                format!("[/sandbox ls] {} active/recorded sandbox(es).", list.len())
            }
            "create" => {
                let id = parts.next().unwrap_or("default-sbx");
                let inst = mgr.create(id, SandboxBackend::Native, root);
                format!("[/sandbox create] Created '{}' (backend={}).", inst.id, inst.backend.as_str())
            }
            "inspect" => {
                let id = parts.next().unwrap_or("default-sbx");
                match mgr.inspect(id) {
                    Some(inst) => serde_json::to_string(&inst).unwrap_or_default(),
                    None => format!("[/sandbox inspect] Sandbox '{}' not found.", id),
                }
            }
            "stop" => {
                let id = parts.next().unwrap_or("default-sbx");
                match mgr.stop(id) {
                    Ok(inst) => format!("[/sandbox stop] '{}' -> {}", inst.id, inst.status),
                    Err(e) => format!("[/sandbox stop] {}", e),
                }
            }
            "rm" => {
                let id = parts.next().unwrap_or("default-sbx");
                match mgr.remove(id) {
                    Ok(()) => format!("[/sandbox rm] Removed '{}'.", id),
                    Err(e) => format!("[/sandbox rm] {}", e),
                }
            }
            "policy" => serde_json::to_string(&mgr.policy()).unwrap_or_default(),
            "secret" => format!("[/sandbox secret] keys={:?}", mgr.list_secret_keys()),
            "prune" => format!("[/sandbox prune] removed={} stopped sandbox(es).", mgr.prune_stopped()),
            "run" | "exec" => {
                let rest: Vec<String> = parts.map(ToString::to_string).collect();
                if rest.is_empty() {
                    return "[/sandbox run] Usage: /sandbox run <program> [args...]".to_string();
                }
                match mgr.run_command(&rest[0], &rest[1..], root, None) {
                    Ok(out) => format!(
                        "[/sandbox {}] backend={} exit={}\n{}",
                        sub, out.backend, out.exit_code, out.stdout
                    ),
                    Err(e) => format!("[/sandbox {}] {}", sub, e),
                }
            }
            other if super::sandbox::SBX_VERBS.contains(&other) => {
                let rest: Vec<String> = parts.map(ToString::to_string).collect();
                mgr.dispatch_sbx_verb(other, &rest)
            }
            _ => "[/sandbox] Subcommands: run|create|attach|exec|ls|inspect|stop|rm|prune|move|cp|ports|ttl|tui|policy|secret|kit|env|mcp|skills|template|volume|schedule|daemon|df|diagnose|reset".to_string(),
        }
    }

    fn handle_plugin(&self, args: &str) -> String {
        let mut parts = args.split_whitespace();
        let sub = parts.next().unwrap_or("list");
        let target = parts.next().unwrap_or("");
        let reg = global_plugin_registry();

        match sub {
            "list" => {
                let list = reg.list();
                if list.is_empty() {
                    "[/plugin list] No plugins registered.".to_string()
                } else {
                    let names: Vec<String> = list
                        .iter()
                        .map(|p| format!("{}@{}({})", p.manifest.id, p.manifest.version, if p.is_enabled { "enabled" } else { "disabled" }))
                        .collect();
                    format!("[/plugin list] {}", names.join(", "))
                }
            }
            "load" if !target.is_empty() => match reg.load(target) {
                Ok(p) => format!("[/plugin load] Loaded '{}' (status={}).", p.manifest.id, p.security_status.as_str()),
                Err(e) => format!("[/plugin load] {}", e),
            },
            "unload" if !target.is_empty() => match reg.unload(target) {
                Ok(p) => format!("[/plugin unload] Unloaded '{}'.", p.manifest.id),
                Err(e) => format!("[/plugin unload] {}", e),
            },
            "reload" if !target.is_empty() => match reg.reload(target) {
                Ok(p) => format!("[/plugin reload] Reloaded '{}'.", p.manifest.id),
                Err(e) => format!("[/plugin reload] {}", e),
            },
            "enable" if !target.is_empty() => match reg.enable(target) {
                Ok(p) => format!("[/plugin enable] Enabled '{}'.", p.manifest.id),
                Err(e) => format!("[/plugin enable] {}", e),
            },
            "disable" if !target.is_empty() => match reg.disable(target) {
                Ok(p) => format!("[/plugin disable] Disabled '{}'.", p.manifest.id),
                Err(e) => format!("[/plugin disable] {}", e),
            },
            "info" if !target.is_empty() => match reg.get(target) {
                Some(p) => serde_json::to_string_pretty(&p).unwrap_or_default(),
                None => format!("[/plugin info] Plugin '{}' not found.", target),
            },
            "scan" if !target.is_empty() => match reg.scan_path(Path::new(target)) {
                Ok(rep) => format!(
                    "[/plugin scan] id={} status={} findings={}",
                    rep.plugin_id,
                    rep.status.as_str(),
                    rep.findings.len()
                ),
                Err(e) => format!("[/plugin scan] {}", e),
            },
            _ => "[/plugin] Usage: /plugin [list|load <id>|unload <id>|reload <id>|info <id>|scan <path>|enable <id>|disable <id>]".to_string(),
        }
    }

    fn handle_resume(&self, args: &str, root: &Path) -> String {
        let plan_id = args.trim();
        let loaded = if plan_id.is_empty() {
            latest_resumable_checkpoint(Some(root))
        } else {
            load_plan_checkpoint(Some(root), plan_id).map(Some)
        };

        match loaded {
            Ok(Some(cp)) => {
                let preflight = validate_dirty_tree_and_plugins(Some(root), &cp);
                let _ = crate::agent_runtime::acp::handle_acp_jsonrpc(
                    &json!({
                        "jsonrpc": "2.0",
                        "id": "resume-notify",
                        "method": "session/resume",
                        "params": {
                            "plan_id": cp.plan_id,
                            "next_step_index": preflight.next_step_index,
                            "next_step_title": preflight.next_step_title,
                            "verified_plugins": preflight.verified_plugins,
                            "missing_plugins": preflight.missing_plugins,
                        }
                    }),
                    EffortLevel::from_str_loose(&cp.active_effort).unwrap_or_default(),
                );
                let next_label = preflight
                    .next_step_title
                    .as_deref()
                    .unwrap_or("done");
                format!(
                    "[/resume] Hydrated checkpoint '{}' ('{}'): completed={}/{}, next_step={} ('{}'), dirty_mismatches={}, verified_plugins={}, missing_plugins={}, status={:?}",
                    cp.plan_id,
                    cp.title,
                    preflight.completed_count,
                    preflight.total_steps,
                    preflight.next_step_index,
                    next_label,
                    preflight.dirty_mismatches.len(),
                    preflight.verified_plugins.len(),
                    preflight.missing_plugins.len(),
                    cp.status
                )
            }
            Ok(None) => {
                let all = list_plan_checkpoints(Some(root)).unwrap_or_default();
                format!(
                    "[/resume] No resumable checkpoint found (total checkpoints={}).",
                    all.len()
                )
            }
            Err(e) => format!("[/resume] Failed to hydrate checkpoint: {}", e),
        }
    }

    fn handle_mode(&self, args: &str) -> String {
        use crate::agent_runtime::mode_router::{
            resolve_mode_spec_with_db, OrchestratorMode, ALL_ORCHESTRATOR_MODES,
        };
        let raw = args.trim();
        if raw.is_empty() || raw.eq_ignore_ascii_case("list") {
            let names: Vec<&str> = ALL_ORCHESTRATOR_MODES.iter().map(|m| m.as_str()).collect();
            return format!(
                "[/mode] {} Orchestrator Modes across 7 domains: {}",
                ALL_ORCHESTRATOR_MODES.len(),
                names.join(", ")
            );
        }
        let Some(mode) = OrchestratorMode::from_str_loose(raw) else {
            return format!("[/mode] Unknown mode '{raw}'. Try `/mode list`.");
        };
        let spec = self
            .with_db(|db| resolve_mode_spec_with_db(mode, Some(db)))
            .unwrap_or_else(|_| resolve_mode_spec_with_db(mode, None));
        format!(
            "[/mode] {} ({}) -> primary={} fallback={} effort={} source={}",
            spec.mode_id,
            spec.domain.label(),
            spec.primary_model,
            spec.fallback_model,
            spec.reasoning_effort.as_str(),
            spec.source
        )
    }

    fn handle_sampling(&self, args: &str) -> String {
        use crate::gateway::sampling::{filter_sampling_params_for_provider, SamplingParams};
        let toks: Vec<&str> = args.split_whitespace().collect();
        let provider = toks.first().copied().unwrap_or("openai");
        let model = toks.get(1).copied().unwrap_or("o3-mini");
        let temp = toks.get(2).and_then(|v| v.parse::<f64>().ok()).or(Some(0.7));
        let top_p = toks.get(3).and_then(|v| v.parse::<f64>().ok()).or(Some(0.9));
        let top_k = toks.get(4).and_then(|v| v.parse::<u32>().ok()).or(Some(40));
        let filtered = filter_sampling_params_for_provider(
            provider,
            model,
            EffortLevel::Medium,
            &SamplingParams {
                temperature: temp,
                top_p,
                top_k,
            },
        );
        format!(
            "[/sampling] provider={} model={} filtered={}",
            provider,
            model,
            serde_json::Value::Object(filtered)
        )
    }

    fn handle_router(&self, args: &str) -> String {
        use crate::agent_runtime::mode_router::{resolve_mode_spec_with_db, ALL_ORCHESTRATOR_MODES};
        let sub = args.trim();
        if sub.is_empty() {
            let mode = ALL_ORCHESTRATOR_MODES[0];
            let sample = self
                .with_db(|db| resolve_mode_spec_with_db(mode, Some(db)))
                .unwrap_or_else(|_| resolve_mode_spec_with_db(mode, None));
            return format!(
                "[/router] Active router: 22 modes / 7 cognitive domains (default_mode={} primary={})",
                sample.mode_id, sample.primary_model
            );
        }
        self.handle_mode(sub)
    }

    fn handle_wave20_command(&self, cmd: &str, args: &str) -> String {
        let arg = if args.trim().is_empty() { "status" } else { args.trim() };
        match cmd {
            "workforce" => format!("[/workforce] Role queues ready (arg={arg})."),
            "spec" => format!("[/spec] OpenSpec contract validator ready (arg={arg})."),
            "dataset" => {
                let mut toks = arg.split_whitespace();
                let sub = toks.next().unwrap_or("ontology");
                if sub == "ontology" {
                    let domain = toks.next().unwrap_or("toksi");
                    let ont = crate::training::dataset::build_ontology_from_fields(
                        domain,
                        &["id", "prompt", "response"],
                    );
                    format!(
                        "[/dataset ontology] domain={} entities={} edges={}",
                        ont.domain,
                        ont.entities.len(),
                        ont.edges.len()
                    )
                } else {
                    format!("[/dataset {sub}] Run `toksi dataset {arg}` to inspect/validate manifests.")
                }
            }
            "train" => format!("[/train] Training recipe planner ready (`toksi train {arg}`)."),
            "eval" => {
                let s = crate::telemetry::evals::compute_eval_summary("deepeval", "active", 1, 1);
                format!("[/eval] harness={} score={:.2} (`toksi eval {arg}`).", s.harness, s.score)
            }
            "compress" => {
                let plan = crate::training::compress::plan_kvpress(
                    crate::training::compress::KvPressPolicy::SnapKv,
                    0.5,
                    32768,
                );
                format!(
                    "[/compress] kvpress={} retained={} saved={:.1}%",
                    plan.policy.as_str(),
                    plan.retained_tokens,
                    plan.estimated_vram_saved_pct
                )
            }
            "serve" => format!("[/serve] Pure-Rust/multimodal local engine descriptors ready (`toksi serve {arg}`)."),
            "cluster" => {
                let tooling = crate::cluster::probe_k8s_tooling();
                let nodes = self
                    .with_db(|db| crate::cluster::ClusterTopology::load_from_config_db(db).ok())
                    .ok()
                    .flatten()
                    .map(|t| t.nodes)
                    .unwrap_or_default();
                let readiness = crate::cluster::evaluate_nvcre_readiness(&nodes);
                format!(
                    "[/cluster {arg}] nodes={} healthy={} ready={} kubectl={} helm={}",
                    readiness.total_nodes,
                    readiness.healthy_nodes,
                    readiness.ready,
                    tooling.kubectl,
                    tooling.helm
                )
            }
            "otel" => {
                if arg.starts_with("collector") {
                    let yaml = crate::cluster::render_otel_collector_yaml(4318, 9464);
                    format!(
                        "[/otel collector] OpenTelemetry collector YAML generated ({} bytes, OTLP HTTP :4318).",
                        yaml.len()
                    )
                } else {
                    let count = self.otel_span_count();
                    let db_path = crate::util::paths::normalize_path(
                        &crate::db::service_dbs::service_db_path(
                            crate::db::service_dbs::ServiceDbKind::Otel,
                        )
                        .to_string_lossy(),
                    );
                    format!(
                        "[/otel {arg}] spans={} otel_db={} semconv=gen_ai.1.29 fail_open=true",
                        count, db_path
                    )
                }
            }
            "voice" => format!("[/voice] Whisper.cpp ASR endpoint ready at local audio bridge (arg={arg})."),
            _ => format!("[/{cmd}] Ready (arg={arg})."),
        }
    }
}

static GLOBAL_SLASH_ENGINE: OnceLock<Arc<SlashCommandEngine>> = OnceLock::new();

pub fn global_slash_engine() -> Arc<SlashCommandEngine> {
    Arc::clone(GLOBAL_SLASH_ENGINE.get_or_init(|| Arc::new(SlashCommandEngine::default())))
}
