// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase21_1_to_21_6_verification.rs
// created: 2026-11-25

#![deny(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use serde_json::json;
use tempfile::TempDir;
use tokio::sync::watch;

use toksi::agent_runtime::chat_ui::{execute_in_chat_cli_command, ChatUiAction, ChatUiState};
use toksi::agent_runtime::mode_router::OrchestratorMode;
use toksi::agent_runtime::orchestration_resume::{
    flush_five_stores_before_exit, prepare_cross_chat_resume, OrchestrationFailureKind,
    WorkerLeaseRegistry, FIVE_STORES,
};
use toksi::agent_runtime::swarm::{plan_isolated_worktree, KanbanFleetBoard, KanbanLane};
use toksi::agents::{
    parse_roomodes_json, preserve_thought_signature, extract_thought_signature,
    semble_semantic_search, RooIgnoreRules, SembleSearchQuery, SubtaskStack,
};
use toksi::commands::daemon::{plan_worker_roles, resolve_daemon_plan};
use toksi::commands::guard::{CommandGuard, GuardDecision, OffensiveCategory};
use toksi::config::PortsConfig;
use toksi::gateway::local_engines::{
    total_mandated_vault_count, ELEVEN_AIML_PILLARS, WAVE21_RECONCILED_VAULTS,
};
use toksi::ipc::client::send_frame_with_timeout;
use toksi::ipc::network_manager::{
    spawn_network_manager_listeners, NetworkServiceRouteTable, PortRouteBinding,
};
use toksi::ipc::server::{serve_worker_ipc, DefaultWorkerIpcHandler};
use toksi::ipc::transport::WorkerRole;
use toksi::ipc::{IpcFrame, IpcOpcode};
use toksi::projectmem::plan_checkpoint::PlanCheckpoint;
use toksi::tools::ide_plugin_host::{handle_ide_http_route, IdePluginHost, IdePluginKind};

#[tokio::test]
async fn test_phase21_1_single_network_manager_and_siagent_decoupling() {
    let plan = resolve_daemon_plan(None);
    assert!(!plan.dashboard, "daemon start must keep :9100 detached");
    let roles = plan_worker_roles(&plan);
    assert!(roles.contains(&WorkerRole::NetworkManager));
    assert!(roles.contains(&WorkerRole::Cluster));
    assert!(!roles.contains(&WorkerRole::Siagent));

    let ports = PortsConfig::default();
    let mut table = NetworkServiceRouteTable::from_daemon_plan(&plan, &ports);
    let dup = table.claim_port(PortRouteBinding {
        port: ports.headroom_proxy,
        service_role: WorkerRole::Port9099,
        protocol: "http".to_string(),
        enabled: true,
    });
    assert!(dup.is_err(), "duplicate port bind must fail before OS bind");

    // Ephemeral listener + DirectWire IPC roundtrip (zero host port collision)
    let uniq_ns = format!("p21-1-{}", std::process::id());
    let temp = TempDir::new().expect("tempdir");
    #[cfg(windows)]
    let endpoint = toksi::ipc::transport::ipc_endpoint_for_ns_and_role(&uniq_ns, "port-9098");
    #[cfg(not(windows))]
    let endpoint = temp
        .path()
        .join(format!("{uniq_ns}-port-9098.sock"))
        .to_string_lossy()
        .replace('\\', "/");
    let _keep = &temp;

    let (kill_tx, kill_rx) = watch::channel(false);
    let handler = Arc::new(DefaultWorkerIpcHandler::new(WorkerRole::Port9098, kill_tx.clone()));
    let ep_srv = endpoint.clone();
    let srv_task = tokio::spawn(async move {
        let _ = serve_worker_ipc(&ep_srv, handler, kill_rx).await;
    });
    tokio::time::sleep(Duration::from_millis(60)).await;

    let nm_handle = spawn_network_manager_listeners(
        vec![PortRouteBinding {
            port: 0,
            service_role: WorkerRole::Port9098,
            protocol: "mcp/ide".to_string(),
            enabled: true,
        }],
        "127.0.0.1",
    )
    .await
    .expect("bind ephemeral port");
    assert_eq!(nm_handle.bound_ports.len(), 1);
    assert_ne!(nm_handle.bound_ports[0].1, 0);

    let fwd = IpcFrame::json(
        IpcOpcode::NetworkForward,
        42,
        &json!({
            "method": "GET",
            "path": "/ide/manifests",
            "headers": [],
            "body": "",
        }),
    )
    .expect("network forward frame");
    let reply = send_frame_with_timeout(&endpoint, &fwd, Duration::from_secs(2))
        .await
        .expect("ipc reply");
    let reply_json = reply.payload_json().expect("reply json");
    assert_eq!(reply_json["status_code"], 200);
    let body_str = reply_json["body"].as_str().expect("body string");
    let parsed_body: serde_json::Value = serde_json::from_str(body_str).expect("parse ide body");
    assert_eq!(parsed_body["count"], 13);

    nm_handle.shutdown();
    let _ = kill_tx.send(true);
    let _ = tokio::time::timeout(Duration::from_secs(2), srv_task).await;
}

#[test]
fn test_phase21_2_codex_grok_chat_tui_and_cli_bridge() {
    let mut state = ChatUiState::new("wave21-chat", PathBuf::from("."));
    assert_eq!(state.active_mode, OrchestratorMode::WriteCodeBuild);
    assert!(state.set_mode("coding").is_ok());
    assert!(state.set_mode("planning").is_ok());
    assert_eq!(state.active_mode, OrchestratorMode::PlanDecompose);
    assert!(state.set_mode("research").is_ok());
    assert_eq!(state.active_mode, OrchestratorMode::ResearchGround);

    state.add_tool_card("c1", "codegraph", "trace symbol", "2 callers", "ok");
    assert!(state.tool_cards[0].collapsed);
    let _ = state.handle_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    assert!(!state.tool_cards[0].collapsed);

    state.open_diff_preview("src/lib.rs", vec!["@@ -1 +1 @@".to_string()]);
    state.open_hitl_modal("h1", "apply_patch", "update src/lib.rs");
    let hitl = state.handle_key_event(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));
    assert_eq!(
        hitl,
        Some(ChatUiAction::ResolveHitl {
            request_id: "h1".to_string(),
            approved: true
        })
    );

    let backend = TestBackend::new(110, 32);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    terminal.draw(|f| state.render(f)).expect("render chat ui");

    let ver = execute_in_chat_cli_command(std::path::Path::new("."), "toksi version")
        .expect("matched cli")
        .expect("version ok");
    assert!(!ver.trim().is_empty());

    let denied = execute_in_chat_cli_command(std::path::Path::new("."), "toksi exec rm -rf /")
        .expect("matched cli");
    assert!(denied.is_err());
}

#[test]
fn test_phase21_3_external_13_ide_plugin_host_protocol() {
    let host = IdePluginHost::new();
    let manifests = host.all_manifests();
    assert_eq!(manifests.len(), 13);
    assert_eq!(IdePluginKind::ALL.len(), 13);

    for ide in [
        "vscode",
        "visual-studio",
        "jetbrains",
        "android-studio",
        "zed",
        "xcode",
        "eclipse",
        "cursor",
        "windsurf",
        "apache-netbeans",
        "kiro",
        "qoder",
        "lapce",
    ] {
        let kind: IdePluginKind = ide.parse().expect("parse ide");
        let manifest = host.manifest_for(kind);
        assert!(manifest.external_package_only);
        assert_eq!(manifest.orchestrator_modes_count, 22);
        assert_eq!(manifest.slash_commands_count, 29);
        assert_eq!(manifest.treesitter_languages_count, 45);
    }

    let (code, resp) = handle_ide_http_route(
        "/ide/handshake",
        "POST",
        br#"{"ide":"apache-netbeans"}"#,
    )
    .expect("handshake");
    assert_eq!(code, 200);
    assert_eq!(resp["result"]["manifest"]["ide"], "apache-netbeans");
    assert_eq!(resp["result"]["manifest"]["packaging_format"], "nbm");
}

#[test]
fn test_phase21_4_zoo_code_agent_orchestrator_11_pillars_and_495_vaults() {
    assert_eq!(total_mandated_vault_count(), 495);
    assert_eq!(WAVE21_RECONCILED_VAULTS, &["Zoo-Code", "agent-orchestrator"]);
    assert_eq!(ELEVEN_AIML_PILLARS.len(), 11);

    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path();
    fs::create_dir_all(root.join("src")).expect("mkdir src");
    fs::write(root.join(".rooignore"), "ignored.rs\n").expect("write rooignore");
    fs::write(root.join("src/keep.rs"), "pub fn network_manager_route() {}\n").expect("write keep");
    fs::write(root.join("src/ignored.rs"), "pub fn network_manager_secret() {}\n")
        .expect("write ignored");

    let ignore = RooIgnoreRules::load_from_workspace(root);
    assert!(ignore.is_ignored("src/ignored.rs") || ignore.is_ignored("ignored.rs"));

    let hits = semble_semantic_search(
        root,
        &SembleSearchQuery {
            query: "network_manager_route".to_string(),
            max_results: 5,
            extensions: vec!["rs".to_string()],
        },
    );
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].file_path, "src/keep.rs");

    let modes = parse_roomodes_json(
        r#"{"customModes":[{"slug":"sec","name":"Sec","allowedMcpServers":["codegraph"]}]}"#,
    )
    .expect("roomodes");
    assert!(modes[0].is_mcp_server_allowed("codegraph"));
    assert!(!modes[0].is_mcp_server_allowed("shell"));

    let mut subtasks = SubtaskStack::new();
    subtasks.push_subtask("p1", None, "orchestrator", "root");
    subtasks.push_subtask("c1", Some("p1".to_string()), "code", "child");
    assert_eq!(subtasks.recover_orphaned_subtasks("p1"), vec!["c1"]);

    let mut sig_payload = json!({ "model": "gemini-4-neon", "effort": "xhigh" });
    preserve_thought_signature(&mut sig_payload, "g4-xhigh-sig-001");
    assert_eq!(
        extract_thought_signature(&sig_payload).as_deref(),
        Some("g4-xhigh-sig-001")
    );

    match CommandGuard::check_pre_exec("test-actor", "git push --force origin main", None) {
        GuardDecision::Deny { category, .. } => {
            assert_eq!(category, OffensiveCategory::DestructiveCommandGuard);
        }
        other => panic!("Expected DCG deny, got {other:?}"),
    }

    let wt = plan_isolated_worktree(root, "Task-21-4", "main", "zoo-code").expect("worktree");
    assert_eq!(wt.branch_name, "toksi/task/task-21-4");
    let mut board = KanbanFleetBoard::new();
    board.upsert_task(wt);
    board
        .transition_lane("task-21-4", KanbanLane::InReview)
        .expect("in review");
    assert_eq!(
        board
            .route_ci_failure("task-21-4", "cargo test failed")
            .expect("ci route"),
        KanbanLane::Working
    );
    assert_eq!(
        board
            .route_pr_review_comment("task-21-4", "maintainer", "needs signoff", true)
            .expect("pr review"),
        KanbanLane::NeedsYou
    );
}

#[test]
fn test_phase21_5_multi_agent_leases_five_store_flush_and_cross_chat_resume() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path();
    fs::create_dir_all(root.join(".git")).expect("mkdir .git");
    fs::create_dir_all(root.join("src")).expect("mkdir src");
    fs::write(root.join("src/mod_a.rs"), "pub fn a() -> u32 { 1 }\n").expect("write a");

    let registry = WorkerLeaseRegistry::for_repo(root);
    registry
        .claim_leases("worker-a", std::process::id(), &["src/mod_a.rs".to_string()])
        .expect("claim worker-a");
    let overlap = registry.claim_leases(
        "worker-b",
        std::process::id(),
        &["src/mod_a.rs".to_string()],
    );
    assert!(overlap.is_err(), "disjoint file lease must reject overlap");

    let mut cp = PlanCheckpoint::new("wave21-resume", "Wave 21 Cross-Chat Resume");
    cp.active_phase = "PHASE_21_5".to_string();
    cp.active_byok_profile = "l0-primary".to_string();
    cp.completed_steps = vec!["step-1".to_string(), "step-2".to_string()];
    cp.pending_steps = vec!["step-3".to_string(), "step-4".to_string()];
    cp.total_steps = 4;

    let flush = flush_five_stores_before_exit(
        root,
        &mut cp,
        OrchestrationFailureKind::QuotaExhausted,
        "worker-a",
        &["src/mod_a.rs".to_string()],
        &["l0-primary".to_string(), "l1-backup".to_string()],
    )
    .expect("flush 5 stores");
    assert_eq!(flush.flushed_stores.len(), FIVE_STORES.len());
    assert_eq!(flush.failover_next_profile.as_deref(), Some("l1-backup"));
    for store in FIVE_STORES {
        assert!(root
            .join(".toksi")
            .join(store)
            .join("last_resume_flush.json")
            .is_file());
    }

    let resume = prepare_cross_chat_resume(root, "wave21-resume").expect("prepare resume");
    assert_eq!(resume.next_step_index, 2);
    assert_eq!(resume.next_step_title.as_deref(), Some("step-3"));
    assert_eq!(resume.blake3_verified_files, vec!["src/mod_a.rs"]);
    assert!(resume.blake3_drifted_files.is_empty());
    assert_eq!(resume.active_byok_profile, "l1-backup");
    assert_eq!(resume.journal_entries_count, 1);
}

#[tokio::test]
async fn test_network_forward_ipc_real_http_dispatch() {
    use toksi::ipc::network_manager::forward_payload_over_ipc;
    use toksi::ipc::transport::ipc_endpoint_for_ns_and_role;

    let uniq = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(1);
    let uniq_ns = format!("p21-real-fwd-{}-{uniq}", std::process::id());
    let temp = TempDir::new().expect("tempdir");
    let src_dir = temp.path().join("scan_target");
    fs::create_dir_all(&src_dir).expect("mkdir scan target");
    fs::write(src_dir.join("lib.rs"), "pub fn safe_add(a: i32, b: i32) -> i32 { a + b }\n")
        .expect("write lib.rs");

    #[cfg(windows)]
    let ep9099 = ipc_endpoint_for_ns_and_role(&uniq_ns, "port-9099");
    #[cfg(not(windows))]
    let ep9099 = temp
        .path()
        .join(format!("{uniq_ns}-9099.sock"))
        .to_string_lossy()
        .replace('\\', "/");

    #[cfg(windows)]
    let ep9098 = ipc_endpoint_for_ns_and_role(&uniq_ns, "port-9098");
    #[cfg(not(windows))]
    let ep9098 = temp
        .path()
        .join(format!("{uniq_ns}-9098.sock"))
        .to_string_lossy()
        .replace('\\', "/");

    let (kill_tx99, kill_rx99) = watch::channel(false);
    let h99 = Arc::new(DefaultWorkerIpcHandler::new(WorkerRole::Port9099, kill_tx99.clone()));
    let ep99_srv = ep9099.clone();
    let srv99 = tokio::spawn(async move {
        let _ = serve_worker_ipc(&ep99_srv, h99, kill_rx99).await;
    });

    let (kill_tx98, kill_rx98) = watch::channel(false);
    let h98 = Arc::new(DefaultWorkerIpcHandler::new(WorkerRole::Port9098, kill_tx98.clone()));
    let ep98_srv = ep9098.clone();
    let srv98 = tokio::spawn(async move {
        let _ = serve_worker_ipc(&ep98_srv, h98, kill_rx98).await;
    });

    tokio::time::sleep(Duration::from_millis(80)).await;

    let models_reply = forward_payload_over_ipc(
        WorkerRole::Port9099,
        &ep9099,
        "GET",
        "/v1/models",
        Vec::new(),
        Vec::new(),
    )
    .await
    .expect("ipc /v1/models");
    assert_eq!(models_reply["status_code"], 200);
    let models_json: serde_json::Value =
        serde_json::from_str(models_reply["body"].as_str().expect("models body str"))
            .expect("models json");
    assert_eq!(models_json["object"], "list");
    assert!(!models_json["data"].as_array().expect("data array").is_empty());

    let metrics_reply = forward_payload_over_ipc(
        WorkerRole::Port9099,
        &ep9099,
        "GET",
        "/metrics",
        Vec::new(),
        Vec::new(),
    )
    .await
    .expect("ipc /metrics");
    assert_eq!(metrics_reply["status_code"], 200);
    let hdrs = metrics_reply["headers"].as_array().expect("headers array");
    assert!(hdrs.iter().any(|pair| {
        pair.get(0).and_then(|k| k.as_str()).is_some_and(|k| k.eq_ignore_ascii_case("content-type"))
            && pair.get(1).and_then(|v| v.as_str()).is_some_and(|v| v.contains("text/plain"))
    }));
    assert!(metrics_reply["body"].as_str().expect("metrics body").contains("toksi_"));

    let scan_req = json!({ "path": src_dir.to_string_lossy().replace('\\', "/") }).to_string();
    let scan_reply = forward_payload_over_ipc(
        WorkerRole::Port9099,
        &ep9099,
        "POST",
        "/v1/security/scan",
        vec![("Content-Type".to_string(), "application/json".to_string())],
        scan_req.into_bytes(),
    )
    .await
    .expect("ipc /v1/security/scan");
    assert_eq!(scan_reply["status_code"], 200);
    let scan_json: serde_json::Value =
        serde_json::from_str(scan_reply["body"].as_str().expect("scan body"))
            .expect("scan json");
    assert!(scan_json.is_object());

    let mcp_req = json!({
        "tool": "rtk",
        "action": "filter",
        "payload": {
            "command": "cargo test",
            "output": "test result: ok. 5 passed; 0 failed"
        }
    })
    .to_string();
    let mcp_reply = forward_payload_over_ipc(
        WorkerRole::Port9098,
        &ep9098,
        "POST",
        "/mcp",
        vec![("Content-Type".to_string(), "application/json".to_string())],
        mcp_req.into_bytes(),
    )
    .await
    .expect("ipc /mcp");
    assert_eq!(mcp_reply["status_code"], 200);
    let mcp_body = mcp_reply["body"].as_str().expect("mcp body");
    let mcp_json: serde_json::Value = serde_json::from_str(mcp_body).expect("mcp json");
    assert_eq!(mcp_json["jsonrpc"], "2.0");
    assert!(mcp_json.get("error").is_none(), "unexpected mcp error: {mcp_body}");
    assert!(mcp_body.contains("passed"), "unexpected mcp response: {mcp_body}");

    let _ = kill_tx99.send(true);
    let _ = kill_tx98.send(true);
    let _ = tokio::time::timeout(Duration::from_secs(2), srv99).await;
    let _ = tokio::time::timeout(Duration::from_secs(2), srv98).await;
}

#[test]
fn test_full_example_config_toml_parses_cleanly() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let cfg_path = manifest_dir.join("full_example_config.toml");
    let raw = fs::read_to_string(&cfg_path).expect("read full_example_config.toml");

    let parsed: toksi::config::ToksiTomlConfig =
        toml::from_str(&raw).expect("parse ToksiTomlConfig");
    assert!(parsed.ports.validate().is_ok());
    assert_eq!(parsed.ide_bridge.supported_ides.len(), 13);
    assert!(parsed
        .byok_profiles
        .iter()
        .any(|p| p.temperature.is_some() && p.top_p.is_some() && p.top_k.is_some()));

    let total_modes: usize = parsed
        .toksi_siagent_router
        .domains
        .values()
        .map(|m| m.len())
        .sum();
    assert_eq!(total_modes, 22, "all 22 router modes must parse cleanly");

    let raw_val: toml::Value = toml::from_str(&raw).expect("parse toml::Value");
    let tbl = raw_val.as_table().expect("top-level table");
    for expected_key in [
        "general",
        "ports",
        "base_urls",
        "default_agents",
        "auth",
        "byok",
        "byok_profiles",
        "repositories",
        "security",
        "test",
        "supervisor",
        "headroom",
        "gateway",
        "plugins",
        "checkpoint",
        "sandbox",
        "cluster",
        "training",
        "telemetry",
        "dashboard",
        "tui",
        "keyring",
        "tools",
        "hooks",
        "ide_bridge",
        "toksi_siagent_router",
    ] {
        assert!(
            tbl.contains_key(expected_key),
            "full_example_config.toml missing section '{expected_key}'"
        );
    }
}

#[test]
fn test_wave20_slash_commands_real_execution() {
    use toksi::agent_runtime::slash::SlashCommandEngine;

    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path();
    let ds_path = root.join("train.jsonl");
    fs::write(
        &ds_path,
        "{\"prompt\":\"hi\",\"response\":\"ok\"}\n{\"prompt\":\"rust\",\"response\":\"safe\"}\n",
    )
    .expect("write dataset");

    let bad_st_path = root.join("bad.safetensors");
    fs::write(&bad_st_path, [0x80, 0x04, 0x95, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
        .expect("write pickle test file");

    let engine = SlashCommandEngine::new(root.to_path_buf());

    let r1 = engine.execute("/workforce register w1 coder codex-cli", "test");
    assert!(r1.handled && r1.output.contains("Registered worker=w1"));
    let r2 = engine.execute("/workforce submit t1 coder fix module", "test");
    assert!(r2.output.contains("Queued task=t1"));
    let r3 = engine.execute("/workforce claim w1", "test");
    assert!(r3.output.contains("claimed task=t1"));
    let r4 = engine.execute("/workforce complete t1 done", "test");
    assert!(r4.output.contains("Completed task=t1"));

    let bad_spec = engine.execute("/spec create s1 empty criteria", "test");
    assert!(bad_spec.output.contains("Validation rejected"));
    let ok_spec = engine.execute(
        "/spec create s1 build gateway | passes cargo test;zero stubs",
        "test",
    );
    assert!(ok_spec.output.contains("Created contract id=s1"));
    let tr_spec = engine.execute("/spec transition s1 reviewed", "test");
    assert!(tr_spec.output.contains("transitioned -> Reviewed"));

    let ds_res = engine.execute("/dataset inspect train.jsonl", "test");
    assert!(ds_res.output.contains("samples=2") && ds_res.output.contains("valid=true"));

    let tr_res = engine.execute("/train recipe unsloth qlora Qwen/Qwen2.5-Coder-7B train.jsonl high", "test");
    assert!(tr_res.output.contains("backend=unsloth") && tr_res.output.contains("method=qlora"));
    let st_res = engine.execute("/train verify-safetensors bad.safetensors", "test");
    assert!(st_res.output.contains("rejected") && st_res.output.contains("pickle"));

    let ev_res = engine.execute("/eval summary deepeval gpt-5.4 10 8", "test");
    assert!(ev_res.output.contains("passed=8/10") && ev_res.output.contains("score=0.800"));

    let cmp_res = engine.execute("/compress kvpress snapkv 0.5 32768", "test");
    assert!(cmp_res.output.contains("retained=16384/32768"));

    let srv_res = engine.execute("/serve profile ollama qwen2.5-coder:7b high", "test");
    assert!(srv_res.output.contains("port=11434") && srv_res.output.contains("endpoint="));

    let vox_res = engine.execute("/voice whisper-large-v3-turbo", "test");
    assert!(vox_res.output.contains("engine=whisper.cpp") && vox_res.output.contains("path_probes="));
}
