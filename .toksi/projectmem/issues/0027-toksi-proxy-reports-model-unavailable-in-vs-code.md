# #0027 toksi proxy reports model unavailable in VS Code antigravity extension while terminal works

- 2026-11-03T11:51:03Z `issue`: toksi proxy reports model unavailable in VS Code antigravity extension while terminal works [crates/toksi/src/main.rs:252 (run)]
- 2026-11-03T12:02:50Z `attempt`: Fixed proxy target resolution: routes Gemini API /v1beta and models to generativelanguage instead of cloudcode, added CORS OPTIONS preflight handler, and fixed /v1 path stripping in byok_target_path (worked)
- 2026-11-03T12:03:15Z `fix`: Proxy routes /v1beta and gemini endpoints to generativelanguage even when x-goog-api-client has antigravity header. Added CORS preflight 204 handler and fixed /v1 path preservation in byok_target_path. Tested by test_non_llm_uri_and_target_resolution.
