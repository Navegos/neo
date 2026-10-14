// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/neo-build/src/core/sysroot.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use std::fs;
use anyhow::Result;

use super::arch::TargetTriple;
use super::config::NeoConfig;
use super::meta::{PkgMeta, SymlinkRecord};
use super::symlink::{LinkEngine, LinkStatus};

pub struct SysrootManager<'a> {
    pub config: &'a NeoConfig,
}

impl<'a> SysrootManager<'a> {
    pub fn new(config: &'a NeoConfig) -> Self {
        Self { config }
    }

    /// Ensure complete GNU Toolchain directory structure exists for a triple
    pub fn ensure_gnu_toolchain_dirs(&self, triple: &TargetTriple) -> Result<()> {
        let platform_str = triple.platform.as_str();
        let triple_root = self.config.triple_dir(platform_str, &triple.raw);

        // 1. Root level GNU directories (System Compilers, OS SDKs, Runtimes)
        let root_dirs = [
            triple_root.join("bin"),
            triple_root.join("include"),
            triple_root.join("lib"),
            triple_root.join("lib64"),
            triple_root.join("libexec"),
            triple_root.join("share"),
            triple_root.join("distro"),
        ];

        for d in &root_dirs {
            fs::create_dir_all(d)?;
        }

        // 2. User level GNU directories (Libraries, Dependencies, Tools)
        let usr_root = self.config.triple_usr_dir(platform_str, &triple.raw);
        let usr_dirs = [
            usr_root.join("bin"),
            usr_root.join("include"),
            usr_root.join("lib"),
            usr_root.join("lib64"),
            usr_root.join("share"),
            usr_root.join("share").join("pkgconfig"),
        ];

        for d in &usr_dirs {
            fs::create_dir_all(d)?;
        }

        // 3. Host level usr directories
        let host_usr = self.config.host_usr_dir();
        fs::create_dir_all(host_usr.join("bin"))?;
        fs::create_dir_all(host_usr.join("include"))?;
        fs::create_dir_all(host_usr.join("lib"))?;
        fs::create_dir_all(host_usr.join("share"))?;

        Ok(())
    }

    /// Link an installed package from its distro/<pkg_name> directory into the GNU sysroot and host usr
    pub fn link_package(&self, triple: &TargetTriple, meta: &mut PkgMeta) -> Result<Vec<SymlinkRecord>> {
        self.ensure_gnu_toolchain_dirs(triple)?;

        let platform_str = triple.platform.as_str();
        let pkg_distro = self.config.pkg_distro_dir(platform_str, &triple.raw, &meta.name);

        if !pkg_distro.exists() {
            anyhow::bail!("Package directory does not exist: {}", pkg_distro.display());
        }

        let is_system = meta.target.target_level == "system";
        let triple_root = self.config.triple_dir(platform_str, &triple.raw);
        let triple_usr = self.config.triple_usr_dir(platform_str, &triple.raw);
        let host_usr = self.config.host_usr_dir();
        let host_compat = triple.is_host_compatible();

        let mut symlink_records = Vec::new();

        // 1. Binaries and DLLs
        let bin_src = pkg_distro.join("bin");
        if bin_src.exists() && bin_src.is_dir() {
            let target_bin_dir = if is_system {
                triple_root.join("bin")
            } else {
                triple_usr.join("bin")
            };

            for entry in fs::read_dir(&bin_src)? {
                let entry = entry?;
                let file_name = entry.file_name();
                let file_path = entry.path();
                let dest_file = target_bin_dir.join(&file_name);

                let is_dll = file_path.extension().map_or(false, |ext| ext == "dll");

                match LinkEngine::link_file(&file_path, &dest_file, true)? {
                    LinkStatus::Conflict(msg) => {
                        eprintln!("[WARN] {msg}");
                    }
                    _ => {
                        symlink_records.push(SymlinkRecord {
                            src: format!("bin/{}", file_name.to_string_lossy()),
                            dest: dest_file.to_string_lossy().to_string(),
                            symlink_type: if is_dll { "dll".to_string() } else { "exec".to_string() },
                        });
                    }
                }

                // If host-compatible, also link into $TOOLCHAINS_DIR/usr/bin
                if host_compat {
                    let host_dest_file = host_usr.join("bin").join(&file_name);
                    let _ = LinkEngine::link_file(&file_path, &host_dest_file, false);
                }
            }
        }

        // 2. Headers
        let inc_src = pkg_distro.join("include");
        if inc_src.exists() && inc_src.is_dir() {
            let target_inc_dir = if is_system {
                triple_root.join("include")
            } else {
                triple_usr.join("include")
            };

            for entry in fs::read_dir(&inc_src)? {
                let entry = entry?;
                let file_name = entry.file_name();
                let file_path = entry.path();
                let dest_target = target_inc_dir.join(&file_name);

                if file_path.is_dir() {
                    LinkEngine::link_dir(&file_path, &dest_target)?;
                } else {
                    LinkEngine::link_file(&file_path, &dest_target, true)?;
                }

                symlink_records.push(SymlinkRecord {
                    src: format!("include/{}", file_name.to_string_lossy()),
                    dest: dest_target.to_string_lossy().to_string(),
                    symlink_type: "header".to_string(),
                });
            }
        }

        // 3. Libraries (lib and lib64)
        for lib_name in ["lib", "lib64"] {
            let lib_src = pkg_distro.join(lib_name);
            if lib_src.exists() && lib_src.is_dir() {
                let target_lib_dir = if is_system {
                    triple_root.join(lib_name)
                } else {
                    triple_usr.join(lib_name)
                };

                for entry in fs::read_dir(&lib_src)? {
                    let entry = entry?;
                    let file_name = entry.file_name();
                    let file_path = entry.path();
                    let dest_target = target_lib_dir.join(&file_name);

                    if file_path.is_file() {
                        LinkEngine::link_file(&file_path, &dest_target, true)?;
                        symlink_records.push(SymlinkRecord {
                            src: format!("{lib_name}/{}", file_name.to_string_lossy()),
                            dest: dest_target.to_string_lossy().to_string(),
                            symlink_type: "lib".to_string(),
                        });
                    }
                }
            }
        }

        // 4. Share / pkgconfig / cmake
        let share_src = pkg_distro.join("share");
        if share_src.exists() && share_src.is_dir() {
            let target_share_dir = if is_system {
                triple_root.join("share")
            } else {
                triple_usr.join("share")
            };

            for entry in fs::read_dir(&share_src)? {
                let entry = entry?;
                let file_name = entry.file_name();
                let file_path = entry.path();
                let dest_target = target_share_dir.join(&file_name);

                if file_path.is_dir() {
                    LinkEngine::link_dir(&file_path, &dest_target)?;
                } else {
                    LinkEngine::link_file(&file_path, &dest_target, true)?;
                }

                symlink_records.push(SymlinkRecord {
                    src: format!("share/{}", file_name.to_string_lossy()),
                    dest: dest_target.to_string_lossy().to_string(),
                    symlink_type: "share".to_string(),
                });
            }
        }

        meta.symlinks = symlink_records.clone();
        meta.save_to_file(self.config.pkg_meta_path(platform_str, &triple.raw, &meta.name))?;

        Ok(symlink_records)
    }
}
