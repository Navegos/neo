// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/neo-build/src/core/distro.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use anyhow::{bail, Context, Result};
use hex;
use sha2::{Digest, Sha256};

use super::arch::TargetTriple;
use super::catalog::SourceSpec;
use super::config::NeoConfig;
use super::meta::{FileRecord, PkgMeta, TargetInfo};

pub struct DistroInstaller<'a> {
    pub config: &'a NeoConfig,
}

impl<'a> DistroInstaller<'a> {
    pub fn new(config: &'a NeoConfig) -> Self {
        Self { config }
    }

    /// Calculate SHA256 of a byte slice
    pub fn calc_sha256(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }

    /// Calculate SHA256 of a file on disk
    pub fn calc_file_sha256<P: AsRef<Path>>(path: P) -> Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        Ok(hex::encode(hasher.finalize()))
    }

    /// Download payload from URL and verify checksum
    pub fn download(&self, url: &str, expected_sha256: Option<&str>) -> Result<Vec<u8>> {
        let response = reqwest::blocking::get(url)
            .with_context(|| format!("Failed to download from {url}"))?;

        if !response.status().is_success() {
            bail!("Download failed with HTTP status: {}", response.status());
        }

        let bytes = response.bytes()?.to_vec();

        if let Some(expected) = expected_sha256 {
            let actual = Self::calc_sha256(&bytes);
            if actual.to_lowercase() != expected.to_lowercase() {
                bail!("SHA256 mismatch for {url}: expected {expected}, got {actual}");
            }
        }

        Ok(bytes)
    }

    /// Unpack zip archive into destination directory with optional prefix stripping
    pub fn unpack_zip(bytes: &[u8], dest: &Path, strip_prefix: Option<&str>) -> Result<Vec<FileRecord>> {
        fs::create_dir_all(dest)?;
        let reader = io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader)?;
        let mut records = Vec::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let raw_path = match file.enclosed_name() {
                Some(p) => p.to_path_buf(),
                None => continue,
            };

            let target_path = if let Some(prefix) = strip_prefix {
                if let Ok(stripped) = raw_path.strip_prefix(prefix) {
                    dest.join(stripped)
                } else {
                    dest.join(&raw_path)
                }
            } else {
                dest.join(&raw_path)
            };

            if file.is_dir() {
                fs::create_dir_all(&target_path)?;
            } else {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut out_file = File::create(&target_path)?;
                let mut content = Vec::new();
                file.read_to_end(&mut content)?;
                io::copy(&mut io::Cursor::new(&content), &mut out_file)?;

                let sha256 = Self::calc_sha256(&content);
                let rel = target_path.strip_prefix(dest)?.to_string_lossy().to_string();
                records.push(FileRecord {
                    path: rel,
                    sha256,
                    size: content.len() as u64,
                });
            }
        }

        Ok(records)
    }

    /// Unpack tar.gz archive into destination directory
    pub fn unpack_tar_gz(bytes: &[u8], dest: &Path, strip_prefix: Option<&str>) -> Result<Vec<FileRecord>> {
        fs::create_dir_all(dest)?;
        let tar = flate2::read::GzDecoder::new(io::Cursor::new(bytes));
        let mut archive = tar::Archive::new(tar);
        let mut records = Vec::new();

        for entry in archive.entries()? {
            let mut entry = entry?;
            let raw_path = entry.path()?.to_path_buf();

            let target_path = if let Some(prefix) = strip_prefix {
                if let Ok(stripped) = raw_path.strip_prefix(prefix) {
                    dest.join(stripped)
                } else {
                    dest.join(&raw_path)
                }
            } else {
                dest.join(&raw_path)
            };

            if entry.header().entry_type().is_dir() {
                fs::create_dir_all(&target_path)?;
            } else {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut content = Vec::new();
                entry.read_to_end(&mut content)?;
                let mut out_file = File::create(&target_path)?;
                io::copy(&mut io::Cursor::new(&content), &mut out_file)?;

                let sha256 = Self::calc_sha256(&content);
                let rel = target_path.strip_prefix(dest)?.to_string_lossy().to_string();
                records.push(FileRecord {
                    path: rel,
                    sha256,
                    size: content.len() as u64,
                });
            }
        }

        Ok(records)
    }

    /// Install a prebuilt source specification into distro/<pkg_name>
    pub fn install_prebuilt(
        &self,
        triple: &TargetTriple,
        name: &str,
        version: &str,
        _category: &str,
        target_level: &str,
        source: &SourceSpec,
    ) -> Result<PkgMeta> {
        let platform_str = triple.platform.as_str();
        let target_dir = self.config.pkg_distro_dir(platform_str, &triple.raw, name);

        // Clean existing dir if reinstalling
        if target_dir.exists() {
            let _ = fs::remove_dir_all(&target_dir);
        }
        fs::create_dir_all(&target_dir)?;

        let bytes = self.download(&source.url, source.sha256.as_deref())?;

        let url_lower = source.url.to_lowercase();
        let records = if url_lower.ends_with(".zip") {
            Self::unpack_zip(&bytes, &target_dir, source.strip_prefix.as_deref())?
        } else if url_lower.ends_with(".tar.gz") || url_lower.ends_with(".tgz") {
            Self::unpack_tar_gz(&bytes, &target_dir, source.strip_prefix.as_deref())?
        } else {
            // Raw binary executable
            let file_name = Path::new(&source.url).file_name().unwrap_or_default();
            let bin_dir = target_dir.join("bin");
            fs::create_dir_all(&bin_dir)?;
            let file_path = bin_dir.join(file_name);
            fs::write(&file_path, &bytes)?;

            let sha256 = Self::calc_sha256(&bytes);
            vec![FileRecord {
                path: format!("bin/{}", file_name.to_string_lossy()),
                sha256,
                size: bytes.len() as u64,
            }]
        };

        let target_info = TargetInfo {
            platform: platform_str.to_string(),
            triple: triple.raw.clone(),
            arch: triple.arch.clone(),
            target_level: target_level.to_string(),
        };

        let mut meta = PkgMeta::new(name, version, "prebuilt_binary", target_info);
        meta.url = Some(source.url.clone());
        meta.sha256 = source.sha256.clone();
        meta.installed_files = records;

        let meta_path = self.config.pkg_meta_path(platform_str, &triple.raw, name);
        meta.save_to_file(meta_path)?;

        Ok(meta)
    }
}
