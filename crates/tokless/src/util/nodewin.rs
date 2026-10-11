use std::io::Write;
use std::path::Path;
use std::time::Duration;
use serde::{Deserialize, Serialize};

use super::exec::{prepend_process_path, run, RunOptions};
use super::logger::L;
use super::paths::{copy_dir_merge, ensure_dir, home, which, IS_WIN};
use super::pathsetup::persist_windows_path_dirs;

pub const NODE_DIST_INDEX_URL: &str = "https://nodejs.org/dist/index.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDistEntry {
    pub version: String,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub lts: serde_json::Value,
}

pub fn node_win_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" | "amd64" => "x64",
        "aarch64" | "arm64" => "arm64",
        "x86" | "386" => "x86",
        _ => "",
    }
}

pub fn node_lts_version(entries: &[NodeDistEntry], want: &str) -> Option<String> {
    for e in entries {
        let is_lts = match &e.lts {
            serde_json::Value::Null => false,
            serde_json::Value::Bool(b) => *b,
            serde_json::Value::String(s) => {
                let t = s.trim();
                !t.is_empty() && t != "false" && t != "null"
            }
            _ => true,
        };
        if !is_lts {
            continue;
        }
        if e.files.iter().any(|f| f == want) {
            return Some(e.version.clone());
        }
    }
    None
}

pub fn fetch_node_index() -> Option<Vec<NodeDistEntry>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("tokless")
        .build()
        .ok()?;
    let resp = client.get(NODE_DIST_INDEX_URL).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json::<Vec<NodeDistEntry>>().ok()
}

pub fn download_to_temp(url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(600))
        .user_agent("tokless")
        .build()?;
    let resp = client.get(url).send()?;
    if !resp.status().is_success() {
        return Err(format!("download failed ({}): {}", resp.status(), url).into());
    }
    let suffix = if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
        ".tar.gz"
    } else {
        ".zip"
    };
    let mut temp = tempfile::Builder::new()
        .prefix("tokless-node-")
        .suffix(suffix)
        .tempfile()?;
    let bytes = resp.bytes()?;
    temp.write_all(&bytes)?;
    temp.flush()?;
    let (_, path) = temp.keep()?;
    Ok(path.to_string_lossy().to_string())
}

pub fn extract_zip_strip_root(
    zip_path: &str,
    dest: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    extract_zip(zip_path, dest, true)
}

pub fn extract_zip_flat(
    zip_path: &str,
    dest: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    extract_zip(zip_path, dest, false)
}

fn extract_zip(
    zip_path: &str,
    dest: &str,
    strip_root: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ensure_dir(dest)?;
    if !strip_root {
        return unpack_zip_to_dir(zip_path, dest);
    }
    let stage = tempfile::Builder::new()
        .prefix("tokless-zip-stage-")
        .tempdir()?;
    let stage_path = stage.path().to_string_lossy().to_string();
    unpack_zip_to_dir(zip_path, &stage_path)?;
    for entry in std::fs::read_dir(stage.path())? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            for child in std::fs::read_dir(&p)? {
                let child = child?;
                let target = Path::new(dest).join(child.file_name());
                if std::fs::rename(child.path(), &target).is_err() {
                    if child.file_type()?.is_dir() {
                        copy_dir_merge(child.path(), &target)?;
                    } else {
                        std::fs::copy(child.path(), &target)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn unpack_zip_to_dir(
    zip_path: &str,
    dest: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ensure_dir(dest)?;
    if IS_WIN {
        let ps = format!(
            "$ErrorActionPreference='Stop'; Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
            zip_path.replace('\'', "''"),
            dest.replace('\'', "''")
        );
        let r = run(
            "powershell",
            &["-NoProfile", "-Command", &ps],
            RunOptions {
                capture: true,
                ..Default::default()
            },
        );
        if r.code == 0 {
            return Ok(());
        }
        let r_tar = run(
            "tar",
            &["-xf", zip_path, "-C", dest],
            RunOptions {
                capture: true,
                ..Default::default()
            },
        );
        if r_tar.code == 0 {
            return Ok(());
        }
        return Err(format!("zip extract failed: {}", r.stderr.trim()).into());
    }
    let r = run(
        "unzip",
        &["-q", "-o", zip_path, "-d", dest],
        RunOptions {
            capture: true,
            ..Default::default()
        },
    );
    if r.code == 0 {
        return Ok(());
    }
    let r_tar = run(
        "tar",
        &["-xf", zip_path, "-C", dest],
        RunOptions {
            capture: true,
            ..Default::default()
        },
    );
    if r_tar.code == 0 {
        return Ok(());
    }
    Err(format!("zip extract failed: {}", r.stderr.trim()).into())
}

pub fn node_install_dir() -> String {
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
    Path::new(&base)
        .join("tokless")
        .join("node")
        .to_string_lossy()
        .to_string()
}

pub fn install_node_windows_zip() -> bool {
    let arch = node_win_arch();
    if arch.is_empty() {
        L.err(&format!(
            "unsupported Windows architecture for Node: {}",
            std::env::consts::ARCH
        ));
        return false;
    }
    let Some(entries) = fetch_node_index() else {
        L.err("couldn't reach nodejs.org to resolve the Node LTS version");
        return false;
    };
    let want = format!("win-{}-zip", arch);
    let Some(v) = node_lts_version(&entries, &want) else {
        L.err(&format!("no Node LTS zip available for win-{}", arch));
        return false;
    };
    let url = format!("https://nodejs.org/dist/{}/node-{}-win-{}.zip", v, v, arch);
    L.info(&format!("Downloading Node.js {} from nodejs.org…", v));
    let tmp = match download_to_temp(&url) {
        Ok(p) => p,
        Err(err) => {
            L.err(&format!("Node download failed: {}", err));
            return false;
        }
    };
    let dest = node_install_dir();
    let _ = std::fs::remove_dir_all(&dest);
    let unpack_res = extract_zip_strip_root(&tmp, &dest);
    let _ = std::fs::remove_file(&tmp);
    if let Err(err) = unpack_res {
        L.err(&format!("Node unpack failed: {}", err));
        return false;
    }
    prepend_process_path(&dest);
    persist_windows_path_dirs(std::slice::from_ref(&dest));
    !which("node").is_empty() && !which("npm").is_empty()
}
