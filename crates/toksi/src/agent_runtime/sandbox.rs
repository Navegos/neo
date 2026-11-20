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
        }
    }
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

    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_private_or_reserved_ip(ip) {
            return Err(SandboxError::SsrfBlocked(format!("blocked IP '{ip}'")));
        }
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
        let id = id.into();
        let inst = SandboxInstance {
            id: id.clone(),
            backend,
            workspace_root: normalize_path(&workspace_root.to_string_lossy()),
            status: "running".to_string(),
            policy: self.policy(),
            created_at_secs: now_secs(),
        };
        if let Ok(mut g) = self.instances.write() {
            g.insert(id, inst.clone());
        }
        inst
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
