# Issue #0008 - CodeGraph tool missing antigravity/droid/pi hooks and package purge, status --json health check, codegraph init vs index --quiet, and opencode autoindexjs unwire

**Created:** 2026-10-26T23:25:16Z
**Location:** `crates/tokless/src/tools/codegraph.rs`
**Commit:** `97e1db7`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-26T23:25:16Z] Opened: CodeGraph tool missing antigravity/droid/pi hooks and package purge, status --json health check, codegraph init vs index --quiet, and opencode autoindexjs unwire
- [2026-10-27T02:46:44Z] Fix confirmed: CodeGraph tool equipped with hooks for antigravity, droid, pi, and opencode/opencode2 autoindex unwiring
- [2026-10-29T19:16:08Z] Opened: CLI test issue
- [2026-10-29T19:16:14Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-29T19:16:19Z] Fix confirmed: Fixed assertion in test_foo
