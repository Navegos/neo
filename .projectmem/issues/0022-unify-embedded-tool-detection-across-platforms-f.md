# #0022 Unify embedded tool detection across platforms: fallback to in-tree toksi binary for rtk, codegraph, context-mode, projectmem, headroom

- 2026-11-01T18:39:51Z `issue`: Unify embedded tool detection across platforms: fallback to in-tree toksi binary for rtk, codegraph, context-mode, projectmem, headroom [crates/toksi/src/commands/doctor_stats.rs:430 (doctor_tool_stats)]
- 2026-11-01T18:40:04Z `attempt`: Update doctor_stats, versions, doctor_probe, projectmem, and rtk tool to resolve in-tree toksi binary for embedded tools [crates/toksi/src/commands/doctor_stats.rs:430 (doctor_tool_stats)] (partial)
- 2026-11-01T18:47:53Z `fix`: Unified embedded tool resolution across Windows, Linux, and Mac: fallback to in-tree toksi binary for rtk, codegraph, context-mode, projectmem, headroom
