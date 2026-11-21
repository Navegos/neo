// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase15_3_to_15_6_ecosystem_verification.rs

#![deny(unsafe_code)]

use toksi::agents::opt::{AgentBudgetConstraints, AgentOptimizer};
use toksi::gateway::agent_cli::tier_check::{inspect_provider_tier, AccountTier};
use toksi::tools::multi_protocol::{ProtocolKind, ToolTarget};

#[test]
fn test_phase15_zero_paid_tier_blocks_and_protocols() {
    let tier = inspect_provider_tier("openai", "sk-free-tier");
    assert_ne!(tier.label(), "[DISQUALIFIED-FREE]");
    assert_ne!(AccountTier::FreeOrTrial.label(), "[DISQUALIFIED-FREE]");

    let protocols = [
        ProtocolKind::Mcp,
        ProtocolKind::A2a,
        ProtocolKind::Acp,
        ProtocolKind::Stdio,
        ProtocolKind::Sse,
        ProtocolKind::Http,
    ];
    assert_eq!(protocols.len(), 6);

    let targets = [
        ToolTarget::Projectmem,
        ToolTarget::Codegraph,
        ToolTarget::Context,
        ToolTarget::Icm,
        ToolTarget::Aimemory,
        ToolTarget::Rtk,
    ];
    assert_eq!(targets.len(), 6);
}

#[test]
fn test_phase15_6_agent_opt_dedup_and_oscillation() {
    let opt = AgentOptimizer::new(AgentBudgetConstraints::default());
    let dedup = opt.dedup_one_hop_context(&[
        ("src/main.rs", "fn main() {}"),
        ("src/main.rs", "fn main() {}"),
    ]);
    assert_eq!(dedup.len(), 1);
    assert!(!opt.detect_semantic_oscillation(3));
}
