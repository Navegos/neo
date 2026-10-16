// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/core/mod.rs
// created: 2026-10-14
// lastModified: 2026-10-15

pub mod arch;
pub mod catalog;
pub mod config;
pub mod distro;
pub mod forge;
pub mod meta;
pub mod symlink;
pub mod sysroot;

pub use arch::{PlatformGroup, TargetTriple};
pub use catalog::{CatalogRegistry, PackageCatalog, PlatformRecipe};
pub use config::NeoConfig;
pub use distro::DistroInstaller;
pub use forge::{ForgeClient, ForgeType, GitHubAsset, GitHubClient, GitHubRelease, RemoteAsset, RemoteRelease, ToolLifecycleStrategy};
pub use meta::{PkgMeta, TargetInfo};
pub use symlink::{LinkEngine, LinkStatus};
pub use sysroot::SysrootManager;

