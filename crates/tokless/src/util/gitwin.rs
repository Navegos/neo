use std::path::Path;
use std::time::Duration;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::exec::prepend_process_path;
use super::logger::L;
use super::nodewin::{download_to_temp, extract_zip_flat};
use super::paths::{exists, home};
use super::pathsetup::persist_windows_path_dirs;

pub const GIT_RELEASES_URL: &str = "https://api.github.com/repos/git-for-windows/git/releases/latest";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GhAsset {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
struct GhRelease {
    #[serde(default)]
    assets: Vec<GhReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct GhReleaseAsset {
    name: String,
    browser_download_url: String,
}

fn asset_list(rel: GhRelease) -> Vec<GhAsset> {
    rel.assets
        .into_iter()
        .map(|a| GhAsset {
            name: a.name,
            url: a.browser_download_url,
        })
        .collect()
}

pub fn pick_min_git_asset(assets: &[GhAsset]) -> Option<String> {
    let min_git_busybox = Regex::new(r"^MinGit-.*-busybox-64-bit\.zip$").ok()?;
    let min_git_plain = Regex::new(r"^MinGit-.*-64-bit\.zip$").ok()?;

    for a in assets {
        if min_git_busybox.is_match(&a.name) {
            return Some(a.url.clone());
        }
    }
    for a in assets {
        if min_git_plain.is_match(&a.name) {
            return Some(a.url.clone());
        }
    }
    None
}

pub fn min_git_download_url() -> Option<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("tokless")
        .build()
        .ok()?;
    let resp = client.get(GIT_RELEASES_URL).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let rel: GhRelease = resp.json().ok()?;
    pick_min_git_asset(&asset_list(rel))
}

pub fn git_install_dir() -> String {
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
        .join("git")
        .to_string_lossy()
        .to_string()
}

pub fn install_git_windows_zip() -> bool {
    let Some(url) = min_git_download_url() else {
        L.err("couldn't resolve a MinGit download from git-for-windows releases");
        return false;
    };
    L.info("Downloading MinGit from github.com/git-for-windows…");
    let zip_path = match download_to_temp(&url) {
        Ok(p) => p,
        Err(err) => {
            L.err(&format!("MinGit download failed: {}", err));
            return false;
        }
    };
    let dest = git_install_dir();
    let _ = std::fs::remove_dir_all(&dest);
    let extract_res = extract_zip_flat(&zip_path, &dest);
    let _ = std::fs::remove_file(&zip_path);
    if let Err(err) = extract_res {
        L.err(&format!("MinGit extract failed: {}", err));
        return false;
    }
    let bin = Path::new(&dest).join("cmd");
    if !exists(bin.join("git.exe")) {
        L.err("MinGit zip didn't contain cmd/git.exe");
        return false;
    }
    let bin_str = bin.to_string_lossy().to_string();
    prepend_process_path(&bin_str);
    persist_windows_path_dirs(&[bin_str]);
    true
}
