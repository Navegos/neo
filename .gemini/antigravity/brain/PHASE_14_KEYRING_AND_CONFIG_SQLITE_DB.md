# Phase 14: OS Keyring Integration, Config SQLite DB & BYOK Gateway Pre-Registration

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .gemini/antigravity/brain/PHASE_14_KEYRING_AND_CONFIG_SQLITE_DB.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 14: OS Keyring Integration, Config SQLite DB & BYOK Gateway Pre-Registration.
Targets:
  - crates/toksi/src/keyring/{mod.rs, os_windows.rs, os_macos.rs, os_linux.rs, envelope.rs}
  - crates/toksi/src/db/{mod.rs, schema.rs, operations.rs, migrate.rs}
  - crates/toksi/src/gateway/agent_cli/{gemini_agy.rs, claude_code.rs, copilot.rs, cursor.rs, mod.rs}
  - crates/toksi/src/headroom/{byok_gateway.rs, proxy.rs}
  - crates/toksi/src/commands/{auth.rs, db.rs, doctor_probe.rs}

Requirements:
1. Native Cross-Platform OS Keyring Manager:
   - Windows: Query and store via native advapi32.dll APIs (CredReadW, CredWriteW, CredDeleteW, CredFree).
   - macOS: Delegate to system Keychain via /usr/bin/security CLI.
   - Linux: Delegate to Secret Service API backend (libsecret / secret-tool).
   - Base64 OAuth Envelope Parser: Decode base64 envelopes, extract OAuth access_token and expiration, validate freshness.
2. Persistent SQLite Configuration Database (~/.toksi/toksi.db):
   - Replace flat JSON files with high-performance SQLite database running in WAL mode.
   - Tables: providers, provider_keys, provider_models, byok_routes, agent_configs, gateway_configs.
   - Support zero-downtime operations: export, import, backup, restore, sync.
3. BYOK Gateway Pre-Registration Pipeline:
   - Enforce route validation and commitment to SQLite byok_routes before traffic dispatch.
   - Inject authenticated headers (Authorization: Bearer <token>, x-goog-api-client, x-api-key) into requests when local agents omit credentials.
4. CLI Subcommands & Diagnostics:
   - toksi auth [status|get|set|delete|sync]
   - toksi db [status|export|import|backup|restore]
   - toksi doctor keyring and config DB probes.
5. Verification:
   - Ensure 0 compiler warnings and passing tests.
```

