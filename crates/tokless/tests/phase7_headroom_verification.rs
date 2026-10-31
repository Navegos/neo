// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/tests/phase7_headroom_verification.rs

use serde_json::json;
use tokless::aimemory::tiered::DisclosureTier;
use tokless::headroom::adaptive_mem::{
    build_adaptive_memory_brief, calculate_headroom, collect_memory_entries,
    inject_adaptive_memory, model_context_limit, select_tier_for_headroom,
};
use tokless::headroom::cache_align::{
    align_system_prompt_to_1024, align_to_1024_tokens, auto_place_anthropic_cache_control,
    estimate_tokens, stabilize_cache_keys, CACHE_ALIGNMENT_TOKEN_BLOCK,
};
use tokless::headroom::compression::code::{
    compress_code, detect_code_language, CodeCompressorConfig, CodeLanguage,
};
use tokless::headroom::compression::json::{
    match_json_span, prune_embedded_json, prune_json, JsonPrunerConfig,
};
use tokless::headroom::compression::log::{
    classify_line, compress_logs, deduplicate_logs, detect_format, LogCompressorConfig,
    LogFormat, LogLevel,
};
use tokless::headroom::compression::{compress_content, detect_content_kind, ContentKind};
use tokless::headroom::responses_compat::transform_sse_bytes;
use std::collections::HashMap;

// ============================================================================
// Requirement 1 & 2: Token Compression Engine (Log, Code, JSON)
// ============================================================================

#[test]
fn test_phase7_log_compression_fatal_error_panic_survival() {
    let raw_logs = r#"
2026-10-31T10:00:00Z [INFO] Service worker initializing database pool
2026-10-31T10:00:01Z [INFO] Retrying connection attempt 1
2026-10-31T10:00:01Z [INFO] Retrying connection attempt 1
2026-10-31T10:00:01Z [INFO] Retrying connection attempt 1
2026-10-31T10:00:02Z [WARN] slow response from backend auth gateway
2026-10-31T10:00:02Z [WARN] slow response from backend auth gateway
2026-10-31T10:00:02Z [WARN] slow response from backend auth gateway
2026-10-31T10:00:03Z [ERROR] failed to connect to database: connection timeout after 3000ms
2026-10-31T10:00:04Z FATAL: worker aborting unrecoverable error in transaction runner
panicked at 'assertion failed: pool.is_active()', src/db/pool.rs:88:5
test result: FAILED. 1 failed; 4 passed
"#;

    let cfg = LogCompressorConfig::default();
    let result = compress_logs(raw_logs, &cfg);
    let comp = &result.compressed;

    // Critical error, fatal, and panic lines MUST survive byte-for-byte
    assert!(
        comp.contains("failed to connect to database: connection timeout after 3000ms"),
        "Error line must survive byte-for-byte"
    );
    assert!(
        comp.contains("FATAL: worker aborting unrecoverable error in transaction runner"),
        "FATAL line must survive byte-for-byte"
    );
    assert!(
        comp.contains("panicked at 'assertion failed: pool.is_active()', src/db/pool.rs:88:5"),
        "Panic line must survive byte-for-byte"
    );
    assert!(
        comp.contains("test result: FAILED. 1 failed; 4 passed"),
        "Summary line must survive"
    );

    // Repeated lines must be collapsed
    assert!(comp.contains("[... repeated 2 times"));
    assert!(result.compressed_line_count < result.original_line_count);

    // Format detection and classification
    assert_eq!(detect_format(&["Compiling tokless v0.1.0", "Finished release"]), LogFormat::Cargo);
    let (level, is_trace, is_summary) = classify_line("FATAL: core dumped");
    assert_eq!(level, LogLevel::Error);
    assert!(!is_trace);
    assert!(!is_summary);

    // Deduplicate logs API
    let dedup = deduplicate_logs("ping\nping\nping\npong");
    assert!(dedup.contains("repeated 2 times"));
    assert!(dedup.contains("pong"));
}

#[test]
fn test_phase7_log_stack_trace_runtime_frames_collapse() {
    let trace = r#"
Traceback (most recent call last):
  File "/usr/lib/python3.11/site-packages/flask/app.py", line 1500, in wsgi_app
  File "/usr/lib/python3.11/site-packages/werkzeug/middleware/dispatcher.py", line 80, in dispatch
  File "/usr/lib/python3.11/site-packages/sqlalchemy/engine/base.py", line 120, in _execute
  File "/app/src/services/billing.py", line 42, in process_invoice
AssertionError: invoice amount must be positive
"#;

    let res = compress_logs(trace, &LogCompressorConfig::default());
    assert!(
        res.compressed.contains("runtime frames collapsed"),
        "Third-party library frames must be collapsed"
    );
    assert!(
        res.compressed.contains("/app/src/services/billing.py"),
        "Application frame must survive"
    );
    assert!(
        res.compressed.contains("AssertionError: invoice amount must be positive"),
        "Terminal exception must survive"
    );
}

#[test]
fn test_phase7_code_ast_aware_compression() {
    let rust_code = r#"
// License boilerplate comment
//! Crate-level doc comment

use std::collections::HashMap;

/// Calculates total memory savings ratio
pub fn calculate_savings(raw: usize, compressed: usize) -> f64 {
    // Check divide by zero
    if raw == 0 {
        return 0.0;
    }

    // Compute percentage reduction
    let diff = (raw - compressed) as f64;
    (diff / raw as f64) * 100.0
}
"#;

    let compressed = compress_code(rust_code, CodeLanguage::Rust, &CodeCompressorConfig::default());

    // Doc comments and code logic must survive
    assert!(compressed.contains("pub fn calculate_savings"));
    assert!(compressed.contains("/// Calculates total memory savings ratio"));
    assert!(compressed.contains("//! Crate-level doc comment"));

    // Standard inline non-doc comments should be pruned
    assert!(!compressed.contains("// License boilerplate comment"));
    assert!(!compressed.contains("// Check divide by zero"));
    assert!(!compressed.contains("// Compute percentage reduction"));

    // Language detection via extension hint and code heuristics
    assert_eq!(detect_code_language("rs"), CodeLanguage::Rust);
    assert_eq!(detect_code_language("pub fn main() { let mut x = 10; }"), CodeLanguage::Rust);
    assert_eq!(detect_code_language("py"), CodeLanguage::Python);
}

#[test]
fn test_phase7_json_structural_minification() {
    let json_payload = r#"{
        "status": "ok",
        "empty_data": null,
        "items": [
            {"id": 1, "name": "first", "extra": null},
            {"id": 2, "name": "second", "extra": null},
            {"id": 3, "name": "third", "extra": null},
            {"id": 4, "name": "fourth", "extra": null},
            {"id": 5, "name": "fifth", "extra": null},
            {"id": 6, "name": "sixth", "extra": null}
        ]
    }"#;

    let pruned = prune_json(json_payload, &JsonPrunerConfig::default());
    let val: serde_json::Value = serde_json::from_str(&pruned).expect("must parse valid JSON");

    assert_eq!(val["status"], "ok");
    assert!(val.get("empty_data").is_none(), "Null fields must be pruned");

    // Array truncation to max_array_items (default 4) + omission note
    let items = val["items"].as_array().expect("items must be array");
    assert_eq!(items.len(), 5); // 4 kept + 1 omission note
    assert!(items.last().unwrap().as_str().unwrap().contains("omitted by tokless"));

    // Embedded JSON pruning and span matching
    let text_with_json = "header text {\"debug\": null, \"active\": true} footer";
    let pruned_embedded = prune_embedded_json(text_with_json, &JsonPrunerConfig::default());
    assert!(pruned_embedded.contains("{\"active\":true}"));

    let span_sample = "prefix {\"k\": 1} suffix";
    let start_idx = span_sample.find('{').unwrap();
    let end_idx = match_json_span(span_sample, start_idx).unwrap();
    assert_eq!(&span_sample[start_idx..end_idx], "{\"k\": 1}");

    // Unified content compression
    assert_eq!(detect_content_kind("{\"k\": null}", None), ContentKind::Json);
    let comp_json = compress_content("{\"k\": null, \"alive\": true}", None);
    assert_eq!(comp_json, "{\"alive\":true}");
}

// ============================================================================
// Requirement 3: 1024-Token Prompt Cache Alignment
// ============================================================================

#[test]
fn test_phase7_cache_alignment_to_1024_tokens() {
    let system_instructions = "You are a professional assistant specialized in systems engineering.";
    let aligned = align_to_1024_tokens(system_instructions);

    assert!(
        aligned.starts_with(system_instructions),
        "Original system prompt must remain as prefix"
    );
    assert!(
        aligned.contains("<!-- tokless:cache-align pad_tokens="),
        "Alignment comment marker must be injected"
    );

    let tokens = estimate_tokens(&aligned);
    assert!(
        tokens >= CACHE_ALIGNMENT_TOKEN_BLOCK && tokens <= CACHE_ALIGNMENT_TOKEN_BLOCK + 10,
        "Aligned tokens must match 1,024-token boundary (got: {})",
        tokens
    );

    // Full system payload alignment
    let mut payload = json!({
        "system": system_instructions,
        "messages": [
            {"role": "user", "content": "How do I profile Rust?"}
        ]
    });
    align_system_prompt_to_1024(&mut payload);
    let updated_system = payload["system"].as_str().unwrap();
    assert!(updated_system.contains("<!-- tokless:cache-align pad_tokens="));
}

#[test]
fn test_phase7_anthropic_ephemeral_cache_control() {
    let mut payload = json!({
        "model": "claude-3-7-sonnet-20250219",
        "tools": [
            {"name": "bash", "description": "Execute bash commands"},
            {"name": "read_file", "description": "Read file contents"}
        ],
        "messages": [
            {"role": "user", "content": "Review this code"}
        ]
    });

    let placed = auto_place_anthropic_cache_control(&mut payload);
    assert!(placed, "Cache control must be auto-placed");

    // Last tool should receive cache_control: {"type": "ephemeral"}
    let last_tool = &payload["tools"][1];
    assert_eq!(last_tool["cache_control"]["type"], "ephemeral");

    // Second placement call should be idempotent and return false
    let placed_again = auto_place_anthropic_cache_control(&mut payload);
    assert!(!placed_again, "Idempotent placement must not re-insert or double-tag");
}

#[test]
fn test_phase7_stabilize_cache_keys() {
    let mut payload = json!({
        "tools": [
            {
                "type": "function",
                "function": {
                    "name": "edit_file",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "target": {"type": "string"},
                            "path": {"type": "string"},
                            "replacement": {"type": "string"}
                        }
                    }
                }
            }
        ]
    });

    stabilize_cache_keys(&mut payload);
    let props = payload["tools"][0]["function"]["parameters"]["properties"].as_object().unwrap();
    let keys: Vec<&String> = props.keys().collect();
    // BTreeMap keys must be sorted alphabetically: ["path", "replacement", "target"]
    assert_eq!(keys, vec!["path", "replacement", "target"]);
}

// ============================================================================
// Requirement 4: Adaptive Memory Governor (L0, L1, L2 Dynamic Scaling)
// ============================================================================

#[test]
fn test_phase7_adaptive_memory_governor_thresholds() {
    // Model limits
    assert_eq!(model_context_limit("claude-3-7-sonnet"), 200_000);
    assert_eq!(model_context_limit("gpt-4o"), 128_000);
    assert_eq!(model_context_limit("gemini-1.5-pro"), 1_000_000);

    // Context Free > 92% -> L2 (Full content, budget 1500)
    let headroom_high = calculate_headroom(10_000, 200_000); // 95% free
    let (tier_high, budget_high) = select_tier_for_headroom(headroom_high);
    assert_eq!(tier_high, DisclosureTier::L2);
    assert_eq!(budget_high, 1500);

    // Context Free 70%..=92% -> L1 (Abstracts, budget 600)
    let headroom_mid = calculate_headroom(40_000, 200_000); // 80% free
    let (tier_mid, budget_mid) = select_tier_for_headroom(headroom_mid);
    assert_eq!(tier_mid, DisclosureTier::L1);
    assert_eq!(budget_mid, 600);

    // Context Free 30%..=70% -> L1 (Compressed, budget 300)
    let headroom_tight = calculate_headroom(100_000, 200_000); // 50% free
    let (tier_tight, budget_tight) = select_tier_for_headroom(headroom_tight);
    assert_eq!(tier_tight, DisclosureTier::L1);
    assert_eq!(budget_tight, 300);

    // Context Free < 30% -> L0 (TOC only, budget 100)
    let headroom_crit = calculate_headroom(180_000, 200_000); // 10% free
    let (tier_crit, budget_crit) = select_tier_for_headroom(headroom_crit);
    assert_eq!(tier_crit, DisclosureTier::L0);
    assert_eq!(budget_crit, 100);
}

#[test]
fn test_phase7_adaptive_memory_brief_injection() {
    // Memory entries collection from projectmem, aimemory, and ICM
    let entries = collect_memory_entries();
    assert!(!entries.is_empty(), "Memory collection must find entries or fallback");

    // High headroom generates L2 brief
    let (brief_l2, tier_l2) = build_adaptive_memory_brief(5_000, 200_000);
    assert_eq!(tier_l2, DisclosureTier::L2);
    assert!(brief_l2.contains("<!-- tokless:adaptive-mem-governor"));
    assert!(brief_l2.contains("tier=L2"));

    // Critical headroom generates L0 brief
    let (brief_l0, tier_l0) = build_adaptive_memory_brief(185_000, 200_000);
    assert_eq!(tier_l0, DisclosureTier::L0);
    assert!(brief_l0.contains("tier=L0"));

    // Payload injection into Anthropic request
    let mut payload = json!({
        "model": "claude-3-5-sonnet",
        "system": "Base developer instructions",
        "messages": [
            {"role": "user", "content": "Refactor codebase"}
        ]
    });

    let injected = inject_adaptive_memory(&mut payload);
    assert!(injected);
    let system_str = payload["system"].as_str().unwrap();
    assert!(system_str.contains("tokless:adaptive-mem-governor"));
    assert!(system_str.contains("Base developer instructions"));
}

// ============================================================================
// Requirement 5: Streaming & BYOK Reverse Proxy Compatibility
// ============================================================================

#[test]
fn test_phase7_sse_stream_transformer() {
    let sse_event = b"event: message_delta\ndata: {\"type\": \"message_delta\", \"delta\": {\"stop_reason\": \"end_turn\"}}\n\n";
    let transformed = transform_sse_bytes(sse_event, HashMap::new());
    assert!(!transformed.is_empty());
    let s = String::from_utf8_lossy(&transformed);
    assert!(s.contains("message_delta"));
}
