# neo — plan

> Editable **intent** file: ideas + plans — what we *mean to do*.
> This is NOT the event log. `events.jsonl` -> `summary.md` records what
> *happened*; this file records what we *intend*. The AI reads it at
> session start and edits it directly (like `PROJECT_MAP.md`): add ideas
> and plans, check items off, move done work down to Shipped. Plans are
> never logged as events.

## Ideas
_Loose thoughts, not yet committed to._

## Active plans
_What we're working toward now. Use `- [ ]` / `- [x]` checklists._

- [ ] Initialize `crates/tokless` crate in root `Cargo.toml`
- [ ] Port `internal/core` (types, registries, detection) to Rust
- [ ] Port `internal/util` (paths, colors, logger, jsonc, toml, exec, deps, prompt, progress, byok, routing state)
- [ ] Port `internal/tools` (rtk, caveman, ponytail, codegraph, contextmode, headroom, projectmem)
- [ ] Port `internal/agents` (agent definitions, detection, configs, hooks, proxy routing for all 12 agents)
- [ ] Port `internal/headroom` (proxy engine, router, autostart, byok gateway, watchdog, daemons)
- [ ] Port `internal/commands` and CLI entrypoint `tokless` binary
- [ ] Verify test suite & CLI execution

## Next
_Queued, but not started._

## Someday / maybe

## Shipped
_Move completed plans here so the top stays about the future._
