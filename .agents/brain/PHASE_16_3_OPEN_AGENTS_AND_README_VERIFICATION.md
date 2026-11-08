# Phase 16.3: Open Agents Directory Protocols & README Verification

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_16_3_OPEN_AGENTS_AND_README_VERIFICATION.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 16.3: Open Agents Directory Protocols & README Verification.
Targets:
  - crates/toksi/src/agents/mod.rs
  - crates/toksi/src/agents/manager.rs
  - crates/toksi/src/agents/open_agents.rs
  - crates/toksi/src/commands/agent.rs
  - crates/toksi/README.md

Requirements:
1. Open Agents & Vendor Agent Directory Support:
   - Support `.agents/` (OpenAgents standard), `.gemini/` (Antigravity), `.copilot/` & `.github/` (Copilot), `.claude/` (Claude), `.codex/` (Codex), `.cursor/` (Cursor), `.grok/` (Grok).
   - User-Level vs Workspace Installation Policy:
     * Auto-install hooks, MCPs, instructions, rules, skills, configs to user directory (`~/.<agent>/` or `~/.agents/`) if selected.
     * Workspace repository install is NOT automatic; user chooses to install into workspace repository when needed.
   - Management operations via CLI (`toksi agent ...`), TUI, and Web Dashboard: `install`, `health`, `mismatch`, `upgrade`, `fix`, `restore`, `remove`, `edit`.
2. Comprehensive Technical README:
   - Full rewrite/expansion covering 5-port architecture, single daemon launch, 17 agents, 39+ providers, smart agent CLI provider bridges, paid quota enforcement, multi-protocol support, rules sync, and full CLI reference.
3. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and 100% tests pass.
```

---

## Technical Specifications & Details

### 1. Agent Management Subcommand in `commands/agent.rs`
```rust
pub fn handle_agent_command(args: &[String]) -> i32 {
    let sub = args.get(0).map(|s| s.as_str()).unwrap_or("health");
    match sub {
        "install" => { /* install hooks/mcps to user dir, or workspace if --workspace specified */ 0 },
        "health" => { /* check hook syntax, mcp configs, missing binaries */ 0 },
        "fix" | "upgrade" => { /* auto-heal mismatches */ 0 },
        "remove" => { /* clean removal */ 0 },
        _ => { eprintln!("Unknown agent subcommand: {}", sub); 1 }
    }
}
```
