# #0029 RTK hook not launching for command runs in Antigravity due to missing workspace .agents/hooks.json, sandbox permission denials on ~/.gemini/config, and invalid context-mode subcommand in hooks.json

- 2026-11-03T21:32:42Z `issue`: RTK hook not launching for command runs in Antigravity due to missing workspace .agents/hooks.json, sandbox permission denials on ~/.gemini/config, and invalid context-mode subcommand in hooks.json [crates/toksi/src/agents/antigravity.rs:210 (antigravity_hooks_file)]
- 2026-11-03T22:29:37Z `attempt`: Implemented run_ctx_hook with ctx-hook, context-mode-hook, and context-mode aliases in main.rs and ctx.rs [crates/toksi/src/main.rs:273 (run)] (worked)
- 2026-11-03T22:29:49Z `fix`: Added ctx-hook and context-mode aliases in toksi CLI, installed fresh binary to C:/Programs/neo/bin/toksi.exe
