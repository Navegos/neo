// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/tests/phase5_codegraph_verification.rs

use std::fs;
use std::path::Path;
use serde_json::json;
use tempfile::tempdir;

use tokless::codegraph::cache::{content_hash, SymbolCache};
use tokless::codegraph::graph::CodeGraphEngine;
use tokless::codegraph::kernel::{extract_file, SymbolEdge, SymbolNode};
use tokless::codegraph::mcp::handle_mcp_message;
use tokless::codegraph::walker::{detect_language, walk_project_files, WalkedFile};
use tokless::codegraph::{resolve_coordinate, run_graph_cli, CodeGraph};
use tokless::projectmem::{append_event, auto_resolve_location, initialize, read_events, Event};

#[test]
fn test_phase5_tree_sitter_multilang_parsing() {
    // 1. Rust parsing
    let rust_code = r#"
/// Authentication middleware service
pub struct AuthService {
    pub secret: String,
}

impl AuthService {
    pub fn new(secret: String) -> Self {
        AuthService { secret }
    }

    pub fn validate_jwt(&self, token: &str) -> bool {
        verify_signature(token, &self.secret)
    }
}

fn verify_signature(token: &str, secret: &str) -> bool {
    !token.is_empty() && !secret.is_empty()
}
"#;
    let rust_ext = extract_file("src/auth.rs", rust_code, "rust").expect("extract rust");
    assert!(!rust_ext.nodes.is_empty());
    let rust_names: Vec<&str> = rust_ext.nodes.iter().map(|n| n.name.as_str()).collect();
    assert!(rust_names.contains(&"AuthService"));
    assert!(rust_names.contains(&"new"));
    assert!(rust_names.contains(&"validate_jwt"));
    assert!(rust_names.contains(&"verify_signature"));

    // 2. TypeScript / JavaScript parsing
    let ts_code = r#"
export interface TokenPayload {
    sub: string;
    exp: number;
}

export class JwtValidator {
    private secret: string;

    constructor(secret: string) {
        this.secret = secret;
    }

    public verify(token: string): TokenPayload | null {
        if (!this.checkExpiry(token)) {
            return null;
        }
        return { sub: "user123", exp: 9999999999 };
    }

    private checkExpiry(token: string): boolean {
        return token.length > 0;
    }
}
"#;
    let ts_ext = extract_file("src/jwt.ts", ts_code, "typescript").expect("extract ts");
    assert!(!ts_ext.nodes.is_empty());
    let ts_names: Vec<&str> = ts_ext.nodes.iter().map(|n| n.name.as_str()).collect();
    assert!(ts_names.contains(&"TokenPayload"));
    assert!(ts_names.contains(&"JwtValidator"));
    assert!(ts_names.contains(&"verify"));
    assert!(ts_names.contains(&"checkExpiry"));

    // 3. Python parsing
    let py_code = r#"
class DatabasePool:
    """Manages active PostgreSQL connections."""
    def __init__(self, max_conn: int = 10):
        self.max_conn = max_conn

    def acquire(self):
        return self._create_connection()

    def _create_connection(self):
        return True
"#;
    let py_ext = extract_file("db.py", py_code, "python").expect("extract python");
    assert!(!py_ext.nodes.is_empty());
    let py_names: Vec<&str> = py_ext.nodes.iter().map(|n| n.name.as_str()).collect();
    assert!(py_names.contains(&"DatabasePool"));
    assert!(py_names.contains(&"acquire"));
    assert!(py_names.contains(&"_create_connection"));

    // 4. Go parsing
    let go_code = r#"
package router

type Router struct {
    routes map[string]string
}

func NewRouter() *Router {
    return &Router{routes: make(map[string]string)}
}

func (r *Router) Handle(path string, handler string) {
    r.routes[path] = handler
}
"#;
    let go_ext = extract_file("router.go", go_code, "go").expect("extract go");
    assert!(!go_ext.nodes.is_empty());
    let go_names: Vec<&str> = go_ext.nodes.iter().map(|n| n.name.as_str()).collect();
    assert!(go_names.contains(&"Router"));
    assert!(go_names.contains(&"NewRouter"));
    assert!(go_names.contains(&"Handle"));

    // 5. C/C++ parsing
    let cpp_code = r#"
#include <string>

class MemoryGovernor {
public:
    MemoryGovernor(size_t limit) : max_bytes(limit) {}
    bool check_budget(size_t requested) {
        return requested <= max_bytes;
    }
private:
    size_t max_bytes;
};
"#;
    let cpp_ext = extract_file("governor.cpp", cpp_code, "cpp").expect("extract cpp");
    assert!(!cpp_ext.nodes.is_empty());
    let cpp_names: Vec<&str> = cpp_ext.nodes.iter().map(|n| n.name.as_str()).collect();
    assert!(cpp_names.contains(&"MemoryGovernor"));
    assert!(cpp_names.contains(&"check_budget"));
}

#[test]
fn test_phase5_walker_and_ignore_rules() {
    let tmp = tempdir().expect("tempdir");
    let root = tmp.path();

    fs::create_dir_all(root.join("src/nested")).expect("create src/nested");
    fs::create_dir_all(root.join("target/release")).expect("create target");
    fs::create_dir_all(root.join("node_modules/dep")).expect("create node_modules");
    fs::create_dir_all(root.join(".git/objects")).expect("create git");
    fs::create_dir_all(root.join(".codegraph")).expect("create .codegraph");

    fs::write(root.join("src/nested/app.rs"), "pub fn run() {}").expect("write app.rs");
    fs::write(root.join("src/handler.go"), "package handler").expect("write handler.go");
    fs::write(root.join("target/release/binary.rs"), "pub fn ignored() {}").expect("write ignored");
    fs::write(root.join("node_modules/dep/index.js"), "module.exports = {}").expect("write npm");
    fs::write(root.join(".git/objects/obj.c"), "int obj;").expect("write git obj");

    let files = walk_project_files(root);
    let rel_paths: Vec<String> = files.iter().map(|f| f.rel_path.clone()).collect();

    assert!(rel_paths.iter().any(|p| p.ends_with("app.rs")));
    assert!(rel_paths.iter().any(|p| p.ends_with("handler.go")));
    assert!(!rel_paths.iter().any(|p| p.contains("target")));
    assert!(!rel_paths.iter().any(|p| p.contains("node_modules")));
    assert!(!rel_paths.iter().any(|p| p.contains(".git")));
    assert!(!rel_paths.iter().any(|p| p.contains(".codegraph")));

    // Language detection
    assert_eq!(detect_language(Path::new("file.rs")), Some("rust"));
    assert_eq!(detect_language(Path::new("file.ts")), Some("typescript"));
    assert_eq!(detect_language(Path::new("file.tsx")), Some("tsx"));
    assert_eq!(detect_language(Path::new("file.js")), Some("javascript"));
    assert_eq!(detect_language(Path::new("file.jsx")), Some("jsx"));
    assert_eq!(detect_language(Path::new("file.py")), Some("python"));
    assert_eq!(detect_language(Path::new("file.go")), Some("go"));
    assert_eq!(detect_language(Path::new("file.c")), Some("c"));
    assert_eq!(detect_language(Path::new("file.cpp")), Some("cpp"));
    assert_eq!(detect_language(Path::new("file.txt")), None);
}

#[test]
fn test_phase5_petgraph_blast_radius_and_call_path() {
    let mut engine = CodeGraphEngine::new();

    // Node A (Entrypoint)
    let node_a = SymbolNode {
        id: "func:order_service::checkout".into(),
        name: "checkout".into(),
        qualified_name: "OrderService::checkout".into(),
        kind: "method".into(),
        visibility: Some("public".into()),
        file_path: "src/order.rs".into(),
        start_line: 10,
        end_line: 25,
        start_column: 4,
        end_column: 5,
        docstring: Some("Processes checkout".into()),
        signature: Some("pub fn checkout(&self) -> bool".into()),
        return_type: Some("bool".into()),
    };

    // Node B (Payment processor)
    let node_b = SymbolNode {
        id: "func:payment::charge".into(),
        name: "charge".into(),
        qualified_name: "Payment::charge".into(),
        kind: "function".into(),
        visibility: Some("public".into()),
        file_path: "src/payment.rs".into(),
        start_line: 5,
        end_line: 15,
        start_column: 0,
        end_column: 1,
        docstring: Some("Charges payment".into()),
        signature: Some("pub fn charge(amt: f64) -> bool".into()),
        return_type: Some("bool".into()),
    };

    // Node C (Gateway call)
    let node_c = SymbolNode {
        id: "func:gateway::transmit".into(),
        name: "transmit".into(),
        qualified_name: "Gateway::transmit".into(),
        kind: "function".into(),
        visibility: Some("private".into()),
        file_path: "src/gateway.rs".into(),
        start_line: 1,
        end_line: 8,
        start_column: 0,
        end_column: 1,
        docstring: None,
        signature: Some("fn transmit() -> bool".into()),
        return_type: Some("bool".into()),
    };

    engine.add_node(node_a.clone());
    engine.add_node(node_b.clone());
    engine.add_node(node_c.clone());

    // checkout -> charge
    engine.add_edge(SymbolEdge {
        source_id: node_a.id.clone(),
        target_id: node_b.id.clone(),
        kind: "calls".into(),
        line: Some(15),
        column: Some(8),
    });

    // charge -> transmit
    engine.add_edge(SymbolEdge {
        source_id: node_b.id.clone(),
        target_id: node_c.id.clone(),
        kind: "calls".into(),
        line: Some(10),
        column: Some(4),
    });

    // 1. Callers of charge: checkout
    let callers = engine.get_callers(&node_b.id);
    assert_eq!(callers.len(), 1);
    assert_eq!(callers[0].0.name, "checkout");

    // 2. Callees of charge: transmit
    let callees = engine.get_callees(&node_b.id);
    assert_eq!(callees.len(), 1);
    assert_eq!(callees[0].0.name, "transmit");

    // 3. Blast radius for charge
    let blast = engine.compute_blast_radius(&node_b.id, 2).expect("blast radius");
    assert_eq!(blast.focal_name, "charge");
    assert!(blast.affected_files.contains(&"src/payment.rs".to_string()));
    assert!(blast.affected_files.contains(&"src/order.rs".to_string()));
    assert!(blast.affected_files.contains(&"src/gateway.rs".to_string()));
    assert!(blast.risk_score > 0.0);
}

#[test]
fn test_phase5_incremental_cache_sqlite() {
    let tmp = tempdir().expect("tempdir");
    let db_path = tmp.path().join(".codegraph").join("symbols.db");
    let mut cache = SymbolCache::open(&db_path).expect("open symbols.db");

    let walked = WalkedFile {
        path: tmp.path().join("src/lib.rs"),
        rel_path: "src/lib.rs".into(),
        language: "rust",
        mtime: 1000,
        size: 200,
    };

    let node = SymbolNode {
        id: "func:lib::init".into(),
        name: "init".into(),
        qualified_name: "lib::init".into(),
        kind: "function".into(),
        visibility: Some("pub".into()),
        file_path: "src/lib.rs".into(),
        start_line: 1,
        end_line: 5,
        start_column: 0,
        end_column: 1,
        docstring: None,
        signature: Some("pub fn init()".into()),
        return_type: None,
    };

    let ext = tokless::codegraph::kernel::FileExtraction {
        nodes: vec![node],
        edges: vec![],
        refs: vec![],
    };

    let hash = content_hash(b"pub fn init() {}");
    cache.save_extraction(&walked, &hash, &ext).expect("save extraction");

    let stats = cache.get_stats();
    assert_eq!(stats.file_count, 1);
    assert_eq!(stats.node_count, 1);

    // Verify incremental detection
    let cached = cache.get_cached_files();
    assert!(cached.contains_key("src/lib.rs"));
    assert_eq!(cached["src/lib.rs"].0, 1000);
    assert_eq!(cached["src/lib.rs"].1, 200);

    // Verify file removal
    cache.remove_file("src/lib.rs").expect("remove file");
    let stats2 = cache.get_stats();
    assert_eq!(stats2.file_count, 0);
    assert_eq!(stats2.node_count, 0);
}

#[test]
fn test_phase5_in_process_mcp_server() {
    let tmp = tempdir().expect("tempdir");
    let root = tmp.path();
    let root_str = root.to_string_lossy().to_string();

    fs::create_dir_all(root.join("src")).expect("create src");
    fs::write(
        root.join("src/service.rs"),
        r#"
pub struct OrderProcessor;

impl OrderProcessor {
    pub fn process_order(&self, id: u64) -> bool {
        validate(id)
    }
}

fn validate(id: u64) -> bool {
    id > 0
}
"#,
    )
    .expect("write service.rs");

    // 1. tools/list
    let list_req = json!({
        "jsonrpc": "2.0",
        "id": 100,
        "method": "tools/list",
        "params": {}
    });
    let list_resp = handle_mcp_message(&list_req).expect("list response");
    let tools = list_resp["result"]["tools"].as_array().expect("tools");
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    assert!(names.contains(&"codegraph_explore"));
    assert!(names.contains(&"codegraph_impact"));
    assert!(names.contains(&"codegraph_symbol"));
    assert!(names.contains(&"codegraph_callers"));
    assert!(names.contains(&"codegraph_callees"));
    assert!(names.contains(&"codegraph_search"));
    assert!(names.contains(&"codegraph_status"));
    assert!(names.contains(&"codegraph_files"));

    // 2. codegraph_explore
    let explore_req = json!({
        "jsonrpc": "2.0",
        "id": 101,
        "method": "tools/call",
        "params": {
            "name": "codegraph_explore",
            "arguments": {
                "query": "OrderProcessor",
                "projectPath": root_str
            }
        }
    });
    let explore_resp = handle_mcp_message(&explore_req).expect("explore resp");
    let text = explore_resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("OrderProcessor"));
    assert!(text.contains("process_order"));

    // 3. codegraph_impact
    let impact_req = json!({
        "jsonrpc": "2.0",
        "id": 102,
        "method": "tools/call",
        "params": {
            "name": "codegraph_impact",
            "arguments": {
                "symbol": "process_order",
                "depth": 2,
                "projectPath": root_str
            }
        }
    });
    let impact_resp = handle_mcp_message(&impact_req).expect("impact resp");
    let text = impact_resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Blast Radius"));
    assert!(text.contains("process_order"));

    // 4. codegraph_symbol
    let sym_req = json!({
        "jsonrpc": "2.0",
        "id": 103,
        "method": "tools/call",
        "params": {
            "name": "codegraph_symbol",
            "arguments": {
                "symbol": "validate",
                "projectPath": root_str
            }
        }
    });
    let sym_resp = handle_mcp_message(&sym_req).expect("symbol resp");
    let text = sym_resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Node: `validate`"));
}

#[test]
fn test_phase5_projectmem_location_resolution() {
    let tmp = tempdir().expect("tempdir");
    let root = tmp.path();

    fs::create_dir_all(root.join("src")).expect("create src");
    fs::write(
        root.join("src/crypto.rs"),
        r#"
pub fn hash_password(pwd: &str) -> String {
    format!("hashed:{}", pwd)
}
"#,
    )
    .expect("write crypto.rs");

    let mut cg = CodeGraph::open(root).expect("open codegraph");
    cg.ensure_indexed().expect("index codegraph");

    // 1. Direct coordinate resolution
    let sym = resolve_coordinate(root, "src/crypto.rs", 2);
    assert_eq!(sym, Some("hash_password".to_string()));

    // 2. Auto-resolve in projectmem location formatting
    let formatted = auto_resolve_location(root, "src/crypto.rs:2");
    assert_eq!(formatted, "src/crypto.rs:2 (hash_password)");

    // 3. ProjectMem event creation with auto-resolved coordinate
    initialize(Some(root), Some("Crypto Service")).expect("init mem");
    let mut evt = Event::issue("Weak hashing algorithm detected", Some("src/crypto.rs:2"));
    append_event(&mut evt, Some(root)).expect("append issue");

    let events = read_events(Some(root)).expect("read events");
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].location.as_deref(),
        Some("src/crypto.rs:2 (hash_password)")
    );
}

#[test]
fn test_phase5_cli_graph_commands() {
    let tmp = tempdir().expect("tempdir");
    let root = tmp.path();
    let root_str = root.to_string_lossy().to_string();

    fs::create_dir_all(root.join("src")).expect("create src");
    fs::write(
        root.join("src/pipeline.rs"),
        r#"
pub fn run_pipeline() -> bool {
    step_one() && step_two()
}

fn step_one() -> bool { true }
fn step_two() -> bool { true }
"#,
    )
    .expect("write pipeline.rs");

    // index
    assert_eq!(
        run_graph_cli(&["index".into(), "--root".into(), root_str.clone(), "--force".into()]),
        0
    );

    // status
    assert_eq!(
        run_graph_cli(&["status".into(), "--root".into(), root_str.clone()]),
        0
    );

    // files
    assert_eq!(
        run_graph_cli(&["files".into(), "--root".into(), root_str.clone()]),
        0
    );

    // symbols
    assert_eq!(
        run_graph_cli(&["symbols".into(), "pipeline".into(), "--root".into(), root_str.clone()]),
        0
    );

    // explore
    assert_eq!(
        run_graph_cli(&["explore".into(), "pipeline".into(), "--root".into(), root_str.clone()]),
        0
    );

    // impact
    assert_eq!(
        run_graph_cli(&["impact".into(), "run_pipeline".into(), "--root".into(), root_str.clone()]),
        0
    );

    // callers
    assert_eq!(
        run_graph_cli(&["callers".into(), "step_one".into(), "--root".into(), root_str.clone()]),
        0
    );

    // callees
    assert_eq!(
        run_graph_cli(&["callees".into(), "run_pipeline".into(), "--root".into(), root_str]),
        0
    );
}

