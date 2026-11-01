# #0024 Headroom proxy routes non-LLM requests to OpenAI instead of actual target and discards query strings, breaking Antigravity onboarding and auth

- 2026-11-01T20:33:19Z `issue`: Headroom proxy routes non-LLM requests to OpenAI instead of actual target and discards query strings, breaking Antigravity onboarding and auth [crates/toksi/src/headroom/proxy.rs:559 (serve_headroom_connection)]
- 2026-11-01T20:38:50Z `attempt`: Forward non-LLM requests directly to upstream service without payload modification and preserve query params (worked)
- 2026-11-01T20:39:06Z `fix`: Headroom proxy detects non-LLM requests, bypasses prompt modifications, preserves query strings, and routes Cloud Code endpoints to https://cloudcode-pa.googleapis.com
