use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use serde::Deserialize;

use super::exec::{
    prepend_process_path, resolve_node_binary, resolve_npm_binary, run, ExecResult, RunOptions,
};
use super::logger::L;
use super::paths::{ensure_dir, home, IS_WIN};
use super::versions::npm_installed_version;

pub const DEFAULT_REGISTRY_URL: &str = "https://registry.npmjs.org/";
pub const NPM_INSTALL_TIMEOUT: Duration = Duration::from_secs(15 * 60);
pub const NPM_CONFIG_TIMEOUT: Duration = Duration::from_secs(30);

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

pub fn npm_registry_base() -> String {
    let r = run_npm_config(&["config", "get", "registry"]);
    let mut v = r.stdout.trim().to_string();
    if r.code == 0 && v.starts_with("http") {
        if !v.ends_with('/') {
            v.push('/');
        }
        return v;
    }
    DEFAULT_REGISTRY_URL.to_string()
}

#[derive(Debug, Deserialize)]
struct RegistryDoc {
    #[serde(rename = "dist-tags", default)]
    dist_tags: HashMap<String, String>,
    #[serde(default)]
    versions: HashMap<String, RegistryVersion>,
}

#[derive(Debug, Deserialize)]
struct RegistryVersion {
    #[allow(dead_code)]
    #[serde(default)]
    version: String,
    dist: Option<RegistryDist>,
}

#[derive(Debug, Deserialize)]
struct RegistryDist {
    #[serde(default)]
    tarball: String,
}

pub fn resolve_from_registry(pkg: &str, spec: &str) -> Option<(String, String)> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("tokless")
        .build()
        .ok()?;
    let url = format!("{}{}", npm_registry_base(), url_query_escape(pkg));
    let resp = client.get(&url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let doc: RegistryDoc = resp.json().ok()?;
    let version = if let Some(v) = doc.dist_tags.get(spec) {
        v.clone()
    } else if doc.versions.contains_key(spec) {
        spec.to_string()
    } else {
        return None;
    };
    let v_info = doc.versions.get(&version)?;
    let dist = v_info.dist.as_ref()?;
    if dist.tarball.is_empty() {
        return None;
    }
    Some((version, dist.tarball.clone()))
}

pub fn run_npm_config(args: &[&str]) -> ExecResult {
    run_npm_with_timeout(args, &[], NPM_CONFIG_TIMEOUT)
}

pub fn run_npm(args: &[&str], env: &[(String, String)]) -> ExecResult {
    run_npm_with_timeout(args, env, NPM_INSTALL_TIMEOUT)
}

pub fn run_npm_with_timeout(
    args: &[&str],
    env: &[(String, String)],
    timeout: Duration,
) -> ExecResult {
    let npm_bin = resolve_npm_binary();
    if npm_bin.is_empty() {
        return ExecResult {
            code: 127,
            stdout: String::new(),
            stderr: "npm not found".to_string(),
        };
    }
    run(
        &npm_bin,
        args,
        RunOptions {
            capture: true,
            env: env.to_vec(),
            timeout: Some(timeout),
            ..Default::default()
        },
    )
}

pub fn build_npm_attempts(
    pkg: &str,
    resolved_version: &str,
    tarball: &str,
    cache_dir: &str,
) -> Vec<Vec<String>> {
    let token = if resolved_version.is_empty() {
        format!("{}@latest", pkg)
    } else {
        format!("{}@{}", pkg, resolved_version)
    };
    let mut online = vec!["--prefer-online".to_string()];
    if !cache_dir.is_empty() {
        online.push("--cache".to_string());
        online.push(cache_dir.to_string());
    }
    let mut a1 = vec!["install".to_string(), "-g".to_string(), token.clone()];
    a1.extend(online.clone());
    let mut attempts = vec![a1];

    if !tarball.is_empty() {
        let mut a2 = vec!["install".to_string(), "-g".to_string(), tarball.to_string()];
        a2.extend(online.clone());
        attempts.push(a2);
    }

    attempts.push(vec!["install".to_string(), "-g".to_string(), token.clone()]);

    let mut a4 = vec![
        "install".to_string(),
        "-g".to_string(),
        token,
        "--registry".to_string(),
        DEFAULT_REGISTRY_URL.to_string(),
    ];
    a4.extend(online);
    attempts.push(a4);

    attempts
}

pub fn install_succeeded(target: &str, actual: Option<&str>) -> Option<String> {
    let act = actual?;
    if target.is_empty() || act == target {
        Some(act.to_string())
    } else {
        None
    }
}

pub fn npm_global_install(pkg: &str, spec: &str) -> (String, bool) {
    let spec = if spec.is_empty() { "latest" } else { spec };

    let (resolved_version, tarball, ok) = match resolve_from_registry(pkg, spec) {
        Some((v, t)) => (v, t, true),
        None => (String::new(), String::new(), false),
    };
    let mut target = resolved_version.clone();
    if !ok && spec != "latest" {
        target = spec.to_string();
    }

    let cache_dir = fresh_cache_dir();

    let attempts = build_npm_attempts(pkg, &resolved_version, &tarball, &cache_dir);
    for (i, args) in attempts.iter().enumerate() {
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let r = run_npm(&arg_refs, &[]);
        if r.code != 0 {
            L.debug(&format!(
                "npm attempt {} failed: {}",
                i + 1,
                first_npm_line(&r.stderr, &r.stdout)
            ));
            continue;
        }
        let actual = npm_installed_version(pkg);
        if let Some(v) = install_succeeded(&target, actual.as_deref()) {
            ensure_npm_global_bin_on_path();
            cleanup_dir(&cache_dir);
            return (v, true);
        }
        L.debug(&format!(
            "npm attempt {} exit 0 but package not installed",
            i + 1
        ));
    }

    let token = if !resolved_version.is_empty() {
        format!("{}@{}", pkg, resolved_version)
    } else {
        format!("{}@{}", pkg, spec)
    };
    if let Some(v) = npm_user_prefix_install(pkg, &token, &cache_dir) {
        cleanup_dir(&cache_dir);
        return (v, true);
    }
    cleanup_dir(&cache_dir);
    (String::new(), false)
}

pub fn npm_install_global(pkg: &str) -> bool {
    npm_global_install(pkg, "latest").1
}

pub fn first_npm_line(stderr: &str, stdout: &str) -> String {
    for s in [stderr, stdout] {
        for l in s.lines() {
            let trimmed = l.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    "no output".to_string()
}

pub fn user_local_npm_prefix() -> String {
    for k in ["npm_config_prefix", "NPM_CONFIG_PREFIX"] {
        if let Ok(v) = std::env::var(k) {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    Path::new(&home())
        .join(".local")
        .to_string_lossy()
        .to_string()
}

pub fn npm_prefix_installed_version(prefix: &str, pkg: &str) -> Option<String> {
    if prefix.is_empty() {
        return None;
    }
    let pj = if IS_WIN {
        Path::new(prefix)
            .join("node_modules")
            .join(pkg)
            .join("package.json")
    } else {
        Path::new(prefix)
            .join("lib")
            .join("node_modules")
            .join(pkg)
            .join("package.json")
    };
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

pub fn npm_user_prefix_install(pkg: &str, token: &str, cache_dir: &str) -> Option<String> {
    let prefix = user_local_npm_prefix();
    if prefix.is_empty() {
        return None;
    }
    let seed = if IS_WIN {
        Path::new(&prefix).to_path_buf()
    } else {
        Path::new(&prefix).join("lib")
    };
    ensure_dir(&seed).ok()?;

    let mut args = vec!["install", "-g", token, "--no-audit", "--no-fund"];
    if !cache_dir.is_empty() {
        args.push("--cache");
        args.push(cache_dir);
    }
    let env = vec![("npm_config_prefix".to_string(), prefix.clone())];
    let r = run_npm(&args, &env);
    if r.code != 0 {
        return None;
    }
    let v = npm_prefix_installed_version(&prefix, pkg)?;
    prepend_process_path(&npm_global_bin_dir(&prefix, IS_WIN));
    Some(v)
}

pub fn npm_prefix() -> String {
    let r = run_npm_config(&["config", "get", "prefix"]);
    if r.code != 0 {
        return String::new();
    }
    r.stdout.trim().to_string()
}

pub fn ensure_npm_global_bin_on_path() {
    let prefix = npm_prefix();
    if prefix.is_empty() {
        return;
    }
    prepend_process_path(&npm_global_bin_dir(&prefix, IS_WIN));
}

pub fn npm_global_bin_dir(prefix: &str, win: bool) -> String {
    if win {
        prefix.to_string()
    } else {
        Path::new(prefix).join("bin").to_string_lossy().to_string()
    }
}

pub fn fresh_cache_dir() -> String {
    match tempfile::Builder::new().prefix("tokless-npm-").tempdir() {
        Ok(d) => d.keep().to_string_lossy().to_string(),
        Err(_) => String::new(),
    }
}

fn cleanup_dir(dir: &str) {
    if !dir.is_empty() {
        let _ = std::fs::remove_dir_all(dir);
    }
}

pub fn node_major() -> i32 {
    let node_bin = resolve_node_binary();
    if node_bin.is_empty() {
        return 0;
    }
    let r = run(
        &node_bin,
        &["--version"],
        RunOptions {
            capture: true,
            ..Default::default()
        },
    );
    let v = r.stdout.trim().trim_start_matches('v');
    let maj = v.split('.').next().unwrap_or(v);
    maj.parse::<i32>().unwrap_or(0)
}

pub fn node_too_old_hint(min: i32) -> String {
    let maj = node_major();
    if maj > 0 && maj < min {
        format!(
            "Node.js v{} is too old (need v{}+). Upgrade without sudo via nvm (https://github.com/nvm-sh/nvm), or: curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash - && sudo apt-get install -y nodejs",
            maj, min
        )
    } else {
        String::new()
    }
}
