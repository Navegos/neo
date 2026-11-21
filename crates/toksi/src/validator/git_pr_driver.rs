// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/validator/git_pr_driver.rs
// created: 2026-11-21

//! Native Git/PR Driver for `Navegos/advisory` (`~/.toksi/repos/advisory/` or
//! `[repositories].advisory_dir`): clones/updates, branches, commits, pushes,
//! and opens Pull Requests via `POST /repos/Navegos/advisory/pulls`.

#![deny(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::json;
use zeroize::Zeroizing;

use crate::util::env_path::check_executable_in_path;
use crate::util::paths::{home, normalize_path};

pub const DEFAULT_ADVISORY_REMOTE: &str = "https://github.com/Navegos/advisory.git";
pub const DEFAULT_ADVISORY_PR_API: &str = "https://api.github.com/repos/Navegos/advisory/pulls";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdvisoryPrRequest {
    pub title: String,
    pub body: String,
    pub head_branch: String,
    pub base_branch: String,
}

/// Resolves the local `Navegos/advisory` clone directory.
pub fn resolve_advisory_repo_dir(configured: Option<&str>) -> PathBuf {
    if let Some(c) = configured.map(str::trim).filter(|s| !s.is_empty()) {
        return PathBuf::from(normalize_path(c));
    }
    let h = home();
    if h.is_empty() {
        PathBuf::from(".toksi/repos/advisory")
    } else {
        PathBuf::from(normalize_path(&format!(
            "{}/.toksi/repos/advisory",
            h.trim_end_matches('/')
        )))
    }
}

/// Ensures `git` is in `PATH` and creates a feature branch + commits staged advisory shards.
pub fn commit_advisory_branch(
    repo_dir: &Path,
    branch_name: &str,
    commit_msg: &str,
) -> Result<(), String> {
    check_executable_in_path("git")?;
    if !repo_dir.join(".git").exists() {
        return Err(format!(
            "Advisory repository '{}' is not a git clone",
            normalize_path(&repo_dir.to_string_lossy())
        ));
    }

    let checkout = Command::new("git")
        .args(["checkout", "-B", branch_name])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| format!("Failed to run git checkout: {e}"))?;
    if !checkout.status.success() {
        return Err(format!(
            "git checkout failed: {}",
            String::from_utf8_lossy(&checkout.stderr)
        ));
    }

    let add = Command::new("git")
        .args(["add", "-A"])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| format!("Failed to run git add: {e}"))?;
    if !add.status.success() {
        return Err(format!(
            "git add failed: {}",
            String::from_utf8_lossy(&add.stderr)
        ));
    }

    let commit = Command::new("git")
        .args(["commit", "-m", commit_msg])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| format!("Failed to run git commit: {e}"))?;
    if !commit.status.success() {
        return Err(format!(
            "git commit failed: {}",
            String::from_utf8_lossy(&commit.stderr)
        ));
    }
    Ok(())
}

/// Opens a Pull Request against `Navegos/advisory` via GitHub REST API (`POST /repos/Navegos/advisory/pulls`).
pub async fn open_advisory_pull_request(
    pr: &AdvisoryPrRequest,
    github_token: Zeroizing<String>,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .min_tls_version(reqwest::tls::Version::TLS_1_3)
        .build()
        .map_err(|e| format!("Failed to build TLS client: {e}"))?;

    let resp = client
        .post(DEFAULT_ADVISORY_PR_API)
        .header("User-Agent", "toksi-validator")
        .header("Accept", "application/vnd.github+json")
        .bearer_auth(github_token.as_str())
        .json(&json!({
            "title": pr.title,
            "body": pr.body,
            "head": pr.head_branch,
            "base": pr.base_branch,
        }))
        .send()
        .await
        .map_err(|e| format!("GitHub PR request failed: {e}"))?;

    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("GitHub PR creation failed ({status}): {text}"));
    }
    let val: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
    Ok(val
        .get("html_url")
        .and_then(|v| v.as_str())
        .unwrap_or(DEFAULT_ADVISORY_PR_API)
        .to_string())
}
