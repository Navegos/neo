// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    use crate::codegraph::cache::{content_hash, SymbolCache};
    use crate::codegraph::graph::CodeGraphEngine;
    use crate::codegraph::kernel::{extract_file, SymbolEdge, SymbolNode, SymbolRef};
    use crate::codegraph::walker::{detect_language, walk_project_files, WalkedFile};
    use crate::codegraph::CodeGraph;

    #[test]
    fn test_extract_rust() {
        let code = r#"
/// Service for processing orders
pub struct OrderService {
    pub id: u64,
}

impl OrderService {
    pub fn new(id: u64) -> Self {
        OrderService { id }
    }

    pub fn process(&self) -> bool {
        validate_order(self.id)
    }
}

fn validate_order(id: u64) -> bool {
    id > 0
}
"#;
        let extraction = extract_file("src/order.rs", code, "rust").expect("extract rust");
        assert!(!extraction.nodes.is_empty());

        let names: Vec<&str> = extraction.nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"OrderService"), "should extract struct OrderService");
        assert!(names.contains(&"new"), "should extract method new");
        assert!(names.contains(&"process"), "should extract method process");
        assert!(names.contains(&"validate_order"), "should extract fn validate_order");

        let order_struct = extraction.nodes.iter().find(|n| n.name == "OrderService").unwrap();
        assert_eq!(order_struct.kind, "struct");
        assert!(order_struct.docstring.as_ref().unwrap().contains("Service for processing orders"));
    }

    #[test]
    fn test_extract_typescript() {
        let code = r#"
export interface User {
    id: string;
    name: string;
}

export class UserManager {
    private users: User[] = [];

    public async addUser(user: User): Promise<void> {
        this.validateUser(user);
        this.users.push(user);
    }

    private validateUser(user: User): boolean {
        return user.name.length > 0;
    }
}
"#;
        let extraction = extract_file("src/user.ts", code, "typescript").expect("extract ts");
        assert!(!extraction.nodes.is_empty());

        let names: Vec<&str> = extraction.nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"User"), "should extract interface User");
        assert!(names.contains(&"UserManager"), "should extract class UserManager");
        assert!(names.contains(&"addUser"), "should extract method addUser");
        assert!(names.contains(&"validateUser"), "should extract method validateUser");
    }

    #[test]
    fn test_extract_python() {
        let code = r#"
class PaymentProcessor:
    """Handles credit card payments."""
    def __init__(self, api_key: str):
        self.api_key = api_key

    def charge(self, amount: float) -> bool:
        return self._send_request(amount)

    def _send_request(self, amount: float) -> bool:
        return amount > 0
"#;
        let extraction = extract_file("payment.py", code, "python").expect("extract python");
        assert!(!extraction.nodes.is_empty());

        let names: Vec<&str> = extraction.nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"PaymentProcessor"));
        assert!(names.contains(&"charge"));
        assert!(names.contains(&"_send_request"));

        let cls = extraction.nodes.iter().find(|n| n.name == "PaymentProcessor").unwrap();
        assert_eq!(cls.kind, "class");
        assert!(cls.docstring.as_ref().unwrap().contains("Handles credit card payments"));
    }

    #[test]
    fn test_extract_go() {
        let code = r#"
package auth

type Session struct {
    Token string
}

func NewSession(token string) *Session {
    return &Session{Token: token}
}

func (s *Session) IsValid() bool {
    return checkToken(s.Token)
}

func checkToken(token string) bool {
    return len(token) > 0
}
"#;
        let extraction = extract_file("auth.go", code, "go").expect("extract go");
        assert!(!extraction.nodes.is_empty());

        let names: Vec<&str> = extraction.nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Session"));
        assert!(names.contains(&"NewSession"));
        assert!(names.contains(&"IsValid"));
        assert!(names.contains(&"checkToken"));
    }

    #[test]
    fn test_extract_cpp() {
        let code = r#"
#include <iostream>

class Engine {
public:
    Engine() {}
    void start() {
        ignite();
    }
private:
    void ignite() {}
};

int main() {
    Engine e;
    e.start();
    return 0;
}
"#;
        let extraction = extract_file("engine.cpp", code, "cpp").expect("extract cpp");
        assert!(!extraction.nodes.is_empty());

        let names: Vec<&str> = extraction.nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Engine"));
        assert!(names.contains(&"start"));
        assert!(names.contains(&"ignite"));
        assert!(names.contains(&"main"));
    }

    #[test]
    fn test_walker_language_detection() {
        assert_eq!(detect_language(Path::new("src/lib.rs")), Some("rust"));
        assert_eq!(detect_language(Path::new("app/index.ts")), Some("typescript"));
        assert_eq!(detect_language(Path::new("comp.tsx")), Some("tsx"));
        assert_eq!(detect_language(Path::new("script.js")), Some("javascript"));
        assert_eq!(detect_language(Path::new("ui.jsx")), Some("jsx"));
        assert_eq!(detect_language(Path::new("main.py")), Some("python"));
        assert_eq!(detect_language(Path::new("server.go")), Some("go"));
        assert_eq!(detect_language(Path::new("kernel.c")), Some("c"));
        assert_eq!(detect_language(Path::new("header.h")), Some("c"));
        assert_eq!(detect_language(Path::new("driver.cpp")), Some("cpp"));
        assert_eq!(detect_language(Path::new("readme.md")), None);
        assert_eq!(detect_language(Path::new("config.json")), None);
    }

    #[test]
    fn test_walker_walk_project_files() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("node_modules/fake")).unwrap();
        fs::create_dir_all(root.join("target/debug")).unwrap();

        fs::write(root.join("src/lib.rs"), "pub fn add(a: i32) -> i32 { a + 1 }").unwrap();
        fs::write(root.join("src/app.py"), "def run(): pass").unwrap();
        fs::write(root.join("node_modules/fake/index.js"), "module.exports = {}").unwrap();
        fs::write(root.join("target/debug/test.rs"), "ignored").unwrap();

        let files = walk_project_files(root);
        let paths: Vec<String> = files.into_iter().map(|f| f.rel_path).collect();

        assert!(paths.contains(&"src/lib.rs".to_string()));
        assert!(paths.contains(&"src/app.py".to_string()));
        assert!(!paths.iter().any(|p| p.contains("node_modules")));
        assert!(!paths.iter().any(|p| p.contains("target")));
    }

    #[test]
    fn test_symbol_cache_lifecycle() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join(".codegraph").join("symbols.db");
        let mut cache = SymbolCache::open(&db_path).expect("open cache");

        let file = WalkedFile {
            path: dir.path().join("main.rs"),
            rel_path: "main.rs".to_string(),
            language: "rust",
            mtime: 12345,
            size: 678,
        };

        let node = SymbolNode {
            id: "function:main_id".to_string(),
            name: "main".to_string(),
            qualified_name: "main".to_string(),
            kind: "function".to_string(),
            visibility: Some("public".to_string()),
            file_path: "main.rs".to_string(),
            start_line: 1,
            end_line: 5,
            start_column: 0,
            end_column: 1,
            docstring: Some("Main entrypoint".to_string()),
            signature: Some("fn main()".to_string()),
            return_type: None,
        };

        let extraction = crate::codegraph::kernel::FileExtraction {
            nodes: vec![node.clone()],
            edges: vec![],
            refs: vec![],
        };

        let hash = content_hash(b"fn main() {}");
        cache.save_extraction(&file, &hash, &extraction).expect("save extraction");

        let stats = cache.get_stats();
        assert_eq!(stats.file_count, 1);
        assert_eq!(stats.node_count, 1);

        let cached_files = cache.get_cached_files();
        assert!(cached_files.contains_key("main.rs"));

        let nodes = cache.get_all_nodes();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].name, "main");

        cache.remove_file("main.rs").expect("remove file");
        let stats_after = cache.get_stats();
        assert_eq!(stats_after.file_count, 0);
        assert_eq!(stats_after.node_count, 0);
    }

    #[test]
    fn test_graph_engine_and_references() {
        let mut engine = CodeGraphEngine::new();

        let node_a = SymbolNode {
            id: "fn:caller".into(),
            name: "caller_fn".into(),
            qualified_name: "caller_fn".into(),
            kind: "function".into(),
            visibility: Some("public".into()),
            file_path: "a.rs".into(),
            start_line: 1,
            end_line: 10,
            start_column: 0,
            end_column: 1,
            docstring: None,
            signature: None,
            return_type: None,
        };

        let node_b = SymbolNode {
            id: "fn:callee".into(),
            name: "target_worker".into(),
            qualified_name: "target_worker".into(),
            kind: "function".into(),
            visibility: Some("public".into()),
            file_path: "b.rs".into(),
            start_line: 1,
            end_line: 15,
            start_column: 0,
            end_column: 1,
            docstring: None,
            signature: None,
            return_type: None,
        };

        engine.add_node(node_a);
        engine.add_node(node_b);

        let refs = vec![SymbolRef {
            from_node_id: "fn:caller".into(),
            name: "target_worker".into(),
            kind: "calls".into(),
            line: 5,
            column: 8,
            file_path: "a.rs".into(),
        }];

        let resolved_edges = engine.resolve_references(&refs);
        assert_eq!(resolved_edges.len(), 1);
        assert_eq!(resolved_edges[0].source_id, "fn:caller");
        assert_eq!(resolved_edges[0].target_id, "fn:callee");

        engine.add_edge(resolved_edges[0].clone());

        let callers = engine.get_callers("fn:callee");
        assert_eq!(callers.len(), 1);
        assert_eq!(callers[0].0.name, "caller_fn");

        let callees = engine.get_callees("fn:caller");
        assert_eq!(callees.len(), 1);
        assert_eq!(callees[0].0.name, "target_worker");
    }

    #[test]
    fn test_blast_radius_computation() {
        let mut engine = CodeGraphEngine::new();

        let node_a = SymbolNode {
            id: "node:a".into(),
            name: "entry_handler".into(),
            qualified_name: "entry_handler".into(),
            kind: "function".into(),
            visibility: None,
            file_path: "api.rs".into(),
            start_line: 1,
            end_line: 10,
            start_column: 0,
            end_column: 0,
            docstring: None,
            signature: None,
            return_type: None,
        };

        let node_b = SymbolNode {
            id: "node:b".into(),
            name: "service_core".into(),
            qualified_name: "service_core".into(),
            kind: "function".into(),
            visibility: None,
            file_path: "service.rs".into(),
            start_line: 1,
            end_line: 20,
            start_column: 0,
            end_column: 0,
            docstring: None,
            signature: None,
            return_type: None,
        };

        let node_c = SymbolNode {
            id: "node:c".into(),
            name: "db_query".into(),
            qualified_name: "db_query".into(),
            kind: "function".into(),
            visibility: None,
            file_path: "db.rs".into(),
            start_line: 1,
            end_line: 15,
            start_column: 0,
            end_column: 0,
            docstring: None,
            signature: None,
            return_type: None,
        };

        engine.add_node(node_a);
        engine.add_node(node_b);
        engine.add_node(node_c);

        // a calls b; b calls c
        engine.add_edge(SymbolEdge {
            source_id: "node:a".into(),
            target_id: "node:b".into(),
            kind: "calls".into(),
            line: None,
            column: None,
        });
        engine.add_edge(SymbolEdge {
            source_id: "node:b".into(),
            target_id: "node:c".into(),
            kind: "calls".into(),
            line: None,
            column: None,
        });

        // Focal node = b
        let report = engine.compute_blast_radius("node:b", 2).expect("compute blast radius");
        assert_eq!(report.focal_name, "service_core");

        // Upstream callers should contain entry_handler
        assert!(report.upstream_dependents.iter().any(|n| n.node.name == "entry_handler"));
        // Downstream dependencies should contain db_query
        assert!(report.downstream_dependencies.iter().any(|n| n.node.name == "db_query"));

        assert!(report.affected_files.contains(&"api.rs".to_string()));
        assert!(report.affected_files.contains(&"service.rs".to_string()));
        assert!(report.affected_files.contains(&"db.rs".to_string()));
        assert!(report.risk_score > 0.0);
    }

    #[test]
    fn test_coordinate_resolution() {
        let mut engine = CodeGraphEngine::new();

        let node = SymbolNode {
            id: "fn:process".into(),
            name: "process".into(),
            qualified_name: "OrderService::process".into(),
            kind: "method".into(),
            visibility: Some("public".into()),
            file_path: "src/order.rs".into(),
            start_line: 10,
            end_line: 25,
            start_column: 4,
            end_column: 5,
            docstring: None,
            signature: None,
            return_type: None,
        };

        engine.add_node(node);

        let resolved = engine.resolve_coordinate("src/order.rs", 15);
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().qualified_name, "OrderService::process");

        let out_of_range = engine.resolve_coordinate("src/order.rs", 50);
        assert!(out_of_range.is_none());
    }

    #[test]
    fn test_codegraph_full_project_index() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/math.rs"),
            "pub fn calculate(x: i32) -> i32 { x * 2 }\npub fn square(x: i32) -> i32 { calculate(x) * x }\n",
        )
        .unwrap();

        let mut cg = CodeGraph::open(root).expect("open codegraph");
        let indexed = cg.ensure_indexed().expect("ensure indexed");
        assert!(indexed);

        let stats = cg.cache.get_stats();
        assert_eq!(stats.file_count, 1);
        assert!(stats.node_count >= 2);

        let found = cg.engine.search_symbols("calculate", None, 5);
        assert!(!found.is_empty());
        assert_eq!(found[0].name, "calculate");

        // Second call without modifications should be cache hit
        let indexed_again = cg.ensure_indexed().expect("second index");
        assert!(!indexed_again, "second pass should be a cache hit");
    }

    #[test]
    fn test_codegraph_mcp_explore_impact_symbol() {
        use crate::codegraph::mcp::handle_mcp_message;
        use serde_json::json;

        let dir = tempdir().unwrap();
        let root = dir.path();
        let root_str = root.to_string_lossy().to_string();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/service.rs"),
            r#"
pub struct PaymentGateway {
    pub key: String,
}

impl PaymentGateway {
    pub fn process_charge(&self, amount: u64) -> bool {
        self.validate(amount)
    }

    fn validate(&self, amount: u64) -> bool {
        amount > 0
    }
}
"#,
        )
        .unwrap();

        // 1. Check tools/list has codegraph_explore, codegraph_impact, codegraph_symbol
        let list_req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        });
        let list_resp = handle_mcp_message(&list_req).expect("tools/list response");
        let tools = list_resp["result"]["tools"].as_array().expect("tools array");
        let tool_names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();

        assert!(tool_names.contains(&"codegraph_explore"));
        assert!(tool_names.contains(&"codegraph_impact"));
        assert!(tool_names.contains(&"codegraph_symbol"));

        // 2. Call codegraph_explore
        let explore_req = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "codegraph_explore",
                "arguments": {
                    "query": "PaymentGateway",
                    "projectPath": root_str
                }
            }
        });
        let explore_resp = handle_mcp_message(&explore_req).expect("explore response");
        let explore_text = explore_resp["result"]["content"][0]["text"].as_str().unwrap();
        assert!(explore_text.contains("PaymentGateway"));
        assert!(explore_text.contains("process_charge"));

        // 3. Call codegraph_impact
        let impact_req = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "codegraph_impact",
                "arguments": {
                    "symbol": "PaymentGateway",
                    "depth": 2,
                    "projectPath": root_str
                }
            }
        });
        let impact_resp = handle_mcp_message(&impact_req).expect("impact response");
        let impact_text = impact_resp["result"]["content"][0]["text"].as_str().unwrap();
        assert!(impact_text.contains("Blast Radius"));
        assert!(impact_text.contains("PaymentGateway"));

        // 4. Call codegraph_symbol
        let symbol_req = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "codegraph_symbol",
                "arguments": {
                    "symbol": "process_charge",
                    "projectPath": root_str
                }
            }
        });
        let symbol_resp = handle_mcp_message(&symbol_req).expect("symbol response");
        let symbol_text = symbol_resp["result"]["content"][0]["text"].as_str().unwrap();
        assert!(symbol_text.contains("Node: `process_charge`"));
        assert!(symbol_text.contains("PaymentGateway::process_charge"));
    }

    #[test]
    fn test_codegraph_cli_subcommands() {
        use crate::codegraph::run_graph_cli;

        let dir = tempdir().unwrap();
        let root = dir.path();
        let root_str = root.to_string_lossy().to_string();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn engine_init() -> bool { true }\npub fn engine_start() -> bool { engine_init() }\n",
        )
        .unwrap();

        // 1. index
        let code = run_graph_cli(&[
            "index".into(),
            "--root".into(),
            root_str.clone(),
            "--force".into(),
        ]);
        assert_eq!(code, 0);

        // 2. status
        let code = run_graph_cli(&[
            "status".into(),
            "--root".into(),
            root_str.clone(),
        ]);
        assert_eq!(code, 0);

        // 3. symbols (search / list)
        let code = run_graph_cli(&[
            "symbols".into(),
            "engine".into(),
            "--root".into(),
            root_str.clone(),
        ]);
        assert_eq!(code, 0);

        // 4. symbol (node inspect)
        let code = run_graph_cli(&[
            "symbol".into(),
            "engine_init".into(),
            "--root".into(),
            root_str.clone(),
        ]);
        assert_eq!(code, 0);

        // 5. explore
        let code = run_graph_cli(&[
            "explore".into(),
            "engine".into(),
            "--root".into(),
            root_str.clone(),
        ]);
        assert_eq!(code, 0);

        // 6. impact
        let code = run_graph_cli(&[
            "impact".into(),
            "engine_init".into(),
            "--root".into(),
            root_str,
        ]);
        assert_eq!(code, 0);
    }

    #[test]
    fn test_projectmem_auto_resolve_location_with_codegraph() {
        use crate::codegraph::resolve_coordinate;
        use crate::projectmem::{append_event, auto_resolve_location, initialize, read_events, Event};

        let dir = tempdir().unwrap();
        let root = dir.path();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/auth.rs"),
            "pub fn verify_token(token: &str) -> bool {\n    !token.is_empty()\n}\n",
        )
        .unwrap();

        // Initialize and index CodeGraph
        let mut cg = CodeGraph::open(root).expect("open codegraph");
        cg.ensure_indexed().expect("index codegraph");

        // Test direct resolve_coordinate
        let resolved = resolve_coordinate(root, "src/auth.rs", 1);
        assert_eq!(resolved, Some("verify_token".to_string()));

        // Test auto_resolve_location helper
        let enriched = auto_resolve_location(root, "src/auth.rs:1");
        assert_eq!(enriched, "src/auth.rs:1 (verify_token)");

        // Initialize ProjectMem in same project
        let _ = initialize(Some(root), Some("Integration Test Project")).unwrap();

        // Create and append an event with raw location coordinate "src/auth.rs:1"
        let mut evt = Event::issue("Token validation vulnerability", Some("src/auth.rs:1"));
        append_event(&mut evt, Some(root)).expect("append event");

        // Verify stored event has auto-resolved AST symbol name
        let events = read_events(Some(root)).expect("read events");
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].location.as_deref(),
            Some("src/auth.rs:1 (verify_token)")
        );
    }
}

