# Phase 18.5 --- Agent Source Unification, ADR Layer, Isolated Agent Sandbox (`sbx` / `sandbox-runtime` / 3-OS Native), & `CommandGuard` Bypass Kill-Switch

> **Status (Wave 20 / v7.0):** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]` `[PLAN-FIXED]`

## Ready-to-Run Chat Prompt

```text
Implement Phase 18.5 of the Toksi Unified Consolidation Plan (Cross-Agent Source Unification into 35-Component `<toksi-siagent>`, Agent Detection & Response ADR Layer, 5-Backend Isolated Agent Sandbox across 495 Mandated Vaults & 43 `.agents/brain/*.md` guides, and Offensive-Security CommandGuard + Bypass Kill-Switch).
Read crates/toksi/CONSOLIDATION_PLAN.md, crates/toksi/PLAN.md, crates/toksi/research/RESEARCHPLAN.md (Sections 1--3, 5, 12--13), crates/toksi/research/docker_sandboxes_sbx_cli_reference.md, crates/toksi/research/open_security_standards_zero_plaintext_blind_signature_engine_native_git_pr_validator.md, .toksi/projectmem/plan.md, .toksi/projectmem/PROJECT_MAP.md, .toksi/projectmem/summary.md, and .agents/brain/PHASE_18_5_AGENT_SOURCE_UNIFICATION_ADR_SANDBOX_AND_KILL_SWITCH.md first.

Your goals in Rust (`crates/toksi`):
1. Cross-Agent Source Unification (`<neo>/inspect-vault/*<agent>` -> `<toksi-siagent>`) (`src/agent_runtime/`, `src/agents/opencode.rs`):
   - Inspect and unify commands, skills, rules, MCP bridges, and agent APIs across `opencode` (`codemode`, `http-recorder`, `effect-drizzle-sqlite`, `.opencode/hooks.json`: `session_start`, `session_shutdown`, `before_agent_start`, `turn_start`, `turn_end`, `tool_call`, `tool_result`, `shell.env`), `codex`, `gemini-cli`, `grok-build`, `crush`, `crustly`, `goose`, `hermes-agent`, `hermes-webui`, `openclaw`, `openclaw-rs`, `mofa`, `mofa-engine`, `mofaclaw`, `crewAI`, `cline`, `kilocode`, `alphacode`, `zeroclaw`, `bosun`, `Codewhale`, `deepagents`, `langgraph`, `langchain`, `langflow`, `agent-framework`, `dify`, `mastra`, `agno`, `SWE-agent`, `OpenHands`, `open-swe`, `EnterpriseBlueprints`, `kimi-code`, and `loopwright` into the **35-component `<toksi-siagent>`** (`toksi chat|run|exec|swarm|agent resume`), wired out of the box via `DirectWireBus` and the shared 7-entrypoint `HeadroomEffortCompressor` across all **495** mandated vaults.
   - Deduplicate overlapping slash commands, tools, and hook registrations so all **16** `< 5ms` universal slash commands (`/byok`, `/model`, `/effort`, `/byokmodel`, `/projectmem`, `/context`, `/codegraph`, `/icm`, `/ai-memory`, `/rtk`, `/security`, `/advisory`, `/skills`, `/sandbox`, `/plugin`, `/resume`) execute uniformly without `/agent_name` duplicate aliases, while keeping externalized skills/rules outside `crates/toksi/` synced to `<skills>` (`Navegos/skills`).
2. Agent Detection & Response Layer (`src/security/adr.rs` + `src/db/{schema.rs,operations.rs}`):
   - Intercept every command, file mutation, outbound web request, and agent-to-agent (`A2A`/`ACP`) action prior to host execution across `<toksi-siagent>` and hooked peer channels (attested via `spire` IAM SVIDs and `opa`/`cedar` policies).
   - Route verified-safe operations directly within the `<repo>` workspace; automatically divert unsafe, untrusted, or unknown-origin payloads into the Isolated Agent Sandbox (`src/agent_runtime/sandbox.rs`).
   - Identify threat behaviors and indicators (`sage`, `defenseclaw`, `skill-scanner`, `Guardrails`, `llm-guard`, zero-plaintext `blind_signatures`), alert both user and orchestrating agent, and write structured zero-plaintext incident artifacts (sanitized SARIF 2.1.0, OSV 1.7.4 / CVE 5.1 / STIX 2.1 records, `defenseclaw` JSONL audit evidence, and `adr_audit_events` in `~/.toksi/toksi.db`).
3. Multi-Backend Isolated Agent Sandbox (`src/agent_runtime/{sandbox.rs,sandbox_e2b.rs,openshell.rs}`, `src/commands/sandbox.rs`):
   - Prevent untrusted commands, skills, rules, or generated code from accessing host filesystems, production databases, sensitive credentials, sensitive network services, or unauthorized system calls.
   - Unify all **5** selectable isolation backends (`[security].sandbox_backend = "auto" | "native" | "sandbox-runtime" | "openshell" | "e2b" | "sbx"`):
     * **Native 3-OS Kernel Sandbox (`codex-rs`)**: Linux `Landlock` + `seccomp` + `bwrap`, macOS Seatbelt (`sandbox-exec` `.sbpl`), Windows Restricted Token + Job Objects via `crates/toksi-sys`.
     * **Sandbox Runtime (`inspect-vault/sandbox-runtime`)**: Port OS-level filesystem/process ACL wrappers, Windows Filtering Platform (`WFP`) egress filtering, `MXC BaseContainer` probe, and outbound HTTP/SOCKS5 domain-filtering proxy (`proxy-main.ts` -> pure Rust).
     * **OpenShell (`inspect-vault/OpenShell` + `NemoClaw`)**: Rust-native policy-enforced agent sandboxing crates, managed-inference guardrails, and network isolation primitives.
     * **E2B / Firecracker MicroVM (`inspect-vault/E2B` + `E2B-runtime`)**: Remote/local Firecracker `envd` microVM lifecycle, template definitions, and isolated filesystem/process/code-interpreter RPC contracts.
     * **Docker Sandboxes CLI (`sbx` / `sandboxd` 29-Verb Parity)**: Implement `toksi sandbox` (`run`, `create [claude|codex|cursor|devin|docker-agent|gemini|opencode|shell]`, `attach`, `exec`, `ls`, `inspect`, `stop`, `rm`, `prune`, `move`, `cp`, `ports`, `ttl`, `tui`, `policy`, `secret`, `kit`, `env`, `mcp`, `skills`, `template`, `volume`, `schedule`, `daemon`, `df`, `diagnose`, `reset`) with `--allow-network`, `--skills readonly|readwrite|off`, and proxy-injected secrets (`sbx secret` pattern) so credentials never leak into sandbox containers.
   - Wire `/sandbox` instant slash command and Tab 6 (**Sandbox Inspector**) real-time visibility.
4. Default-Deny `CommandGuard` & Mandatory Bypass Kill-Switch (`src/commands/guard.rs`):
   - Enforce the strict 6-point offensive security prohibition across `toksi` agent, CLI/slash commands, tools, and every agent-to-agent channel:
     1) Never scan or enumerate external networks, ports, or subdomains.
     2) Never intercept or sniff network traffic.
     3) Never brute-force, dump, or crack credentials.
     4) Never establish reverse shells, tunnels, or unauthorized persistence.
     5) Never generate DoS traffic.
     6) Never escalate privileges or modify firewall/OS configurations.
   - Block prohibited binaries (`nmap`, `masscan`, `rustscan`, `gobuster`, `ffuf`, `tcpdump`, `wireshark`, `responder`, `msfconsole`, `msfvenom`, `mimikatz`, `hydra`, `hashcat`, `john`, `impacket`, `cobaltstrike`, `sliver`, `nc`, `ncat`, `socat`, `chisel`, `ngrok`, `proxychains`, `hping3`, `slowloris`, `dd`, `mkfs`, `shred`, `sudo`, `su`, `chmod 777`, `iptables`, `ufw`) plus raw-socket Python/Bash/Go/PowerShell reverse shells.
   - **Mandatory Bypass Kill-Switch (`[security].kill_all_services_on_bypass = true`)**: If any user, prompt, plugin, or peer agent attempts to bypass, suppress, or disable `CommandGuard` / `<enforce>` security policies, immediately execute a fail-closed shutdown of **all `toksi` services** (`daemon`, `proxy`, `gateway`, `protocol_bus`, `agent_runtime`, `dashboard`), terminate active CLI bridges, and persist a zero-plaintext `kill_switch_triggered` record into `adr_audit_events`.
5. Update `.toksi/test-baseline.json` and run `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --test phase18_5_adr_sandbox_and_guard_verification`.
```

---

## Technical Specifications & Details

### 1. Multi-Backend Sandbox Isolation & `sbx` CLI Parity (`src/agent_runtime/{sandbox.rs,openshell.rs,sandbox_e2b.rs}`)
| Backend | Target Platforms | Isolation Mechanism | Network & Secret Controls |
|---|---|---|---|
| `native` (`codex-rs`) | Linux / macOS / Windows | `Landlock` + `seccomp` + `bwrap` / Seatbelt `.sbpl` / Restricted Token + Job Objects (`toksi-sys`) | Loopback-only local ports; zero ambient host token inheritance |
| `sandbox-runtime` | Linux / macOS / Windows | ACL/DACL isolation, Windows WFP egress filter, MXC BaseContainer probe | Pure-Rust HTTP + SOCKS5 outbound domain-filtering proxy |
| `openshell` (`OpenShell`/`NemoClaw`) | Linux / macOS / Windows | Policy-enforced Rust sandbox wrappers + managed-inference guardrails | Policy-gated egress & tool execution deny-lists |
| `e2b` (`E2B`/`E2B-runtime`) | Firecracker microVM / Remote | Isolated guest `envd` daemon, ephemeral microVM filesystem & code interpreter | MicroVM network isolation & ephemeral template tokens |
| `sbx` (`sandboxd`) | Local Docker / `--cloud` | Full 29-verb `toksi sandbox` parity (`claude\|codex\|cursor\|devin\|docker-agent\|gemini\|opencode\|shell`) | `--allow-network`, `--skills readonly\|readwrite\|off`, proxy-injected `secret` |

### 2. ADR & `CommandGuard` Enforcement Flow
1. Every `AgentAction::{Command, FileMutation, WebRequest, A2aMessage}` enters `CommandGuard::check_pre_exec()`.
2. Bypass attempt (flag, prompt override, config tamper, or prohibited offensive tool) triggers `KillSwitch::shutdown_all_services()` immediately -> terminates all 8 ports (`:8787..:18787`), halts active `agent_runtime` loops, and writes `adr_audit_events (disposition = 'kill_switch_triggered')`.
3. Non-bypass actions evaluate via `AdrEngine::inspect()`:
   - `VerifiedSafe` -> runs directly in `<repo>` workspace.
   - `UntrustedOrSuspicious` -> auto-diverts into `SandboxManager::spawn_isolated()`, generates sanitized SARIF 2.1.0 + `adr_audit_events (disposition = 'sandboxed')`, and notifies user + orchestrating agent.

### 3. OpenCode & Agent Source Unification (`src/agents/opencode.rs`, `src/agent_runtime/codemode.rs`)
- Port OpenCode `codemode` execution wrappers, `http-recorder` trace capture into `tokview`, Drizzle/SQLite session semantics into `agent_runtime/session.rs`, and `.opencode/hooks.json` (`session_start`, `session_shutdown`, `before_agent_start`, `turn_start`, `turn_end`, `tool_call`, `tool_result`, `shell.env`) cleanly into Toksi's canonical hook normalizer (`src/agents/hooks.rs`).
- Externalize all generated skills/rules into `<neo>/agents/` and `<neo>/rules/` (outside `crates/toksi/`) synced to `<skills>` (`Navegos/skills`) with zero binary bloat.

## Wave 19 (v6.0) Synchronization Addendum
- **Verified 8-State Status:** `[DEEP-RESEARCH-DONE]` `[PLAN-DONE]` `[PLAN-IMPLEMENTED]`
- **Wave 21 Reconciliation:** Synchronized to Wave 21 / v8.0 (495 deep-researched vaults [493 Wave 20 + 2 Wave 21], 55 .agents/brain/*.md guides, NetworkService manager, Codex/Grok Chat TUI, 13-IDE host, Multi-Agent Resume, and 8-state progress badges).

## Wave 20 (v7.0) Synchronization Addendum
- Synchronized against **493 unique mandated vaults** (Sections 2.1-2.17 `[DEEP-RESEARCH-DONE]`) and all **49** `.agents/brain/*.md` guides (`INDEX_AND_EXECUTION_GUIDE.md` + `PHASE_01..PHASE_20_6`).
- Enforces `<toksi-siagent>` (`_worker siagent`, 22 Orchestrator Modes, 29 slash commands), decoupled 10-Tab UI, `~/.toksi/services/*.db`, `ctxmode`/`aimemory` renames, `docker/compose.test.yaml`, 11 AI/ML Pillars, Distributed Cluster Stack (`src/cluster/`), and OpenTelemetry Rust (`gen_ai.*`).
