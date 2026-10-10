use std::path::Path;
use std::time::Duration;
use serde::{Deserialize, Serialize};

use crate::util::paths::{read_file_safe, tokless_data_dir, write_file_atomic};
use crate::util::headroom::{headroom_bin, headroom_installed};
use crate::util::headroom_proxy::{headroom_proxy_port, headroom_proxy_url};

const PROXY_READY_TIMEOUT: Duration = Duration::from_secs(15);
const PROXY_POLL_INTERVAL: Duration = Duration::from_millis(300);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyOwnership {
    pub pid: u32,
    pub executable: String,
    pub args: Vec<String>,
    pub start_fingerprint: String,
}

pub fn proxy_port() -> u16 {
    headroom_proxy_port()
}

pub fn proxy_url() -> String {
    headroom_proxy_url()
}

pub fn proxy_ownership_file() -> String {
    Path::new(&tokless_data_dir())
        .join("proxy.json")
        .to_string_lossy()
        .to_string()
}

pub fn proxy_live_z(port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/livez", port);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build();
    if let Ok(c) = client {
        if let Ok(res) = c.get(&url).send() {
            return res.status().is_success();
        }
    }
    false
}

pub fn proxy_running() -> bool {
    proxy_live_z(proxy_port())
}

pub fn ensure_proxy_up() {
    if proxy_running() {
        return;
    }
    let _ = start_proxy();
}

pub fn ensure_copilot_proxy_up() {
    ensure_proxy_up();
}

pub fn start_proxy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if proxy_running() {
        return Ok(());
    }
    if !headroom_installed() {
        return Err("headroom is not installed".into());
    }

    let bin = headroom_bin();
    let port = proxy_port().to_string();
    let mut cmd = std::process::Command::new(&bin);
    cmd.args(["proxy", "--port", &port]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW | DETACHED_PROCESS
        cmd.creation_flags(0x08000000 | 0x00000008);
    }

    let child = cmd.spawn()?;
    let pid = child.id();

    let ownership = ProxyOwnership {
        pid,
        executable: bin.clone(),
        args: vec!["proxy".to_string(), "--port".to_string(), port],
        start_fingerprint: chrono::Utc::now().to_rfc3339(),
    };
    if let Ok(json) = serde_json::to_string(&ownership) {
        let _ = write_file_atomic(proxy_ownership_file(), &json);
    }

    // Poll until ready
    let start = std::time::Instant::now();
    while start.elapsed() < PROXY_READY_TIMEOUT {
        if proxy_live_z(proxy_port()) {
            return Ok(());
        }
        std::thread::sleep(PROXY_POLL_INTERVAL);
    }

    Err("proxy started but did not become ready in time".into())
}

pub fn stop_proxy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let p_file = proxy_ownership_file();
    if let Some(raw) = read_file_safe(&p_file) {
        if let Ok(ownership) = serde_json::from_str::<ProxyOwnership>(&raw) {
            if ownership.pid > 0 {
                #[cfg(windows)]
                {
                    let _ = std::process::Command::new("taskkill")
                        .args(["/F", "/PID", &ownership.pid.to_string()])
                        .output();
                }
                #[cfg(unix)]
                {
                    unsafe {
                        libc::kill(ownership.pid as i32, libc::SIGTERM);
                    }
                }
            }
        }
        let _ = std::fs::remove_file(&p_file);
    }
    Ok(())
}

pub fn run_proxy_foreground() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bin = headroom_bin();
    let port = proxy_port().to_string();
    let mut child = std::process::Command::new(&bin)
        .args(["proxy", "--port", &port])
        .spawn()?;
    let _ = child.wait();
    Ok(())
}

pub fn run_proxy_watchdog() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        if !proxy_running() {
            let _ = start_proxy();
        }
        std::thread::sleep(Duration::from_secs(5));
    }
}

