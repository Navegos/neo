// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/versions.rs
// created: 2026-10-05
// lastModified: 2026-10-20

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::exec::{resolve_npm_binary, resolve_rtk_bin, run, RunOptions};
use super::headroom::{headroom_bin, headroom_installed, headroom_installed_version};
use super::npminstall::{
    npm_prefix, npm_prefix_installed_version, npm_registry_base, url_query_escape,
    user_local_npm_prefix,
};
use super::paths::{exists, home, IS_WIN};
use super::projectmem::{projectmem_installed, projectmem_installed_version};

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
const TOOL_IDS: &[&str] = &["rtk", "codegraph", "context-mode", "tokless", "headroom"];

pub fn version_cache_path() -> String {
    let h = home();
    if h.is_empty() {
        return String::new();
    }
    Path::new(&h)
        .join(".cache")
        .join("tokless")
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
        .user_agent("tokless")
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

pub fn npm_latest(pkg: &str) -> Option<String> {
    if let Some(v) = npm_view_latest(pkg) {
        return Some(v);
    }
    #[derive(Deserialize)]
    struct NpmDoc {
        #[serde(rename = "dist-tags", default)]
        dist_tags: HashMap<String, String>,
    }
    let url = format!("{}{}", npm_registry_base(), url_query_escape(pkg));
    let doc: NpmDoc = fetch_json(&url)?;
    doc.dist_tags
        .get("latest")
        .filter(|s| !s.is_empty())
        .cloned()
}

pub fn npm_view_latest(pkg: &str) -> Option<String> {
    let npm_bin = resolve_npm_binary();
    if npm_bin.is_empty() {
        return None;
    }
    let spec = format!("{}@latest", pkg);
    let r = run(
        &npm_bin,
        &["info", &spec, "version", "--json"],
        RunOptions {
            capture: true,
            env: vec![
                ("NPM_CONFIG_PREFER_OFFLINE".to_string(), "false".to_string()),
                ("NPM_CONFIG_PREFER_ONLINE".to_string(), "true".to_string()),
            ],
            timeout: Some(Duration::from_secs(8)),
            ..Default::default()
        },
    );
    if r.code != 0 {
        return None;
    }
    let s = r.stdout.trim().trim_matches('"');
    let re = Regex::new(r"(\d+\.\d+\.\d+)").ok()?;
    let caps = re.captures(s)?;
    Some(caps.get(1)?.as_str().to_string())
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
        return None;
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

pub fn npm_installed_version(pkg: &str) -> Option<String> {
    let npm_bin = resolve_npm_binary();
    if !npm_bin.is_empty() {
        let r = run(
            &npm_bin,
            &["ls", "-g", "--depth=0", "--json", pkg],
            RunOptions {
                capture: true,
                ..Default::default()
            },
        );
        #[derive(Deserialize)]
        struct NpmLs {
            #[serde(default)]
            dependencies: HashMap<String, NpmLsDep>,
        }
        #[derive(Deserialize)]
        struct NpmLsDep {
            #[serde(default)]
            version: String,
        }
        if let Ok(j) = serde_json::from_str::<NpmLs>(&r.stdout) {
            if let Some(d) = j.dependencies.get(pkg) {
                if !d.version.is_empty() {
                    return Some(d.version.clone());
                }
            }
        }
        if let Some(v) = npm_prefix_installed_version(&npm_prefix(), pkg) {
            return Some(v);
        }
        if let Some(v) = npm_prefix_installed_version(&user_local_npm_prefix(), pkg) {
            return Some(v);
        }
    }
    bun_installed_version(pkg)
}

pub fn npm_installed_version_exported(pkg: &str) -> Option<String> {
    npm_installed_version(pkg)
}

pub fn bun_installed_version(pkg: &str) -> Option<String> {
    let h = home();
    let bin_link = Path::new(&h).join(".bun").join("bin").join(pkg);
    if let Ok(real) = std::fs::canonicalize(&bin_link) {
        if let Some(dir) = real.parent() {
            if let Some(v) = read_pkg_version(dir.join("package.json")) {
                return Some(v);
            }
        }
    }
    let global_pj = Path::new(&h)
        .join(".bun")
        .join("install")
        .join("global")
        .join("node_modules")
        .join(pkg)
        .join("package.json");
    read_pkg_version(global_pj)
}

fn read_pkg_version(pj: impl AsRef<Path>) -> Option<String> {
    let raw = std::fs::read_to_string(pj).ok()?;
    #[derive(Deserialize)]
    struct PkgJson {
        #[serde(default)]
        version: String,
    }
    let p: PkgJson = serde_json::from_str(&raw).ok()?;
    if p.version.is_empty() {
        None
    } else {
        Some(p.version)
    }
}

pub fn gather_versions() -> HashMap<String, VersionInfo> {
    gather_versions_impl(false)
}

pub fn gather_versions_force() -> HashMap<String, VersionInfo> {
    gather_versions_impl(true)
}

fn gather_versions_impl(force: bool) -> HashMap<String, VersionInfo> {
    if std::env::var("TOKLESS_TEST").unwrap_or_default() == "1" {
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
                channel: "uv".to_string(),
                present: headroom_installed(),
            },
        );
        out.insert(
            "tokless".to_string(),
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

    let rtk_inst = rtk_installed_version();
    out.insert(
        "rtk".to_string(),
        VersionInfo {
            present: rtk_inst.is_some(),
            installed: rtk_inst,
            latest: latest.get("rtk").cloned().flatten(),
            channel: "github".to_string(),
        },
    );

    let cg_inst = npm_installed_version("@colbymchenry/codegraph");
    out.insert(
        "codegraph".to_string(),
        VersionInfo {
            present: cg_inst.is_some(),
            installed: cg_inst,
            latest: latest.get("codegraph").cloned().flatten(),
            channel: "npm".to_string(),
        },
    );

    let cm_inst = npm_installed_version("context-mode");
    out.insert(
        "context-mode".to_string(),
        VersionInfo {
            present: cm_inst.is_some(),
            installed: cm_inst,
            latest: latest.get("context-mode").cloned().flatten(),
            channel: "npm".to_string(),
        },
    );

    out.insert(
        "headroom".to_string(),
        VersionInfo {
            installed: headroom_installed_version(),
            latest: latest.get("headroom").cloned().flatten(),
            channel: "uv".to_string(),
            present: headroom_installed(),
        },
    );

    out.insert(
        "projectmem".to_string(),
        VersionInfo {
            installed: projectmem_installed_version(),
            latest: None,
            channel: "uv".to_string(),
            present: projectmem_installed(),
        },
    );

    let tk_inst = npm_installed_version("tokless");
    out.insert(
        "tokless".to_string(),
        VersionInfo {
            present: tk_inst.is_some(),
            installed: tk_inst,
            latest: latest.get("tokless").cloned().flatten(),
            channel: "npm".to_string(),
        },
    );

    out
}

pub fn latest_version_for(id: &str) -> Option<String> {
    cached_latest(false).get(id).cloned().flatten()
}

pub fn installed_version_for(id: &str) -> Option<String> {
    match id {
        "rtk" => rtk_installed_version(),
        "codegraph" => npm_installed_version("@colbymchenry/codegraph"),
        "context-mode" => npm_installed_version("context-mode"),
        "headroom" => headroom_installed_version(),
        "projectmem" => projectmem_installed_version(),
        "tokless" => npm_installed_version("tokless"),
        _ => None,
    }
}

pub fn installed_path_for(id: &str) -> Option<String> {
    let p = match id {
        "rtk" => resolve_rtk_bin(),
        "codegraph" => npm_pkg_dir("@colbymchenry/codegraph"),
        "context-mode" => npm_pkg_dir("context-mode"),
        "tokless" => npm_pkg_dir("tokless"),
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

pub fn npm_pkg_dir(pkg: &str) -> String {
    let mut dirs = Vec::new();
    for prefix in [npm_prefix(), user_local_npm_prefix()] {
        if prefix.is_empty() {
            continue;
        }
        if IS_WIN {
            dirs.push(
                Path::new(&prefix)
                    .join("node_modules")
                    .join(pkg)
                    .to_string_lossy()
                    .to_string(),
            );
        } else {
            dirs.push(
                Path::new(&prefix)
                    .join("lib")
                    .join("node_modules")
                    .join(pkg)
                    .to_string_lossy()
                    .to_string(),
            );
        }
    }
    dirs.push(
        Path::new(&home())
            .join(".bun")
            .join("install")
            .join("global")
            .join("node_modules")
            .join(pkg)
            .to_string_lossy()
            .to_string(),
    );
    for d in dirs {
        if exists(Path::new(&d).join("package.json")) {
            return d;
        }
    }
    String::new()
}

fn fetch_latest_for(id: &str) -> Option<String> {
    match id {
        "rtk" => github_latest_release("rtk-ai/rtk"),
        "codegraph" => npm_latest("@colbymchenry/codegraph"),
        "context-mode" => npm_latest("context-mode"),
        "tokless" => npm_latest("tokless"),
        "headroom" => pypi_latest("headroom-ai"),
        _ => None,
    }
}

fn cached_latest(force: bool) -> HashMap<String, Option<String>> {
    if std::env::var("TOKLESS_TEST").unwrap_or_default() == "1" {
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
