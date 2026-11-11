# Phase 17.2: Multi-Model `<byok, model, effort>` DB, Keyring Fix, `headroom-core` `live_zone`, & Mandatory BYOK Headroom Pass

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 13.3–13.4), .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_17_2_MULTI_MODEL_DB_KEYRING_AND_HEADROOM_PASS.md first.

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
3. Mandatory BYOK Headroom Pass: Zero `/byok/` compression exemptions — EVERY configured BYOK request flows through Headroom + enabled tools (`mcp_pruner`, `SmartCrusher`, `CcrStore`, `TokenEconomizer`, `chroma` semantic cache).
4. Configured-Only Keyring & Multi-Instance BYOK: Users can configure multiple `<byok, model, effort>` rows under the same BYOK provider (separate models, separate endpoints, separate quotas) via CLI, TUI, Web Cockpit, or `/byokmodel`.

Requirements:
1. SQLite Multi-Instance BYOK Schema (`db/schema.rs` & `operations.rs` & `migrate.rs`):
   - Add `byok_profiles` table (`UNIQUE(provider_id, instance_label, model_id, thinking_effort)` with `base_url`, `auth_mode`, `headroom_enabled`, `priority`, `weight`).
   - Add `agent_profile_bindings` (`agent_id`, `primary_profile`, `fallback_chain`, `wire_mode`, `routing_algo`) and `quota_windows` (`hour|day|week|month|year`).
   - Fix `export_dump()` to export `provider_models`, `byok_profiles`, `gateway_configs`, `agent_profile_bindings`, and `quota_windows`.
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
