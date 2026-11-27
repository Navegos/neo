
# Open Security Standards, Zero-Plaintext Blind Signature Engine & Native Git/PR Validator (`Navegos/neo` + `Navegos/advisory`)

## 1. Exact Two-Repository & Module Architecture

### Repository 1: `Navegos/neo` (`https://github.com/Navegos/neo.git` --- Rust Workspace)
Respects `neo/crates/toksi/src/db/` strictly as the internal SQLite layer (`~/.toksi/toksi.db`) and leverages existing `keyring/`, `codegraph/`, `env_injector/`, `gateway/`, `telemetry/`, `tools/`, `commands/`, `agents/`, `core/`, and `rules/` modules without rewriting core runtime loops.

```text
neo/                                                    # [https://github.com/Navegos/neo.git](https://github.com/Navegos/neo.git)
|-- Cargo.toml
|-- crates/
|   `-- toksi/
|       `-- src/
|           |-- agents/                                 # Existing agent runtime + SARIF provenance hooks
|           |-- aimemory/                               # Existing AI memory
|           |-- codegraph/                              # Existing 38 Tree-sitter parsers (reused by security::polyglot)
|           |-- commands/                               # Existing CLI/slash commands + pre-execution CommandGuard (guard.rs)
|           |-- contextmode/                            # Existing context pruning
|           |-- core/                                   # Existing core primitives & traits
|           |-- dashboard/                              # Existing web UI dashboard
|           |-- db/                                     # Strictly internal SQLite (~/.toksi/toksi.db) + blind signature/embargo tables
|           |-- env_injector/                           # Existing env sanitization & secret injection
|           |-- gateway/                                # Existing LLM/MCP/A2A/HTTP gateway
|           |-- headroom/                               # Existing headroom optimizer
|           |-- icm/                                    # Existing ICM module
|           |-- keyring/                                # Existing OS keyring (used for GitHub PR, NVD, DefectDojo, TAXII/MISP tokens)
|           |-- projectmem/                             # Existing project memory
|           |-- rtk/                                    # Existing RTK module
|           |-- rules/                                  # Existing rules + ATR/.scm/blind-OSV loaders
|           |-- telemetry/                              # Existing telemetry & audit events
|           |-- tools/                                  # Existing agent tools (guarded by CommandGuard)
|           |-- tui/                                    # Existing 60 FPS Ratatui TUI
|           |-- util/                                   # Existing forward-slash path & utility helpers
|           |-- security/                               # NEW zero-plaintext detection, compilation, search & export engine:
|           |   |-- mod.rs
|           |   |-- schema.rs                           # OSV 1.7.4, CVE 5.1, STIX 2.1, SARIF 2.1.0, BlindSignatures, Taxonomy
|           |   |-- polyglot.rs                         # Bridges codegraph's 38 Tree-sitter grammars + ignore AST/n-gram extractor
|           |   |-- blind_compiler.rs                   # compile-blind gateway (raw -> BLAKE3 + sparse::DFA before any disk/SQLite I/O)
|           |   |-- scanner.rs                          # Keyed BLAKE3 + zero-copy sparse::DFA + ATR/.scm scanner -> SARIF 2.1.0
|           |   |-- advisory_client.rs                  # Navegos/advisory O(1) lookup + ~/.toksi/feed/ cache + DisclosureGate + 12-DB dedup
|           |   `-- secure_export.rs                    # Hardened TLS 1.3 (rustls + secrecy + keyring) exporter (GitHub SARIF, DefectDojo, TAXII, MISP)
|           |-- validator/                              # NEW native validation, FP pruning, 3-tree sharding & Git/PR driver:
|           |   |-- mod.rs
|           |   |-- validate.rs                         # OSV 1.7.4, CVE 5.1, STIX 2.1, SARIF 2.1.0 & zero-plaintext verifier
|           |   |-- fp_pruner.rs                        # False-positive scrubber against corpus/benign/
|           |   |-- shard_builder.rs                    # Symmetrical 3-tree builder (advisories/, domains/, lookup/ + paired index.json/index.html)
|           |   `-- git_pr_driver.rs                    # Clones/updates Navegos/advisory at ~/.toksi/repos/advisory/, commits, pushes & opens PR via REST API
|           |-- lib.rs
|           `-- main.rs                                 # Wires `toksi compile-blind` and `toksi advisory {search,validate,prune-fp,build-shards,publish-pr}`
|-- agents/                                             # Audited SKILL.md bundles, AGENTS.md default-deny guardrails, command templates
`-- rules/
    |-- agent-threats/                                  # ATR (.yaml) detection rules
    `-- polyglot-code/                                  # Tree-sitter S-expression queries (.scm) mapped to CWEs
	

Repository 2: Navegos/advisory (https://github.com/Navegos/advisory.git -> https://navegos.github.io/advisory/)

advisory/                                               # Zero-Rust, zero-plaintext static GitHub Pages repository
|-- .nojekyll                                           # Disables Jekyll so nested folders & raw JSON serve cleanly
|-- index.html                                          # Root zero-dependency static browser explorer UI
|-- index.json                                          # Tiny root router manifest (~1 KB, permanently bounded)
|-- taxonomy.json                                       # Canonical 41+ language, script, web/API, CI/CD, shell & config registry
|-- CONTRIBUTING.md                                     # Schema & zero-plaintext contribution rules
|-- corpus/
|   `-- benign/                                         # Clean code & command fixtures for false-positive pruning
|       |-- _global/
|       `-- <taxonomy>/
|-- advisories/                                         # TREE 1: Pure OSV 1.7.4 & CVE 5.1 JSONs (zero raw exploits/binaries)
|   |-- index.html + index.json
|   `-- <taxonomy>/                                     # e.g., rust/crate, shell/bash, ci-cd/github-actions, javascript/script/mjs
|       |-- index.html + index.json
|       `-- <year>/                                     # e.g., 2026
|           |-- index.html + index.json
|           `-- <ID>/                                   # e.g., TOKSI-2026-0001
|               |-- index.html + index.json
|               `-- <ID>.json
|-- domains/                                            # TREE 2: Symmetrical paginated manifests (<= 500 compact rows/page)
|   |-- index.html + index.json
|   `-- <taxonomy>/
|       |-- index.html + index.json
|       `-- <year>/
|           |-- index.html + index.json
|           `-- page-001.json
|-- lookup/                                             # TREE 3: Symmetrical O(1) prefix-sharded ID maps
|   |-- index.html + index.json
|   |-- _global/
|   |   `-- {cve,vuln,ghsa,toksi}/<year>/<prefix>.json
|   `-- <taxonomy>/
|       |-- index.html + index.json
|       `-- {cve,vuln,ghsa,toksi}/<year>/<prefix>.json
|-- threat-intel/                                       # Open STIX 2.1 JSON bundles (with x_blind_signatures)
`-- findings/                                           # Published sanitized SARIF v2.1.0 baseline reports


# Objective: Embedded Zero-Plaintext Blind Signature Engine (`crates/toksi/src/security/`) & Native Git/PR Validator (`crates/toksi/src/validator/`) for `Navegos/neo` + `Navegos/advisory`

Extend the existing Rust crate at `crates/toksi/` in `[https://github.com/Navegos/neo.git](https://github.com/Navegos/neo.git)` (`src/{agents,aimemory,codegraph,commands,contextmode,core,dashboard,db,env_injector,gateway,headroom,icm,keyring,projectmem,rtk,rules,telemetry,tools,tui,util,lib.rs,main.rs}`) **without rewriting, breaking, or duplicating its core agent loop, command dispatcher, or SQLite persistence layer (`src/db/` -> `~/.toksi/toksi.db`)**. Consolidate both the zero-plaintext security engine (`crates/toksi/src/security/`) and the native validator/pruner/shard-builder/Git-PR driver (`crates/toksi/src/validator/`) directly inside `crates/toksi` so **no raw exploit string, PoC code, malicious payload, SQLite malware blob, or plaintext regex can ever be written to disk (`~/.toksi/toksi.db` or `Navegos/advisory`) or reconstructed in RAM during scanning**, and so `toksi` can validate, prune false positives, rebuild 3-tree static shards, push Git branches, and open Pull Requests against `[https://github.com/Navegos/advisory.git](https://github.com/Navegos/advisory.git)` (`[https://navegos.github.io/advisory/](https://navegos.github.io/advisory/)`) directly without needing GitHub Actions. Research and reuse patterns from `neo/inspect-vault/*` where applicable.

## Core Security, Privacy & Governance Invariants

1. **Zero-Plaintext Blind Signatures (`BLAKE3` + Serialized `sparse::DFA`):**
   - **Zero-Plaintext Rule:** Never persist raw exploit strings, PoC code, malicious payloads, SQLite malware blobs, or human-readable regexes on disk, in SQLite (`src/db/`), in Git commits, in SARIF reports, or in RAM during scanning.
   - **Exact & Tokenized Command/AST Matching:** Normalize input commands and AST token windows (lowercase + whitespace collapse + sliding n-grams `3, 4, 5`) and match against one-way keyed `BLAKE3` digests (`blake3::keyed_hash`).
   - **Wildcard & Pattern Matching:** Compile regex rules offline via `regex_automata::dfa::dense::Builder` -> `to_sparse()` -> `to_bytes_little_endian()`, Base64-encode into JSON, and execute zero-copy state-machine transitions (`sparse::DFA::from_bytes`) at runtime without plaintext regexes ever existing on disk or in RAM.
   - **Schema-Compliant Embedding:**
     - **OSV 1.7.4:** `database_specific.blind_signatures`
     - **CVE JSON 5.1:** `containers.adp[].x_blind_signatures`
     - **STIX 2.1:** `x_blind_signatures` extension on `indicator` / `attack-pattern` SDOs

2. **Responsible Disclosure & Embargo Policy (`DisclosureGate`):**
   - Block public publication of undisclosed zero-days, unpatched issues, or `TLP:RED`/`AMBER` private details unless a fix/patch is released in the affected package/repo, the issue is marked resolved with documented solutions, or the user explicitly passes `--force-publish` (still strictly zero-plaintext, hashing any PoC/exploit via `blind_compiler` into SQLite `db/` or OSV/CVE/SARIF/STIX schemas for reuse in future scans).

3. **12-Source Upstream Deduplication (`UpstreamDeduplicator`):**
   - Before minting a new `TOKSI-*` advisory, search `[https://navegos.github.io/advisory/](https://navegos.github.io/advisory/)` (`Navegos/advisory`) AND check NVD (`[https://nvd.nist.gov/](https://nvd.nist.gov/)`), Red Hat Security Data (`[https://access.redhat.com/hydra/rest/securitydata](https://access.redhat.com/hydra/rest/securitydata)`), GitLab Advisory DB (`[https://advisories.gitlab.com/](https://advisories.gitlab.com/)`), OSV (`[https://osv.dev/list](https://osv.dev/list)`, `[https://google.github.io/osv.dev/data/](https://google.github.io/osv.dev/data/)`, `osv-schema`, `osv-scanner`), RustSec (`[https://rustsec.org/advisories/](https://rustsec.org/advisories/)`, `rustsec` crate), cURL (`[https://curl.se/docs/vuln.json](https://curl.se/docs/vuln.json)`), GitHub Advisory Database (`[https://github.com/advisories](https://github.com/advisories)`), FriendsOfPHP (`[https://github.com/FriendsOfPHP/security-advisories](https://github.com/FriendsOfPHP/security-advisories)`), Go Vulnerability DB (`[https://vuln.go.dev/](https://vuln.go.dev/)`), PyPA Advisory Database (`[https://github.com/pypa/advisory-database](https://github.com/pypa/advisory-database)`), and RubySec (`[https://rubysec.com/](https://rubysec.com/)`) to prevent duplicates.

4. **Intent-Based & Capability-Based AI Agent Command Guardrails (`CommandGuard`):**
   - Enforce default-deny `AGENTS.md` allowlists in `crates/toksi/src/commands/guard.rs` (and `tools/` shell invocations) blocking unauthorized offensive reconnaissance, exploitation, credential-dumping, tunneling, DoS, and destructive binaries (`nmap`, `masscan`, `rustscan`, `gobuster`, `ffuf`, `tcpdump`, `wireshark`, `responder`, `msfconsole`, `msfvenom`, `mimikatz`, `hydra`, `hashcat`, `john`, `impacket`, `cobaltstrike`, `sliver`, `nc`, `ncat`, `socat`, `chisel`, `ngrok`, `proxychains`, `hping3`, `slowloris`, `dd`, `mkfs`, `shred`, `sudo`, `su`, `chmod 777`, `iptables`, `ufw`) plus custom Python/Bash/Go/PowerShell raw-socket reverse shells.

## Architectural Deliverables

### 1. `crates/toksi/src/security/` (Zero-Plaintext Engine)
- **`schema.rs`:** `serde` models for OSV 1.7.4, CVE JSON 5.1, STIX 2.1, SARIF 2.1.0 (`serde-sarif`), `SKILL.md`, `AGENTS.md`, ATR YAML, `taxonomy.json`, `BlindSignatures`, `TargetKind` (`package`, `source_repo`, `agent`, `skill`, `rule`, `command`, `script`, `code_snippet`), `SourceOrigin` (`github_repo`, `external_git_repo`, `package_registry`, `distributed`), and `DistributionChannel` (`oci_registry`, `release_asset`, `archive_bundle`, `raw_url`, `local_dir`, `single_file`).
- **`polyglot.rs`:** Reuses `crates/toksi/src/codegraph/`'s existing 38 `tree-sitter` parsers + `ignore` walker across all 41+ taxonomies to extract AST sinks, lockfile dependencies, and normalized sliding token windows.
- **`blind_compiler.rs` (`compile-blind`):** Sole serialization gateway converting raw patterns/tokens into keyed `BLAKE3` digests and little-endian `sparse::DFA` Base64 buffers and stripping all plaintext before any write to SQLite (`src/db/`), `~/.toksi/`, or `Navegos/advisory`.
- **`scanner.rs`:** Evaluates keyed `BLAKE3` n-grams, zero-copy `sparse::DFA::from_bytes` state machines, ATR YAML rules, and `.scm` queries -> outputs sanitized SARIF 2.1.0 logs.
- **`advisory_client.rs`:** Searches `[https://navegos.github.io/advisory/](https://navegos.github.io/advisory/)` in O(1) via `lookup/` prefix shards and `domains/` paginated manifests (cached in `~/.toksi/feed/`), enforcing `DisclosureGate` and `UpstreamDeduplicator` prior to publishing.
- **`secure_export.rs`:** Hardened TLS 1.3 (`rustls` + `secrecy`, pulling credentials from `crates/toksi/src/keyring/` and redacting paths via `env_injector`/`util`) for GitHub Code Scanning (gzip+base64 SARIF), DefectDojo, TAXII 2.1/OpenCTI, and MISP.

### 2. `crates/toksi/src/validator/` (Native Validation, Pruning, Sharding & Git/PR Driver)
- **`validate.rs`:** Validates OSV 1.7.4, CVE 5.1, STIX 2.1, SARIF 2.1.0, ATR, `SKILL.md`, `AGENTS.md`, and enforces zero-plaintext hygiene.
- **`fp_pruner.rs`:** Evaluates `BLAKE3` n-grams and `sparse::DFA` automata against `corpus/benign/`, surgically strips false-positive entries, and deletes or marks `withdrawn` empty advisories.
- **`shard_builder.rs`:** Generates the symmetrical 3-tree GitHub Pages layout (`advisories/`, `domains/`, `lookup/`) with paired `index.json` + `index.html` in every directory (max 500 compact rows per `page-NNN.json`).
- **`git_pr_driver.rs`:** Manages a local working clone of `[https://github.com/Navegos/advisory.git](https://github.com/Navegos/advisory.git)` at `~/.toksi/repos/advisory/` or a dir configured by the user, runs `validate` -> `prune-fp` -> `build-shards` atomically on a feature branch, commits, pushes via Git, and opens a Pull Request via GitHub REST API (`POST /repos/Navegos/advisory/pulls`) using `crates/toksi/src/keyring/` tokens so GitHub Pages serves directly from `main` without GitHub Actions.

### 3. Integration Hooks into Existing `crates/toksi/src/` Modules
- **`db/`:** Keep `src/db/` strictly for internal SQLite (`~/.toksi/toksi.db`); add non-breaking tables/mappers storing only compiled `BlindSignatures`, TLP embargo state, and scan provenance (never raw exploit code).
- **`commands/guard.rs` & `main.rs`:** Wire pre-execution `CommandGuard` into `commands/` + `tools/`, and expose CLI subcommands (`toksi compile-blind`, `toksi advisory search`, `toksi advisory validate`, `toksi advisory prune-fp`, `toksi advisory build-shards`, `toksi advisory publish-pr`).
- **`agents/`, `rules/`, `telemetry/`:** Export SARIF provenance (`tool.driver`, `tool.extensions`, `invocations`), load ATR/`.scm`/blind-OSV rules into `rules/`, and emit security audit events to `telemetry/`.

### 4. `Navegos/advisory` Static Repository Scaffold
- Zero Rust code; holds `.nojekyll`, root `index.html` explorer UI, `index.json` (~1 KB router), `taxonomy.json` (41+ languages, scripts, web/API, CI/CD, shells, IaC/containers, build tools, configs, SQL, docs), `corpus/benign/`, symmetrical `advisories/`, `domains/`, `lookup/` trees with paired `index.json` + `index.html`, `threat-intel/` (STIX 2.1), and `findings/` (sanitized SARIF 2.1.0).

## Execution Steps
1. **Inspect First:** Read root `Cargo.toml`, `crates/toksi/Cargo.toml`, and `crates/toksi/src/{lib.rs,main.rs,agents,codegraph,commands,core,db,keyring,rules,tools}` to align with existing Tree-sitter bindings, SQLite schema in `db/`, `keyring` APIs, CLI `clap` structure, and `tokio` runtime.
2. **Implement `security/` & `validator/`:** Build `crates/toksi/src/security/{schema.rs, polyglot.rs, blind_compiler.rs, scanner.rs, advisory_client.rs, secure_export.rs}` and `crates/toksi/src/validator/{validate.rs, fp_pruner.rs, shard_builder.rs, git_pr_driver.rs}`.
3. **Wire Runtime Hooks:** Connect `CommandGuard`, SQLite blind-signature/embargo tables in `db/`, `rules/` loaders, `agents/` SARIF provenance, and `main.rs` CLI subcommands.
4. **Scaffold `Navegos/advisory` & Verify:** Generate the initial `Navegos/advisory` static structure (`taxonomy.json`, root `index.html`, `corpus/benign/`) and run `cargo check` and `cargo test` confirming zero plaintext leaks on disk or in SQLite/RAM, cross-endian `sparse::DFA` execution, `CommandGuard` blocking, `DisclosureGate` embargo enforcement, false-positive pruning, 3-tree shard generation, and Git PR creation.


we also need to implement that not publishing the same cve or problem's over again, or available in the external Security vulnerability database's:
our database https://github.com/Navegos/advisory.git
(https://nvd.nist.gov/)
(https://access.redhat.com/hydra/rest/securitydata)
(https://advisories.gitlab.com/)
(https://osv.dev/list, osv-schema, osv-scanner)
(https://rustsec.org/advisories/, https://crates.io/crates/rustsec)
(https://curl.se/docs/vuln.json)
(GitHub Advisory Database, https://github.com/advisories)
(https://github.com/FriendsOfPHP/security-advisories)
(https://vuln.go.dev/)
(https://github.com/pypa/advisory-database)
(https://rubysec.com/)
(https://google.github.io/osv.dev/data/)

we also need to implement search in the our database github repo https://github.com/Navegos/advisory.git
