# #0028 VS Code Antigravity extension fails to connect due to missing keep-alive loop, route discrimination, and gzip content-encoding leak in proxy stream

- 2026-11-03T20:20:51Z `issue`: VS Code Antigravity extension fails to connect due to missing keep-alive loop, route discrimination, and gzip content-encoding leak in proxy stream [crates/toksi/src/headroom/proxy.rs:543 (serve_headroom_connection)]
- 2026-11-03T20:21:46Z `fix`: Added HTTP/1.1 keep-alive loop, Google Cloud Code route discrimination, stripped Content-Encoding on decompressed stream, and added VS Code settings candidate path [crates/toksi/src/headroom/proxy.rs:543 (serve_headroom_connection)]
- 2026-11-03T20:53:41Z `fix`: Eliminated false chunking of HTTP/2 responses lacking Content-Length that broke protobuf JSON parsing with invalid value []. Wired GatewayProxyOrchestrator semantic cache into proxy loop. [crates/toksi/src/headroom/byok_gateway.rs:1640 (forward_streaming_to_byok_target)]
