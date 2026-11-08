# Phase 15.6: Advanced Vault Intelligence Engines Consolidation

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_15_6_ADVANCED_VAULT_INTELLIGENCE_ENGINES.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 15.6: Advanced Vault Intelligence Engines Consolidation.
Targets:
  - crates/toksi/src/telemetry/tokentracker.rs
  - crates/toksi/src/codegraph/jcodemunch.rs
  - crates/toksi/src/codegraph/repomix.rs
  - crates/toksi/src/codegraph/ast_grep.rs
  - crates/toksi/src/codegraph/scip.rs
  - crates/toksi/src/aimemory/mem0_bridge.rs
  - crates/toksi/src/gateway/semantic_cache.rs
  - crates/toksi/src/gateway/normalizer/litellm.rs

Requirements:
1. Ingest & Consolidate Remaining Inspect-Vault Engines:
   - TokenTracker (`neo/inspect-vault/TokenTracker`): Real-time token tracking, byte-to-token estimators, quota stats.
   - jcodemunch-mcp (`neo/inspect-vault/jcodemunch-mcp`): AST outline navigation, semantic symbol chunking.
   - Repomix (`neo/inspect-vault/repomix`): Codebase packager, tree generation with accurate token weights and .gitignore rules.
   - ast-grep (`neo/inspect-vault/ast-grep`): Structural code search and AST pattern matching.
   - scip (`neo/inspect-vault/scip`): SCIP code intelligence protocol indexer.
   - mem0 (`neo/inspect-vault/mem0`): Personalized long-term memory extraction.
   - Chroma (`neo/inspect-vault/chroma`): Vector embedding similarity search backing semantic cache.
   - LiteLLM (`neo/inspect-vault/litellm`): 100+ model alias mapping, token pricing tables, and drop-in format conversions.
2. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. Repomix Codebase Packaging Engine in `repomix.rs`
```rust
pub struct RepomixConfig {
    pub include_patterns: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub max_file_size_bytes: usize,
    pub token_counter_enabled: bool,
}

pub fn pack_codebase(root: &std::path::Path, cfg: &RepomixConfig) -> Result<String, String> {
    // Generate directory tree, file manifests, and concatenated representations
    // while honoring .gitignore and exact token counting.
    Ok("packed".to_string())
}
```

### 2. LiteLLM Provider Normalization in `litellm.rs`
```rust
pub struct ModelCostRecord {
    pub model_name: &'static str,
    pub input_cost_per_token: f64,
    pub output_cost_per_token: f64,
    pub cache_read_cost_per_token: f64,
    pub max_tokens: usize,
}

pub fn normalize_model_name(requested_model: &str) -> &'static str {
    match requested_model {
        "gpt-4o" | "openai/gpt-4o" => "gpt-4o",
        "claude-3-7-sonnet" | "anthropic/claude-3-7-sonnet" => "claude-3-7-sonnet-20250219",
        "gemini-2.0-flash" | "google/gemini-2.0-flash" => "gemini-2.0-flash",
        _ => "default-model",
    }
}
```
