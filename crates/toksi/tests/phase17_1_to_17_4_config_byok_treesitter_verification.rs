// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase17_1_to_17_4_config_byok_treesitter_verification.rs

#![deny(unsafe_code)]

use toksi::codegraph::kernel::langs::{grammar_for, LANGUAGES};
use toksi::config::ToksiTomlConfig;
use toksi::headroom::effort::{EffortLevel, HeadroomEffortCompressor};

#[test]
fn test_phase17_1_config_toml_sections_and_defaults() {
    let cfg = ToksiTomlConfig::default();
    assert_eq!(cfg.ports.headroom_proxy, 8787);
    assert_eq!(cfg.ports.grok_oauth_proxy, 8788);
    assert_eq!(cfg.ports.copilot_vscode_proxy, 8789);
    assert_eq!(cfg.ports.copilot_cli_proxy, 8790);
    assert_eq!(cfg.ports.protocol_bus, 9098);
    assert_eq!(cfg.ports.ai_gateway, 9099);
    assert_eq!(cfg.ports.dashboard, 9100);
    assert_eq!(cfg.ports.byok_gateway, 18787);
    assert!(cfg.test.incremental);
    assert_eq!(cfg.test.baseline_path, ".toksi/test-baseline.json");
}

#[test]
fn test_phase17_2_to_17_3_treesitter_45_langs_and_effort_compressor() {
    assert!(LANGUAGES.len() >= 45, "Expected 45+ language IDs");
    for id in ["rust", "typescript", "python", "go", "dockerfile", "vue", "scss"] {
        assert!(grammar_for(id).is_some(), "Missing grammar for {}", id);
    }

    let comp = HeadroomEffortCompressor::gateway_default();
    assert!(comp.align_1024);
    assert_eq!(EffortLevel::from_str_loose("medium"), Some(EffortLevel::Medium));
}
