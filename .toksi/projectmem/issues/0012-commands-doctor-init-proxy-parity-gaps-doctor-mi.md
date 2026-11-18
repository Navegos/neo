# Issue #0012 - Commands doctor/init/proxy parity gaps: doctor missing per-agent tool/proxy/instruction rows and proxy port checks; init missing excluded-tool unwire and persisted selection load

**Created:** 2026-10-26T23:26:43Z
**Location:** `crates/tokless/src/commands/doctor.rs`
**Commit:** `97e1db7`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-10-26T23:26:43Z] Opened: Commands doctor/init/proxy parity gaps: doctor missing per-agent tool/proxy/instruction rows and proxy port checks; init missing excluded-tool unwire and persisted selection load
- [2026-10-27T02:45:11Z] Fix confirmed: Implemented doctor per-agent tool rows and Ports overview, init excluded tools unwire and selection persistence, proxy lifecycle lock and rollback transaction
- [2026-10-29T20:02:41Z] Opened: CLI test issue
- [2026-10-29T20:02:48Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-29T20:02:53Z] Fix confirmed: Fixed assertion in test_foo
