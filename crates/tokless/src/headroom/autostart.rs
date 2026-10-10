use crate::util::paths::{tokless_abs_strict, which, IS_WIN};
use crate::util::exec::{run, RunOptions};

pub use super::proxy::{ensure_proxy_up, start_proxy, stop_proxy};

pub fn grok_oauth_proxy_owned() -> bool {
    false
}

const PROXY_AUTOSTART_TASK: &str = "tokless-headroom-proxy";

pub fn enable_proxy_autostart() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bin = tokless_abs_strict();
    if bin.is_empty() {
        return Ok(());
    }
    if IS_WIN {
        if which("schtasks").is_empty() {
            return Err("schtasks not found".into());
        }
        let tr = format!("\"{}\" __proxy-watch", bin);
        let res = run(
            "schtasks",
            &["/create", "/tn", PROXY_AUTOSTART_TASK, "/tr", &tr, "/sc", "ONLOGON", "/rl", "LIMITED", "/f"],
            RunOptions::default(),
        );
        if res.code != 0 {
            return Err(format!("schtasks create failed: {}", res.stderr).into());
        }
        let _ = run("schtasks", &["/run", "/tn", PROXY_AUTOSTART_TASK], RunOptions::default());
    }
    Ok(())
}

pub fn disable_proxy_autostart() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if IS_WIN && !which("schtasks").is_empty() {
        let _ = run("schtasks", &["/end", "/tn", PROXY_AUTOSTART_TASK], RunOptions::default());
        let _ = run("schtasks", &["/delete", "/tn", PROXY_AUTOSTART_TASK, "/f"], RunOptions::default());
    }
    Ok(())
}

pub fn proxy_autostart_configured() -> bool {
    if IS_WIN && !which("schtasks").is_empty() {
        let res = run("schtasks", &["/query", "/tn", PROXY_AUTOSTART_TASK], RunOptions { capture: true, ..Default::default() });
        return res.code == 0;
    }
    false
}

pub fn proxy_autostart_enabled() -> bool {
    proxy_autostart_configured() && super::proxy::proxy_running()
}
