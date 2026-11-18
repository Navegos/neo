# Issue #0032 - Antigravity eligibility check returns 404 not found Google upstream service endpoint not found

**Created:** 2026-11-04T16:13:34Z
**Location:** `crates/toksi/src/headroom/proxy.rs:672 (serve_headroom_connection)`
**Commit:** `15bb1ea`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-04T16:13:34Z] Opened: Antigravity eligibility check returns 404 not found Google upstream service endpoint not found
- [2026-11-04T16:34:14Z] Attempt (worked): Implemented intelligent sibling upstream fallback between aicode.googleapis.com and cloudcode-pa.googleapis.com on 404, fixed primary discrimination for v1internal vs v1main, and added proxy route logging
- [2026-11-04T16:34:30Z] Fix confirmed: Automatic sibling upstream fallback between aicode.googleapis.com and cloudcode-pa.googleapis.com on 404, eliminating 404 endpoint not found during Antigravity eligibility checks
- [2026-10-31T00:34:57Z] Opened: CLI test issue
- [2026-10-31T00:35:03Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-31T00:35:10Z] Fix confirmed: Fixed assertion in test_foo
