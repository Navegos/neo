// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/tests/phase9_legacy_healing_verification.rs
// created: 2026-10-31

use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use tokless::agents::antigravity::{antigravity_shim_applicable, antigravity_shim_compatible, antigravity_shim_wired};
use tokless::agents::copilot::{copilot_cli_proxy_applicable, copilot_cli_proxy_wired};
use tokless::agents::grok::{grok_shim_wired, install_grok_shim};
use tokless::commands::run_doctor;
use tokless::env_injector::heal::{heal_legacy_hijacks_in, is_tokless_shim_or_sidecar};

fn make_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let p = env::temp_dir().join(format!("tokless-phase9-{label}-{nanos}"));
    fs::create_dir_all(&p).expect("create temp dir");
    p
}

#[test]
fn test_antigravity_binary_unhijack_and_healing() {
    let bin_dir = make_temp_dir("agy-bin");
    let data_dir = make_temp_dir("agy-data");

    // Simulate legacy hijacked agy setup:
    // agy.real.exe: original binary
    // agy.cmd: shim script
    // antigravity-cli-proxy.json: state file
    let real_exe = bin_dir.join("agy.real.exe");
    let shim_cmd = bin_dir.join("agy.cmd");
    let state_file = data_dir.join("antigravity-cli-proxy.json");

    fs::write(&real_exe, b"REAL_ANTIGRAVITY_CORE_BINARY").unwrap();
    fs::write(
        &shim_cmd,
        "@echo off\r\nrem tokless:antigravity-cli-proxy\r\nset \"REAL=agy.real.exe\"\r\n\"%REAL%\" %*\r\n",
    )
    .unwrap();
    fs::write(&state_file, r#"{"bin":"agy.cmd","real":"agy.real.exe"}"#).unwrap();

    assert!(real_exe.exists());
    assert!(shim_cmd.exists());
    assert!(state_file.exists());

    // Execute healing
    let report = heal_legacy_hijacks_in(&[bin_dir.clone()], &[data_dir.clone()]);

    assert!(!report.is_clean());
    assert_eq!(report.errors.len(), 0);
    assert_eq!(report.restored_binaries.len(), 1);
    assert_eq!(report.removed_shims.len(), 1);
    assert_eq!(report.removed_state_files.len(), 1);

    // Verify canonical binary is restored with intact content
    let canonical_exe = bin_dir.join("agy.exe");
    assert!(canonical_exe.exists());
    assert_eq!(fs::read(&canonical_exe).unwrap(), b"REAL_ANTIGRAVITY_CORE_BINARY");
    assert!(!real_exe.exists());

    // Verify shim wrapper and state file are completely purged
    assert!(!shim_cmd.exists());
    assert!(!state_file.exists());

    // Verify antigravity agent module reports no shim applicability/wiring
    assert!(!antigravity_shim_applicable());
    assert!(antigravity_shim_compatible());
    assert!(!antigravity_shim_wired());

    // Clean up
    let _ = fs::remove_dir_all(bin_dir);
    let _ = fs::remove_dir_all(data_dir);
}

#[test]
fn test_grok_binary_unhijack_and_healing() {
    let bin_dir = make_temp_dir("grok-bin");
    let data_dir = make_temp_dir("grok-data");

    // Simulate legacy hijacked grok setup:
    let real_grok = bin_dir.join("grok.real.exe");
    let shim_grok = bin_dir.join("grok.cmd");
    let state_grok = data_dir.join("grok-cli-proxy.json");

    fs::write(&real_grok, b"REAL_GROK_BINARY_DATA").unwrap();
    fs::write(
        &shim_grok,
        "@echo off\r\nrem tokless:grok-cli-proxy\r\n\"%TOKLESS%\" __grok-proxy-ensure\r\n\"%REAL%\" %*\r\n",
    )
    .unwrap();
    fs::write(&state_grok, r#"{"bin":"grok.cmd","real":"grok.real.exe"}"#).unwrap();

    let report = heal_legacy_hijacks_in(&[bin_dir.clone()], &[data_dir.clone()]);

    assert!(!report.is_clean());
    assert_eq!(report.errors.len(), 0);

    let canonical_grok = bin_dir.join("grok.exe");
    assert!(canonical_grok.exists());
    assert_eq!(fs::read(&canonical_grok).unwrap(), b"REAL_GROK_BINARY_DATA");
    assert!(!real_grok.exists());
    assert!(!shim_grok.exists());
    assert!(!state_grok.exists());

    // Verify grok agent module install_grok_shim no longer hijacks binaries
    let shim_res = install_grok_shim();
    assert_eq!(shim_res, Ok(false));
    assert!(!grok_shim_wired());

    let _ = fs::remove_dir_all(bin_dir);
    let _ = fs::remove_dir_all(data_dir);
}

#[test]
fn test_copilot_binary_unhijack_and_sidecar_purge() {
    let bin_dir = make_temp_dir("copilot-bin");
    let data_dir = make_temp_dir("copilot-data");

    // Simulate legacy copilot hijacked state with marker & digest sidecars
    let real_copilot = bin_dir.join("copilot.real");
    let shim_copilot = bin_dir.join("copilot.cmd");
    let digest_sidecar = bin_dir.join("copilot.cmd.digest");
    let marker_sidecar = bin_dir.join("copilot.real.tokless");
    let state_file = data_dir.join("copilot-cli-proxy.json");

    fs::write(&real_copilot, b"REAL_COPILOT_NATIVE_EXEC").unwrap();
    fs::write(
        &shim_copilot,
        "@echo off\r\nrem tokless:copilot-shim\r\n\"tokless.exe\" __copilot %*\r\n",
    )
    .unwrap();
    fs::write(&digest_sidecar, "sha256-hash-val").unwrap();
    fs::write(&marker_sidecar, "tokless-marker-val").unwrap();
    fs::write(&state_file, "{}").unwrap();

    let report = heal_legacy_hijacks_in(&[bin_dir.clone()], &[data_dir.clone()]);

    assert!(!report.is_clean());
    assert_eq!(report.errors.len(), 0);

    // On Windows, canonical name for copilot.real is copilot.exe
    #[cfg(windows)]
    let canonical = bin_dir.join("copilot.exe");
    #[cfg(not(windows))]
    let canonical = bin_dir.join("copilot");

    assert!(canonical.exists());
    assert_eq!(fs::read(&canonical).unwrap(), b"REAL_COPILOT_NATIVE_EXEC");
    assert!(!real_copilot.exists());

    // Shims and sidecars deleted
    assert!(!shim_copilot.exists());
    assert!(!digest_sidecar.exists());
    assert!(!marker_sidecar.exists());
    assert!(!state_file.exists());

    // Copilot CLI proxy is purged
    assert!(!copilot_cli_proxy_applicable());
    assert!(!copilot_cli_proxy_wired());

    let _ = fs::remove_dir_all(bin_dir);
    let _ = fs::remove_dir_all(data_dir);
}

#[test]
fn test_shim_detection_predicates() {
    let temp_dir = make_temp_dir("predicates");

    let agy_shim = temp_dir.join("agy.cmd");
    fs::write(&agy_shim, "@echo off\r\nrem tokless:antigravity-cli-proxy\r\n").unwrap();
    assert!(is_tokless_shim_or_sidecar(&agy_shim));

    let grok_shim = temp_dir.join("grok.cmd");
    fs::write(&grok_shim, "@echo off\r\n\"%TOKLESS%\" __grok-proxy-ensure\r\n").unwrap();
    assert!(is_tokless_shim_or_sidecar(&grok_shim));

    let copilot_digest = temp_dir.join("copilot.digest");
    fs::write(&copilot_digest, "abc").unwrap();
    assert!(is_tokless_shim_or_sidecar(&copilot_digest));

    let clean_bin = temp_dir.join("clean.exe");
    fs::write(&clean_bin, b"MZ\x90\x00\x03\x00\x00\x00").unwrap();
    assert!(!is_tokless_shim_or_sidecar(&clean_bin));

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn test_doctor_fix_invocation() {
    // Calling run_doctor with fix=true must succeed with exit code 0
    let exit_code = run_doctor(true, false, true);
    assert_eq!(exit_code, 0, "tokless doctor --fix must succeed");
}

