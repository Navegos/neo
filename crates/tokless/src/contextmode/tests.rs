// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/contextmode/tests.rs
// created: 2026-10-27

#[cfg(test)]
mod tests {
    use std::path::Path;
    use serde_json::json;

    use crate::contextmode::fetch::{html_to_markdown, ssrf_guard};
    use super::super::filter::{byte_safe_prefix, process_sandbox_output, slice_lines};
    use super::super::fts::{
        chunk_json, chunk_markdown, chunk_plain_text, sanitize_query, sanitize_trigram_query,
        ContentStore, SearchOptions,
    };
    use super::super::mcp::ContextMcpServer;
    use super::super::sandbox::{ExecRequest, PolyglotSandbox};
    use super::super::security::{check_boundary, check_command_deny, check_path_deny};

    #[test]
    fn test_security_command_deny() {
        assert!(check_command_deny("rm -rf /").is_some());
        assert!(check_command_deny("rm -rf /*").is_some());
        assert!(check_command_deny("mkfs.ext4 /dev/sda1").is_some());
        assert!(check_command_deny(":(){ :|:& };:").is_some());
        assert!(check_command_deny("shutdown -h now").is_some());

        // Safe commands allowed
        assert!(check_command_deny("cargo test").is_none());
        assert!(check_command_deny("git status").is_none());
        assert!(check_command_deny("ls -la").is_none());
    }

    #[test]
    fn test_security_path_deny() {
        assert!(check_path_deny(Path::new(".env")).is_some());
        assert!(check_path_deny(Path::new("config/.env.local")).is_some());
        assert!(check_path_deny(Path::new(".ssh/id_rsa")).is_some());
        assert!(check_path_deny(Path::new("certs/server.key")).is_some());
        assert!(check_path_deny(Path::new(".aws/credentials")).is_some());

        // Normal files allowed
        assert!(check_path_deny(Path::new("src/main.rs")).is_none());
        assert!(check_path_deny(Path::new("package.json")).is_none());
        assert!(check_path_deny(Path::new("README.md")).is_none());
    }

    #[test]
    fn test_security_boundary() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let inside = root.join("src").join("lib.rs");

        assert!(check_boundary(&inside, root).is_ok());

        let outside = Path::new("..").join("secret.txt");
        assert!(check_boundary(&outside, root).is_err());
    }

    #[test]
    fn test_filter_slicing() {
        let mut lines = Vec::new();
        for i in 1..=50 {
            lines.push(format!("line {}", i));
        }
        let text = lines.join("\n");

        let sliced = slice_lines(&text, 10);
        assert!(sliced.contains("line 1"));
        assert!(sliced.contains("line 5"));
        assert!(sliced.contains("omitted by context-mode"));
        assert!(sliced.contains("line 50"));

        let short_text = "line 1\nline 2";
        assert_eq!(slice_lines(short_text, 10), short_text);
    }

    #[test]
    fn test_filter_byte_safe_prefix() {
        let text = "hello world";
        assert_eq!(byte_safe_prefix(text, 5), "hello");
        assert_eq!(byte_safe_prefix(text, 50), text);

        let multi = "こんにちは"; // 3 bytes per char
        assert_eq!(byte_safe_prefix(multi, 4), "こ");
        assert_eq!(byte_safe_prefix(multi, 6), "こん");
    }

    #[test]
    fn test_filter_process_sandbox_output_rtk() {
        // Output with cargo test failure format
        let raw = "running 2 tests\ntest test_ok ... ok\ntest test_bad ... FAILED\n\n\x1b[31mfailures:\x1b[0m\n    test_bad\n\ntest result: FAILED. 1 passed; 1 failed";
        let processed = process_sandbox_output(Some("cargo test"), raw, 10, 1024);
        assert!(!processed.contains("\x1b[31m")); // ANSI stripped
        assert!(processed.contains("FAILED"));
    }

    #[test]
    fn test_fts_query_sanitizer() {
        let q = sanitize_query("error in *auth* (jwt) OR token", "AND");
        assert!(q.contains("\"auth\""));
        assert!(q.contains("\"jwt\""));
        assert!(q.contains("\"token\""));
        assert!(!q.contains('*'));
        assert!(!q.contains('('));

        let tri = sanitize_trigram_query("foo bar baz", "AND");
        assert!(tri.contains("\"foo\""));
        assert!(tri.contains("\"bar\""));
    }

    #[test]
    fn test_fts_chunk_markdown() {
        let doc = "# Title\n\nIntroduction paragraph.\n\n## Subheading\n\n```rust\nfn main() {}\n```\n\nMore text.";
        let chunks = chunk_markdown(doc, 1024);

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].title, "Title");
        assert!(chunks[0].content.contains("Introduction paragraph"));

        assert_eq!(chunks[1].title, "Subheading");
        assert!(chunks[1].has_code);
        assert!(chunks[1].content.contains("fn main()"));
    }

    #[test]
    fn test_fts_chunk_plain_text() {
        let text = "Section 1\nContent A\n\nSection 2\nContent B\n\nSection 3\nContent C";
        let chunks = chunk_plain_text(text, 1024);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].title, "Section 1");
        assert_eq!(chunks[1].title, "Section 2");
    }

    #[test]
    fn test_fts_chunk_json() {
        let val = json!({
            "user": {
                "name": "Alice",
                "age": 30
            },
            "items": ["apple", "banana"]
        });
        let chunks = chunk_json(&val, 1024);
        assert!(!chunks.is_empty());
        assert!(chunks.iter().any(|c| c.title.contains("user")));
        assert!(chunks.iter().any(|c| c.title.contains("items")));
    }

    #[test]
    fn test_fts_store_index_and_bm25_search() {
        let mut store = ContentStore::open_in_memory().unwrap();

        let doc1 = "# Authentication\n\nJWT tokens are validated using the HS256 algorithm with shared secret.";
        let doc2 = "# Database\n\nPostgreSQL connection pooling is managed via pgx with max 20 connections.";

        store.index(doc1, "auth-docs", Some("markdown"), None).unwrap();
        store.index(doc2, "db-docs", Some("markdown"), None).unwrap();

        // Search for JWT
        let res1 = store.search(&SearchOptions {
            query: "jwt algorithm".to_string(),
            limit: 5,
            source: None,
            content_type: None,
            session_id: None,
        }).unwrap();

        assert!(!res1.is_empty());
        assert_eq!(res1[0].label, "auth-docs");
        assert!(res1[0].content.contains("HS256"));

        // Search for Database
        let res2 = store.search(&SearchOptions {
            query: "postgresql connection".to_string(),
            limit: 5,
            source: None,
            content_type: None,
            session_id: None,
        }).unwrap();

        assert!(!res2.is_empty());
        assert_eq!(res2[0].label, "db-docs");

        // Purge
        let purged = store.purge_all().unwrap();
        assert_eq!(purged, 2);

        let res3 = store.search(&SearchOptions {
            query: "jwt".to_string(),
            limit: 5,
            source: None,
            content_type: None,
            session_id: None,
        }).unwrap();
        assert!(res3.is_empty());
    }

    #[test]
    fn test_ssrf_guard() {
        assert!(ssrf_guard("https://example.com/api").is_ok());
        assert!(ssrf_guard("http://api.github.com").is_ok());

        // Blocked endpoints
        assert!(ssrf_guard("http://127.0.0.1:8080").is_err());
        assert!(ssrf_guard("http://localhost:3000").is_err());
        assert!(ssrf_guard("http://169.254.169.254/latest/meta-data").is_err());
        assert!(ssrf_guard("http://10.0.0.1/admin").is_err());
        assert!(ssrf_guard("http://192.168.1.1").is_err());
        assert!(ssrf_guard("file:///etc/passwd").is_err());
    }

    #[test]
    fn test_html_to_markdown() {
        let html = r#"
            <html>
                <head>
                    <title>Test Page</title>
                    <script>alert(1);</script>
                    <style>.foo { color: red; }</style>
                </head>
                <body>
                    <h1>Main Header</h1>
                    <p>This is a paragraph with <a href="https://example.com">a link</a> and <code>code</code>.</p>
                    <pre><code>fn test() {
    assert!(true);
}</code></pre>
                    <ul>
                        <li>Item 1</li>
                        <li>Item 2</li>
                    </ul>
                </body>
            </html>
        "#;

        let md = html_to_markdown(html);
        assert!(!md.contains("alert(1)"));
        assert!(!md.contains(".foo"));
        assert!(md.contains("# Main Header"));
        assert!(md.contains("[a link](https://example.com)"));
        assert!(md.contains("`code`"));
        assert!(md.contains("fn test()"));
        assert!(md.contains("* Item 1"));
    }

    #[test]
    fn test_sandbox_execution() {
        let sandbox = PolyglotSandbox::new();
        let code = if cfg!(windows) {
            "Write-Output 'hello from sandbox'"
        } else {
            "echo 'hello from sandbox'"
        };

        let req = ExecRequest {
            language: "shell".to_string(),
            code: code.to_string(),
            cwd: None,
            timeout_secs: Some(10),
            max_lines: Some(50),
            max_bytes: None,
        };

        let res = sandbox.execute(&req).unwrap();
        assert_eq!(res.exit_code, 0);
        assert!(res.stdout.contains("hello from sandbox"));
    }

    #[test]
    fn test_mcp_server_tools() {
        let temp = tempfile::tempdir().unwrap();
        let server = ContextMcpServer::new(Some(temp.path().to_path_buf()));

        // 1. ctx_index
        let index_out = server.execute_tool(
            "ctx_index",
            &json!({
                "content": "# Test Doc\n\nContent for testing MCP server index.",
                "source": "unit-test"
            }),
        );
        assert!(index_out.contains("Indexed 'unit-test'"));

        // 2. ctx_search
        let search_out = server.execute_tool(
            "ctx_search",
            &json!({
                "query": "testing"
            }),
        );
        assert!(search_out.contains("Results for query 'testing'"));
        assert!(search_out.contains("Test Doc"));

        // 3. ctx_stats
        let stats_out = server.execute_tool("ctx_stats", &json!({}));
        assert!(stats_out.contains("Context-Mode Session Statistics"));

        // 4. ctx_doctor
        let doc_out = server.execute_tool("ctx_doctor", &json!({}));
        assert!(doc_out.contains("Context-Mode Doctor Diagnostics"));

        // 5. ctx_purge
        let purge_out = server.execute_tool("ctx_purge", &json!({}));
        assert!(purge_out.contains("Successfully purged"));
    }
}
