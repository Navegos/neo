use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::commands::helpers::{bust_version_cache_file, InitOptions};
use crate::util::colors::*;
use crate::util::exec::run_cmd;
use crate::util::install::refresh_install_marker;
use crate::util::logger::L;
use crate::util::paths::{is_win, temp_dir, tokless_abs_strict, which};
use crate::util::version::{semver_gte, tokless_version};
use crate::util::versions::github_latest;

const INSTALL_SH: &str = "https://raw.githubusercontent.com/Navegos/neo/main/scripts/install.sh";
const INSTALL_PS1: &str = "https://raw.githubusercontent.com/Navegos/neo/main/scripts/install.ps1";

pub fn tokless_release_asset() -> &'static str {
    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "amd64"
    };
    if cfg!(target_os = "macos") {
        if arch == "arm64" {
            "tokless-darwin-arm64.tar.gz"
        } else {
            "tokless-darwin-amd64.tar.gz"
        }
    } else if cfg!(target_os = "linux") {
        if arch == "arm64" {
            "tokless-linux-arm64.tar.gz"
        } else {
            "tokless-linux-amd64.tar.gz"
        }
    } else if cfg!(target_os = "windows") {
        if arch == "arm64" {
            "tokless-windows-arm64.zip"
        } else {
            "tokless-windows-amd64.zip"
        }
    } else {
        ""
    }
}

pub fn run_self_update() -> i32 {
    let local = tokless_version();
    let latest = fetch_latest_release_tag();
    if latest.is_empty() {
        L.err("Could not reach latest version.");
        L.raw(&format!("  {}", C.cyan(&format!("curl -fsSL {} | bash", INSTALL_SH))));
        return 1;
    }
    if semver_gte(&local, &latest) {
        L.raw(&format!(
            "  {} tokless {}",
            C.green(Sym::CHECK),
            C.gray(&format!("v{}", local))
        ));
        return 0;
    }
    if run_self_update_to(&latest) {
        refresh_install_marker(&latest);
        bust_version_cache_file();
        L.raw(&format!(
            "  {} tokless {}",
            C.green(Sym::CHECK),
            C.gray(&format!("v{} → v{} updated", local, latest))
        ));
        0
    } else {
        1
    }
}

pub fn maybe_self_update(opts: &InitOptions) {
    if opts.dry_run
        || tokless_version().ends_with("-dev")
        || env::var("TOKLESS_SELF_UPDATED").as_deref() == Ok("1")
        || env::var("TOKLESS_TEST").as_deref() == Ok("1")
    {
        return;
    }
    let local = tokless_version();
    let latest = fetch_latest_release_tag();
    if latest.is_empty() || semver_gte(&local, &latest) {
        return;
    }
    if run_self_update_to(&latest) {
        refresh_install_marker(&latest);
        bust_version_cache_file();
        L.raw(&format!(
            "  {} tokless {}",
            C.green(Sym::CHECK),
            C.gray(&format!("v{} → v{} updated", local, latest))
        ));
        reexec_after_self_update();
    }
}

fn fetch_latest_release_tag() -> String {
    if env::var("TOKLESS_TEST").as_deref() == Ok("1") {
        return tokless_version().to_string();
    }
    if let Some(tag) = github_latest("Navegos/neo") {
        if !tag.is_empty() {
            return tag;
        }
    }
    let cmd = if is_win() {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "(Invoke-RestMethod -Uri 'https://api.github.com/repos/Navegos/neo/releases/latest').tag_name",
            ])
            .output()
    } else {
        Command::new("curl")
            .args([
                "-fsSL",
                "https://api.github.com/repos/Navegos/neo/releases/latest",
            ])
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

pub fn self_update_unix_direct(latest: &str) -> bool {
    if is_win() {
        return false;
    }
    let current_exe = tokless_abs_strict();
    if current_exe.is_empty() {
        return false;
    }
    let asset = tokless_release_asset();
    if asset.is_empty() || which("curl").is_empty() || which("tar").is_empty() {
        return false;
    }
    let tag = if latest.starts_with('v') {
        latest.to_string()
    } else {
        format!("v{}", latest)
    };
    let url = format!(
        "https://github.com/Navegos/neo/releases/download/{}/{}",
        tag, asset
    );
    let tmp_dir = Path::new(&temp_dir()).join(format!("tokless-selfupdate-{}", std::process::id()));
    let _ = fs::create_dir_all(&tmp_dir);
    let archive_path = tmp_dir.join(asset);
    let archive_str = archive_path.to_string_lossy().to_string();

    let curl_res = run_cmd("curl", &["-fsSL", "-o", &archive_str, &url], &[]);
    if curl_res.code != 0 {
        let _ = fs::remove_dir_all(&tmp_dir);
        return false;
    }
    let tmp_str = tmp_dir.to_string_lossy().to_string();
    let tar_res = run_cmd("tar", &["-xzf", &archive_str, "-C", &tmp_str], &[]);
    if tar_res.code != 0 {
        let _ = fs::remove_dir_all(&tmp_dir);
        return false;
    }
    let extracted = tmp_dir.join("tokless");
    if !extracted.is_file() {
        let _ = fs::remove_dir_all(&tmp_dir);
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(&extracted) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(&extracted, perms);
        }
    }
    let staging = format!("{}.new", current_exe);
    let ok = fs::copy(&extracted, &staging).is_ok() && fs::rename(&staging, &current_exe).is_ok();
    let _ = fs::remove_file(&staging);
    let _ = fs::remove_dir_all(&tmp_dir);
    ok
}

pub fn run_self_update_to(latest: &str) -> bool {
    if env::var("TOKLESS_TEST").as_deref() == Ok("1") {
        return true;
    }
    if !is_win() && self_update_unix_direct(latest) {
        return true;
    }
    if is_win() {
        let status = Command::new("powershell")
            .env("TOKLESS_INSTALLER_NO_INIT", "1")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &format!("$env:TOKLESS_INSTALLER_NO_INIT='1'; irm {} | iex", INSTALL_PS1),
            ])
            .status();
        status.map(|s| s.success()).unwrap_or(false)
    } else {
        let status = Command::new("sh")
            .env("TOKLESS_INSTALLER_NO_INIT", "1")
            .args(["-c", &format!("curl -fsSL {} | TOKLESS_INSTALLER_NO_INIT=1 bash", INSTALL_SH)])
            .status();
        status.map(|s| s.success()).unwrap_or(false)
    }
}

pub fn reexec_after_self_update() {
    let exe = tokless_abs_strict();
    if exe.is_empty() || !Path::new(&exe).is_file() {
        return;
    }
    let args: Vec<String> = env::args().skip(1).collect();
    if let Ok(status) = Command::new(&exe)
        .args(&args)
        .env("TOKLESS_SELF_UPDATED", "1")
        .status()
    {
        std::process::exit(status.code().unwrap_or(0));
    }
}
