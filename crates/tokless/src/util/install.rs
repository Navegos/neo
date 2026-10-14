// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/install.rs
// created: 2026-10-06
// lastModified: 2026-10-14

use std::path::Path;
use super::paths::{ensure_dir, home, read_file_safe, tokless_abs, tokless_abs_strict, tokless_data_dir, write_file, IS_WIN};

pub use super::paths::{install_marker_path, InstallRecord};

pub fn write_install_marker(method: &str, path: &str, version: &str) -> std::io::Result<()> {
    ensure_dir(tokless_data_dir())?;
    let rec = InstallRecord {
        method: method.to_string(),
        path: path.to_string(),
        version: version.to_string(),
        at: chrono::Utc::now().to_rfc3339(),
    };
    let json = serde_json::to_string_pretty(&rec).unwrap_or_default();
    write_file(install_marker_path(), &json)
}

pub fn refresh_install_marker(version: &str) {
    let mut method = "self-update".to_string();
    let mut path = String::new();
    if let Some(raw) = read_file_safe(install_marker_path()) {
        if let Ok(m) = serde_json::from_str::<InstallRecord>(&raw) {
            if !m.method.is_empty() {
                method = m.method;
            }
            if stable_executable(&m.path) {
                path = m.path;
            }
        }
    }
    let current = tokless_abs_strict();
    if stable_executable(&current) {
        path = current;
    }
    if path.is_empty() {
        return;
    }
    let _ = write_install_marker(&method, &path, version);
}

fn stable_executable(p: &str) -> bool {
    !p.is_empty() && Path::new(p).is_file()
}

pub fn install_info() -> (InstallRecord, bool) {
    let exe = tokless_abs();
    if let Some(raw) = read_file_safe(install_marker_path()) {
        if let Ok(m) = serde_json::from_str::<InstallRecord>(&raw) {
            if !m.method.is_empty() && same_path(&m.path, &exe) {
                return (m, true);
            }
        }
    }
    (
        InstallRecord {
            method: infer_install_method(&exe),
            path: exe,
            version: String::new(),
            at: String::new(),
        },
        false,
    )
}

fn same_path(a: &str, b: &str) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    let norm = |p: &str| -> String {
        let clean = Path::new(p).to_string_lossy().to_string();
        if IS_WIN {
            clean.to_lowercase().replace('\\', "/")
        } else {
            clean
        }
    };
    norm(a) == norm(b)
}

pub fn infer_install_method(exe: &str) -> String {
    if exe.is_empty() {
        return "unknown".to_string();
    }
    let slash = if IS_WIN {
        exe.replace('\\', "/").to_lowercase()
    } else {
        exe.replace('\\', "/")
    };
    let dir = Path::new(exe).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();

    if slash.contains("/cellar/")
        || slash.starts_with("/opt/homebrew/")
        || slash.starts_with("/home/linuxbrew/")
        || slash.starts_with("/usr/local/homebrew/")
    {
        return "homebrew".to_string();
    }
    if slash.contains("/node_modules/.bin/") {
        return "npm".to_string();
    }
    if is_cargo_bin_dir(&dir) {
        return "cargo install".to_string();
    }
    if is_install_script_dir(&dir) {
        return "install script".to_string();
    }
    if slash.contains("/target/release/") || slash.contains("/target/debug/") || slash.contains("/dist/release/") {
        return "source build".to_string();
    }
    "unknown".to_string()
}

fn is_install_script_dir(dir: &str) -> bool {
    if same_path(dir, &Path::new(&home()).join(".local").join("bin").to_string_lossy()) {
        return true;
    }
    if !IS_WIN {
        return false;
    }
    if let Ok(base) = std::env::var("LOCALAPPDATA") {
        if same_path(dir, &Path::new(&base).join("Programs").join("tokless").to_string_lossy()) {
            return true;
        }
    }
    false
}

fn is_cargo_bin_dir(dir: &str) -> bool {
    if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
        if same_path(dir, &Path::new(&cargo_home).join("bin").to_string_lossy()) {
            return true;
        }
    }
    same_path(dir, &Path::new(&home()).join(".cargo").join("bin").to_string_lossy())
}
