# Phase 14: OS Keyring Integration, Config SQLite DB & BYOK Gateway Pre-Registration

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_14_KEYRING_AND_CONFIG_SQLITE_DB.md first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 14: OS Keyring Integration, Config SQLite DB & BYOK Gateway Pre-Registration.
Targets:
  - crates/toksi/src/keyring/
  - crates/toksi/src/db/
  - crates/toksi/src/commands/auth.rs
  - crates/toksi/src/commands/db.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly across `crates/toksi/src/`. Isolate unavoidable Windows Credential Manager Win32 FFI calls (`advapi32.dll`: `CredReadW`, `CredWriteW`, `CredDeleteW`) inside leaf crate `crates/toksi-sys` with explicit `// SAFETY:` proofs.
2. Zero Warnings Policy: Compile cleanly under `RUSTFLAGS="-D warnings"`.
3. Zero Panics Policy: Strictly prohibit `.unwrap()`, `.expect()`, `panic!()`. Use `thiserror` for domain errors.
4. Database Storage Mandate: Store `toksi.db` by default in user home (`~/.toksi/toksi.db`), NOT in `localdata/toksi` (`AppData/Local/toksi`). Multi-instance `<byok, model, effort>` support via `byok_profiles` table (`UNIQUE(provider_id, instance_label, model_id, thinking_effort)`), `agent_profile_bindings`, `quota_windows`, `plugins_registry` (Unified Plugin manifest, `blake3_hash`, capabilities, protocols, `security_scan_status`), `blind_signatures`, `advisory_embargoes`, and `adr_audit_events`.
5. Migration & Transfer Invariant: Auto-migrate legacy `LOCALAPPDATA/toksi/toksi.db` and JSON registries on startup; fix `db/operations.rs` `export_dump()` (must include `provider_models`, `byok_profiles`, `gateway_configs`, `agent_profile_bindings`, `quota_windows`, `plugins_registry`, `blind_signatures`, `advisory_embargoes`, `adr_audit_events`) and `import_dump(..., overwrite: true)`. Zero raw exploit PoCs/regexes ever persisted in SQLite.
6. Keyring Normalization & Privacy: Fix `keyring/mod.rs:216` broken `("antigravity", "gemini", "antigravity")` lookup so `gemini:antigravity` alias resolves cleanly; display ONLY user-configured agents/BYOK credentials in UI/CLI; zeroize all secret buffers on drop.

Requirements:
1. Native OS Keyring Integration (`crates/toksi/src/keyring/`):
   - Windows Credential Manager (`advapi32.dll`: `CredReadW`, `CredWriteW`, `CredDeleteW`)
   - macOS Keychain (`/usr/bin/security` CLI bridge)
   - Linux Secret Service (`secret-tool` / D-Bus)
   - Base64 OAuth envelope parser extracting access tokens, refresh tokens, and expiration timestamps.
2. SQLite Configuration Database (`crates/toksi/src/db/`):
   - SQLite WAL mode connection pool (`~/.toksi/toksi.db`).
   - Tables: `providers`, `provider_keys`, `models`, `byok_routes`, `agent_configs`.
   - Operations: Export, import, backup, restore, and row-level table merge.
3. Pre-Registration Pipeline:
   - Synchronize discovered OS credentials (e.g. GitHub Copilot, Gemini CLI, Claude Code) into the gateway provider keys table with automatic token refresh.
4. Expose CLI commands `toksi auth ...` and `toksi db ...` for **35-component `<toksi-siagent>`** (**495 mandated vaults**, **43** `.agents/brain/*.md` guides `PHASE_01`..`PHASE_19_6`, `DirectWireBus`, shared 7-entrypoint `HeadroomEffortCompressor`, zero paid-tier restrictions, canonical `<repo>/.toksi/projectmem/`, and externalized zero-bloat `<skills>` & `<advisory>`).
5. Ensure 0 compiler warnings and passing unit tests.
```

---

## Technical Specifications & Details

### 1. Architectural Scope & Subsystems
- **Windows Credential Manager** (`src/keyring/os_windows.rs`): Direct Win32 FFI querying credentials prefixed with `toksi:`, `git:https://github.com`, and `gemini:`.
- **OAuth Envelope Parser** (`src/keyring/envelope.rs`): Decodes JSON/base64 token blobs to extract valid bearer tokens without writing unencrypted secrets to disk.
- **SQLite Configuration Store** (`src/db/`): Stores routing rules, provider priorities, and virtual keys in user home (`~/.toksi/toksi.db`).
- **Automated Migration Engine** (`src/db/migrate.rs`): Detects and imports legacy SQLite databases from `LOCALAPPDATA` on first boot.

### 2. Disciplinary Standards & Invariants
- All sensitive string types must zeroize their allocations on drop.
- Database queries must use parameterized SQL to prevent SQL injection vulnerabilities.

### 3. Verification Criteria
- `cargo check -p toksi` passes with 0 warnings.
- `cargo test -p toksi -- keyring` and `cargo test -p toksi -- db` pass 100%.
- `toksi auth status` displays authenticated providers without revealing raw secrets.

---

## Wave 18 (v5.2) Synchronization Addendum (`AGY_ADC_AUTH=true`, `plugins_registry`, SPIFFE IAM & Zero-Plaintext Security DDL)
- Extend `byok_profiles` table in SQLite with `auth_mode`, `gcp_project`, and `gcp_location` (default `us-central1`), and add `plugins_registry`, `blind_signatures` (`blake3_ngram | sparse_dfa_b64`), `advisory_embargoes` (`DisclosureGate`), and `adr_audit_events` (`allowed | sandboxed | blocked | kill_switch_triggered`) ensuring zero plaintext exploits ever touch disk across all **495 mandated vaults**, **35-component `<toksi-siagent>`**, and **43** `.agents/brain/*.md` guides (`zero paid-tier restrictions`, canonical `<repo>/.toksi/projectmem/`).
- Add `src/keyring/adc.rs` implementing first-class Google Cloud Application Default Credentials (`AGY_ADC_AUTH=true`), reading VS Code `"gcp.project"` / `"gcp.location"` and `~/.config/gcloud/application_default_credentials.json` (`%APPDATA%/gcloud/application_default_credentials.json` on Windows) with actionable `gcloud auth application-default login --project <GCP_PROJECT>` diagnostics when absent (`PHASE_18_3`), plus SPIRE SVID token storage in OS Keyring (`PHASE_18_6`).

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`
- **Wave 21 Reconciliation:** Synchronized to Wave 21 / v8.0 (495 deep-researched vaults [493 Wave 20 + 2 Wave 21], 55 .agents/brain/*.md guides, NetworkService manager, Codex/Grok Chat TUI, 13-IDE host, Multi-Agent Resume, and 8-state progress badges).

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
