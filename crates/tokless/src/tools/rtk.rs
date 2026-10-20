// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tools/rtk.rs
// created: 2026-10-05
// lastModified: 2026-10-20

use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::agents::{
    allow_antigravity_entry, allow_claude_bash_pattern, configure_cursor_rtk_permissions,
    disallow_claude_bash_pattern, has_antigravity_rtk_hook, has_codex_rtk_hook,
    has_copilot_ide_rtk_hook, has_copilot_rtk_hook, has_cursor_rtk_hook,
    has_cursor_rtk_permissions, has_droid_rtk_hook, has_grok_rtk_hook,
    install_antigravity_rtk_hook, install_codex_rtk_hook, install_copilot_ide_rtk_hook_safe,
    install_copilot_rtk_hook_safe, install_cursor_rtk_hook, install_droid_rtk_hook,
    install_grok_codegraph_session_hook, install_grok_rtk_hook, omp_agent_dir_resolved,
    pi_agent_dir_resolved, remove_antigravity_entry, remove_antigravity_rtk_hook,
    remove_codex_rtk_hook, remove_copilot_ide_rtk_hook_safe, remove_copilot_rtk_hook_safe,
    remove_cursor_rtk_hook, remove_cursor_rtk_permissions, remove_droid_rtk_hook,
    remove_grok_codegraph_session_hook, remove_grok_rtk_hook,
};
use crate::core::{Channel, DynError, RunOpts, ToolManifest};
use crate::util::exec::run_cmd;
use crate::util::instructions::{cursor_project_rule_content, install_cursor_project_rules};
use crate::util::jsonc::{stringify_json, try_parse_jsonc, OrderedMap, Value};
use crate::util::paths::{
    amp_paths_resolved, claude_code_paths, cline_paths_resolved, cursor_paths_resolved, ensure_dir,
    exists, home, is_win, kilo_paths_resolved, opencode_paths_resolved, persisted_tokless_command,
    prepend_process_path, read_file_safe, resolve_rtk_bin, temp_dir, tokless_persisted_abs, which,
    write_file_atomic, zcode_paths_resolved,
};
use super::copilot_transaction::with_copilot_transaction;
use super::helpers::{
    has_owner, is_test, kilo_has_owner, kilo_remove_owner, kilo_write_owner, remove_owner_safe,
    sync_copilot_ide_instructions_safe, write_owner,
};

pub fn rtk_asset_for_platform() -> &'static str {
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x86_64"
    };

    if cfg!(target_os = "macos") {
        if arch == "aarch64" {
            "rtk-aarch64-apple-darwin.tar.gz"
        } else {
            "rtk-x86_64-apple-darwin.tar.gz"
        }
    } else if cfg!(target_os = "linux") {
        if arch == "aarch64" {
            "rtk-aarch64-unknown-linux-gnu.tar.gz"
        } else {
            "rtk-x86_64-unknown-linux-musl.tar.gz"
        }
    } else if cfg!(target_os = "windows") {
        if arch == "aarch64" {
            "rtk-aarch64-pc-windows-msvc.zip"
        } else {
            "rtk-x86_64-pc-windows-msvc.zip"
        }
    } else {
        ""
    }
}

pub fn rtk_ensure_installed(opts: &RunOpts) -> Result<bool, DynError> {
    if is_test() {
        let shim_dir = Path::new(&temp_dir()).join("tokless-test-rtk");
        let _ = fs::create_dir_all(&shim_dir);
        let shim_path = shim_dir.join(if is_win() { "rtk.bat" } else { "rtk" });
        let _ = fs::write(&shim_path, if is_win() { "@echo ok" } else { "#!/bin/sh\necho ok" });
        prepend_process_path(&shim_dir.to_string_lossy());
        return Ok(true);
    }

    if let Some(ref report) = opts.report {
        report("checking", 0.1);
    }
    let p = resolve_rtk_bin();
    if !p.is_empty() && !opts.upgrade {
        if let Some(ref report) = opts.report {
            report("already installed", 1.0);
        }
        return Ok(true);
    }

    if opts.dry_run {
        return Ok(true);
    }

    let asset = rtk_asset_for_platform();
    if !asset.is_empty() && rtk_install_prebuilt(asset, opts) {
        if let Some(ref report) = opts.report {
            report("ready", 1.0);
        }
        return Ok(true);
    }

    if !is_win() && !which("curl").is_empty() && !which("sh").is_empty() {
        let res = run_cmd(
            "sh",
            &["-c", "curl -fsSL https://raw.githubusercontent.com/Navegos/rtk/master/install.sh | sh"],
            &[],
        );
        if res.code == 0 {
            return Ok(true);
        }
    }

    if !which("cargo").is_empty() {
        let res = run_cmd("cargo", &["install", "--git", "https://github.com/Navegos/rtk"], &[]);
        if res.code == 0 {
            return Ok(true);
        }
    }

    Err("Cannot install rtk on this platform. See https://github.com/Navegos/rtk for manual install.".into())
}

fn rtk_install_prebuilt(asset: &str, opts: &RunOpts) -> bool {
    let url = format!("https://github.com/Navegos/rtk/releases/latest/download/{}", asset);
    let dest = Path::new(&home()).join(".local").join("bin");
    let _ = fs::create_dir_all(&dest);

    if let Some(ref report) = opts.report {
        report("downloading binary", 0.3);
    }

    if is_win() {
        let temp = env::var("TEMP").unwrap_or_else(|_| "C:\\Temp".into());
        let zip_dest = format!("{}\\rtk.zip", temp);
        let ps = format!(
            "$ErrorActionPreference='Stop'; Invoke-WebRequest -UseBasicParsing -Uri '{}' -OutFile '{}'; Expand-Archive -Force -Path '{}' -DestinationPath '{}'; Remove-Item '{}'",
            url,
            zip_dest,
            zip_dest,
            dest.to_string_lossy(),
            zip_dest
        );
        let res = run_cmd("powershell", &["-NoProfile", "-Command", &ps], &[]);
        if res.code != 0 {
            return false;
        }
        prepend_process_path(&dest.to_string_lossy());
        return true;
    }

    if let Some(ref report) = opts.report {
        report("extracting", 0.8);
    }
    let tar_dest = format!("{}/rtk.tar.gz", temp_dir());
    let curl_res = run_cmd("curl", &["-fsSL", "-o", &tar_dest, &url], &[]);
    if curl_res.code != 0 {
        return false;
    }
    let tar_res = run_cmd("tar", &["-xzf", &tar_dest, "-C", &dest.to_string_lossy()], &[]);
    let _ = fs::remove_file(&tar_dest);
    if tar_res.code != 0 {
        return false;
    }
    prepend_process_path(&dest.to_string_lossy());
    true
}

// --- OMP Extension ---
const OMP_RTK_EXTENSION: &str = r#"import { spawn } from "node:child_process"
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent"
const TOKLESS = %TOKLESS%

export default function (pi: ExtensionAPI) {
  pi.on("tool_call", async (event: any) => {
    if (event?.toolName !== "bash" || typeof event?.input?.command !== "string") return
    const payload = { tool_name: "Bash", tool_input: { command: event.input.command } }
    const stdout = await new Promise<string>((resolve) => {
      let settled = false
      const finish = (value = "") => {
        if (settled) return
        settled = true
        resolve(value)
      }
      try {
        const child = spawn(TOKLESS, ["rtk-hook", "omp"], { stdio: ["pipe", "pipe", "ignore"] })
        let output = ""
        child.stdout.setEncoding("utf8")
        child.stdout.on("data", (chunk) => { output += chunk })
        child.once("error", () => finish())
        child.stdin.once("error", () => finish())
        child.once("close", (code) => finish(code === 0 ? output : ""))
        child.stdin.end(JSON.stringify(payload))
      } catch {
        finish()
      }
    })
    if (!stdout) return
    let rewritten
    try {
      rewritten = JSON.parse(stdout)?.hookSpecificOutput?.updatedInput?.command
    } catch {
      return
    }
    if (typeof rewritten !== "string") return
    event.input.command = rewritten
  })
}
"#;

fn omp_rtk_extension_path() -> String {
    Path::new(&omp_agent_dir_resolved())
        .join("extensions")
        .join("tokless-rtk.ts")
        .to_string_lossy()
        .to_string()
}

pub fn write_omp_rtk_extension() -> bool {
    let p = omp_rtk_extension_path();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let exe = tokless_persisted_abs();
    let quoted_exe = format!("{:?}", exe);
    let content = OMP_RTK_EXTENSION.replace("%TOKLESS%", &quoted_exe);
    write_file_atomic(&p, &content).is_ok()
}

pub fn omp_rtk_extension_valid() -> bool {
    let p = omp_rtk_extension_path();
    if let Some(raw) = read_file_safe(&p) {
        raw.contains("TOKLESS") && raw.contains("rtk-hook") && raw.contains("omp")
    } else {
        false
    }
}

// --- Kilo Plugin ---
const KILO_RTK_PLUGIN: &str = r#"import type { Plugin } from "@kilocode/plugin"

// tokless-kilo-rtk-v1
const rtk = %RTK%
const server: Plugin = async ({ $ }) => ({
  "tool.execute.before": async (input, output) => {
    const tool = String(input?.tool ?? "").toLowerCase()
    if (tool !== "bash" && tool !== "shell") return
    if (!output || typeof output.args !== "object" || output.args === null) return
    const args = output.args as Record<string, unknown>
    const command = args.command
    if (typeof command !== "string") return
    try {
      const result = await `${rtk} rtk-rewrite -- ${command}`.quiet().nothrow()
      const rewritten = String(result.stdout).trim()
      if (rewritten && rewritten !== command) args.command = rewritten
    } catch {}
  },
})

export default server
"#;

fn kilo_rtk_path() -> String {
    Path::new(&kilo_paths_resolved().plugins_dir)
        .join("rtk.ts")
        .to_string_lossy()
        .to_string()
}

fn kilo_rtk_backup_path() -> String {
    Path::new(&kilo_paths_resolved().plugins_dir)
        .join("rtk.ts.tokless-backup")
        .to_string_lossy()
        .to_string()
}

fn kilo_rtk_wire(opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let p = kilo_rtk_path();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    if let Some(existing) = read_file_safe(&p) {
        if !existing.contains("tokless-kilo-rtk-v1") {
            let _ = write_file_atomic(&kilo_rtk_backup_path(), &existing);
        }
    }
    let exe = tokless_persisted_abs();
    let quoted = format!("{:?}", exe);
    let content = KILO_RTK_PLUGIN.replace("%RTK%", &quoted);
    write_file_atomic(&p, &content)?;
    let _ = kilo_write_owner("rtk");
    Ok(kilo_rtk_verify())
}

fn kilo_rtk_unwire(_opts: &RunOpts) -> Result<bool, DynError> {
    let p = kilo_rtk_path();
    let backup = kilo_rtk_backup_path();
    if let Some(orig) = read_file_safe(&backup) {
        let _ = write_file_atomic(&p, &orig);
        let _ = fs::remove_file(&backup);
    } else if exists(&p) {
        let _ = fs::remove_file(&p);
    }
    let _ = kilo_remove_owner("rtk");
    Ok(true)
}

fn kilo_rtk_verify() -> bool {
    let p = kilo_rtk_path();
    let plugin_ok = read_file_safe(&p)
        .map(|r| r.contains("tokless-kilo-rtk-v1") && r.contains("rtk-rewrite"))
        .unwrap_or(false);
    plugin_ok && kilo_has_owner("rtk")
}

// --- Cline Hook ---
const CLINE_RTK_MARKER: &str = "tokless-cline-rtk-v1";

fn cline_rtk_hook_path() -> String {
    let name = if is_win() { "PreToolUse.cjs" } else { "PreToolUse" };
    Path::new(&cline_paths_resolved().hooks_dir)
        .join(name)
        .to_string_lossy()
        .to_string()
}

fn cline_rtk_backup_path() -> String {
    format!("{}.tokless-backup", cline_rtk_hook_path())
}

fn cline_rtk_wire(opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let p = cline_rtk_hook_path();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    if let Some(existing) = read_file_safe(&p) {
        if !existing.contains(CLINE_RTK_MARKER) {
            let _ = write_file_atomic(&cline_rtk_backup_path(), &existing);
        }
    }
    let exe = tokless_persisted_abs();
    let content = if is_win() {
        format!(
            "#!/usr/bin/env node\n// {}\nconst {{ spawnSync }} = require(\"child_process\");\nconst r = spawnSync({:?}, [\"rtk-hook\", \"cline\"], {{ stdio: \"inherit\" }});\nprocess.exit(typeof r.status === \"number\" ? r.status : 0);\n",
            CLINE_RTK_MARKER, exe
        )
    } else {
        format!("#!/bin/sh\n# {}\nexec {:?} rtk-hook cline\n", CLINE_RTK_MARKER, exe)
    };
    write_file_atomic(&p, &content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(&p) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(&p, perms);
        }
    }
    let _ = write_owner("cline", "rtk");
    Ok(cline_rtk_verify())
}

fn cline_rtk_unwire(_opts: &RunOpts) -> Result<bool, DynError> {
    let p = cline_rtk_hook_path();
    let backup = cline_rtk_backup_path();
    if let Some(orig) = read_file_safe(&backup) {
        let _ = write_file_atomic(&p, &orig);
        let _ = fs::remove_file(&backup);
    } else if exists(&p) {
        let _ = fs::remove_file(&p);
    }
    remove_owner_safe("cline", "rtk")?;
    Ok(true)
}

fn cline_rtk_verify() -> bool {
    let p = cline_rtk_hook_path();
    let hook_ok = read_file_safe(&p)
        .map(|r| r.contains(CLINE_RTK_MARKER) && r.contains("rtk-hook") && r.contains("cline"))
        .unwrap_or(false);
    hook_ok && has_owner("cline", "rtk")
}

// --- Pi Extension ---
const PI_RTK_EXTENSION: &str = r#"const TOKLESS_BIN = %TOKLESS%

export default function (pi: any) {
  pi.on("tool_call", async (event: any) => {
    if (event?.toolName !== "bash" || typeof event?.input?.command !== "string") return
    const command = event.input.command
    try {
      const result = await pi.exec(TOKLESS_BIN, ["rtk-rewrite", "--", command])
      if (result.code !== 0 || typeof result.stdout !== "string") return
      const rewritten = result.stdout.trim()
      if (!rewritten || rewritten === command) return
      event.input.command = rewritten
    } catch {}
  })
}
"#;

fn pi_rtk_path() -> String {
    Path::new(&pi_agent_dir_resolved())
        .join("extensions")
        .join("rtk.ts")
        .to_string_lossy()
        .to_string()
}

pub fn normalize_pi_rtk_extension(raw: &str) -> String {
    raw.replace("\r\n", "\n")
}

fn pi_rtk_wire(opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let p = pi_rtk_path();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let exe = tokless_persisted_abs();
    let content = PI_RTK_EXTENSION.replace("%TOKLESS%", &format!("{:?}", exe));
    write_file_atomic(&p, &content)?;
    let _ = write_owner("pi", "rtk");
    Ok(pi_rtk_verify())
}

fn pi_rtk_verify() -> bool {
    let p = pi_rtk_path();
    let ext_ok = read_file_safe(&p)
        .map(|r| r.contains("TOKLESS_BIN") && r.contains("rtk-rewrite"))
        .unwrap_or(false);
    ext_ok && has_owner("pi", "rtk")
}

// --- OpenCode Plugin ---
const OPENCODE_RTK_PLUGIN: &str = r#"import { spawn } from "node:child_process"

const TOKLESS_BIN = %TOKLESS%

export const ToklessRtkPlugin = async () => ({
  "tool.execute.before": async (input: any, output: any) => {
    if (String(input?.tool ?? "").toLowerCase() !== "bash") return
    const command = output?.args?.command
    if (typeof command !== "string") return
    const rewritten = await new Promise<string>((resolve) => {
      const child = spawn(TOKLESS_BIN, ["rtk-rewrite", "--", command], { stdio: ["ignore", "pipe", "ignore"] })
      let stdout = ""
      child.stdout.setEncoding("utf8")
      child.stdout.on("data", (chunk) => { stdout += chunk })
      child.once("error", () => resolve(""))
      child.once("close", (code) => resolve(code === 0 ? stdout.trim() : ""))
    })
    if (rewritten && rewritten !== command) output.args.command = rewritten
  },
})

export default ToklessRtkPlugin
"#;

fn opencode_rtk_path() -> String {
    Path::new(&opencode_paths_resolved().plugins_dir)
        .join("tokless-rtk.ts")
        .to_string_lossy()
        .to_string()
}

pub fn opencode_rtk_plugin_valid() -> bool {
    let p = opencode_rtk_path();
    read_file_safe(&p)
        .map(|r| r.contains("TOKLESS_BIN") && r.contains("rtk-rewrite"))
        .unwrap_or(false)
}

fn opencode_rtk_wire(opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let p = opencode_rtk_path();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let exe = tokless_persisted_abs();
    let content = OPENCODE_RTK_PLUGIN.replace("%TOKLESS%", &format!("{:?}", exe));
    write_file_atomic(&p, &content)?;
    let _ = write_owner("opencode", "rtk");
    Ok(opencode_rtk_plugin_valid() && has_owner("opencode", "rtk"))
}

// --- Amp RTK Plugin ---

const AMP_RTK_PLUGIN: &str = r#"// tokless-rtk
const TOKLESS_BIN = %TOKLESS%

export default function (amp: any) {
  amp.on("tool.call", async (event: any) => {
    const cmd = amp.helpers?.shellCommandFromToolCall?.(event)
    const command = typeof cmd === "string" ? cmd : cmd?.command
    if (typeof command !== "string" || command.length === 0) return
    try {
      const { spawnSync } = await import("node:child_process")
      const r = spawnSync(TOKLESS_BIN, ["rtk-rewrite", "--", command], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] })
      if (r.status !== 0 || typeof r.stdout !== "string") return
      const rewritten = r.stdout.trim()
      if (!rewritten || rewritten === command) return
      const key = typeof event.input?.cmd === "string" ? "cmd" : "command"
      return { action: "modify", input: { ...event.input, [key]: rewritten } }
    } catch {}
  })
}
"#;

pub fn amp_rtk_plugin_path() -> String {
    Path::new(&amp_paths_resolved().dir)
        .join("plugins")
        .join("tokless-rtk.ts")
        .to_string_lossy()
        .to_string()
}

pub fn amp_rtk_wire(opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let exe = tokless_persisted_abs();
    if exe.is_empty() {
        return Ok(false);
    }
    let p = amp_rtk_plugin_path();
    let quoted_exe = stringify_json(&Value::String(exe));
    let want = AMP_RTK_PLUGIN.replace("%TOKLESS%", &quoted_exe);
    if let Some(raw) = read_file_safe(&p) {
        if raw != want && !raw.starts_with("// tokless-rtk") {
            return Ok(false);
        }
    }
    if let Some(parent) = Path::new(&p).parent() {
        ensure_dir(parent)?;
    }
    write_file_atomic(&p, &want)?;
    Ok(amp_rtk_verify())
}

pub fn amp_rtk_verify() -> bool {
    let p = amp_rtk_plugin_path();
    let exe = tokless_persisted_abs();
    let quoted_exe = stringify_json(&Value::String(exe));
    let want = AMP_RTK_PLUGIN.replace("%TOKLESS%", &quoted_exe);
    read_file_safe(&p).map(|r| r == want).unwrap_or(false)
}

pub fn amp_rtk_unwire() -> Result<bool, DynError> {
    let p = amp_rtk_plugin_path();
    if let Some(raw) = read_file_safe(&p) {
        let exe = tokless_persisted_abs();
        let quoted_exe = stringify_json(&Value::String(exe));
        let want = AMP_RTK_PLUGIN.replace("%TOKLESS%", &quoted_exe);
        if raw == want || raw.starts_with("// tokless-rtk") {
            let _ = fs::remove_file(&p);
        }
    }
    crate::agents::amp::remove_amp_rtk_permissions();
    Ok(true)
}

// --- ZCode RTK PreToolUse Hook ---

fn zcode_rtk_hook_managed(hook: &OrderedMap) -> bool {
    let Some(Value::Array(arr)) = hook.get("args") else {
        return false;
    };
    if arr.len() != 2 {
        return false;
    }
    let a = arr.get(0).and_then(|v| v.as_str());
    let b = arr.get(1).and_then(|v| v.as_str());
    let cmd = hook.get("command").and_then(|v| v.as_str());
    if a != Some("rtk-hook") || b != Some("zcode") || cmd.is_none() {
        return false;
    }
    let base = Path::new(cmd.unwrap())
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    base == "tokless"
}

pub fn zcode_wire_rtk(opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let exe = tokless_persisted_abs();
    if exe.is_empty() {
        return Ok(false);
    }
    let p = zcode_paths_resolved().config;
    let mut cfg = read_file_safe(&p)
        .and_then(|raw| try_parse_jsonc(&raw))
        .and_then(|v| match v {
            Value::Object(m) => Some(m),
            _ => None,
        })
        .unwrap_or_default();

    let hooks_val = cfg.entry("hooks".to_string()).or_insert_with(|| Value::Object(OrderedMap::new()));
    let hooks = match hooks_val {
        Value::Object(m) => m,
        _ => return Ok(false),
    };
    hooks.insert("enabled".to_string(), Value::Bool(true));

    let events_val = hooks.entry("events".to_string()).or_insert_with(|| Value::Object(OrderedMap::new()));
    let events = match events_val {
        Value::Object(m) => m,
        _ => return Ok(false),
    };

    let pre_val = events.entry("PreToolUse".to_string()).or_insert_with(|| Value::Array(Vec::new()));
    let arr = match pre_val {
        Value::Array(a) => a,
        _ => return Ok(false),
    };

    let mut managed = false;
    for group_val in arr.iter_mut() {
        if let Value::Object(group) = group_val {
            if group.get("matcher").and_then(|v| v.as_str()) != Some("Bash") {
                continue;
            }
            if let Some(Value::Array(hook_arr)) = group.get_mut("hooks") {
                for hook_val in hook_arr.iter_mut() {
                    if let Value::Object(hook) = hook_val {
                        if zcode_rtk_hook_managed(hook) {
                            hook.insert("type".to_string(), Value::String("process".to_string()));
                            hook.insert("command".to_string(), Value::String(exe.clone()));
                            hook.insert(
                                "args".to_string(),
                                Value::Array(vec![
                                    Value::String("rtk-hook".to_string()),
                                    Value::String("zcode".to_string()),
                                ]),
                            );
                            hook.insert("enabled".to_string(), Value::Bool(true));
                            hook.insert("timeoutMs".to_string(), Value::Number(10000.into()));
                            managed = true;
                        }
                    }
                }
            }
        }
    }

    if !managed {
        let mut hook = OrderedMap::new();
        hook.insert("type".to_string(), Value::String("process".to_string()));
        hook.insert("command".to_string(), Value::String(exe));
        hook.insert(
            "args".to_string(),
            Value::Array(vec![
                Value::String("rtk-hook".to_string()),
                Value::String("zcode".to_string()),
            ]),
        );
        hook.insert("enabled".to_string(), Value::Bool(true));
        hook.insert("timeoutMs".to_string(), Value::Number(10000.into()));

        let mut group = OrderedMap::new();
        group.insert("matcher".to_string(), Value::String("Bash".to_string()));
        group.insert("hooks".to_string(), Value::Array(vec![Value::Object(hook)]));

        arr.push(Value::Object(group));
    }

    if let Some(parent) = Path::new(&p).parent() {
        ensure_dir(parent)?;
    }
    write_file_atomic(&p, &stringify_json(&Value::Object(cfg)))?;
    Ok(zcode_rtk_verify())
}

pub fn zcode_unwire_rtk() -> Result<bool, DynError> {
    let p = zcode_paths_resolved().config;
    let Some(raw) = read_file_safe(&p) else {
        return Ok(true);
    };
    let Some(Value::Object(mut cfg)) = try_parse_jsonc(&raw) else {
        return Ok(false);
    };
    let Some(Value::Object(mut hooks)) = cfg.remove("hooks") else {
        return Ok(true);
    };
    let Some(Value::Object(mut events)) = hooks.remove("events") else {
        cfg.insert("hooks".to_string(), Value::Object(hooks));
        return Ok(true);
    };
    if let Some(Value::Array(arr)) = events.remove("PreToolUse") {
        let mut kept_groups = Vec::new();
        for group_val in arr {
            if let Value::Object(mut group) = group_val {
                if let Some(Value::Array(hook_arr)) = group.remove("hooks") {
                    let kept_hooks: Vec<Value> = hook_arr
                        .into_iter()
                        .filter(|h| match h {
                            Value::Object(m) => !zcode_rtk_hook_managed(m),
                            _ => true,
                        })
                        .collect();
                    if !kept_hooks.is_empty() {
                        group.insert("hooks".to_string(), Value::Array(kept_hooks));
                        kept_groups.push(Value::Object(group));
                    }
                } else {
                    kept_groups.push(Value::Object(group));
                }
            } else {
                kept_groups.push(group_val);
            }
        }
        if !kept_groups.is_empty() {
            events.insert("PreToolUse".to_string(), Value::Array(kept_groups));
        }
    }
    if !events.is_empty() {
        hooks.insert("events".to_string(), Value::Object(events));
    }
    if !hooks.is_empty() && (hooks.len() > 1 || !hooks.contains_key("enabled")) {
        cfg.insert("hooks".to_string(), Value::Object(hooks));
    }
    write_file_atomic(&p, &stringify_json(&Value::Object(cfg)))?;
    Ok(true)
}

pub fn zcode_rtk_verify() -> bool {
    let p = zcode_paths_resolved().config;
    let Some(raw) = read_file_safe(&p) else {
        return false;
    };
    let Some(Value::Object(cfg)) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(Value::Object(hooks)) = cfg.get("hooks") else {
        return false;
    };
    if hooks.get("enabled").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }
    let Some(Value::Object(events)) = hooks.get("events") else {
        return false;
    };
    let Some(Value::Array(pre)) = events.get("PreToolUse") else {
        return false;
    };
    for group in pre {
        if let Value::Object(g) = group {
            if g.get("matcher").and_then(|v| v.as_str()) != Some("Bash") {
                continue;
            }
            if let Some(Value::Array(hook_arr)) = g.get("hooks") {
                for hook in hook_arr {
                    if let Value::Object(h) = hook {
                        if let Some(Value::Array(args)) = h.get("args") {
                            if args.len() == 2
                                && args[0].as_str() == Some("rtk-hook")
                                && args[1].as_str() == Some("zcode")
                            {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

// --- Claude PreToolUse Hook ---

pub fn remove_claude_rtk_hook_group(groups: Vec<Value>) -> Vec<Value> {
    let mut out = Vec::new();
    for g in groups {
        let gm = match g {
            Value::Object(m) => m,
            other => {
                out.push(other);
                continue;
            }
        };
        let is_bash = gm.get("matcher").and_then(|v| v.as_str()) == Some("Bash");
        if is_bash {
            let has_rtk = gm
                .get("hooks")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter().any(|h| {
                        h.as_object()
                            .and_then(|hm| hm.get("command"))
                            .and_then(|c| c.as_str())
                            .map(|s| s.contains("rtk"))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);
            if has_rtk {
                continue;
            }
        }
        out.push(Value::Object(gm));
    }
    out
}

pub fn override_claude_rtk_hook(cmd: &str) -> Result<(), DynError> {
    let cp = claude_code_paths();
    let _ = ensure_dir(&cp.dir);
    let raw = read_file_safe(&cp.settings).unwrap_or_default();
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => OrderedMap::new(),
    };

    if !matches!(cfg.get("hooks"), Some(Value::Object(_))) {
        cfg.insert("hooks".into(), Value::Object(OrderedMap::new()));
    }
    let hooks = match cfg.get_mut("hooks") {
        Some(Value::Object(m)) => m,
        _ => unreachable!(),
    };

    let existing = match hooks.get("PreToolUse") {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    let mut pre_tool_use = remove_claude_rtk_hook_group(existing);

    let mut gm = OrderedMap::new();
    gm.insert("matcher".into(), Value::String("Bash".into()));
    let mut hm = OrderedMap::new();
    hm.insert("type".into(), Value::String("command".into()));
    hm.insert("command".into(), Value::String(cmd.to_string()));
    gm.insert("hooks".into(), Value::Array(vec![Value::Object(hm)]));
    pre_tool_use.push(Value::Object(gm));

    hooks.insert("PreToolUse".into(), Value::Array(pre_tool_use));
    write_file_atomic(&cp.settings, &stringify_json(&Value::Object(cfg)))?;
    Ok(())
}

pub fn claude_rtk_hook_managed() -> bool {
    let cp = claude_code_paths();
    let raw = read_file_safe(&cp.settings).unwrap_or_default();
    raw.contains("rtk-hook") && raw.contains("claude")
}

fn claude_rtk_wire(opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let exe = tokless_persisted_abs();
    if exe.is_empty() {
        return Err("No tokless executable found".into());
    }
    let cmd = persisted_tokless_command(&exe, &["rtk-hook", "claude"]);
    override_claude_rtk_hook(&cmd)?;
    allow_claude_bash_pattern("Bash(rtk *)");
    let _ = write_owner("claude", "rtk");
    Ok(claude_rtk_hook_managed() && has_owner("claude", "rtk"))
}

fn claude_rtk_unwire(_opts: &RunOpts) -> Result<bool, DynError> {
    let cp = claude_code_paths();
    if let Some(raw) = read_file_safe(&cp.settings) {
        if let Some(Value::Object(mut cfg)) = try_parse_jsonc(&raw) {
            if let Some(Value::Object(hooks)) = cfg.get_mut("hooks") {
                if let Some(Value::Array(pre)) = hooks.get("PreToolUse").cloned() {
                    let filtered = remove_claude_rtk_hook_group(pre);
                    if filtered.is_empty() {
                        hooks.remove("PreToolUse");
                    } else {
                        hooks.insert("PreToolUse".into(), Value::Array(filtered));
                    }
                }
            }
            let _ = write_file_atomic(&cp.settings, &stringify_json(&Value::Object(cfg)));
        }
    }
    disallow_claude_bash_pattern("Bash(rtk *)");
    remove_owner_safe("claude", "rtk")?;
    Ok(true)
}

pub fn build_rtk() -> ToolManifest {
    let mut tool = ToolManifest::new("rtk", "RTK");
    tool.description = "Command output compression and formatting utility.".into();
    tool.homepage = "https://github.com/Navegos/rtk".into();
    tool.install_hint = "Prebuilt binary from GitHub releases (no Rust required).".into();
    tool.channel = Channel::GitHub;
    tool.install = Some(Arc::new(move |opts| rtk_ensure_installed(opts)));

    // Claude
    tool.wire_for.insert("claude".into(), Arc::new(|opts| claude_rtk_wire(opts)));
    tool.unwire_for.insert("claude".into(), Arc::new(|opts| claude_rtk_unwire(opts)));
    tool.verify_for.insert(
        "claude".into(),
        Arc::new(|| Some(claude_rtk_hook_managed() && has_owner("claude", "rtk"))),
    );

    // OpenCode
    tool.wire_for.insert("opencode".into(), Arc::new(|opts| opencode_rtk_wire(opts)));
    tool.unwire_for.insert(
        "opencode".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            let p = opencode_rtk_path();
            if exists(&p) {
                let _ = fs::remove_file(&p);
            }
            remove_owner_safe("opencode", "rtk")?;
            Ok(true)
        }),
    );
    tool.verify_for.insert(
        "opencode".into(),
        Arc::new(|| Some(opencode_rtk_plugin_valid() && has_owner("opencode", "rtk"))),
    );

    // Codex
    tool.wire_for.insert(
        "codex".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            let ok = install_codex_rtk_hook();
            let _ = write_owner("codex", "rtk");
            Ok(ok && has_owner("codex", "rtk"))
        }),
    );
    tool.unwire_for.insert(
        "codex".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            remove_codex_rtk_hook();
            remove_owner_safe("codex", "rtk")?;
            Ok(true)
        }),
    );
    tool.verify_for.insert(
        "codex".into(),
        Arc::new(|| Some(has_codex_rtk_hook() && has_owner("codex", "rtk"))),
    );

    // Cursor
    tool.wire_for.insert(
        "cursor".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            let _ = install_cursor_project_rules(".", false);
            Ok(install_cursor_rtk_hook() && configure_cursor_rtk_permissions())
        }),
    );
    tool.unwire_for.insert(
        "cursor".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            remove_cursor_rtk_hook();
            remove_cursor_rtk_permissions();
            let p = Path::new(&cursor_paths_resolved().rules_dir).join("rtk.mdc");
            if p.exists() {
                let _ = fs::remove_file(p);
            }
            Ok(true)
        }),
    );
    tool.verify_for.insert(
        "cursor".into(),
        Arc::new(|| {
            let rules_p = Path::new(&cursor_paths_resolved().rules_dir)
                .join("rtk.mdc")
                .to_string_lossy()
                .to_string();
            let rules_ok = if let Some(raw) = read_file_safe(&rules_p) {
                let expected = cursor_project_rule_content("rtk");
                !expected.is_empty() && raw.replace("\r\n", "\n") == expected.replace("\r\n", "\n")
            } else {
                true
            };
            Some(has_cursor_rtk_hook() && has_cursor_rtk_permissions() && rules_ok)
        }),
    );

    // Antigravity
    tool.wire_for.insert(
        "antigravity".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            let ok = install_antigravity_rtk_hook();
            allow_antigravity_entry("command(rtk)");
            allow_antigravity_entry("command(rtk )");
            let _ = write_owner("antigravity", "rtk");
            Ok(ok && has_owner("antigravity", "rtk"))
        }),
    );
    tool.unwire_for.insert(
        "antigravity".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            remove_antigravity_rtk_hook();
            remove_antigravity_entry("command(rtk)");
            remove_antigravity_entry("command(rtk )");
            remove_owner_safe("antigravity", "rtk")?;
            Ok(true)
        }),
    );
    tool.verify_for.insert(
        "antigravity".into(),
        Arc::new(|| Some(has_antigravity_rtk_hook() && has_owner("antigravity", "rtk"))),
    );

    // Copilot
    tool.wire_for.insert(
        "copilot".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            with_copilot_transaction(&[], || {
                install_copilot_rtk_hook_safe()?;
                install_copilot_ide_rtk_hook_safe()?;
                if !write_owner("copilot", "rtk") {
                    return Err("failed to write copilot rtk instructions".into());
                }
                sync_copilot_ide_instructions_safe()?;
                Ok(has_copilot_rtk_hook() && has_copilot_ide_rtk_hook() && has_owner("copilot", "rtk"))
            })
        }),
    );
    tool.unwire_for.insert(
        "copilot".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            with_copilot_transaction(&[], || {
                remove_copilot_rtk_hook_safe()?;
                remove_copilot_ide_rtk_hook_safe()?;
                remove_owner_safe("copilot", "rtk")?;
                sync_copilot_ide_instructions_safe()?;
                Ok(true)
            })
        }),
    );
    tool.verify_for.insert(
        "copilot".into(),
        Arc::new(|| Some(has_copilot_rtk_hook() && has_copilot_ide_rtk_hook() && has_owner("copilot", "rtk"))),
    );

    // Droid
    tool.wire_for.insert(
        "droid".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            let ok = install_droid_rtk_hook();
            let _ = write_owner("droid", "rtk");
            Ok(ok && has_owner("droid", "rtk"))
        }),
    );
    tool.unwire_for.insert(
        "droid".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            remove_droid_rtk_hook();
            remove_owner_safe("droid", "rtk")?;
            Ok(true)
        }),
    );
    tool.verify_for.insert(
        "droid".into(),
        Arc::new(|| Some(has_droid_rtk_hook() && has_owner("droid", "rtk"))),
    );

    // Pi
    tool.wire_for.insert("pi".into(), Arc::new(|opts| pi_rtk_wire(opts)));
    tool.unwire_for.insert(
        "pi".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            let p = pi_rtk_path();
            if exists(&p) {
                let _ = fs::remove_file(&p);
            }
            remove_owner_safe("pi", "rtk")?;
            Ok(true)
        }),
    );
    tool.verify_for.insert("pi".into(), Arc::new(|| Some(pi_rtk_verify())));

    // OMP
    tool.wire_for.insert(
        "omp".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            let ok = write_omp_rtk_extension();
            let _ = write_owner("omp", "rtk");
            Ok(ok && has_owner("omp", "rtk"))
        }),
    );
    tool.unwire_for.insert(
        "omp".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            let p = omp_rtk_extension_path();
            if exists(&p) {
                let _ = fs::remove_file(&p);
            }
            remove_owner_safe("omp", "rtk")?;
            Ok(true)
        }),
    );
    tool.verify_for.insert(
        "omp".into(),
        Arc::new(|| Some(omp_rtk_extension_valid() && has_owner("omp", "rtk"))),
    );

    // Kilo
    tool.wire_for.insert("kilo".into(), Arc::new(|opts| kilo_rtk_wire(opts)));
    tool.unwire_for.insert("kilo".into(), Arc::new(|opts| kilo_rtk_unwire(opts)));
    tool.verify_for.insert("kilo".into(), Arc::new(|| Some(kilo_rtk_verify())));

    // Cline
    tool.wire_for.insert("cline".into(), Arc::new(|opts| cline_rtk_wire(opts)));
    tool.unwire_for.insert("cline".into(), Arc::new(|opts| cline_rtk_unwire(opts)));
    tool.verify_for.insert("cline".into(), Arc::new(|| Some(cline_rtk_verify())));

    // Grok
    tool.wire_for.insert(
        "grok".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            install_grok_rtk_hook()?;
            install_grok_codegraph_session_hook()?;
            let _ = write_owner("grok", "rtk");
            Ok(has_grok_rtk_hook() && has_owner("grok", "rtk"))
        }),
    );
    tool.unwire_for.insert(
        "grok".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            remove_grok_rtk_hook();
            remove_grok_codegraph_session_hook();
            remove_owner_safe("grok", "rtk")?;
            Ok(true)
        }),
    );
    tool.verify_for.insert(
        "grok".into(),
        Arc::new(|| Some(has_grok_rtk_hook() && has_owner("grok", "rtk"))),
    );

    // Amp
    tool.wire_for.insert("amp".into(), Arc::new(|opts| amp_rtk_wire(opts)));
    tool.unwire_for.insert("amp".into(), Arc::new(|_opts| amp_rtk_unwire()));
    tool.verify_for.insert("amp".into(), Arc::new(|| Some(amp_rtk_verify())));

    // Dsh
    tool.wire_for.insert(
        "dsh".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            let exe = tokless_persisted_abs();
            let (changed, _, err) = crate::agents::dsh::configure_dsh_rtk(&exe);
            if let Some(e) = err {
                return Err(e.into());
            }
            Ok(changed || crate::agents::dsh::dsh_rtk_wired())
        }),
    );
    tool.unwire_for.insert(
        "dsh".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            let _ = crate::agents::dsh::remove_dsh_rtk();
            Ok(true)
        }),
    );
    tool.verify_for.insert(
        "dsh".into(),
        Arc::new(|| Some(crate::agents::dsh::dsh_rtk_wired())),
    );

    // ZCode
    tool.wire_for.insert("zcode".into(), Arc::new(|opts| zcode_wire_rtk(opts)));
    tool.unwire_for.insert("zcode".into(), Arc::new(|_opts| zcode_unwire_rtk()));
    tool.verify_for.insert("zcode".into(), Arc::new(|| Some(zcode_rtk_verify())));

    tool
}
