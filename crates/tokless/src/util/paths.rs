// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/paths.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::path::Path;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};

static HOME_OVERRIDE: RwLock<Option<String>> = RwLock::new(None);
static WRITE_FILE_OVERRIDE: RwLock<
    Option<Box<dyn Fn(&str, &str) -> std::io::Result<()> + Send + Sync>>,
> = RwLock::new(None);

pub const IS_WIN: bool = cfg!(windows);
pub fn is_win() -> bool {
    IS_WIN
}
pub fn temp_dir() -> String {
    std::env::temp_dir().to_string_lossy().to_string()
}

pub use crate::util::exec::{
    find_binary, prepend_process_path, resolve_bun_binary, resolve_codegraph_bin,
    resolve_node_binary, resolve_npm_binary, resolve_rtk_bin, rtk_install_dirs, run_cmd,
    which_any,
};
pub use crate::util::pathsetup::{
    ensure_persistent_path, ensure_process_path, expected_bin_dirs, persist_windows_path_dirs,
    self_heal_path,
};
pub use crate::util::preflight::{
    is_windows_mount, is_wsl, windows_cursor_home_from_wsl, windows_home_from_wsl,
};

pub fn set_home_override(p: impl Into<String>) {
    let mut guard = HOME_OVERRIDE.write().unwrap();
    let s = p.into();
    if s.is_empty() {
        *guard = None;
    } else {
        *guard = Some(s);
    }
}

pub fn clear_home_override() {
    let mut guard = HOME_OVERRIDE.write().unwrap();
    *guard = None;
}

pub fn set_write_file_override<F>(f: F)
where
    F: Fn(&str, &str) -> std::io::Result<()> + Send + Sync + 'static,
{
    let mut guard = WRITE_FILE_OVERRIDE.write().unwrap();
    *guard = Some(Box::new(f));
}

pub fn clear_write_file_override() {
    let mut guard = WRITE_FILE_OVERRIDE.write().unwrap();
    *guard = None;
}

pub fn home() -> String {
    if let Some(ref h) = *HOME_OVERRIDE.read().unwrap() {
        if !h.is_empty() {
            return h.clone();
        }
    }
    if IS_WIN {
        if let Some(dir) = dirs::home_dir() {
            let s = dir.to_string_lossy().to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    if let Ok(h) = std::env::var("HOME") {
        if !h.is_empty() {
            return h;
        }
    }
    dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

pub fn split_command(cmd: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let chars: Vec<char> = cmd.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let r = chars[i];
        if r == '\\' && i + 1 < chars.len() {
            let next = chars[i + 1];
            if (in_double && (next == '"' || next == '\\'))
                || (!in_single
                    && !in_double
                    && (next == '"' || next == '\'' || next == ' ' || next == '\t'))
            {
                token.push(next);
                i += 2;
                continue;
            }
        }
        if r == '\'' && !in_double {
            in_single = !in_single;
            i += 1;
            continue;
        }
        if r == '"' && !in_single {
            in_double = !in_double;
            i += 1;
            continue;
        }
        if (r == ' ' || r == '\t') && !in_single && !in_double {
            if !token.is_empty() {
                tokens.push(token.clone());
                token.clear();
            }
            i += 1;
            continue;
        }
        token.push(r);
        i += 1;
    }
    if !token.is_empty() {
        tokens.push(token);
    }
    tokens
}

pub fn persisted_tokless_command(exe: &str, args: &[&str]) -> String {
    if exe.is_empty() {
        return String::new();
    }
    let mut exe_str = exe.to_string();
    if IS_WIN {
        exe_str = exe_str.replace('\\', "/");
        if exe_str.contains([' ', '\t']) {
            exe_str = format!("\"{}\"", exe_str.replace('"', "\\\""));
        }
    } else if exe_str.contains([' ', '\t']) {
        exe_str = sh_quote(&exe_str);
    }

    let mut formatted_args = Vec::with_capacity(args.len());
    for &arg in args {
        if arg.contains([' ', '\t']) {
            if IS_WIN {
                formatted_args.push(format!("\"{}\"", arg.replace('"', "\\\"")));
            } else {
                formatted_args.push(sh_quote(arg));
            }
        } else {
            formatted_args.push(arg.to_string());
        }
    }

    if formatted_args.is_empty() {
        exe_str
    } else {
        format!("{} {}", exe_str, formatted_args.join(" "))
    }
}

pub fn tokless_data_dir() -> String {
    if let Some(ref h) = *HOME_OVERRIDE.read().unwrap() {
        if !h.is_empty() {
            if IS_WIN {
                return Path::new(h)
                    .join("AppData")
                    .join("Local")
                    .join("tokless")
                    .to_string_lossy()
                    .to_string();
            }
            return Path::new(h)
                .join(".local")
                .join("share")
                .join("tokless")
                .to_string_lossy()
                .to_string();
        }
    }
    if IS_WIN {
        let base = std::env::var("LOCALAPPDATA")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                Path::new(&home())
                    .join("AppData")
                    .join("Local")
                    .to_string_lossy()
                    .to_string()
            });
        return Path::new(&base).join("tokless").to_string_lossy().to_string();
    }
    Path::new(&home())
        .join(".local")
        .join("share")
        .join("tokless")
        .to_string_lossy()
        .to_string()
}

pub fn install_marker_path() -> String {
    Path::new(&tokless_data_dir())
        .join("install.json")
        .to_string_lossy()
        .to_string()
}

pub fn exists(p: impl AsRef<Path>) -> bool {
    p.as_ref().exists()
}

pub fn ensure_dir(p: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(p)
}

pub fn read_file_safe(p: impl AsRef<Path>) -> Option<String> {
    std::fs::read_to_string(p).ok()
}

pub fn remove_file_retry(p: impl AsRef<Path>) -> std::io::Result<()> {
    let p = p.as_ref();
    let mut err = match std::fs::remove_file(p) {
        Ok(()) => return Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => e,
    };
    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        match std::fs::remove_file(p) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => err = e,
        }
    }
    Err(err)
}

pub fn write_file_atomic(p: impl AsRef<Path>, content: &str) -> std::io::Result<()> {
    let p = p.as_ref();
    if let Some(ref f) = *WRITE_FILE_OVERRIDE.read().unwrap() {
        return f(&p.to_string_lossy(), content);
    }
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() {
            ensure_dir(parent)?;
        }
    }
    let parent = p
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temp = tempfile::Builder::new()
        .prefix(".tokless-write-")
        .tempfile_in(parent)?;
    use std::io::Write;
    temp.write_all(content.as_bytes())?;
    temp.flush()?;
    temp.as_file().sync_all()?;
    temp.persist(p).map_err(|e| e.error)?;
    Ok(())
}

pub fn write_file(p: impl AsRef<Path>, content: &str) -> std::io::Result<()> {
    let p = p.as_ref();
    if let Some(ref f) = *WRITE_FILE_OVERRIDE.read().unwrap() {
        return f(&p.to_string_lossy(), content);
    }
    if let Ok(meta) = std::fs::symlink_metadata(p) {
        if meta.file_type().is_symlink() {
            return std::fs::write(p, content);
        }
    }
    write_file_atomic(p, content)
}

pub fn write_file_mode(p: impl AsRef<Path>, content: &str, _mode: u32) -> std::io::Result<()> {
    let p = p.as_ref();
    write_file(p, content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(p) {
            let mut perms = meta.permissions();
            perms.set_mode(_mode);
            let _ = std::fs::set_permissions(p, perms);
        }
    }
    Ok(())
}

pub fn which(bin: &str) -> String {
    which::which(bin)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn stable_executable(path: &str) -> bool {
    if path.is_empty() || path.trim() != path {
        return false;
    }
    if path
        .chars()
        .any(|r| r.is_control() || (r.is_whitespace() && r != ' '))
    {
        return false;
    }
    let p = Path::new(path);
    if !p.is_absolute() {
        return false;
    }
    let Ok(meta) = std::fs::symlink_metadata(p) else {
        return false;
    };
    if !meta.file_type().is_file() {
        return false;
    }
    if IS_WIN {
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let pathext = std::env::var("PATHEXT")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| ".EXE;.CMD;.BAT;.COM".to_string());
        for candidate in pathext.split(';') {
            let clean = candidate.trim().trim_start_matches('.').to_lowercase();
            if !clean.is_empty() && ext == clean {
                return true;
            }
        }
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        return meta.permissions().mode() & 0o111 != 0;
    }
    #[cfg(not(unix))]
    true
}

pub fn is_test_executable(exe: &str) -> bool {
    let base = Path::new(exe)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    base.contains("test")
        || base.contains("deps")
        || exe.contains("deps")
        || exe.contains("go-build")
}

pub fn is_go_test_executable(exe: &str) -> bool {
    is_test_executable(exe)
}

pub fn tokless_abs() -> String {
    if let Ok(exe) = std::env::current_exe() {
        if let Ok(resolved) = std::fs::canonicalize(&exe) {
            let path_str = resolved.to_string_lossy().to_string();
            let clean_str = if IS_WIN && path_str.starts_with(r"\\?\") {
                path_str[4..].to_string()
            } else {
                path_str
            };
            if !clean_str.contains([' ', '\t']) && !is_test_executable(&clean_str) {
                return clean_str;
            }
        }
    }
    which_tokless_or_bare()
}

pub fn tokless_abs_strict() -> String {
    if let Ok(exe) = std::env::current_exe() {
        if let Ok(resolved) = std::fs::canonicalize(&exe) {
            let path_str = resolved.to_string_lossy().to_string();
            let clean_str = if IS_WIN && path_str.starts_with(r"\\?\") {
                path_str[4..].to_string()
            } else {
                path_str
            };
            if !clean_str.contains([' ', '\t']) && !is_test_executable(&clean_str) {
                return clean_str;
            }
        }
    }
    let w = which("tokless");
    if !w.is_empty() && Path::new(&w).is_absolute() {
        return w;
    }
    String::new()
}

pub fn tokless_persisted_abs() -> String {
    if let Some(raw) = read_file_safe(install_marker_path()) {
        if let Ok(marker) = serde_json::from_str::<InstallRecord>(&raw) {
            if stable_executable(&marker.path) {
                return marker.path;
            }
        }
    }
    let mut known = vec![
        Path::new(&home())
            .join(".local")
            .join("bin")
            .join("tokless")
            .to_string_lossy()
            .to_string(),
    ];
    if IS_WIN {
        known.clear();
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            if !local.is_empty() {
                known.push(
                    Path::new(&local)
                        .join("Programs")
                        .join("tokless")
                        .join("tokless.exe")
                        .to_string_lossy()
                        .to_string(),
                );
            }
        }
    } else {
        known.push("/usr/local/bin/tokless".to_string());
    }
    for path in known {
        if stable_executable(&path) {
            return path;
        }
    }
    let path = which("tokless");
    if Path::new(&path).is_absolute() && stable_executable(&path) {
        return path;
    }
    if std::env::var("TOKLESS_TEST").unwrap_or_default() == "1" {
        return "tokless".to_string();
    }
    String::new()
}

fn which_tokless_or_bare() -> String {
    let w = which("tokless");
    if !w.is_empty() {
        w
    } else {
        "tokless".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct InstallRecord {
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub at: String,
}

// Agent Paths

pub struct ClaudePaths {
    pub dir: String,
    pub settings: String,
    pub global_json: String,
    pub instructions: String,
}

pub fn claude_code_paths() -> ClaudePaths {
    let h = home();
    let (dir, global_json) = match std::env::var("CLAUDE_CONFIG_DIR") {
        Ok(d) if !d.is_empty() => {
            let gj = Path::new(&d).join(".claude.json").to_string_lossy().to_string();
            (d, gj)
        }
        _ => (
            Path::new(&h).join(".claude").to_string_lossy().to_string(),
            Path::new(&h).join(".claude.json").to_string_lossy().to_string(),
        ),
    };
    ClaudePaths {
        settings: Path::new(&dir).join("settings.json").to_string_lossy().to_string(),
        instructions: Path::new(&dir).join("CLAUDE.md").to_string_lossy().to_string(),
        dir,
        global_json,
    }
}

pub struct OpenCodePaths {
    pub dir: String,
    pub config: String,
    pub instructions: String,
    pub plugins_dir: String,
    pub rules_dir: String,
}

fn opencode_config_dir() -> String {
    if let Ok(d) = std::env::var("OPENCODE_CONFIG_DIR") {
        if !d.is_empty() {
            return d;
        }
    }
    if let Ok(x) = std::env::var("XDG_CONFIG_HOME") {
        if !x.is_empty() {
            return Path::new(&x).join("opencode").to_string_lossy().to_string();
        }
    }
    Path::new(&home())
        .join(".config")
        .join("opencode")
        .to_string_lossy()
        .to_string()
}

pub fn opencode_paths_resolved() -> OpenCodePaths {
    let dir = opencode_config_dir();
    let candidates = [
        Path::new(&dir).join("opencode.jsonc").to_string_lossy().to_string(),
        Path::new(&dir).join("opencode.json").to_string_lossy().to_string(),
        Path::new(&dir).join("config.json").to_string_lossy().to_string(),
    ];
    let mut config = Path::new(&dir).join("opencode.jsonc").to_string_lossy().to_string();
    if let Ok(custom) = std::env::var("OPENCODE_CONFIG") {
        if !custom.is_empty() {
            config = custom;
        } else {
            for c in candidates {
                if exists(&c) {
                    config = c;
                    break;
                }
            }
        }
    } else {
        for c in candidates {
            if exists(&c) {
                config = c;
                break;
            }
        }
    }
    OpenCodePaths {
        instructions: Path::new(&dir).join("AGENTS.md").to_string_lossy().to_string(),
        plugins_dir: Path::new(&dir).join("plugins").to_string_lossy().to_string(),
        rules_dir: Path::new(&dir).join("rules").to_string_lossy().to_string(),
        dir,
        config,
    }
}

pub fn open_code_paths_resolved() -> OpenCodePaths {
    opencode_paths_resolved()
}

pub fn opencode_config_dir_v2() -> String {
    let v1_dir = opencode_config_dir();
    let v1_path = Path::new(&v1_dir);
    let v2_name = format!("{}2", v1_path.file_name().and_then(|n| n.to_str()).unwrap_or("opencode"));
    let mut v2_dir = v1_path.parent().map(|p| p.join(&v2_name)).unwrap_or_else(|| Path::new(&v2_name).to_path_buf()).to_string_lossy().to_string();
    if let Ok(x) = std::env::var("XDG_CONFIG_HOME") {
        if !x.is_empty() {
            v2_dir = Path::new(&x).join("opencode2").to_string_lossy().to_string();
        }
    }
    let legacy_dir = Path::new(&home()).join(".opencode2").to_string_lossy().to_string();
    for dir in [&v1_dir, &v2_dir, &legacy_dir] {
        for name in ["opencode2.jsonc", "opencode2.json", "opencode2.config.json"] {
            if exists(Path::new(dir).join(name).to_string_lossy().as_ref()) {
                return dir.clone();
            }
        }
    }
    for dir in [&v1_dir, &v2_dir, &legacy_dir] {
        for name in ["opencode.jsonc", "opencode.json", "config.json"] {
            if exists(Path::new(dir).join(name).to_string_lossy().as_ref()) {
                return dir.clone();
            }
        }
        if dir == &v2_dir && exists(dir) {
            return dir.clone();
        }
    }
    v1_dir
}

pub fn opencode_paths_v2() -> OpenCodePaths {
    let dir = opencode_config_dir_v2();
    let candidates = [
        "opencode2.jsonc",
        "opencode2.json",
        "opencode2.config.json",
        "opencode.jsonc",
        "opencode.json",
        "config.json",
    ];
    let mut config = Path::new(&dir).join("opencode.jsonc").to_string_lossy().to_string();
    for name in candidates {
        let candidate = Path::new(&dir).join(name).to_string_lossy().to_string();
        if exists(&candidate) {
            config = candidate;
            break;
        }
    }
    OpenCodePaths {
        instructions: Path::new(&dir).join("AGENTS.md").to_string_lossy().to_string(),
        plugins_dir: Path::new(&dir).join("plugins").to_string_lossy().to_string(),
        rules_dir: Path::new(&dir).join("rules").to_string_lossy().to_string(),
        dir,
        config,
    }
}

pub struct KiloPaths {
    pub dir: String,
    pub config: String,
    pub plugins_dir: String,
    pub instructions: String,
}

pub fn kilo_paths_resolved() -> KiloPaths {
    let mut dir = std::env::var("KILO_CONFIG_DIR").unwrap_or_default();
    if dir.starts_with("~/") {
        dir = Path::new(&home()).join(&dir[2..]).to_string_lossy().to_string();
    }
    if dir.is_empty() {
        if let Ok(x) = std::env::var("XDG_CONFIG_HOME") {
            if !x.is_empty() {
                dir = Path::new(&x).join("kilo").to_string_lossy().to_string();
            }
        }
        if dir.is_empty() {
            dir = Path::new(&home())
                .join(".config")
                .join("kilo")
                .to_string_lossy()
                .to_string();
        }
    }
    if !Path::new(&dir).is_absolute() {
        if let Ok(cwd) = std::env::current_dir() {
            dir = cwd.join(&dir).to_string_lossy().to_string();
        }
    }
    KiloPaths {
        config: Path::new(&dir).join("kilo.jsonc").to_string_lossy().to_string(),
        plugins_dir: Path::new(&dir).join("plugin").to_string_lossy().to_string(),
        instructions: Path::new(&dir).join("AGENTS.md").to_string_lossy().to_string(),
        dir,
    }
}

pub struct ClinePaths {
    pub dir: String,
    pub data_dir: String,
    pub mcp_config: String,
    pub hooks_dir: String,
    pub rules_dir: String,
    pub instructions: String,
}

pub fn cline_paths_resolved() -> ClinePaths {
    let mut dir = std::env::var("CLINE_DIR").unwrap_or_default();
    if dir.starts_with("~/") {
        dir = Path::new(&home()).join(&dir[2..]).to_string_lossy().to_string();
    }
    if dir.is_empty() {
        dir = Path::new(&home()).join(".cline").to_string_lossy().to_string();
    }
    if !Path::new(&dir).is_absolute() {
        if let Ok(cwd) = std::env::current_dir() {
            dir = cwd.join(&dir).to_string_lossy().to_string();
        }
    }
    let mut data_dir = std::env::var("CLINE_DATA_DIR").unwrap_or_default();
    if data_dir.starts_with("~/") {
        data_dir = Path::new(&home()).join(&data_dir[2..]).to_string_lossy().to_string();
    }
    if data_dir.is_empty() {
        data_dir = Path::new(&dir).join("data").to_string_lossy().to_string();
    }
    let mut mcp_config = std::env::var("CLINE_MCP_SETTINGS_PATH").unwrap_or_default();
    if mcp_config.starts_with("~/") {
        mcp_config = Path::new(&home()).join(&mcp_config[2..]).to_string_lossy().to_string();
    }
    if mcp_config.is_empty() {
        mcp_config = Path::new(&data_dir)
            .join("settings")
            .join("cline_mcp_settings.json")
            .to_string_lossy()
            .to_string();
    }
    let rules_dir = Path::new(&dir).join("rules").to_string_lossy().to_string();
    ClinePaths {
        hooks_dir: Path::new(&dir).join("hooks").to_string_lossy().to_string(),
        instructions: Path::new(&rules_dir).join("AGENTS.md").to_string_lossy().to_string(),
        dir,
        data_dir,
        mcp_config,
        rules_dir,
    }
}

pub struct CodexPaths {
    pub dir: String,
    pub config: String,
    pub instructions: String,
}

pub fn codex_paths_resolved() -> CodexPaths {
    let dir = std::env::var("CODEX_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| Path::new(&home()).join(".codex").to_string_lossy().to_string());
    CodexPaths {
        config: Path::new(&dir).join("config.toml").to_string_lossy().to_string(),
        instructions: Path::new(&dir).join("AGENTS.md").to_string_lossy().to_string(),
        dir,
    }
}

pub struct AntigravityPaths {
    pub dir: String,
    pub mcp_config: String,
    pub mcp_config_ide: String,
    pub mcp_config_cli: String,
    pub settings: String,
    pub instructions: String,
}

pub fn antigravity_paths_resolved() -> AntigravityPaths {
    let gemini = Path::new(&home()).join(".gemini").to_string_lossy().to_string();
    let dir = Path::new(&gemini).join("antigravity").to_string_lossy().to_string();
    let mcp_config = Path::new(&dir).join("mcp_config.json").to_string_lossy().to_string();
    AntigravityPaths {
        mcp_config: mcp_config.clone(),
        mcp_config_ide: mcp_config,
        mcp_config_cli: Path::new(&gemini)
            .join("config")
            .join("mcp_config.json")
            .to_string_lossy()
            .to_string(),
        settings: Path::new(&gemini).join("settings.json").to_string_lossy().to_string(),
        instructions: Path::new(&gemini).join("GEMINI.md").to_string_lossy().to_string(),
        dir,
    }
}

pub struct CopilotPaths {
    pub dir: String,
    pub mcp_config: String,
    pub settings: String,
    pub hooks_dir: String,
    pub instructions: String,
}

pub fn copilot_paths_resolved() -> CopilotPaths {
    let mut dir = std::env::var("COPILOT_HOME").unwrap_or_default();
    if dir.is_empty() {
        if let Ok(x) = std::env::var("XDG_CONFIG_HOME") {
            if !x.is_empty() {
                dir = Path::new(&x).join("copilot").to_string_lossy().to_string();
            }
        }
        if dir.is_empty() {
            dir = Path::new(&home()).join(".copilot").to_string_lossy().to_string();
        }
    }
    CopilotPaths {
        mcp_config: Path::new(&dir).join("mcp-config.json").to_string_lossy().to_string(),
        settings: Path::new(&dir).join("settings.json").to_string_lossy().to_string(),
        hooks_dir: Path::new(&dir).join("hooks").to_string_lossy().to_string(),
        instructions: Path::new(&dir)
            .join("copilot-instructions.md")
            .to_string_lossy()
            .to_string(),
        dir,
    }
}

pub struct CursorPaths {
    pub dir: String,
    pub mcp_config: String,
    pub rules_dir: String,
    pub cli_config: String,
}

pub fn cursor_paths_resolved() -> CursorPaths {
    let mut dir = Path::new(&home()).join(".cursor").to_string_lossy().to_string();
    if !IS_WIN && std::env::consts::OS != "macos" {
        if let Ok(env) = std::env::var("CURSOR_CONFIG_DIR") {
            if !env.is_empty() {
                dir = env;
            }
        } else if let Ok(x) = std::env::var("XDG_CONFIG_HOME") {
            if !x.is_empty() {
                dir = Path::new(&x).join("cursor").to_string_lossy().to_string();
            }
        }
    }
    CursorPaths {
        mcp_config: Path::new(&dir).join("mcp.json").to_string_lossy().to_string(),
        rules_dir: Path::new(&dir).join("rules").to_string_lossy().to_string(),
        cli_config: Path::new(&dir).join("cli-config.json").to_string_lossy().to_string(),
        dir,
    }
}

pub fn cursor_global_mcp_path() -> String {
    cursor_paths_resolved().mcp_config
}

pub struct GrokPaths {
    pub dir: String,
    pub config: String,
    pub bin: String,
    pub install_dir: String,
}

pub fn grok_paths_resolved() -> GrokPaths {
    let dir = std::env::var("GROK_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| Path::new(&home()).join(".grok").to_string_lossy().to_string());
    let install_dir = Path::new(&home()).join(".local").join("bin").to_string_lossy().to_string();
    let bin_name = if IS_WIN { "grok.exe" } else { "grok" };
    GrokPaths {
        config: Path::new(&dir).join("config.toml").to_string_lossy().to_string(),
        bin: Path::new(&install_dir).join(bin_name).to_string_lossy().to_string(),
        install_dir,
        dir,
    }
}

pub struct OmpPaths {
    pub dir: String,
    pub agent_dir: String,
    pub config: String,
    pub rtk_extension: String,
}

pub fn omp_paths_resolved() -> OmpPaths {
    let dir = Path::new(&home()).join(".omp").to_string_lossy().to_string();
    let agent_dir = std::env::var("OMP_CODING_AGENT_DIR")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::env::var("PI_CODING_AGENT_DIR")
                .ok()
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| Path::new(&dir).join("agent").to_string_lossy().to_string());
    OmpPaths {
        config: Path::new(&agent_dir).join("mcp.json").to_string_lossy().to_string(),
        rtk_extension: Path::new(&agent_dir)
            .join("extensions")
            .join("tokless-rtk.ts")
            .to_string_lossy()
            .to_string(),
        agent_dir,
        dir,
    }
}

pub struct PiPaths {
    pub dir: String,
    pub agent_dir: String,
    pub settings: String,
    pub mcp_config: String,
    pub rtk_extension: String,
}

pub fn pi_paths_resolved() -> PiPaths {
    let dir = Path::new(&home()).join(".pi").to_string_lossy().to_string();
    let agent_dir = std::env::var("PI_CODING_AGENT_DIR")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| Path::new(&dir).join("agent").to_string_lossy().to_string());
    PiPaths {
        settings: Path::new(&agent_dir).join("settings.json").to_string_lossy().to_string(),
        mcp_config: Path::new(&agent_dir).join("mcp.json").to_string_lossy().to_string(),
        rtk_extension: Path::new(&agent_dir)
            .join("extensions")
            .join("rtk.ts")
            .to_string_lossy()
            .to_string(),
        agent_dir,
        dir,
    }
}

pub struct ZcodePaths {
    pub dir: String,
    pub config: String,
    pub instructions: String,
    pub mcp_config: String,
}

pub fn zcode_paths_resolved() -> ZcodePaths {
    let mut home_val = std::env::var("ZCODE_DIST_HOME").unwrap_or_default();
    if home_val.starts_with("~/") {
        home_val = Path::new(&home()).join(&home_val[2..]).to_string_lossy().to_string();
    }
    if home_val.is_empty() {
        home_val = Path::new(&home()).join(".zcode").join("runtime").to_string_lossy().to_string();
    }
    ZcodePaths {
        dir: home_val,
        config: Path::new(&home()).join(".zcode").join("cli").join("config.json").to_string_lossy().to_string(),
        instructions: Path::new(&home()).join(".zcode").join("AGENTS.md").to_string_lossy().to_string(),
        mcp_config: Path::new(&home()).join(".zcode").join("cli").join("config.json").to_string_lossy().to_string(),
    }
}

pub struct KimiPaths {
    pub dir: String,
    pub config: String,
    pub instructions: String,
    pub mcp_config: String,
}

pub fn kimi_paths_resolved() -> KimiPaths {
    let mut home_val = std::env::var("KIMI_CODE_HOME").unwrap_or_default();
    if home_val.starts_with("~/") {
        home_val = Path::new(&home()).join(&home_val[2..]).to_string_lossy().to_string();
    }
    if home_val.is_empty() {
        home_val = Path::new(&home()).join(".kimi-code").to_string_lossy().to_string();
    }
    KimiPaths {
        config: Path::new(&home_val).join("config.toml").to_string_lossy().to_string(),
        instructions: Path::new(&home_val).join("AGENTS.md").to_string_lossy().to_string(),
        mcp_config: Path::new(&home_val).join("mcp.json").to_string_lossy().to_string(),
        dir: home_val,
    }
}

pub struct AmpPaths {
    pub dir: String,
    pub config: String,
    pub instructions: String,
}

pub fn amp_paths_resolved() -> AmpPaths {
    let mut dir = std::env::var("AMP_CONFIG_DIR").unwrap_or_default();
    if dir.starts_with("~/") {
        dir = Path::new(&home()).join(&dir[2..]).to_string_lossy().to_string();
    }
    if dir.is_empty() {
        dir = Path::new(&home()).join(".config").join("amp").to_string_lossy().to_string();
    }
    AmpPaths {
        config: Path::new(&dir).join("settings.json").to_string_lossy().to_string(),
        instructions: Path::new(&dir).join("AGENTS.md").to_string_lossy().to_string(),
        dir,
    }
}

pub fn copy_dir_merge(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    if !src.exists() {
        return Ok(());
    }
    ensure_dir(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_merge(entry.path(), target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
