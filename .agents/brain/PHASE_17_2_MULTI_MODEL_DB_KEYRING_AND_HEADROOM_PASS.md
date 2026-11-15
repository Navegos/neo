# Phase 17.2: Multi-Model `<byok, model, effort>` DB, Keyring Fix, `headroom-core` `live_zone`, & Mandatory BYOK Headroom Pass

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 13.3–13.4), .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, .agents/brain/PHASE_17_2_MULTI_MODEL_DB_KEYRING_AND_HEADROOM_PASS.md first.

Execute Phase 17.2: Multi-Model `<byok, model, effort>` DB, Keyring Fix, `headroom-core` `live_zone`, & Mandatory BYOK Headroom Pass.
Sources:
  - neo/inspect-vault/headroom (crates/headroom-core, crates/headroom-proxy)
  - neo/inspect-vault/chroma
Targets:
  - crates/toksi/src/db/{mod.rs, schema.rs, operations.rs, migrate.rs}
  - crates/toksi/src/commands/{byok.rs, db.rs, auth.rs}
  - crates/toksi/src/keyring/{mod.rs, envelope.rs, os_windows.rs, os_macos.rs, os_linux.rs}
  - crates/toksi/src/headroom/{mod.rs, proxy.rs, byok_gateway.rs, cache_align.rs, ccr.rs, smart_crusher.rs, cloudcode_patch.rs}
  - crates/toksi/src/tools/headroom.rs
  - crates/toksi/src/gateway/semantic_cache.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): `#![deny(unsafe_code)]` enforced strictly; Windows Credential Manager FFI lives in `crates/toksi-sys`.
2. Zero Panics & Prompt Cache Preservation: NEVER rewrite/strip early history turns in place if doing so breaks provider prompt caching. Use `headroom-core`'s `compute_frozen_count` + `live_zone` byte-range surgery!
3. Universal Mandatory `HeadroomEffortCompressor` Pass: Zero `/byok/`, gateway (`:9099`), BYOK/Vertex ADC (`:18787`), or Protocol Bus (`:9098`) compression exemptions — EVERY inference/protocol entrypoint invokes the shared `HeadroomEffortCompressor` (`Low`/`Medium`/`High`: `live_zone`, `CcrStore`, `SmartCrusher`, `TokenEconomizer`, `agent-opt`, `mcp_pruner`, `rtk`, `chroma`/`lance` semantic cache).
4. Configured-Only Keyring & Multi-Instance BYOK: Users can configure multiple `<byok, model, effort>` rows under the same BYOK provider (separate models, separate endpoints, separate quotas) via CLI, TUI, Web Cockpit, or `/byokmodel`.

Requirements:
1. SQLite Multi-Instance BYOK Schema (`db/schema.rs` & `operations.rs` & `migrate.rs`):
   - Add `byok_profiles` table (`UNIQUE(provider_id, instance_label, model_id, thinking_effort)` with `base_url`, `auth_mode`, `headroom_enabled`, `priority`, `weight`).
   - Add `agent_profile_bindings` (`agent_id`, `primary_profile`, `fallback_chain`, `wire_mode`, `routing_algo`), `quota_windows` (`hour|day|week|month|year`), and `plugins_registry` (`plugin_id`, `version`, `manifest_path`, `blake3_hash`, `capabilities_json`, `protocols_json`, `security_scan_status`, `is_enabled`, `updated_at`).
   - Fix `export_dump()` to export `provider_models`, `byok_profiles`, `gateway_configs`, `agent_profile_bindings`, `quota_windows`, and `plugins_registry`.
   - Fix `import_dump(..., overwrite: bool)` so `overwrite == true` cleanly resets & re-imports without stale row collisions.
   - Add `toksi byok add|list|update|delete|set-url|set-model` CLI CRUD subcommands.
2. Fix OS Keyring Alias Normalization (`keyring/mod.rs:216`):
   - Fix broken `("antigravity", "gemini", "antigravity")` lookup so `gemini:antigravity` alias resolves cleanly across Windows Credential Manager / macOS Keychain / Linux Secret Service.
   - Restrict Keyring discovery displays strictly to user-configured agents & BYOK providers.
3. Port Pure-Rust `headroom-core` & Retire Python Spawns:
   - Delete Python `headroom` process spawning (`headroom/proxy.rs:1271`, `tools/headroom.rs:64`) and remove dead `cloudcode_patch.rs`.
   - Port `compute_frozen_count` + `live_zone` byte-range surgery (`headroom/proxy.rs:785-827`), `CcrStore` (`<<ccr:HASH>>` SQLite store + MCP `headroom_{compress,retrieve,stats,read}`), `SmartCrusher`, `SearchCompressor`, `DiffCompressor`, `AuthMode::{Payg, OAuth, Subscription}`, and Gemini `contents[]` compression.
4. Remove `/byok/` Headroom Bypass & Wire Live Chroma ANN Semantic Cache:
   - Remove early return for `/byok/` in `headroom/proxy.rs:619-631` so all configured BYOK providers go through Headroom compression and telemetry accounting.
   - Upgrade `gateway/semantic_cache.rs` to Chroma HNSW/USearch/SPANN ANN + `foyer` disk spill and wire live `store()` / `lookup_semantic()` calls inside `headroom/proxy.rs`.
5. Verification:
   - Run `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted `cargo test -p toksi`.
```

---

## Technical Specifications & Details

### 1. Multi-Instance `<byok, model, effort>` DDL
See `RESEARCHPLAN.md` Section 13.4 for exact `byok_profiles`, `agent_profile_bindings`, and `quota_windows` schemas.

### 2. Verification Criteria
- Multiple profiles under one provider (e.g. `google:flash-low` and `google:pro-high`) coexist cleanly in `~/.toksi/toksi.db` and survive `toksi db export` -> wipe -> `toksi db import --overwrite`.
- `/byok/` requests measurably reduce uncompressed tail tokens while keeping `frozen_prefix` byte-identical across turns.
- Zero Python `headroom` subprocess calls anywhere in `crates/toksi/src/`.

---

## Wave 18 (v5.0) Synchronization Addendum
- Extend `byok_profiles` table schema with `auth_mode`, `gcp_project`, and `gcp_location` (`us-central1` default) for `AGY_ADC_AUTH=true` Vertex routing (`PHASE_18_3`), and persist Unified Plugin descriptors/scan verdicts in `plugins_registry` (`PHASE_18_2`).
- Wire `Low` / `Medium` / `High` thinking-effort aggressiveness into `headroom-core` `live_zone` compression passes and cross-provider reasoning budget translation (`reasoning_effort`, `thinking.budget_tokens`, `generationConfig.thinkingConfig.thinkingBudget`).
