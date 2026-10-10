use std::env;
use std::process::Command;
use crate::commands::helpers::InitOptions;
use crate::util::colors::*;
use crate::util::logger::L;
use crate::util::paths::is_win;
use crate::util::version::tokless_version;

const INSTALL_SH: &str = "https://raw.githubusercontent.com/HoangP8/tokless/main/scripts/install.sh";
const INSTALL_PS1: &str = "https://raw.githubusercontent.com/HoangP8/tokless/main/scripts/install.ps1";

pub fn run_self_update() -> i32 {
    let local = tokless_version();
    let latest = fetch_latest_release_tag();
    if latest.is_empty() {
        L.err("Could not reach latest version.");
        L.raw(&format!("  {}", C.cyan(&format!("curl -fsSL {} | bash", INSTALL_SH))));
        return 1;
    }
    if crate::util::version::semver_gte(&local, &latest) {
        L.raw(&format!("  {} tokless {}", C.green(Sym::CHECK), C.gray(&format!("v{}", local))));
        return 0;
    }
    if run_self_update_to(&latest) {
        L.raw(&format!("  {} tokless {}", C.green(Sym::CHECK), C.gray(&format!("v{} → v{} updated", local, latest))));
        0
    } else {
        1
    }
}

pub fn maybe_self_update(opts: &InitOptions) {
    if opts.dry_run || tokless_version().ends_with("-dev") {
        return;
    }
    let local = tokless_version();
    let latest = fetch_latest_release_tag();
    if latest.is_empty() || crate::util::version::semver_gte(&local, &latest) {
        return;
    }
    if run_self_update_to(&latest) {
        L.raw(&format!("  {} tokless {}", C.green(Sym::CHECK), C.gray(&format!("v{} → v{} updated", local, latest))));
    }
}

fn fetch_latest_release_tag() -> String {
    if env::var("TOKLESS_TEST").as_deref() == Ok("1") {
        return tokless_version().to_string();
    }
    // Attempt simple curl or fallback
    let cmd = if is_win() {
        Command::new("powershell")
            .args(["-NoProfile", "-Command", "(Invoke-RestMethod -Uri 'https://api.github.com/repos/HoangP8/tokless/releases/latest').tag_name"])
            .output()
    } else {
        Command::new("curl")
            .args(["-fsSL", "https://api.github.com/repos/HoangP8/tokless/releases/latest"])
            .output()
    };

    if let Ok(out) = cmd {
        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if text.starts_with('v') {
            return text[1..].to_string();
        }
    }
    String::new()
}

fn run_self_update_to(_latest: &str) -> bool {
    if env::var("TOKLESS_TEST").as_deref() == Ok("1") {
        return true;
    }
    if is_win() {
        let status = Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &format!("irm {} | iex", INSTALL_PS1)])
            .status();
        status.map(|s| s.success()).unwrap_or(false)
    } else {
        let status = Command::new("sh")
            .args(["-c", &format!("curl -fsSL {} | bash", INSTALL_SH)])
            .status();
        status.map(|s| s.success()).unwrap_or(false)
    }
}
