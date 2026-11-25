// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/tools/ide_plugin_host.rs
// created: 2026-11-25

//! Standardized External 13-IDE Plugin Host Protocol (`toksi ide-bridge` / `:9098/ide/*`).
//!
//! Zero IDE extension binaries or Node/JVM packages are embedded inside `crates/toksi/`.
//! External extensions communicate with this host over JSON-RPC 2.0, ACP, MCP, SSE, HTTP,
//! or DirectWire IPC (`NamedPipe` / `UDS`).

#![deny(unsafe_code)]

use std::fmt;
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::agent_runtime::mode_router::{resolve_mode_spec, OrchestratorMode, ALL_ORCHESTRATOR_MODES};
use crate::agent_runtime::slash::{global_slash_engine, CORE_SLASH_COMMANDS};
use crate::codegraph::kernel::langs::LANGUAGES;
use crate::commands::guard::{CommandGuard, GuardDecision};
use crate::projectmem::plan_checkpoint::list_plan_checkpoints;
use crate::security::semgrep_engine::scan_source_ast;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IdePluginKind {
    VsCode,
    VisualStudio,
    JetBrains,
    AndroidStudio,
    Zed,
    Xcode,
    Eclipse,
    Cursor,
    Windsurf,
    ApacheNetBeans,
    Kiro,
    Qoder,
    Lapce,
}

impl IdePluginKind {
    pub const ALL: &'static [IdePluginKind] = &[
        Self::VsCode,
        Self::VisualStudio,
        Self::JetBrains,
        Self::AndroidStudio,
        Self::Zed,
        Self::Xcode,
        Self::Eclipse,
        Self::Cursor,
        Self::Windsurf,
        Self::ApacheNetBeans,
        Self::Kiro,
        Self::Qoder,
        Self::Lapce,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::VsCode => "vscode",
            Self::VisualStudio => "visual-studio",
            Self::JetBrains => "jetbrains",
            Self::AndroidStudio => "android-studio",
            Self::Zed => "zed",
            Self::Xcode => "xcode",
            Self::Eclipse => "eclipse",
            Self::Cursor => "cursor",
            Self::Windsurf => "windsurf",
            Self::ApacheNetBeans => "apache-netbeans",
            Self::Kiro => "kiro",
            Self::Qoder => "qoder",
            Self::Lapce => "lapce",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::VsCode => "Visual Studio Code",
            Self::VisualStudio => "Visual Studio IDE",
            Self::JetBrains => "JetBrains IDEs",
            Self::AndroidStudio => "Android Studio",
            Self::Zed => "Zed",
            Self::Xcode => "Xcode",
            Self::Eclipse => "Eclipse IDE",
            Self::Cursor => "Cursor",
            Self::Windsurf => "Windsurf",
            Self::ApacheNetBeans => "Apache NetBeans",
            Self::Kiro => "Kiro",
            Self::Qoder => "Qoder",
            Self::Lapce => "Lapce",
        }
    }

    pub const fn packaging_format(self) -> &'static str {
        match self {
            Self::VsCode | Self::Cursor | Self::Windsurf | Self::Kiro | Self::Qoder => "vsix",
            Self::VisualStudio => "vsix-vstudio",
            Self::JetBrains | Self::AndroidStudio => "jetbrains-zip",
            Self::Zed => "zed-wasm-extension",
            Self::Xcode => "xcode-source-editor-extension",
            Self::Eclipse => "eclipse-p2-bundle",
            Self::ApacheNetBeans => "nbm",
            Self::Lapce => "lapce-wasi-volt",
        }
    }

    pub const fn default_transport(self) -> IdeTransportKind {
        match self {
            Self::Zed | Self::Lapce => IdeTransportKind::Stdio,
            _ => IdeTransportKind::Http,
        }
    }
}

impl fmt::Display for IdePluginKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for IdePluginKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key = s.trim().to_ascii_lowercase().replace('_', "-");
        match key.as_str() {
            "vscode" | "vs-code" => Ok(Self::VsCode),
            "visual-studio" | "visualstudio" | "vs" | "vstudio" => Ok(Self::VisualStudio),
            "jetbrains" | "intellij" | "idea" => Ok(Self::JetBrains),
            "android-studio" | "androidstudio" => Ok(Self::AndroidStudio),
            "zed" => Ok(Self::Zed),
            "xcode" => Ok(Self::Xcode),
            "eclipse" => Ok(Self::Eclipse),
            "cursor" => Ok(Self::Cursor),
            "windsurf" => Ok(Self::Windsurf),
            "apache-netbeans" | "netbeans" => Ok(Self::ApacheNetBeans),
            "kiro" => Ok(Self::Kiro),
            "qoder" => Ok(Self::Qoder),
            "lapce" => Ok(Self::Lapce),
            other => Err(format!("Unsupported IDE target '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdeTransportKind {
    Stdio,
    Pipe,
    Uds,
    Sse,
    Http,
}

impl IdeTransportKind {
    pub const ALL: &'static [IdeTransportKind] =
        &[Self::Stdio, Self::Pipe, Self::Uds, Self::Sse, Self::Http];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stdio => "stdio",
            Self::Pipe => "pipe",
            Self::Uds => "uds",
            Self::Sse => "sse",
            Self::Http => "http",
        }
    }
}

impl fmt::Display for IdeTransportKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for IdeTransportKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "stdio" => Ok(Self::Stdio),
            "pipe" | "named-pipe" | "namedpipe" => Ok(Self::Pipe),
            "uds" | "unix" | "socket" => Ok(Self::Uds),
            "sse" => Ok(Self::Sse),
            "http" | "rest" => Ok(Self::Http),
            other => Err(format!("Unsupported IDE bridge transport '{other}'")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdeCapabilityManifest {
    pub ide: String,
    pub display_name: String,
    pub protocol_version: String,
    pub external_package_only: bool,
    pub packaging_format: String,
    pub default_transport: String,
    pub supported_transports: Vec<String>,
    pub protocols: Vec<String>,
    pub endpoints: Vec<String>,
    pub orchestrator_modes_count: usize,
    pub slash_commands_count: usize,
    pub treesitter_languages_count: usize,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct IdePluginHost;

impl IdePluginHost {
    pub fn new() -> Self {
        Self
    }

    pub fn manifest_for(&self, ide: IdePluginKind) -> IdeCapabilityManifest {
        IdeCapabilityManifest {
            ide: ide.as_str().to_string(),
            display_name: ide.display_name().to_string(),
            protocol_version: "1.0".to_string(),
            external_package_only: true,
            packaging_format: ide.packaging_format().to_string(),
            default_transport: ide.default_transport().as_str().to_string(),
            supported_transports: IdeTransportKind::ALL
                .iter()
                .map(|t| t.as_str().to_string())
                .collect(),
            protocols: vec![
                "json-rpc-2.0".to_string(),
                "acp".to_string(),
                "mcp".to_string(),
                "sse".to_string(),
                "direct-wire".to_string(),
            ],
            endpoints: vec![
                "/ide/health".to_string(),
                "/ide/manifests".to_string(),
                "/ide/manifest/<ide>".to_string(),
                "/ide/handshake".to_string(),
                "/ide/rpc".to_string(),
            ],
            orchestrator_modes_count: ALL_ORCHESTRATOR_MODES.len(),
            slash_commands_count: CORE_SLASH_COMMANDS.len(),
            treesitter_languages_count: LANGUAGES.len(),
            capabilities: vec![
                "streaming_chat".to_string(),
                "mode_selection_22".to_string(),
                "inline_diff_patch".to_string(),
                "codegraph_sast_45_langs".to_string(),
                "projectmem_resume".to_string(),
                "byok_model_effort_controls".to_string(),
            ],
        }
    }

    pub fn all_manifests(&self) -> Vec<IdeCapabilityManifest> {
        IdePluginKind::ALL
            .iter()
            .map(|ide| self.manifest_for(*ide))
            .collect()
    }

    pub fn handle_jsonrpc(&self, req: &Value) -> Value {
        let id = req.get("id").cloned().unwrap_or_else(|| json!(1));
        let method = req.get("method").and_then(Value::as_str).unwrap_or("");
        let params = req.get("params").cloned().unwrap_or_else(|| json!({}));

        match method {
            "ide/handshake" | "initialize" => {
                let raw_ide = params.get("ide").and_then(Value::as_str).unwrap_or("vscode");
                match raw_ide.parse::<IdePluginKind>() {
                    Ok(kind) => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "status": "ready",
                            "protocol_version": "1.0",
                            "external_package_only": true,
                            "manifest": self.manifest_for(kind),
                            "protocol_bus_port": crate::config::resolved_ports().protocol_bus,
                        }
                    }),
                    Err(err) => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32602, "message": err }
                    }),
                }
            }
            "ide/modes" => {
                let modes: Vec<Value> = ALL_ORCHESTRATOR_MODES
                    .iter()
                    .map(|m| {
                        let spec = resolve_mode_spec(*m);
                        json!({
                            "mode": m.as_str(),
                            "domain": m.domain().key(),
                            "domain_label": m.domain().label(),
                            "primary_model": spec.primary_model,
                            "fallback_model": spec.fallback_model,
                            "effort": spec.reasoning_effort.as_str(),
                        })
                    })
                    .collect();
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "count": modes.len(), "modes": modes }
                })
            }
            "ide/slash" => {
                let cmd = params.get("command").and_then(Value::as_str).unwrap_or("");
                let actor = params.get("ide").and_then(Value::as_str).unwrap_or("ide-bridge");
                let res = global_slash_engine().execute(cmd, actor);
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "handled": res.handled,
                        "output": res.output,
                    }
                })
            }
            "ide/diagnostics" => {
                let file = params
                    .get("file")
                    .and_then(Value::as_str)
                    .unwrap_or("untitled.rs");
                let lang = params
                    .get("language")
                    .and_then(Value::as_str)
                    .unwrap_or("rust");
                let content = params.get("content").and_then(Value::as_str).unwrap_or("");
                let findings = if content.is_empty() {
                    Vec::new()
                } else {
                    scan_source_ast(file, lang, content)
                };
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "file": file,
                        "language": lang,
                        "treesitter_languages_count": LANGUAGES.len(),
                        "findings": findings,
                    }
                })
            }
            "ide/diff/preview" => {
                let file_path = params
                    .get("file_path")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let diff_text = params
                    .get("diff")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let hunks = diff_text.lines().filter(|l| l.starts_with("@@")).count();
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "file_path": file_path,
                        "hunk_count": hunks,
                        "bytes": diff_text.len(),
                        "ready_to_apply": !file_path.trim().is_empty() && !diff_text.trim().is_empty(),
                    }
                })
            }
            "ide/checkpoints" => {
                let ws = params
                    .get("workspace")
                    .and_then(Value::as_str)
                    .map(Path::new);
                let cps = list_plan_checkpoints(ws).unwrap_or_default();
                let summary: Vec<Value> = cps
                    .into_iter()
                    .map(|c| {
                        json!({
                            "plan_id": c.plan_id,
                            "title": c.title,
                            "status": c.status,
                            "current_step": c.current_step,
                            "total_steps": c.total_steps,
                            "updated_at": c.updated_at,
                        })
                    })
                    .collect();
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "checkpoints": summary }
                })
            }
            "ide/chat/turn" => {
                let prompt = params.get("prompt").and_then(Value::as_str).unwrap_or("");
                let actor = params.get("ide").and_then(Value::as_str).unwrap_or("ide-bridge");
                let raw_mode = params
                    .get("mode")
                    .and_then(Value::as_str)
                    .unwrap_or("write_code_build");
                let mode = OrchestratorMode::from_str_loose(raw_mode)
                    .unwrap_or(OrchestratorMode::WriteCodeBuild);
                let spec = resolve_mode_spec(mode);

                match CommandGuard::check_pre_exec(actor, prompt, None) {
                    GuardDecision::Allow => {}
                    GuardDecision::Deny { reason, .. }
                    | GuardDecision::KillSwitchTriggered { reason, .. } => {
                        return json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": { "code": -32001, "message": reason }
                        });
                    }
                }

                if prompt.trim().starts_with('/') {
                    let slash = global_slash_engine().execute(prompt, actor);
                    return json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "mode": spec.mode_id,
                            "handled_as_slash": slash.handled,
                            "output": slash.output,
                        }
                    });
                }

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "status": "accepted",
                        "mode": spec.mode_id,
                        "primary_model": spec.primary_model,
                        "effort": spec.reasoning_effort.as_str(),
                        "prompt_chars": prompt.len(),
                    }
                })
            }
            other => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {other}")
                }
            }),
        }
    }
}

pub fn handle_ide_http_route(path: &str, method: &str, body: &[u8]) -> Option<(u16, Value)> {
    if !path.starts_with("/ide/") && path != "/ide" {
        return None;
    }
    let host = IdePluginHost::new();
    let m = method.trim().to_ascii_uppercase();

    if path == "/ide/health" && m == "GET" {
        return Some((
            200,
            json!({
                "status": "ok",
                "service": "toksi-ide-plugin-host",
                "ide_count": IdePluginKind::ALL.len(),
                "external_package_only": true,
            }),
        ));
    }

    if path == "/ide/manifests" && m == "GET" {
        let manifests = host.all_manifests();
        return Some((
            200,
            json!({
                "count": manifests.len(),
                "external_package_only": true,
                "ides": manifests,
            }),
        ));
    }

    if let Some(raw_id) = path.strip_prefix("/ide/manifest/")
        && m == "GET"
    {
        return match raw_id.parse::<IdePluginKind>() {
            Ok(kind) => Some((200, json!(host.manifest_for(kind)))),
            Err(err) => Some((404, json!({ "error": err }))),
        };
    }

    if path == "/ide/handshake" && m == "POST" {
        let params: Value = serde_json::from_slice(body).unwrap_or_else(|_| json!({}));
        let rpc = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "ide/handshake",
            "params": params,
        });
        return Some((200, host.handle_jsonrpc(&rpc)));
    }

    if path == "/ide/rpc" && m == "POST" {
        return match serde_json::from_slice::<Value>(body) {
            Ok(rpc) => Some((200, host.handle_jsonrpc(&rpc))),
            Err(e) => Some((
                400,
                json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32700, "message": format!("Parse error: {e}") }
                }),
            )),
        };
    }

    Some((404, json!({ "error": format!("Unknown IDE host route: {m} {path}") })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_13_ide_manifests_external_only() {
        let host = IdePluginHost::new();
        let all = host.all_manifests();
        assert_eq!(all.len(), 13);
        for m in &all {
            assert!(m.external_package_only);
            assert_eq!(m.orchestrator_modes_count, 22);
            assert_eq!(m.slash_commands_count, 29);
            assert_eq!(m.treesitter_languages_count, 45);
        }
    }

    #[test]
    fn test_ide_handshake_and_http_routes() {
        let (code, body) =
            handle_ide_http_route("/ide/manifests", "GET", b"").expect("manifests route");
        assert_eq!(code, 200);
        assert_eq!(body["count"], 13);

        let (hcode, hbody) = handle_ide_http_route(
            "/ide/handshake",
            "POST",
            br#"{"ide":"jetbrains"}"#,
        )
        .expect("handshake route");
        assert_eq!(hcode, 200);
        assert_eq!(hbody["result"]["manifest"]["ide"], "jetbrains");
        assert_eq!(hbody["result"]["external_package_only"], true);
    }
}
