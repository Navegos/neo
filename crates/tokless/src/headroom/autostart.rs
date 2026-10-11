use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub use super::grok_daemon::grok_oauth_proxy_owned;
pub use super::proxy::{ensure_proxy_up, start_proxy, stop_proxy};

use super::proxy::{
    proxy_args_match_recorded, proxy_port_from_runtime, proxy_running,
    proxy_supervised_args_match, restore_proxy_after_autostart_rollback,
    stop_headroom_daemon_for_handoff, PROXY_POLL_INTERVAL, PROXY_READY_TIMEOUT,
};
use super::watchdog::{clear_proxy_stop_request, request_proxy_stop};
use crate::util::headroom::headroom_paths_resolved;
use crate::util::paths::{
    ensure_dir, exists, home, is_test_executable, read_file_safe, tokless_abs, tokless_abs_strict,
    which, write_file,
};

pub const ERR_PROXY_AUTOSTART_UNAVAILABLE: &str = "proxy autostart unavailable";
pub const PROXY_AUTOSTART_LABEL: &str = "com.tokless.headroom.proxy";
pub const PROXY_AUTOSTART_UNIT: &str = "tokless-proxy.service";
pub const BYOK_GATEWAY_AUTOSTART_UNIT: &str = "tokless-byok-gateway.service";
pub const PROXY_AUTOSTART_TASK: &str = "tokless-headroom-proxy";

// ---------------------------------------------------------------------------
// macOS LaunchAgents
// ---------------------------------------------------------------------------

pub fn proxy_autostart_plist_path() -> String {
    Path::new(&home())
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{}.plist", PROXY_AUTOSTART_LABEL))
        .to_string_lossy()
        .to_string()
}

pub fn proxy_autostart_log_path() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("proxy.log")
        .to_string_lossy()
        .to_string()
}

pub fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&#34;"),
            '\'' => out.push_str("&#39;"),
            '\t' => out.push_str("&#x9;"),
            '\n' => out.push_str("&#xA;"),
            '\r' => out.push_str("&#xD;"),
            _ => out.push(c),
        }
    }
    out
}

fn xml_unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#34;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&#x9;", "\t")
        .replace("&#xA;", "\n")
        .replace("&#xD;", "\r")
        .replace("&amp;", "&")
}

pub fn proxy_autostart_plist_body(tokless_bin: &str) -> String {
    let log_path = proxy_autostart_log_path();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{}</string>
    <key>Comment</key>
    <string>tokless-managed</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>__proxy-run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{}</string>
    <key>StandardErrorPath</key>
    <string>{}</string>
</dict>
</plist>
"#,
        xml_escape(PROXY_AUTOSTART_LABEL),
        xml_escape(tokless_bin),
        xml_escape(&log_path),
        xml_escape(&log_path)
    )
}

pub fn mac_current_uid() -> String {
    #[cfg(unix)]
    {
        unsafe { libc::getuid() }.to_string()
    }
    #[cfg(not(unix))]
    {
        "501".to_string()
    }
}

pub fn launchctl_domain() -> String {
    format!("gui/{}", mac_current_uid())
}

pub fn run_launchctl(args: &[&str]) -> Result<Vec<u8>, (Vec<u8>, String)> {
    match Command::new("launchctl").args(args).output() {
        Ok(out) => {
            let mut combined = out.stdout;
            combined.extend_from_slice(&out.stderr);
            if out.status.success() {
                Ok(combined)
            } else {
                let msg = String::from_utf8_lossy(&combined).trim().to_string();
                Err((combined, msg))
            }
        }
        Err(e) => Err((Vec::new(), e.to_string())),
    }
}

pub fn launchctl_service_not_found(output: &[u8]) -> bool {
    let s = String::from_utf8_lossy(output).to_lowercase();
    s.contains("could not find service")
        || s.contains("service not found")
        || s.contains("no such process")
}

pub fn bootout_proxy_agent() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let dom = launchctl_domain();
    match run_launchctl(&["bootout", &dom, PROXY_AUTOSTART_LABEL]) {
        Ok(_) => Ok(()),
        Err((output, err_msg)) => {
            let target = format!("{}/{}", dom, PROXY_AUTOSTART_LABEL);
            if run_launchctl(&["print", &target]).is_ok() {
                return Err(format!("bootout {}: {}", PROXY_AUTOSTART_LABEL, err_msg).into());
            }
            if launchctl_service_not_found(&output) {
                return Ok(());
            }
            Err(format!("bootout {}: {}", PROXY_AUTOSTART_LABEL, err_msg).into())
        }
    }
}

pub fn launchctl_service_enabled() -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let dom = launchctl_domain();
    match run_launchctl(&["print-disabled", &dom]) {
        Ok(out) => {
            let needle = format!("\"{}\" => true", PROXY_AUTOSTART_LABEL);
            let s = String::from_utf8_lossy(&out);
            Ok(!s.contains(&needle))
        }
        Err((_, err)) => Err(format!("query disabled launch agents: {}", err).into()),
    }
}

fn enable_proxy_autostart_darwin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bin = tokless_abs();
    if bin.is_empty() || is_test_executable(&bin) {
        return Ok(());
    }
    if which("launchctl").is_empty() {
        return Err(format!(
            "{}: launchctl not found; keeping proxy running for this session",
            ERR_PROXY_AUTOSTART_UNAVAILABLE
        )
        .into());
    }
    let path = proxy_autostart_plist_path();
    let body = proxy_autostart_plist_body(&bin);
    if let Some(ref cur) = read_file_safe(&path) {
        if !cur.contains("tokless-managed") {
            return Err(format!("refusing to replace non-tokless launch agent {}", path).into());
        }
        if cur == &body
            && proxy_autostart_enabled_darwin()
            && proxy_running()
            && proxy_supervised_args_match(proxy_port_from_runtime())
        {
            return Ok(());
        }
    }
    let old_plist = read_file_safe(&path);
    if let Some(parent) = Path::new(&proxy_autostart_log_path()).parent() {
        ensure_dir(parent)?;
    }
    let dom = launchctl_domain();
    let service_target = format!("{}/{}", dom, PROXY_AUTOSTART_LABEL);
    let mut old_loaded = false;
    let mut old_enabled = true;
    if old_plist.is_some() {
        match run_launchctl(&["print", &service_target]) {
            Ok(_) => old_loaded = true,
            Err((out, e)) => {
                if !launchctl_service_not_found(&out) {
                    return Err(format!("query {}: {}", PROXY_AUTOSTART_LABEL, e).into());
                }
            }
        }
        old_enabled = launchctl_service_enabled()?;
    }
    let port = proxy_port_from_runtime();
    let proxy_was_running = proxy_running()
        && (proxy_args_match_recorded(port) || proxy_supervised_args_match(port));

    let rollback = |old_plist: &Option<String>,
                    old_loaded: bool,
                    old_enabled: bool,
                    proxy_was_running: bool| {
        let _ = bootout_proxy_agent();
        if let Some(old) = old_plist {
            let _ = write_file(&path, old);
        } else {
            let _ = std::fs::remove_file(&path);
        }
        if old_loaded {
            let _ = run_launchctl(&["bootstrap", &dom, &path]);
            let action = if old_enabled { "enable" } else { "disable" };
            let _ = run_launchctl(&[action, &service_target]);
        } else if !old_enabled {
            let _ = run_launchctl(&["disable", &service_target]);
        }
        if !old_loaded && proxy_was_running {
            let _ = restore_proxy_after_autostart_rollback(proxy_was_running);
        }
    };

    if read_file_safe(&path).as_deref() != Some(body.as_str()) {
        if let Some(parent) = Path::new(&path).parent() {
            ensure_dir(parent)?;
        }
        write_file(&path, &body)?;
    }
    if let Err(err) = bootout_proxy_agent() {
        rollback(&old_plist, old_loaded, old_enabled, proxy_was_running);
        return Err(err);
    }
    if let Err(err) = stop_headroom_daemon_for_handoff() {
        rollback(&old_plist, old_loaded, old_enabled, proxy_was_running);
        return Err(err);
    }
    if let Err((_, err)) = run_launchctl(&["bootstrap", &dom, &path]) {
        rollback(&old_plist, old_loaded, old_enabled, proxy_was_running);
        return Err(format!("launchctl bootstrap {}: {}", PROXY_AUTOSTART_LABEL, err).into());
    }
    if let Err((_, err)) = run_launchctl(&["enable", &service_target]) {
        rollback(&old_plist, old_loaded, old_enabled, proxy_was_running);
        return Err(format!("launchctl enable {}: {}", PROXY_AUTOSTART_LABEL, err).into());
    }

    let deadline = Instant::now() + PROXY_READY_TIMEOUT;
    while Instant::now() < deadline {
        if proxy_running() && proxy_supervised_args_match(proxy_port_from_runtime()) {
            return Ok(());
        }
        std::thread::sleep(PROXY_POLL_INTERVAL);
    }
    rollback(&old_plist, old_loaded, old_enabled, proxy_was_running);
    Err(format!("{} loaded but proxy is not ready", PROXY_AUTOSTART_LABEL).into())
}

fn disable_proxy_autostart_darwin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if which("launchctl").is_empty() {
        return Ok(());
    }
    let path = proxy_autostart_plist_path();
    let Some(raw) = read_file_safe(&path) else {
        return Ok(());
    };
    if !raw.contains("tokless-managed") {
        return Ok(());
    }
    bootout_proxy_agent()?;
    if let Err(err) = std::fs::remove_file(&path) {
        if err.kind() != std::io::ErrorKind::NotFound {
            let _ = write_file(&path, &raw);
            let _ = run_launchctl(&["bootstrap", &launchctl_domain(), &path]);
            return Err(err.into());
        }
    }
    Ok(())
}

fn proxy_autostart_enabled_darwin() -> bool {
    let path = proxy_autostart_plist_path();
    let Some(raw) = read_file_safe(&path) else {
        return false;
    };
    if !raw.contains("__proxy-run") {
        return false;
    }
    let target = format!("{}/{}", launchctl_domain(), PROXY_AUTOSTART_LABEL);
    let Ok(out) = Command::new("launchctl").args(["print", &target]).output() else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    s.contains("state = \"running\"")
}

fn proxy_autostart_configured_darwin() -> bool {
    let Some(raw) = read_file_safe(proxy_autostart_plist_path()) else {
        return false;
    };
    let bin = tokless_abs();
    !bin.is_empty() && !is_test_executable(&bin) && raw == proxy_autostart_plist_body(&bin)
}

// ---------------------------------------------------------------------------
// Linux systemd user units
// ---------------------------------------------------------------------------

fn linux_systemd_user_dir() -> std::path::PathBuf {
    let cfg = std::env::var("XDG_CONFIG_HOME").unwrap_or_default();
    let base = if cfg.is_empty() || !Path::new(&cfg).is_absolute() {
        Path::new(&home()).join(".config")
    } else {
        Path::new(&cfg).to_path_buf()
    };
    base.join("systemd").join("user")
}

pub fn proxy_autostart_unit_path() -> String {
    linux_systemd_user_dir()
        .join(PROXY_AUTOSTART_UNIT)
        .to_string_lossy()
        .to_string()
}

pub fn byok_gateway_autostart_unit_path() -> String {
    linux_systemd_user_dir()
        .join(BYOK_GATEWAY_AUTOSTART_UNIT)
        .to_string_lossy()
        .to_string()
}

pub fn shell_quote(s: &str) -> String {
    let mut b = String::with_capacity(s.len() + 2);
    b.push('"');
    for r in s.chars() {
        match r {
            '%' => {
                b.push_str("%%");
                continue;
            }
            '\\' | '"' | '$' | '`' => {
                b.push('\\');
            }
            _ => {}
        }
        b.push(r);
    }
    b.push('"');
    b
}

pub fn proxy_autostart_unit_body(tokless_bin: &str) -> String {
    let q = shell_quote(tokless_bin);
    format!(
        "[Unit]\nDescription=tokless headroom proxy (compression + BYOK route)\n# tokless-managed\nAfter=default.target\n\n[Service]\nType=simple\nExecStart={} __proxy-run\nUnsetEnvironment=TOKLESS_PROXY_PROVIDER TOKLESS_HEADROOM_PROXY_PORT TOKLESS_HEADROOM_ANTHROPIC_URL TOKLESS_HEADROOM_OPENAI_URL TOKLESS_HEADROOM_GEMINI_URL TOKLESS_HEADROOM_CLOUDCODE_URL OPENAI_TARGET_API_URL ANTHROPIC_TARGET_API_URL GROK_MODELS_BASE_URL\nRestart=on-failure\n\n[Install]\nWantedBy=default.target\n",
        q
    )
}

pub fn is_wsl() -> bool {
    if !std::env::var("WSL_INTEROP").unwrap_or_default().is_empty()
        || !std::env::var("WSL_DISTRO_NAME")
            .unwrap_or_default()
            .is_empty()
    {
        return true;
    }
    if let Ok(b) = std::fs::read_to_string("/proc/version") {
        return b.to_lowercase().contains("microsoft");
    }
    false
}

pub fn loginctl_linger(user: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let out = Command::new("loginctl")
        .args(["show-user", user, "-p", "Linger", "--value"])
        .output()?;
    if !out.status.success() {
        return Err(format!(
            "loginctl show-user failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )
        .into());
    }
    match String::from_utf8_lossy(&out.stdout).trim() {
        "yes" => Ok(true),
        "no" => Ok(false),
        other => Err(format!("unexpected linger state {:?}", other).into()),
    }
}

pub fn systemd_user_state(
    action: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let out = Command::new("systemctl")
        .args(["--user", action, PROXY_AUTOSTART_UNIT])
        .output()?;
    let mut combined = out.stdout;
    combined.extend_from_slice(&out.stderr);
    let raw_state = String::from_utf8_lossy(&combined).trim().to_string();
    let state = raw_state.lines().next().unwrap_or("").trim();

    if action == "is-enabled" {
        match state {
            "enabled" | "static" | "indirect" | "disabled" | "masked" | "generated"
            | "transient" | "bad-setting" | "alias" | "not-found" => {
                return Ok(state == "enabled" || state == "static" || state == "indirect");
            }
            _ => {}
        }
    } else if action == "is-active" {
        match state {
            "active" | "inactive" | "failed" | "activating" | "deactivating" | "maintenance"
            | "unknown" => {
                return Ok(state == "active");
            }
            _ => {}
        }
    }
    if !out.status.success() {
        return Err(format!("systemctl {}: ({})", action, state).into());
    }
    Err(format!("systemctl {} returned unknown state {:?}", action, state).into())
}

fn enable_proxy_autostart_linux() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bin = tokless_abs();
    if bin.is_empty() || is_test_executable(&bin) {
        return Ok(());
    }
    if which("systemctl").is_empty() {
        if is_wsl() {
            return Err(format!(
                "{}: WSL proxy autostart requires systemd user services; enable systemd in /etc/wsl.conf",
                ERR_PROXY_AUTOSTART_UNAVAILABLE
            )
            .into());
        }
        return Err(format!(
            "{}: systemctl not found; keeping proxy running for this session",
            ERR_PROXY_AUTOSTART_UNAVAILABLE
        )
        .into());
    }
    let env_check = Command::new("systemctl")
        .args(["--user", "show-environment"])
        .output();
    if env_check.map(|o| !o.status.success()).unwrap_or(true) {
        if is_wsl() {
            return Err(format!(
                "{}: WSL proxy autostart requires a running systemd user bus; enable systemd in /etc/wsl.conf",
                ERR_PROXY_AUTOSTART_UNAVAILABLE
            )
            .into());
        }
        return Err(format!(
            "{}: systemd user bus unavailable; keeping proxy running for this session",
            ERR_PROXY_AUTOSTART_UNAVAILABLE
        )
        .into());
    }

    let path = proxy_autostart_unit_path();
    let body = proxy_autostart_unit_body(&bin);
    if let Some(ref cur) = read_file_safe(&path) {
        if cur == &body {
            let enabled = systemd_user_state("is-enabled")
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("query enabled state of {}: {}", PROXY_AUTOSTART_UNIT, e).into() })?;
            let active = systemd_user_state("is-active")
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("query active state of {}: {}", PROXY_AUTOSTART_UNIT, e).into() })?;
            if enabled
                && active
                && proxy_running()
                && proxy_supervised_args_match(proxy_port_from_runtime())
            {
                return Ok(());
            }
        }
        if !cur.contains("tokless-managed") {
            return Err(format!("refusing to replace non-tokless systemd unit {}", path).into());
        }
    }

    let old_unit = read_file_safe(&path);
    let user = std::env::var("USER").unwrap_or_default();
    if user.is_empty() {
        return Err(format!(
            "{}: cannot enable linger: USER is not set",
            ERR_PROXY_AUTOSTART_UNAVAILABLE
        )
        .into());
    }
    if which("loginctl").is_empty() {
        return Err(format!(
            "{}: loginctl not found; cannot enable proxy linger",
            ERR_PROXY_AUTOSTART_UNAVAILABLE
        )
        .into());
    }
    let linger_was_enabled =
        loginctl_linger(&user).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("query user linger: {}", e).into() })?;

    let mut old_enabled = false;
    let mut old_active = false;
    if old_unit.is_some() {
        old_enabled = systemd_user_state("is-enabled")
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("query enabled state of {}: {}", PROXY_AUTOSTART_UNIT, e).into() })?;
        old_active = systemd_user_state("is-active")
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("query active state of {}: {}", PROXY_AUTOSTART_UNIT, e).into() })?;
    }

    let port = proxy_port_from_runtime();
    let proxy_was_running = proxy_running()
        && (proxy_args_match_recorded(port) || proxy_supervised_args_match(port));

    let rollback = |old_unit: &Option<String>,
                    old_enabled: bool,
                    old_active: bool,
                    linger_was_enabled: bool,
                    proxy_was_running: bool| {
        let _ = Command::new("systemctl")
            .args(["--user", "disable", "--now", PROXY_AUTOSTART_UNIT])
            .status();
        if let Some(old) = old_unit {
            let _ = write_file(&path, old);
        } else {
            let _ = std::fs::remove_file(&path);
        }
        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();
        if old_enabled {
            let _ = Command::new("systemctl")
                .args(["--user", "enable", PROXY_AUTOSTART_UNIT])
                .status();
        }
        if old_active {
            let _ = Command::new("systemctl")
                .args(["--user", "start", PROXY_AUTOSTART_UNIT])
                .status();
        }
        if !linger_was_enabled {
            let _ = Command::new("loginctl")
                .args(["disable-linger", &user])
                .status();
        }
        if !old_active && proxy_was_running {
            let _ = restore_proxy_after_autostart_rollback(proxy_was_running);
        }
    };

    if read_file_safe(&path).as_deref() != Some(body.as_str()) {
        if let Some(parent) = Path::new(&path).parent() {
            ensure_dir(parent)?;
        }
        write_file(&path, &body)?;
        let st = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();
        if st.map(|s| !s.success()).unwrap_or(true) {
            rollback(
                &old_unit,
                old_enabled,
                old_active,
                linger_was_enabled,
                proxy_was_running,
            );
            return Err("reload systemd user units failed".into());
        }
    }

    if old_unit.is_some() {
        let st = Command::new("systemctl")
            .args(["--user", "stop", PROXY_AUTOSTART_UNIT])
            .status();
        if st.map(|s| !s.success()).unwrap_or(true) {
            rollback(
                &old_unit,
                old_enabled,
                old_active,
                linger_was_enabled,
                proxy_was_running,
            );
            return Err(format!("stop {} failed", PROXY_AUTOSTART_UNIT).into());
        }
    }

    if let Err(err) = stop_headroom_daemon_for_handoff() {
        rollback(
            &old_unit,
            old_enabled,
            old_active,
            linger_was_enabled,
            proxy_was_running,
        );
        return Err(err);
    }

    let enable_st = Command::new("systemctl")
        .args(["--user", "enable", "--now", PROXY_AUTOSTART_UNIT])
        .status();
    if enable_st.map(|s| !s.success()).unwrap_or(true) {
        rollback(
            &old_unit,
            old_enabled,
            old_active,
            linger_was_enabled,
            proxy_was_running,
        );
        return Err(format!("enable {} failed", PROXY_AUTOSTART_UNIT).into());
    }

    let deadline = Instant::now() + PROXY_READY_TIMEOUT;
    let mut ready = false;
    while Instant::now() < deadline {
        if proxy_running() && proxy_supervised_args_match(proxy_port_from_runtime()) {
            ready = true;
            break;
        }
        std::thread::sleep(PROXY_POLL_INTERVAL);
    }
    if !ready {
        rollback(
            &old_unit,
            old_enabled,
            old_active,
            linger_was_enabled,
            proxy_was_running,
        );
        return Err(format!("{} enabled but proxy is not ready", PROXY_AUTOSTART_UNIT).into());
    }

    let linger_st = Command::new("loginctl")
        .args(["enable-linger", &user])
        .status();
    if linger_st.map(|s| !s.success()).unwrap_or(true) {
        rollback(
            &old_unit,
            old_enabled,
            old_active,
            linger_was_enabled,
            proxy_was_running,
        );
        return Err("enable user linger failed".into());
    }

    Ok(())
}

fn disable_proxy_autostart_linux() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if which("systemctl").is_empty() {
        return Ok(());
    }
    let path = proxy_autostart_unit_path();
    let Some(raw) = read_file_safe(&path) else {
        return Ok(());
    };
    if !raw.contains("tokless-managed") {
        return Ok(());
    }
    let was_enabled = systemd_user_state("is-enabled")
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("query enabled state of {}: {}", PROXY_AUTOSTART_UNIT, e).into() })?;
    let was_active = systemd_user_state("is-active")
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("query active state of {}: {}", PROXY_AUTOSTART_UNIT, e).into() })?;

    let rollback = || {
        let _ = write_file(&path, &raw);
        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();
        if was_enabled {
            let _ = Command::new("systemctl")
                .args(["--user", "enable", PROXY_AUTOSTART_UNIT])
                .status();
        }
        if was_active {
            let _ = Command::new("systemctl")
                .args(["--user", "start", PROXY_AUTOSTART_UNIT])
                .status();
        }
    };

    let disable_st = Command::new("systemctl")
        .args(["--user", "disable", "--now", PROXY_AUTOSTART_UNIT])
        .status();
    if disable_st.map(|s| !s.success()).unwrap_or(true) {
        rollback();
        return Err(format!("disable {} failed", PROXY_AUTOSTART_UNIT).into());
    }
    if let Err(err) = std::fs::remove_file(&path) {
        if err.kind() != std::io::ErrorKind::NotFound {
            rollback();
            return Err(err.into());
        }
    }
    let reload_st = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();
    if reload_st.map(|s| !s.success()).unwrap_or(true) {
        rollback();
        return Err("reload systemd user units failed".into());
    }
    Ok(())
}

fn proxy_autostart_enabled_linux() -> bool {
    let path = proxy_autostart_unit_path();
    let Some(raw) = read_file_safe(&path) else {
        return false;
    };
    if !raw.contains("__proxy-run") {
        return false;
    }
    let Ok(out) = Command::new("systemctl")
        .args(["--user", "is-enabled", PROXY_AUTOSTART_UNIT])
        .output()
    else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    s == "enabled" || s == "static" || s == "indirect"
}

fn proxy_autostart_configured_linux() -> bool {
    let Some(raw) = read_file_safe(proxy_autostart_unit_path()) else {
        return false;
    };
    let bin = tokless_abs();
    !bin.is_empty() && !is_test_executable(&bin) && raw == proxy_autostart_unit_body(&bin)
}

// ---------------------------------------------------------------------------
// Windows Scheduled Tasks
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScheduledTask {
    pub has_logon_trigger: bool,
    pub run_level: String,
    pub command: String,
    pub arguments: String,
}

fn decode_xml_bytes(raw: &[u8]) -> String {
    if raw.len() >= 2 && raw[0] == 0xFF && raw[1] == 0xFE {
        let u16s: Vec<u16> = raw[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16_lossy(&u16s);
    }
    if raw.len() >= 2 && raw[0] == 0xFE && raw[1] == 0xFF {
        let u16s: Vec<u16> = raw[2..]
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16_lossy(&u16s);
    }
    if raw.len() >= 4 && raw[1] == 0 && raw[3] == 0 {
        let u16s: Vec<u16> = raw
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16_lossy(&u16s);
    }
    String::from_utf8_lossy(raw).to_string()
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open_prefix = format!("<{}", tag);
    let close_tag = format!("</{}>", tag);
    let mut search_from = 0;
    while let Some(rel_start) = xml[search_from..].find(&open_prefix) {
        let start = search_from + rel_start;
        let after_name = start + open_prefix.len();
        let next_ch = xml[after_name..].chars().next()?;
        if next_ch != '>' && next_ch != ' ' && next_ch != '\t' && next_ch != '\r' && next_ch != '\n' && next_ch != '/' {
            search_from = after_name;
            continue;
        }
        let gt_rel = xml[after_name..].find('>')?;
        let gt_pos = after_name + gt_rel;
        if xml[..gt_pos].ends_with('/') {
            return Some(String::new());
        }
        let content_start = gt_pos + 1;
        let end_rel = xml[content_start..].find(&close_tag)?;
        let content_end = content_start + end_rel;
        return Some(xml[content_start..content_end].to_string());
    }
    None
}

pub fn parse_scheduled_task(raw: &[u8]) -> Option<ScheduledTask> {
    let text = decode_xml_bytes(raw);
    if !text.contains("<Task") || !text.contains("</Task>") {
        return None;
    }
    let triggers = extract_xml_tag(&text, "Triggers")?;
    let has_logon = extract_xml_tag(&triggers, "LogonTrigger").is_some();
    if !has_logon {
        return None;
    }
    let principals = extract_xml_tag(&text, "Principals").unwrap_or_default();
    let principal = extract_xml_tag(&principals, "Principal").unwrap_or_default();
    let run_level = extract_xml_tag(&principal, "RunLevel")
        .map(|s| xml_unescape(s.trim()))
        .unwrap_or_default();

    let actions = extract_xml_tag(&text, "Actions").unwrap_or_default();
    let exec = extract_xml_tag(&actions, "Exec").unwrap_or_default();
    let command = extract_xml_tag(&exec, "Command")
        .map(|s| xml_unescape(&s))
        .unwrap_or_default();
    let arguments = extract_xml_tag(&exec, "Arguments")
        .map(|s| xml_unescape(&s))
        .unwrap_or_default();

    Some(ScheduledTask {
        has_logon_trigger: true,
        run_level,
        command,
        arguments,
    })
}

pub fn scheduled_task_managed(raw: &[u8]) -> bool {
    let Some(task) = parse_scheduled_task(raw) else {
        return false;
    };
    let trimmed = task.command.trim().trim_matches('"');
    let base = Path::new(trimmed)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    (base.eq_ignore_ascii_case("tokless") || base.eq_ignore_ascii_case("tokless.exe"))
        && task.arguments.trim() == "__proxy-watch"
        && task.run_level == "LeastPrivilege"
}

fn clean_win_path(p: &str) -> String {
    p.trim()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_string()
}

pub fn scheduled_task_matches(raw: &[u8], bin: &str) -> bool {
    let Some(task) = parse_scheduled_task(raw) else {
        return false;
    };
    if !scheduled_task_managed(raw) {
        return false;
    }
    let cmd_clean = clean_win_path(&task.command);
    let want_clean = clean_win_path(bin);
    !want_clean.is_empty()
        && cmd_clean == want_clean
        && task.arguments.trim() == "__proxy-watch"
        && task.run_level == "LeastPrivilege"
}

pub fn scheduled_task_not_found(output: &[u8]) -> bool {
    let s = String::from_utf8_lossy(output).to_lowercase();
    s.contains("cannot find") || s.contains("does not exist") || s.contains("not found")
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    chars.next();
                    cur.push('"');
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ',' if !in_quotes => {
                fields.push(std::mem::take(&mut cur));
            }
            _ => cur.push(c),
        }
    }
    fields.push(cur);
    fields
}

pub fn scheduled_task_running() -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let out = Command::new("schtasks")
        .args(["/query", "/tn", PROXY_AUTOSTART_TASK, "/fo", "CSV", "/nh"])
        .output()?;
    let mut combined = out.stdout;
    combined.extend_from_slice(&out.stderr);
    if !out.status.success() {
        if scheduled_task_not_found(&combined) {
            return Ok(false);
        }
        return Err(String::from_utf8_lossy(&combined).trim().to_string().into());
    }
    let text = String::from_utf8_lossy(&combined);
    let first_line = text
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty())
        .ok_or("empty task state")?;
    let fields = parse_csv_line(first_line);
    let last = fields.last().map(|s| s.trim()).unwrap_or("");
    if last.is_empty() {
        return Err("empty task state".into());
    }
    match last.to_lowercase().as_str() {
        "running" => Ok(true),
        "ready" | "queued" => Ok(false),
        _ => Err(format!("unexpected task state {:?}", last).into()),
    }
}

pub fn read_scheduled_task() -> Result<Vec<u8>, (Vec<u8>, String)> {
    match Command::new("schtasks")
        .args(["/query", "/tn", PROXY_AUTOSTART_TASK, "/xml"])
        .output()
    {
        Ok(out) => {
            let mut combined = out.stdout;
            combined.extend_from_slice(&out.stderr);
            if out.status.success() {
                Ok(combined)
            } else {
                let err = String::from_utf8_lossy(&combined).trim().to_string();
                Err((combined, err))
            }
        }
        Err(e) => Err((Vec::new(), e.to_string())),
    }
}

pub fn end_proxy_autostart_task() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let out = Command::new("schtasks")
        .args(["/end", "/tn", PROXY_AUTOSTART_TASK])
        .output()?;
    let mut combined = out.stdout;
    combined.extend_from_slice(&out.stderr);
    if out.status.success() || scheduled_task_not_found(&combined) {
        return Ok(());
    }
    Err(String::from_utf8_lossy(&combined).trim().to_string().into())
}

pub fn delete_proxy_autostart_task() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let out = Command::new("schtasks")
        .args(["/delete", "/tn", PROXY_AUTOSTART_TASK, "/f"])
        .output()?;
    let mut combined = out.stdout;
    combined.extend_from_slice(&out.stderr);
    if out.status.success() || scheduled_task_not_found(&combined) {
        return Ok(());
    }
    Err(String::from_utf8_lossy(&combined).trim().to_string().into())
}

pub fn rollback_scheduled_task(
    old_task: &[u8],
    was_running: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if old_task.is_empty() {
        return delete_proxy_autostart_task();
    }
    let mut tmp = tempfile::Builder::new()
        .prefix("tokless-task-")
        .suffix(".xml")
        .tempfile()?;
    use std::io::Write;
    tmp.write_all(old_task)?;
    tmp.flush()?;
    let path_str = tmp.path().to_string_lossy().to_string();
    let create_st = Command::new("schtasks")
        .args(["/create", "/tn", PROXY_AUTOSTART_TASK, "/xml", &path_str, "/f"])
        .status()?;
    if !create_st.success() {
        return Err("schtasks rollback create failed".into());
    }
    if was_running {
        let run_st = Command::new("schtasks")
            .args(["/run", "/tn", PROXY_AUTOSTART_TASK])
            .status()?;
        if !run_st.success() {
            return Err("schtasks rollback run failed".into());
        }
    }
    Ok(())
}

fn enable_proxy_autostart_windows() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bin = tokless_abs_strict();
    if bin.is_empty() {
        return Ok(());
    }
    if which("schtasks").is_empty() {
        return Err(format!(
            "{}: schtasks not found; keeping proxy running for this session",
            ERR_PROXY_AUTOSTART_UNAVAILABLE
        )
        .into());
    }

    let mut old_task: Vec<u8> = Vec::new();
    let mut old_task_running = false;
    match read_scheduled_task() {
        Ok(task_bytes) => {
            if scheduled_task_matches(&task_bytes, &bin) {
                let running = scheduled_task_running()
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("query state of {}: {}", PROXY_AUTOSTART_TASK, e).into() })?;
                if running
                    && proxy_running()
                    && proxy_supervised_args_match(proxy_port_from_runtime())
                {
                    return Ok(());
                }
            }
            if !scheduled_task_managed(&task_bytes) {
                return Err(format!(
                    "refusing to replace non-tokless scheduled task {}",
                    PROXY_AUTOSTART_TASK
                )
                .into());
            }
            old_task = task_bytes;
            old_task_running = scheduled_task_running()
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("query state of {}: {}", PROXY_AUTOSTART_TASK, e).into() })?;
        }
        Err((out, err)) => {
            if !scheduled_task_not_found(&out) {
                return Err(format!("query {}: {}", PROXY_AUTOSTART_TASK, err).into());
            }
        }
    }

    let port = proxy_port_from_runtime();
    let proxy_was_running = proxy_running()
        && (proxy_args_match_recorded(port) || proxy_supervised_args_match(port));

    request_proxy_stop().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("proxy stop request: {}", e).into() })?;

    if old_task_running {
        if let Err(err) = end_proxy_autostart_task() {
            let _ = clear_proxy_stop_request();
            return Err(format!("end {}: {}", PROXY_AUTOSTART_TASK, err).into());
        }
    }
    let old_task_ended = !old_task.is_empty();

    let tr = format!("\"{}\" __proxy-watch", bin);
    let create_out = Command::new("schtasks")
        .args([
            "/create",
            "/tn",
            PROXY_AUTOSTART_TASK,
            "/tr",
            &tr,
            "/sc",
            "ONLOGON",
            "/rl",
            "LIMITED",
            "/f",
        ])
        .output()?;

    if !create_out.status.success() {
        let _ = rollback_scheduled_task(&old_task, false);
        let _ = clear_proxy_stop_request();
        if old_task_ended && old_task_running {
            let _ = Command::new("schtasks")
                .args(["/run", "/tn", PROXY_AUTOSTART_TASK])
                .status();
        } else {
            let _ = restore_proxy_after_autostart_rollback(proxy_was_running);
        }
        return Err(format!(
            "schtasks create failed: {}",
            String::from_utf8_lossy(&create_out.stderr).trim()
        )
        .into());
    }

    let do_rollback = |stop_request_cleared: bool| {
        if stop_request_cleared {
            let _ = request_proxy_stop();
        }
        let _ = stop_headroom_daemon_for_handoff();
        if rollback_scheduled_task(&old_task, false).is_ok() {
            let _ = clear_proxy_stop_request();
            if old_task_ended && old_task_running {
                let _ = Command::new("schtasks")
                    .args(["/run", "/tn", PROXY_AUTOSTART_TASK])
                    .status();
            } else {
                let _ = restore_proxy_after_autostart_rollback(proxy_was_running);
            }
        }
    };

    if let Err(err) = stop_headroom_daemon_for_handoff() {
        do_rollback(false);
        return Err(err);
    }
    if let Err(err) = clear_proxy_stop_request() {
        do_rollback(false);
        return Err(format!("clear proxy stop request: {}", err).into());
    }

    let run_out = Command::new("schtasks")
        .args(["/run", "/tn", PROXY_AUTOSTART_TASK])
        .output()?;
    if !run_out.status.success() {
        do_rollback(true);
        return Err(format!(
            "initial start of {}: {}",
            PROXY_AUTOSTART_TASK,
            String::from_utf8_lossy(&run_out.stderr).trim()
        )
        .into());
    }

    let deadline = Instant::now() + PROXY_READY_TIMEOUT;
    while Instant::now() < deadline {
        match scheduled_task_running() {
            Ok(running) => {
                if running
                    && proxy_running()
                    && proxy_supervised_args_match(proxy_port_from_runtime())
                {
                    return Ok(());
                }
            }
            Err(state_err) => {
                do_rollback(true);
                return Err(format!("query state of {}: {}", PROXY_AUTOSTART_TASK, state_err).into());
            }
        }
        std::thread::sleep(PROXY_POLL_INTERVAL);
    }

    do_rollback(true);
    Err(format!("{} started but proxy is not ready", PROXY_AUTOSTART_TASK).into())
}

fn disable_proxy_autostart_windows() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if which("schtasks").is_empty() {
        return Ok(());
    }
    let out = match read_scheduled_task() {
        Ok(b) => b,
        Err((b, err)) => {
            if scheduled_task_not_found(&b) {
                return Ok(());
            }
            return Err(format!("query {}: {}", PROXY_AUTOSTART_TASK, err).into());
        }
    };
    if !scheduled_task_managed(&out) {
        return Ok(());
    }
    let running = scheduled_task_running()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("query state of {}: {}", PROXY_AUTOSTART_TASK, e).into() })?;
    if running {
        end_proxy_autostart_task()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("end {}: {}", PROXY_AUTOSTART_TASK, e).into() })?;
    }
    if let Err(err) = delete_proxy_autostart_task() {
        let _ = rollback_scheduled_task(&out, running);
        return Err(format!("delete {}: {}", PROXY_AUTOSTART_TASK, err).into());
    }
    Ok(())
}

fn proxy_autostart_enabled_windows() -> bool {
    let Ok(out) = read_scheduled_task() else {
        return false;
    };
    let Ok(running) = scheduled_task_running() else {
        return false;
    };
    scheduled_task_matches(&out, &tokless_abs_strict())
        && running
        && proxy_running()
        && proxy_supervised_args_match(proxy_port_from_runtime())
}

fn proxy_autostart_configured_windows() -> bool {
    let Ok(out) = read_scheduled_task() else {
        return false;
    };
    scheduled_task_matches(&out, &tokless_abs_strict())
}

// ---------------------------------------------------------------------------
// Cross-platform entrypoints
// ---------------------------------------------------------------------------

pub fn enable_proxy_autostart() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if cfg!(target_os = "windows") {
        enable_proxy_autostart_windows()
    } else if cfg!(target_os = "macos") {
        enable_proxy_autostart_darwin()
    } else if cfg!(target_os = "linux") {
        enable_proxy_autostart_linux()
    } else {
        Ok(())
    }
}

pub fn disable_proxy_autostart() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if cfg!(target_os = "windows") {
        disable_proxy_autostart_windows()
    } else if cfg!(target_os = "macos") {
        disable_proxy_autostart_darwin()
    } else if cfg!(target_os = "linux") {
        disable_proxy_autostart_linux()
    } else {
        Ok(())
    }
}

pub fn proxy_autostart_enabled() -> bool {
    if cfg!(target_os = "windows") {
        proxy_autostart_enabled_windows()
    } else if cfg!(target_os = "macos") {
        proxy_autostart_enabled_darwin()
    } else if cfg!(target_os = "linux") {
        proxy_autostart_enabled_linux()
    } else {
        false
    }
}

pub fn proxy_autostart_configured() -> bool {
    if cfg!(target_os = "windows") {
        proxy_autostart_configured_windows()
    } else if cfg!(target_os = "macos") {
        proxy_autostart_configured_darwin()
    } else if cfg!(target_os = "linux") {
        proxy_autostart_configured_linux()
    } else {
        false
    }
}

pub fn stop_proxy_supervisor_for_patch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !proxy_autostart_configured() {
        return Ok(());
    }
    if cfg!(target_os = "windows") {
        request_proxy_stop()?;
        end_proxy_autostart_task()
    } else if cfg!(target_os = "macos") {
        bootout_proxy_agent()
    } else if cfg!(target_os = "linux") {
        let st = Command::new("systemctl")
            .args(["--user", "stop", PROXY_AUTOSTART_UNIT])
            .status()?;
        if !st.success() {
            return Err(format!("systemctl stop {} failed", PROXY_AUTOSTART_UNIT).into());
        }
        Ok(())
    } else {
        Ok(())
    }
}

pub fn stop_byok_gateway_supervisor() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if cfg!(target_os = "linux") {
        let unit_path = byok_gateway_autostart_unit_path();
        if !exists(&unit_path) {
            return Ok(());
        }
        let Ok(out) = Command::new("systemctl")
            .args(["--user", "is-active", BYOK_GATEWAY_AUTOSTART_UNIT])
            .output()
        else {
            return Ok(());
        };
        if !out.status.success() || String::from_utf8_lossy(&out.stdout).trim() != "active" {
            return Ok(());
        }
        let st = Command::new("systemctl")
            .args(["--user", "stop", BYOK_GATEWAY_AUTOSTART_UNIT])
            .status()?;
        if !st.success() {
            return Err(format!("systemctl stop {} failed", BYOK_GATEWAY_AUTOSTART_UNIT).into());
        }
    }
    Ok(())
}
