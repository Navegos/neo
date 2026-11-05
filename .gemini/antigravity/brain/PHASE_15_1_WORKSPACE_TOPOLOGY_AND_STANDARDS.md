# Phase 15.1: Workspace Topology, Forward-Slash Normalization & Standards Normalization

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .gemini/antigravity/brain/PHASE_15_1_WORKSPACE_TOPOLOGY_AND_STANDARDS.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 15.1: Workspace Topology, Forward-Slash Normalization & Standards Normalization.
Targets:
  - crates/toksi/src/util/paths.rs
  - crates/toksi/src/db/mod.rs
  - crates/toksi/src/db/operations.rs
  - crates/toksi/src/commands/init.rs
  - crates/toksi/src/commands/doctor.rs
  - crates/toksi/src/agents/mcp_adapters.rs
  - crates/toksi/src/agents/hooks.rs

Requirements:
1. Universal Forward-Slash (/) Path Standard:
   - Ensure `normalize_path(p: &str) -> String` converts all backward slashes `\` to `/` across all operating systems (Windows, Linux, macOS) for Git Bash and POSIX compatibility.
   - Exception for Shell & Config Escapes: For JavaScript / TypeScript / Node.js / Python (hook, MCP) JSON5 / TOML configurations requiring cmd execution (e.g., `.agents/hooks.json` or `settings.json` command strings executing via Windows `cmd.exe`), use default escaped double backslashes: `C:\\Programs\\neo\\bin\\toksi.exe`.
2. Default SQLite Database Location:
   - Store `toksi.db` by default in `~/.toksi/toksi.db` (`users/.toksi/toksi.db`), NOT in `localdata/toksi` (`AppData/Local/toksi`).
   - Implement seamless startup migration from legacy `LOCALAPPDATA/toksi` to `~/.toksi/toksi.db`.
3. Workspace Root Consolidation:
   - In workspace directory structure, ONLY the first (root) workspace directory contains `.toksi/`.
   - Subfolder `.toksi` directories (e.g., `crates/toksi/.toksi`) are automatically discovered by `toksi init` and `toksi doctor --fix`, imported into root `<root>/.toksi/`, and the legacy subfolder `.toksi` deleted.
   - Crucial: Merge SQLite tables (`toksi.db`, `memory.db`, `symbols.db`) row-by-row without data loss during consolidation.
4. Tool Directory Layout:
   - All tool directories live strictly inside `.toksi/<tool>` in both user home and workspace root:
     * `.toksi/codegraph`
     * `.toksi/headroom`
     * `.toksi/projectmem`
     * `.toksi/context` (renamed from `.context-mode`)
     * `.toksi/icm`
     * `.toksi/ai-memory`
     * `.toksi/rtk`
   - Automatically migrate loose legacy folders (`.codegraph`, `.projectmem`, `.context-mode`, `.icm`, `.ai-memory`, `.rtk`) into `.toksi/<tool>`.
5. Root-Only Workspace Agent Dirs:
   - Agent config folders (`.claude`, `.cursor`, `.vscode`, `.grok`, `.cline`) ONLY exist in the first (root) workspace directory.
6. Standardized Tool Hooks & MCP Naming:
   - Uniform hook command syntax: `<tool>-hook <agent>` (e.g., `codegraph-hook agy`, `projectmem-hook agy`, `context-hook agy`, `rtk-hook agy`).
   - Clean MCP tool identifiers: `toksi-projectmem` -> `projectmem`, `toksi-context-mode` -> `context`, `toksi-codegraph-index` -> `codegraph`.
   - MCP instruction directories: `context-mode` -> `context`.
   - Differentiate agent CLI vs agent editor: `antigravity;agy;antigravity-ide`, `copilot-cli;copilot-vscode`, `claude-cli;claude-desktop`, `cursor-cli;cursor-ide`, `grok-cli;grok-ide`.
7. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. Forward-Slash Path Normalizer
In `crates/toksi/src/util/paths.rs`:
```rust
pub fn normalize_path(path: &str) -> String {
    let replaced = path.replace('\\', "/");
    // Preserve Windows drive letter format C:/...
    if replaced.len() >= 2 && replaced.chars().nth(1) == Some(':') {
        let drive = replaced.chars().next().unwrap().to_ascii_uppercase();
        format!("{}:{}", drive, &replaced[2..])
    } else {
        replaced
    }
}
```

### 2. User Home Database Storage
In `crates/toksi/src/util/paths.rs` and `crates/toksi/src/db/mod.rs`:
- Update `default_toksi_data_dir()` to return `~/.toksi` (i.e. `format!("{}/.toksi", normalize_path(&home()))`).
- On startup, check if legacy `LOCALAPPDATA/toksi/toksi.db` exists and `~/.toksi/toksi.db` does not exist: copy or migrate rows into `~/.toksi/toksi.db`.

### 3. Subfolder `.toksi` Discovery & Row-Level SQLite Merge
In `crates/toksi/src/commands/init.rs` and `crates/toksi/src/db/operations.rs`:
- Traverse subdirectories looking for `.toksi` or legacy `.tokless`.
- If found outside workspace root:
  * For `.toksi/toksi.db`: attach or open database, copy rows `INSERT OR IGNORE INTO root.table SELECT * FROM sub.table` for `providers`, `provider_keys`, `byok_routes`, `agent_configs`.
  * For file-based stores (`events.jsonl`, `wiki/`): merge contents cleanly.
  * Remove the subfolder `.toksi` directory.

### 4. Normalized Hooks & MCP Names
In `crates/toksi/src/agents/mcp_adapters.rs` and `crates/toksi/src/agents/hooks.rs`:
- MCP tool map:
  * `projectmem` for `toksi mem mcp`
  * `context` for `toksi ctx mcp`
  * `codegraph` for `toksi graph mcp`
- Hooks command: `<tool>-hook <agent>` format.

