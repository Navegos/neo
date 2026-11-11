# Phase 05: CodeGraph Pure Rust Engine

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_05_CODEGRAPH_PURE_RUST.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Always treat Source: neo/inspect-vault/codegraph as newly added/updated code that we must inspect, convert, and import into consolidated toksi, checking for missing features, addons, updates, and bugfixes.

Execute Phase 05: CodeGraph Pure Rust Engine.
Source: neo/inspect-vault/codegraph (codegraph-kernel in Rust + TS wrapper)
Target: crates/toksi/src/codegraph/

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly in `crates/toksi`. Wrap any legacy C-ABI Tree-sitter grammar shims in isolated leaf crate `crates/toksi-grammars` returning safe `tree_sitter_language::LanguageFn`. Zero compiler warnings (`RUSTFLAGS="-D warnings"`).
2. Zero Panics Policy: Strictly prohibit `.unwrap()`, `.expect()`, `panic!()`. Use `thiserror` for domain errors and `anyhow::Result` with `.context()` in CLI.
3. Universal Forward-Slash (`/`) Standard: Use `normalize_path()` for all AST nodes, file coordinates, and symbols.
4. Strict `<repo>/.toksi/codegraph/` Repo-Only & Monorepo Partitioning: Store workspace `symbols.db` and graph cache ONLY inside workspace root `<repo>/.toksi/codegraph/` (NEVER in `~/.toksi/codegraph/`, NEVER outside a `.git` repository root, NEVER inside `<repo>/.git/` or subfolder crates). Fix `commands/index.rs::find_project_dir()` so it walks to `.git` root rather than stopping at subcrate `Cargo.toml`/`package.json`, partition `<repo>/crates/project1..N/` by `subproject_id`, and merge nested `.toksi/codegraph` dirs into root.
5. Dynamic Ignore Purging: Integrate ripgrep-grade ignore walker respecting `.gitignore` and `.ignore`; purge matching indexed symbols dynamically when ignore rules change.
6. Standardized Naming: Tool identifier is `codegraph` (purge `toksi-codegraph-index`, `agy-hook codegraph-index`, and `toksi-codegraph-init.js`). Hook syntax is `codegraph-hook <agent>`. Slash command is `/codegraph`.

Requirements:
1. Eliminate Node.js and TypeScript wrapper completely.
2. In-Tree Tree-sitter parsers via `tree-sitter = "0.25"` + `crates/toksi-grammars` supporting **all 41 specified polyglot languages** (plus Dart & R bonus) and porting the 8+ unported `codegraph-kernel` extractors (`csharp`, `dart`, `java`, `kotlin`, `lua`, `php`, `rlang`, `ruby`, `scala`, `swift`, `cfnptr`) + Vue/Svelte extractors:
   Rust, TypeScript, JavaScript, Python, Go, C, C++, Zig, Swift, Java, C#, Kotlin, Scala, PHP, Ruby, Lua, Julia, Elixir, Erlang, OCaml, Haskell, HTML, CSS, SCSS, TSX, Vue, Svelte, GraphQL, Bash, Dockerfile, HCL, Nix, Make, CMake, JSON, YAML, TOML, Protocol Buffers, SQL, Markdown, LaTeX (+ Dart, R).
3. High-speed file traversal using `ignore` crate (respecting `.gitignore`/`.ignore`/`.git/` boundary).
4. Petgraph AST dependency, call-path, and blast-radius engine + real wiring of `ast_grep.rs` (Tree-sitter CST + `MatchStrictness`), `scip.rs` (Protobuf + descriptor AST), `jcodemunch.rs` (O(1) SQLite byte-offset slices), and `repomix.rs` (secretlint + BPE).
5. Incremental AST symbol cache in `<repo>/.toksi/codegraph/symbols.db`.
6. Serve `codegraph_explore`, `codegraph_impact`, and `codegraph_symbol` via in-process multi-protocol server (`MCP`, `A2A` v1.0, `ACP` v2.0, `stdio`, `SSE`, `HTTP`, `DirectWireBus`).
7. Connect to `crates/toksi/src/projectmem/` to auto-resolve `location: "file:line"` coordinates to AST symbol names.
8. Expose the native CLI command `toksi graph ...`.
9. Ensure 0 compiler warnings and targeted `cargo test` pass.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Tree-sitter AST Kernel** (`src/codegraph/kernel/`): Native parser integrations for 41+ polyglot languages (via `crates/toksi-grammars`) extracting symbols, functions, classes, imports, and calls, partitioned cleanly by monorepo `subproject_id`.
- **Dependency & Blast Radius Graph** (`src/codegraph/graph.rs`): Petgraph directed graph computing upstream callers, downstream dependencies, and transitive impact sets.
- **Dynamic Ignore Walker & Purge** (`src/codegraph/walker.rs`): High-speed crawler ignoring build artifacts (`target/`, `node_modules/`, `dist/`). Purges indexed symbols when `.gitignore` changes.
- **Multi-Protocol Dispatch** (`src/codegraph/mcp.rs`): Handles `codegraph_explore`, `codegraph_impact`, and `codegraph_symbol` over MCP, A2A, ACP, stdio, and HTTP.

### 2. Disciplinary Standards & Invariants
- Symbol file paths must always be normalized to forward slashes `/`.
- Tree-sitter node queries must gracefully handle syntax errors in modified working trees without panicking.

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- codegraph` passes 100%.
- `toksi graph explore "main"` computes callers and blast radius in <50ms.
