# Phase 07: Headroom Deep Integration & Gateway Catalog

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .gemini/antigravity/brain/PHASE_07_HEADROOM_AND_GATEWAY_CATALOG.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Source: neo/inspect-vault/headroom, neo/inspect-vault/bifrost, neo\inspect-vault\tokless as updated please rerun

Execute Phase 07: Headroom Deep Integration & Gateway Catalog.
Sources: neo/inspect-vault/headroom (crates/headroom-core) and neo/inspect-vault/bifrost, neo\inspect-vault\tokless (core/providers/, plugins/)
Targets: crates/toksi/src/headroom/, crates/toksi/src/gateway/, crates/toksi/src/

Requirements:
1. Drop Python headroom-py completely.
2. Implement core token compression in crates/toksi/src/headroom/:
   - compression/log.rs: repetition-collapsing log compressor; guarantees FATAL, ERROR, and panic lines survive byte-for-byte.
   - compression/code.rs: AST-aware whitespace and comment pruner.
   - compression/json.rs: structural JSON minifier.
   - pruner/mcp_pruner.rs: dynamic MCP tool schema optimizer (slashes 5,000–15,000 wasted input tokens per turn).
   - cache_align.rs: 1024-token prompt cache alignment for 90% cache discounts on Anthropic, OpenAI, Gemini.
   - adaptive_mem.rs: dynamic memory governor (scales injected memory between L0/L1/L2 based on headroom).
3. Implement Native Enterprise AI Gateway (crates/toksi/src/gateway/):
   - providers/: 39+ in-tree provider clients (reqwest, async streaming).
   - cascade.rs: dynamic failover with 8 failure categories.
   - loadbalancer.rs: round-robin, least-latency, lowest-cost, weighted selectors.
   - routing.rs: task complexity router (SIMPLE, MEDIUM, COMPLEX).
   - governance.rs: virtual keys (x-toksi-vk), USD budgets, TPM/RPM limits.
   - semantic_cache.rs: dual-tier caching (Direct SHA-256 + vector similarity >= 0.92).
   - compat/: drop-in SDK endpoints (/v1/chat/completions, /v1/messages, /v1beta/models/*).
4. Verification:
   - Ensure 0 compiler warnings and passing unit tests.
```

