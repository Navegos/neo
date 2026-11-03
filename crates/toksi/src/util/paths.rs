// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/util/paths.rs
// created: 2026-10-05
// lastModified: 2026-11-02

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

pub fn persisted_toksi_command(exe: &str, args: &[&str]) -> String {
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

pub fn persisted_tokless_command(exe: &str, args: &[&str]) -> String {
    persisted_toksi_command(exe, args)
}

pub fn toksi_data_dir() -> String {
    let dir = if let Some(ref h) = *HOME_OVERRIDE.read().unwrap() {
        if !h.is_empty() {
            if IS_WIN {
                Path::new(h)
                    .join("AppData")
                    .join("Local")
                    .join("toksi")
                    .to_string_lossy()
                    .to_string()
            } else {
                Path::new(h)
                    .join(".local")
                    .join("share")
                    .join("toksi")
                    .to_string_lossy()
                    .to_string()
            }
        } else {
            default_toksi_data_dir()
        }
    } else {
        default_toksi_data_dir()
    };

    // Auto-migrate from legacy tokless data dir if it exists
    let p = Path::new(&dir);
    let legacy = dir.replace("toksi", "tokless");
    if legacy != dir {
        let lp = Path::new(&legacy);
        if lp.exists() {
            let _ = std::fs::create_dir_all(p);
            if let Ok(entries) = std::fs::read_dir(lp) {
                for entry in entries.flatten() {
                    let dest = p.join(entry.file_name());
                    if !dest.exists() {
                        let _ = std::fs::copy(entry.path(), dest);
                    }
                }
            }
        }
    }

    // Auto-migrate from ~/.tokless to ~/.toksi if ~/.tokless exists
    let h = home();
    if !h.is_empty() {
        let legacy_dot = Path::new(&h).join(".tokless");
        let toksi_dot = Path::new(&h).join(".toksi");
        if legacy_dot.exists() {
            let _ = std::fs::create_dir_all(&toksi_dot);
            if let Ok(entries) = std::fs::read_dir(&legacy_dot) {
                for entry in entries.flatten() {
                    let dest = toksi_dot.join(entry.file_name());
                    if !dest.exists() {
                        let _ = std::fs::copy(entry.path(), dest);
                    }
                }
            }
        }
    }
    dir
}

fn default_toksi_data_dir() -> String {
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
        return Path::new(&base).join("toksi").to_string_lossy().to_string();
    }
    Path::new(&home())
        .join(".local")
        .join("share")
        .join("toksi")
        .to_string_lossy()
        .to_string()
}

pub fn tokless_data_dir() -> String {
    toksi_data_dir()
}

pub fn install_marker_path() -> String {
    Path::new(&toksi_data_dir())
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
        .prefix(".toksi-write-")
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
    if bin.is_empty() {
        return String::new();
    }
    if let Some(real_path) = find_real_binary(bin, &Vec::<std::path::PathBuf>::new()) {
        return real_path.to_string_lossy().to_string();
    }
    which::which(bin)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Possible candidate extensions for binary, executable, and script lookups.
pub const CANDIDATE_EXTENSIONS: &[&str] = &[
    "exe", "cmd", "bat", "ps1", "sh", "zsh", "app", "command",
];

/// Returns the prioritized candidate binary extensions for the current platform:
/// - Windows: exe, cmd, bat, ps1
/// - Linux: sh, zsh, ps1 (and bare executable "")
/// - macOS: app, command, sh, zsh, ps1 (and bare executable "")
pub fn platform_candidate_extensions() -> &'static [&'static str] {
    if cfg!(target_os = "windows") {
        &["exe", "cmd", "bat", "ps1"]
    } else if cfg!(target_os = "macos") {
        &["app", "command", "sh", "zsh", "ps1", ""]
    } else {
        &["", "sh", "zsh", "ps1"]
    }
}

/// Returns known binary aliases for agent and tool executables.
pub fn known_binary_aliases(name: &str) -> Vec<String> {
    let trimmed = name.trim();
    let lower = trimmed.to_ascii_lowercase();
    match lower.as_str() {
        "antigravity ide" | "antigravity-ide" | "antigravity_ide" | "antigravityide" => vec![
            "Antigravity IDE".to_string(),
            "antigravity-ide".to_string(),
            "antigravity_ide".to_string(),
            "Antigravity".to_string(),
            "antigravity".to_string(),
            "agy".to_string(),
        ],
        "antigravity" => vec![
            "Antigravity".to_string(),
            "antigravity".to_string(),
            "Antigravity IDE".to_string(),
            "antigravity-ide".to_string(),
            "agy".to_string(),
        ],
        "agy" => vec![
            "agy".to_string(),
            "Antigravity".to_string(),
            "antigravity".to_string(),
            "Antigravity IDE".to_string(),
            "antigravity-ide".to_string(),
        ],
        "claude" | "claude-code" => vec![
            "claude".to_string(),
            "claude-code".to_string(),
            "Claude".to_string(),
        ],
        "codex" => vec![
            "codex".to_string(),
            "Codex".to_string(),
        ],
        "copilot" | "github-copilot" => vec![
            "copilot".to_string(),
            "github-copilot".to_string(),
            "Copilot".to_string(),
        ],
        "cursor" | "cursor-agent" | "agent" => vec![
            "cursor".to_string(),
            "cursor-agent".to_string(),
            "agent".to_string(),
            "Cursor".to_string(),
        ],
        "factory" | "droid" => vec![
            "droid".to_string(),
            "Factory".to_string(),
            "factory".to_string(),
        ],
        "grok" => vec![
            "grok".to_string(),
            "Grok".to_string(),
        ],
        "opencode" => vec![
            "opencode".to_string(),
            "OpenCode".to_string(),
            "opencode2".to_string(),
        ],
        "opencode2" | "opencoe2" => vec![
            "opencode2".to_string(),
            "opencoe2".to_string(),
            "opencode".to_string(),
            "OpenCode".to_string(),
        ],
        "toksi" => vec![
            "toksi".to_string(),
        ],
        "tokless" => vec![
            "tokless".to_string(),
        ],
        _ => vec![trimmed.to_string()],
    }
}

/// Tests whether a candidate path points to a real executable or script,
/// rejecting 0-byte placeholders, Toksi shims, hijack wrappers, and corrupted files.
pub fn is_real_binary(path: &Path) -> bool {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return false;
    };
    if meta.file_type().is_dir() {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext == "app" {
            let macos_dir = path.join("Contents").join("MacOS");
            if macos_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(macos_dir) {
                    for entry in entries.flatten() {
                        if entry.path().is_file()
                            && entry.metadata().map(|m| m.len() > 0).unwrap_or(false)
                        {
                            return true;
                        }
                    }
                }
            }
        }
        return false;
    }
    if !meta.file_type().is_file() {
        return false;
    }
    if meta.len() == 0 {
        return false;
    }

    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if file_name.ends_with(".toksi")
        || file_name.ends_with(".tokless")
        || file_name.ends_with(".digest")
        || file_name.ends_with(".removing")
    {
        return false;
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if meta.len() <= 128 * 1024 {
        if let Ok(bytes) = std::fs::read(path) {
            if (ext == "exe" || (IS_WIN && ext.is_empty())) && bytes.len() >= 2 {
                if bytes[0] != 0x4D || bytes[1] != 0x5A {
                    if ext == "exe" {
                        return false;
                    }
                } else {
                    return true;
                }
            }

            let check_len = bytes.len().min(16 * 1024);
            if let Ok(text) = std::str::from_utf8(&bytes[..check_len]) {
                const SHIM_MARKERS: &[&str] = &[
                    "toksi:antigravity-cli-proxy",
                    "toksi:grok-cli-proxy",
                    "toksi:copilot-shim",
                    "toksi:antigravity-launcher",
                    "toksi:grok-launcher",
                    "__proxy-ensure",
                    "__grok-proxy-ensure",
                    "__copilot",
                    "rem toksi:",
                    "# toksi:",
                    "// toksi:",
                    "/* toksi:",
                    "TOKSI_ANTIGRAVITY_HEADROOM_MANAGED",
                ];
                for marker in SHIM_MARKERS {
                    if text.contains(marker) {
                        return false;
                    }
                }
                if (ext == "cmd" || ext == "bat")
                    && text.starts_with("@echo off")
                    && (text.contains("%REAL%")
                        || text.contains("GOOGLE_GEMINI_BASE_URL")
                        || text.contains("HEADROOM_WRAP_SILENT"))
                {
                    return false;
                }
                if (ext == "sh" || ext == "zsh" || ext == "bash" || ext == "command" || ext.is_empty())
                    && text.starts_with("#!/")
                    && (text.contains("exec \"$REAL\"")
                        || text.contains("toksi")
                        || text.contains("tokless"))
                {
                    return false;
                }
                if ext == "ps1"
                    && (text.contains("$REAL") || text.contains("toksi") || text.contains("tokless"))
                    && text.contains("headroom")
                {
                    return false;
                }
            }
        }
    } else if ext == "exe" || (IS_WIN && ext.is_empty()) {
        if let Ok(mut f) = std::fs::File::open(path) {
            use std::io::Read;
            let mut magic = [0u8; 2];
            if f.read_exact(&mut magic).is_ok() && &magic == b"MZ" {
                return true;
            }
            if ext == "exe" {
                return false;
            }
        }
    }

    if ext == "cmd"
        || ext == "bat"
        || ext == "ps1"
        || ext == "sh"
        || ext == "zsh"
        || ext == "bash"
        || ext == "command"
    {
        return true;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o111 != 0 {
            return true;
        }
    }

    true
}

/// Collects directories from the PATH environment variable.
pub fn path_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();
    let path_val = std::env::var("PATH").or_else(|_| std::env::var("Path")).unwrap_or_default();
    let sep = if IS_WIN { ';' } else { ':' };
    for part in path_val.split(sep) {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            dirs.push(std::path::PathBuf::from(trimmed));
        }
    }
    dirs
}

/// Finds a real binary for the given name or alias across extra directories and PATH,
/// checking candidate extensions: ps1, cmd, bat, exe, bash, sh, app, and no extension.
pub fn find_real_binary(bin: &str, extra_dirs: &[impl AsRef<Path>]) -> Option<std::path::PathBuf> {
    if bin.is_empty() {
        return None;
    }

    let direct = Path::new(bin);
    if (direct.is_absolute() || bin.contains('/') || bin.contains('\\')) && is_real_binary(direct) {
        return Some(direct.to_path_buf());
    }

    let mut search_dirs = Vec::new();
    let mut seen_dirs = std::collections::HashSet::new();

    for d in extra_dirs {
        let p = d.as_ref();
        if !p.as_os_str().is_empty() && p.is_dir() {
            let canon = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
            if seen_dirs.insert(canon) {
                search_dirs.push(p.to_path_buf());
            }
        }
    }

    for p in path_dirs() {
        if p.is_dir() {
            let canon = p.canonicalize().unwrap_or_else(|_| p.clone());
            if seen_dirs.insert(canon) {
                search_dirs.push(p);
            }
        }
    }

    let names = known_binary_aliases(bin);
    let mut extensions: Vec<&str> = platform_candidate_extensions().to_vec();
    for &ext in CANDIDATE_EXTENSIONS {
        if !extensions.contains(&ext) {
            extensions.push(ext);
        }
    }
    if !extensions.contains(&"") {
        extensions.push("");
    }

    let direct_ext = Path::new(bin)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !direct_ext.is_empty() {
        extensions.insert(0, "");
    }

    for dir in &search_dirs {
        for name in &names {
            for ext in &extensions {
                let candidate = if ext.is_empty() {
                    dir.join(name)
                } else {
                    dir.join(format!("{name}.{ext}"))
                };
                if is_real_binary(&candidate) {
                    return Some(candidate);
                }
            }
        }
    }

    None
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
    is_real_binary(p)
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

pub fn toksi_abs() -> String {
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
    which_toksi_or_bare()
}

pub fn tokless_abs() -> String {
    toksi_abs()
}

pub fn toksi_abs_strict() -> String {
    if let Ok(exe) = std::env::current_exe() {
        if let Ok(resolved) = std::fs::canonicalize(&exe) {
            let path_str = resolved.to_string_lossy().to_string();
            let clean_str = if IS_WIN && path_str.starts_with(r"\\?\") {
                path_str[4..].to_string()
            } else {
                path_str
            };
            if !clean_str.contains([' ', '\t']) && !is_test_executable(&clean_str) {
                let stem = Path::new(&clean_str)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if stem == "toksi" {
                    return clean_str;
                }
            }
        }
    }
    let w = which("toksi");
    if !w.is_empty() && Path::new(&w).is_absolute() {
        return w;
    }
    String::new()
}

pub fn tokless_abs_strict() -> String {
    toksi_abs_strict()
}

pub fn toksi_persisted_abs() -> String {
    if let Ok(cur) = std::env::current_exe() {
        let s = cur.to_string_lossy().to_string();
        if stable_executable(&s) {
            let stem = cur
                .file_stem()
                .and_then(|x| x.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if stem == "toksi" {
                return s;
            }
        }
    }
    if let Some(raw) = read_file_safe(install_marker_path()) {
        if let Ok(marker) = serde_json::from_str::<InstallRecord>(&raw) {
            if stable_executable(&marker.path) {
                let base = Path::new(&marker.path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if base == "toksi" {
                    return marker.path;
                }
            }
        }
    }
    let mut known = vec![
        Path::new(&home())
            .join(".local")
            .join("bin")
            .join("toksi")
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
                        .join("toksi")
                        .join("toksi.exe")
                        .to_string_lossy()
                        .to_string(),
                );
            }
        }
    } else {
        known.push("/usr/local/bin/toksi".to_string());
    }
    for path in known {
        if stable_executable(&path) {
            return path;
        }
    }
    let path = which("toksi");
    if Path::new(&path).is_absolute() && stable_executable(&path) {
        return path;
    }
    if std::env::var("TOKSI_TEST").unwrap_or_default() == "1"
        || std::env::var("TOKLESS_TEST").unwrap_or_default() == "1"
    {
        return "toksi".to_string();
    }
    String::new()
}

pub fn tokless_persisted_abs() -> String {
    toksi_persisted_abs()
}

fn which_toksi_or_bare() -> String {
    let w = which("toksi");
    if !w.is_empty() {
        w
    } else {
        "toksi".to_string()
    }
}

#[allow(dead_code)]
fn which_tokless_or_bare() -> String {
    which_toksi_or_bare()
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
            .join("toksi-rtk.ts")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_binary_aliases() {
        let aliases = known_binary_aliases("Antigravity IDE");
        assert!(aliases.contains(&"agy".to_string()));
        assert!(aliases.contains(&"Antigravity".to_string()));

        let cursor_aliases = known_binary_aliases("cursor");
        assert!(cursor_aliases.contains(&"agent".to_string()));
        assert!(cursor_aliases.contains(&"cursor-agent".to_string()));

        let droid_aliases = known_binary_aliases("droid");
        assert!(droid_aliases.contains(&"Factory".to_string()));

        let opencode_aliases = known_binary_aliases("opencoe2");
        assert!(opencode_aliases.contains(&"opencode2".to_string()));

        let toksi_aliases = known_binary_aliases("toksi");
        assert!(toksi_aliases.contains(&"toksi".to_string()));
        assert!(!toksi_aliases.contains(&"tokless".to_string()));

        let tokless_aliases = known_binary_aliases("tokless");
        assert!(tokless_aliases.contains(&"tokless".to_string()));
        assert!(!tokless_aliases.contains(&"toksi".to_string()));
    }

    #[test]
    fn test_is_real_binary_and_find_real_binary() {
        let temp_dir = std::env::temp_dir().join(format!(
            "toksi-path-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);

        // 1. Zero-byte file should NOT be real
        let zero_byte = temp_dir.join("zero.exe");
        std::fs::write(&zero_byte, b"").unwrap();
        assert!(!is_real_binary(&zero_byte));

        // 2. Fake .exe without MZ magic should NOT be real
        let fake_exe = temp_dir.join("fake.exe");
        std::fs::write(&fake_exe, b"NOT_AN_EXE").unwrap();
        assert!(!is_real_binary(&fake_exe));

        // 3. Real .exe with MZ header
        let real_exe = temp_dir.join("real.exe");
        std::fs::write(&real_exe, b"MZ\x90\x00\x03\x00\x00\x00").unwrap();
        assert!(is_real_binary(&real_exe));

        // 4. Shim batch file should NOT be real
        let shim_bat = temp_dir.join("agy.cmd");
        std::fs::write(
            &shim_bat,
            b"@echo off\r\nrem toksi:antigravity-cli-proxy\r\nset \"REAL=agy.real.exe\"\r\n\"%REAL%\" %*\r\n",
        )
        .unwrap();
        assert!(!is_real_binary(&shim_bat));

        // 5. Real PowerShell script
        let real_ps1 = temp_dir.join("agy.ps1");
        std::fs::write(&real_ps1, b"Write-Output 'Antigravity CLI'\n").unwrap();
        assert!(is_real_binary(&real_ps1));

        // 6. Real Shell script
        let real_sh = temp_dir.join("opencode.sh");
        std::fs::write(&real_sh, b"#!/bin/sh\necho 'OpenCode'\n").unwrap();
        assert!(is_real_binary(&real_sh));

        // 7. Real Zsh script
        let real_zsh = temp_dir.join("claude.zsh");
        std::fs::write(&real_zsh, b"#!/bin/zsh\necho 'Claude'\n").unwrap();
        assert!(is_real_binary(&real_zsh));

        // 8. Real macOS .command script
        let real_command = temp_dir.join("cursor.command");
        std::fs::write(&real_command, b"#!/bin/sh\necho 'Cursor'\n").unwrap();
        assert!(is_real_binary(&real_command));

        // 9. Find real binary in directory with extensions
        let found_agy = find_real_binary("agy", &[&temp_dir]);
        assert_eq!(found_agy, Some(real_ps1));

        let found_opencode = find_real_binary("opencode", &[&temp_dir]);
        assert_eq!(found_opencode, Some(real_sh));

        let found_claude = find_real_binary("claude", &[&temp_dir]);
        assert_eq!(found_claude, Some(real_zsh));

        let found_cursor = find_real_binary("cursor", &[&temp_dir]);
        assert_eq!(found_cursor, Some(real_command));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
