# Phase 16.2: High-Performance Web Telemetry Cockpit on Port 9100

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .gemini/antigravity/brain/PHASE_16_2_WEB_COCKPIT_DASHBOARD.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 16.2: High-Performance Web Telemetry Cockpit on Port 9100.
Targets:
  - crates/toksi/src/dashboard/mod.rs
  - crates/toksi/src/dashboard/api.rs
  - crates/toksi/src/dashboard/sse.rs
  - crates/toksi/src/dashboard/assets/index.html
  - crates/toksi/src/dashboard/assets/style.css
  - crates/toksi/src/dashboard/assets/app.js
  - crates/toksi/src/commands/dashboard.rs

Requirements:
1. Isolated Dedicated Listener on Port 9100:
   - Web Dashboard Cockpit binds to `127.0.0.1:9100` (or `DEFAULT_DASHBOARD_PORT`), leaving Port 9099 strictly isolated for AI Gateway LLM inference streaming.
   - Non-blocking background launch via `toksi dashboard [--port 9100] [--open]` without monopolizing the terminal window.
2. 17-Agent Live Observation Grid:
   - Interactive cards for all 17 AI coding assistants (Antigravity, Claude Code, GitHub Copilot, Cursor, Codex, OpenCode, Cline, Grok, etc.).
   - Displays real-time connection status, active provider, model name, prompt/completion token count, and ground-truth tokens/USD saved.
3. Realistic Upstream Quota & Paid Status Tracking:
   - Visual quota gauges displaying exact `x-ratelimit-remaining-tokens` and `x-ratelimit-remaining-requests` from live SSE streams.
   - Distinct badges for billing status: `[PAID TIER ACTIVE]` vs `[FREE TIER DISQUALIFIED]`.
4. Interactive Gateway & Load Balancer Controls:
   - Provider toggling: enable/disable providers in active rotation.
   - Routing algorithm selector: Round-Robin, Least-Latency, Lowest-Cost, Weighted.
   - Complexity tier routing rules: Simple (Groq/Cerebras), Medium (Gemini Flash/Haiku), Complex (Claude 3.7 Sonnet/o3-mini).
5. OS Keyring & Database Management Tools:
   - Web UI triggers for SQLite DB Export, Import, Backup, and Restore.
   - Keyring credential inspection and status overview.
6. Modern Glassmorphic Dark UI Aesthetic:
   - Zero-dependency embedded SPA (`include_str!`).
   - Fast, reactive UI with SSE streaming metrics updates, micro-animations, and full mouse click/interactive capabilities.
7. Verification:
   - Run `cargo check -p toksi` and `cargo test -p toksi` ensuring 0 compiler warnings and all tests pass.
```

---

## Technical Specifications & Details

### 1. HTTP Endpoints on Port 9100
- `GET /`: Embedded `index.html`
- `GET /style.css`: Embedded `style.css`
- `GET /app.js`: Embedded `app.js`
- `GET /api/telemetry/stream`: SSE real-time event broadcaster
- `GET /api/agents`: 17-agent status JSON
- `GET /api/providers`: 39+ provider registry and quota status
- `POST /api/gateway/route`: Update load balancer or provider weight
- `POST /api/db/backup`: Trigger SQLite database snapshot

### 2. Assets Embedded in Binary
In `crates/toksi/src/dashboard/assets/`:
- Bundled directly using `include_str!("assets/index.html")`, `include_str!("assets/style.css")`, `include_str!("assets/app.js")`. Zero runtime file dependency or external CDN calls.

