# Phase 19.2 — Dedicated Multi-Process Supervisor (`toksi _worker <role>`) & Direct-Wire IPC

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-UNIMPLEMENTED]`

## Ready-to-Run Chat Prompt
```text
Read ALL 10 canonical workspace files FIRST before editing or planning anything:
1. crates/toksi/research/RESEARCHPLAN.md
2. crates/toksi/CONSOLIDATION_PLAN.md
3. crates/toksi/PLAN.md
4. .toksi/projectmem/plan.md
5. .toksi/projectmem/PROJECT_MAP.md
6. .toksi/projectmem/summary.md
7. crates/toksi/research/full_ai_ecosystem_agent_runtimes_local_inference_engines_and_quota_architecture.md
8. crates/toksi/research/docker_sandboxes_sbx_cli_reference.md
9. crates/toksi/research/folders_structures_default_knowledge_used_by_the_known_cli_agents_ides.md
10. crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md
Plus .agents/brain/PHASE_19_2_DEDICATED_MULTIPROCESS_SUPERVISOR_AND_IPC.md.

Execute Phase 19.2: Dedicated Multi-Process Supervisor (`toksi _worker <role>`) & Direct-Wire IPC (RESEARCHPLAN.txt:586, 596).
Sources (inspect-vault/*): pingora (pingora-core/pool/proxy), iggy (core/binary_protocol/server), quinn (quinn/quinn-proto), rpmalloc, rpmalloc-rs
Targets:
  - crates/toksi/src/commands/daemon.rs (refactor single-process tokio::spawn tasks at L491-588)
  - crates/toksi/src/commands/worker.rs (NEW multi-call worker entrypoint `toksi _worker <role>`)
  - crates/toksi/src/ipc/{mod.rs,frame.rs,pipe_windows.rs,uds_unix.rs,client.rs,server.rs} (NEW Direct-Wire IPC mesh)
  - crates/toksi/src/config/mod.rs (`[supervisor]` section)
  - crates/toksi/src/db/{schema.rs,operations.rs} (`supervisor_workers` table)
  - crates/toksi/src/commands/guard.rs (global multi-process kill-switch fan-out)
  - crates/toksi-sys/Cargo.toml & src/lib.rs (optional Named Pipe / allocator helpers with // SAFETY:)
  - crates/toksi/src/dashboard/api.rs (`/api/supervisor/workers`), crates/toksi/src/tui/* (Tab 7 worker status/controls)
  - crates/toksi/tests/phase19_2_multiprocess_supervisor_ipc_verification.rs (NEW)

Requirements:
1. Implement multi-call child binary entrypoint `toksi _worker <role> [--port <p>] [--pipe <id>]`:
   - 8 isolated Network Port workers: `port-8787` (Headroom Proxy), `port-8788` (Grok OAuth), `port-8789` (Copilot VS Code), `port-8790` (Copilot CLI), `port-9098` (Protocol Bus), `port-9099` (Enterprise AI Gateway + `/metrics`), `port-9100` (Web Cockpit Dashboard), `port-18787` (BYOK & Vertex ADC Gateway).
   - Dedicated `<toksi-siagent>` runtime worker: `toksi _worker siagent`.
   - Dedicated tool workers: `codegraph`, `ctxmode`, `projectmem`, `icm`, `aimemory`, `rtk`, `security`.
2. Refactor `src/commands/daemon.rs` into a lean parent supervisor (`toksi daemon start|stop|restart|status`):
   - Spawn detached child workers (`CREATE_NO_WINDOW` on Windows, `process_group(0)` on Unix), store PIDs under `~/.toksi/supervisor/pids/<role>.pid` and SQLite `supervisor_workers`.
   - Implement heartbeat monitoring, exponential backoff restart (`restart_backoff_ms`, `max_restarts_per_min`), Pingora-grade graceful worker drain, and clean child tree reaping on `stop`/`SIGINT`/`SIGTERM`.
3. Build `src/ipc/` Direct-Wire IPC transport:
   - Windows Named Pipes (`//./pipe/toksi-<role>-<uid>`) and POSIX Unix Domain Sockets (`$XDG_RUNTIME_DIR/toksi/<role>.sock`) using length-prefixed `iggy`-style binary frames, with `DirectWireBus` in-process fallback for one-shot commands and optional `quinn` QUIC transport for remote nodes.
   - Connect all 7 Headroom-compressed entrypoints (`:8787`..`:8790`, `:9098`, `:9099`, `:18787`), `<toksi-siagent>`, and all tool workers out of the box over Direct-Wire IPC.
4. Wire `src/commands/guard.rs` Mandatory Bypass Kill-Switch to broadcast immediate fail-closed `KillSwitchTriggered` across IPC to terminate ALL supervised worker subprocesses and host daemon if any bypass attempt occurs.
5. Add optional `rpmalloc` cargo feature (`rpmalloc-rs@0.2.2`) benchmarkable for heavy multi-worker AST/SSE loads without changing default `#![deny(unsafe_code)]` on `crates/toksi`.
6. Update `.toksi/test-baseline.json` and all 6 canonical plan destinations; verify with `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test -p toksi --test phase19_2_multiprocess_supervisor_ipc_verification`.
```

---

## Technical Specifications & Details

### 1. Verified Reference Vault Primitives
- **`pingora` (24 crates):** `pingora-core` graceful reload & listener drain, `pingora-pool` lock-free upstream pooling, `pingora-limits` rate limiters.
- **`iggy` (55 crates):** `core/binary_protocol` compact header/payload wire framing and deterministic offset streaming.
- **`quinn` (7 crates):** `quinn` / `quinn-proto` multiplexed zero-HOL QUIC streams for optional remote worker nodes.
- **`rpmalloc-rs` (`0.2.2`):** Isolated in `crates/toksi-sys` behind opt-in cargo feature flag.

### 2. SQLite `supervisor_workers` Table Contract
```sql
CREATE TABLE IF NOT EXISTS supervisor_workers (
    worker_role      TEXT PRIMARY KEY,
    pid              INTEGER,
    ipc_endpoint     TEXT NOT NULL,
    status           TEXT NOT NULL DEFAULT 'stopped',
    restart_count    INTEGER NOT NULL DEFAULT 0,
    last_heartbeat   INTEGER NOT NULL
);
```

## Verification Criteria
- `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `phase19_2_multiprocess_supervisor_ipc_verification` pass 100% using ephemeral `TempDir` sockets/pipes and `:0` ports.

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
