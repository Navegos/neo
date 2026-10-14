// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/neo-build/tests/test_basic.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use std::path::Path;
use nbuild::core::{CatalogRegistry, NeoConfig, PkgMeta, PlatformGroup, TargetInfo, TargetTriple};

#[test]
fn test_target_triple_parsing() {
    let host = TargetTriple::detect_host();
    assert!(!host.raw.is_empty());
    assert!(host.is_host_compatible());

    let win = TargetTriple::parse("x86_64-pc-windows-msvc").unwrap();
    assert_eq!(win.platform, PlatformGroup::Windows);
    assert_eq!(win.arch, "x86_64");

    let xros = TargetTriple::parse("aarch64-apple-xros").unwrap();
    assert_eq!(xros.platform, PlatformGroup::Apple);
    assert_eq!(xros.arch, "aarch64");
    assert_eq!(xros.os, "xros");

    let linux = TargetTriple::parse("x86_64-unknown-linux-gnu").unwrap();
    assert_eq!(linux.platform, PlatformGroup::Linux);
    assert_eq!(linux.arch, "x86_64");
}

#[test]
fn test_config_paths() {
    let cfg = NeoConfig::new().with_toolchains_dir(r"C:\opt\neo-toolchains");
    assert_eq!(cfg.host_usr_dir(), Path::new(r"C:\opt\neo-toolchains\usr"));
    assert_eq!(cfg.host_usr_bin(), Path::new(r"C:\opt\neo-toolchains\usr\bin"));

    // Verify clean folder structure without version in folder name
    let distro_dir = cfg.pkg_distro_dir("windows", "x86_64-pc-windows-msvc", "cmake");
    assert_eq!(
        distro_dir,
        Path::new(r"C:\opt\neo-toolchains\windows\x86_64-pc-windows-msvc\distro\cmake")
    );

    let meta_file = cfg.pkg_meta_path("windows", "x86_64-pc-windows-msvc", "cmake");
    assert_eq!(
        meta_file,
        Path::new(r"C:\opt\neo-toolchains\windows\x86_64-pc-windows-msvc\distro\cmake\pkg_meta.json")
    );
}

#[test]
fn test_pkg_meta_serialization() {
    let target = TargetInfo {
        platform: "windows".to_string(),
        triple: "x86_64-pc-windows-msvc".to_string(),
        arch: "x86_64".to_string(),
        target_level: "user".to_string(),
    };

    let mut meta = PkgMeta::new("libpng", "1.8.0", "source_build", target);
    meta.abiversion = Some("18".to_string());
    meta.soversion = Some("18".to_string());
    meta.tag_name = Some("libpng18".to_string());
    meta.commit = Some("9ec49c2d56cec19107ddc458b648ce224c9697b3".to_string());
    meta.url = Some("https://github.com/pnggroup/libpng.git".to_string());

    let json_str = serde_json::to_string_pretty(&meta).unwrap();
    assert!(json_str.contains("\"rawversion\": \"1.8.0\""));
    assert!(json_str.contains("\"abiversion\": \"18\""));
    assert!(json_str.contains("\"tag_name\": \"libpng18\""));

    let loaded: PkgMeta = serde_json::from_str(&json_str).unwrap();
    assert_eq!(loaded.name, "libpng");
    assert_eq!(loaded.version, "1.8.0");
    assert_eq!(loaded.abiversion.as_deref(), Some("18"));
    assert_eq!(loaded.soversion.as_deref(), Some("18"));
}

#[test]
fn test_catalog_loading() {
    let mut registry = CatalogRegistry::new();
    registry.add_search_path("catalog");
    let _ = registry.load_all();

    if let Some(ninja) = registry.get("ninja") {
        assert_eq!(ninja.name, "ninja");
        assert_eq!(ninja.category, "tool");
    }
}

