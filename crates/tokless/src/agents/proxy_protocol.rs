use crate::util::headroom_proxy::{headroom_proxy_openai_url, headroom_proxy_url};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyProtocol {
    AnthropicNative,
    OpenAICompatible,
    GeminiNative,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyWireKind {
    ManagedRoute,
    AdditiveProvider,
    Manual,
}

pub fn proxy_endpoint_for(id: &str) -> String {
    match id {
        "codex" | "opencode" | "kilo" | "pi" | "droid" | "copilot" | "cline" | "omp" => {
            headroom_proxy_openai_url()
        }
        "claude" | "antigravity" => headroom_proxy_url(),
        _ => String::new(),
    }
}

