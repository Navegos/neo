use crate::core::get_agent;
use crate::util::which;

use super::antigravity::{
    antigravity_proxy_wired, configure_antigravity_proxy, remove_antigravity_proxy,
};
use super::claude::{claude_proxy_wired, configure_claude_proxy, remove_claude_proxy};
use super::cline::{cline_proxy_wired, configure_cline_proxy, remove_cline_proxy};
use super::codex::{codex_proxy_wired, configure_codex_proxy, remove_codex_proxy};
use super::copilot::{
    configure_copilot_all_proxy, configure_copilot_proxy, copilot_proxy_wired,
    remove_copilot_proxy,
};
use super::droid::{configure_droid_proxy, droid_proxy_wired, remove_droid_proxy};
use super::grok::{
    grok_proxy_applicable, grok_proxy_wired, remove_grok_proxy_checked,
};
use super::kilo::{configure_kilo_proxy, kilo_proxy_wired, remove_kilo_proxy};
use super::omp::{configure_omp_proxy, omp_proxy_wired, remove_omp_proxy};
use super::opencode::{
    configure_open_code_proxy, open_code_proxy_satisfied, open_code_proxy_wired,
    remove_open_code_proxy,
};
use super::pi::{configure_pi_proxy, pi_proxy_wired, remove_pi_proxy};

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
        id: "cursor",
        protocol: ProxyProtocol::None,
        wire_kind: ProxyWireKind::Manual,
    },
    ProxyAgentSpec {
        id: "antigravity",
        protocol: ProxyProtocol::GeminiNative,
        wire_kind: ProxyWireKind::ManagedRoute,
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
    id != "grok" && id != "cursor"
}

pub fn configure_proxy_agent_with_path(id: &str) -> (bool, String) {
    match id {
        "claude" => configure_claude_proxy(),
        "omp" => configure_omp_proxy(),
        "codex" => configure_codex_proxy(),
        "opencode" => configure_open_code_proxy(),
        "kilo" => configure_kilo_proxy(),
        "pi" => configure_pi_proxy(),
        "droid" => configure_droid_proxy(),
        "copilot" => {
            let changed = configure_copilot_all_proxy();
            let (_, file) = configure_copilot_proxy();
            (changed, file)
        }
        "cline" => configure_cline_proxy(),
        "antigravity" => configure_antigravity_proxy(),
        _ => (false, String::new()),
    }
}

pub fn configure_proxy_agent(id: &str) -> bool {
    if id == "grok" || id == "cursor" {
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
        "kilo" => remove_kilo_proxy(),
        "pi" => remove_pi_proxy(),
        "droid" => remove_droid_proxy(),
        "copilot" => remove_copilot_proxy(),
        "cline" => remove_cline_proxy(),
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
        "kilo" => kilo_proxy_wired(),
        "pi" => pi_proxy_wired(),
        "droid" => droid_proxy_wired(),
        "copilot" => copilot_proxy_wired(),
        "cline" => cline_proxy_wired(),
        "grok" => grok_proxy_wired(),
        "antigravity" => antigravity_proxy_wired(),
        _ => false,
    }
}
