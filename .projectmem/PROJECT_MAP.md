# Project Map - neo

Status: auto-detected from project manifests (an AI session may refine this).

## Project purpose
--------------------------------------------------------------------------------
Unified tooling and headroom proxy pipeline for AI coding agents, ported from Go to Rust.

## Stack
- Tags: rust
- Detected from: Cargo.toml

## Structure
- `crates/tokless`: Rust implementation of tokless CLI, headroom proxy, agent wiring, and MCP integration
- `crates/nbuild`: Universal toolchain manager, local GitHub Actions workflow runner, and package engine for AI agents
- `crates/archinfo`: Hardware CPU feature detection, ISA mapping, and target triple platform analysis
- `inspect-vault/tokless`: Reference Go implementation
- `inspect-vault/buildtools`: Reference PowerShell and early buildtools implementation
