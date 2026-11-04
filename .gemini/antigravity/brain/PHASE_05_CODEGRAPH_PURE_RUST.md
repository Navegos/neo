# Phase 05: CodeGraph Pure Rust Engine

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, .projectmem/plan.md, and .gemini/antigravity/brain/PHASE_05_CODEGRAPH_PURE_RUST.md first to understand the global master architecture and cross-tool coupling.

Execute Phase 05: CodeGraph Pure Rust Engine.
Source: neo/inspect-vault/codegraph (codegraph-kernel in Rust + TS wrapper)
Target: crates/toksi/src/codegraph/

Requirements:
1. Eliminate Node.js and TypeScript wrapper completely.
2. Integrate embedded Tree-sitter parsers directly via Rust dependencies (Rust, TS/JS, Python, Go, C/C++).
3. Implement high-speed file traversal using the `ignore` crate (respecting .gitignore and hidden files).
4. Build AST dependency, call-path, and blast-radius engine using `petgraph`.
5. Implement incremental AST symbol cache in .codegraph/symbols.db.
6. Serve `codegraph_explore`, `codegraph_impact`, and `codegraph_symbol` via in-process MCP server.
7. Connect to crates/toksi/src/projectmem/ to auto-resolve `location: "file:line"` coordinates to AST symbol names.
8. Wire into `toksi graph <explore|impact|symbols|index>` CLI.
9. Ensure 100% functional parity, zero missing code, 0 compiler warnings, and passing unit tests.
```

