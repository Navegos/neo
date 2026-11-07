# Phase 15.4: Ground-Truth Token Accounting & In-Tree Rules Sync

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_15_4_TOKEN_ACCOUNTING_AND_RULES_SYNC.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 15.4: Ground-Truth Token Accounting & In-Tree Rules Sync.
Targets:
  - crates/toksi/src/telemetry/collector.rs
  - crates/toksi/src/telemetry/tracker.rs
  - crates/toksi/src/telemetry/prometheus.rs
  - crates/toksi/src/rules/mod.rs
  - crates/toksi/src/rules/sync.rs
  - crates/toksi/src/rules/principles.rs
  - crates/toksi/src/rules/caveman.rs
  - crates/toksi/src/rules/ponytail.rs
  - crates/toksi/src/main.rs

Requirements:
1. Ground-Truth Session Token Accounting:
   - Extract real prompt, completion, and cached tokens directly from live incoming/outgoing SSE response streams and HTTP headers:
     * Anthropic: `message_delta.usage` (`input_tokens`, `output_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`)
     * OpenAI: `usage.prompt_tokens`, `usage.completion_tokens`, `cached_tokens`
     * Google Gemini: `usageMetadata.promptTokenCount`, `usageMetadata.candidatesTokenCount`, `usageMetadata.cachedContentTokenCount`
     * Upstream rate limit headers: `x-ratelimit-remaining-tokens`, `x-ratelimit-remaining-requests`, `anthropic-ratelimit-requests-remaining`
   - Strict prohibition of synthetic, estimated, or virtual mathematical tokens.
   - Ground-truth savings calculated from real byte reduction and exact MCP schema tool definitions stripped.
   - Attribute metrics directly to the active connected agent ID and upstream provider ID.
2. In-Tree Rules, Skills & Instructions Sync:
   - Implement universal synchronizer in `rules/sync.rs` updating `rules*.md`, `skills*.md`, `instructions*.md`, `agents*.md` across both workspace root and user home config directories for all 17 agents:
     * Antigravity: `GEMINI.md`, `~/.gemini/rules.md`, `.agents/rules.md`
     * Claude Code: `CLAUDE.md`, `~/.claude/CLAUDE.md`
     * Cursor: `.cursorrules`, `.cursor/rules/*.mdc`
     * Copilot: `.github/copilot-instructions.md`
     * Cline: `.clinerules`, `.cline/rules.md`
     * OpenCode, Codex, Droid, Grok, Kilo, OMP, Pi, Amp, Kimi, ZCode, DSH: `AGENTS.md`, `INSTRUCTIONS.md`, `rules.md`
   - Content injected: Karpathy Principles (`principles.rs`), Caveman (`caveman.rs`), Ponytail (`ponytail.rs`).
   - Zero external repo dependencies; rules compiled directly into the binary via `include_str!`.
3. Complete CLI Usage Help:
   - Update `help_text()` in `crates/toksi/src/main.rs` with comprehensive documentation for all subcommands, ports (8787, 8789, 8790, 9099, 9100), flags, and tools.
4. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. Real Token Extraction in `collector.rs`
```rust
pub fn record_live_session_usage(
    agent_id: &str,
    provider_id: &str,
    model: &str,
    prompt_tokens: u64,
    completion_tokens: u64,
    cached_tokens: u64,
    bytes_saved: u64,
    remaining_tokens_quota: Option<u64>,
) {
    // Record into lock-free ring buffer
    // Update Prometheus metrics at :9099/metrics
    // Broadcast live event to Web Dashboard SSE on port 9100
}
```

### 2. Universal Rules Synchronizer in `rules/sync.rs`
- Discover root workspace and user home directories.
- Inject compiled guidelines into each agent configuration file cleanly without clobbering existing custom user instructions.

