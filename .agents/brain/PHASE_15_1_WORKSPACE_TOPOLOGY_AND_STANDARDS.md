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
5. Workspace Topology: Single root .toksi directory rule. Subfolder .toksi folders are auto-discovered, table rows merged into root SQLite DBs, and subfolders deleted.

Requirements:
1. Universal Forward-Slash (/) Path Standard:
   - Ensure `normalize_path(p: &str) -> String` converts all backward slashes `\` to `/` across all operating systems (Windows, Linux, macOS) for Git Bash and POSIX compatibility.
   - Exception for Shell & Config Escapes: For JavaScript / TypeScript / Node.js / Python (hook, MCP) JSON5 / TOML configurations requiring cmd execution (e.g., `.agents/hooks.json` or `settings.json` command strings executing via Windows `cmd.exe`), use default escaped double backslashes: `C:\\Programs\\neo\\bin\\toksi.exe`.
2. Environment PATH Validation Mandate ("Rule of Test and Inform User"):
   - Implement `check_executable_in_path(cmd: &str) -> Result<(), String>` in `src/util/env_path.rs`.
   - Never encode full paths in settings, configurations, or database records. Keep all references portable by using executable names.
   - Test whether required binaries (`bash`, `cmd`, `toksi`, `agy.exe`, `copilot.exe`, `claude`, `cursor`, `grok`, etc.) exist in `PATH`.
   - If missing from `PATH`, log: `[ERROR] Required executable '<cmd>' not found in system PATH. Please add '<cmd>' to your PATH environment variable and retry.`
3. Default SQLite Database Location:
   - Store `toksi.db` by default in `~/.toksi/toksi.db` (`users/.toksi/toksi.db`), NOT in `localdata/toksi` (`AppData/Local/toksi`).
   - Implement seamless startup migration from legacy `LOCALAPPDATA/toksi` to `~/.toksi/toksi.db`.
4. Workspace Root Consolidation:
   - In workspace directory structure, ONLY the first (root) workspace directory contains `.toksi/`.
   - Subfolder `.toksi` directories (e.g., `crates/toksi/.toksi`) are automatically discovered by `toksi init` and `toksi doctor --fix`, imported into root `<root>/.toksi/`, and the legacy subfolder `.toksi` deleted.
   - Crucial: Merge SQLite tables (`toksi.db`, `memory.db`, `symbols.db`) row-by-row without data loss during consolidation.
5. Tool Directory Layout:
   - All tool directories live strictly inside `.toksi/<tool>` in both user home and workspace root:
     * `.toksi/codegraph`
     * `.toksi/headroom`
     * `.toksi/projectmem`
     * `.toksi/context` (renamed from `.context-mode`)
     * `.toksi/icm`
     * `.toksi/ai-memory`
     * `.toksi/rtk`
   - Automatically migrate loose legacy folders (`.codegraph`, `.projectmem`, `.context-mode`, `.icm`, `.ai-memory`, `.rtk`) into `.toksi/<tool>`.
6. Root-Only Workspace Agent Dirs:
   - Agent config folders (`.claude`, `.cursor`, `.vscode`, `.grok`, `.cline`, `.agents`) ONLY exist in the first (root) workspace directory.
7. Standardized Tool Hooks & MCP Naming:
   - Uniform hook command syntax: `<tool>-hook <agent>` (e.g., `codegraph-hook agy`, `projectmem-hook agy`, `context-hook agy`, `rtk-hook agy`).
   - Clean MCP tool identifiers: `toksi-projectmem` -> `projectmem`, `toksi-context-mode` -> `context`, `toksi-codegraph-index` -> `codegraph`.
   - MCP instruction directories: `context-mode` -> `context`.
   - MCP settings to skills mapping: map `.gemini/mcp`, `.copilot/mcp`, `.claude/mcp` to `skills/<tool>`.
   - Differentiate agent CLI vs agent editor: `antigravity;agy;antigravity-ide`, `copilot-cli;copilot-vscode`, `claude-cli;claude-desktop`, `cursor-cli;cursor-ide`, `grok-cli;grok-ide`.
8. Verification:
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
