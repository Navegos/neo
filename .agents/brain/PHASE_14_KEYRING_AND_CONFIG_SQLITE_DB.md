# Phase 14: OS Keyring Integration, Config SQLite DB & BYOK Gateway Pre-Registration

## Ready-to-Run Chat Prompt
```text
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, .projectmem/plan.md, .projectmem/PROJECT_MAP.md, .projectmem/summary.md, .agents/brain/PHASE_14_KEYRING_AND_CONFIG_SQLITE_DB.md, first to understand the global master architecture, cross-tool coupling, directory topology, features, etc.

Execute Phase 14: OS Keyring Integration, Config SQLite DB & BYOK Gateway Pre-Registration.
Targets:
  - crates/toksi/src/keyring/
  - crates/toksi/src/db/
  - crates/toksi/src/commands/auth.rs
  - crates/toksi/src/commands/db.rs

Disciplinary & Portability Requirements:
1. Safe Rust (Edition 2024): Safe Rust enforced crate-wide. Unsafe code permitted exclusively in isolated Win32 Credential Manager FFI calls (`os_windows.rs`: advapi32.dll) accompanied by explicit `// SAFETY:` rationale.
2. Zero Warnings Policy: Compile cleanly under `RUSTFLAGS="-D warnings"`.
3. Zero Panics Policy: Strictly prohibit .unwrap(), .expect(), panic!(). Use thiserror for domain errors.
4. Database Storage Mandate: Store `toksi.db` by default in user home (`~/.toksi/toksi.db`), NOT in `localdata/toksi` (`AppData/Local/toksi`).
5. Migration Invariant: Auto-migrate existing configurations from legacy `LOCALAPPDATA/toksi/toksi.db` and JSON registries on startup.
6. Zeroize Memory: In-memory credential buffers must implement `Zeroize` to scrub secrets from RAM on drop.

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
4. Expose CLI commands `toksi auth ...` and `toksi db ...`.
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
