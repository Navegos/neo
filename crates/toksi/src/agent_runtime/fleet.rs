// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/fleet.rs
// created: 2026-11-21

//! Off-the-back `<toksi-siagent>` runtime synthesis catalog unifying all Section 2.5
//! coding runtimes & orchestration frameworks (`opencode`, `temporal`, `restate`, `langgraph`,
//! `deepagents`, `langchain`, `langchain-litellm`, `langflow`, `cline`, `kilocode`, `alphacode`,
//! `bosun`, `Codewhale`, `zeroclaw`, `agent-framework`, `dify`, `mastra`, `agno`, `SWE-agent`,
//! `OpenHands`, `open-swe`, `EnterpriseBlueprints`, `kimi-code`, `loopwright`, `hermes-webui`,
//! `agents-structures`) over `DirectWireBus` and `:8787`/`:9098`/`:9099`/`:18787`.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeVaultCapability {
    pub vault_id: &'static str,
    pub role: &'static str,
    pub wired_off_the_back: bool,
}

pub const SECTION_2_5_RUNTIME_VAULTS: &[RuntimeVaultCapability] = &[
    RuntimeVaultCapability { vault_id: "opencode", role: "codemode-http-recorder-sqlite", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "temporal", role: "durable-workflow-history", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "restate", role: "durable-execution-journal", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "langgraph", role: "stateful-graph-checkpoint", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "deepagents", role: "hierarchical-deep-planning", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "langchain", role: "chain-tool-abstraction", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "langchain-litellm", role: "unified-provider-adapter", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "langflow", role: "visual-flow-graph-export", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "cline", role: "plan-act-mode-loop", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "kilocode", role: "multi-mode-orchestrator", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "alphacode", role: "alphagrep-fast-search", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "bosun", role: "daemon-fleet-supervisor", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "Codewhale", role: "extension-plugin-host", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "zeroclaw", role: "zero-overhead-agent-core", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "agent-framework", role: "typed-a2a-message-broker", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "dify", role: "workflow-dsl-adapter", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "mastra", role: "ts-workflow-memory-patterns", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "agno", role: "multimodal-team-runtime", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "SWE-agent", role: "aci-repo-patch-harness", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "OpenHands", role: "sandboxed-action-execution", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "open-swe", role: "async-issue-to-pr-graph", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "EnterpriseBlueprints", role: "identity-delegation-hitl", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "kimi-code", role: "cli-wire-acp-session", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "loopwright", role: "yaml-loop-state-machine", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "hermes-webui", role: "web-cockpit-session-ui", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "agents-structures", role: "canonical-dotagents-layouts", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "Zoo-Code", role: "semble-dcg-roomodes-thought-sig", wired_off_the_back: true },
    RuntimeVaultCapability { vault_id: "agent-orchestrator", role: "git-worktree-kanban-fleet-ci-routing", wired_off_the_back: true },
];
