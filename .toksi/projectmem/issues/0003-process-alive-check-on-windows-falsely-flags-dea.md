# #0003 Process alive check on Windows falsely flags dead processes with exit code 259 (STILL_ACTIVE) as running

- 2026-10-20T18:37:30Z `issue`: Process alive check on Windows falsely flags dead processes with exit code 259 (STILL_ACTIVE) as running [crates/tokless/src/agents/proxy_provider.rs:174]
- 2026-10-20T18:44:35Z `fix`: Use WaitForSingleObject with 0 timeout on Windows handle to detect still running processes without still_active 259 false-positives; eliminate unwrap in mcp_proxy; atomic write_file for instructions and versions
