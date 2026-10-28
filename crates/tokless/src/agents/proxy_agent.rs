// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/proxy_agent.rs
// created: 2026-10-05
// lastModified: 2026-10-20

use std::path::Path;

use crate::core::get_agent;
use crate::util::headroom::headroom_installed;
use crate::util::headroom_proxy::{proxy_routing_enabled, proxy_routing_preference_set};
use crate::util::paths;
use crate::util::which;

use super::amp::{amp_rtk_permissions_wired, configure_amp_rtk_permissions_auto, remove_amp_rtk_permissions};
use super::antigravity::{
    antigravity_global_settings_file, antigravity_proxy_wired, configure_antigravity_proxy,
    remove_antigravity_proxy,
};
use super::claude::{claude_proxy_wired, configure_claude_proxy, remove_claude_proxy};
use super::cline::{cline_providers_file, cline_proxy_wired, configure_cline_proxy, remove_cline_proxy};
use super::codex::{codex_proxy_wired, configure_codex_proxy, remove_codex_proxy};
use super::copilot::{
    configure_copilot_all_proxy, configure_copilot_proxy, copilot_config_path,
    copilot_proxy_wired, remove_copilot_proxy,
};
use super::droid::{configure_droid_proxy, droid_proxy_wired, droid_settings_file, remove_droid_proxy};
use super::dsh::{configure_dsh_proxy, dsh_patch_file, dsh_proxy_wired, remove_dsh_proxy};
use super::grok::{
    grok_proxy_applicable, grok_proxy_wired, remove_grok_proxy_checked,
};
use super::kilo::{configure_kilo_proxy, kilo_proxy_wired, remove_kilo_proxy};
use super::kimi::{configure_kimi_proxy, kimi_proxy_wired, remove_kimi_proxy};
use super::omp::{configure_omp_proxy, omp_models_file, omp_proxy_wired, remove_omp_proxy};
use super::opencode::{
    configure_open_code_proxy, configure_open_code_proxy_v2, open_code_config_files,
    open_code_proxy_satisfied, open_code_proxy_v2_wired, open_code_proxy_wired,
    open_code_v2_config_files, remove_open_code_proxy, remove_open_code_proxy_v2,
};
use super::pi::{configure_pi_proxy, pi_models_file, pi_proxy_wired, remove_pi_proxy};
use super::zcode::{configure_zcode_proxy, remove_zcode_proxy, zcode_provider_config_path, zcode_proxy_wired};

pub use super::proxy_detection::{
    detect_all_proxies, detect_proxy, proxy_capabilities, proxy_capability_for, proxy_state_label,
    ProxyCapability, ProxyConfigState, ProxyDetection, ProxyState, PROXY_CACHE_POLICY,
};
pub use super::proxy_protocol::{proxy_endpoint_for, ProxyProtocol, ProxyWireKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProxyAgentSpec {
    pub id: &'static str,
    pub protocol: ProxyProtocol,
    pub wire_kind: ProxyWireKind,
}

pub const PROXY_AGENT_SPECS: &[ProxyAgentSpec] = &[
    ProxyAgentSpec {
        id: "claude",
        protocol: ProxyProtocol::AnthropicNative,
        wire_kind: ProxyWireKind::ManagedRoute,
    },
    ProxyAgentSpec {
        id: "omp",
        protocol: ProxyProtocol::OpenAICompatible,
        wire_kind: ProxyWireKind::AdditiveProvider,
    },
    ProxyAgentSpec {
        id: "codex",
        protocol: ProxyProtocol::OpenAICompatible,
        wire_kind: ProxyWireKind::ManagedRoute,
    },
    ProxyAgentSpec {
        id: "opencode",
        protocol: ProxyProtocol::OpenAICompatible,
        wire_kind: ProxyWireKind::AdditiveProvider,
    },
    ProxyAgentSpec {
        id: "opencode2",
        protocol: ProxyProtocol::OpenAICompatible,
        wire_kind: ProxyWireKind::AdditiveProvider,
    },
    ProxyAgentSpec {
        id: "kilo",
        protocol: ProxyProtocol::OpenAICompatible,
        wire_kind: ProxyWireKind::AdditiveProvider,
    },
    ProxyAgentSpec {
        id: "pi",
        protocol: ProxyProtocol::OpenAICompatible,
        wire_kind: ProxyWireKind::AdditiveProvider,
    },
    ProxyAgentSpec {
        id: "droid",
        protocol: ProxyProtocol::OpenAICompatible,
        wire_kind: ProxyWireKind::AdditiveProvider,
    },
    ProxyAgentSpec {
        id: "copilot",
        protocol: ProxyProtocol::OpenAICompatible,
        wire_kind: ProxyWireKind::ManagedRoute,
    },
    ProxyAgentSpec {
        id: "cline",
        protocol: ProxyProtocol::OpenAICompatible,
        wire_kind: ProxyWireKind::ManagedRoute,
    },
    ProxyAgentSpec {
        id: "kimi",
        protocol: ProxyProtocol::OpenAICompatible,
        wire_kind: ProxyWireKind::AdditiveProvider,
    },
    ProxyAgentSpec {
        id: "zcode",
        protocol: ProxyProtocol::OpenAICompatible,
        wire_kind: ProxyWireKind::AdditiveProvider,
    },
    ProxyAgentSpec {
        id: "dsh",
        protocol: ProxyProtocol::OpenAICompatible,
        wire_kind: ProxyWireKind::AdditiveProvider,
    },
    ProxyAgentSpec {
        id: "amp",
        protocol: ProxyProtocol::OpenAICompatible,
        wire_kind: ProxyWireKind::Manual,
    },
    ProxyAgentSpec {
        id: "cursor",
        protocol: ProxyProtocol::None,
        wire_kind: ProxyWireKind::Manual,
    },
    ProxyAgentSpec {
        id: "antigravity",
        protocol: ProxyProtocol::GeminiNative,
        wire_kind: ProxyWireKind::ManagedRoute,
    },
    ProxyAgentSpec {
        id: "grok",
        protocol: ProxyProtocol::OpenAICompatible,
        wire_kind: ProxyWireKind::AdditiveProvider,
    },
];

pub fn proxy_spec_for(id: &str) -> Option<&'static ProxyAgentSpec> {
    PROXY_AGENT_SPECS.iter().find(|s| s.id == id)
}

pub fn proxy_agent_ids() -> Vec<String> {
    PROXY_AGENT_SPECS.iter().map(|s| s.id.to_string()).collect()
}

pub fn active_proxy_agents() -> Vec<String> {
    PROXY_AGENT_SPECS
        .iter()
        .filter(|s| proxy_agent_uses_headroom(s.id))
        .map(|s| s.id.to_string())
        .collect()
}

pub fn proxy_agent_applicable(id: &str) -> bool {
    if id == "grok" {
        return grok_proxy_applicable();
    }
    if id == "copilot" {
        return copilot_proxy_wired() || !which("copilot").is_empty();
    }
    if proxy_agent_wired(id) {
        return true;
    }
    if let Some(agent) = get_agent(id) {
        if let Some(ref detect) = agent.detect {
            return detect().installed;
        }
    }
    false
}

pub fn proxy_agent_uses_headroom(id: &str) -> bool {
    id != "grok" && id != "cursor" && id != "amp"
}

pub fn configure_proxy_agent_with_path(id: &str) -> (bool, String) {
    match id {
        "claude" => configure_claude_proxy(),
        "omp" => configure_omp_proxy(),
        "codex" => configure_codex_proxy(),
        "opencode" => configure_open_code_proxy(),
        "opencode2" => configure_open_code_proxy_v2(),
        "kilo" => configure_kilo_proxy(),
        "pi" => configure_pi_proxy(),
        "droid" => configure_droid_proxy(),
        "copilot" => {
            let changed = configure_copilot_all_proxy();
            let (_, file) = configure_copilot_proxy();
            (changed, file)
        }
        "cline" => configure_cline_proxy(),
        "kimi" => configure_kimi_proxy(),
        "zcode" => configure_zcode_proxy(),
        "dsh" => configure_dsh_proxy(),
        "amp" => configure_amp_rtk_permissions_auto(),
        "antigravity" => configure_antigravity_proxy(),
        _ => (false, String::new()),
    }
}

pub fn configure_proxy_agent(id: &str) -> bool {
    if id == "grok" || id == "cursor" || id == "amp" {
        return false;
    }
    if proxy_spec_for(id).is_none() {
        return false;
    }
    let _ = configure_proxy_agent_with_path(id);
    if id == "copilot" {
        return copilot_proxy_wired();
    }
    if id == "opencode" {
        return open_code_proxy_satisfied();
    }
    proxy_agent_wired(id)
}

pub fn remove_proxy_agent(id: &str) -> bool {
    remove_proxy_agent_checked(id).unwrap_or(false)
}

pub fn remove_proxy_agent_checked(id: &str) -> Result<bool, String> {
    if id == "grok" {
        return remove_grok_proxy_checked();
    }
    let removed = match id {
        "claude" => remove_claude_proxy(),
        "omp" => remove_omp_proxy(),
        "codex" => remove_codex_proxy(),
        "opencode" => remove_open_code_proxy(),
        "opencode2" => remove_open_code_proxy_v2(),
        "kilo" => remove_kilo_proxy(),
        "pi" => remove_pi_proxy(),
        "droid" => remove_droid_proxy(),
        "copilot" => remove_copilot_proxy(),
        "cline" => remove_cline_proxy(),
        "kimi" => remove_kimi_proxy(),
        "zcode" => remove_zcode_proxy(),
        "dsh" => remove_dsh_proxy(),
        "amp" => remove_amp_rtk_permissions(),
        "antigravity" => remove_antigravity_proxy(),
        _ => false,
    };
    Ok(removed)
}

pub fn proxy_agent_wired(id: &str) -> bool {
    match id {
        "claude" => claude_proxy_wired(),
        "omp" => omp_proxy_wired(),
        "codex" => codex_proxy_wired(),
        "opencode" => open_code_proxy_wired(),
        "opencode2" => open_code_proxy_v2_wired(),
        "kilo" => kilo_proxy_wired(),
        "pi" => pi_proxy_wired(),
        "droid" => droid_proxy_wired(),
        "copilot" => copilot_proxy_wired(),
        "cline" => cline_proxy_wired(),
        "kimi" => kimi_proxy_wired(),
        "zcode" => zcode_proxy_wired(),
        "dsh" => dsh_proxy_wired(),
        "amp" => amp_rtk_permissions_wired(),
        "grok" => grok_proxy_wired(),
        "antigravity" => antigravity_proxy_wired(),
        _ => false,
    }
}

pub fn proxy_agent_config_files(id: &str) -> Vec<String> {
    match id {
        "claude" => vec![paths::claude_code_paths().settings],
        "omp" => vec![omp_models_file()],
        "codex" => vec![paths::codex_paths_resolved().config],
        "opencode" => open_code_config_files(),
        "opencode2" => open_code_v2_config_files(),
        "kilo" => {
            let dir = paths::kilo_paths_resolved().dir;
            vec![
                Path::new(&dir).join("config.json").to_string_lossy().to_string(),
                Path::new(&dir).join("kilo.json").to_string_lossy().to_string(),
                Path::new(&dir).join("kilo.jsonc").to_string_lossy().to_string(),
                Path::new(&dir).join("opencode.json").to_string_lossy().to_string(),
                Path::new(&dir).join("opencode.jsonc").to_string_lossy().to_string(),
            ]
        }
        "pi" => vec![pi_models_file()],
        "droid" => vec![droid_settings_file()],
        "copilot" => vec![copilot_config_path()],
        "cline" => vec![cline_providers_file()],
        "kimi" => vec![paths::kimi_paths_resolved().config],
        "zcode" => vec![zcode_provider_config_path()],
        "dsh" => vec![dsh_patch_file()],
        "amp" => vec![paths::amp_paths_resolved().config],
        "antigravity" => vec![antigravity_global_settings_file()],
        _ => Vec::new(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyReadiness {
    Ready,
    InstallMissing,
    AgentUnwired,
    RoutingDisabled,
    Manual,
    ByokExcluded,
    NotApplicable,
}

pub fn proxy_readiness_for(id: &str) -> ProxyReadiness {
    if let Some(spec) = proxy_spec_for(id)
        && spec.wire_kind == ProxyWireKind::Manual
    {
        return ProxyReadiness::Manual;
    }
    if id == "grok" {
        return ProxyReadiness::ByokExcluded;
    }
    if proxy_spec_for(id).is_none() {
        return ProxyReadiness::NotApplicable;
    }
    if !proxy_agent_applicable(id) {
        return ProxyReadiness::NotApplicable;
    }
    if proxy_routing_preference_set() && !proxy_routing_enabled() {
        return ProxyReadiness::RoutingDisabled;
    }
    if !proxy_agent_uses_headroom(id) {
        return ProxyReadiness::ByokExcluded;
    }
    if !headroom_installed() {
        return ProxyReadiness::InstallMissing;
    }
    if id == "opencode" {
        if !open_code_proxy_satisfied() {
            return ProxyReadiness::AgentUnwired;
        }
        return ProxyReadiness::Ready;
    }
    if !proxy_agent_wired(id) {
        return ProxyReadiness::AgentUnwired;
    }
    ProxyReadiness::Ready
}
