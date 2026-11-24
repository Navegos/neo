// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase19_2_multiprocess_supervisor_ipc_verification.rs
// created: 2026-11-24

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::watch;
use toksi::commands::{daemon_help_text_for_test, parse_worker_args, plan_worker_roles, resolve_daemon_plan};
use toksi::config::ToksiTomlConfig;
use toksi::db::{SupervisorWorkerRecord, ToksiConfigDb};
use toksi::ipc::client::send_frame_with_timeout;
use toksi::ipc::frame::{IpcFrame, IpcOpcode};
use toksi::ipc::server::{serve_worker_ipc, DefaultWorkerIpcHandler};
use toksi::ipc::transport::{ipc_endpoint_for_ns_and_role, WorkerRole};

#[test]
fn test_phase19_2_ipc_binary_frame_encode_decode_and_checksum() {
    let payload = serde_json::json!({ "role": "siagent", "seq": 42 });
    let frame = IpcFrame::json(IpcOpcode::SiagentPrompt, 99, &payload).expect("encode json frame");
    let encoded = frame.encode().expect("encode frame");
    let decoded = IpcFrame::decode(&encoded).expect("decode frame");

    assert_eq!(decoded.opcode, IpcOpcode::SiagentPrompt);
    assert_eq!(decoded.correlation_id, 99);
    assert_eq!(decoded.payload_json().expect("json"), payload);

    // Flipping a byte inside checksum must fail validation
    let mut tampered = encoded;
    let last = tampered.len() - 1;
    tampered[last] ^= 0xFF;
    assert!(IpcFrame::decode(&tampered).is_err());
}

#[test]
fn test_phase19_2_worker_roles_and_supervisor_db_crud() {
    assert_eq!(WorkerRole::ALL.len(), 16);
    assert_eq!(WorkerRole::PORT_WORKERS.len(), 8);
    assert_eq!(WorkerRole::TOOL_WORKERS.len(), 7);
    assert_eq!("siagent".parse::<WorkerRole>().expect("siagent"), WorkerRole::Siagent);
    assert_eq!("port-8787".parse::<WorkerRole>().expect("8787"), WorkerRole::Port8787);

    let parsed = parse_worker_args(&[
        "ctxmode".to_string(),
        "--port".to_string(),
        "9099".to_string(),
        "--pipe".to_string(),
        "test-pipe".to_string(),
    ])
    .expect("parse worker args");
    assert_eq!(parsed.role, WorkerRole::Ctxmode);
    assert_eq!(parsed.port_override, Some(9099));
    assert_eq!(parsed.pipe_override.as_deref(), Some("test-pipe"));

    let plan = resolve_daemon_plan(Some(&["claude".to_string()]));
    let roles = plan_worker_roles(&plan);
    assert!(roles.contains(&WorkerRole::Port8787));
    assert!(roles.contains(&WorkerRole::Port9098));
    assert!(roles.contains(&WorkerRole::Port9099));
    assert!(roles.contains(&WorkerRole::Siagent));
    assert!(roles.contains(&WorkerRole::Ctxmode));

    let cfg = ToksiTomlConfig::default();
    assert_eq!(cfg.supervisor.restart_backoff_ms, 500);
    assert_eq!(cfg.supervisor.max_restarts_per_min, 10);
    assert!(daemon_help_text_for_test().contains("daemon"));

    let mut db = ToksiConfigDb::open_in_memory().expect("open in-memory db");
    db.upsert_supervisor_worker(&SupervisorWorkerRecord {
        worker_role: "siagent".to_string(),
        pid: Some(4242),
        ipc_endpoint: ipc_endpoint_for_ns_and_role("phase19-2-db-test", "siagent"),
        status: "running".to_string(),
        restart_count: 1,
        last_heartbeat: 1000,
    })
    .expect("upsert worker");
    db.update_worker_heartbeat("siagent", 2000).expect("heartbeat");
    let rows = db.list_supervisor_workers().expect("list workers");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].last_heartbeat, 2000);
    db.mark_worker_stopped("siagent").expect("stop worker");
    let after = db.list_supervisor_workers().expect("list after stop");
    assert_eq!(after[0].status, "stopped");
    assert_eq!(after[0].pid, None);
}

#[tokio::test]
async fn test_phase19_2_live_ephemeral_ipc_pipe_or_uds_and_kill_switch() {
    let uniq_ns = format!("t19-2-{}", std::process::id());
    let temp = tempfile::tempdir().expect("tempdir");
    #[cfg(windows)]
    let endpoint = ipc_endpoint_for_ns_and_role(&uniq_ns, "siagent");
    #[cfg(not(windows))]
    let endpoint = temp.path().join(format!("{uniq_ns}-siagent.sock")).to_string_lossy().replace('\\', "/");
    let _keep_temp = &temp;

    let (kill_tx, kill_rx) = watch::channel(false);
    let handler = Arc::new(DefaultWorkerIpcHandler::new(WorkerRole::Siagent, kill_tx.clone()));
    let ep_srv = endpoint.clone();
    let srv_task = tokio::spawn(async move {
        let _ = serve_worker_ipc(&ep_srv, handler, kill_rx).await;
    });

    tokio::time::sleep(Duration::from_millis(80)).await;

    let ping = IpcFrame::new(IpcOpcode::Ping, 7, Vec::new());
    let pong = send_frame_with_timeout(&endpoint, &ping, Duration::from_secs(2))
        .await
        .expect("ping over IPC");
    assert_eq!(pong.opcode, IpcOpcode::Pong);
    assert_eq!(pong.correlation_id, 7);
    let json = pong.payload_json().expect("pong json");
    assert_eq!(json["role"], "siagent");
    assert_eq!(json["status"], "ok");

    // KillSwitchTriggered shuts down the listener cleanly
    let ks = IpcFrame::json(
        IpcOpcode::KillSwitchTriggered,
        8,
        &serde_json::json!({ "reason": "test" }),
    )
    .expect("ks frame");
    let ks_reply = send_frame_with_timeout(&endpoint, &ks, Duration::from_secs(2))
        .await
        .expect("kill switch ack");
    assert_eq!(ks_reply.opcode, IpcOpcode::StatusReply);
    assert_eq!(ks_reply.payload_json().expect("ks json")["status"], "draining");
    assert!(*kill_tx.borrow());

    let _ = tokio::time::timeout(Duration::from_secs(2), srv_task).await;
}
