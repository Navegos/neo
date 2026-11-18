# Issue #0005 - RTK hook cline/copilot/codex regressions: copilot unconditionally allows non-rtk commands, missing post-tool hook, cline ignores toolName/commands array

**Created:** 2026-10-26T23:24:40Z
**Location:** `crates/tokless/src/commands/rtk_hook.rs`
**Commit:** `97e1db7`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-26T23:24:40Z] Opened: RTK hook cline/copilot/codex regressions: copilot unconditionally allows non-rtk commands, missing post-tool hook, cline ignores toolName/commands array
- [2026-10-27T02:03:54Z] Fix confirmed: Fixed OrderedMap type mismatches in rtk_hook parse and tests using try_parse_jsonc_map
- [2026-10-29T18:30:37Z] Opened: CLI test issue
- [2026-10-29T18:30:43Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-29T18:30:48Z] Fix confirmed: Fixed assertion in test_foo
