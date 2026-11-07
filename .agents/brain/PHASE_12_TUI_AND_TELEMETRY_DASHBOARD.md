# Phase 12: TUI, Prometheus Telemetry & Web Dashboard

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_12_TUI_AND_TELEMETRY_DASHBOARD.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

The code in Source: neo/inspect-vault/bifrost as updated please rerun

Execute Phase 12: TUI, Prometheus Telemetry & Web Dashboard.
Sources: neo/inspect-vault/bifrost (plugins/telemetry/, ui/)
Targets: crates/toksi/src/telemetry/, crates/toksi/src/tui/, crates/toksi/src/dashboard/

Requirements:
1. Implement Native Prometheus Telemetry Subsystem:
   - prometheus.rs: Native metrics exposition in Prometheus text format at HTTP endpoint http://127.0.0.1:9099/metrics.
   - collector.rs: Asynchronous, lock-free non-blocking channel collector (<10 µs latency overhead).
   - cost.rs: In-tree model catalog pricing table with real-time financial tracking ($ USD spend and savings).
   - tracker.rs: Multi-level latency breakdown (transport, provider, internal overhead).
2. Implement full multi-tab interactive TUI in crates/toksi/src/tui/ using `ratatui` and `crossterm`:
   - Tab 1: Live Monitor & Waterfall
   - Tab 2: Agent Wire Manager (17 agents)
   - Tab 3: Gateway & Provider Key Manager
   - Tab 4: Memory Explorer
   - Tab 5: CodeGraph Visualizer
   - Tab 6: Governance & Quotas
   - Tab 7: Diagnostic Logs
3. Implement embedded local Web server (127.0.0.1:9099) in crates/toksi/src/dashboard/:
   - Single-page application embedded via `include_str!` (zero npm/node build dependencies).
   - Live token savings waterfall, dollar counter, agent fleet grid, and SVG decision DAG visualizer.
4. Ensure zero compiler warnings and passing tests.
```

