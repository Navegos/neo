// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/dashboard/sse.rs
// created: 2026-11-09

use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast;

use super::state::global_cockpit_state;

/// Creates a metrics snapshot broadcast payload for dashboard clients.
pub fn create_metrics_snapshot() -> serde_json::Value {
    let collector = crate::telemetry::global_collector();
    let snap = crate::telemetry::snapshot_from_collector(collector);

    let state = global_cockpit_state();
    let quotas: Vec<super::state::UpstreamQuotaGauge> = state.quotas.read().map(|q| q.values().cloned().collect()).unwrap_or_default();

    serde_json::json!({
        "type": "snapshot",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "payload": {
            "tokens_saved_lifetime": snap.tokens_saved_lifetime,
            "rtk_tokens_saved": snap.rtk_tokens_saved,
            "headroom_tokens_saved": snap.headroom_tokens_saved,
            "caveman_tokens_saved": snap.caveman_tokens_saved,
            "cost_saved_usd": (snap.cost_saved_usd * 100.0).round() / 100.0,
            "total_calls": snap.total_calls,
            "input_tokens": snap.input_tokens,
            "output_tokens": snap.output_tokens,
            "cached_tokens": snap.cached_tokens,
            "ttft_ms": snap.ttft_ms,
            "tpot_ms": snap.tpot_ms,
            "gpu_utilization_pct": snap.gpu_utilization_pct,
            "openinference_spans_total": snap.openinference_spans_total,
            "quotas": quotas,
        }
    })
}

/// Spawns the background SSE ticker that broadcasts updates every 500ms.
pub fn start_sse_ticker(tx: broadcast::Sender<String>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let snapshot = create_metrics_snapshot();
            let _ = tx.send(snapshot.to_string());
        }
    })
}

/// Streams SSE events to connected HTTP clients.
pub async fn handle_sse_stream(
    mut stream: TcpStream,
    broadcast_tx: broadcast::Sender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let header = "HTTP/1.1 200 OK\r\n\
        Content-Type: text/event-stream\r\n\
        Cache-Control: no-cache\r\n\
        Connection: keep-alive\r\n\
        Access-Control-Allow-Origin: *\r\n\r\n";

    stream.write_all(header.as_bytes()).await?;
    stream.flush().await?;

    let mut rx = broadcast_tx.subscribe();

    // Send initial handshake snapshot
    let initial_msg = format!("data: {}\n\n", create_metrics_snapshot());
    stream.write_all(initial_msg.as_bytes()).await?;
    stream.flush().await?;

    loop {
        match rx.recv().await {
            Ok(msg) => {
                let sse_line = format!("data: {}\n\n", msg);
                if stream.write_all(sse_line.as_bytes()).await.is_err() {
                    break;
                }
                if stream.flush().await.is_err() {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(_) => break,
        }
    }

    Ok(())
}
