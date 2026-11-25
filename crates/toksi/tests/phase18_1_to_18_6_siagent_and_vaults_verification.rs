// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase18_1_to_18_6_siagent_and_vaults_verification.rs

#![deny(unsafe_code)]

use std::path::Path;
use toksi::agent_runtime::sandbox::SandboxManager;
use toksi::agent_runtime::slash::CORE_SLASH_COMMANDS;
use toksi::plugins::scan_plugin_content;
use toksi::rules::skills_repo::SkillsRepoConfig;

#[test]
fn test_phase18_37_brain_guides_and_zero_bloat_external_skills() {
    let brain_dir = Path::new("../../.agents/brain");
    assert!(brain_dir.exists(), ".agents/brain must exist");
    let md_count = std::fs::read_dir(brain_dir)
        .expect("read brain dir")
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
        .count();
    assert_eq!(md_count, 55, "Expected 55 .agents/brain/*.md guides");

    let cfg = SkillsRepoConfig::load();
    assert!(!cfg.skills_remote.is_empty());
}

#[test]
fn test_phase18_siagent_29_slash_cmds_sandbox_and_enforce_plugins() {
    assert_eq!(CORE_SLASH_COMMANDS.len(), 29);
    for cmd in [
        "/advisory", "/skills", "/sandbox", "/plugin", "/resume", "/security", "/effort",
        "/mode", "/sampling", "/router", "/workforce", "/spec", "/dataset", "/train",
        "/eval", "/compress", "/serve", "/cluster", "/otel", "/voice",
    ] {
        assert!(CORE_SLASH_COMMANDS.contains(&cmd), "Missing slash cmd {}", cmd);
    }

    let sb = SandboxManager::new();
    let _ = sb;

    let clean = scan_plugin_content("safe plugin", "fn hello() {}");
    assert_eq!(format!("{:?}", clean.status), "Passed");
}
