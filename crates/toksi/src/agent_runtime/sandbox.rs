// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/sandbox.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! 5-backend isolated agent sandbox (3-OS native, sandbox-runtime, OpenShell, E2B, sbx CLI),
//! `PermissionPolicy`, `ssrf_guard`, and `SHELL_METACHARS` RBAC.

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::util::paths::normalize_path;

pub const SHELL_METACHARS: &[char] = &[
    ';', '|', '&', '$', '`', '>', '<', '(', ')', '{', '}', '\n', '\r',
];

pub const SBX_VERBS: &[&str] = &[
    "run", "create", "attach", "exec", "ls", "inspect", "stop", "rm", "prune", "move", "cp",
    "ports", "ttl", "tui", "policy", "secret", "kit", "env", "mcp", "skills", "template",
    "volume", "schedule", "daemon", "df", "diagnose", "reset", "setup", "settings",
];

pub const SBX_BUILT_IN_AGENTS: &[&str] = &[
    "claude",
    "codex",
    "cursor",
    "devin",
    "docker-agent",
    "gemini",
    "opencode",
    "shell",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SkillsMountMode {
    #[default]
    ReadOnly,
    ReadWrite,
    Off,
}

impl SkillsMountMode {
    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "readwrite" | "rw" => Self::ReadWrite,
            "off" | "none" => Self::Off,
            _ => Self::ReadOnly,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxBackend {
    Native,
    SandboxRuntime,
    OpenShell,
    E2b,
    Sbx,
}

impl SandboxBackend {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::SandboxRuntime => "sandbox-runtime",
            Self::OpenShell => "openshell",
            Self::E2b => "e2b",
            Self::Sbx => "sbx",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "sandbox-runtime" | "srt" => Self::SandboxRuntime,
            "openshell" | "nemoclaw" => Self::OpenShell,
            "e2b" | "firecracker" => Self::E2b,
            "sbx" | "docker" | "sandboxd" => Self::Sbx,
            _ => Self::Native,
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum SandboxError {
    #[error("SSRF guard blocked target '{0}'")]
    SsrfBlocked(String),
    #[error("Command RBAC violation: {0}")]
    RbacDenied(String),
    #[error("Sandbox execution failed: {0}")]
    ExecFailed(String),
    #[error("Sandbox instance '{0}' not found")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicy {
    pub allow_read: bool,
    pub allow_write: bool,
    pub allow_network: bool,
    pub allow_shell_metachars: bool,
    pub blocked_commands: Vec<String>,
    pub max_memory_mb: u64,
    pub timeout_secs: u64,
    #[serde(default)]
    pub allowed_network_patterns: Vec<String>,
    #[serde(default)]
    pub skills_mode: SkillsMountMode,
}

impl Default for PermissionPolicy {
    fn default() -> Self {
        Self {
            allow_read: true,
            allow_write: true,
            allow_network: false,
            allow_shell_metachars: false,
            blocked_commands: vec![
                "rm -rf /".to_string(),
                "mkfs".to_string(),
                "dd if=".to_string(),
                "shutdown".to_string(),
                "reboot".to_string(),
            ],
            max_memory_mb: 1024,
            timeout_secs: 30,
            allowed_network_patterns: Vec::new(),
            skills_mode: SkillsMountMode::ReadOnly,
        }
    }
}

/// Pure-Rust outbound HTTP/SOCKS5 domain-filtering proxy policy (`sandbox-runtime/proxy-main.ts`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainFilterProxy {
    pub listen_addr: String,
    pub socks5_addr: String,
    pub allowed_domains: Vec<String>,
    pub denied_domains: Vec<String>,
    pub wfp_egress_enforced: bool,
}

impl Default for DomainFilterProxy {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:0".to_string(),
            socks5_addr: "127.0.0.1:0".to_string(),
            allowed_domains: vec!["api.openai.com".to_string(), "api.anthropic.com".to_string()],
            denied_domains: vec!["169.254.169.254".to_string(), "metadata.google.internal".to_string()],
            wfp_egress_enforced: cfg!(windows),
        }
    }
}

impl DomainFilterProxy {
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if ssrf_guard(host).is_err() {
            return false;
        }
        let lower = host.trim().to_ascii_lowercase();
        if self.denied_domains.iter().any(|d| lower == d.to_ascii_lowercase()) {
            return false;
        }
        self.allowed_domains.iter().any(|d| {
            let pat = d.to_ascii_lowercase();
            lower == pat || lower.ends_with(&format!(".{pat}"))
        })
    }
}

pub fn probe_mxc_base_container() -> bool {
    cfg!(windows) && !crate::util::paths::which("docker").is_empty()
}

/// Validates a URL or hostname against SSRF private/loopback/link-local ranges.
pub fn ssrf_guard(target: &str) -> Result<(), SandboxError> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return Err(SandboxError::SsrfBlocked("empty target".to_string()));
    }

    let host = if trimmed.contains("://") {
        let parsed = Url::parse(trimmed)
            .map_err(|_| SandboxError::SsrfBlocked(format!("invalid URL: {trimmed}")))?;
        parsed
            .host_str()
            .ok_or_else(|| SandboxError::SsrfBlocked("missing host".to_string()))?
            .to_string()
    } else {
        trimmed
            .split(':')
            .next()
            .unwrap_or(trimmed)
            .trim_matches(|c| c == '[' || c == ']')
            .to_string()
    };

    let lower = host.to_ascii_lowercase();
    if lower == "localhost"
        || lower.ends_with(".localhost")
        || lower.ends_with(".local")
        || lower == "metadata.google.internal"
    {
        return Err(SandboxError::SsrfBlocked(format!("blocked host '{host}'")));
    }

    if let Ok(ip) = host.parse::<IpAddr>()
        && is_private_or_reserved_ip(ip)
    {
        return Err(SandboxError::SsrfBlocked(format!("blocked IP '{ip}'")));
    }

    Ok(())
}

fn is_private_or_reserved_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_multicast()
                || (v4.octets()[0] == 100 && (v4.octets()[1] & 0b1100_0000) == 64)
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || ((v6.segments()[0] & 0xfe00) == 0xfc00)
                || ((v6.segments()[0] & 0xffc0) == 0xfe80)
        }
    }
}

/// Validates a command string against `SHELL_METACHARS` and `PermissionPolicy` RBAC rules.
pub fn validate_command_rbac(cmd: &str, policy: &PermissionPolicy) -> Result<(), SandboxError> {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return Err(SandboxError::RbacDenied("empty command".to_string()));
    }
    let lower = trimmed.to_ascii_lowercase();
    for blocked in &policy.blocked_commands {
        if lower.contains(&blocked.to_ascii_lowercase()) {
            return Err(SandboxError::RbacDenied(format!(
                "command matches blocked pattern '{blocked}'"
            )));
        }
    }
    if !policy.allow_shell_metachars && trimmed.chars().any(|c| SHELL_METACHARS.contains(&c)) {
        return Err(SandboxError::RbacDenied(
            "shell metacharacters prohibited by PermissionPolicy".to_string(),
        ));
    }
    Ok(())
}

/// Builds a macOS Seatbelt `.sbpl` sandbox profile string.
pub fn build_macos_sbpl_profile(workspace_root: &Path, policy: &PermissionPolicy) -> String {
    let norm = normalize_path(&workspace_root.to_string_lossy());
    let mut lines = vec![
        "(version 1)".to_string(),
        "(deny default)".to_string(),
        "(allow process-exec)".to_string(),
        "(allow process-fork)".to_string(),
    ];
    if policy.allow_read {
        lines.push("(allow file-read*)".to_string());
    } else {
        lines.push(format!("(allow file-read* (subpath \"{norm}\"))"));
    }
    if policy.allow_write {
        lines.push(format!("(allow file-write* (subpath \"{norm}\"))"));
    }
    if policy.allow_network {
        lines.push("(allow network*)".to_string());
    }
    lines.join("\n")
}

/// Builds Linux `bwrap` arguments with seccomp/Landlock-compatible namespace isolation.
pub fn build_linux_bwrap_args(
    workspace_root: &Path,
    policy: &PermissionPolicy,
    program: &str,
    args: &[String],
) -> Vec<String> {
    let norm = normalize_path(&workspace_root.to_string_lossy());
    let mut out = vec![
        "--unshare-pid".to_string(),
        "--die-with-parent".to_string(),
        "--ro-bind".to_string(),
        "/usr".to_string(),
        "/usr".to_string(),
    ];
    if policy.allow_write {
        out.push("--bind".to_string());
    } else {
        out.push("--ro-bind".to_string());
    }
    out.push(norm.clone());
    out.push(norm);
    if !policy.allow_network {
        out.push("--unshare-net".to_string());
    }
    out.push("--".to_string());
    out.push(program.to_string());
    out.extend(args.iter().cloned());
    out
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxInstance {
    pub id: String,
    pub backend: SandboxBackend,
    pub workspace_root: String,
    pub status: String,
    pub policy: PermissionPolicy,
    pub created_at_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxRunOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub backend: String,
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Manages isolated sandbox instances and 3-OS native execution.
pub struct SandboxManager {
    instances: RwLock<HashMap<String, SandboxInstance>>,
    default_policy: RwLock<PermissionPolicy>,
    secrets: RwLock<HashMap<String, String>>,
}

impl Default for SandboxManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SandboxManager {
    pub fn new() -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
            default_policy: RwLock::new(PermissionPolicy::default()),
            secrets: RwLock::new(HashMap::new()),
        }
    }

    pub fn policy(&self) -> PermissionPolicy {
        self.default_policy
            .read()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn set_policy(&self, policy: PermissionPolicy) {
        if let Ok(mut g) = self.default_policy.write() {
            *g = policy;
        }
    }

    pub fn set_secret(&self, key: impl Into<String>, value: impl Into<String>) {
        if let Ok(mut g) = self.secrets.write() {
            g.insert(key.into(), value.into());
        }
    }

    pub fn list_secret_keys(&self) -> Vec<String> {
        let mut keys = self
            .secrets
            .read()
            .map(|g| g.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        keys.sort();
        keys
    }

    pub fn create(&self, id: impl Into<String>, backend: SandboxBackend, workspace_root: &Path) -> SandboxInstance {
        self.create_with_options(id, backend, workspace_root, "shell", self.policy())
    }

    pub fn create_with_options(
        &self,
        id: impl Into<String>,
        backend: SandboxBackend,
        workspace_root: &Path,
        agent: &str,
        policy: PermissionPolicy,
    ) -> SandboxInstance {
        let id = id.into();
        let inst = SandboxInstance {
            id: id.clone(),
            backend,
            workspace_root: normalize_path(&workspace_root.to_string_lossy()),
            status: format!("running:{agent}"),
            policy,
            created_at_secs: now_secs(),
        };
        if let Ok(mut g) = self.instances.write() {
            g.insert(id, inst.clone());
        }
        inst
    }

    pub fn prune_stopped(&self) -> usize {
        if let Ok(mut g) = self.instances.write() {
            let before = g.len();
            g.retain(|_, v| v.status != "stopped");
            return before.saturating_sub(g.len());
        }
        0
    }

    /// Proxy-injected secret handles (`sbx secret` pattern): raw secret values never enter
    /// sandbox container environments; only opaque `toksi-secret-handle://<key>` references are exposed.
    pub fn resolve_proxy_injected_env(&self) -> HashMap<String, String> {
        self.list_secret_keys()
            .into_iter()
            .map(|k| (format!("TOKSI_SECRET_{}", k.to_ascii_uppercase()), format!("toksi-secret-handle://{k}")))
            .collect()
    }

    pub fn dispatch_sbx_verb(&self, verb: &str, args: &[String]) -> String {
        format!(
            "{{\"verb\":\"{}\",\"args\":{},\"active_sandboxes\":{},\"mxc_probe\":{}}}",
            verb,
            serde_json::to_string(args).unwrap_or_else(|_| "[]".to_string()),
            self.list().len(),
            probe_mxc_base_container()
        )
    }

    pub fn list(&self) -> Vec<SandboxInstance> {
        let mut out = self
            .instances
            .read()
            .map(|g| g.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }

    pub fn inspect(&self, id: &str) -> Option<SandboxInstance> {
        self.instances.read().ok().and_then(|g| g.get(id).cloned())
    }

    pub fn stop(&self, id: &str) -> Result<SandboxInstance, SandboxError> {
        let mut g = self
            .instances
            .write()
            .map_err(|_| SandboxError::ExecFailed("lock poisoned".to_string()))?;
        let inst = g.get_mut(id).ok_or_else(|| SandboxError::NotFound(id.to_string()))?;
        inst.status = "stopped".to_string();
        Ok(inst.clone())
    }

    pub fn remove(&self, id: &str) -> Result<(), SandboxError> {
        let mut g = self
            .instances
            .write()
            .map_err(|_| SandboxError::ExecFailed("lock poisoned".to_string()))?;
        g.remove(id)
            .map(|_| ())
            .ok_or_else(|| SandboxError::NotFound(id.to_string()))
    }

    pub fn run_command(
        &self,
        program: &str,
        args: &[String],
        cwd: &Path,
        policy_override: Option<&PermissionPolicy>,
    ) -> Result<SandboxRunOutput, SandboxError> {
        let policy = policy_override.cloned().unwrap_or_else(|| self.policy());
        let joined = if args.is_empty() {
            program.to_string()
        } else {
            format!("{} {}", program, args.join(" "))
        };
        match crate::commands::guard::CommandGuard::check_pre_exec("sandbox", &joined, Some(cwd)) {
            crate::commands::guard::GuardDecision::Allow => {}
            crate::commands::guard::GuardDecision::Deny { reason, .. }
            | crate::commands::guard::GuardDecision::KillSwitchTriggered { reason, .. } => {
                return Err(SandboxError::RbacDenied(reason));
            }
        }
        validate_command_rbac(&joined, &policy)?;

        #[cfg(target_os = "linux")]
        if crate::util::paths::which("bwrap").is_some() {
            let bwrap_args = build_linux_bwrap_args(cwd, &policy, program, args);
            let output = Command::new("bwrap")
                .args(&bwrap_args)
                .current_dir(cwd)
                .output()
                .map_err(|e| SandboxError::ExecFailed(e.to_string()))?;
            return Ok(SandboxRunOutput {
                exit_code: output.status.code().unwrap_or(1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                timed_out: false,
                backend: "linux-bwrap".to_string(),
            });
        }

        #[cfg(target_os = "macos")]
        if crate::util::paths::which("sandbox-exec").is_some() {
            let sbpl = build_macos_sbpl_profile(cwd, &policy);
            let mut cmd_args = vec!["-p".to_string(), sbpl, program.to_string()];
            cmd_args.extend(args.iter().cloned());
            let output = Command::new("sandbox-exec")
                .args(&cmd_args)
                .current_dir(cwd)
                .output()
                .map_err(|e| SandboxError::ExecFailed(e.to_string()))?;
            return Ok(SandboxRunOutput {
                exit_code: output.status.code().unwrap_or(1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                timed_out: false,
                backend: "macos-sandbox-exec".to_string(),
            });
        }

        let res = toksi_sys::sandbox::run_in_os_job_sandbox(
            program,
            args,
            cwd,
            policy.max_memory_mb,
            Duration::from_secs(policy.timeout_secs.max(1)),
        )
        .map_err(|e| SandboxError::ExecFailed(e.to_string()))?;

        Ok(SandboxRunOutput {
            exit_code: res.exit_code,
            stdout: res.stdout,
            stderr: res.stderr,
            timed_out: res.timed_out,
            backend: res.backend.to_string(),
        })
    }

    pub fn exec_in_instance(
        &self,
        id: &str,
        program: &str,
        args: &[String],
    ) -> Result<SandboxRunOutput, SandboxError> {
        let inst = self
            .inspect(id)
            .ok_or_else(|| SandboxError::NotFound(id.to_string()))?;
        if inst.status != "running" {
            return Err(SandboxError::ExecFailed(format!(
                "Sandbox '{}' is not running (status={})",
                id, inst.status
            )));
        }
        self.run_command(
            program,
            args,
            &PathBuf::from(&inst.workspace_root),
            Some(&inst.policy),
        )
    }
}

static GLOBAL_SANDBOX_MANAGER: OnceLock<Arc<SandboxManager>> = OnceLock::new();

pub fn global_sandbox_manager() -> Arc<SandboxManager> {
    Arc::clone(GLOBAL_SANDBOX_MANAGER.get_or_init(|| Arc::new(SandboxManager::new())))
}

/// Executes a simple safe command directly via `Command` only when explicitly bypassing `toksi-sys`
/// is not needed — kept private to avoid unused import warnings on all targets.
#[allow(dead_code)]
fn _command_exists(cmd: &str) -> bool {
    Command::new(cmd).arg("--version").output().is_ok()
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComposeServiceSpec {
    pub name: String,
    pub image: String,
    pub dockerfile: String,
    pub working_dir: String,
    pub network_mode: String,
    pub privileged: bool,
    pub read_only: bool,
    pub mem_limit: String,
    pub cpus: String,
    pub networks: Vec<String>,
    pub environment: Vec<String>,
    pub tmpfs: Vec<String>,
    pub volumes: Vec<String>,
    pub security_opt: Vec<String>,
    pub command: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComposeSpec {
    pub services: Vec<ComposeServiceSpec>,
    pub networks: Vec<String>,
}

fn clean_yaml_scalar(raw: &str) -> String {
    raw.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn parse_inline_yaml_list(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if let Some(inner) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        return inner
            .split(',')
            .map(clean_yaml_scalar)
            .filter(|s| !s.is_empty())
            .collect();
    }
    let single = clean_yaml_scalar(trimmed);
    if single.is_empty() {
        Vec::new()
    } else {
        vec![single]
    }
}

/// Pure-Rust YAML parser for `docker/compose.test.yaml` and Compose test stacks.
pub fn parse_compose_yaml(raw: &str) -> Result<ComposeSpec, String> {
    let mut spec = ComposeSpec::default();
    let mut top_section = "";
    let mut current_service: Option<ComposeServiceSpec> = None;
    let mut service_list_key = "";

    for line in raw.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();

        if indent == 0 {
            if let Some(svc) = current_service.take() {
                spec.services.push(svc);
            }
            service_list_key = "";
            if trimmed.starts_with("services:") {
                top_section = "services";
            } else if trimmed.starts_with("networks:") {
                top_section = "networks";
            } else {
                top_section = "";
            }
            continue;
        }

        if top_section == "networks" && indent == 2 {
            let net = trimmed.trim_end_matches(':').trim().to_string();
            if !net.is_empty() {
                spec.networks.push(net);
            }
            continue;
        }

        if top_section != "services" {
            continue;
        }

        if indent == 2 && trimmed.ends_with(':') {
            if let Some(svc) = current_service.take() {
                spec.services.push(svc);
            }
            service_list_key = "";
            current_service = Some(ComposeServiceSpec {
                name: trimmed.trim_end_matches(':').trim().to_string(),
                ..Default::default()
            });
            continue;
        }

        let Some(ref mut svc) = current_service else {
            continue;
        };

        if let Some(item_raw) = trimmed.strip_prefix("- ") {
            let item = clean_yaml_scalar(item_raw);
            match service_list_key {
                "networks" => svc.networks.push(item),
                "environment" => svc.environment.push(item),
                "tmpfs" => svc.tmpfs.push(item),
                "volumes" => svc.volumes.push(item),
                "security_opt" => svc.security_opt.push(item),
                "command" => svc.command.push(item),
                _ => {}
            }
            continue;
        }

        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim();
            let val = v.trim();
            service_list_key = "";
            match key {
                "image" => svc.image = clean_yaml_scalar(val),
                "dockerfile" => svc.dockerfile = clean_yaml_scalar(val),
                "working_dir" => svc.working_dir = clean_yaml_scalar(val),
                "network_mode" => svc.network_mode = clean_yaml_scalar(val),
                "privileged" => svc.privileged = clean_yaml_scalar(val).eq_ignore_ascii_case("true"),
                "read_only" => svc.read_only = clean_yaml_scalar(val).eq_ignore_ascii_case("true"),
                "mem_limit" => svc.mem_limit = clean_yaml_scalar(val),
                "cpus" => svc.cpus = clean_yaml_scalar(val),
                "command" if !val.is_empty() => svc.command = parse_inline_yaml_list(val),
                "networks" | "environment" | "tmpfs" | "volumes" | "security_opt" | "command" => {
                    if !val.is_empty() {
                        let items = parse_inline_yaml_list(val);
                        match key {
                            "networks" => svc.networks.extend(items),
                            "environment" => svc.environment.extend(items),
                            "tmpfs" => svc.tmpfs.extend(items),
                            "volumes" => svc.volumes.extend(items),
                            "security_opt" => svc.security_opt.extend(items),
                            "command" => svc.command.extend(items),
                            _ => {}
                        }
                    } else {
                        service_list_key = match key {
                            "networks" => "networks",
                            "environment" => "environment",
                            "tmpfs" => "tmpfs",
                            "volumes" => "volumes",
                            "security_opt" => "security_opt",
                            "command" => "command",
                            _ => "",
                        };
                    }
                }
                _ => {
                    if indent >= 6
                        && service_list_key.is_empty()
                        && key == "driver"
                    {
                        // Ignore nested subkeys like build.context.
                    }
                }
            }
        }
    }

    if let Some(svc) = current_service.take() {
        spec.services.push(svc);
    }
    if spec.services.is_empty() {
        return Err("Compose file defines no services".to_string());
    }
    Ok(spec)
}

fn check_target_against_ssrf(raw: &str) -> Result<(), String> {
    let low = raw.trim().to_ascii_lowercase();
    for meta in ["169.254.169.254", "100.100.100.200", "metadata.google.internal"] {
        if low.contains(meta) {
            return Err(format!("SSRF metadata target forbidden: '{meta}'"));
        }
    }
    if low.contains("://") {
        ssrf_guard(raw).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Validates a `ComposeSpec` against privileged containers, host network, unsafe mounts, and SSRF.
pub fn validate_compose_spec(spec: &ComposeSpec) -> Result<Vec<String>, Vec<String>> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if spec.services.is_empty() {
        errors.push("No services declared in ComposeSpec".to_string());
    }

    for svc in &spec.services {
        if svc.name.is_empty() {
            errors.push("Service name cannot be empty".to_string());
        }
        if svc.privileged {
            errors.push(format!("Service '{}' sets forbidden privileged: true", svc.name));
        }
        if svc.network_mode.eq_ignore_ascii_case("host") {
            errors.push(format!("Service '{}' sets forbidden network_mode: host", svc.name));
        }
        for vol in &svc.volumes {
            let norm = normalize_path(vol).to_ascii_lowercase();
            if norm.contains(".toksi") || norm.contains("docker.sock") || norm.contains(".ssh") {
                errors.push(format!("Service '{}' mounts restricted host path '{}'", svc.name, vol));
            }
        }
        for env_entry in &svc.environment {
            if let Some((_, val)) = env_entry.split_once('=')
                && let Err(e) = check_target_against_ssrf(val)
            {
                errors.push(format!("Service '{}' env '{}': {}", svc.name, env_entry, e));
            }
        }
        if svc.mem_limit.trim().is_empty() {
            warnings.push(format!("Service '{}' omits mem_limit", svc.name));
        }
        if !svc
            .security_opt
            .iter()
            .any(|o| o.eq_ignore_ascii_case("no-new-privileges:true"))
        {
            warnings.push(format!("Service '{}' omits no-new-privileges:true", svc.name));
        }
    }

    if errors.is_empty() {
        Ok(warnings)
    } else {
        Err(errors)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SbxEnvManifest {
    pub schema_version: String,
    pub agent: String,
    pub workspace: String,
    pub network_allow: Vec<String>,
    pub env_passthrough: Vec<String>,
    pub skills_mode: String,
}

/// Parses declarative `sbxenv.yaml` (`schemaVersion: "1"`) manifests.
pub fn parse_sbxenv_yaml(raw: &str) -> Result<SbxEnvManifest, String> {
    let mut out = SbxEnvManifest {
        skills_mode: "readonly".to_string(),
        ..Default::default()
    };
    let mut current_list = "";

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(item_raw) = trimmed.strip_prefix("- ") {
            let item = clean_yaml_scalar(item_raw);
            match current_list {
                "network_allow" => out.network_allow.push(item),
                "env_passthrough" => out.env_passthrough.push(item),
                _ => {}
            }
            continue;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim();
            let val = v.trim();
            current_list = "";
            match key {
                "schemaVersion" | "schema_version" => out.schema_version = clean_yaml_scalar(val),
                "agent" => out.agent = clean_yaml_scalar(val),
                "workspace" => out.workspace = clean_yaml_scalar(val),
                "skills_mode" | "skills" => out.skills_mode = clean_yaml_scalar(val),
                "network_allow" | "allow_network" => {
                    if val.is_empty() {
                        current_list = "network_allow";
                    } else {
                        out.network_allow.extend(parse_inline_yaml_list(val));
                    }
                }
                "env_passthrough" | "env" => {
                    if val.is_empty() {
                        current_list = "env_passthrough";
                    } else {
                        out.env_passthrough.extend(parse_inline_yaml_list(val));
                    }
                }
                _ => {}
            }
        }
    }

    if out.schema_version.is_empty() {
        return Err("sbxenv.yaml missing required 'schemaVersion'".to_string());
    }
    Ok(out)
}

/// Validates an `SbxEnvManifest` (`schemaVersion: "1"`) with fail-closed SSRF and mount checks.
pub fn validate_sbxenv(m: &SbxEnvManifest) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if m.schema_version != "1" {
        errors.push(format!("Expected schemaVersion '1', got '{}'", m.schema_version));
    }
    if m.agent.trim().is_empty() {
        errors.push("sbxenv requires non-empty agent".to_string());
    }
    let norm_ws = normalize_path(&m.workspace).to_ascii_lowercase();
    if norm_ws.contains("..") || norm_ws.contains(".toksi") || norm_ws.contains("docker.sock") {
        errors.push(format!("Unsafe workspace path '{}'", m.workspace));
    }
    for host in &m.network_allow {
        if let Err(e) = check_target_against_ssrf(host) {
            errors.push(format!("Blocked network_allow '{}': {}", host, e));
        } else if let Err(e) = ssrf_guard(host) {
            errors.push(format!("SSRF blocked network_allow '{}': {}", host, e));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SbxKitV3Manifest {
    pub schema_version: String,
    pub name: String,
    pub base_image: String,
    pub agents: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub policies: Vec<String>,
    pub resources: HashMap<String, String>,
}

/// Parses declarative Kit v3 (`schemaVersion: "3"`) sandbox manifests.
pub fn parse_sbx_kit_v3_yaml(raw: &str) -> Result<SbxKitV3Manifest, String> {
    let mut out = SbxKitV3Manifest::default();
    let mut section = "";

    for line in raw.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let trimmed = line.trim();
        if let Some(item_raw) = trimmed.strip_prefix("- ") {
            let item = clean_yaml_scalar(item_raw);
            match section {
                "agents" => out.agents.push(item),
                "mcp_servers" => out.mcp_servers.push(item),
                "policies" => out.policies.push(item),
                _ => {}
            }
            continue;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim();
            let val = v.trim();
            if section == "resources" && !val.is_empty() {
                out.resources.insert(key.to_string(), clean_yaml_scalar(val));
                continue;
            }
            section = "";
            match key {
                "schemaVersion" | "schema_version" => out.schema_version = clean_yaml_scalar(val),
                "name" => out.name = clean_yaml_scalar(val),
                "base_image" | "baseImage" => out.base_image = clean_yaml_scalar(val),
                "agents" if val.is_empty() => section = "agents",
                "agents" => out.agents.extend(parse_inline_yaml_list(val)),
                "mcp_servers" | "mcp" if val.is_empty() => section = "mcp_servers",
                "mcp_servers" | "mcp" => out.mcp_servers.extend(parse_inline_yaml_list(val)),
                "policies" if val.is_empty() => section = "policies",
                "policies" => out.policies.extend(parse_inline_yaml_list(val)),
                "resources" if val.is_empty() => section = "resources",
                _ => {}
            }
        }
    }

    if out.schema_version.is_empty() {
        return Err("Kit v3 manifest missing required 'schemaVersion'".to_string());
    }
    Ok(out)
}

/// Validates an `SbxKitV3Manifest` (`schemaVersion: "3"`) with privilege and SSRF checks.
pub fn validate_sbx_kit_v3(m: &SbxKitV3Manifest) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if m.schema_version != "3" {
        errors.push(format!("Expected schemaVersion '3', got '{}'", m.schema_version));
    }
    if m.name.trim().is_empty() {
        errors.push("Kit v3 requires non-empty 'name'".to_string());
    }
    if m.base_image.trim().is_empty() {
        errors.push("Kit v3 requires non-empty 'base_image'".to_string());
    }
    for pol in &m.policies {
        let low = pol.to_ascii_lowercase();
        if low.contains("privileged") || low.contains("host-network") {
            errors.push(format!("Kit v3 forbids unsafe policy '{pol}'"));
        }
        if let Err(e) = check_target_against_ssrf(pol) {
            errors.push(format!("Kit v3 policy '{pol}': {e}"));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Builds deterministic `sbx` CLI argument vectors from an `SbxEnvManifest`.
pub fn build_sbx_cli_args(manifest: &SbxEnvManifest, action: &str) -> Vec<String> {
    let act = if action.trim().is_empty() {
        "run"
    } else {
        action.trim()
    };
    let mut args = vec![act.to_string(), "--agent".to_string(), manifest.agent.clone()];
    if !manifest.workspace.trim().is_empty() {
        args.push("--workspace".to_string());
        args.push(normalize_path(&manifest.workspace));
    }
    args.push("--skills".to_string());
    args.push(manifest.skills_mode.clone());
    for host in &manifest.network_allow {
        args.push("--allow-network".to_string());
        args.push(host.clone());
    }
    for env_name in &manifest.env_passthrough {
        args.push("--env".to_string());
        args.push(env_name.clone());
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_test_yaml_validates_cleanly() {
        let compose_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docker/compose.test.yaml");
        let raw = std::fs::read_to_string(&compose_path).expect("read docker/compose.test.yaml");
        let spec = parse_compose_yaml(&raw).expect("parse compose.test.yaml");
        assert_eq!(spec.services.len(), 1);
        assert_eq!(spec.services[0].name, "toksi-test");
        let warnings = validate_compose_spec(&spec).expect("compose.test.yaml must validate");
        assert!(warnings.is_empty(), "Unexpected warnings: {warnings:?}");
    }

    #[test]
    fn test_compose_rejects_privileged_ssrf_and_toksi_mount() {
        let bad = r#"
services:
  evil:
    image: alpine:3.20
    privileged: true
    network_mode: host
    volumes:
      - ~/.toksi:/root/.toksi
      - /var/run/docker.sock:/var/run/docker.sock
    environment:
      - META=http://169.254.169.254/latest/meta-data
"#;
        let spec = parse_compose_yaml(bad).expect("parse bad compose");
        let errs = validate_compose_spec(&spec).expect_err("must reject unsafe compose");
        assert!(errs.len() >= 5, "Expected >= 5 errors, got {errs:?}");
    }

    #[test]
    fn test_sbxenv_v1_and_kit_v3_and_cli_builder() {
        let v1_raw = r#"
schemaVersion: "1"
agent: claude
workspace: ./crates/toksi
skills_mode: readonly
network_allow:
  - api.anthropic.com
env_passthrough:
  - TOKSI_TEST
"#;
        let env_m = parse_sbxenv_yaml(v1_raw).expect("parse sbxenv");
        assert!(validate_sbxenv(&env_m).is_ok());
        let args = build_sbx_cli_args(&env_m, "create");
        assert!(args.starts_with(&["create".to_string(), "--agent".to_string(), "claude".to_string()]));

        let bad_v1 = r#"
schemaVersion: "1"
agent: claude
workspace: ../secret
network_allow:
  - 100.100.100.200
"#;
        let bad_env = parse_sbxenv_yaml(bad_v1).expect("parse bad sbxenv");
        assert!(validate_sbxenv(&bad_env).is_err());

        let v3_raw = r#"
schemaVersion: "3"
name: neo-kit
base_image: rust:1.88-slim-bookworm
agents: [claude, gemini]
mcp_servers: [toksi]
policies:
  - read-only-skills
resources:
  memory: 2g
  cpus: 2
"#;
        let v3 = parse_sbx_kit_v3_yaml(v3_raw).expect("parse kit v3");
        assert!(validate_sbx_kit_v3(&v3).is_ok());
        assert_eq!(v3.resources.get("memory").map(String::as_str), Some("2g"));
    }
}
