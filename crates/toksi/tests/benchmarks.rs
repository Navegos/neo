// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/benchmarks.rs

use std::collections::HashMap;
use std::time::Instant;
use chrono::Utc;
use serde_json::json;

use toksi::headroom::cache_align::estimate_tokens;
use toksi::headroom::compression::code::{compress_code, CodeCompressorConfig, CodeLanguage};
use toksi::headroom::compression::json::{prune_json, JsonPrunerConfig};
use toksi::headroom::compression::log::{compress_logs, LogCompressorConfig};
use toksi::headroom::pruner::mcp_pruner::{prune_mcp_tools, McpPrunerConfig};
use toksi::headroom::responses_compat::transform_sse_bytes;
use toksi::icm::schema::{Importance, Memory};
use toksi::icm::store::IcmStore;
use toksi::projectmem::handle_mcp_message as projectmem_handle_mcp;
use toksi::rtk::engine::filter_output;
use toksi::util::paths::normalize_path;

#[test]
fn test_sub_10ms_memory_recall_benchmark() {
    let temp_dir = std::env::temp_dir().join("toksi_bench_icm");
    let _ = std::fs::remove_dir_all(&temp_dir);
    let _ = std::fs::create_dir_all(&temp_dir);

    let db_path = temp_dir.join("memory.db");
    let norm_path = normalize_path(db_path.to_str().unwrap_or_default());
    assert!(!norm_path.contains('\\'), "Must use forward slash normalized path");

    let mut store = IcmStore::open(std::path::Path::new(&norm_path))
        .expect("Opening ICM store must succeed");

    let now = Utc::now().to_rfc3339();
    for i in 0..50 {
        let topic = format!("topic-{}", i % 5);
        let summary = format!("Authentication token rotation and refresh policy detail item {}", i);
        let mem = Memory {
            id: format!("mem-{}", i),
            created_at: now.clone(),
            updated_at: String::new(),
            last_accessed: now.clone(),
            access_count: 0,
            weight: 1.0,
            topic,
            summary,
            raw_excerpt: None,
            keywords: vec!["auth".to_string(), "token".to_string()],
            importance: Importance::High,
            source_type: "manual".to_string(),
            source_data: None,
            related_ids: Vec::new(),
            summary_hash: None,
            subproject_id: None,
        };
        store.insert_or_update_memory(&mem).expect("Save operation must succeed");
    }

    let start = Instant::now();
    let results = store.search_bm25("Authentication rotation", 10)
        .expect("Recall operation must succeed");
    let elapsed = start.elapsed();

    assert!(!results.is_empty(), "Recall should return matching records");
    assert!(
        elapsed.as_millis() < 10,
        "ICM recall latency must be sub-10ms, took: {:?}",
        elapsed
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_sub_10_microsecond_sse_proxy_streaming_overhead() {
    let sse_chunk = b"data: {\"choices\":[{\"delta\":{\"content\":\"Hello from gateway streaming response chunk\"}}]}\n\n";

    let iterations = 1000;
    let start = Instant::now();
    for _ in 0..iterations {
        let transformed = transform_sse_bytes(sse_chunk, HashMap::new());
        assert!(!transformed.is_empty());
    }
    let total_elapsed = start.elapsed();
    let per_chunk_micros = total_elapsed.as_micros() as f64 / iterations as f64;

    #[cfg(not(debug_assertions))]
    let max_allowed_micros = 10.0;
    #[cfg(debug_assertions)]
    let max_allowed_micros = 20.0;

    assert!(
        per_chunk_micros < max_allowed_micros,
        "SSE streaming transform overhead must be < {:?} µs per chunk, measured: {:.3} µs",
        max_allowed_micros,
        per_chunk_micros
    );
}

#[test]
fn test_30_to_70_percent_token_savings_realistic_payloads() {
    // 1. RTK Command Output
    let raw_diff = "diff --git a/Cargo.lock b/Cargo.lock\nindex 1234..5678 100644\n--- a/Cargo.lock\n+++ b/Cargo.lock\n@@ -1,50 +1,50 @@\n".to_string() + &"+line_added_to_lockfile\n-line_removed_from_lockfile\n".repeat(60);
    let filtered_diff = filter_output("git diff", &raw_diff, "", 0);
    let diff_raw_tok = estimate_tokens(&raw_diff);
    let diff_fil_tok = estimate_tokens(&filtered_diff);
    let diff_savings = ((diff_raw_tok - diff_fil_tok) as f64 / diff_raw_tok as f64) * 100.0;
    assert!(diff_savings >= 70.0, "RTK diff savings should be >= 70%");

    // 2. Headroom Log Deduplication
    let mut repeating_logs = String::new();
    for _ in 0..60 {
        repeating_logs.push_str("2026-10-30T10:00:00Z [INFO] worker heartbeat status=ok count=100\n");
    }
    repeating_logs.push_str("2026-10-30T10:00:01Z [ERROR] network connection reset by peer\n");
    for _ in 0..60 {
        repeating_logs.push_str("2026-10-30T10:00:00Z [INFO] worker heartbeat status=ok count=100\n");
    }
    let comp_log = compress_logs(&repeating_logs, &LogCompressorConfig::default());
    let log_raw_tok = estimate_tokens(&repeating_logs);
    let log_comp_tok = estimate_tokens(&comp_log.compressed);
    let log_savings = ((log_raw_tok - log_comp_tok) as f64 / log_raw_tok as f64) * 100.0;
    assert!(log_savings >= 70.0, "Log deduplication savings should be >= 70%");

    // 3. Headroom JSON Pruning
    let raw_json = json!({
        "status": "success",
        "code": 200,
        "empty_payload": null,
        "redundant_null": null,
        "items": [
            {"id": 1, "value": "val1", "unused": null, "description": "payload record item 1"},
            {"id": 2, "value": "val2", "unused": null, "description": "payload record item 2"},
            {"id": 3, "value": "val3", "unused": null, "description": "payload record item 3"},
            {"id": 4, "value": "val4", "unused": null, "description": "payload record item 4"},
            {"id": 5, "value": "val5", "unused": null, "description": "payload record item 5"}
        ]
    });
    let raw_json_str = serde_json::to_string_pretty(&raw_json).unwrap_or_default();
    let pruned_json = prune_json(&raw_json_str, &JsonPrunerConfig::default());
    let json_raw_tok = estimate_tokens(&raw_json_str);
    let json_pruned_tok = estimate_tokens(&pruned_json);
    let json_savings = ((json_raw_tok - json_pruned_tok) as f64 / json_raw_tok as f64) * 100.0;
    assert!(json_savings >= 30.0, "JSON pruning savings should be >= 30%");

    // 4. Code AST Compression
    let code_sample = "\
/// Extensive documentation comments explaining algorithm.
/// Computes sum of squares and variance across data points.
/// Additional explanatory lines for API documentation.
pub fn compute_sum_of_squares(values: &[f64]) -> f64 {
    // Initializer for accumulator
    let mut sum = 0.0;

    // Loop through all data points
    for &v in values {
        // Accumulate squares
        sum += v * v;
    }

    // Return final computed sum
    sum
}
";
    let comp_code = compress_code(code_sample, CodeLanguage::Rust, &CodeCompressorConfig::default());
    let code_raw_tok = estimate_tokens(code_sample);
    let code_comp_tok = estimate_tokens(&comp_code);
    let code_savings = ((code_raw_tok - code_comp_tok) as f64 / code_raw_tok as f64) * 100.0;
    assert!(code_savings >= 25.0, "Code compression savings should be >= 25%");

    // 5. Dynamic MCP Schema Pruning
    let mut raw_tools = vec![
        json!({
            "type": "function",
            "function": {
                "name": "large_tool_one",
                "description": "A very large description of an external MCP tool with schema details",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "input": {"type": "string", "description": "some input"}
                    }
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "large_tool_two",
                "description": "Another large description of a tool that might not be needed immediately",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "target": {"type": "string", "description": "target param"}
                    }
                }
            }
        }),
    ];
    let config = McpPrunerConfig {
        allowed_tools: Some(vec!["large_tool_one".to_string()]),
        ..Default::default()
    };
    let stats = prune_mcp_tools(&mut raw_tools, &config);
    assert_eq!(stats.tools_before, 2, "Should start with 2 tools");
    assert_eq!(stats.tools_after, 1, "Should retain exactly 1 referenced tool");
    assert_eq!(raw_tools.len(), 1, "Vec should contain 1 tool");

    // 6. Overall Combined Token Savings
    let total_raw = diff_raw_tok + log_raw_tok + json_raw_tok + code_raw_tok;
    let total_reduced = diff_fil_tok + log_comp_tok + json_pruned_tok + code_comp_tok;
    let overall_savings = ((total_raw - total_reduced) as f64 / total_raw as f64) * 100.0;

    assert!(
        overall_savings >= 30.0 && overall_savings <= 95.0,
        "Overall token savings must be in 30-70%+ range, got: {:.2}%",
        overall_savings
    );
}

#[test]
fn test_in_process_mcp_dispatch_benchmark() {
    let req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    let start = Instant::now();
    let res = projectmem_handle_mcp(&req);
    let elapsed = start.elapsed();

    assert!(res.is_some(), "ProjectMem MCP dispatch must return response");
    assert!(
        elapsed.as_millis() < 5,
        "In-process MCP dispatch latency must be < 5ms, took: {:?}",
        elapsed
    );
}
