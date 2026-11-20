// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/providers/ecosystem/mod.rs
// created: 2026-10-31
// lastModified: 2026-11-13

use std::collections::HashMap;
use serde_json::{json, Value};
use crate::gateway::providers::{ProviderClient, ProviderId, UnifiedRequest, UnifiedResponse};

pub struct EcosystemProvider {
    pub provider_id: ProviderId,
    pub endpoint_url: String,
}

impl EcosystemProvider {
    pub fn new(provider_id: ProviderId) -> Self {
        Self {
            provider_id,
            endpoint_url: provider_id.default_endpoint().to_string(),
        }
    }
}

impl ProviderClient for EcosystemProvider {
    fn id(&self) -> ProviderId {
        self.provider_id
    }

    fn endpoint(&self) -> String {
        format!("{}/chat/completions", self.endpoint_url.trim_end_matches('/'))
    }

    fn auth_headers(&self, api_key: &str) -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        if self.provider_id == ProviderId::GitHubCopilot {
            h.insert("Editor-Version".to_string(), "vscode/1.96.0".to_string());
            h.insert("Copilot-Integration-Id".to_string(), "vscode-chat".to_string());
        }
        h.insert("content-type".to_string(), "application/json".to_string());
        h
    }

    fn format_request(&self, req: &UnifiedRequest) -> Value {
        json!({
            "model": req.model,
            "messages": req.messages,
            "max_tokens": req.max_tokens.unwrap_or(4096),
        })
    }

    fn parse_response(&self, resp: &Value) -> Result<UnifiedResponse, String> {
        let id = resp.get("id").and_then(Value::as_str).unwrap_or("eco").to_string();
        let model = resp.get("model").and_then(Value::as_str).unwrap_or("copilot").to_string();
        let content = resp
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        let (input_tokens, output_tokens) = crate::gateway::providers::parse_openai_usage(resp);

        Ok(UnifiedResponse {
            id,
            model,
            content,
            role: "assistant".to_string(),
            finish_reason: Some("stop".to_string()),
            input_tokens,
            output_tokens,
            raw: resp.clone(),
        })
    }
}

/// All 12 Category 4 Agentic Service & Cloud Platform Providers (webhooks, PR/review MCP/A2A hooks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category4ServiceId {
    Devin,
    FactoryAI,
    OpenHandsCloud,
    AugmentCode,
    MagicDev,
    PoolsideAI,
    SweepAI,
    CodeRabbit,
    Qodo,
    GritIo,
    Ellipsis,
    GitHubCopilotAgentHQ,
}

#[derive(Debug, Clone, Copy)]
pub struct Category4Connector {
    pub id: Category4ServiceId,
    pub slug: &'static str,
    pub webhook_event: &'static str,
    pub a2a_capability: &'static str,
    pub supports_pr_review: bool,
}

pub const CATEGORY_4_CONNECTORS: [Category4Connector; 12] = [
    Category4Connector { id: Category4ServiceId::Devin, slug: "devin", webhook_event: "devin.session.completed", a2a_capability: "autonomous-task", supports_pr_review: true },
    Category4Connector { id: Category4ServiceId::FactoryAI, slug: "factory-ai", webhook_event: "factory.droid.finished", a2a_capability: "code-droid", supports_pr_review: true },
    Category4Connector { id: Category4ServiceId::OpenHandsCloud, slug: "openhands-cloud", webhook_event: "openhands.action.finished", a2a_capability: "cloud-sandbox", supports_pr_review: true },
    Category4Connector { id: Category4ServiceId::AugmentCode, slug: "augment-code", webhook_event: "augment.context.synced", a2a_capability: "codebase-rag", supports_pr_review: true },
    Category4Connector { id: Category4ServiceId::MagicDev, slug: "magic-dev", webhook_event: "magic.ltm.ready", a2a_capability: "long-context-synth", supports_pr_review: false },
    Category4Connector { id: Category4ServiceId::PoolsideAI, slug: "poolside-ai", webhook_event: "poolside.eval.done", a2a_capability: "rl-code-gen", supports_pr_review: false },
    Category4Connector { id: Category4ServiceId::SweepAI, slug: "sweep-ai", webhook_event: "sweep.pr.created", a2a_capability: "issue-to-pr", supports_pr_review: true },
    Category4Connector { id: Category4ServiceId::CodeRabbit, slug: "coderabbit", webhook_event: "coderabbit.review.completed", a2a_capability: "pr-review-mcp", supports_pr_review: true },
    Category4Connector { id: Category4ServiceId::Qodo, slug: "qodo", webhook_event: "qodo.pr.analyzed", a2a_capability: "test-and-review", supports_pr_review: true },
    Category4Connector { id: Category4ServiceId::GritIo, slug: "grit-io", webhook_event: "grit.migration.applied", a2a_capability: "ast-migration", supports_pr_review: true },
    Category4Connector { id: Category4ServiceId::Ellipsis, slug: "ellipsis", webhook_event: "ellipsis.review.posted", a2a_capability: "pr-review-bot", supports_pr_review: true },
    Category4Connector { id: Category4ServiceId::GitHubCopilotAgentHQ, slug: "copilot-agent-hq", webhook_event: "copilot.agent.completed", a2a_capability: "github-agent-hq", supports_pr_review: true },
];

pub fn find_category4_connector(slug: &str) -> Option<&'static Category4Connector> {
    let norm = slug.trim().to_ascii_lowercase();
    CATEGORY_4_CONNECTORS.iter().find(|c| c.slug == norm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category4_12_connectors_registered() {
        assert_eq!(CATEGORY_4_CONNECTORS.len(), 12);
        assert!(find_category4_connector("coderabbit").map(|c| c.supports_pr_review).unwrap_or(false));
        assert!(find_category4_connector("devin").is_some());
    }
}

