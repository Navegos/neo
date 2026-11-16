// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase15_2_unified_8port_daemon_verification.rs
// created: 2026-11-16
// lastModified: 2026-11-16

use std::sync::Mutex;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

static RUNTIME_OVERRIDE_TEST_LOCK: Mutex<()> = Mutex::new(());

use toksi::commands::daemon::resolve_daemon_plan;
use toksi::config::{
    clear_runtime_overrides, set_runtime_base_url_override, set_runtime_headroom_port_override,
    ConfigError, PortsConfig, ToksiTomlConfig,
};
use toksi::dashboard::spawn_isolated_dashboard;
use toksi::env_injector::{proxy_url_from_ports, TeardownGuard, CLOUD_CODE_URL_KEY};
use toksi::gateway::spawn_isolated_gateway;
use toksi::headroom::copilot_daemon::{run_native_copilot_proxy, validate_copilot_ports};
use toksi::headroom::grok_daemon::run_native_grok_proxy;
use toksi::tools::multi_protocol::run_protocol_bus_server;

#[test]
fn test_phase15_2_config_toml_8port_overrides_and_validation() {
    let ports = PortsConfig::default();
    assert_eq!(ports.headroom_proxy, 8787);
    assert_eq!(ports.grok_oauth_proxy, 8788);
    assert_eq!(ports.copilot_vscode_proxy, 8789);
    assert_eq!(ports.copilot_cli_proxy, 8790);
    assert_eq!(ports.protocol_bus, 9098);
    assert_eq!(ports.ai_gateway, 9099);
    assert_eq!(ports.dashboard, 9100);
    assert_eq!(ports.byok_gateway, 18787);
    assert!(ports.validate().is_ok());

    let mut colliding = ports.clone();
    colliding.protocol_bus = colliding.ai_gateway;
    assert!(matches!(
        colliding.validate(),
        Err(ConfigError::PortCollision { port: 9099, .. })
    ));

    let raw = r#"
[general]
default_agents = ["claude", "copilot"]

[ports]
headroom = 18701
grok_oauth = 18702
copilot_vscode = 18703
copilot_cli = 18704
protocol_bus = 19098
ai_gateway = 19099
dashboard = 19100
byok_gateway = 28787

[base_urls]
openai = "https://custom.openai.test/v1"
"#;
    let parsed: ToksiTomlConfig = toml::from_str(raw).expect("valid config.toml");
    assert_eq!(parsed.ports.protocol_bus, 19098);
    assert_eq!(parsed.ports.byok_gateway, 28787);
    assert!(parsed.ports.validate().is_ok());
    assert_eq!(
        proxy_url_from_ports(&parsed.ports),
        "http://127.0.0.1:18701"
    );

    assert!(validate_copilot_ports().is_ok());

    let _lock = RUNTIME_OVERRIDE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_runtime_overrides();
    set_runtime_headroom_port_override(Some(28701));
    assert_eq!(
        toksi::config::runtime_headroom_port_override(),
        Some(28701)
    );
    clear_runtime_overrides();
}

#[test]
fn test_phase15_2_daemon_plan_skips_unconfigured_agents() {
    let claude = vec!["claude".to_string()];
    let plan = resolve_daemon_plan(Some(&claude));
    assert!(plan.headroom_proxy);
    assert!(plan.protocol_bus);
    assert!(plan.ai_gateway);
    assert!(plan.dashboard);
    assert!(!plan.grok_oauth_proxy);
    assert!(!plan.copilot_vscode_proxy);
    assert!(!plan.copilot_cli_proxy);

    let all = vec!["all".to_string()];
    let plan_all = resolve_daemon_plan(Some(&all));
    assert!(plan_all.headroom_proxy);
    assert!(plan_all.grok_oauth_proxy);
    assert!(plan_all.copilot_vscode_proxy);
    assert!(plan_all.copilot_cli_proxy);
    assert!(plan_all.protocol_bus);
    assert!(plan_all.byok_gateway);
    assert!(plan_all.ai_gateway);
    assert!(plan_all.dashboard);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_phase15_2_copilot_real_sse_proxying_and_livez() {
    let _lock = RUNTIME_OVERRIDE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let upstream_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock copilot upstream");
    let upstream_port = upstream_listener.local_addr().expect("local addr").port();

    let upstream_task = tokio::spawn(async move {
        if let Ok((mut sock, _)) = upstream_listener.accept().await {
            let mut buf = [0u8; 8192];
            let _ = sock.read(&mut buf).await;
            let sse = "data: {\"id\":\"chatcmpl-1\",\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\ndata: [DONE]\n\n";
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                sse.len(),
                sse
            );
            let _ = sock.write_all(resp.as_bytes()).await;
            let _ = sock.shutdown().await;
        }
    });

    set_runtime_base_url_override(
        "copilot-vscode",
        &format!("http://127.0.0.1:{}", upstream_port),
    );

    let proxy_probe = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("reserve proxy port");
    let proxy_port = proxy_probe.local_addr().expect("proxy addr").port();
    drop(proxy_probe);

    let proxy_task = tokio::spawn(async move {
        let _ = run_native_copilot_proxy(proxy_port, false).await;
    });

    let grok_probe = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("reserve grok port");
    let grok_port = grok_probe.local_addr().expect("grok addr").port();
    drop(grok_probe);

    let grok_task = tokio::spawn(async move {
        let _ = run_native_grok_proxy(grok_port).await;
    });

    tokio::time::sleep(Duration::from_millis(150)).await;

    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("reqwest client");

    let livez = client
        .get(format!("http://127.0.0.1:{}/livez", proxy_port))
        .send()
        .await
        .expect("copilot livez");
    assert_eq!(livez.status().as_u16(), 200);
    let livez_json: serde_json::Value = livez.json().await.expect("livez json");
    assert_eq!(
        livez_json.get("subservice").and_then(|v| v.as_str()),
        Some("copilot-vscode-proxy")
    );

    let grok_livez = client
        .get(format!("http://127.0.0.1:{}/livez", grok_port))
        .send()
        .await
        .expect("grok livez");
    assert_eq!(grok_livez.status().as_u16(), 200);
    let grok_json: serde_json::Value = grok_livez.json().await.expect("grok json");
    assert_eq!(
        grok_json.get("subservice").and_then(|v| v.as_str()),
        Some("grok-oauth")
    );

    let chat_resp = client
        .post(format!(
            "http://127.0.0.1:{}/chat/completions",
            proxy_port
        ))
        .json(&serde_json::json!({
            "model": "gpt-4o",
            "stream": true,
            "messages": [{"role": "user", "content": "ping"}]
        }))
        .send()
        .await
        .expect("copilot chat completions");
    let status = chat_resp.status().as_u16();
    let chat_text = chat_resp.text().await.expect("chat text");
    assert_eq!(status, 200, "unexpected status {} body: {}", status, chat_text);
    assert!(chat_text.contains("chatcmpl-1"));
    assert!(chat_text.contains("[DONE]"));

    clear_runtime_overrides();
    proxy_task.abort();
    grok_task.abort();
    upstream_task.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_phase15_2_gateway_dashboard_protocol_bus_isolation() {
    let p_bus = {
        let l = TcpListener::bind("127.0.0.1:0").await.expect("bus port");
        l.local_addr().expect("addr").port()
    };
    let p_gw = {
        let l = TcpListener::bind("127.0.0.1:0").await.expect("gw port");
        l.local_addr().expect("addr").port()
    };
    let p_dash = {
        let l = TcpListener::bind("127.0.0.1:0").await.expect("dash port");
        l.local_addr().expect("addr").port()
    };

    let bus_task = tokio::spawn(async move {
        let _ = run_protocol_bus_server(p_bus).await;
    });
    let mut gw = spawn_isolated_gateway("127.0.0.1", p_gw).expect("spawn isolated gateway");
    let mut dash = spawn_isolated_dashboard("127.0.0.1", p_dash).expect("spawn isolated dashboard");

    assert_ne!(gw.thread_name, dash.thread_name);
    tokio::time::sleep(Duration::from_millis(150)).await;

    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("reqwest client");

    let (r_bus, r_gw, r_dash) = tokio::join!(
        client
            .post(format!("http://127.0.0.1:{}/mcp", p_bus))
            .json(&serde_json::json!({
                "tool": "rtk",
                "action": "filter",
                "payload": { "command": "cargo test", "output": "test result: ok. 5 passed" }
            }))
            .send(),
        client
            .get(format!("http://127.0.0.1:{}/healthz", p_gw))
            .send(),
        client
            .get(format!("http://127.0.0.1:{}/api/status", p_dash))
            .send(),
    );

    let bus_resp = r_bus.expect("bus resp");
    assert_eq!(bus_resp.status().as_u16(), 200);
    let bus_txt = bus_resp.text().await.expect("bus txt");
    assert!(bus_txt.contains("passed"));

    let gw_resp = r_gw.expect("gw resp");
    assert_eq!(gw_resp.status().as_u16(), 200);

    let dash_resp = r_dash.expect("dash resp");
    assert_eq!(dash_resp.status().as_u16(), 200);

    gw.shutdown();
    dash.shutdown();
    bus_task.abort();
}

#[test]
fn test_phase15_2_teardown_guard_never_injects_cloud_code_url() {
    let guard = TeardownGuard::with_snapshot(Default::default());
    assert!(guard.snapshot().is_some());
    assert!(toksi::env_injector::is_forbidden_auth_bypass_key(
        CLOUD_CODE_URL_KEY
    ));
    drop(guard);
}
