// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/core/distro.rs
// created: 2026-10-14
// lastModified: 2026-10-15

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
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

    /// Recursively copy all files and directories
    pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let from = entry.path();
            let to = dst.join(entry.file_name());
            if ty.is_dir() {
                Self::copy_dir_all(&from, &to)?;
            } else {
                fs::copy(&from, &to)?;
            }
        }
        Ok(())
    }

    /// Recursively scan installed directory and record file paths, sha256 hashes and sizes
    pub fn scan_installed_files(root: &Path) -> Result<Vec<FileRecord>> {
        let mut records = Vec::new();

        fn visit(dir: &Path, root: &Path, records: &mut Vec<FileRecord>) -> Result<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit(&path, root, records)?;
                } else if path.is_file() {
                    let mut file = File::open(&path)?;
                    let mut hasher = Sha256::new();
                    let mut buffer = [0u8; 65536];
                    let mut size = 0u64;
                    loop {
                        let n = file.read(&mut buffer)?;
                        if n == 0 {
                            break;
                        }
                        hasher.update(&buffer[..n]);
                        size += n as u64;
                    }
                    let sha256 = hex::encode(hasher.finalize());
                    let rel = path.strip_prefix(root)?.to_string_lossy().replace('\\', "/");
                    records.push(FileRecord {
                        path: rel,
                        sha256,
                        size,
                    });
                }
            }
            Ok(())
        }

        visit(root, root, &mut records)?;
        Ok(records)
    }

    /// Locate 7z binary on system if available
    pub fn find_7z_binary() -> Option<PathBuf> {
        if let Ok(p) = which::which("7z") {
            return Some(p);
        }
        if let Ok(p) = which::which("7za") {
            return Some(p);
        }
        #[cfg(windows)]
        {
            for path in &[
                r"C:\Program Files\7-Zip\7z.exe",
                r"C:\Program Files (x86)\7-Zip\7z.exe",
            ] {
                let pb = PathBuf::from(path);
                if pb.exists() {
                    return Some(pb);
                }
            }
        }
        None
    }

    /// Extract archive using system tar or 7z tool
    pub fn unpack_with_external_tool(
        bytes: &[u8],
        tool_cmd: &str,
        tool_args: &[&str],
        dest: &Path,
        file_ext: &str,
        strip_prefix: Option<&str>,
    ) -> Result<Vec<FileRecord>> {
        fs::create_dir_all(dest)?;
        let nonce = format!("{}_{}", std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
        let temp_archive = std::env::temp_dir().join(format!("nbuild_{}.{}", nonce, file_ext));
        let temp_extract = std::env::temp_dir().join(format!("nbuild_out_{}", nonce));
        fs::create_dir_all(&temp_extract)?;

        fs::write(&temp_archive, bytes)?;

        let mut cmd = Command::new(tool_cmd);
        for arg in tool_args {
            let replaced = arg
                .replace("{ARCHIVE}", &temp_archive.to_string_lossy())
                .replace("{DEST}", &temp_extract.to_string_lossy());
            cmd.arg(replaced);
        }

        let status = cmd.status().with_context(|| format!("Executing {tool_cmd} to unpack archive"))?;
        let _ = fs::remove_file(&temp_archive);

        if !status.success() {
            let _ = fs::remove_dir_all(&temp_extract);
            bail!("{tool_cmd} extraction failed with code {:?}", status.code());
        }

        // Determine directory to copy from
        let copy_from = if let Some(prefix) = strip_prefix {
            let candidate = temp_extract.join(prefix);
            if candidate.is_dir() {
                candidate
            } else {
                temp_extract.clone()
            }
        } else {
            let entries: Vec<_> = fs::read_dir(&temp_extract)?.filter_map(|e| e.ok()).collect();
            if entries.len() == 1 && entries[0].path().is_dir() {
                let inner = entries[0].path();
                if inner.join("bin").exists()
                    || inner.join("lib").exists()
                    || inner.join("include").exists()
                    || inner.join("share").exists()
                {
                    inner
                } else {
                    temp_extract.clone()
                }
            } else {
                temp_extract.clone()
            }
        };

        Self::copy_dir_all(&copy_from, dest)?;
        let _ = fs::remove_dir_all(&temp_extract);

        Self::scan_installed_files(dest)
    }

    /// Extract any supported archive format: zip, tar.gz, tar.xz, tar.zst, tar.bz2, 7z, rar, or raw binary
    pub fn unpack_archive(
        bytes: &[u8],
        filename_or_url: &str,
        dest: &Path,
        strip_prefix: Option<&str>,
    ) -> Result<Vec<FileRecord>> {
        let lower = filename_or_url.to_lowercase();

        if lower.ends_with(".zip") {
            Self::unpack_zip(bytes, dest, strip_prefix)
        } else if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
            Self::unpack_tar_gz(bytes, dest, strip_prefix)
        } else if lower.ends_with(".tar.xz") || lower.ends_with(".txz") {
            Self::unpack_with_external_tool(bytes, "tar", &["-xf", "{ARCHIVE}", "-C", "{DEST}"], dest, "tar.xz", strip_prefix)
        } else if lower.ends_with(".tar.zst") || lower.ends_with(".tzst") {
            Self::unpack_with_external_tool(bytes, "tar", &["-xf", "{ARCHIVE}", "-C", "{DEST}"], dest, "tar.zst", strip_prefix)
        } else if lower.ends_with(".tar.bz2") || lower.ends_with(".tbz2") || lower.ends_with(".bz2") {
            Self::unpack_with_external_tool(bytes, "tar", &["-xf", "{ARCHIVE}", "-C", "{DEST}"], dest, "tar.bz2", strip_prefix)
        } else if lower.ends_with(".7z") {
            if let Some(seven_zip) = Self::find_7z_binary() {
                let cmd_str = seven_zip.to_string_lossy().to_string();
                Self::unpack_with_external_tool(bytes, &cmd_str, &["x", "-y", "{ARCHIVE}", "-o{DEST}"], dest, "7z", strip_prefix)
            } else {
                Self::unpack_with_external_tool(bytes, "tar", &["-xf", "{ARCHIVE}", "-C", "{DEST}"], dest, "7z", strip_prefix)
            }
        } else if lower.ends_with(".rar") {
            if let Some(seven_zip) = Self::find_7z_binary() {
                let cmd_str = seven_zip.to_string_lossy().to_string();
                Self::unpack_with_external_tool(bytes, &cmd_str, &["x", "-y", "{ARCHIVE}", "-o{DEST}"], dest, "rar", strip_prefix)
            } else {
                Self::unpack_with_external_tool(bytes, "tar", &["-xf", "{ARCHIVE}", "-C", "{DEST}"], dest, "rar", strip_prefix)
            }
        } else {
            // Raw binary executable
            let file_name = Path::new(filename_or_url).file_name().unwrap_or_default();
            let bin_dir = dest.join("bin");
            fs::create_dir_all(&bin_dir)?;
            let file_path = bin_dir.join(file_name);
            fs::write(&file_path, bytes)?;

            let sha256 = Self::calc_sha256(bytes);
            Ok(vec![FileRecord {
                path: format!("bin/{}", file_name.to_string_lossy()),
                sha256,
                size: bytes.len() as u64,
            }])
        }
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
        let records = Self::unpack_archive(&bytes, &source.url, &target_dir, source.strip_prefix.as_deref())?;

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
