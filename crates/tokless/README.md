# tokless

[![Crates.io](https://img.shields.io/crates/v/tokless.svg)](https://crates.io/crates/tokless)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-brightgreen.svg)]()

**tokless** is a high-performance headroom proxy, token compression engine, and agent wire integration platform for AI coding assistants.

Designed for modern AI development environments, `tokless` transparently sits between local AI coding agents and upstream model backends or MCP servers, reducing token consumption by 30-70% while improving response latency and preventing context saturation.

---

## Features

- **Transparent Streaming Headroom Proxy**:
  - High-throughput asynchronous HTTP/SSE streaming proxy server.
  - Automatically intercepts agent requests and responses with zero user latency overhead.
  - Real-time token metering, rate limiting, and quota enforcement.
- **BYOK (Bring Your Own Key) Gateway**:
  - Centralized local credential management for model providers (Anthropic, OpenAI, Google Gemini, OpenRouter, Groq).
  - Securely injects API keys at the proxy boundary, keeping secrets out of local project transcripts and dotfiles.
- **Support for 12 AI Coding Agents**:
  - **Antigravity** (Google AGY CLI & IDE)
  - **Claude Code** (Anthropic CLI)
  - **Cline** (VS Code / JetBrains extension)
  - **Codex** (OpenAI Codex CLI)
  - **GitHub Copilot** (VS Code / Visual Studio)
  - **Cursor** (AI Code Editor)
  - **Droid** (Factory Droid agent)
  - **Grok** (xAI Grok agent)
  - **Kilo** (Kilo code assistant)
  - **OMP** (Open Multi-Agent Platform)
  - **OpenCode** (Open-source agent runner)
  - **Pi** (Pi agent rust harness)
- **Token Compression & Filtering Suite**:
  - **RTK (Runtime Token Kernel)**: Condenses CLI command output, strips redundant build noise, pagers, and whitespace while preserving essential error lines and exit codes.
  - **Caveman Compressor**: Compresses system prompts and instructions into terse ASD-STE100 Simplified Technical English, eliminating grammatical fluff while preserving 100% technical tokens, code blocks, and semantics.
  - **Ponytail Discipline**: Injects strict build guidelines enforcing YAGNI, surgical edits, and minimal runnable checks.
  - **CodeGraph**: Out-of-context repository indexing, symbol navigation, and call-graph blast radius computation via MCP.
  - **ContextMode**: Sandbox code execution (`ctx_execute`, `ctx_batch_execute`, `ctx_search`) to keep large analysis out of the LLM context window.
  - **ProjectMem**: Structured cross-session memory in `.projectmem/` tracking issues, attempts, fixes, and architectural decisions.
- **Agent Health Doctor & Autostart**:
  - Diagnoses agent configurations, detects hook status, validates proxy connectivity, and self-heals broken routes.

---

## Installation

### From Crates.io
```bash
cargo install tokless
```

### From Source
```bash
git clone https://github.com/Navegos/neo.git
cd neo
cargo build --release -p tokless
```

---

## Quick Start & CLI Usage

### Initialize Agent Hooks
Automatically detect installed coding agents and inject proxy routing hooks:
```bash
tokless init
```

### Run Health Doctor
Verify proxy connectivity, agent hooks, and token savings metrics:
```bash
tokless doctor
```

### Start Headroom Proxy
Launch the local proxy server:
```bash
# Start in foreground
tokless proxy

# Start in background with custom port
tokless proxy --port 8080 --daemon
```

### Wrap MCP Server with Headroom Optimization
```bash
tokless runmcp -- command arg1 arg2
```

### View Proxy & Token Statistics
```bash
tokless info
```

### Disable Agent Hooks
Revert agent configurations to their native unproxied states:
```bash
tokless disable
```

---

## Architecture

```
[ AI Coding Agent ] (Claude, Copilot, Cursor, Antigravity, ...)
         │
         ▼  (Local HTTP/SSE)
┌──────────────────────────────────────────────┐
│                  tokless                     │
│  ┌────────────────────────────────────────┐  │
│  │ Headroom Proxy & Router                │  │
│  └────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────┐  │
│  │ Token Compression & Instruction Filter │  │
│  │ (RTK / Caveman / Ponytail)             │  │
│  └────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────┐  │
│  │ BYOK Secure Gateway                    │  │
│  └────────────────────────────────────────┘  │
└──────────────────────────────────────────────┘
         │
         ▼  (HTTPS)
[ Upstream LLM / Provider API ]
```

---

## License

Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

