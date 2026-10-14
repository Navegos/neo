# projectmem - neo

_Last updated: 2026-10-14_

## Project purpose
--------------------------------------------------------------------------------
Unified tooling and headroom proxy pipeline for AI coding agents, ported from Go to Rust.

## Recent issues
- No issues logged yet.

## Decisions
- Port tokless Go codebase from inspect-vault/tokless to Rust in crates/tokless [crates/tokless]
- Go tokless ported to Rust in crates/tokless with zero compiler warnings and 17 unit tests passing [crates/tokless]

## Notes
- start of Neo
- New feature: feat: add routing store and ownership management [crates/tokless/src/agents/hooks.rs]
- Completed 100% Go-to-Rust conversion of inspect-vault/tokless into crates/tokless across util, headroom, agents, tools, and commands (zero compiler warnings, 17/17 unit tests passing). [crates/tokless/src]
- New feature: Add SPDX license headers and metadata to source files [crates/archinfo/build.rs]

## Key files
- `.editorconfig`
- `.gitattributes`
- `.gitignore`
- `LICENSE`
- `README.md`
- `crates/tokless/src/agents/hooks.rs`
- `Cargo.toml`
- `crates/.gitkeep`
- `crates/tokless/Cargo.toml`
- `crates/tokless/src/agents/mod.rs`
- `crates/tokless/src/agents/proxy_agent.rs`
- `crates/tokless/src/agents/proxy_protocol.rs`
- `crates/tokless/src/commands/codex_perm_hook.rs`
- `crates/archinfo/build.rs`
- `crates/archinfo/.codegraph/.gitignore`
- `crates/archinfo/.gitattributes`
- `crates/archinfo/.github/workflows/release.yml`
- `crates/archinfo/.gitignore`
- `crates/archinfo/Cargo.lock`
- `crates/archinfo/Cargo.toml`

## Open questions
- None logged yet.
