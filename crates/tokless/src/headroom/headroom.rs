use std::borrow::Borrow;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::core::RunOpts;
use crate::util::exec::ExecResult;
use crate::util::headroom::{
    headroom_bin, headroom_env, headroom_installed, headroom_paths_resolved,
    headroom_uv_bootstrap_env,
};
use crate::util::logger::L;
use crate::util::paths::{ensure_dir, exists, which, IS_WIN};

pub const HEADROOM_PACKAGE: &str = "headroom-ai[proxy]";
pub const HEADROOM_VERSION_TIMEOUT: Duration = Duration::from_secs(30);

pub fn run_headroom(
    command: &str,
    args: &[&str],
    env: &[(String, String)],
    _timeout: Duration,
) -> ExecResult {
    let mut cmd = Command::new(command);
    cmd.args(args);
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    match cmd.output() {
        Ok(output) => ExecResult {
            code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        },
        Err(err) => ExecResult {
            code: 127,
            stdout: String::new(),
            stderr: err.to_string(),
        },
    }
}

pub fn headroom_install_args(upgrade: bool) -> Vec<String> {
    let mut args = vec!["tool".to_string(), "install".to_string()];
    if upgrade {
        args.push("--upgrade".to_string());
    }
    args.push("--python".to_string());
    args.push("3.14".to_string());
    args.push(HEADROOM_PACKAGE.to_string());
    args
}

pub fn headroom_python_install_args() -> Vec<String> {
    vec![
        "python".to_string(),
        "install".to_string(),
        "3.14".to_string(),
    ]
}

pub fn headroom_native_build_risk() -> bool {
    headroom_native_build_risk_for(std::env::consts::OS, std::env::consts::ARCH)
}

pub fn headroom_native_build_risk_for(goos: &str, goarch: &str) -> bool {
    goos == "windows"
        || ((goos == "darwin" || goos == "macos") && (goarch == "amd64" || goarch == "x86_64"))
}

pub fn headroom_failure(
    stage: &str,
    result: &ExecResult,
) -> Box<dyn std::error::Error + Send + Sync> {
    headroom_failure_for(stage, result, headroom_native_build_risk())
}

pub fn headroom_failure_for(
    stage: &str,
    result: &ExecResult,
    native_risk: bool,
) -> Box<dyn std::error::Error + Send + Sync> {
    let mut hint = String::new();
    if native_risk {
        let mut manual = vec![headroom_paths_resolved().uv];
        manual.extend(headroom_install_args(false));
        hint = format!(
            " Rust/native toolchain may be required on this platform. Manual: {}",
            manual.join(" ")
        );
    }
    let mut detail = result.stderr.trim().to_string();
    if detail.is_empty() {
        detail = result.stdout.trim().to_string();
    }
    if !detail.is_empty() {
        let first_line = detail.split('\n').next().unwrap_or("");
        detail = format!(": {}", first_line);
    }
    format!("headroom {} failed{}{}", stage, detail, hint).into()
}

/// Picks the official Astral uv installer for the OS.
/// Unix: curl preferred, wget fallback. Windows: powershell irm|iex.
/// UV_INSTALL_DIR / UV_NO_MODIFY_PATH must already be in the process env (see headroom_uv_bootstrap_env).
pub fn headroom_uv_bootstrap_cmd(
    windows: bool,
    have_curl: bool,
    have_wget: bool,
) -> Result<(String, Vec<String>), Box<dyn std::error::Error + Send + Sync>> {
    if windows {
        return Ok((
            "powershell".to_string(),
            vec![
                "-NoProfile".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-Command".to_string(),
                "irm https://astral.sh/uv/install.ps1 | iex".to_string(),
            ],
        ));
    }
    if have_curl {
        return Ok((
            "sh".to_string(),
            vec![
                "-c".to_string(),
                "curl -LsSf https://astral.sh/uv/install.sh | sh".to_string(),
            ],
        ));
    }
    if have_wget {
        return Ok((
            "sh".to_string(),
            vec![
                "-c".to_string(),
                "wget -qO- https://astral.sh/uv/install.sh | sh".to_string(),
            ],
        ));
    }
    Err("need curl or wget to bootstrap uv".into())
}

pub fn headroom_uv() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let p = headroom_paths_resolved();
    if exists(&p.uv) && headroom_uv_works(&p.uv) {
        return Ok(p.uv);
    }
    let system = which("uv");
    if !system.is_empty() && headroom_uv_works(&system) {
        return Ok(system);
    }
    if let Some(parent) = Path::new(&p.uv).parent() {
        ensure_dir(parent)?;
    }
    let (command, args) = headroom_uv_bootstrap_cmd(
        IS_WIN,
        !which("curl").is_empty(),
        !which("wget").is_empty(),
    )?;
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let env = headroom_uv_bootstrap_env();
    let result = run_headroom(&command, &arg_refs, &env, Duration::from_secs(120));
    if result.code != 0 || !exists(&p.uv) || !headroom_uv_works(&p.uv) {
        return Err(headroom_failure("uv bootstrap", &result));
    }
    Ok(p.uv)
}

pub fn headroom_uv_works(uv: &str) -> bool {
    run_headroom(uv, &["--version"], &headroom_env(), Duration::from_secs(10)).code == 0
}

pub fn headroom_version_probe() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bin = headroom_bin();
    let result = run_headroom(&bin, &["--version"], &[], HEADROOM_VERSION_TIMEOUT);
    if result.code != 0 {
        return Err(headroom_failure("executable verification", &result));
    }
    Ok(())
}

pub fn ensure_installed<O: Borrow<RunOpts>>(
    opts: O,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let opts = opts.borrow();
    if std::env::var("TOKLESS_TEST").unwrap_or_default() == "1" {
        return Ok(true);
    }
    if opts.dry_run {
        L.sub("[dry-run] would install headroom-ai[proxy] with managed Python 3.14");
        return Ok(true);
    }
    let uv = headroom_uv()?;
    if headroom_installed() && !opts.upgrade {
        headroom_version_probe()?;
        return Ok(true);
    }
    let py_args_owned = headroom_python_install_args();
    let py_args: Vec<&str> = py_args_owned.iter().map(|s| s.as_str()).collect();
    let env = headroom_env();
    let py_result = run_headroom(&uv, &py_args, &env, Duration::from_secs(600));
    if py_result.code != 0 {
        return Err(headroom_failure("managed Python", &py_result));
    }
    let install_args_owned = headroom_install_args(opts.upgrade);
    let install_args: Vec<&str> = install_args_owned.iter().map(|s| s.as_str()).collect();
    let result = run_headroom(&uv, &install_args, &env, Duration::from_secs(600));
    if result.code != 0 {
        return Err(headroom_failure("package install", &result));
    }
    if !headroom_installed() {
        return Err(format!("headroom executable verification failed: {}", headroom_bin()).into());
    }
    headroom_version_probe()?;
    Ok(true)
}
