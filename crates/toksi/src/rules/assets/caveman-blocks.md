# .blocks Specification

Structured inline header metadata (`# /// block` ... `# ///`) enabling machine-verifiable tool/script contracts and smoke verification.

## Required Block Fields

```toml
# /// block
# name = "<block-name>"
# summary = "<one-line purpose>"
# effects = "none | filesystem | network | process"
#
# [params]
# param_name = { type = "str", required = true, help = "..." }
#
# [returns]
# keys = ["field_a", "field_b"]
# doc = "<output contract summary>"
# ///
```

## Verification Contract (`release-smoke`)

- Strict semver (`^\d+\.\d+\.\d+$`) and repository identifier validation before execution.
- Isolated environment harness (`HOME`, `CLAUDE_CONFIG_DIR`, `CODEX_HOME`, `CAVEMAN_PLAIN=1`, `NO_COLOR=1`) preserving unrelated user config sentinels.
- Byte-exact recovery verification (`byte_exact_recovery`) and clean uninstall checks leaving user config intact.
- Offline / local compilation only: runtime downloads prohibited inside toksi core.
