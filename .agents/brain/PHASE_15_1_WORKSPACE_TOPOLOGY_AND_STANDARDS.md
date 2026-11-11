# Phase 15.1: Workspace Topology, Forward-Slash Normalization, Standards & PATH Validation

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_15_1_WORKSPACE_TOPOLOGY_AND_STANDARDS.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 15.1: Workspace Topology, Forward-Slash Normalization, Standards & PATH Validation.
Targets:
  - crates/toksi/src/util/paths.rs
  - crates/toksi/src/util/env_path.rs
  - crates/toksi/src/db/mod.rs
  - crates/toksi/src/db/operations.rs
  - crates/toksi/src/commands/init.rs
  - crates/toksi/src/commands/doctor.rs
  - crates/toksi/src/agents/mcp_adapters.rs
  - crates/toksi/src/agents/hooks.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors and anyhow::Result with .context() in CLI.
3. Universal Forward-Slash (/) Standard: Use normalize_path() for all paths across Windows, Linux, and macOS. Escaped \\ permitted exclusively in cmd.exe commands inside JSON5/TOML configs.
4. PATH Validation Mandate: Validate required binaries in system PATH. Never hardcode absolute paths. On missing binary, log clean remediation instruction.
5. Workspace Topology: Strict `<repo>/.toksi/<tool>` repo-only storage (NEVER in `~/.toksi/<tool>` for workspace tools, NEVER outside `.git` repo root, NEVER inside `<repo>/.git/` or subfolder crates). `commands/index.rs::find_project_dir()` walks to `.git` root and partitions `<repo>/crates/*` via `subproject_id`. Subfolder `.toksi/` and `.<agent>/` dirs are merged into root and deleted.

Requirements:
1. Universal Forward-Slash (`/`) Path Standard:
   - Ensure `normalize_path(p: &str) -> String` converts all backward slashes `\` to `/` across all operating systems (Windows, Linux, macOS) for Git Bash and POSIX compatibility.
   - Exception for Shell & Config Escapes: For JavaScript / TypeScript / Node.js / Python (hook, MCP) JSON5 / TOML configurations requiring cmd execution (e.g., `.agents/hooks.json` or `settings.json` command strings executing via Windows `cmd.exe`), use default escaped double backslashes: `C:\\Programs\\neo\\bin\\toksi.exe`.
2. Environment PATH Validation Mandate ("Rule of Test and Inform User"):
   - Implement `check_executable_in_path(cmd: &str) -> Result<(), String>` in `src/util/env_path.rs`.
   - Never encode full paths in settings, configurations, or database records. Keep all references portable by using executable names.
   - Test whether required binaries (`bash`, `cmd`, `toksi`, `agy`, `copilot`, `claude`, `codex`, `cursor`, `grok`, etc.) exist in `PATH`.
   - If missing from `PATH`, log non-blocking actionable notice (and expose `in_path: false` in `AI Agent Fleet & Quotas` UI so missing agents appear greyed out cleanly).
3. Default SQLite Database & TOML Location:
   - Store global config DB by default in `~/.toksi/toksi.db` (`dirs::home_dir()`) + declarative overrides in `~/.toksi/config.toml` (`src/config/mod.rs`), NOT in `localdata/toksi` (`AppData/Local/toksi`).
   - Implement seamless startup migration from legacy `LOCALAPPDATA/toksi` to `~/.toksi/toksi.db`.
4. Workspace Root Consolidation & Monorepo Partitioning:
   - Fix `commands/index.rs::find_project_dir()` so it NEVER stops at nested `Cargo.toml`/`package.json` inside `<repo>/crates/*`; walk strictly to `.git` root.
   - Only `<repo>/.toksi/` exists for project artifacts; if cwd is not inside a `.git` repo, refuse workspace indexing cleanly without creating home/folder stores.
   - Auto-discover subfolder `.toksi/` (e.g. `crates/toksi/.toksi`) and `.projectmem/`, merge SQLite rows (`symbols.db`, `memory.db`, JSONL logs) tagged with `subproject_id`, and delete nested `.toksi` / `.<agent>` folders.
5. Strict `<repo>/.toksi/<tool>` Layout:
   - Project intelligence stores live strictly inside workspace root `<repo>/.toksi/<tool>/`:
     * `<repo>/.toksi/codegraph`
     * `<repo>/.toksi/headroom`
     * `<repo>/.toksi/projectmem`
     * `<repo>/.toksi/context` (renamed from `.context-mode`)
     * `<repo>/.toksi/icm`
     * `<repo>/.toksi/ai-memory`
     * `<repo>/.toksi/rtk`
   - Automatically migrate loose legacy folders (`.codegraph`, `.projectmem`, `.context-mode`, `.icm`, `.ai-memory`, `.rtk`) into `<repo>/.toksi/<tool>`. Remove home-dir workspace leakage (`~/.toksi/{projectmem,icm,ai-memory,context,codegraph}`).
6. Root-Only Workspace Agent Dirs:
   - Agent config folders (`.claude`, `.cursor`, `.vscode`, `.grok`, `.cline`, `.agents`) ONLY exist in root `<repo>/` (or user `~/.<agent>/`), and are NEVER installed into `<repo>/` automatically unless the user opts into repo mode.
7. Standardized Tool Hooks & MCP Naming:
   - Uniform hook command syntax: `<tool>-hook <agent>` (e.g., `codegraph-hook agy`, `projectmem-hook agy`, `context-hook agy`, `rtk-hook agy`; purge `agy-hook codegraph-index` & `rtk hook cursor`).
   - Clean MCP tool identifiers: `toksi-projectmem` -> `projectmem`, `toksi-context-mode` -> `context`, `toksi-codegraph` / `toksi-codegraph-index` -> `codegraph` (purge `tokless_*` in `paths.rs`).
   - MCP instruction directories: `context-mode` -> `context`.
   - MCP settings to skills mapping: map `.gemini/mcp`, `.copilot/mcp`, `.claude/mcp`, `.agents/mcp` to `skills/<tool>`.
   - Differentiate agent CLI vs agent editor: `antigravity-cli` vs `antigravity-ide` (fixing `hooks.rs` collapse), `copilot-cli` vs `copilot-vscode`, `claude-cli` vs `claude-desktop`, `cursor-cli` vs `cursor-ide`, `grok-cli` vs `grok-ide`, plus `"openagents"`.
8. Verification:
   - Run `cargo check -p toksi`, `cargo clippy -p toksi --all-targets -- -D warnings`, and targeted `cargo test -p toksi`.
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
        let drive = replaced.chars().next().map(|c| c.to_ascii_uppercase()).unwrap_or('C');
        format!("{}:{}", drive, &replaced[2..])
    } else {
        replaced
    }
}
```

### 2. PATH Verification Mandate
In `crates/toksi/src/util/env_path.rs`:
```rust
pub fn check_executable_in_path(cmd: &str) -> Result<(), String> {
    if let Some(paths) = std::env::var_os("PATH") {
        for path in std::env::split_paths(&paths) {
            let full = path.join(cmd);
            if full.is_file() {
                return Ok(());
            }
            #[cfg(windows)]
            {
                let with_exe = path.join(format!("{}.exe", cmd));
                if with_exe.is_file() {
                    return Ok(());
                }
            }
        }
    }
    Err(format!(
        "[ERROR] Required executable '{}' not found in system PATH. Please add '{}' to your PATH environment variable and retry.",
        cmd, cmd
    ))
}
```

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- paths` and `cargo test -p toksi -- env_path` pass 100%.
- Single `.toksi` workspace invariant successfully enforced.
