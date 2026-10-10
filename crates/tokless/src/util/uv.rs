use std::path::Path;
use super::paths::{home, which, stable_executable, IS_WIN};
use super::exec::{run, RunOptions};

pub const UV_INSTALL_DOCS: &str = "https://docs.astral.sh/uv/getting-started/installation/";

pub fn uv_candidate_paths() -> Vec<String> {
    let h = home();
    if IS_WIN {
        let mut paths = vec![
            Path::new(&h).join(".local").join("bin").join("uv.exe").to_string_lossy().to_string()
        ];
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            paths.push(Path::new(&local).join("uv").join("uv.exe").to_string_lossy().to_string());
        }
        return paths;
    }
    vec![
        Path::new(&h).join(".local").join("bin").join("uv").to_string_lossy().to_string(),
        Path::new(&h).join(".cargo").join("bin").join("uv").to_string_lossy().to_string(),
    ]
}

pub fn resolve_uv() -> String {
    let p = which("uv");
    if !p.is_empty() {
        return p;
    }
    for c in uv_candidate_paths() {
        if stable_executable(&c) {
            return c;
        }
    }
    String::new()
}

pub fn ensure_uv() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let p = resolve_uv();
    if !p.is_empty() {
        return Ok(p);
    }
    if std::env::var("TOKLESS_TEST").unwrap_or_default() == "1" {
        return Err(format!("uv not found on PATH; install uv: {}", UV_INSTALL_DOCS).into());
    }

    if IS_WIN {
        let ps = "irm https://astral.sh/uv/install.ps1 | iex";
        let res = run("powershell", &["-NoProfile", "-ExecutionPolicy", "Bypass", "-c", ps], RunOptions::default());
        if res.code != 0 {
            return Err(format!("uv install failed: {}; install uv manually: {}", res.stderr, UV_INSTALL_DOCS).into());
        }
    } else {
        let cmd = "curl -LsSf https://astral.sh/uv/install.sh | sh";
        let res = run("sh", &["-c", cmd], RunOptions::default());
        if res.code != 0 {
            return Err(format!("uv install failed: {}; install uv manually: {}", res.stderr, UV_INSTALL_DOCS).into());
        }
    }

    let p = resolve_uv();
    if !p.is_empty() {
        return Ok(p);
    }
    Err(format!("uv install finished but uv not found on PATH; install uv manually: {}", UV_INSTALL_DOCS).into())
}
