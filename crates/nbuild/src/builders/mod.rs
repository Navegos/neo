// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/neo-build/src/builders/mod.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{bail, Context, Result};

pub struct BuildContext {
    pub source_dir: PathBuf,
    pub build_dir: PathBuf,
    pub install_prefix: PathBuf,
    pub target_triple: String,
    pub env_vars: BTreeMap<String, String>,
    pub extra_args: Vec<String>,
}

pub trait BuildEngine: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, project_dir: &Path) -> bool;
    fn configure(&self, ctx: &BuildContext) -> Result<()>;
    fn build(&self, ctx: &BuildContext) -> Result<()>;
    fn install(&self, ctx: &BuildContext) -> Result<()>;
}

/// CMake Builder (Ninja / Make / MSBuild generator)
pub struct CMakeBuilder;

impl BuildEngine for CMakeBuilder {
    fn name(&self) -> &'static str {
        "cmake"
    }

    fn can_handle(&self, dir: &Path) -> bool {
        dir.join("CMakeLists.txt").exists()
    }

    fn configure(&self, ctx: &BuildContext) -> Result<()> {
        let mut cmd = Command::new("cmake");
        cmd.arg("-B").arg(&ctx.build_dir)
           .arg("-S").arg(&ctx.source_dir)
           .arg(format!("-DCMAKE_INSTALL_PREFIX={}", ctx.install_prefix.display()));

        if which::which("ninja").is_ok() {
            cmd.arg("-G").arg("Ninja");
        }

        for (k, v) in &ctx.env_vars {
            cmd.env(k, v);
        }

        let status = cmd.status().context("Executing cmake configure")?;
        if !status.success() {
            bail!("cmake configure failed with exit code: {:?}", status.code());
        }
        Ok(())
    }

    fn build(&self, ctx: &BuildContext) -> Result<()> {
        let mut cmd = Command::new("cmake");
        cmd.arg("--build").arg(&ctx.build_dir).arg("--config").arg("Release");
        let status = cmd.status().context("Executing cmake build")?;
        if !status.success() {
            bail!("cmake build failed with exit code: {:?}", status.code());
        }
        Ok(())
    }

    fn install(&self, ctx: &BuildContext) -> Result<()> {
        let mut cmd = Command::new("cmake");
        cmd.arg("--install").arg(&ctx.build_dir).arg("--config").arg("Release");
        let status = cmd.status().context("Executing cmake install")?;
        if !status.success() {
            bail!("cmake install failed with exit code: {:?}", status.code());
        }
        Ok(())
    }
}

/// Meson Builder
pub struct MesonBuilder;

impl BuildEngine for MesonBuilder {
    fn name(&self) -> &'static str {
        "meson"
    }

    fn can_handle(&self, dir: &Path) -> bool {
        dir.join("meson.build").exists()
    }

    fn configure(&self, ctx: &BuildContext) -> Result<()> {
        let mut cmd = Command::new("meson");
        cmd.arg("setup")
           .arg(&ctx.build_dir)
           .arg(&ctx.source_dir)
           .arg(format!("--prefix={}", ctx.install_prefix.display()));

        let status = cmd.status().context("Executing meson setup")?;
        if !status.success() {
            bail!("meson setup failed with exit code: {:?}", status.code());
        }
        Ok(())
    }

    fn build(&self, ctx: &BuildContext) -> Result<()> {
        let mut cmd = Command::new("meson");
        cmd.arg("compile").arg("-C").arg(&ctx.build_dir);
        let status = cmd.status().context("Executing meson compile")?;
        if !status.success() {
            bail!("meson compile failed with exit code: {:?}", status.code());
        }
        Ok(())
    }

    fn install(&self, ctx: &BuildContext) -> Result<()> {
        let mut cmd = Command::new("meson");
        cmd.arg("install").arg("-C").arg(&ctx.build_dir);
        let status = cmd.status().context("Executing meson install")?;
        if !status.success() {
            bail!("meson install failed with exit code: {:?}", status.code());
        }
        Ok(())
    }
}

/// Ninja Builder
pub struct NinjaBuilder;

impl BuildEngine for NinjaBuilder {
    fn name(&self) -> &'static str {
        "ninja"
    }

    fn can_handle(&self, dir: &Path) -> bool {
        dir.join("build.ninja").exists()
    }

    fn configure(&self, _ctx: &BuildContext) -> Result<()> {
        Ok(())
    }

    fn build(&self, ctx: &BuildContext) -> Result<()> {
        let mut cmd = Command::new("ninja");
        cmd.arg("-C").arg(&ctx.build_dir);
        let status = cmd.status().context("Executing ninja")?;
        if !status.success() {
            bail!("ninja failed with exit code: {:?}", status.code());
        }
        Ok(())
    }

    fn install(&self, ctx: &BuildContext) -> Result<()> {
        let mut cmd = Command::new("ninja");
        cmd.arg("-C").arg(&ctx.build_dir).arg("install");
        let status = cmd.status().context("Executing ninja install")?;
        if !status.success() {
            bail!("ninja install failed with exit code: {:?}", status.code());
        }
        Ok(())
    }
}

/// Cargo Builder
pub struct CargoBuilder;

impl BuildEngine for CargoBuilder {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn can_handle(&self, dir: &Path) -> bool {
        dir.join("Cargo.toml").exists()
    }

    fn configure(&self, _ctx: &BuildContext) -> Result<()> {
        Ok(())
    }

    fn build(&self, ctx: &BuildContext) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&ctx.source_dir)
           .arg("build")
           .arg("--release")
           .arg("--target")
           .arg(&ctx.target_triple);
        let status = cmd.status().context("Executing cargo build")?;
        if !status.success() {
            bail!("cargo build failed with exit code: {:?}", status.code());
        }
        Ok(())
    }

    fn install(&self, ctx: &BuildContext) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&ctx.source_dir)
           .arg("install")
           .arg("--path")
           .arg(".")
           .arg("--root")
           .arg(&ctx.install_prefix);
        let status = cmd.status().context("Executing cargo install")?;
        if !status.success() {
            bail!("cargo install failed with exit code: {:?}", status.code());
        }
        Ok(())
    }
}

/// Buck2 Builder (adaptable for distributed builds)
pub struct Buck2Builder;

impl BuildEngine for Buck2Builder {
    fn name(&self) -> &'static str {
        "buck2"
    }

    fn can_handle(&self, dir: &Path) -> bool {
        dir.join(".buckconfig").exists() || dir.join("BUCK").exists()
    }

    fn configure(&self, _ctx: &BuildContext) -> Result<()> {
        Ok(())
    }

    fn build(&self, ctx: &BuildContext) -> Result<()> {
        let mut cmd = Command::new("buck2");
        cmd.current_dir(&ctx.source_dir).arg("build").arg("//...");
        let status = cmd.status().context("Executing buck2 build")?;
        if !status.success() {
            bail!("buck2 build failed with exit code: {:?}", status.code());
        }
        Ok(())
    }

    fn install(&self, _ctx: &BuildContext) -> Result<()> {
        Ok(())
    }
}

/// Get appropriate build engine for a given source directory or builder name
pub fn detect_builder(builder_name: Option<&str>, source_dir: &Path) -> Option<Box<dyn BuildEngine>> {
    if let Some(name) = builder_name {
        match name.to_lowercase().as_str() {
            "cmake" => return Some(Box::new(CMakeBuilder)),
            "meson" => return Some(Box::new(MesonBuilder)),
            "ninja" => return Some(Box::new(NinjaBuilder)),
            "cargo" => return Some(Box::new(CargoBuilder)),
            "buck2" => return Some(Box::new(Buck2Builder)),
            _ => {}
        }
    }

    if CMakeBuilder.can_handle(source_dir) {
        Some(Box::new(CMakeBuilder))
    } else if MesonBuilder.can_handle(source_dir) {
        Some(Box::new(MesonBuilder))
    } else if CargoBuilder.can_handle(source_dir) {
        Some(Box::new(CargoBuilder))
    } else if Buck2Builder.can_handle(source_dir) {
        Some(Box::new(Buck2Builder))
    } else {
        None
    }
}

