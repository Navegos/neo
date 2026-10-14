// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/proxy_detection.rs
// created: 2026-10-11
// lastModified: 2026-10-14

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::util::{
    claude_code_paths, codex_paths_resolved, has_block, has_jsonc_comments, home,
    opencode_paths_resolved, stringify_json,
};

use super::antigravity::{
    antigravity_env_file, antigravity_ide_applicable, antigravity_ide_compatible,
    antigravity_proxy_wired, antigravity_shim_compatible,
};
use super::cline::detect_cline_proxy;
use super::copilot::detect_copilot_proxy;
use super::droid::droid_proxy_wired;
use super::grok::detect_grok_proxy;
use super::kilo::{kilo_proxy_wired, kilo_transport_plugin_target};
use super::omp::{omp_models_file, omp_proxy_wired, omp_yaml_headroom_managed, omp_yaml_scan};
use super::opencode::{open_code_proxy_provider_block_for, open_code_proxy_wired};
use super::codex::codex_proxy_wired;
use super::pi::{pi_models_file, pi_proxy_provider_entry, pi_proxy_wired, PI_PROXY_PROVIDER};
use super::proxy_agent::PROXY_AGENT_SPECS;
use super::proxy_protocol::{
    proxy_endpoint_for, ProxyProtocol, ProxyWireKind,
};
use super::proxy_provider::{
    json_equal, provider_spec_active, proxy_wire_model, try_parse_jsonc_map, ProxyRouteStashEntry,
};

pub const PROXY_CACHE_POLICY: &str =
    "cache mode preserves provider-prefix cache; semantic response cache disabled";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProxyState {
    ProxyStateManaged,
    ProxyStateManagedBYOK,
    ProxyStateManagedOAuth,
    ProxyStateNativeDirect,
    ProxyStateNativeUnconfigured,
    ProxyStateUnconfigured,
    ProxyStateAbsent,
    ProxyStateForeignBYOK,
    ProxyStateConflict,
    ProxyStateUnreadable,
    #[default]
    ProxyStateUnknown,
}

impl ProxyState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProxyState::ProxyStateManaged => "managed",
            ProxyState::ProxyStateManagedBYOK => "managed-byok",
            ProxyState::ProxyStateManagedOAuth => "managed-oauth",
            ProxyState::ProxyStateNativeDirect => "native-direct",
            ProxyState::ProxyStateNativeUnconfigured | ProxyState::ProxyStateUnconfigured => {
                "unconfigured"
            }
            ProxyState::ProxyStateAbsent => "absent",
            ProxyState::ProxyStateForeignBYOK => "foreign-byok",
            ProxyState::ProxyStateConflict => "conflict",
            ProxyState::ProxyStateUnreadable => "unreadable",
            ProxyState::ProxyStateUnknown => "unknown",
        }
    }
}

impl std::fmt::Display for ProxyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProxyConfigState {
    #[default]
    Absent,
    NativeUnconfigured,
    Managed,
    ForeignBYOK,
    Conflict,
    Unreadable,
    Unknown,
}

impl ProxyConfigState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProxyConfigState::Absent => "absent",
            ProxyConfigState::NativeUnconfigured => "unconfigured",
            ProxyConfigState::Managed => "managed",
            ProxyConfigState::ForeignBYOK => "foreign-byok",
            ProxyConfigState::Conflict => "conflict",
            ProxyConfigState::Unreadable => "unreadable",
            ProxyConfigState::Unknown => "unknown",
        }
    }

    pub fn to_proxy_state(self) -> ProxyState {
        match self {
            ProxyConfigState::Absent => ProxyState::ProxyStateAbsent,
            ProxyConfigState::NativeUnconfigured => ProxyState::ProxyStateNativeUnconfigured,
            ProxyConfigState::Managed => ProxyState::ProxyStateManaged,
            ProxyConfigState::ForeignBYOK => ProxyState::ProxyStateForeignBYOK,
            ProxyConfigState::Conflict => ProxyState::ProxyStateConflict,
            ProxyConfigState::Unreadable => ProxyState::ProxyStateUnreadable,
            ProxyConfigState::Unknown => ProxyState::ProxyStateUnknown,
        }
    }
}

impl std::fmt::Display for ProxyConfigState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProxyCapability {
    pub id: String,
    pub protocol: ProxyProtocol,
    pub wire_kind: ProxyWireKind,
    pub default_flow: String,
    pub byok_supported: bool,
    pub oauth_supported: bool,
    pub cache_preservation: String,
    pub config_surface: String,
    pub notes: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProxyDetection {
    pub id: String,
    pub capability: ProxyCapability,
    pub state: ProxyState,
    pub detail: String,
    pub default_flow: String,
    pub byok_supported: bool,
    pub oauth_supported: bool,
    pub cache_preservation: String,
    pub config_surface: String,
    pub notes: String,
}

pub fn proxy_state_label(state: ProxyState) -> &'static str {
    state.as_str()
}

fn build_capability(id: &str, protocol: ProxyProtocol, wire_kind: ProxyWireKind) -> ProxyCapability {
    let (default_flow, byok_supported, oauth_supported, config_surface, notes) = match id {
        "claude" => (
            "native-oauth-or-api-key",
            true,
            true,
            "~/.claude/settings.json (env.ANTHROPIC_BASE_URL)",
            "Routes Anthropic Messages API traffic through Headroom while preserving native auth headers.",
        ),
        "omp" => (
            "byok-provider-stash",
            true,
            false,
            "~/.omp/agent/models.yml + ~/.omp/agent/config.yml",
            "Rewrites native BYOK providers through Headroom and adds a managed headroom provider.",
        ),
        "codex" => (
            "openai-compatible-provider",
            true,
            true,
            "~/.codex/config.toml (model_providers.headroom)",
            "Configures model_provider = headroom pointing at the Headroom proxy.",
        ),
        "opencode" => (
            "transport-plugin-or-provider",
            true,
            true,
            "~/.config/opencode/opencode.json + plugins/tokless-headroom.js",
            "Installs a managed fetch-wrapper transport plugin and additive headroom provider.",
        ),
        "kilo" => (
            "transport-plugin",
            true,
            true,
            "~/.config/kilo/opencode.json + plugins/tokless-headroom.js",
            "Installs a managed fetch-wrapper transport plugin preserving provider credentials.",
        ),
        "pi" => (
            "byok-provider-stash",
            true,
            false,
            "~/.pi/agent/models.json",
            "Routes native BYOK providers in models.json through Headroom and adds a headroom provider.",
        ),
        "droid" => (
            "additive-custom-model",
            true,
            false,
            "~/.factory/settings.json (customModels)",
            "Registers a Headroom [tokless] custom model entry.",
        ),
        "grok" => (
            "native-cli-only",
            true,
            true,
            "~/.config/grok/config.toml",
            "Grok owns its OAuth context in its native CLI and does not route through the shared Headroom daemon.",
        ),
        "copilot" => (
            "cli-launcher-and-vscode-byok",
            true,
            true,
            "~/.tokless/bin/copilot + VS Code chatLanguageModels.json",
            "Installs a managed CLI launcher and optional VS Code BYOK model provider.",
        ),
        "cline" => (
            "openai-compatible-byok",
            true,
            false,
            "~/.cline/data/globalState.json",
            "Routes existing OpenAI-compatible BYOK configuration through Headroom.",
        ),
        "cursor" => (
            "manual",
            true,
            false,
            "Cursor IDE Override OpenAI Base URL",
            "Manual IDE setting; not automatically mutated by Tokless.",
        ),
        "antigravity" => (
            "cli-launcher-and-ide-endpoint",
            true,
            true,
            "~/.tokless/antigravity.env + ~/.tokless/bin/agy",
            "Configures GEMINI_API_BASE_URL and CLOUDCODE_URL via a managed launcher and IDE settings.",
        ),
        _ => ("unknown", false, false, "", ""),
    };

    ProxyCapability {
        id: id.to_string(),
        protocol,
        wire_kind,
        default_flow: default_flow.to_string(),
        byok_supported,
        oauth_supported,
        cache_preservation: PROXY_CACHE_POLICY.to_string(),
        config_surface: config_surface.to_string(),
        notes: notes.to_string(),
    }
}

pub fn proxy_capabilities() -> Vec<ProxyCapability> {
    let mut out = Vec::with_capacity(PROXY_AGENT_SPECS.len() + 1);
    let mut seen_grok = false;
    for spec in PROXY_AGENT_SPECS {
        if spec.id == "grok" {
            seen_grok = true;
        }
        out.push(build_capability(spec.id, spec.protocol, spec.wire_kind));
    }
    if !seen_grok {
        out.push(build_capability(
            "grok",
            ProxyProtocol::OpenAICompatible,
            ProxyWireKind::AdditiveProvider,
        ));
    }
    out
}

pub fn proxy_capability_for(id: &str) -> Option<ProxyCapability> {
    if id == "grok" {
        return Some(build_capability(
            "grok",
            ProxyProtocol::OpenAICompatible,
            ProxyWireKind::AdditiveProvider,
        ));
    }
    PROXY_AGENT_SPECS
        .iter()
        .find(|s| s.id == id)
        .map(|s| build_capability(s.id, s.protocol, s.wire_kind))
}

pub(crate) fn proxy_detection(id: &str, detail: &str, state: ProxyState) -> ProxyDetection {
    let cap = proxy_capability_for(id).unwrap_or_else(|| ProxyCapability {
        id: id.to_string(),
        protocol: ProxyProtocol::None,
        wire_kind: ProxyWireKind::Manual,
        default_flow: "manual".to_string(),
        byok_supported: false,
        oauth_supported: false,
        cache_preservation: PROXY_CACHE_POLICY.to_string(),
        config_surface: String::new(),
        notes: String::new(),
    });
    ProxyDetection {
        id: cap.id.clone(),
        default_flow: cap.default_flow.clone(),
        byok_supported: cap.byok_supported,
        oauth_supported: cap.oauth_supported,
        cache_preservation: cap.cache_preservation.clone(),
        config_surface: cap.config_surface.clone(),
        notes: cap.notes.clone(),
        capability: cap,
        state,
        detail: detail.to_string(),
    }
}

pub fn detect_proxy(id: &str) -> ProxyDetection {
    let Some(cap) = proxy_capability_for(id) else {
        return proxy_detection(id, "unsupported agent", ProxyState::ProxyStateUnknown);
    };

    match id {
        "claude" => detect_claude_proxy(&cap),
        "omp" => detect_omp_proxy(&cap),
        "codex" => {
            if codex_proxy_wired() {
                return proxy_detection(
                    id,
                    "exact managed endpoint; model_providers.headroom through proxy",
                    ProxyState::ProxyStateManaged,
                );
            }
            let path = codex_paths_resolved().config;
            let raw = match fs::read_to_string(&path) {
                Ok(r) => r,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    return proxy_detection(id, "config absent", ProxyState::ProxyStateAbsent);
                }
                Err(_) => {
                    return proxy_detection(
                        id,
                        "config unreadable",
                        ProxyState::ProxyStateUnreadable,
                    );
                }
            };
            if raw.trim().is_empty() {
                return proxy_detection(id, "config empty", ProxyState::ProxyStateUnreadable);
            }
            if has_block(&raw, "model_providers.headroom") {
                return proxy_detection(
                    id,
                    "headroom provider differs",
                    ProxyState::ProxyStateConflict,
                );
            }
            proxy_detection(
                id,
                "headroom provider not configured",
                ProxyState::ProxyStateNativeUnconfigured,
            )
        }
        "opencode" => {
            if open_code_proxy_wired() {
                return proxy_detection(
                    id,
                    "managed transport plugin preserves provider endpoint and credentials",
                    ProxyState::ProxyStateManaged,
                );
            }
            let spec = provider_spec_active();
            let desired = Value::Object(open_code_proxy_provider_block_for(
                &proxy_endpoint_for(id),
                &spec,
            ));
            detect_provider_proxy(
                &cap,
                Path::new(&opencode_paths_resolved().config),
                "provider",
                &spec.key,
                &desired,
            )
        }
        "kilo" => {
            if kilo_proxy_wired() {
                return proxy_detection(
                    id,
                    "managed transport plugin preserves provider endpoint and credentials",
                    ProxyState::ProxyStateManaged,
                );
            }
            let target = kilo_transport_plugin_target();
            let raw = match fs::read_to_string(&target) {
                Ok(r) => r,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    return proxy_detection(id, "config absent", ProxyState::ProxyStateAbsent);
                }
                Err(_) => {
                    return proxy_detection(
                        id,
                        "config unreadable",
                        ProxyState::ProxyStateUnreadable,
                    );
                }
            };
            if has_jsonc_comments(&raw) {
                return proxy_detection(
                    id,
                    "config has JSONC comments",
                    ProxyState::ProxyStateUnreadable,
                );
            }
            proxy_detection(
                id,
                "transport plugin not configured",
                ProxyState::ProxyStateNativeUnconfigured,
            )
        }
        "pi" => {
            if pi_proxy_wired() {
                return proxy_detection(
                    id,
                    "native BYOK providers routed through headroom",
                    ProxyState::ProxyStateManaged,
                );
            }
            let desired = Value::Object(pi_proxy_provider_entry(&proxy_endpoint_for(id)));
            detect_provider_proxy(
                &cap,
                Path::new(&pi_models_file()),
                "providers",
                PI_PROXY_PROVIDER,
                &desired,
            )
        }
        "droid" => detect_droid_proxy(&cap),
        "grok" => detect_grok_proxy(&cap),
        "copilot" => detect_copilot_proxy(&cap),
        "cline" => detect_cline_proxy(&cap),
        "cursor" => proxy_detection(
            id,
            "manual configuration not observable",
            ProxyState::ProxyStateUnknown,
        ),
        "antigravity" => detect_antigravity_proxy_full(&cap),
        _ => proxy_detection(id, "unsupported agent", ProxyState::ProxyStateUnknown),
    }
}

pub fn detect_all_proxies() -> Vec<ProxyDetection> {
    proxy_capabilities()
        .into_iter()
        .map(|cap| detect_proxy(&cap.id))
        .collect()
}

fn detect_claude_proxy(cap: &ProxyCapability) -> ProxyDetection {
    let path = claude_code_paths().settings;
    let raw = match fs::read_to_string(&path) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return proxy_detection(&cap.id, "settings file absent", ProxyState::ProxyStateAbsent);
        }
        Err(_) => {
            return proxy_detection(
                &cap.id,
                "settings unreadable",
                ProxyState::ProxyStateUnreadable,
            );
        }
    };
    let Some(cfg) = try_parse_jsonc_map(&raw) else {
        return proxy_detection(
            &cap.id,
            "settings unreadable",
            ProxyState::ProxyStateUnreadable,
        );
    };
    let Some(env_val) = cfg.get("env") else {
        return proxy_detection(
            &cap.id,
            "documented endpoint not configured",
            ProxyState::ProxyStateNativeUnconfigured,
        );
    };
    let Some(container) = env_val.as_object() else {
        return proxy_detection(
            &cap.id,
            "documented env slot unreadable",
            ProxyState::ProxyStateUnreadable,
        );
    };
    let Some(v) = container.get("ANTHROPIC_BASE_URL") else {
        return proxy_detection(
            &cap.id,
            "documented endpoint not configured",
            ProxyState::ProxyStateNativeUnconfigured,
        );
    };
    let Some(s) = v.as_str() else {
        return proxy_detection(
            &cap.id,
            "documented endpoint unreadable",
            ProxyState::ProxyStateUnreadable,
        );
    };
    if s.trim().is_empty() {
        return proxy_detection(
            &cap.id,
            "documented endpoint unreadable",
            ProxyState::ProxyStateUnreadable,
        );
    }
    if s == proxy_endpoint_for(&cap.id) {
        return proxy_detection(
            &cap.id,
            "exact managed endpoint",
            ProxyState::ProxyStateManaged,
        );
    }
    proxy_detection(
        &cap.id,
        "documented endpoint differs",
        ProxyState::ProxyStateForeignBYOK,
    )
}

fn detect_omp_proxy(cap: &ProxyCapability) -> ProxyDetection {
    if omp_proxy_wired() {
        return proxy_detection(
            &cap.id,
            "native BYOK providers routed through headroom",
            ProxyState::ProxyStateManaged,
        );
    }
    let path = omp_models_file();
    let raw = match fs::read_to_string(&path) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return proxy_detection(&cap.id, "models file absent", ProxyState::ProxyStateAbsent);
        }
        Err(_) => {
            return proxy_detection(
                &cap.id,
                "models file unreadable",
                ProxyState::ProxyStateUnreadable,
            );
        }
    };
    if raw.trim().is_empty() {
        return proxy_detection(
            &cap.id,
            "models file unreadable",
            ProxyState::ProxyStateUnreadable,
        );
    }
    let scan = omp_yaml_scan(&raw);
    if !scan
        .providers
        .iter()
        .any(|p| p.name == provider_spec_active().key)
    {
        return proxy_detection(
            &cap.id,
            "headroom provider not configured",
            ProxyState::ProxyStateNativeUnconfigured,
        );
    }
    if !omp_yaml_headroom_managed(&raw) {
        return proxy_detection(
            &cap.id,
            "headroom provider differs",
            ProxyState::ProxyStateConflict,
        );
    }
    proxy_detection(
        &cap.id,
        "managed headroom provider; default role state observed separately",
        ProxyState::ProxyStateManaged,
    )
}

fn detect_provider_proxy(
    cap: &ProxyCapability,
    path: &Path,
    container_key: &str,
    reserved: &str,
    desired: &Value,
) -> ProxyDetection {
    let raw = match fs::read_to_string(path) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return proxy_detection(&cap.id, "config file absent", ProxyState::ProxyStateAbsent);
        }
        Err(_) => {
            return proxy_detection(
                &cap.id,
                "config unreadable",
                ProxyState::ProxyStateUnreadable,
            );
        }
    };
    if has_jsonc_comments(&raw) {
        return proxy_detection(
            &cap.id,
            "comment-bearing config not safely inspectable",
            ProxyState::ProxyStateUnreadable,
        );
    }
    let Some(cfg) = try_parse_jsonc_map(&raw) else {
        return proxy_detection(
            &cap.id,
            "config unreadable",
            ProxyState::ProxyStateUnreadable,
        );
    };
    let Some(providers_val) = cfg.get(container_key) else {
        return proxy_detection(
            &cap.id,
            "reserved provider slot not configured",
            ProxyState::ProxyStateNativeUnconfigured,
        );
    };
    let Some(providers) = providers_val.as_object() else {
        return proxy_detection(
            &cap.id,
            "documented provider slot unreadable",
            ProxyState::ProxyStateUnreadable,
        );
    };
    if let Some(existing) = providers.get(reserved) {
        if json_equal(existing, desired) || stringify_json(existing) == stringify_json(desired) {
            return proxy_detection(
                &cap.id,
                "exact reserved provider",
                ProxyState::ProxyStateManaged,
            );
        }
        return proxy_detection(
            &cap.id,
            "reserved provider differs",
            ProxyState::ProxyStateConflict,
        );
    }
    if !providers.is_empty() {
        return proxy_detection(
            &cap.id,
            "foreign provider present; reserved slot absent",
            ProxyState::ProxyStateNativeUnconfigured,
        );
    }
    proxy_detection(
        &cap.id,
        "reserved provider slot not configured",
        ProxyState::ProxyStateNativeUnconfigured,
    )
}

fn detect_droid_proxy(cap: &ProxyCapability) -> ProxyDetection {
    let path = Path::new(&home()).join(".factory").join("settings.json");
    let raw = match fs::read_to_string(&path) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return proxy_detection(&cap.id, "settings file absent", ProxyState::ProxyStateAbsent);
        }
        Err(_) => {
            return proxy_detection(
                &cap.id,
                "settings unreadable",
                ProxyState::ProxyStateUnreadable,
            );
        }
    };
    if has_jsonc_comments(&raw) {
        return proxy_detection(
            &cap.id,
            "comment-bearing config not safely inspectable",
            ProxyState::ProxyStateUnreadable,
        );
    }
    let Some(cfg) = try_parse_jsonc_map(&raw) else {
        return proxy_detection(
            &cap.id,
            "settings unreadable",
            ProxyState::ProxyStateUnreadable,
        );
    };
    let Some(v) = cfg.get("customModels") else {
        return proxy_detection(
            &cap.id,
            "reserved custom model absent",
            ProxyState::ProxyStateNativeUnconfigured,
        );
    };
    let Some(models) = v.as_array() else {
        return proxy_detection(
            &cap.id,
            "customModels slot unreadable",
            ProxyState::ProxyStateUnreadable,
        );
    };
    let wire_model = proxy_wire_model();
    for entry in models {
        let Some(m) = entry.as_object() else {
            continue;
        };
        let model = m.get("model").and_then(|v| v.as_str()).unwrap_or("");
        let display = m.get("displayName").and_then(|v| v.as_str()).unwrap_or("");
        if model == wire_model && display == "Headroom [tokless]" {
            if droid_proxy_wired() {
                return proxy_detection(
                    &cap.id,
                    "exact reserved custom model",
                    ProxyState::ProxyStateManaged,
                );
            }
            return proxy_detection(
                &cap.id,
                "reserved custom model differs",
                ProxyState::ProxyStateConflict,
            );
        }
    }
    proxy_detection(
        &cap.id,
        "reserved custom model absent",
        ProxyState::ProxyStateNativeUnconfigured,
    )
}

fn env_file_val(raw: &str, key: &str) -> String {
    let prefix = format!("{}=", key);
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            return rest.trim().to_string();
        }
    }
    String::new()
}

fn detect_antigravity_proxy_full(cap: &ProxyCapability) -> ProxyDetection {
    let path = antigravity_env_file();
    let raw = match fs::read_to_string(&path) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return proxy_detection(
                &cap.id,
                "automatic route not configured",
                ProxyState::ProxyStateAbsent,
            );
        }
        Err(_) => {
            return proxy_detection(
                &cap.id,
                "env file unreadable",
                ProxyState::ProxyStateUnreadable,
            );
        }
    };
    let value = env_file_val(&raw, "GEMINI_API_BASE_URL");
    let cloud = env_file_val(&raw, "CLOUDCODE_URL");
    if value.is_empty() && cloud.is_empty() {
        return proxy_detection(
            &cap.id,
            "automatic route not configured",
            ProxyState::ProxyStateNativeUnconfigured,
        );
    }
    let endpoint = proxy_endpoint_for(&cap.id);
    if value == endpoint && cloud == endpoint {
        if antigravity_proxy_wired() {
            let mut surfaces = "CLI launcher".to_string();
            if antigravity_ide_applicable() {
                surfaces.push_str(" and IDE setting");
            }
            return proxy_detection(
                &cap.id,
                &format!("managed {}", surfaces),
                ProxyState::ProxyStateManaged,
            );
        }
        if !antigravity_shim_compatible() {
            return proxy_detection(
                &cap.id,
                "existing agy launcher is not Tokless-owned",
                ProxyState::ProxyStateConflict,
            );
        }
        if !antigravity_ide_compatible() {
            return proxy_detection(
                &cap.id,
                "IDE endpoint differs or settings are not safely editable",
                ProxyState::ProxyStateForeignBYOK,
            );
        }
        return proxy_detection(
            &cap.id,
            "automatic route incomplete",
            ProxyState::ProxyStateNativeUnconfigured,
        );
    }
    proxy_detection(
        &cap.id,
        "documented endpoint differs",
        ProxyState::ProxyStateForeignBYOK,
    )
}

// --- Classifiers used by agent modules and unit tests ---

pub fn classify_open_code_jsonc_proxy<F>(
    raw: &str,
    managed_provider: &str,
    endpoint: &str,
    stash: &HashMap<String, ProxyRouteStashEntry>,
    is_plugin_entry: F,
) -> ProxyConfigState
where
    F: Fn(&Value) -> bool,
{
    if raw.trim().is_empty() {
        return ProxyConfigState::Unreadable;
    }
    if has_jsonc_comments(raw) {
        return ProxyConfigState::Unreadable;
    }
    let Some(cfg) = try_parse_jsonc_map(raw) else {
        return ProxyConfigState::Unreadable;
    };
    let has_plugin = cfg
        .get("plugin")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().any(&is_plugin_entry))
        .unwrap_or(false);
    if has_plugin {
        return ProxyConfigState::Managed;
    }
    let Some(providers) = cfg.get("provider").and_then(|v| v.as_object()) else {
        if cfg.contains_key("provider") {
            return ProxyConfigState::Unreadable;
        }
        return ProxyConfigState::NativeUnconfigured;
    };
    if let Some(existing) = providers.get(managed_provider) {
        let spec = provider_spec_active();
        let desired = Value::Object(open_code_proxy_provider_block_for(endpoint, &spec));
        if json_equal(existing, &desired) {
            return ProxyConfigState::Managed;
        }
        return ProxyConfigState::Conflict;
    }
    if !stash.is_empty() {
        return ProxyConfigState::Managed;
    }
    ProxyConfigState::NativeUnconfigured
}

pub fn classify_pi_models_json_proxy(
    raw: &str,
    endpoint: &str,
    stash: &HashMap<String, ProxyRouteStashEntry>,
) -> ProxyConfigState {
    if raw.trim().is_empty() || has_jsonc_comments(raw) {
        return ProxyConfigState::Unreadable;
    }
    let Some(cfg) = try_parse_jsonc_map(raw) else {
        return ProxyConfigState::Unreadable;
    };
    let Some(providers) = cfg.get("providers").and_then(|v| v.as_object()) else {
        if cfg.contains_key("providers") {
            return ProxyConfigState::Unreadable;
        }
        return ProxyConfigState::NativeUnconfigured;
    };
    if let Some(existing) = providers.get(PI_PROXY_PROVIDER) {
        let desired = Value::Object(pi_proxy_provider_entry(endpoint));
        if json_equal(existing, &desired) {
            return ProxyConfigState::Managed;
        }
        return ProxyConfigState::Conflict;
    }
    if !stash.is_empty() {
        return ProxyConfigState::Managed;
    }
    ProxyConfigState::NativeUnconfigured
}

pub fn classify_omp_yaml_proxy(
    raw: &str,
    _endpoint: &str,
    stash: &HashMap<String, ProxyRouteStashEntry>,
) -> ProxyConfigState {
    if raw.trim().is_empty() {
        return ProxyConfigState::Unreadable;
    }
    let scan = omp_yaml_scan(raw);
    if !scan
        .providers
        .iter()
        .any(|p| p.name == provider_spec_active().key)
    {
        if !stash.is_empty() {
            return ProxyConfigState::Managed;
        }
        return ProxyConfigState::NativeUnconfigured;
    }
    if !omp_yaml_headroom_managed(raw) {
        return ProxyConfigState::Conflict;
    }
    ProxyConfigState::Managed
}

pub fn classify_grok_toml_proxy(
    raw: &str,
    endpoint: &str,
    stashed: Option<&ProxyRouteStashEntry>,
) -> ProxyConfigState {
    if raw.trim().is_empty() {
        return ProxyConfigState::NativeUnconfigured;
    }
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            if k.trim() == "base_url" {
                let val = v.trim().trim_matches('"').trim_matches('\'');
                if val == endpoint {
                    return if stashed.is_some() {
                        ProxyConfigState::Managed
                    } else {
                        ProxyConfigState::Conflict
                    };
                }
                if !val.is_empty() {
                    return ProxyConfigState::ForeignBYOK;
                }
            }
        }
    }
    ProxyConfigState::NativeUnconfigured
}

pub fn classify_claude_settings_proxy(raw: &str, endpoint: &str) -> ProxyConfigState {
    let Some(cfg) = try_parse_jsonc_map(raw) else {
        return ProxyConfigState::Unreadable;
    };
    let Some(env_val) = cfg.get("env") else {
        return ProxyConfigState::NativeUnconfigured;
    };
    let Some(env_obj) = env_val.as_object() else {
        return ProxyConfigState::Unreadable;
    };
    let Some(v) = env_obj.get("ANTHROPIC_BASE_URL") else {
        return ProxyConfigState::NativeUnconfigured;
    };
    let Some(s) = v.as_str() else {
        return ProxyConfigState::Unreadable;
    };
    if s.trim().is_empty() {
        return ProxyConfigState::Unreadable;
    }
    if s == endpoint {
        ProxyConfigState::Managed
    } else {
        ProxyConfigState::ForeignBYOK
    }
}

pub fn classify_codex_toml_proxy(raw: &str, endpoint: &str) -> ProxyConfigState {
    if raw.trim().is_empty() {
        return ProxyConfigState::Unreadable;
    }
    if super::codex::codex_proxy_owned(raw, endpoint) {
        return ProxyConfigState::Managed;
    }
    if has_block(raw, "model_providers.headroom") {
        return ProxyConfigState::Conflict;
    }
    ProxyConfigState::NativeUnconfigured
}

pub fn classify_droid_settings_proxy(raw: &str, endpoint: &str) -> ProxyConfigState {
    if has_jsonc_comments(raw) {
        return ProxyConfigState::Unreadable;
    }
    let Some(cfg) = try_parse_jsonc_map(raw) else {
        return ProxyConfigState::Unreadable;
    };
    let Some(v) = cfg.get("customModels") else {
        return ProxyConfigState::NativeUnconfigured;
    };
    let Some(models) = v.as_array() else {
        return ProxyConfigState::Unreadable;
    };
    let wire_model = proxy_wire_model();
    for entry in models {
        let Some(m) = entry.as_object() else {
            continue;
        };
        let model = m.get("model").and_then(|v| v.as_str()).unwrap_or("");
        let display = m.get("displayName").and_then(|v| v.as_str()).unwrap_or("");
        if model == wire_model && display == "Headroom [tokless]" {
            let base_url = m.get("baseUrl").and_then(|v| v.as_str()).unwrap_or("");
            if base_url == endpoint {
                return ProxyConfigState::Managed;
            }
            return ProxyConfigState::Conflict;
        }
    }
    ProxyConfigState::NativeUnconfigured
}

pub fn classify_cline_providers_proxy(raw: &str) -> ProxyConfigState {
    if raw.trim().is_empty() || has_jsonc_comments(raw) {
        return ProxyConfigState::Unreadable;
    }
    if try_parse_jsonc_map(raw).is_none() {
        return ProxyConfigState::Unreadable;
    }
    let cap = build_capability(
        "cline",
        ProxyProtocol::OpenAICompatible,
        ProxyWireKind::ManagedRoute,
    );
    let det = detect_cline_proxy(&cap);
    match det.state {
        ProxyState::ProxyStateManaged
        | ProxyState::ProxyStateManagedBYOK
        | ProxyState::ProxyStateManagedOAuth => ProxyConfigState::Managed,
        ProxyState::ProxyStateAbsent => ProxyConfigState::Absent,
        ProxyState::ProxyStateNativeUnconfigured | ProxyState::ProxyStateUnconfigured => {
            ProxyConfigState::NativeUnconfigured
        }
        ProxyState::ProxyStateForeignBYOK | ProxyState::ProxyStateNativeDirect => {
            ProxyConfigState::ForeignBYOK
        }
        ProxyState::ProxyStateConflict => ProxyConfigState::Conflict,
        _ => ProxyConfigState::Unreadable,
    }
}

pub fn classify_antigravity_proxy() -> (ProxyConfigState, String) {
    let cap = build_capability(
        "antigravity",
        ProxyProtocol::GeminiNative,
        ProxyWireKind::ManagedRoute,
    );
    let det = detect_antigravity_proxy_full(&cap);
    let state = match det.state {
        ProxyState::ProxyStateAbsent => ProxyConfigState::Absent,
        ProxyState::ProxyStateNativeUnconfigured | ProxyState::ProxyStateUnconfigured => {
            ProxyConfigState::NativeUnconfigured
        }
        ProxyState::ProxyStateManaged
        | ProxyState::ProxyStateManagedBYOK
        | ProxyState::ProxyStateManagedOAuth => ProxyConfigState::Managed,
        ProxyState::ProxyStateForeignBYOK => ProxyConfigState::ForeignBYOK,
        ProxyState::ProxyStateConflict => ProxyConfigState::Conflict,
        ProxyState::ProxyStateUnreadable => ProxyConfigState::Unreadable,
        _ => ProxyConfigState::Unknown,
    };
    (state, det.detail)
}
