// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase19_3_live_safe_test_isolation_verification.rs

#![deny(unsafe_code)]

use std::path::Path;

use tempfile::TempDir;
use toksi::commands::test_runner::{
    SuiteBaseline, build_docker_test_command_args, check_executable_in_path, live_safe_test_env,
};
use toksi::config::ToksiTomlConfig;

#[test]
fn test_phase19_3_dockerfile_test_contains_pinned_non_root_isolation() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");
    let dockerfile_path = repo_root.join("docker/Dockerfile.test");
    assert!(
        dockerfile_path.exists(),
        "docker/Dockerfile.test must exist at {}",
        dockerfile_path.display()
    );
    let body = std::fs::read_to_string(&dockerfile_path).expect("read Dockerfile.test");

    assert!(body.contains("FROM rust:"));
    assert!(body.contains("-slim"));
    assert!(!body.contains("rust:latest"));
    for pkg in ["git", "sqlite3", "curl"] {
        assert!(body.contains(pkg), "missing package {pkg} in Dockerfile.test");
    }
    assert!(body.contains("USER toksi"));
    assert!(body.contains("TOKSI_TEST=1"));
    assert!(body.contains("TOKSI_HOME=/tmp/toksi-test-home"));
}

#[test]
fn test_phase19_3_test_config_and_suite_baseline_isolation_defaults() {
    let toml_text = r#"
[testing]
incremental = false
live_safe_isolation = true
sandbox_backend = "sbx"
test_image = "toksi-test:custom"
"#;
    let parsed: ToksiTomlConfig = toml::from_str(toml_text).expect("parse [testing] alias");
    assert!(!parsed.test.incremental);
    assert!(parsed.test.live_safe_isolation);
    assert_eq!(parsed.test.sandbox_backend, "sbx");
    assert_eq!(parsed.test.test_image, "toksi-test:custom");

    let suite_json = r#"{
        "watched_paths": ["crates/toksi/src/"],
        "tree_blake3": "abc",
        "last_passed_unix": 0,
        "cargo_args": ["test"]
    }"#;
    let suite: SuiteBaseline = serde_json::from_str(suite_json).expect("deserialize SuiteBaseline");
    assert_eq!(suite.isolation, "ephemeral_tempdir");
}

#[test]
fn test_phase19_3_live_safe_env_docker_args_and_path_probe() {
    let dir = TempDir::new().expect("tempdir");
    let envs = live_safe_test_env(dir.path(), "ns-iso-1");
    assert!(envs.iter().any(|(k, v)| k == "TOKSI_TEST" && v == "1"));
    assert!(envs
        .iter()
        .any(|(k, v)| k == "TOKSI_IPC_NS" && v == "ns-iso-1"));
    let home_val = envs
        .iter()
        .find(|(k, _)| k == "TOKSI_HOME")
        .map(|(_, v)| v.clone())
        .expect("TOKSI_HOME present");
    assert!(!home_val.contains('\\'));
    assert!(!home_val.ends_with(".toksi"));

    let docker_args = build_docker_test_command_args(
        dir.path(),
        "toksi-test:local",
        &["test".to_string(), "-p".to_string(), "toksi".to_string()],
    );
    assert!(docker_args.windows(2).any(|w| w == ["--network", "none"]));
    assert!(docker_args.iter().any(|a| a == "TOKSI_TEST=1"));
    assert!(docker_args.iter().any(|a| a == "toksi-test:local"));

    assert!(!check_executable_in_path(""));
    assert!(!check_executable_in_path("toksi_nonexistent_binary_99999"));
}
