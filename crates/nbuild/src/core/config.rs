// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/core/config.rs
// created: 2026-10-14
// lastModified: 2026-10-15

use std::env;
use std::path::{Path, PathBuf};

/// Default toolchain directory for Windows and Unix
pub const DEFAULT_WINDOWS_TOOLCHAINS_DIR: &str = r"C:\opt\neo-toolchains";
pub const ALT_WINDOWS_TOOLCHAINS_DIR: &str = r"C:\opt\toolchains";

pub const DEFAULT_UNIX_TOOLCHAINS_DIR: &str = "/opt/neo-toolchains";
pub const ALT_UNIX_TOOLCHAINS_DIR: &str = "/opt/toolchains";

#[derive(Debug, Clone)]
pub struct NeoConfig {
    pub toolchains_dir: PathBuf,
    pub catalog_extra_paths: Vec<PathBuf>,
}

impl Default for NeoConfig {
    fn default() -> Self {
        Self {
            toolchains_dir: Self::detect_default_toolchains_dir(),
            catalog_extra_paths: Vec::new(),
        }
    }
}

impl NeoConfig {
    pub fn new() -> Self {
        let mut cfg = Self::default();

        // 1. Check TOOLCHAINS_DIR environment variable
        if let Ok(val) = env::var("TOOLCHAINS_DIR") {
            if !val.trim().is_empty() {
                cfg.toolchains_dir = PathBuf::from(val.trim());
            }
        }

        cfg
    }

    pub fn with_toolchains_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.toolchains_dir = dir.as_ref().to_path_buf();
        self
    }

    /// Detects default toolchains root according to OS
    pub fn detect_default_toolchains_dir() -> PathBuf {
        if cfg!(windows) {
            let primary = Path::new(DEFAULT_WINDOWS_TOOLCHAINS_DIR);
            let alt = Path::new(ALT_WINDOWS_TOOLCHAINS_DIR);
            if !primary.exists() && alt.exists() {
                alt.to_path_buf()
            } else {
                primary.to_path_buf()
            }
        } else {
            let primary = Path::new(DEFAULT_UNIX_TOOLCHAINS_DIR);
            let alt = Path::new(ALT_UNIX_TOOLCHAINS_DIR);
            if !primary.exists() && alt.exists() {
                alt.to_path_buf()
            } else {
                primary.to_path_buf()
            }
        }
    }

    /// Returns $TOOLCHAINS_DIR/usr (Host active prefix)
    pub fn host_usr_dir(&self) -> PathBuf {
        self.toolchains_dir.join("usr")
    }

    /// Returns $TOOLCHAINS_DIR/usr/bin (Added to Host PATH)
    pub fn host_usr_bin(&self) -> PathBuf {
        self.host_usr_dir().join("bin")
    }

    /// Returns $TOOLCHAINS_DIR/<platform>
    pub fn platform_dir(&self, platform: &str) -> PathBuf {
        self.toolchains_dir.join(platform)
    }

    /// Returns $TOOLCHAINS_DIR/<platform>/<target-triple>
    pub fn triple_dir(&self, platform: &str, triple: &str) -> PathBuf {
        self.platform_dir(platform).join(triple)
    }

    /// Returns $TOOLCHAINS_DIR/<platform>/<target-triple>/distro
    pub fn distro_dir(&self, platform: &str, triple: &str) -> PathBuf {
        self.triple_dir(platform, triple).join("distro")
    }

    /// Returns $TOOLCHAINS_DIR/<platform>/<target-triple>/distro/<pkg_name>
    /// Note: package folder is purely the package name (e.g. "cmake", "libpng", "zlib"), not versioned.
    pub fn pkg_distro_dir(&self, platform: &str, triple: &str, pkg_name: &str) -> PathBuf {
        self.distro_dir(platform, triple).join(pkg_name)
    }

    /// Returns $TOOLCHAINS_DIR/<platform>/<target-triple>/usr (GNU sysroot user layer)
    pub fn triple_usr_dir(&self, platform: &str, triple: &str) -> PathBuf {
        self.triple_dir(platform, triple).join("usr")
    }

    /// Returns the pkg_meta.json file path for a package
    pub fn pkg_meta_path(&self, platform: &str, triple: &str, pkg_name: &str) -> PathBuf {
        self.pkg_distro_dir(platform, triple, pkg_name).join("pkg_meta.json")
    }
}

