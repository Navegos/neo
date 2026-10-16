// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/lib.rs
// created: 2026-10-14
// lastModified: 2026-10-15

pub mod agent;
pub mod builders;
pub mod cli;
pub mod core;
pub mod pipeline;

pub use core::{
    CatalogRegistry, DistroInstaller, ForgeClient, ForgeType, GitHubClient, GitHubRelease,
    LinkEngine, NeoConfig, PackageCatalog, PkgMeta, PlatformGroup, RemoteAsset, RemoteRelease,
    SysrootManager, TargetTriple, ToolLifecycleStrategy,
};
pub use pipeline::Workflow;

