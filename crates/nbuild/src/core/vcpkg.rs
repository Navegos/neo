// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/core/vcpkg.rs
// created: 2026-10-17

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{bail, Context, Result};

use super::arch::TargetTriple;
use super::catalog::PackageCatalog;
use super::config::NeoConfig;
use super::distro::DistroInstaller;
use super::meta::{PkgMeta, TargetInfo};

#[derive(Debug, Clone)]
pub struct VcpkgInstance {
    pub exe: PathBuf,
    pub root: PathBuf,
}

pub struct VcpkgManager;

impl VcpkgManager {
    pub fn new() -> Self {
        Self
    }

    /// Locate vcpkg executable and root directory across toolchains, env vars, and system paths
    pub fn locate_vcpkg(config: &NeoConfig, triple: &TargetTriple) -> Result<VcpkgInstance> {
        let exe_name = if cfg!(windows) { "vcpkg.exe" } else { "vcpkg" };

        // 1. Check VCPKG_ROOT / VCPKG_PATH environment variables
        for var_name in &["VCPKG_ROOT", "VCPKG_PATH"] {
            if let Ok(val) = env::var(var_name) {
                let root = PathBuf::from(val.trim());
                if root.exists() {
                    let exe_candidates = [
                        root.join(exe_name),
                        root.join("bin").join(exe_name),
                    ];
                    for exe in &exe_candidates {
                        if exe.is_file() {
                            return Ok(VcpkgInstance { exe: exe.clone(), root });
                        }
                    }
                }
            }
        }

        // 2. Check canonical toolchain distro vcpkg
        let distro_vcpkg = config.pkg_distro_dir(triple.platform.as_str(), &triple.raw, "vcpkg");
        if distro_vcpkg.exists() {
            let exe_candidates = [
                distro_vcpkg.join("bin").join(exe_name),
                distro_vcpkg.join(exe_name),
            ];
            for exe in &exe_candidates {
                if exe.is_file() {
                    return Ok(VcpkgInstance { exe: exe.clone(), root: distro_vcpkg });
                }
            }
        }

        // 3. Check toolchain host active prefix ($TOOLCHAINS_DIR/usr/bin/vcpkg.exe)
        let host_bin = config.host_usr_bin().join(exe_name);
        if host_bin.is_file() {
            let root = config.pkg_distro_dir(triple.platform.as_str(), &triple.raw, "vcpkg");
            return Ok(VcpkgInstance { exe: host_bin, root });
        }

        // 4. Check system PATH via which
        if let Ok(exe) = which::which(exe_name) {
            let root = if let Some(parent) = exe.parent() {
                if parent.file_name().map_or(false, |n| n == "bin") {
                    parent.parent().unwrap_or(parent).to_path_buf()
                } else {
                    parent.to_path_buf()
                }
            } else {
                PathBuf::from(".")
            };
            return Ok(VcpkgInstance { exe, root });
        }

        // 5. Check well-known default locations on Windows
        #[cfg(windows)]
        {
            let candidates = [
                r"C:\opt\libs\vcpkg",
                r"C:\vcpkg",
                r"C:\opt\vcpkg",
                r"C:\src\vcpkg",
            ];
            for path_str in &candidates {
                let root = PathBuf::from(path_str);
                let exe = root.join("vcpkg.exe");
                if exe.is_file() {
                    return Ok(VcpkgInstance { exe, root });
                }
            }
        }

        // 6. Check well-known default locations on Unix
        #[cfg(unix)]
        {
            let candidates = [
                "/opt/vcpkg",
                "/usr/local/vcpkg",
            ];
            for path_str in &candidates {
                let root = PathBuf::from(path_str);
                let exe = root.join("vcpkg");
                if exe.is_file() {
                    return Ok(VcpkgInstance { exe, root });
                }
            }
        }

        bail!("vcpkg was not found. Install it via 'nbuild get vcpkg' or set VCPKG_ROOT.");
    }

    /// Install/provision a package from vcpkg and assemble into distro/<pkg_name>
    pub fn install_package(
        &self,
        config: &NeoConfig,
        triple: &TargetTriple,
        cat: &PackageCatalog,
    ) -> Result<PkgMeta> {
        let vcpkg = Self::locate_vcpkg(config, triple)?;
        let vcpkg_triplet = triple.to_vcpkg_triplet();
        let port_spec = cat.vcpkg_port.as_deref().unwrap_or(&cat.name);
        let base_port = port_spec.split('[').next().unwrap_or(port_spec);

        let target_dir = config.pkg_distro_dir(triple.platform.as_str(), &triple.raw, &cat.name);
        let meta_path = config.pkg_meta_path(triple.platform.as_str(), &triple.raw, &cat.name);

        // Delete previous package files cleanly
        DistroInstaller::clean_existing_package(&target_dir, &meta_path)?;
        fs::create_dir_all(&target_dir)?;

        let installed_root = vcpkg.root.join("installed").join(&vcpkg_triplet);

        // Check if vcpkg list shows port already installed
        let is_installed = self.check_is_installed(&vcpkg.exe, base_port, &vcpkg_triplet);
        if !is_installed {
            // Run vcpkg install --recurse <port_spec>:<triplet>
            let mut cmd = Command::new(&vcpkg.exe);
            cmd.current_dir(&vcpkg.root)
               .arg("install")
               .arg("--recurse")
               .arg(format!("{}:{}", port_spec, vcpkg_triplet));

            let status = cmd.status().with_context(|| format!("Executing vcpkg install for {port_spec}"))?;
            if !status.success() {
                // If feature-qualified spec failed, try base port
                if port_spec != base_port {
                    let mut fallback = Command::new(&vcpkg.exe);
                    fallback.current_dir(&vcpkg.root)
                           .arg("install")
                           .arg("--recurse")
                           .arg(format!("{}:{}", base_port, vcpkg_triplet));
                    let fb_status = fallback.status().with_context(|| format!("Fallback vcpkg install for {base_port}"))?;
                    if !fb_status.success() {
                        bail!("vcpkg install failed with exit code: {:?}", fb_status.code());
                    }
                } else {
                    bail!("vcpkg install failed with exit code: {:?}", status.code());
                }
            }
        }

        if !installed_root.exists() {
            bail!("vcpkg installed directory not found at: {}", installed_root.display());
        }

        // Collect files from vcpkg installed directory into distro/<pkg_name>
        let mut deployed_count = 0;

        // 1. Tools / Executables
        let tools_dir = installed_root.join("tools").join(&cat.name);
        let target_bin = target_dir.join("bin");
        if tools_dir.exists() {
            fs::create_dir_all(&target_bin)?;
            let scan_dirs = [tools_dir.join("bin"), tools_dir.clone()];
            for sdir in &scan_dirs {
                if sdir.is_dir() {
                    for entry in fs::read_dir(sdir)?.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            let file_name = entry.file_name();
                            let is_exe = path.extension().map_or(false, |e| e == "exe") || cfg!(unix);
                            if is_exe {
                                fs::copy(&path, target_bin.join(&file_name))?;
                                deployed_count += 1;
                            }
                        }
                    }
                }
            }
        }

        // Check if vcpkg_tools specifies named tools to pull from installed_root/tools or bin
        for tool_name in &cat.vcpkg_tools {
            let possible_tool = installed_root.join("tools").join(&cat.name).join(if cfg!(windows) { format!("{tool_name}.exe") } else { tool_name.clone() });
            if possible_tool.is_file() {
                fs::create_dir_all(&target_bin)?;
                fs::copy(&possible_tool, target_bin.join(possible_tool.file_name().unwrap()))?;
                deployed_count += 1;
            }
        }

        // Check installed_root/bin for DLLs or tools matching package
        let vcpkg_bin = installed_root.join("bin");
        if vcpkg_bin.is_dir() {
            for entry in fs::read_dir(&vcpkg_bin)?.flatten() {
                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();
                let lower = file_name.to_lowercase();
                let matches_pkg = lower.contains(&cat.name.to_lowercase())
                    || cat.vcpkg_libs.iter().any(|p| lower.contains(&p.to_lowercase()));
                if matches_pkg && path.is_file() {
                    fs::create_dir_all(&target_bin)?;
                    fs::copy(&path, target_bin.join(&file_name))?;
                    deployed_count += 1;
                }
            }
        }

        // 2. Include / Headers
        let vcpkg_inc = installed_root.join("include");
        let target_inc = target_dir.join("include");
        if vcpkg_inc.is_dir() {
            // Check direct folder include/<pkg_name>
            let pkg_inc_folder = vcpkg_inc.join(&cat.name);
            if pkg_inc_folder.is_dir() {
                fs::create_dir_all(&target_inc)?;
                DistroInstaller::copy_dir_all(&pkg_inc_folder, &target_inc.join(&cat.name))?;
                deployed_count += 1;
            }

            // Check header files
            for entry in fs::read_dir(&vcpkg_inc)?.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    let lower = file_name.to_lowercase();
                    let matches_header = cat.vcpkg_headers.iter().any(|h| lower == h.to_lowercase() || lower.contains(&h.to_lowercase()))
                        || lower.contains(&cat.name.to_lowercase());
                    if matches_header {
                        fs::create_dir_all(&target_inc)?;
                        fs::copy(&path, target_inc.join(&file_name))?;
                        deployed_count += 1;
                    }
                }
            }
        }

        // 3. Libraries (lib)
        let vcpkg_lib = installed_root.join("lib");
        let target_lib = target_dir.join("lib");
        if vcpkg_lib.is_dir() {
            for entry in fs::read_dir(&vcpkg_lib)?.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    let lower = file_name.to_lowercase();
                    let matches_lib = cat.vcpkg_libs.iter().any(|l| lower.contains(&l.to_lowercase()))
                        || lower.contains(&cat.name.to_lowercase());
                    if matches_lib {
                        fs::create_dir_all(&target_lib)?;
                        fs::copy(&path, target_lib.join(&file_name))?;
                        deployed_count += 1;
                    }
                }
            }
        }

        // 4. Share (cmake / pkgconfig)
        let vcpkg_share_pkg = installed_root.join("share").join(&cat.name);
        let target_share_pkg = target_dir.join("share").join(&cat.name);
        if vcpkg_share_pkg.is_dir() {
            fs::create_dir_all(&target_share_pkg)?;
            DistroInstaller::copy_dir_all(&vcpkg_share_pkg, &target_share_pkg)?;
            deployed_count += 1;
        }

        if deployed_count == 0 {
            bail!("No package files found in vcpkg installed directory for '{}'", cat.name);
        }

        // Detect version
        let version = self.detect_version(&vcpkg.exe, base_port, &vcpkg_triplet)
            .unwrap_or_else(|| cat.version.clone());

        let records = DistroInstaller::scan_installed_files(&target_dir)?;

        let target_info = TargetInfo {
            platform: triple.platform.as_str().to_string(),
            triple: triple.raw.clone(),
            arch: triple.arch.clone(),
            target_level: cat.target_level.clone(),
        };

        let mut meta = PkgMeta::new(&cat.name, &version, "vcpkg", target_info);
        meta.installed_files = records;
        if !cat.env_exports.is_empty() {
            meta.env_exports = cat.env_exports.clone();
        }
        meta.save_to_file(&meta_path)?;

        Ok(meta)
    }

    fn check_is_installed(&self, exe: &Path, base_port: &str, triplet: &str) -> bool {
        let output = Command::new(exe)
            .args(["list", &format!("{}:{}", base_port, triplet)])
            .output();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.contains(base_port)
        } else {
            false
        }
    }

    fn detect_version(&self, exe: &Path, base_port: &str, triplet: &str) -> Option<String> {
        let output = Command::new(exe)
            .args(["list", &format!("{}:{}", base_port, triplet)])
            .output()
            .ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with(base_port) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let ver = parts[1].split('#').next().unwrap_or(parts[1]);
                    return Some(ver.to_string());
                }
            }
        }
        None
    }
}

