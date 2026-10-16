// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/core/doctor.rs
// created: 2026-10-16

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result};

use super::arch::TargetTriple;
use super::config::NeoConfig;
use super::distro::DistroInstaller;
use super::env_ops::EnvManager;
use super::meta::{FileRecord, PkgMeta, TargetInfo};
use super::symlink::LinkEngine;
use super::sysroot::SysrootManager;

#[derive(Debug, Default)]
pub struct DoctorReport {
    pub topology_ok: bool,
    pub topology_msg: String,
    pub broken_links_found: usize,
    pub broken_links_repaired: usize,
    pub broken_packages_found: usize,
    pub broken_packages_repaired: usize,
    pub env_toolchains_ok: bool,
    pub env_path_ok: bool,
    pub smoke_test_ok: bool,
}

pub struct Doctor;

impl Doctor {
    /// Check whether nbuild is installed in the canonical toolchain structure
    pub fn check_topology(config: &NeoConfig, triple: &TargetTriple) -> (bool, PathBuf, PathBuf) {
        let canonical_bin = config
            .pkg_distro_dir(triple.platform.as_str(), &triple.raw, "neo")
            .join("bin")
            .join(if cfg!(windows) { "nbuild.exe" } else { "nbuild" });

        let current_exe = env::current_exe().unwrap_or_default();

        let is_canonical = if current_exe.exists() && canonical_bin.exists() {
            if let (Ok(c1), Ok(c2)) = (fs::canonicalize(&current_exe), fs::canonicalize(&canonical_bin)) {
                c1 == c2
            } else {
                current_exe == canonical_bin
            }
        } else {
            false
        };

        (is_canonical, current_exe, canonical_bin)
    }

    /// Install current executable and companion tools into canonical distro/neo/bin
    pub fn self_install(config: &NeoConfig, triple: &TargetTriple) -> Result<PathBuf> {
        let (is_canonical, current_exe, canonical_bin) = Self::check_topology(config, triple);
        if is_canonical {
            return Ok(canonical_bin);
        }

        let distro_neo = config.pkg_distro_dir(triple.platform.as_str(), &triple.raw, "neo");
        let bin_dir = distro_neo.join("bin");
        fs::create_dir_all(&bin_dir)?;

        let exe_name = if cfg!(windows) { "nbuild.exe" } else { "nbuild" };
        let dest_exe = bin_dir.join(exe_name);

        println!("[nbuild] Installing self to canonical location: {}", dest_exe.display());

        // Copy or rename current running executable
        if current_exe.exists() {
            fs::copy(&current_exe, &dest_exe)
                .with_context(|| format!("Failed to copy {} to {}", current_exe.display(), dest_exe.display()))?;
        }

        // Also look for sibling binaries (archinfo, tokless) in current exe directory
        if let Some(parent) = current_exe.parent() {
            let companions = if cfg!(windows) {
                vec!["archinfo.exe", "tokless.exe"]
            } else {
                vec!["archinfo", "tokless"]
            };

            for companion in companions {
                let comp_src = parent.join(companion);
                let comp_dest = bin_dir.join(companion);
                if comp_src.exists() && !comp_dest.exists() {
                    let _ = fs::copy(&comp_src, &comp_dest);
                    println!("[nbuild] Provisioned companion binary: {}", companion);
                }
            }
        }

        // Generate pkg_meta.json for neo package
        let mut installed_files = Vec::new();
        for entry in fs::read_dir(&bin_dir)?.flatten() {
            let p = entry.path();
            if p.is_file() {
                let rel = format!("bin/{}", p.file_name().unwrap_or_default().to_string_lossy());
                let bytes = fs::read(&p).unwrap_or_default();
                installed_files.push(FileRecord {
                    path: rel,
                    sha256: DistroInstaller::calc_sha256(&bytes),
                    size: bytes.len() as u64,
                });
            }
        }

        let target_info = TargetInfo {
            platform: triple.platform.as_str().to_string(),
            triple: triple.raw.clone(),
            arch: triple.arch.clone(),
            target_level: "system".to_string(),
        };

        let mut meta = PkgMeta::new("neo", env!("CARGO_PKG_VERSION"), "prebuilt_binary", target_info);
        meta.url = Some("https://api.github.com/repos/Navegos/neo/releases/latest".to_string());
        meta.installed_files = installed_files;

        // Provision sysroot and host symlinks
        let sysroot = SysrootManager::new(config);
        sysroot.link_package(triple, &mut meta)?;

        Ok(dest_exe)
    }

    /// Scan and repair broken symlinks
    pub fn heal_symlinks(config: &NeoConfig, triple: &TargetTriple) -> Result<(usize, usize)> {
        let sysroot = SysrootManager::new(config);
        sysroot.ensure_gnu_toolchain_dirs(triple)?;

        let mut found = 0;
        let mut repaired = 0;

        let host_bin = config.host_usr_bin();
        let triple_bin = config.triple_usr_dir(triple.platform.as_str(), &triple.raw).join("bin");

        let dirs_to_check = [host_bin, triple_bin];

        for check_dir in dirs_to_check {
            if !check_dir.exists() {
                continue;
            }

            for entry in fs::read_dir(&check_dir)?.flatten() {
                let path = entry.path();
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

                // Check if file is broken link / zero byte or points nowhere
                let is_broken = if path.is_symlink() {
                    match fs::read_link(&path) {
                        Ok(target) => !target.exists(),
                        Err(_) => true,
                    }
                } else if path.exists() {
                    // Check if file metadata is accessible
                    fs::metadata(&path).is_err()
                } else {
                    true
                };

                if is_broken {
                    found += 1;
                    // Attempt to heal: find target in distro packages
                    let distro_root = config.distro_dir(triple.platform.as_str(), &triple.raw);
                    let mut healed = false;

                    if distro_root.exists() {
                        if let Ok(distro_entries) = fs::read_dir(&distro_root) {
                            for pkg in distro_entries.flatten() {
                                let candidate = pkg.path().join("bin").join(&file_name);
                                if candidate.exists() {
                                    let _ = fs::remove_file(&path);
                                    let _ = LinkEngine::link_file(&candidate, &path, false);
                                    healed = true;
                                    repaired += 1;
                                    break;
                                }
                            }
                        }
                    }

                    if !healed {
                        // Purge dead link
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }

        Ok((found, repaired))
    }

    /// Check package file integrity and heal missing files
    pub fn heal_packages(config: &NeoConfig, triple: &TargetTriple) -> Result<(usize, usize)> {
        let distro_root = config.distro_dir(triple.platform.as_str(), &triple.raw);
        if !distro_root.exists() {
            return Ok((0, 0));
        }

        let mut broken_found = 0;
        let mut broken_repaired = 0;

        for entry in fs::read_dir(&distro_root)?.flatten() {
            let pkg_dir = entry.path();
            let meta_path = pkg_dir.join("pkg_meta.json");
            if !meta_path.exists() {
                continue;
            }

            if let Ok(mut meta) = PkgMeta::load_from_file(&meta_path) {
                let mut package_corrupted = false;
                for record in &meta.installed_files {
                    let file_path = pkg_dir.join(&record.path);
                    if !file_path.exists() {
                        package_corrupted = true;
                        break;
                    }
                }

                if package_corrupted {
                    broken_found += 1;
                    println!("[nbuild] Package '{}' has missing or corrupted files. Healing symlinks...", meta.name);
                    let sysroot = SysrootManager::new(config);
                    if sysroot.link_package(triple, &mut meta).is_ok() {
                        broken_repaired += 1;
                    }
                }
            }
        }

        Ok((broken_found, broken_repaired))
    }

    /// Run smoke test to verify installation is operational
    pub fn smoke_test(config: &NeoConfig, triple: &TargetTriple) -> bool {
        let canonical_bin = config
            .pkg_distro_dir(triple.platform.as_str(), &triple.raw, "neo")
            .join("bin")
            .join(if cfg!(windows) { "nbuild.exe" } else { "nbuild" });

        if !canonical_bin.exists() {
            return false;
        }

        let output = Command::new(&canonical_bin)
            .arg("--version")
            .output();

        match output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }

    /// Run full diagnosis and self-healing suite
    pub fn run_doctor(config: &mut NeoConfig, triple: &TargetTriple, heal: bool) -> Result<DoctorReport> {
        let mut report = DoctorReport::default();

        // 1. Check environment
        report.env_toolchains_ok = EnvManager::get_toolchains_dir_env().is_some();
        report.env_path_ok = EnvManager::is_usr_bin_in_path(config);

        if heal && (!report.env_toolchains_ok || !report.env_path_ok) {
            println!("[nbuild] Healing environment and PATH...");
            if let Ok(new_cfg) = EnvManager::ensure_environment(false) {
                *config = new_cfg;
                report.env_toolchains_ok = true;
                report.env_path_ok = true;
            }
        }

        // 2. Check topology
        let (is_canonical, current_exe, canonical_bin) = Self::check_topology(config, triple);
        report.topology_ok = is_canonical;

        if !is_canonical {
            report.topology_msg = format!(
                "Not installed in canonical directory structure.\n  Running exe: {}\n  Expected:    {}",
                current_exe.display(),
                canonical_bin.display()
            );
            if heal {
                println!("[nbuild] Healing directory topology by installing self into canonical location...");
                if let Ok(installed) = Self::self_install(config, triple) {
                    report.topology_ok = true;
                    report.topology_msg = format!("Self-installed successfully at: {}", installed.display());
                }
            }
        } else {
            report.topology_msg = format!("Installed at canonical location: {}", canonical_bin.display());
        }

        // 3. Symlinks
        if heal {
            let (found, repaired) = Self::heal_symlinks(config, triple)?;
            report.broken_links_found = found;
            report.broken_links_repaired = repaired;
        }

        // 4. Packages
        if heal {
            let (pkg_found, pkg_repaired) = Self::heal_packages(config, triple)?;
            report.broken_packages_found = pkg_found;
            report.broken_packages_repaired = pkg_repaired;
        }

        // 5. Smoke test
        report.smoke_test_ok = Self::smoke_test(config, triple);

        Ok(report)
    }
}
