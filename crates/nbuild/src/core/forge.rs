// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/core/distro.rs
// created: 2026-10-15
// lastModified: 2026-10-15

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{bail, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgeType {
    GitHub,
    GitLab,
    Codeberg,
    Bitbucket,
    Launchpad,
    GenericGit,
}

impl ForgeType {
    pub fn detect(url_or_repo: &str) -> Self {
        let lower = url_or_repo.to_lowercase();
        if lower.contains("gitlab.com") || lower.starts_with("gitlab:") {
            Self::GitLab
        } else if lower.contains("codeberg.org") || lower.contains("gitea") || lower.starts_with("codeberg:") {
            Self::Codeberg
        } else if lower.contains("bitbucket.org") || lower.starts_with("bitbucket:") {
            Self::Bitbucket
        } else if lower.contains("launchpad.net") || lower.starts_with("lp:") {
            Self::Launchpad
        } else if lower.contains("github.com") || (!lower.contains("://") && lower.contains('/') && !lower.contains(".git")) {
            Self::GitHub
        } else {
            Self::GenericGit
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

pub type GitHubAsset = RemoteAsset;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRelease {
    pub tag_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_commitish: Option<String>,
    #[serde(default)]
    pub assets: Vec<RemoteAsset>,
}

pub type GitHubRelease = RemoteRelease;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompressionFormat {
    TarZst,
    TarXz,
    Zip,
    SevenZip,
    TarGz,
    TarBz2,
    Rar,
    RawBinary,
    Other,
}

impl CompressionFormat {
    pub fn from_filename(name: &str) -> Self {
        let lower = name.to_lowercase();
        if lower.ends_with(".tar.zst") || lower.ends_with(".tzst") {
            Self::TarZst
        } else if lower.ends_with(".tar.xz") || lower.ends_with(".txz") {
            Self::TarXz
        } else if lower.ends_with(".zip") {
            Self::Zip
        } else if lower.ends_with(".7z") {
            Self::SevenZip
        } else if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
            Self::TarGz
        } else if lower.ends_with(".tar.bz2") || lower.ends_with(".tbz2") || lower.ends_with(".bz2") {
            Self::TarBz2
        } else if lower.ends_with(".rar") {
            Self::Rar
        } else if lower.ends_with(".exe") {
            Self::RawBinary
        } else {
            Self::Other
        }
    }

    /// Score preference based on target OS platform (higher is better)
    pub fn preference_score(&self, platform: &str) -> i32 {
        let plat = platform.to_lowercase();
        if plat == "windows" {
            match self {
                Self::Zip => 100,      // Windows primary native standard
                Self::TarZst => 95,   // ultra-fast modern decompression
                Self::TarXz => 90,    // standard for LLVM / MSVC toolchains
                Self::SevenZip => 85,
                Self::TarGz => 70,
                Self::TarBz2 => 60,
                Self::Rar => 40,
                Self::RawBinary => 30,
                Self::Other => 10,
            }
        } else if plat == "linux" || plat == "freebsd" {
            match self {
                Self::TarZst => 100,  // modern Linux distro favorite
                Self::TarXz => 95,
                Self::TarGz => 90,
                Self::TarBz2 => 80,
                Self::SevenZip => 60,
                Self::Zip => 50,
                Self::Rar => 30,
                Self::RawBinary => 20,
                Self::Other => 10,
            }
        } else { // apple / macos / generic
            match self {
                Self::TarZst => 100,
                Self::TarXz => 95,
                Self::TarGz => 90,
                Self::Zip => 85,
                Self::SevenZip => 60,
                Self::TarBz2 => 50,
                Self::Rar => 30,
                Self::RawBinary => 20,
                Self::Other => 10,
            }
        }
    }
}

/// Strip archive extensions to extract stem (e.g. "windows-x86_64.zip" -> "windows-x86_64")
pub fn strip_archive_extension(name: &str) -> &str {
    let lower = name.to_lowercase();
    for ext in &[
        ".tar.zst", ".tar.xz", ".tar.bz2", ".tar.gz",
        ".tzst", ".txz", ".tbz2", ".tgz",
        ".zip", ".7z", ".rar", ".bz2", ".xz", ".gz", ".exe",
    ] {
        if lower.ends_with(ext) {
            return &name[..name.len() - ext.len()];
        }
    }
    name
}

pub fn is_non_payload_asset(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".sha256")
        || lower.ends_with(".sha512")
        || lower.ends_with(".asc")
        || lower.ends_with(".sig")
        || lower.ends_with(".md5")
        || lower.ends_with(".txt")
        || lower.ends_with(".json")
        || lower.ends_with(".yaml")
        || lower.ends_with(".yml")
        || lower.ends_with(".log")
}

pub fn matches_platform(name: &str, platform: &str) -> bool {
    let name_lower = name.to_lowercase();
    let plat_lower = platform.to_lowercase();

    match plat_lower.as_str() {
        "windows" => {
            (name_lower.contains("win") || name_lower.contains("windows"))
                && !name_lower.contains("darwin")
        }
        "linux" => {
            name_lower.contains("linux") && !name_lower.contains("darwin")
        }
        "apple" | "darwin" | "macos" => {
            (name_lower.contains("macos") || name_lower.contains("darwin") || name_lower.contains("apple"))
                && !name_lower.contains("windows")
        }
        "freebsd" => name_lower.contains("freebsd"),
        _ => name_lower.contains(&plat_lower),
    }
}

pub fn matches_arch(name: &str, arch: &str) -> bool {
    let name_lower = name.to_lowercase();
    let arch_lower = arch.to_lowercase();

    match arch_lower.as_str() {
        "x86_64" | "x64" | "amd64" => {
            name_lower.contains("x86_64")
                || name_lower.contains("x64")
                || name_lower.contains("amd64")
                || (!name_lower.contains("arm")
                    && !name_lower.contains("aarch64")
                    && !name_lower.contains("i686")
                    && !name_lower.contains("x86-32")
                    && !name_lower.contains("win32"))
        }
        "aarch64" | "arm64" => {
            name_lower.contains("aarch64") || name_lower.contains("arm64")
        }
        "armv7" | "armhf" => {
            name_lower.contains("armv7") || name_lower.contains("armhf")
        }
        "riscv64" => name_lower.contains("riscv64"),
        _ => name_lower.contains(&arch_lower),
    }
}

impl RemoteRelease {
    /// Strip leading 'v' or non-digit prefix, e.g. "v3.30.5" -> "3.30.5"
    pub fn clean_version(&self) -> String {
        let tag = self.tag_name.trim();
        tag.trim_start_matches(|c: char| !c.is_numeric()).to_string()
    }

    /// Match an asset from release for platform and architecture, ranking by compression format
    pub fn find_asset(&self, platform: &str, arch: &str) -> Option<&RemoteAsset> {
        self.find_asset_with_hints(platform, arch, None)
    }

    /// Match an asset using optional hints and compression format ranking
    pub fn find_asset_with_hints(
        &self,
        platform: &str,
        arch: &str,
        hints: Option<&[String]>,
    ) -> Option<&RemoteAsset> {
        let mut candidates: Vec<(&RemoteAsset, i32)> = Vec::new();

        for asset in &self.assets {
            let name = &asset.name;
            if is_non_payload_asset(name) {
                continue;
            }

            if !matches_platform(name, platform) || !matches_arch(name, arch) {
                continue;
            }

            let name_lower = name.to_lowercase();
            let format = CompressionFormat::from_filename(name);
            let mut score = format.preference_score(platform);

            if let Some(hint_list) = hints {
                let mut matched_hint = false;
                for hint in hint_list {
                    let h_lower = hint.to_lowercase();
                    let h_stem = strip_archive_extension(&h_lower);

                    if name_lower.contains(&h_lower) {
                        score += 50;
                        matched_hint = true;
                        break;
                    } else if !h_stem.is_empty() && name_lower.contains(h_stem) {
                        score += 30;
                        matched_hint = true;
                        break;
                    }
                }
                if !matched_hint && !hint_list.is_empty() {
                    // Penalty if hints were specified but this asset did not match any of them
                    score -= 40;
                }
            }

            candidates.push((asset, score));
        }

        candidates.sort_by(|a, b| b.1.cmp(&a.1));
        candidates.first().map(|(asset, _)| *asset)
    }
}

pub struct ForgeClient {
    client: reqwest::blocking::Client,
}

pub type GitHubClient = ForgeClient;

impl ForgeClient {
    pub fn new() -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("nbuild/0.1.0"));

        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { client })
    }

    /// Fetch release directly from a complete API releases endpoint URL (GitHub, GitLab, Codeberg, etc.)
    pub fn fetch_release_by_url(&self, releases_url: &str) -> Result<RemoteRelease> {
        let mut req = self.client.get(releases_url);

        if releases_url.contains("api.github.com") {
            if let Ok(token) = env::var("GITHUB_TOKEN").or_else(|_| env::var("GH_TOKEN")) {
                if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", token.trim())) {
                    req = req.header(AUTHORIZATION, val);
                }
            }
        } else if releases_url.contains("gitlab.com") {
            if let Ok(token) = env::var("GITLAB_TOKEN").or_else(|_| env::var("GL_TOKEN")) {
                req = req.header("PRIVATE-TOKEN", token.trim());
            }
        } else if releases_url.contains("codeberg.org") {
            if let Ok(token) = env::var("CODEBERG_TOKEN").or_else(|_| env::var("GITEA_TOKEN")) {
                if let Ok(val) = HeaderValue::from_str(&format!("token {}", token.trim())) {
                    req = req.header(AUTHORIZATION, val);
                }
            }
        }

        let resp = req.send().with_context(|| format!("Releases API request failed for {releases_url}"))?;
        if !resp.status().is_success() {
            bail!("Releases API request for {} failed with HTTP status: {}", releases_url, resp.status());
        }

        let release: RemoteRelease = resp.json()
            .with_context(|| format!("Failed to parse release JSON from {releases_url}"))?;
        Ok(release)
    }

    /// Validate URL safety (prevent dangerous protocols)
    pub fn validate_url(url: &str) -> Result<()> {
        let lower = url.to_lowercase();
        if lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("git://") || lower.starts_with("ssh://") || lower.starts_with("git@") {
            Ok(())
        } else {
            bail!("Disallowed or insecure protocol in repository URL: {}", url);
        }
    }

    /// Fetch latest release from any supported forge (GitHub, GitLab, Codeberg, Bitbucket)
    pub fn fetch_latest_release(&self, repo_spec: &str) -> Result<RemoteRelease> {
        let forge = ForgeType::detect(repo_spec);
        match forge {
            ForgeType::GitHub => self.fetch_github_latest(repo_spec),
            ForgeType::GitLab => self.fetch_gitlab_latest(repo_spec),
            ForgeType::Codeberg => self.fetch_codeberg_latest(repo_spec),
            ForgeType::Bitbucket => self.fetch_bitbucket_latest(repo_spec),
            ForgeType::Launchpad => self.fetch_launchpad_latest(repo_spec),
            ForgeType::GenericGit => {
                bail!("Generic git repository does not have a REST releases API. Use git clone instead: {}", repo_spec);
            }
        }
    }

    // --- GitHub API ---
    fn fetch_github_latest(&self, repo: &str) -> Result<RemoteRelease> {
        let clean_repo = repo.trim_start_matches("https://github.com/").trim_end_matches(".git");
        let url = format!("https://api.github.com/repos/{clean_repo}/releases/latest");
        let mut req = self.client.get(&url);

        if let Ok(token) = env::var("GITHUB_TOKEN").or_else(|_| env::var("GH_TOKEN")) {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", token.trim())) {
                req = req.header(AUTHORIZATION, val);
            }
        }

        let resp = req.send().with_context(|| format!("GitHub API request failed for {url}"))?;
        if !resp.status().is_success() {
            bail!("GitHub API request for {} failed with HTTP status: {}", repo, resp.status());
        }

        let release: RemoteRelease = resp.json().with_context(|| format!("Failed to parse GitHub JSON for {repo}"))?;
        Ok(release)
    }

    // --- GitLab API ---
    fn fetch_gitlab_latest(&self, repo: &str) -> Result<RemoteRelease> {
        // e.g. https://gitlab.com/api/v4/projects/{url_encoded_path}/releases/permalink/latest
        let clean_path = repo.trim_start_matches("https://gitlab.com/").trim_end_matches(".git");
        let encoded_path = urlencoding_simple(clean_path);
        let url = format!("https://gitlab.com/api/v4/projects/{encoded_path}/releases/permalink/latest");

        let mut req = self.client.get(&url);
        if let Ok(token) = env::var("GITLAB_TOKEN").or_else(|_| env::var("GL_TOKEN")) {
            req = req.header("PRIVATE-TOKEN", token.trim());
        }

        let resp = req.send().with_context(|| format!("GitLab API request failed for {url}"))?;
        if !resp.status().is_success() {
            bail!("GitLab API request for {} failed with HTTP status: {}", repo, resp.status());
        }

        #[derive(Deserialize)]
        struct GitLabReleaseJson {
            tag_name: String,
            name: Option<String>,
            released_at: Option<String>,
            commit: Option<GitLabCommit>,
            assets: Option<GitLabAssets>,
        }
        #[derive(Deserialize)]
        struct GitLabCommit {
            id: Option<String>,
        }
        #[derive(Deserialize)]
        struct GitLabAssets {
            links: Option<Vec<GitLabAssetLink>>,
        }
        #[derive(Deserialize)]
        struct GitLabAssetLink {
            name: String,
            url: String,
        }

        let gl: GitLabReleaseJson = resp.json().with_context(|| format!("Parsing GitLab release JSON for {repo}"))?;
        let assets = gl.assets.and_then(|a| a.links).unwrap_or_default().into_iter().map(|l| RemoteAsset {
            name: l.name,
            browser_download_url: l.url,
            size: 0,
            content_type: None,
        }).collect();

        Ok(RemoteRelease {
            tag_name: gl.tag_name,
            name: gl.name,
            published_at: gl.released_at,
            target_commitish: gl.commit.and_then(|c| c.id),
            assets,
        })
    }

    // --- Codeberg / Gitea API ---
    fn fetch_codeberg_latest(&self, repo: &str) -> Result<RemoteRelease> {
        let clean_repo = repo.trim_start_matches("https://codeberg.org/").trim_end_matches(".git");
        let url = format!("https://codeberg.org/api/v1/repos/{clean_repo}/releases/latest");
        let mut req = self.client.get(&url);

        if let Ok(token) = env::var("CODEBERG_TOKEN").or_else(|_| env::var("GITEA_TOKEN")) {
            if let Ok(val) = HeaderValue::from_str(&format!("token {}", token.trim())) {
                req = req.header(AUTHORIZATION, val);
            }
        }

        let resp = req.send().with_context(|| format!("Codeberg API request failed for {url}"))?;
        if !resp.status().is_success() {
            bail!("Codeberg API request for {} failed with HTTP status: {}", repo, resp.status());
        }

        let release: RemoteRelease = resp.json().with_context(|| format!("Parsing Codeberg JSON for {repo}"))?;
        Ok(release)
    }

    // --- Bitbucket API ---
    fn fetch_bitbucket_latest(&self, repo: &str) -> Result<RemoteRelease> {
        let clean_repo = repo.trim_start_matches("https://bitbucket.org/").trim_end_matches(".git");
        let url = format!("https://api.bitbucket.org/2.0/repositories/{clean_repo}/downloads");
        let mut req = self.client.get(&url);

        if let Ok(token) = env::var("BITBUCKET_TOKEN") {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", token.trim())) {
                req = req.header(AUTHORIZATION, val);
            }
        }

        let resp = req.send().with_context(|| format!("Bitbucket API request failed for {url}"))?;
        if !resp.status().is_success() {
            bail!("Bitbucket API request for {} failed with HTTP status: {}", repo, resp.status());
        }

        #[derive(Deserialize)]
        struct BitbucketDownloads {
            values: Option<Vec<BitbucketDownloadItem>>,
        }
        #[derive(Deserialize)]
        struct BitbucketDownloadItem {
            name: String,
            links: BitbucketDownloadLinks,
            size: Option<u64>,
        }
        #[derive(Deserialize)]
        struct BitbucketDownloadLinks {
            #[serde(rename = "self")]
            self_link: BitbucketHref,
        }
        #[derive(Deserialize)]
        struct BitbucketHref {
            href: String,
        }

        let bb: BitbucketDownloads = resp.json().with_context(|| format!("Parsing Bitbucket downloads JSON for {repo}"))?;
        let assets = bb.values.unwrap_or_default().into_iter().map(|item| RemoteAsset {
            name: item.name,
            browser_download_url: item.links.self_link.href,
            size: item.size.unwrap_or(0),
            content_type: None,
        }).collect();

        Ok(RemoteRelease {
            tag_name: "latest".to_string(),
            name: Some("Latest Downloads".to_string()),
            published_at: None,
            target_commitish: None,
            assets,
        })
    }

    // --- Launchpad API ---
    fn fetch_launchpad_latest(&self, repo: &str) -> Result<RemoteRelease> {
        let project = repo.trim_start_matches("https://launchpad.net/").trim_start_matches("lp:").trim_end_matches('/');
        let url = format!("https://api.launchpad.net/1.0/{project}/releases");

        let resp = self.client.get(&url).send().with_context(|| format!("Launchpad API request failed for {url}"))?;
        if !resp.status().is_success() {
            bail!("Launchpad API request for {} failed with HTTP status: {}", project, resp.status());
        }

        #[derive(Deserialize)]
        struct LaunchpadEntries {
            entries: Option<Vec<LaunchpadReleaseEntry>>,
        }
        #[derive(Deserialize)]
        struct LaunchpadReleaseEntry {
            version: String,
            date_released: Option<String>,
        }

        let lp: LaunchpadEntries = resp.json().with_context(|| format!("Parsing Launchpad releases for {project}"))?;
        let latest = lp.entries.and_then(|mut e| e.pop()).with_context(|| format!("No releases found on Launchpad for {project}"))?;

        Ok(RemoteRelease {
            tag_name: latest.version,
            name: None,
            published_at: latest.date_released,
            target_commitish: None,
            assets: Vec::new(),
        })
    }

    /// Securely clone repository using git CLI (handles GitHub, GitLab, Bitbucket, Codeberg, custom servers)
    pub fn git_clone<P: AsRef<Path>>(
        repo_url: &str,
        tag_or_branch: Option<&str>,
        dest: P,
    ) -> Result<()> {
        Self::validate_url(repo_url)?;

        let dest = dest.as_ref();
        let mut cmd = Command::new("git");
        cmd.arg("clone").arg("--depth").arg("1");

        if let Some(ref_name) = tag_or_branch {
            cmd.arg("--branch").arg(ref_name);
        }

        cmd.arg(repo_url).arg(dest);

        let status = cmd.status().with_context(|| format!("Failed to spawn git clone for {repo_url}"))?;
        if !status.success() {
            bail!("git clone failed with exit status {:?}", status.code());
        }

        Ok(())
    }
}

fn urlencoding_simple(s: &str) -> String {
    s.replace('/', "%2F").replace(':', "%3A")
}

/// Strategy for installing or updating a tool based on local presence and remote releases
#[derive(Debug, Clone)]
pub enum ToolLifecycleStrategy {
    /// Tool is missing locally; download prebuilt binary release from remote forge
    DownloadPrebuilt {
        version: String,
        url: String,
        asset_name: String,
    },
    /// Tool exists locally; can use existing local tool to compile updated release from source
    BuildSelfHosted {
        existing_version: String,
        target_version: String,
        bootstrap_bin: PathBuf,
        git_repo_url: String,
        git_tag: String,
    },
    /// Local version is already up to date
    UpToDate {
        version: String,
    },
}

impl ToolLifecycleStrategy {
    /// Determine lifecycle action for a tool given local distro install and remote release
    pub fn resolve(
        local_distro_path: &Path,
        repo: &str,
        platform: &str,
        arch: &str,
        binary_subpath: &str,
        git_repo_url: &str,
    ) -> Result<Self> {
        let client = ForgeClient::new()?;
        let latest = client.fetch_latest_release(repo)?;
        let remote_version = latest.clean_version();

        let bin_path = local_distro_path.join(binary_subpath);
        let meta_path = local_distro_path.join("pkg_meta.json");

        if !local_distro_path.exists() || !bin_path.exists() {
            // Path A: Not found locally -> Download prebuilt release asset
            if let Some(asset) = latest.find_asset(platform, arch) {
                return Ok(Self::DownloadPrebuilt {
                    version: remote_version,
                    url: asset.browser_download_url.clone(),
                    asset_name: asset.name.clone(),
                });
            } else {
                bail!("No matching prebuilt asset found for {platform}/{arch} in {repo} release {}", latest.tag_name);
            }
        }

        // Path B: Tool found locally -> Check version in pkg_meta.json
        let local_version = if meta_path.exists() {
            if let Ok(meta) = crate::core::meta::PkgMeta::load_from_file(&meta_path) {
                meta.version
            } else {
                "0.0.0".to_string()
            }
        } else {
            "0.0.0".to_string()
        };

        if local_version == remote_version {
            return Ok(Self::UpToDate { version: local_version });
        }

        // Outdated: Bootstrap self-build using the current local tool
        Ok(Self::BuildSelfHosted {
            existing_version: local_version,
            target_version: remote_version,
            bootstrap_bin: bin_path,
            git_repo_url: git_repo_url.to_string(),
            git_tag: latest.tag_name,
        })
    }
}

