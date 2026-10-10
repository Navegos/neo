use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use serde::{Deserialize, Serialize};

use super::paths::{home, read_file_safe, write_file};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VersionInfo {
    pub installed: Option<String>,
    pub latest: Option<String>,
    pub channel: String,
    pub present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheShape {
    pub ts: i64,
    pub map: HashMap<String, VersionInfo>,
}

pub fn version_cache_path() -> String {
    let h = home();
    if h.is_empty() {
        return String::new();
    }
    Path::new(&h).join(".cache").join("tokless").join("versions.json").to_string_lossy().to_string()
}

pub fn load_cache() -> (Option<HashMap<String, VersionInfo>>, bool) {
    let p = version_cache_path();
    if let Some(raw) = read_file_safe(p) {
        if let Ok(obj) = serde_json::from_str::<CacheShape>(&raw) {
            let now = chrono::Utc::now().timestamp_millis();
            let fresh = (now - obj.ts) <= 6 * 3600 * 1000;
            return (Some(obj.map), fresh);
        }
    }
    (None, false)
}

pub fn save_cache(m: &HashMap<String, VersionInfo>) {
    let p = version_cache_path();
    if p.is_empty() {
        return;
    }
    let obj = CacheShape {
        ts: chrono::Utc::now().timestamp_millis(),
        map: m.clone(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&obj) {
        let _ = write_file(p, &json);
    }
}

pub fn semver_compare(a: &str, b: &str) -> i32 {
    let parse_nums = |s: &str| -> Vec<u64> {
        let clean = s.trim_start_matches('v');
        clean.split('.').filter_map(|p| p.parse::<u64>().ok()).collect()
    };
    let a_parts = parse_nums(a);
    let b_parts = parse_nums(b);
    for i in 0..std::cmp::max(a_parts.len(), b_parts.len()) {
        let an = a_parts.get(i).copied().unwrap_or(0);
        let bn = b_parts.get(i).copied().unwrap_or(0);
        if an < bn {
            return -1;
        }
        if an > bn {
            return 1;
        }
    }
    0
}

pub fn fetch_json<T: for<'de> Deserialize<'de>>(url: &str) -> Option<T> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("tokless")
        .build()
        .ok()?;
    let resp = client.get(url).header("Accept", "application/json").send().ok()?;
    if resp.status().is_success() {
        resp.json::<T>().ok()
    } else {
        None
    }
}

pub fn pypi_latest(pkg: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct PyPIDoc {
        info: PyPIInfo,
    }
    #[derive(Deserialize)]
    struct PyPIInfo {
        version: String,
    }
    let url = format!("https://pypi.org/pypi/{}/json", pkg);
    let doc: PyPIDoc = fetch_json(&url)?;
    Some(doc.info.version)
}

pub fn npm_latest(pkg: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct NpmDoc {
        #[serde(rename = "dist-tags")]
        dist_tags: HashMap<String, String>,
    }
    let url = format!("https://registry.npmjs.org/{}", pkg);
    let doc: NpmDoc = fetch_json(&url)?;
    doc.dist_tags.get("latest").cloned()
}

pub fn github_latest(repo: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct GhDoc {
        tag_name: String,
    }
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let doc: GhDoc = fetch_json(&url)?;
    Some(doc.tag_name.trim_start_matches('v').to_string())
}

pub fn installed_version_for(id: &str) -> Option<String> {
    match id {
        "rtk" => {
            let p = crate::util::exec::resolve_rtk_bin();
            if !p.is_empty() {
                let r = crate::util::exec::run(&p, &["--version"], crate::util::exec::RunOptions { capture: true, ..Default::default() });
                if r.code == 0 {
                    return r.stdout.split_whitespace().last().map(|s| s.trim_start_matches('v').to_string());
                }
            }
            None
        }
        "headroom" => {
            let p = crate::util::headroom::headroom_bin();
            if !p.is_empty() {
                let r = crate::util::exec::run(&p, &["--version"], crate::util::exec::RunOptions { capture: true, ..Default::default() });
                if r.code == 0 {
                    return r.stdout.split_whitespace().last().map(|s| s.trim_start_matches('v').to_string());
                }
            }
            None
        }
        _ => None,
    }
}

pub fn latest_version_for(id: &str) -> Option<String> {
    match id {
        "rtk" => github_latest("rtk-ai/rtk"),
        "codegraph" => npm_latest("@colbymchenry/codegraph"),
        "context-mode" => npm_latest("context-mode"),
        "headroom" => pypi_latest("headroom-ai"),
        "tokless" => github_latest("HoangP8/tokless"),
        _ => None,
    }
}

pub fn installed_path_for(id: &str) -> Option<String> {
    match id {
        "rtk" => {
            let p = crate::util::exec::resolve_rtk_bin();
            if !p.is_empty() { Some(p) } else { None }
        }
        "headroom" => {
            let p = crate::util::headroom::headroom_bin();
            if !p.is_empty() { Some(p) } else { None }
        }
        _ => None,
    }
}

pub fn gather_versions() -> HashMap<String, VersionInfo> {
    gather_versions_impl(false)
}

pub fn gather_versions_force() -> HashMap<String, VersionInfo> {
    gather_versions_impl(true)
}

fn gather_versions_impl(_force: bool) -> HashMap<String, VersionInfo> {
    let mut out = HashMap::new();
    for id in ["rtk", "codegraph", "context-mode", "headroom", "tokless"] {
        let inst = installed_version_for(id);
        let lat = latest_version_for(id);
        out.insert(id.to_string(), VersionInfo {
            installed: inst.clone(),
            latest: lat,
            channel: "default".into(),
            present: inst.is_some(),
        });
    }
    out
}

pub fn count_outdated(m: &HashMap<String, VersionInfo>) -> usize {
    m.values().filter(|v| {
        if let (Some(inst), Some(lat)) = (&v.installed, &v.latest) {
            semver_compare(inst, lat) < 0
        } else {
            false
        }
    }).count()
}

