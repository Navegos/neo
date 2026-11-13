// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/util/versions.rs
// created: 2026-10-05
// lastModified: 2026-11-13

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::exec::{resolve_rtk_bin, run, RunOptions};
use super::headroom::{headroom_bin, headroom_installed, headroom_installed_version};
use super::paths::{home, toksi_persisted_abs, which};
use crate::tools::projectmem::{projectmem_installed, projectmem_installed_version};

pub(crate) fn url_query_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
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

const CACHE_TTL_MS: i64 = 6 * 3600 * 1000;
const TOOL_IDS: &[&str] = &["rtk", "codegraph", "context-mode", "toksi", "headroom"];

pub fn version_cache_path() -> String {
    let h = home();
    if h.is_empty() {
        return String::new();
    }
    Path::new(&h)
        .join(".cache")
        .join("toksi")
        .join("versions.json")
        .to_string_lossy()
        .to_string()
}

pub fn load_cache() -> (Option<HashMap<String, VersionInfo>>, bool) {
    let p = version_cache_path();
    if p.is_empty() {
        return (None, false);
    }
    if let Ok(raw) = std::fs::read_to_string(p) {
        if let Ok(obj) = serde_json::from_str::<CacheShape>(&raw) {
            let now = chrono::Utc::now().timestamp_millis();
            let fresh = (now - obj.ts) <= CACHE_TTL_MS;
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
    if let Some(parent) = Path::new(&p).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let obj = CacheShape {
        ts: chrono::Utc::now().timestamp_millis(),
        map: m.clone(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&obj) {
        let _ = crate::util::paths::write_file(p, &json);
    }
}

pub fn fetch_json<T: for<'de> Deserialize<'de>>(url: &str) -> Option<T> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("toksi")
        .build()
        .ok()?;
    let resp = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .ok()?;
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
    let url = format!("https://pypi.org/pypi/{}/json", url_query_escape(pkg));
    let doc: PyPIDoc = fetch_json(&url)?;
    if doc.info.version.is_empty() {
        None
    } else {
        Some(doc.info.version)
    }
}

pub fn github_latest_release(repo: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct GhDoc {
        #[serde(default)]
        tag_name: String,
        #[serde(default)]
        name: String,
    }
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let doc: GhDoc = fetch_json(&url)?;
    let tag = if !doc.tag_name.is_empty() {
        doc.tag_name
    } else {
        doc.name
    };
    if tag.is_empty() {
        None
    } else {
        Some(tag.trim_start_matches('v').to_string())
    }
}

pub fn github_latest(repo: &str) -> Option<String> {
    github_latest_release(repo)
}

pub fn rtk_installed_version() -> Option<String> {
    let p = resolve_rtk_bin();
    if p.is_empty() {
        return Some(crate::util::version::toksi_version().to_string());
    }
    let r = run(
        &p,
        &["--version"],
        RunOptions {
            capture: true,
            ..Default::default()
        },
    );
    let src = if !r.stdout.is_empty() {
        &r.stdout
    } else {
        &r.stderr
    };
    let re = Regex::new(r"(\d+\.\d+\.\d+)").ok()?;
    let caps = re.captures(src)?;
    Some(caps.get(1)?.as_str().to_string())
}

pub fn gather_versions() -> HashMap<String, VersionInfo> {
    gather_versions_impl(false)
}

pub fn gather_versions_force() -> HashMap<String, VersionInfo> {
    gather_versions_impl(true)
}

fn gather_versions_impl(force: bool) -> HashMap<String, VersionInfo> {
    if std::env::var("TOKSI_TEST").unwrap_or_default() == "1" {
        let mut out = HashMap::new();
        out.insert(
            "rtk".to_string(),
            VersionInfo {
                installed: Some("0.43.0".to_string()),
                latest: Some("0.43.0".to_string()),
                channel: "github".to_string(),
                present: false,
            },
        );
        out.insert(
            "codegraph".to_string(),
            VersionInfo {
                installed: None,
                latest: Some("1.1.6".to_string()),
                channel: "npm".to_string(),
                present: false,
            },
        );
        out.insert(
            "context-mode".to_string(),
            VersionInfo {
                installed: None,
                latest: Some("1.0.169".to_string()),
                channel: "npm".to_string(),
                present: false,
            },
        );
        out.insert(
            "headroom".to_string(),
            VersionInfo {
                installed: Some("0.35.0".to_string()),
                latest: None,
                channel: "binary".to_string(),
                present: headroom_installed(),
            },
        );
        out.insert(
            "toksi".to_string(),
            VersionInfo {
                installed: Some("0.1.0".to_string()),
                latest: Some("0.1.0".to_string()),
                channel: "npm".to_string(),
                present: false,
            },
        );
        return out;
    }

    let latest = cached_latest(force);
    let mut out = HashMap::new();

    let rtk_inst = rtk_installed_version().or_else(|| Some(crate::util::version::toksi_version().to_string()));
    out.insert(
        "rtk".to_string(),
        VersionInfo {
            present: true,
            installed: rtk_inst,
            latest: latest.get("rtk").cloned().flatten(),
            channel: "in-tree".to_string(),
        },
    );

    let in_tree_ver = Some(crate::util::version::toksi_version().to_string());
    out.insert(
        "codegraph".to_string(),
        VersionInfo {
            present: true,
            installed: in_tree_ver.clone(),
            latest: latest.get("codegraph").cloned().flatten(),
            channel: "in-tree".to_string(),
        },
    );

    out.insert(
        "context-mode".to_string(),
        VersionInfo {
            present: true,
            installed: in_tree_ver.clone(),
            latest: latest.get("context-mode").cloned().flatten(),
            channel: "in-tree".to_string(),
        },
    );

    out.insert(
        "headroom".to_string(),
        VersionInfo {
            installed: headroom_installed_version(),
            latest: latest.get("headroom").cloned().flatten(),
            channel: "in-tree".to_string(),
            present: headroom_installed(),
        },
    );

    out.insert(
        "projectmem".to_string(),
        VersionInfo {
            installed: projectmem_installed_version(),
            latest: None,
            channel: "in-tree".to_string(),
            present: projectmem_installed(),
        },
    );

    out.insert(
        "toksi".to_string(),
        VersionInfo {
            present: true,
            installed: in_tree_ver,
            latest: latest.get("toksi").cloned().flatten(),
            channel: "in-tree".to_string(),
        },
    );

    out
}

pub fn latest_version_for(id: &str) -> Option<String> {
    cached_latest(false).get(id).cloned().flatten()
}

pub fn installed_version_for(id: &str) -> Option<String> {
    match id {
        "rtk" => rtk_installed_version().or_else(|| Some(crate::util::version::toksi_version().to_string())),
        "codegraph" | "context-mode" | "context" | "icm" | "aimemory" | "ai-memory" | "toksi" => {
            Some(crate::util::version::toksi_version().to_string())
        }
        "headroom" => headroom_installed_version(),
        "projectmem" => projectmem_installed_version(),
        _ => None,
    }
}

pub fn installed_path_for(id: &str) -> Option<String> {
    let p = match id {
        "rtk" => {
            let r = resolve_rtk_bin();
            if !r.is_empty() {
                r
            } else {
                toksi_persisted_abs()
            }
        }
        "codegraph" | "context-mode" | "context" | "icm" | "aimemory" | "ai-memory" | "toksi" => {
            toksi_persisted_abs()
        }
        "projectmem" => {
            let pj = which("pjm");
            if !pj.is_empty() {
                pj
            } else {
                toksi_persisted_abs()
            }
        }
        "headroom" => {
            if headroom_installed() {
                headroom_bin()
            } else {
                String::new()
            }
        }
        _ => String::new(),
    };
    if p.is_empty() {
        None
    } else {
        Some(p)
    }
}

fn fetch_latest_for(id: &str) -> Option<String> {
    match id {
        "rtk" => github_latest_release("Navegos/rtk").or_else(|| github_latest_release("rtk-ai/rtk")),
        "codegraph" => github_latest_release("Navegos/codegraph").or_else(|| github_latest_release("rtk-ai/rtk")),
        "context-mode" => github_latest_release("Navegos/context-mode").or_else(|| github_latest_release("mksglu/context-mode")),
        "toksi" => github_latest_release("Navegos/toksi"),
        "headroom" => github_latest_release("Navegos/headroom").or_else(|| github_latest_release("headroomlabs-ai/headroom")),
        _ => None,
    }
}

fn cached_latest(force: bool) -> HashMap<String, Option<String>> {
    if std::env::var("TOKSI_TEST").unwrap_or_default() == "1" {
        let mut m = HashMap::new();
        for (k, v) in gather_versions() {
            m.insert(k, v.latest);
        }
        return m;
    }

    let (cache, fresh) = load_cache();
    let mut result: HashMap<String, Option<String>> = HashMap::new();
    if let Some(map) = cache {
        for (k, v) in map {
            if v.latest.is_some() {
                result.insert(k, v.latest);
            }
        }
    }

    let mut todo = Vec::new();
    for &id in TOOL_IDS {
        if result.get(id).and_then(|v| v.as_ref()).is_some() && fresh && !force {
            continue;
        }
        todo.push(id);
    }

    let mut fetched = false;
    if !todo.is_empty() {
        let mut handles = Vec::new();
        for id in todo {
            let id_owned = id.to_string();
            handles.push(std::thread::spawn(move || {
                let v = fetch_latest_for(&id_owned);
                (id_owned, v)
            }));
        }
        for h in handles {
            if let Ok((id, Some(v))) = h.join() {
                result.insert(id, Some(v));
                fetched = true;
            }
        }
    }

    if fetched || force {
        let mut store = HashMap::new();
        for (k, v) in &result {
            if v.is_some() {
                store.insert(
                    k.clone(),
                    VersionInfo {
                        latest: v.clone(),
                        ..Default::default()
                    },
                );
            }
        }
        save_cache(&store);
    }

    result
}

fn parse_semver_parts(s: &str) -> Vec<i64> {
    s.trim_start_matches('v')
        .split('.')
        .map(|p| p.parse::<i64>().unwrap_or(0))
        .collect()
}

pub fn semver_compare(a: &str, b: &str) -> i32 {
    let pa = parse_semver_parts(a);
    let pb = parse_semver_parts(b);
    let n = std::cmp::max(pa.len(), pb.len());
    for i in 0..n {
        let da = pa.get(i).copied().unwrap_or(0);
        let db = pb.get(i).copied().unwrap_or(0);
        if da != db {
            return if da > db { 1 } else { -1 };
        }
    }
    0
}

pub fn semver_compare_opt(a: Option<&str>, b: Option<&str>) -> i32 {
    match (a, b) {
        (None, None) => 0,
        (None, Some(_)) => -1,
        (Some(_), None) => 1,
        (Some(va), Some(vb)) => semver_compare(va, vb),
    }
}

pub fn semver_gte(a: &str, b: &str) -> bool {
    semver_compare(a, b) >= 0
}

pub fn count_outdated(m: &HashMap<String, VersionInfo>) -> usize {
    m.values()
        .filter(|v| {
            if let (Some(inst), Some(lat)) = (&v.installed, &v.latest) {
                semver_compare(inst, lat) < 0
            } else {
                false
            }
        })
        .count()
}

pub fn bust_version_cache() {
    let p = version_cache_path();
    if !p.is_empty() {
        let _ = std::fs::remove_file(p);
    }
}
