# #0021 Legacy external rtk binary fallback in rtk_rewrite causes unrecognized commands to emit bare rtk

- 2026-11-01T18:18:09Z `issue`: Legacy external rtk binary fallback in rtk_rewrite causes unrecognized commands to emit bare rtk [crates/toksi/src/commands/rtk_hook.rs:52 (rtk_rewrite)]
- 2026-11-01T18:19:30Z `attempt`: Remove legacy external rtk binary fallback in rtk_rewrite and support toksi rtk prefix [crates/toksi/src/commands/rtk_hook.rs:43 (rtk_rewrite)] (partial)
- 2026-11-01T18:23:54Z `fix`: Purged external rtk binary fallback from rtk_rewrite; unsupported commands stay unmutated, supported commands rewrite to toksi rtk
