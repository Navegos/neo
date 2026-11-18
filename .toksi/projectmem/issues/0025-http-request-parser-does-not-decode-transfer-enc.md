# Issue #0025 - HTTP request parser does not decode Transfer-Encoding chunked, corrupting JSON bodies with chunk headers and failing requests with HTTP 400

**Created:** 2026-11-02T16:30:47Z
**Location:** `crates/toksi/src/headroom/byok_gateway.rs:950 (read_http_request)`
**Commit:** `34b9ff7`
**Status:** Resolved (fixed by: Fixed assertion in test_foo)

## History

- [2026-11-02T16:30:47Z] Opened: HTTP request parser does not decode Transfer-Encoding chunked, corrupting JSON bodies with chunk headers and failing requests with HTTP 400
- [2026-11-02T16:33:25Z] Attempt (worked): Implement HTTP chunked transfer decoding in read_http_request and read_chunked_body
- [2026-11-02T16:33:47Z] Fix confirmed: Add Transfer-Encoding chunked decoding loop to read_chunked_body, stripping chunk size headers (e.g. 10103\r\n) and reassembling valid JSON body with Content-Length
- [2026-10-30T23:49:39Z] Opened: CLI test issue
- [2026-10-30T23:49:46Z] Attempt (failed): Failed: thread 'test_foo' panicked at 'assertion failed: left == right' (cmd: `cargo test`)
  - Details: running 1 test test test_foo ... FAILED thread 'test_foo' panicked at 'assertion failed: left == right'
- [2026-10-30T23:49:53Z] Fix confirmed: Fixed assertion in test_foo
