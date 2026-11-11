# Phase 17.3: 41+ Tree-sitter Languages, `<repo>/.toksi/<tool>` Repo-Only Storage, Monorepo Partitioning, & `.ignore` Purge

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 13.2–13.3), .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_17_3_TREESITTER_41_LANGS_AND_REPO_ISOLATION.md first.

Execute Phase 17.3: 41+ Tree-sitter Languages, `<repo>/.toksi/<tool>` Repo-Only Storage, Monorepo Partitioning, & `.ignore` Purge.
Sources:
  - neo/inspect-vault/codegraph (codegraph-kernel/src/*.rs & src/extraction/*.ts)
  - neo/inspect-vault/ast-grep
  - neo/inspect-vault/scip
  - neo/inspect-vault/jcodemunch-mcp
  - neo/inspect-vault/repomix
Targets:
  - Cargo.toml (workspace member `crates/toksi-grammars`)
  - crates/toksi-grammars/{Cargo.toml, src/lib.rs}
  - crates/toksi/Cargo.toml
  - crates/toksi/src/codegraph/{mod.rs, walker.rs, cache.rs, kernel/*, ast_grep.rs, scip.rs, jcodemunch.rs, repomix.rs}
  - crates/toksi/src/commands/{index.rs, init.rs, doctor.rs}
  - crates/toksi/src/{projectmem/{global,store}.rs, icm/mcp.rs, aimemory/mod.rs, contextmode/fts.rs}

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly in `crates/toksi`. Isolate any older C-ABI `extern "C" fn tree_sitter_*` grammar shims inside `crates/toksi-grammars` returning safe `tree_sitter_language::LanguageFn`.
2. Zero Panics & Forward-Slash (`/`) Normalization: Handle syntax errors in dirty working trees without panicking; normalize every path with `normalize_path()`.
3. Strict `<repo>/.toksi/<tool>` Repo-Only Storage: Workspace indexes (`codegraph`, `projectmem`, `context`, `icm`, `ai-memory`) MUST live ONLY in root `<repo>/.toksi/<tool>/`. NEVER create project stores in `~/.toksi/<tool>/`, NEVER index non-`.git` directories, NEVER index `<repo>/.git/`, and NEVER create `.toksi/` inside nested `<repo>/crates/*` folders.
4. Dynamic `.gitignore` / `.ignore` Vector Purge: Watch/diff ignore rules and delete newly ignored paths from `symbols.db`, FTS5 (`context`, `icm`), and Chroma vector caches.

Requirements:
1. 41+ Polyglot Tree-sitter Languages & Extractors:
   - Upgrade `Cargo.toml` + `crates/toksi-grammars` + `codegraph/kernel/langs.rs` + `walker.rs` from 8 existing languages (`Rust`, `TypeScript`/`TSX`, `JavaScript`/`JSX`, `Python`, `Go`, `C`, `C++`) to all 41 specified languages:
     `zig`, `swift`, `java`, `C#` (`csharp`), `kotlin`, `scala`, `php`, `ruby`, `lua`, `julia`, `elixir`, `erlang`, `ocaml`, `haskell`, `html`, `css`, `scss`, `vue`, `svelte`, `graphql`, `bash`, `dockerfile`, `hcl`, `nix`, `make`, `cmake`, `json`, `yaml`, `toml`, `Protocol Buffers` (`proto`), `sql`, `markdown`, `latex` (+ bonus `dart` and `r`).
   - Port the unported extractors already present in `inspect-vault/codegraph/codegraph-kernel/src/` (`csharp`, `dart`, `java`, `kotlin`, `lua`, `php`, `rlang`, `ruby`, `scala`, `swift`, `cfnptr`) plus `vue` / `svelte` `<script>`/`<style>` embedded extractors.
2. Eliminate Stubbed Intelligence Modules & Wire Real Implementations:
   - `ast_grep.rs`: Replace line-regex heuristic with real Tree-sitter CST metavariable matcher (`$VAR`, `$$$`) + `MatchStrictness` (`Cst|Smart|Ast|Relaxed|Signature`).
   - `scip.rs`: Replace stub with real SCIP Protobuf wire encoding + descriptor AST extraction exported from `symbols.db`.
   - `jcodemunch.rs`: Wire O(1) SQLite `byte_offset..byte_len` source slice extraction directly into `codegraph/mod.rs`, CLI, and MCP.
   - `repomix.rs`: Wire secretlint redaction + BPE token counts + `.gitignore`/`.repomixignore` directly into `codegraph/mod.rs` + A2A pack export.
3. Monorepo Root Discovery & Home Storage Cleanup:
   - Fix `commands/index.rs::find_project_dir()` so it NEVER stops at intermediate `Cargo.toml`/`package.json` files inside `<repo>/crates/*`; walk strictly to `.git` root (refusing cleanly if outside any git repo).
   - Add `subproject_id` partitioning across `<repo>/crates/project1..N/` in `codegraph`, `projectmem`, `context`, `icm`, `aimemory`, and `rtk`.
   - Auto-merge any legacy nested `<repo>/crates/*/.toksi/` or home `~/.toksi/{projectmem,icm,ai-memory,context,codegraph}` artifacts into root `<repo>/.toksi/<tool>/` cleanly.
4. Verification:
   - Run `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted `cargo test -p toksi`.
```

---

## Technical Specifications & Details

### 1. Crates.io Compatibility Reference (`tree-sitter = "0.25"`)
- Direct `LanguageFn` crates: `tree-sitter-zig 1.1.2`, `tree-sitter-swift 0.7.4`, `tree-sitter-java 0.23.5`, `tree-sitter-c-sharp 0.23.5`, `tree-sitter-kotlin-sg 0.4.1`, `tree-sitter-scala 0.26.10`, `tree-sitter-php 0.24.2`, `tree-sitter-ruby 0.23.1`, `tree-sitter-lua 0.5.0`, `tree-sitter-julia 0.23.1`, `tree-sitter-elixir 0.3.5`, `tree-sitter-erlang 0.21.0`, `tree-sitter-ocaml 0.24.x`, `tree-sitter-haskell 0.24.1`, `tree-sitter-html 0.23.2`, `tree-sitter-css 0.25.0`, `tree-sitter-svelte-next 0.1.1`, `tree-sitter-graphql 0.3.0`, `tree-sitter-bash 0.25.1`, `tree-sitter-hcl 1.1.0`, `tree-sitter-nix 0.3.0`, `tree-sitter-make 1.1.1`, `tree-sitter-cmake 0.7.6`, `tree-sitter-json 0.24.8`, `tree-sitter-yaml 0.7.2`, `tree-sitter-toml-ng 0.7.0`, `tree-sitter-proto 0.6.0`, `tree-sitter-sequel 0.3.11`, `tree-sitter-md 0.5.4`, `codebook-tree-sitter-latex 0.6.1`, plus `tree-sitter-dart 0.3.0` and `tree-sitter-r 1.2.0`.
- Leaf-wrapped in `crates/toksi-grammars` if older ABI (`scss`, `dockerfile`, `vue` as needed) so `crates/toksi` preserves `#![deny(unsafe_code)]`.

### 2. Verification Criteria
- Indexing a fixture workspace with files in all 41+ languages extracts real symbols under `#![deny(unsafe_code)]`.
- Running `toksi graph index` from `<repo>/crates/subpkg` writes ONLY to `<repo>/.toksi/codegraph/symbols.db` with `subproject_id = "subpkg"`, and never creates files in `~/.toksi/codegraph/` or outside `<repo>`.
