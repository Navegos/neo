# Phase 07: Headroom Deep Integration & Gateway Catalog

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_07_HEADROOM_AND_GATEWAY_CATALOG.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Sources: neo/inspect-vault/headroom, neo/inspect-vault/bifrost, and neo/inspect-vault/tokless is updated; treat as newly added/updated code that must be converted, imported, and rechecked for new features, bugfixes, and upstream provider catalog additions into consolidated toksi.

Execute Phase 07: Headroom Deep Integration & Gateway Catalog.
Sources:
  - neo/inspect-vault/headroom (crates/headroom-core)
  - neo/inspect-vault/bifrost (core/providers/, plugins/)
  - neo/inspect-vault/tokless (crates/toksi/src/)
Targets:
  - crates/toksi/src/headroom/
  - crates/toksi/src/gateway/

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): #![deny(unsafe_code)] enforced. Zero compiler warnings (RUSTFLAGS="-D warnings").
2. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors.
3. Dedicated Port Topology:
   - Port 8787: Headroom Streaming Proxy (Antigravity, Grok)
   - Port 8789: GitHub Copilot VS Code Proxy (`DEFAULT_COPILOT_PROXY_PORT`)
   - Port 8790: GitHub Copilot CLI Proxy (`DEFAULT_COPILOT_CLI_PROXY_PORT`)
   - Port 9099: Enterprise AI Gateway & Drop-in SDKs (Inference thread pool, <10 µs latency overhead)
   - Port 9100: Web Dashboard Cockpit (Isolated listener, zero backpressure on LLM streaming)
4. Strict Paid Quota Enforcement: All BYOK providers must have an active paid commercial tier. Free-tier accounts are gracefully disqualified without failure, logging an informative notice and falling back cleanly.
5. Ground-Truth Token Accounting: Real tokens parsed from live SSE streams; prohibition of virtual/synthetic math.

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
   - providers/: 39+ in-tree provider clients (reqwest, async streaming), including self-hosted Bifrost, SageMaker, Mantle.
   - cascade.rs: dynamic failover with 8 failure categories.
   - loadbalancer.rs: round-robin, least-latency, lowest-cost, weighted selectors.
   - routing.rs: task complexity router (SIMPLE, MEDIUM, COMPLEX).
   - governance.rs: virtual keys (x-toksi-vk), USD budgets, TPM/RPM limits.
   - semantic_cache.rs: dual-tier caching (Direct SHA-256 + vector similarity >= 0.92).
   - compat/: drop-in SDK endpoints (/v1/chat/completions, /v1/messages, /v1beta/models/*).
4. Supervise Copilot dual-port architecture: Port 8789 for VS Code extension and Port 8790 for Copilot CLI with collision prevention.
5. Ensure 0 compiler warnings and passing unit tests.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Dynamic MCP Schema Pruner** (`src/headroom/pruner/mcp_pruner.rs`): Inspects active chat turns, detects unused tools, and strips verbose schema descriptions, saving 5k-15k tokens per request.
- **Log Compression Engine** (`src/headroom/compression/log.rs`): Collapses repeating stack traces while ensuring panic, fatal, and error lines survive byte-for-byte.
- **Prompt Cache Aligner** (`src/headroom/cache_align.rs`): Pads system prompts to multiples of 1024 tokens to maximize provider cache hits.
- **Enterprise AI Gateway & Cascade** (`src/gateway/`): Dispatches LLM calls across 39+ providers with circuit breakers and cost/latency-optimized load balancing.

### 2. Disciplinary Standards & Invariants
- High-throughput async streaming using Tokio work-stealing scheduler. Zero blocking calls on the reactor threads.
- Port 9099 dedicated strictly to inference; no web dashboard assets served from port 9099.

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- headroom` and `cargo test -p toksi -- gateway` pass 100%.
- Prompt cache aligner produces exact 1024-token aligned boundaries.
