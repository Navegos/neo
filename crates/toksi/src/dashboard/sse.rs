// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/dashboard/sse.rs
// created: 2026-11-09

use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast;

use super::state::global_cockpit_state;

/// Creates a metrics snapshot broadcast payload for dashboard clients.
pub fn create_metrics_snapshot() -> serde_json::Value {
    let collector = crate::telemetry::global_collector();
    let tokens_saved: u64 = collector.tokens_saved_total.read().map(|m| m.values().sum()).unwrap_or(0);
    let cost_saved = collector.cost_savings_usd_micros.load(Ordering::Relaxed) as f64 / 1_000_000.0;
    let total_calls: u64 = collector.upstream_requests_total.read().map(|m| m.values().sum()).unwrap_or(0);
    let input_tokens: u64 = collector.input_tokens_total.read().map(|m| m.values().sum()).unwrap_or(0);
    let output_tokens: u64 = collector.output_tokens_total.read().map(|m| m.values().sum()).unwrap_or(0);
    let cached_tokens: u64 = collector.cached_tokens_total.read().map(|m| m.values().sum()).unwrap_or(0);

    let state = global_cockpit_state();
    let quotas: Vec<super::state::UpstreamQuotaGauge> = state.quotas.read().map(|q| q.values().cloned().collect()).unwrap_or_default();

    serde_json::json!({
        "type": "snapshot",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "payload": {
            "tokens_saved_lifetime": tokens_saved,
            "cost_saved_usd": (cost_saved * 100.0).round() / 100.0,
            "total_calls": total_calls,
            "input_tokens": input_tokens,
            "output_tokens": output_tokens,
            "cached_tokens": cached_tokens,
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
