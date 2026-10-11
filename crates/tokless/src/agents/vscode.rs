use std::collections::HashMap;
use std::path::Path;

use crate::core::Detection;
use crate::util::exec::find_binary;
use crate::util::paths::{exists, home};

fn detect_agent_base(
    cli: &str,
    config_dir: &str,
    known_dirs: &[String],
    desktop_paths: &[String],
) -> Detection {
    let dir_refs: Vec<&str> = known_dirs.iter().map(|s| s.as_str()).collect();
    let has_cli = !find_binary(cli, &dir_refs).is_empty();
    let has_desktop = desktop_paths.iter().any(|p| exists(p));

    if has_cli && has_desktop {
        return Detection {
            installed: true,
            source: "cli+desktop".into(),
        };
    }
    if has_cli {
        return Detection {
            installed: true,
            source: "cli".into(),
        };
    }
    if has_desktop {
        return Detection {
            installed: true,
            source: "desktop".into(),
        };
    }
    if std::env::var("TOKLESS_TEST").unwrap_or_default() == "1" && exists(config_dir) {
        return Detection {
            installed: true,
            source: "config".into(),
        };
    }
    Detection {
        installed: false,
        source: String::new(),
    }
}

pub fn detect_vscode_agent(
    cli: &str,
    config_dir: &str,
    known_dirs: &[String],
    extension_id: &str,
) -> Detection {
    let detection = detect_agent_base(cli, config_dir, known_dirs, &[]);
    if !vscode_extension_installed(extension_id) {
        return detection;
    }
    if detection.source == "cli" {
        return Detection {
            installed: true,
            source: "cli+ide".into(),
        };
    }
    Detection {
        installed: true,
        source: "ide".into(),
    }
}

pub fn vscode_extension_installed(id: &str) -> bool {
    let prefix = format!("{}-", id);
    for dir in vscode_extension_dirs() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        let obsolete = vscode_obsolete_extensions(&dir);
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let is_obsolete = obsolete.get(&name).copied().unwrap_or(false);
            if is_dir && !is_obsolete && (name == id || name.starts_with(&prefix)) {
                return true;
            }
        }
    }
    false
}

pub fn vscode_obsolete_extensions(dir: &str) -> HashMap<String, bool> {
    let path = Path::new(dir).join(".obsolete");
    let raw = match std::fs::read_to_string(path) {
        Ok(r) => r,
        Err(_) => return HashMap::new(),
    };
    let parsed: HashMap<String, bool> = match serde_json::from_str(&raw) {
        Ok(m) => m,
        Err(_) => return HashMap::new(),
    };
    parsed
        .into_iter()
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect()
}

pub fn vscode_extension_dirs() -> Vec<String> {
    let h = home();
    let code_server_data = std::env::var("XDG_DATA_HOME")
        .ok()
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| {
            Path::new(&h)
                .join(".local")
                .join("share")
                .to_string_lossy()
                .to_string()
        });
    let mut dirs = vec![
        Path::new(&h)
            .join(".vscode")
            .join("extensions")
            .to_string_lossy()
            .to_string(),
        Path::new(&h)
            .join(".vscode-insiders")
            .join("extensions")
            .to_string_lossy()
            .to_string(),
        Path::new(&h)
            .join(".vscode-oss")
            .join("extensions")
            .to_string_lossy()
            .to_string(),
        Path::new(&h)
            .join(".vscode-server")
            .join("extensions")
            .to_string_lossy()
            .to_string(),
        Path::new(&h)
            .join(".vscode-server-insiders")
            .join("extensions")
            .to_string_lossy()
            .to_string(),
        Path::new(&h)
            .join(".windsurf")
            .join("extensions")
            .to_string_lossy()
            .to_string(),
        Path::new(&h)
            .join(".cursor")
            .join("extensions")
            .to_string_lossy()
            .to_string(),
        Path::new(&code_server_data)
            .join("code-server")
            .join("extensions")
            .to_string_lossy()
            .to_string(),
    ];
    if let Ok(dir) = std::env::var("VSCODE_EXTENSIONS") {
        if !dir.is_empty() {
            dirs.push(dir);
        }
    }
    dirs
}
