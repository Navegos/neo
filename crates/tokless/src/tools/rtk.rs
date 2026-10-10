use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::agents::*;
use crate::core::{Channel, DynError, RunOpts, ToolManifest};
use crate::util::exec::run_cmd;
use crate::util::jsonc::{try_parse_jsonc, stringify_json, OrderedMap, Value};
use crate::util::paths::*;

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
    if env::var("TOKLESS_TEST").as_deref() == Ok("1") {
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
        let res = run_cmd("sh", &["-c", "curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/master/install.sh | sh"], &[]);
        if res.code == 0 {
            return Ok(true);
        }
    }

    if !which("cargo").is_empty() {
        let res = run_cmd("cargo", &["install", "--git", "https://github.com/rtk-ai/rtk"], &[]);
        if res.code == 0 {
            return Ok(true);
        }
    }

    Err("Cannot install rtk on this platform. See https://github.com/rtk-ai/rtk for manual install.".into())
}

fn rtk_install_prebuilt(asset: &str, opts: &RunOpts) -> bool {
    let url = format!("https://github.com/rtk-ai/rtk/releases/latest/download/{}", asset);
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
            url, zip_dest, zip_dest, dest.to_string_lossy(), zip_dest
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
    Path::new(&omp_agent_dir_resolved()).join("extensions").join("tokless-rtk.ts").to_string_lossy().to_string()
}

fn write_omp_rtk_extension() -> bool {
    let p = omp_rtk_extension_path();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let exe = tokless_persisted_abs();
    let quoted_exe = format!("{:?}", exe);
    let content = OMP_RTK_EXTENSION.replace("%TOKLESS%", &quoted_exe);
    write_file(&p, &content).is_ok()
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
    Path::new(&kilo_paths_resolved().plugins_dir).join("rtk.ts").to_string_lossy().to_string()
}

fn kilo_rtk_wire(opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let p = kilo_rtk_path();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let exe = tokless_persisted_abs();
    let quoted = format!("{:?}", exe);
    let content = KILO_RTK_PLUGIN.replace("%RTK%", &quoted);
    write_file(&p, &content)?;
    Ok(true)
}

fn kilo_rtk_unwire(_opts: &RunOpts) -> Result<bool, DynError> {
    let p = kilo_rtk_path();
    if exists(&p) {
        let _ = fs::remove_file(&p);
    }
    Ok(true)
}

// --- Cline Hook ---
const CLINE_RTK_MARKER: &str = "tokless-cline-rtk-v1";

fn cline_rtk_hook_path() -> String {
    let name = if is_win() { "PreToolUse.cjs" } else { "PreToolUse" };
    Path::new(&cline_paths_resolved().hooks_dir).join(name).to_string_lossy().to_string()
}

fn cline_rtk_wire(opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let p = cline_rtk_hook_path();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let exe = tokless_persisted_abs();
    let content = if is_win() {
        format!(
            "#!/usr/bin/env node\n// {}\nconst {{ spawnSync }} = require(\"child_process\");\nconst r = spawnSync({:?}, [\"rtk-hook\", \"cline\"], {{ stdio: \"inherit\" }});\nprocess.exit(typeof r.status === \"number\" ? r.status : 0);\n",
            CLINE_RTK_MARKER, exe
        )
    } else {
        format!("#!/bin/sh\n# {}\nexec {:?} rtk-hook cline\n", CLINE_RTK_MARKER, exe)
    };
    write_file(&p, &content)?;
    Ok(true)
}

fn cline_rtk_unwire(_opts: &RunOpts) -> Result<bool, DynError> {
    let p = cline_rtk_hook_path();
    if exists(&p) {
        let _ = fs::remove_file(&p);
    }
    Ok(true)
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

fn pi_rtk_wire(opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let p = Path::new(&pi_agent_dir_resolved()).join("extensions").join("rtk.ts").to_string_lossy().to_string();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let exe = tokless_persisted_abs();
    let content = PI_RTK_EXTENSION.replace("%TOKLESS%", &format!("{:?}", exe));
    write_file(&p, &content)?;
    Ok(true)
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

fn opencode_rtk_wire(opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let p = Path::new(&opencode_paths_resolved().plugins_dir).join("tokless-rtk.ts").to_string_lossy().to_string();
    let _ = ensure_dir(Path::new(&p).parent().unwrap_or(Path::new("")));
    let exe = tokless_persisted_abs();
    let content = OPENCODE_RTK_PLUGIN.replace("%TOKLESS%", &format!("{:?}", exe));
    write_file(&p, &content)?;
    Ok(true)
}

// --- Claude PreToolUse Hook ---
fn claude_rtk_wire(opts: &RunOpts) -> Result<bool, DynError> {
    if opts.dry_run {
        return Ok(true);
    }
    let exe = tokless_persisted_abs();
    if exe.is_empty() {
        return Err("No tokless executable found".into());
    }
    let cmd = persisted_tokless_command(&exe, &["rtk-hook", "claude"]);
    let cp = claude_code_paths();
    let _ = ensure_dir(&cp.dir);
    let raw = read_file_safe(&cp.settings).unwrap_or_default();
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => OrderedMap::new(),
    };

    let hooks = match cfg.get_mut("hooks") {
        Some(Value::Object(m)) => m,
        _ => {
            cfg.insert("hooks".into(), Value::Object(OrderedMap::new()));
            match cfg.get_mut("hooks") {
                Some(Value::Object(m)) => m,
                _ => unreachable!(),
            }
        }
    };

    let mut pre_tool_use = match hooks.get("PreToolUse") {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };

    let mut managed = false;
    for group in &mut pre_tool_use {
        if let Value::Object(gm) = group {
            if let Some(Value::String(m)) = gm.get("matcher") {
                if m == "Bash" {
                    if let Some(Value::Array(hook_arr)) = gm.get_mut("hooks") {
                        for h in hook_arr {
                            if let Value::Object(hm) = h {
                                hm.insert("command".into(), Value::String(cmd.clone()));
                                managed = true;
                            }
                        }
                    }
                }
            }
        }
    }

    if !managed {
        let mut gm = OrderedMap::new();
        gm.insert("matcher".into(), Value::String("Bash".into()));
        let mut hm = OrderedMap::new();
        hm.insert("type".into(), Value::String("command".into()));
        hm.insert("command".into(), Value::String(cmd));
        gm.insert("hooks".into(), Value::Array(vec![Value::Object(hm)]));
        pre_tool_use.push(Value::Object(gm));
    }

    hooks.insert("PreToolUse".into(), Value::Array(pre_tool_use));
    write_file(&cp.settings, &stringify_json(&Value::Object(cfg)))?;
    allow_claude_bash_pattern("Bash(rtk *)");
    Ok(true)
}

fn claude_rtk_unwire(_opts: &RunOpts) -> Result<bool, DynError> {
    let cp = claude_code_paths();
    let raw = match read_file_safe(&cp.settings) {
        Some(r) => r,
        None => return Ok(true),
    };
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => return Ok(true),
    };
    if let Some(Value::Object(hooks)) = cfg.get_mut("hooks") {
        if let Some(Value::Array(pre_tool_use)) = hooks.get_mut("PreToolUse") {
            pre_tool_use.retain(|group| {
                if let Value::Object(gm) = group {
                    if let Some(Value::String(m)) = gm.get("matcher") {
                        return m != "Bash";
                    }
                }
                true
            });
        }
    }
    let _ = write_file(&cp.settings, &stringify_json(&Value::Object(cfg)));
    disallow_claude_bash_pattern("Bash(rtk *)");
    Ok(true)
}

pub fn build_rtk() -> ToolManifest {
    let mut tool = ToolManifest::new("rtk", "RTK");
    tool.description = "Command output compression and formatting utility.".into();
    tool.homepage = "https://github.com/rtk-ai/rtk".into();
    tool.install_hint = "Prebuilt binary from GitHub releases (no Rust required).".into();
    tool.channel = Channel::GitHub;
    tool.install = Some(Arc::new(move |opts| rtk_ensure_installed(opts)));

    // Wiring for agents
    tool.wire_for.insert("claude".into(), Arc::new(|opts| claude_rtk_wire(opts)));
    tool.unwire_for.insert("claude".into(), Arc::new(|opts| claude_rtk_unwire(opts)));
    tool.verify_for.insert("claude".into(), Arc::new(|| {
        let cp = claude_code_paths();
        let raw = read_file_safe(&cp.settings).unwrap_or_default();
        Some(raw.contains("rtk-hook") && raw.contains("claude"))
    }));

    tool.wire_for.insert("opencode".into(), Arc::new(|opts| opencode_rtk_wire(opts)));
    tool.unwire_for.insert("opencode".into(), Arc::new(|_opts| {
        let p = Path::new(&opencode_paths_resolved().plugins_dir).join("tokless-rtk.ts").to_string_lossy().to_string();
        if exists(&p) { let _ = fs::remove_file(&p); }
        Ok(true)
    }));
    tool.verify_for.insert("opencode".into(), Arc::new(|| {
        let p = Path::new(&opencode_paths_resolved().plugins_dir).join("tokless-rtk.ts").to_string_lossy().to_string();
        Some(exists(&p))
    }));

    tool.wire_for.insert("codex".into(), Arc::new(|_opts| Ok(install_codex_rtk_hook())));
    tool.unwire_for.insert("codex".into(), Arc::new(|_opts| { remove_codex_rtk_hook(); Ok(true) }));
    tool.verify_for.insert("codex".into(), Arc::new(|| Some(has_codex_rtk_hook())));

    tool.wire_for.insert("cursor".into(), Arc::new(|opts| {
        if opts.dry_run { return Ok(true); }
        Ok(install_cursor_rtk_hook() && configure_cursor_rtk_permissions())
    }));
    tool.unwire_for.insert("cursor".into(), Arc::new(|_opts| {
        remove_cursor_rtk_hook();
        remove_cursor_rtk_permissions();
        Ok(true)
    }));
    tool.verify_for.insert("cursor".into(), Arc::new(|| Some(has_cursor_rtk_hook() && has_cursor_rtk_permissions())));

    tool.wire_for.insert("antigravity".into(), Arc::new(|opts| {
        if opts.dry_run { return Ok(true); }
        Ok(install_antigravity_rtk_hook())
    }));
    tool.unwire_for.insert("antigravity".into(), Arc::new(|_opts| {
        remove_antigravity_rtk_hook();
        remove_antigravity_entry("command(rtk)");
        remove_antigravity_entry("command(rtk )");
        Ok(true)
    }));
    tool.verify_for.insert("antigravity".into(), Arc::new(|| Some(has_antigravity_rtk_hook())));

    tool.wire_for.insert("copilot".into(), Arc::new(|opts| {
        if opts.dry_run { return Ok(true); }
        install_copilot_rtk_hook_safe()?;
        install_copilot_ide_rtk_hook_safe()?;
        Ok(has_copilot_rtk_hook() && has_copilot_ide_rtk_hook())
    }));
    tool.unwire_for.insert("copilot".into(), Arc::new(|_opts| {
        remove_copilot_rtk_hook_safe()?;
        remove_copilot_ide_rtk_hook_safe()?;
        Ok(true)
    }));
    tool.verify_for.insert("copilot".into(), Arc::new(|| Some(has_copilot_rtk_hook() && has_copilot_ide_rtk_hook())));

    tool.wire_for.insert("droid".into(), Arc::new(|opts| {
        if opts.dry_run { return Ok(true); }
        Ok(install_droid_rtk_hook())
    }));
    tool.unwire_for.insert("droid".into(), Arc::new(|_opts| { remove_droid_rtk_hook(); Ok(true) }));
    tool.verify_for.insert("droid".into(), Arc::new(|| Some(has_droid_rtk_hook())));

    tool.wire_for.insert("pi".into(), Arc::new(|opts| pi_rtk_wire(opts)));
    tool.unwire_for.insert("pi".into(), Arc::new(|_opts| {
        let p = Path::new(&pi_agent_dir_resolved()).join("extensions").join("rtk.ts").to_string_lossy().to_string();
        if exists(&p) { let _ = fs::remove_file(&p); }
        Ok(true)
    }));
    tool.verify_for.insert("pi".into(), Arc::new(|| Some(has_pi_rtk_extension())));

    tool.wire_for.insert("omp".into(), Arc::new(|opts| {
        if opts.dry_run { return Ok(true); }
        Ok(write_omp_rtk_extension())
    }));
    tool.unwire_for.insert("omp".into(), Arc::new(|_opts| {
        let p = omp_rtk_extension_path();
        if exists(&p) { let _ = fs::remove_file(&p); }
        Ok(true)
    }));
    tool.verify_for.insert("omp".into(), Arc::new(|| Some(has_omp_rtk_extension())));

    tool.wire_for.insert("kilo".into(), Arc::new(|opts| kilo_rtk_wire(opts)));
    tool.unwire_for.insert("kilo".into(), Arc::new(|opts| kilo_rtk_unwire(opts)));
    tool.verify_for.insert("kilo".into(), Arc::new(|| {
        let p = kilo_rtk_path();
        Some(exists(&p))
    }));

    tool.wire_for.insert("cline".into(), Arc::new(|opts| cline_rtk_wire(opts)));
    tool.unwire_for.insert("cline".into(), Arc::new(|opts| cline_rtk_unwire(opts)));
    tool.verify_for.insert("cline".into(), Arc::new(|| {
        let p = cline_rtk_hook_path();
        Some(exists(&p))
    }));

    tool.wire_for.insert("grok".into(), Arc::new(|opts| {
        if opts.dry_run { return Ok(true); }
        install_grok_rtk_hook()?;
        install_grok_codegraph_session_hook()?;
        Ok(has_grok_rtk_hook() && has_grok_codegraph_session_hook())
    }));
    tool.unwire_for.insert("grok".into(), Arc::new(|_opts| {
        remove_grok_rtk_hook();
        remove_grok_codegraph_session_hook();
        Ok(true)
    }));
    tool.verify_for.insert("grok".into(), Arc::new(|| Some(has_grok_rtk_hook())));

    tool
}
