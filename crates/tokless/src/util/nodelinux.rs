// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/nodelinux.rs
// created: 2026-10-11
// lastModified: 2026-10-14

use std::path::Path;

use super::exec::{prepend_process_path, run, RunOptions};
use super::logger::L;
use super::nodewin::{download_to_temp, fetch_node_index, node_lts_version};
use super::paths::{ensure_dir, home, which};

pub fn download_and_extract_tar_gz(
    url: &str,
    dest: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tmp = download_to_temp(url)?;
    let _ = ensure_dir(dest);
    let res = extract_tar_gz_flat(&tmp, dest);
    let _ = std::fs::remove_file(&tmp);
    res
}

pub fn node_unix_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" | "amd64" => "x64",
        "aarch64" | "arm64" => "arm64",
        _ => "",
    }
}

pub fn node_unix_dist(goos: &str, arch: &str) -> (String, String) {
    if goos == "darwin" || goos == "macos" {
        (format!("osx-{}-tar", arch), "darwin".to_string())
    } else {
        (format!("linux-{}", arch), "linux".to_string())
    }
}

pub fn node_unix_artifact(goos: &str, arch: &str, v: &str) -> (String, String, String) {
    let (index_token, dl_os) = node_unix_dist(goos, arch);
    let file_base = format!("node-{}-{}-{}", v, dl_os, arch);
    let url = format!("https://nodejs.org/dist/{}/{}.tar.gz", v, file_base);
    (index_token, file_base, url)
}

pub fn node_unix_install_dir() -> String {
    Path::new(&home())
        .join(".local")
        .join("share")
        .join("tokless")
        .join("node")
        .to_string_lossy()
        .to_string()
}

pub fn install_node_unix_tarball() -> bool {
    let arch = node_unix_arch();
    if arch.is_empty() {
        L.err(&format!(
            "unsupported architecture for Node install: {}",
            std::env::consts::ARCH
        ));
        return false;
    }
    let goos = if std::env::consts::OS == "macos" {
        "darwin"
    } else {
        std::env::consts::OS
    };
    let (index_token, _) = node_unix_dist(goos, arch);
    let Some(entries) = fetch_node_index() else {
        L.err("couldn't fetch the Node.js release index");
        return false;
    };
    let Some(v) = node_lts_version(&entries, &index_token) else {
        L.err(&format!("no Node LTS tarball for {}", index_token));
        return false;
    };
    L.info(&format!("Downloading Node.js {} from nodejs.org…", v));
    let (_, _, url) = node_unix_artifact(goos, arch, &v);
    let tar_path = match download_to_temp(&url) {
        Ok(p) => p,
        Err(err) => {
            L.err(&format!("Node download failed: {}", err));
            return false;
        }
    };
    let dest = node_unix_install_dir();
    let _ = std::fs::remove_dir_all(&dest);
    let extract_res = extract_tar_gz_strip_root(&tar_path, &dest);
    let _ = std::fs::remove_file(&tar_path);
    if let Err(err) = extract_res {
        L.err(&format!("Node extract failed: {}", err));
        return false;
    }

    let local_bin = Path::new(&home()).join(".local").join("bin");
    let _ = ensure_dir(&local_bin);
    for b in ["node", "npm", "npx"] {
        let link = local_bin.join(b);
        let _ = std::fs::remove_file(&link);
        let target = Path::new(&dest).join("bin").join(b);
        #[cfg(unix)]
        let link_res = std::os::unix::fs::symlink(&target, &link);
        #[cfg(windows)]
        let link_res = std::os::windows::fs::symlink_file(&target, &link)
            .or_else(|_| std::fs::copy(&target, &link).map(|_| ()));
        if let Err(err) = link_res {
            L.err(&format!(
                "couldn't link {} into {}: {}",
                b,
                local_bin.to_string_lossy(),
                err
            ));
            return false;
        }
    }
    let local_bin_str = local_bin.to_string_lossy().to_string();
    prepend_process_path(&local_bin_str);
    let prefix = Path::new(&home())
        .join(".local")
        .to_string_lossy()
        .to_string();
    let _ = run(
        &local_bin.join("npm").to_string_lossy(),
        &["config", "set", "prefix", &prefix],
        RunOptions::default(),
    );
    !which("node").is_empty() && !which("npm").is_empty()
}

pub fn extract_tar_gz_strip_root(
    path: &str,
    dest: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    extract_tar_gz(path, dest, true)
}

pub fn extract_tar_gz_flat(
    path: &str,
    dest: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    extract_tar_gz(path, dest, false)
}

fn extract_tar_gz(
    path: &str,
    dest: &str,
    strip_root: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ensure_dir(dest)?;
    let args: Vec<&str> = if strip_root {
        vec!["-xzf", path, "--strip-components=1", "-C", dest]
    } else {
        vec!["-xzf", path, "-C", dest]
    };
    let r = run(
        "tar",
        &args,
        RunOptions {
            capture: true,
            ..Default::default()
        },
    );
    if r.code != 0 {
        return Err(format!("tar extract failed: {}", r.stderr.trim()).into());
    }
    Ok(())
}
