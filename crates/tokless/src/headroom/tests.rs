// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/headroom/tests.rs

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::aimemory::tiered::DisclosureTier;
    use crate::headroom::adaptive_mem::{
        build_adaptive_memory_brief, calculate_headroom, inject_adaptive_memory,
        model_context_limit, select_tier_for_headroom,
    };
    use crate::headroom::cache_align::{
        align_to_1024_tokens, auto_place_anthropic_cache_control, estimate_tokens,
        stabilize_cache_keys,
    };
    use crate::headroom::compression::code::{
        compress_code, detect_code_language, CodeCompressorConfig, CodeLanguage,
    };
    use crate::headroom::compression::json::{
        match_json_span, prune_embedded_json, prune_json, JsonPrunerConfig,
    };
    use crate::headroom::compression::log::{
        classify_line, compress_logs, deduplicate_logs, detect_format, LogCompressorConfig,
        LogFormat, LogLevel,
    };
    use crate::headroom::compression::{compress_content, detect_content_kind, ContentKind};

    // ─────────────────────────────────────────────────────────────────────────────
    // 1. Lossless Log Deduplication Tests
    // ─────────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_log_format_detection() {
        let cargo_log = vec![
            "   Compiling serde v1.0.210",
            "   Compiling tokio v1.40.0",
            "error[E0308]: mismatched types",
        ];
        assert_eq!(detect_format(&cargo_log), LogFormat::Cargo);

        let pytest_log = vec![
            "=== FAILURES ===",
            "test_something_broken",
            "pytest session starts",
        ];
        assert_eq!(detect_format(&pytest_log), LogFormat::Pytest);

        let npm_log = vec![
            "npm ERR! code ELIFECYCLE",
            "npm ERR! errno 1",
            "npm warn deprecated core-js",
        ];
        assert_eq!(detect_format(&npm_log), LogFormat::Npm);
    }

    #[test]
    fn test_log_line_classification() {
        let (err_level, is_trace, is_summary) = classify_line("error[E0425]: cannot find value");
        assert_eq!(err_level, LogLevel::Error);
        assert!(!is_trace);
        assert!(!is_summary);

        let (fatal_level, _, _) = classify_line("FATAL: database connection pool exhausted");
        assert_eq!(fatal_level, LogLevel::Error);

        let (_, is_trace_line, _) = classify_line("  File \"app/service.py\", line 42, in handle");
        assert!(is_trace_line);

        let (_, _, is_sum) = classify_line("test result: FAILED. 2 failed; 10 passed");
        assert!(is_sum);
    }

    #[test]
    fn test_fatal_panic_error_survival_byte_for_byte() {
        let raw = "\
Compiling lib v0.1.0
Compiling lib v0.1.0
Compiling lib v0.1.0
error[E0382]: use of moved value: `data`
panicked at 'index out of bounds', src/lib.rs:12:9
FATAL: process terminated ungracefully
test result: FAILED. 1 failed; 5 passed";

        let result = compress_logs(raw, &LogCompressorConfig::default());
        let comp = &result.compressed;

        assert!(comp.contains("error[E0382]: use of moved value: `data`"));
        assert!(comp.contains("panicked at 'index out of bounds', src/lib.rs:12:9"));
        assert!(comp.contains("FATAL: process terminated ungracefully"));
        assert!(comp.contains("test result: FAILED. 1 failed; 5 passed"));
        assert!(comp.contains("[... repeated 2 times"));
    }

    #[test]
    fn test_log_stack_trace_runtime_frame_collapse() {
        let trace = "\
Traceback (most recent call last):
  File \"/usr/lib/python3.11/site-packages/flask/app.py\", line 100, in wsgi_app
  File \"/usr/lib/python3.11/site-packages/werkzeug/middleware/dispatcher.py\", line 50, in dispatch
  File \"/app/backend/controller.py\", line 33, in handle_request
ValueError: invalid ID parameter";

        let res = compress_logs(trace, &LogCompressorConfig::default());
        assert!(res.compressed.contains("runtime frames collapsed"));
        assert!(res.compressed.contains("/app/backend/controller.py"));
        assert!(res.compressed.contains("ValueError: invalid ID parameter"));
    }

    #[test]
    fn test_deduplicate_logs_api() {
        let logs = "Running heartbeat check\nRunning heartbeat check\nRunning heartbeat check\nFinished";
        let out = deduplicate_logs(logs);
        assert!(out.contains("repeated 2 times"));
        assert!(out.contains("Finished"));
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // 2. AST-Aware Code Compression Tests
    // ─────────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_detect_code_language_heuristic() {
        assert_eq!(detect_code_language("pub fn main() { let mut x = 10; }"), CodeLanguage::Rust);
        assert_eq!(detect_code_language("def run_task(args):\n    import sys\n    return 0"), CodeLanguage::Python);
        assert_eq!(detect_code_language("interface Config { port: string; }\nconst start = () => {};"), CodeLanguage::Typescript);
        assert_eq!(detect_code_language("package main\nimport \"fmt\"\nfunc main() {}"), CodeLanguage::Go);
        assert_eq!(detect_code_language("#include <iostream>\nclass Engine {};"), CodeLanguage::Cpp);
    }

    #[test]
    fn test_rust_ast_aware_compression() {
        let rust_code = r#"
        // Standard copyright header
        //! Crate documentation

        use std::collections::HashMap;

        /// Computes factorial recursively
        pub fn factorial(n: u64) -> u64 {
            // Base case
            if n <= 1 {
                return 1;
            }


            // Recursive call
            n * factorial(n - 1)
        }
        "#;

        let res = compress_code(rust_code, CodeLanguage::Rust, &CodeCompressorConfig::default());
        assert!(res.contains("pub fn factorial(n: u64) -> u64"));
        assert!(res.contains("/// Computes factorial recursively"));
        assert!(res.contains("//! Crate documentation"));
        assert!(!res.contains("// Standard copyright header"));
        assert!(!res.contains("// Base case"));
        assert!(!res.contains("\n\n\n"));
    }

    #[test]
    fn test_python_ast_aware_compression() {
        let py_code = r#"
        # File: calculate.py

        def calculate_area(radius: float) -> float:
            # Formula is pi * r^2
            import math
            return math.pi * (radius ** 2)
        "#;

        let res = compress_code(py_code, CodeLanguage::Python, &CodeCompressorConfig::default());
        assert!(res.contains("def calculate_area(radius: float) -> float:"));
        assert!(res.contains("return math.pi * (radius ** 2)"));
        assert!(!res.contains("# File: calculate.py"));
        assert!(!res.contains("# Formula is pi * r^2"));
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // 3. Structural JSON Pruner Tests
    // ─────────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_json_pruning_and_minification() {
        let pretty = r#"{
            "status": "success",
            "redundant_null": null,
            "data": {
                "name": "headroom",
                "version": 2,
                "empty_field": null
            }
        }"#;

        let pruned = prune_json(pretty, &JsonPrunerConfig::default());
        let parsed: serde_json::Value = serde_json::from_str(&pruned).unwrap();
        assert_eq!(parsed["status"], "success");
        assert_eq!(parsed["data"]["name"], "headroom");
        assert_eq!(parsed["data"]["version"], 2);
        assert!(parsed.get("redundant_null").is_none());
        assert!(parsed["data"].get("empty_field").is_none());
        assert!(!pruned.contains(' '));
    }

    #[test]
    fn test_json_array_truncation() {
        let payload = r#"{"logs": ["line1", "line2", "line3", "line4", "line5", "line6", "line7"]}"#;
        let mut cfg = JsonPrunerConfig::default();
        cfg.max_array_items = 2;

        let pruned = prune_json(payload, &cfg);
        assert!(pruned.contains(r#""logs":["line1","line2","[... 5 items omitted by tokless]"]"#));
    }

    #[test]
    fn test_embedded_json_detection_and_pruning() {
        let text = "Result from CLI command:\n{\n  \"ok\": true,\n  \"unused\": null\n}\nSuccess verified.";
        let res = prune_embedded_json(text, &JsonPrunerConfig::default());
        assert!(res.contains(r#"{"ok":true}"#));
        assert!(!res.contains("\"unused\": null"));
        assert!(res.contains("Success verified."));
    }

    #[test]
    fn test_match_json_span() {
        let sample = "Prefix { \"a\": [1, 2, {\"nested\": true}] } Postfix";
        let start = sample.find('{').unwrap();
        let end = match_json_span(sample, start).unwrap();
        assert_eq!(&sample[start..end], "{ \"a\": [1, 2, {\"nested\": true}] }");
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // 4. 1024-Token Prompt Cache Alignment Tests
    // ─────────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_estimate_tokens() {
        let short = "Hello world";
        assert!(estimate_tokens(short) >= 2);

        let long = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(40);
        let count = estimate_tokens(&long);
        assert!(count > 100);
    }

    #[test]
    fn test_align_to_1024_tokens_exact_boundary() {
        let prompt = "System instruction block. ".repeat(10);
        let aligned = align_to_1024_tokens(&prompt);
        assert!(aligned.contains("<!-- tokless:cache-align pad_tokens="));
        let tokens = estimate_tokens(&aligned);
        assert!(tokens >= 1024);
    }

    #[test]
    fn test_anthropic_cache_control_placement_on_tools() {
        let mut body = json!({
            "model": "claude-3-5-sonnet-20241022",
            "tools": [
                {"name": "cmd1", "description": "run cmd1"},
                {"name": "cmd2", "description": "run cmd2"}
            ],
            "messages": [
                {"role": "user", "content": "Help me build this"}
            ]
        });

        let changed = auto_place_anthropic_cache_control(&mut body);
        assert!(changed);
        assert_eq!(body["tools"][1]["cache_control"]["type"], "ephemeral");

        // Idempotency: Second call should not touch already placed marker
        let changed2 = auto_place_anthropic_cache_control(&mut body);
        assert!(!changed2);
    }

    #[test]
    fn test_anthropic_cache_control_placement_on_system_array() {
        let mut body = json!({
            "model": "claude-3-7-sonnet",
            "system": [
                {"type": "text", "text": "Base instructions"},
                {"type": "text", "text": "Rules block"}
            ],
            "messages": [
                {"role": "user", "content": "Hello"}
            ]
        });

        let changed = auto_place_anthropic_cache_control(&mut body);
        assert!(changed);
        assert_eq!(body["system"][1]["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn test_stabilize_cache_keys() {
        let mut body = json!({
            "tools": [
                {
                    "type": "function",
                    "function": {
                        "name": "sample_tool",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "zeta": {"type": "string"},
                                "alpha": {"type": "number"},
                                "gamma": {"type": "boolean"}
                            }
                        }
                    }
                }
            ]
        });

        stabilize_cache_keys(&mut body);
        let props = body["tools"][0]["function"]["parameters"]["properties"].as_object().unwrap();
        let keys: Vec<&String> = props.keys().collect();
        assert_eq!(keys, vec!["alpha", "gamma", "zeta"]);
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // 5. Dynamic Memory Governor Tests
    // ─────────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_model_context_limits() {
        assert_eq!(model_context_limit("claude-3-5-sonnet"), 200_000);
        assert_eq!(model_context_limit("claude-3-7-sonnet"), 200_000);
        assert_eq!(model_context_limit("gpt-4o"), 128_000);
        assert_eq!(model_context_limit("gemini-1.5-pro"), 1_000_000);
    }

    #[test]
    fn test_calculate_headroom() {
        // 20,000 used of 200,000 => 90% free
        let headroom = calculate_headroom(20_000, 200_000);
        assert!((headroom - 0.90).abs() < 0.001);

        // 160,000 used of 200,000 => 20% free
        let low_headroom = calculate_headroom(160_000, 200_000);
        assert!((low_headroom - 0.20).abs() < 0.001);
    }

    #[test]
    fn test_tier_scaling_selection() {
        let (tier_max, budget_max) = select_tier_for_headroom(0.96);
        assert_eq!(tier_max, DisclosureTier::L2);
        assert_eq!(budget_max, 1500);

        let (tier_high, budget_high) = select_tier_for_headroom(0.85);
        assert_eq!(tier_high, DisclosureTier::L1);
        assert_eq!(budget_high, 600);

        let (tier_mid, budget_mid) = select_tier_for_headroom(0.45);
        assert_eq!(tier_mid, DisclosureTier::L1);
        assert_eq!(budget_mid, 300);

        let (tier_low, budget_low) = select_tier_for_headroom(0.15);
        assert_eq!(tier_low, DisclosureTier::L0);
        assert_eq!(budget_low, 100);
    }

    #[test]
    fn test_build_adaptive_memory_brief() {
        // High headroom (> 92%) scales up to L2 (Full content)
        let (brief_l2, tier_l2) = build_adaptive_memory_brief(5_000, 200_000);
        assert_eq!(tier_l2, DisclosureTier::L2);
        assert!(brief_l2.contains("<!-- tokless:adaptive-mem-governor"));
        assert!(brief_l2.contains("tier=L2"));

        // Moderate headroom (70%..=92%) scales to L1 (Abstracts)
        let (brief_l1, tier_l1) = build_adaptive_memory_brief(40_000, 200_000);
        assert_eq!(tier_l1, DisclosureTier::L1);
        assert!(brief_l1.contains("tier=L1"));

        // Low headroom (< 30%) scales to L0 (TOC)
        let (brief_l0, tier_l0) = build_adaptive_memory_brief(180_000, 200_000);
        assert_eq!(tier_l0, DisclosureTier::L0);
        assert!(brief_l0.contains("tier=L0"));
    }

    #[test]
    fn test_inject_adaptive_memory_into_anthropic_payload() {
        let mut body = json!({
            "model": "claude-3-5-sonnet-20241022",
            "system": "Original prompt",
            "messages": [
                {"role": "user", "content": "Refactor codebase"}
            ]
        });

        let injected = inject_adaptive_memory(&mut body);
        assert!(injected);
        let sys = body["system"].as_str().unwrap();
        assert!(sys.contains("tokless:adaptive-mem-governor"));
        assert!(sys.contains("Original prompt"));
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // 6. Unified Content Compressor Tests
    // ─────────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_detect_and_compress_content() {
        let json_data = "{\n  \"key\": \"value\",\n  \"null_key\": null\n}";
        assert_eq!(detect_content_kind(json_data, None), ContentKind::Json);
        assert_eq!(compress_content(json_data, None), r#"{"key":"value"}"#);

        let code_data = "pub fn test_add() {\n    // Inline comment\n    assert_eq!(1 + 1, 2);\n}";
        match detect_content_kind(code_data, Some("rs")) {
            ContentKind::Code(CodeLanguage::Rust) => {}
            _ => panic!("failed to detect rust code"),
        }
        let comp_code = compress_content(code_data, Some("rs"));
        assert!(!comp_code.contains("// Inline comment"));
        assert!(comp_code.contains("pub fn test_add"));
    }
}
