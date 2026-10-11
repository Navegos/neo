use std::fmt;
use crate::util::headroom_proxy::{headroom_proxy_openai_url, headroom_proxy_url};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProxyProtocol {
    AnthropicNative,
    OpenAICompatible,
    GeminiNative,
    #[default]
    None,
}

impl ProxyProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AnthropicNative => "anthropic-native",
            Self::OpenAICompatible => "openai-compatible",
            Self::GeminiNative => "gemini-native",
            Self::None => "none",
        }
    }
}

impl fmt::Display for ProxyProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub const PROXY_PROTOCOL_ANTHROPIC_NATIVE: ProxyProtocol = ProxyProtocol::AnthropicNative;
pub const PROXY_PROTOCOL_OPENAI_COMPATIBLE: ProxyProtocol = ProxyProtocol::OpenAICompatible;
pub const PROXY_PROTOCOL_GEMINI_NATIVE: ProxyProtocol = ProxyProtocol::GeminiNative;
pub const PROXY_PROTOCOL_NONE: ProxyProtocol = ProxyProtocol::None;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProxyWireKind {
    ManagedRoute,
    AdditiveProvider,
    #[default]
    Manual,
}

impl ProxyWireKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ManagedRoute => "managed-route",
            Self::AdditiveProvider => "additive-provider/model",
            Self::Manual => "manual",
        }
    }
}

impl fmt::Display for ProxyWireKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub const PROXY_WIRE_MANAGED_ROUTE: ProxyWireKind = ProxyWireKind::ManagedRoute;
pub const PROXY_WIRE_ADDITIVE_PROVIDER: ProxyWireKind = ProxyWireKind::AdditiveProvider;
pub const PROXY_WIRE_MANUAL: ProxyWireKind = ProxyWireKind::Manual;

pub fn proxy_endpoint_for(id: &str) -> String {
    match id {
        "codex" | "opencode" | "kilo" | "pi" | "droid" | "copilot" | "cline" | "omp" => {
            headroom_proxy_openai_url()
        }
        "claude" | "antigravity" => headroom_proxy_url(),
        _ => String::new(),
    }
}
