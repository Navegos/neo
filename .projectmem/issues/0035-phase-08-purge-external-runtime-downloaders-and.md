# #0035 Phase 08: Purge external runtime downloaders and rewire agent harnesses to in-tree modules

- 2026-11-05T14:39:48Z `issue`: Phase 08: Purge external runtime downloaders and rewire agent harnesses to in-tree modules [crates/toksi/src/util/]
- 2026-11-06T00:50:00Z `attempt`: Purged downloader references, simplified headroom paths, wired MCP aliases to in-tree toksi, verified 5/5 phase 8 tests and 321 unit tests pass (worked)
- 2026-11-06T00:50:17Z `fix`: Deleted runtime downloaders (npminstall, nodewin, nodelinux, uv, projectmem), wired 17 agents and tools to native in-tree Rust modules, enforced system detection policy via which::which, 0 compiler warnings, all tests pass
