# #0020 Headroom proxy startup fails identity verification when routing disabled

- 2026-11-01T15:56:07Z `issue`: Headroom proxy startup fails identity verification when routing disabled [crates/toksi/src/headroom/proxy.rs:498 (run_proxy_foreground)]
- 2026-11-01T15:57:40Z `attempt`: Remove proxy_routing_enabled check in run_proxy_foreground, enable routing in start_proxy, extend identity timeout and error fallback on Windows (worked)
- 2026-11-01T15:59:19Z `fix`: Prevent premature exit in run_proxy_foreground by removing routing_enabled guard on serve mode, enable routing preference on start_proxy, and improve Windows identity verification fallback/timeouts
