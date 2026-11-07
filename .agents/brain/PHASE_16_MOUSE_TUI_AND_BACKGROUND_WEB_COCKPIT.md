# Phase 16: Mouse-Capable TUI & Enterprise Web UI Cockpit

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_16_MOUSE_TUI_AND_BACKGROUND_WEB_COCKPIT.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 16: Mouse-Capable TUI & Enterprise Web UI Cockpit.
Targets:
  - crates/toksi/src/tui/{mod.rs, app.rs, views/*, widgets/*}
  - crates/toksi/src/dashboard/{assets/index.html, assets/style.css, assets/app.js, api.rs, server.rs}
  - crates/toksi/README.md

Requirements:
1. Mouse-Enabled Interactive TUI (crates/toksi/src/tui/):
   - Enable mouse capture in Crossterm (`EnableMouseCapture` on start, `DisableMouseCapture` on exit).
   - Route mouse click events in `app.rs`: clicking tabs switches tabs, clicking rows selects agents/providers, mouse wheel scrolls logs.
2. Background Web Cockpit on Port 9100 (crates/toksi/src/dashboard/assets/):
   - Modern glassmorphic dark UI served in the background on `http://127.0.0.1:9100`.
   - Real-time 17-agent observation grid with live ground-truth token savings, connected provider, model, latency breakdown, upstream quota gauges, and paid tier status badges.
   - Interactive Gateway & Load Balancer controls (toggle providers, adjust weights/priorities, switch algorithms, complexity tiers).
   - OS Keyring and SQLite DB management tools (Export, Import, Backup, Restore).
3. Comprehensive Technical README (crates/toksi/README.md):
   - Full rewrite/expansion covering 5-port architecture, single daemon launch, 17 agents, 39+ providers, smart agent CLI provider bridges, paid quota enforcement, rules sync, and full CLI reference.
4. Verification:
   - Ensure 0 compiler warnings, clean test execution, and pristine UI presentation.
```

---

## Detailed Architectural Requirements

### 1. Crossterm Mouse Capture in TUI
In `crates/toksi/src/tui/mod.rs`:
- On terminal initialization:
  ```rust
  execute!(stdout, EnterAlternateScreen, Hide, EnableMouseCapture)?;
  ```
- In `TerminalGuard::drop`:
  ```rust
  let _ = execute!(stdout(), Show, LeaveAlternateScreen, DisableMouseCapture);
  ```
- In `crates/toksi/src/tui/app.rs`:
  * Handle `MouseEventKind::Down(MouseButton::Left)`:
    - Tab bar click: detect row and column bounds, switch active tab.
    - Table click: select agent row or provider card.
    - Button click: toggle wire/unwire or trigger precheck.
  * Handle `MouseEventKind::ScrollUp` and `MouseEventKind::ScrollDown`: scroll active view.

### 2. Background Web Cockpit on Port 9100
In `crates/toksi/src/dashboard/assets/`:
- `index.html`:
  * Add 17-Agent Observation Grid (cards for Antigravity, Claude, Copilot, Cursor, etc.).
  * Add Provider Management view with paid tier badges (`Paid Tier Active`, `Free Tier Disqualified`).
  * Add Load Balancer strategy selector (Round-Robin, Least-Latency, Lowest-Cost, Weighted).
  * Add Upstream Quota gauges displaying real `x-ratelimit-remaining-tokens`.
- `style.css`:
  * Modern dark glassmorphic theme with responsive CSS grid.
- `app.js`:
  * Connect to `/api/telemetry/stream` SSE stream on port 9100.
  * Full mouse click and toggle event handlers.

### 3. README.md Rewrite
Full technical documentation in `crates/toksi/README.md` covering:
- Master 5-port architecture.
- Single daemon launch (`toksi daemon start`).
- 17 coding assistants setup.
- 39+ BYOK providers and Local Agent CLI bridges.
- Paid quota enforcement policy.
- Full CLI reference with flags.

### 4. Verification
Run:
```bash
cargo check -p toksi
cargo test -p toksi
```

