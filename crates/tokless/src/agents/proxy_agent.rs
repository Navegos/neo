use super::proxy_protocol::{ProxyProtocol, ProxyWireKind};

pub struct ProxyAgentSpec {
    pub id: &'static str,
    pub protocol: ProxyProtocol,
    pub wire_kind: ProxyWireKind,
}

pub const PROXY_AGENT_SPECS: &[ProxyAgentSpec] = &[
    ProxyAgentSpec { id: "claude", protocol: ProxyProtocol::AnthropicNative, wire_kind: ProxyWireKind::ManagedRoute },
    ProxyAgentSpec { id: "omp", protocol: ProxyProtocol::OpenAICompatible, wire_kind: ProxyWireKind::AdditiveProvider },
    ProxyAgentSpec { id: "codex", protocol: ProxyProtocol::OpenAICompatible, wire_kind: ProxyWireKind::ManagedRoute },
    ProxyAgentSpec { id: "opencode", protocol: ProxyProtocol::OpenAICompatible, wire_kind: ProxyWireKind::AdditiveProvider },
    ProxyAgentSpec { id: "kilo", protocol: ProxyProtocol::OpenAICompatible, wire_kind: ProxyWireKind::AdditiveProvider },
    ProxyAgentSpec { id: "pi", protocol: ProxyProtocol::OpenAICompatible, wire_kind: ProxyWireKind::AdditiveProvider },
    ProxyAgentSpec { id: "droid", protocol: ProxyProtocol::OpenAICompatible, wire_kind: ProxyWireKind::AdditiveProvider },
    ProxyAgentSpec { id: "copilot", protocol: ProxyProtocol::OpenAICompatible, wire_kind: ProxyWireKind::ManagedRoute },
    ProxyAgentSpec { id: "cline", protocol: ProxyProtocol::OpenAICompatible, wire_kind: ProxyWireKind::ManagedRoute },
    ProxyAgentSpec { id: "cursor", protocol: ProxyProtocol::None, wire_kind: ProxyWireKind::Manual },
    ProxyAgentSpec { id: "antigravity", protocol: ProxyProtocol::GeminiNative, wire_kind: ProxyWireKind::ManagedRoute },
];

pub fn proxy_spec_for(id: &str) -> Option<&'static ProxyAgentSpec> {
    PROXY_AGENT_SPECS.iter().find(|s| s.id == id)
}

pub fn proxy_agent_ids() -> Vec<String> {
    PROXY_AGENT_SPECS.iter().map(|s| s.id.to_string()).collect()
}

pub fn proxy_agent_uses_headroom(id: &str) -> bool {
    id != "grok" && id != "cursor"
}

pub fn proxy_agent_applicable(id: &str) -> bool {
    if id == "copilot" {
        return proxy_agent_wired(id) || !crate::util::paths::which("copilot").is_empty();
    }
    if proxy_agent_wired(id) {
        return true;
    }
    if let Some(agent) = crate::core::get_agent(id) {
        if let Some(ref detect) = agent.detect {
            return detect().installed;
        }
    }
    false
}

pub fn proxy_agent_wired(_id: &str) -> bool {
    false
}

pub fn configure_proxy_agent(_id: &str) -> bool {
    true
}

pub fn remove_proxy_agent(_id: &str) -> bool {
    true
}

pub fn active_proxy_agents() -> Vec<String> {
    proxy_agent_ids().into_iter().filter(|id| proxy_agent_wired(id)).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProxyState {
    ProxyStateManaged,
    ProxyStateManagedBYOK,
    ProxyStateManagedOAuth,
    ProxyStateNativeDirect,
    ProxyStateNativeUnconfigured,
    ProxyStateAbsent,
    ProxyStateForeignBYOK,
    ProxyStateConflict,
    ProxyStateUnreadable,
    #[default]
    ProxyStateUnknown,
}

#[derive(Debug, Clone, Default)]
pub struct ProxyDetection {
    pub state: ProxyState,
    pub detail: String,
}

pub fn detect_proxy(id: &str) -> ProxyDetection {
    if proxy_agent_wired(id) {
        return ProxyDetection {
            state: ProxyState::ProxyStateManagedBYOK,
            detail: "managed route through headroom".into(),
        };
    }
    if proxy_agent_applicable(id) {
        return ProxyDetection {
            state: ProxyState::ProxyStateNativeDirect,
            detail: "native direct route".into(),
        };
    }
    ProxyDetection {
        state: ProxyState::ProxyStateNativeUnconfigured,
        detail: "not configured".into(),
    }
}

pub fn proxy_state_label(state: ProxyState) -> &'static str {
    match state {
        ProxyState::ProxyStateManaged => "managed",
        ProxyState::ProxyStateManagedBYOK => "managed (BYOK)",
        ProxyState::ProxyStateManagedOAuth => "managed (OAuth)",
        ProxyState::ProxyStateNativeDirect => "native direct",
        ProxyState::ProxyStateNativeUnconfigured => "unconfigured",
        ProxyState::ProxyStateAbsent => "absent",
        ProxyState::ProxyStateForeignBYOK => "foreign BYOK",
        ProxyState::ProxyStateConflict => "conflict",
        ProxyState::ProxyStateUnreadable => "unreadable",
        ProxyState::ProxyStateUnknown => "unknown",
    }
}
