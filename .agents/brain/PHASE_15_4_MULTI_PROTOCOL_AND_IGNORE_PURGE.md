# Phase 15.4: Multi-Protocol Architecture & Dynamic Ignore Purging

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_15_4_MULTI_PROTOCOL_AND_IGNORE_PURGE.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 15.4: Multi-Protocol Architecture & Dynamic Ignore Purging.
Targets:
  - crates/toksi/src/codegraph/walker.rs
  - crates/toksi/src/codegraph/cache.rs
  - crates/toksi/src/contextmode/fts.rs
  - crates/toksi/src/icm/store.rs
  - crates/toksi/src/aimemory/wiki.rs
  - crates/toksi/src/projectmem/precheck.rs
  - crates/toksi/src/tools/multi_protocol.rs

Requirements:
1. Multi-Protocol Capabilities Across All Tools:
   - Expose unified multi-protocol interfaces for all tools (`rtk`, `codegraph`, `context`, `projectmem`, `icm`, `aimemory`):
     * MCP (Model Context Protocol)
     * A2A (Agent-to-Agent / Agent2Agent protocol)
     * ACP (Agent Communication Protocol / Agent Client Protocol)
     * stdio (Standard Input / Standard Output stream)
     * SSE (Server-Sent Events)
     * HTTP (RESTful & chunked streaming)
2. .gitignore & .ignore Optimization:
   - Integrate ripgrep-grade ignore walker to respect `.gitignore` and `.ignore` files.
   - Exclude build outputs (`target/`, `node_modules/`, `dist/`, `*.lock`), vendor bundles, and massive asset folders from indexing.
3. Dynamic Vector & Cache Purging:
   - When `.gitignore` or `.ignore` patterns are added or modified, automatically detect newly ignored files and purge their existing records from SQLite FTS5 BM25 search indexes and vector embedding caches.
   - Prevents memory databases from hoarding dead or excluded file representations.
4. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. Dynamic Ignore Purge in `walker.rs` & `cache.rs`
```rust
pub fn purge_ignored_entries(db: &rusqlite::Connection, root: &std::path::Path) -> rusqlite::Result<usize> {
    use ignore::WalkBuilder;
    let mut allowed_files = std::collections::HashSet::new();
    let walker = WalkBuilder::new(root)
        .standard_filters(true) // respects .gitignore, .ignore
        .hidden(true)
        .build();

    for result in walker {
        if let Ok(entry) = result {
            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                if let Ok(canon) = entry.path().canonicalize() {
                    allowed_files.insert(normalize_path(&canon.to_string_lossy()));
                }
            }
        }
    }

    // Purge rows from symbols / documents that are not in allowed_files
    let mut stmt = db.prepare("SELECT file_path FROM documents")?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    let mut purged = 0;
    for file in rows.flatten() {
        if !allowed_files.contains(&file) {
            db.execute("DELETE FROM documents WHERE file_path = ?1", [&file])?;
            purged += 1;
        }
    }
    Ok(purged)
}
```
