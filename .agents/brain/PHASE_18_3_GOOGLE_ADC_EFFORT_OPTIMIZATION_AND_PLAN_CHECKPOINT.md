# Phase 18.3 — Google Cloud ADC (`AGY_ADC_AUTH=true`), Thinking-Effort Optimization & Safe Plan Checkpoints

## Ready-to-Run Chat Prompt

```text
Implement Phase 18.3 of the Toksi Unified Consolidation Plan (Google Cloud ADC Gateway, Thinking-Effort-Aware Optimization, and Safe Plan Checkpoint/Resume).
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 1–3, 4.3, 12–13), .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, and .agents/brain/PHASE_18_3_GOOGLE_ADC_EFFORT_OPTIMIZATION_AND_PLAN_CHECKPOINT.md first.

Your goals in Rust (`crates/toksi`):
1. Google Cloud ADC (`AGY_ADC_AUTH=true`) First-Class Gateway & Auth (`src/keyring/adc.rs`, `src/db/byok.rs`, `src/gateway/vertex_adc.rs`):
   - Extend `byok_profiles` SQLite schema with `auth_mode`, `gcp_project`, and `gcp_location` (default `us-central1`).
   - Support reading VS Code `"gcp.project"` / `"gcp.location"`, `~/.toksi/config.toml`, `GOOGLE_CLOUD_PROJECT`, `GOOGLE_CLOUD_LOCATION`, and OS environment overrides (`AGY_ADC_AUTH=true`).
   - Locate Application Default Credentials automatically at `~/.config/gcloud/application_default_credentials.json` (Unix) or `%APPDATA%/gcloud/application_default_credentials.json` (Windows), or `GOOGLE_APPLICATION_CREDENTIALS` service-account JSON.
   - Perform native pure-Rust OAuth2 refresh-token / service-account JWT exchange via `https://oauth2.googleapis.com/token` (cached cleanly in-memory with 60s expiry skew; zero shelling out per request) and sign upstream Vertex AI calls to `https://{location}-aiplatform.googleapis.com/v1beta1/projects/{project}/locations/{location}/publishers/google/models/{model}:streamGenerateContent`.
   - When ADC credentials or project ID are absent while `AGY_ADC_AUTH=true`, emit clear actionable guidance: `gcloud auth application-default login --project <GCP_PROJECT>` plus `toksi auth adc --project <id> --location us-central1`.
2. Thinking-Effort-Aware Token Optimization (`Low` / `Medium` / `High`) (`src/headroom/effort.rs`, `src/gateway/effort_router.rs`, `/effort`):
   - Parse requested effort tier from: CLI flag `--effort low|medium|high`, slash command `/effort [low|medium|high]`, request header `x-toksi-effort`, or JSON body fields (`reasoning_effort`, `thinking.budget_tokens`, `generationConfig.thinkingConfig.thinkingBudget`).
   - Dynamically translate effort tiers across upstream providers:
     * OpenAI / Codex / Copilot / DeepSeek / Grok -> `reasoning_effort: "low" | "medium" | "high"`.
     * Anthropic -> `thinking: { type: "enabled", budget_tokens: 1024 | 8192 | 32768 }` (auto-omitted for non-thinking models).
     * Gemini / Vertex ADC -> `generationConfig.thinkingConfig.thinkingBudget` (e.g. `1024 | 8192 | 24576` or model-specific min/max clamp; silently stripped if model does not support thinking).
   - Tune Toksi compression aggressiveness by effort tier:
     * `Low`: fast heuristic pruning, RTK max compaction, top-K AST slices, zero multi-hop SCIP expansion.
     * `Medium`: balanced Headroom semantic compression + AST body pruning + ICM summary cache.
     * `High`: lossless reasoning/system prompt preservation, deep CodeGraph SCIP + blast-radius context, zero truncation of user constraints or error backtraces.
   - Surface `/effort` switching (`< 5ms`) and expose live effort metrics in Telemetry & `tokview`.
3. Safe Plan Checkpoint & Crash-Proof Resume (`src/projectmem/plan_checkpoint.rs`, `src/agent_runtime/checkpoint.rs`, `/resume`):
   - Persist atomic plan execution state (`write tmp -> fsync -> rename`) inside `<repo>/.toksi/projectmem/plan_checkpoints/<plan_id>.json` paired with append-only `events.jsonl` and checkbox sync in `<repo>/.toksi/projectmem/plan.md`.
   - Record step ID, status (`pending | running | completed | paused | failed`), files touched (with SHA-256 pre/post digest), verified test baseline delta, active effort/model profile, active enabled plugin snapshot (`active_plugins` synced with `plugins_registry`), and handoff summary before yielding on milestones, rate-limit `429` exhaustion, network drop, `SIGINT`, or crash.
   - Implement `/resume` slash command (one of **13** universal slash commands alongside `/security` and `/plugin`) and `toksi agent resume [<plan_id>]` CLI + `:9098` ACP/MCP notification to validate dirty working tree diffs, hydrate ProjectMem context, re-verify enabled plugins under `<enforce>`, and restart from the exact incomplete step without re-running completed work.
4. Update `.toksi/test-baseline.json` and run `cargo test --test phase18_3_adc_effort_checkpoint_verification`.
```

---

## Technical Specifications & Details

### 1. Google Cloud ADC (`AGY_ADC_AUTH=true`) Resolution Order
1. Explicit CLI/env overrides: `AGY_ADC_AUTH=true`, `TOKSI_GCP_PROJECT` / `GOOGLE_CLOUD_PROJECT`, `TOKSI_GCP_LOCATION` / `GOOGLE_CLOUD_LOCATION`.
2. `~/.toksi/config.toml` `[auth.adc]` section (`enabled`, `project`, `location`, `credentials_path`) and SQLite `byok_profiles` (`auth_mode = 'adc'`, `gcp_project`, `gcp_location`).
3. IDE settings bridge (VS Code `settings.json` keys `"gcp.project"`, `"gcp.location"`, `"agy.adcAuth"`).
4. Credential file discovery:
   - `GOOGLE_APPLICATION_CREDENTIALS` env path (if set).
   - Windows: `%APPDATA%/gcloud/application_default_credentials.json`.
   - Linux/macOS: `$XDG_CONFIG_HOME/gcloud/application_default_credentials.json` or `~/.config/gcloud/application_default_credentials.json`.
5. Credential types supported natively:
   - `authorized_user` (`client_id`, `client_secret`, `refresh_token`) -> POST `https://oauth2.googleapis.com/token`.
   - `service_account` (`client_email`, `private_key`) -> RS256 JWT assertion -> POST `https://oauth2.googleapis.com/token`.

### 2. Effort-to-Compression & Wire Mapping Matrix
| Tier | Headroom Policy | Context Budget | OpenAI `reasoning_effort` | Anthropic `budget_tokens` | Gemini `thinkingBudget` |
|---|---|---|---|---|---|
| `Low` | Aggressive RTK + Headroom prune | Minimal AST signatures | `"low"` | `1024` (or omitted) | `1024` (or `0` when allowed) |
| `Medium` | Balanced semantic + AST prune | Targeted call slices | `"medium"` | `8192` | `8192` |
| `High` | Lossless reasoning & constraints | Deep SCIP + blast radius | `"high"` | `32768` | `24576` (clamped per model) |

### 3. Plan Checkpoint Schema (`<repo>/.toksi/projectmem/plan_checkpoints/<plan_id>.json`)
```json
{
  "schema_version": 1,
  "plan_id": "wave18-sync-2026-11-14",
  "repo_root": "d:/AI/neo",
  "status": "paused",
  "current_step": 4,
  "active_effort": "High",
  "active_model": "gemini-2.5-pro",
  "active_plugins": ["plugin-id@1.0.0"],
  "steps": [
    {
      "id": 1,
      "title": "Repair stale .projectmem/ pre-reads",
      "status": "completed",
      "verified_by": "regex audit exit 0",
      "completed_at": "2026-11-14T23:05:00Z"
    }
  ],
  "dirty_files_sha256": {},
  "last_error": null
}
```
