# caveman-shrink

Compress MCP/OpenAI tool catalogs (`stdin` -> `stdout`) so large tool lists cost fewer tokens without altering tool selection schemas.

## Guarantees

- **Structural selection surface preserved**: tool names, parameter names, enums, `required`, `default`, `const`, and `$ref` targets remain byte-exact.
- **Annotation bloat stripped**: drops non-essential titles, comments, examples, and verbose schema descriptions while retaining lead sentence plus constraint-bearing sentences.
- **Fail-open**: malformed or incompressible input passes through unchanged.
- **Bounded input**: input capped at 32 MiB (`cave_input_too_large` beyond limit).
- **Reversible CCR recovery**: original catalog bytes commit under `CAVEMAN_CCR_DB` (or `~/.caveman/ccr.db`) keyed by handle `ccr_...` (`caveman-shrink recover ccr_...`).
- **Offline / in-tree policy**: toksi embeds tool schema compaction locally; never perform runtime network binary fetches.
