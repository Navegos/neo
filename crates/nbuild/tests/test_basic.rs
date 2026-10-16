// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/tests/test_basic.rs
// created: 2026-10-14
// lastModified: 2026-10-15

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
    registry.add_search_path("crates/nbuild/catalog");
    let _ = registry.load_all();

    if let Some(ninja) = registry.get("ninja") {
        assert_eq!(ninja.name, "ninja");
        assert_eq!(ninja.category, "tool");
        assert_eq!(ninja.get_remote_repo(), Some("ninja-build/ninja"));
        assert!(ninja.get_releases_url().is_some());
    }
}

#[test]
fn test_github_release_matching() {
    use nbuild::core::forge::{GitHubAsset, GitHubRelease};

    let rel = GitHubRelease {
        tag_name: "v3.30.5".to_string(),
        name: Some("CMake 3.30.5".to_string()),
        published_at: Some("2024-10-01T00:00:00Z".to_string()),
        target_commitish: Some("master".to_string()),
        assets: vec![
            GitHubAsset {
                name: "cmake-3.30.5-windows-x86_64.zip".to_string(),
                browser_download_url: "https://example.com/cmake.zip".to_string(),
                size: 45000000,
                content_type: None,
            },
            GitHubAsset {
                name: "cmake-3.30.5-linux-x86_64.tar.gz".to_string(),
                browser_download_url: "https://example.com/cmake.tar.gz".to_string(),
                size: 48000000,
                content_type: None,
            },
        ],
    };

    assert_eq!(rel.clean_version(), "3.30.5");

    let win_asset = rel.find_asset("windows", "x86_64").expect("Windows asset should match");
    assert_eq!(win_asset.name, "cmake-3.30.5-windows-x86_64.zip");

    let linux_asset = rel.find_asset("linux", "x86_64").expect("Linux asset should match");
    assert_eq!(linux_asset.name, "cmake-3.30.5-linux-x86_64.tar.gz");
}

#[test]
fn test_forge_detection_and_validation() {
    use nbuild::core::forge::{ForgeClient, ForgeType};

    assert_eq!(ForgeType::detect("Kitware/CMake"), ForgeType::GitHub);
    assert_eq!(ForgeType::detect("https://github.com/Kitware/CMake.git"), ForgeType::GitHub);
    assert_eq!(ForgeType::detect("https://gitlab.com/libeigen/eigen.git"), ForgeType::GitLab);
    assert_eq!(ForgeType::detect("gitlab:libeigen/eigen"), ForgeType::GitLab);
    assert_eq!(ForgeType::detect("https://codeberg.org/forgejo/forgejo.git"), ForgeType::Codeberg);
    assert_eq!(ForgeType::detect("https://bitbucket.org/workspace/repo.git"), ForgeType::Bitbucket);
    assert_eq!(ForgeType::detect("https://launchpad.net/bzr"), ForgeType::Launchpad);
    assert_eq!(ForgeType::detect("lp:bzr"), ForgeType::Launchpad);
    assert_eq!(ForgeType::detect("https://git.kernel.org/pub/scm/linux.git"), ForgeType::GenericGit);
    assert_eq!(ForgeType::detect("git@custom.internal:team/repo.git"), ForgeType::GenericGit);

    assert!(ForgeClient::validate_url("https://github.com/Kitware/CMake").is_ok());
    assert!(ForgeClient::validate_url("git@gitlab.com:user/repo.git").is_ok());
    assert!(ForgeClient::validate_url("ssh://git@custom.com/repo.git").is_ok());

    assert!(ForgeClient::validate_url("file:///etc/passwd").is_err());
    assert!(ForgeClient::validate_url("ftp://internal.vault/secret").is_err());
}

#[test]
fn test_compression_format_ranking_and_stem_matching() {
    use nbuild::core::forge::{strip_archive_extension, CompressionFormat, GitHubAsset, GitHubRelease};

    assert_eq!(strip_archive_extension("cmake-3.30.5-windows-x86_64.zip"), "cmake-3.30.5-windows-x86_64");
    assert_eq!(strip_archive_extension("ninja-linux.tar.gz"), "ninja-linux");
    assert_eq!(strip_archive_extension("llvm-project.tar.xz"), "llvm-project");
    assert_eq!(strip_archive_extension("package.tar.zst"), "package");
    assert_eq!(strip_archive_extension("data.tar.bz2"), "data");
    assert_eq!(strip_archive_extension("bundle.7z"), "bundle");
    assert_eq!(strip_archive_extension("archive.rar"), "archive");

    assert_eq!(CompressionFormat::from_filename("tool.tar.zst"), CompressionFormat::TarZst);
    assert_eq!(CompressionFormat::from_filename("tool.tar.xz"), CompressionFormat::TarXz);
    assert_eq!(CompressionFormat::from_filename("tool.zip"), CompressionFormat::Zip);
    assert_eq!(CompressionFormat::from_filename("tool.7z"), CompressionFormat::SevenZip);
    assert_eq!(CompressionFormat::from_filename("tool.tar.gz"), CompressionFormat::TarGz);
    assert_eq!(CompressionFormat::from_filename("tool.tar.bz2"), CompressionFormat::TarBz2);
    assert_eq!(CompressionFormat::from_filename("tool.rar"), CompressionFormat::Rar);

    // Multi-compression release test:
    let rel = GitHubRelease {
        tag_name: "v3.31.0".to_string(),
        name: Some("CMake 3.31.0".to_string()),
        published_at: None,
        target_commitish: None,
        assets: vec![
            GitHubAsset {
                name: "cmake-3.31.0-windows-x86_64.tar.gz".to_string(),
                browser_download_url: "https://example.com/win.tar.gz".to_string(),
                size: 40000000,
                content_type: None,
            },
            GitHubAsset {
                name: "cmake-3.31.0-windows-x86_64.zip".to_string(),
                browser_download_url: "https://example.com/win.zip".to_string(),
                size: 35000000,
                content_type: None,
            },
            GitHubAsset {
                name: "cmake-3.31.0-windows-x86_64.zip.sha256".to_string(),
                browser_download_url: "https://example.com/win.zip.sha256".to_string(),
                size: 64,
                content_type: None,
            },
            GitHubAsset {
                name: "cmake-3.31.0-linux-x86_64.tar.gz".to_string(),
                browser_download_url: "https://example.com/linux.tar.gz".to_string(),
                size: 45000000,
                content_type: None,
            },
            GitHubAsset {
                name: "cmake-3.31.0-linux-x86_64.tar.zst".to_string(),
                browser_download_url: "https://example.com/linux.tar.zst".to_string(),
                size: 30000000,
                content_type: None,
            },
        ],
    };

    // On Windows, .zip has higher score than .tar.gz (and .sha256 is ignored)
    let win_hints = vec!["windows-x86_64".to_string()];
    let win_match = rel.find_asset_with_hints("windows", "x86_64", Some(&win_hints)).unwrap();
    assert_eq!(win_match.name, "cmake-3.31.0-windows-x86_64.zip");

    // On Linux, .tar.zst has higher score than .tar.gz
    let linux_hints = vec!["linux-x86_64".to_string()];
    let linux_match = rel.find_asset_with_hints("linux", "x86_64", Some(&linux_hints)).unwrap();
    assert_eq!(linux_match.name, "cmake-3.31.0-linux-x86_64.tar.zst");

    // Even if old hint had .tar.gz, stem matching matches .tar.zst when it's better:
    let old_linux_hints = vec!["cmake-3.31.0-linux-x86_64.tar.gz".to_string()];
    let linux_match2 = rel.find_asset_with_hints("linux", "x86_64", Some(&old_linux_hints)).unwrap();
    assert!(linux_match2.name.contains("linux-x86_64"));
}

#[test]
fn test_neo_catalog_loading() {
    let mut reg = CatalogRegistry::new();
    reg.add_search_path("catalog");
    reg.add_search_path("crates/nbuild/catalog");
    reg.load_all().unwrap();

    let neo = reg.get("neo").expect("neo package catalog should be loaded");
    assert_eq!(neo.name, "neo");
    assert_eq!(
        neo.get_releases_url(),
        Some("https://api.github.com/repos/Navegos/neo/releases/latest")
    );
    let hints = neo.get_asset_hints("windows", "x86_64");
    assert!(hints.is_some());
    assert!(hints.unwrap().contains(&"neo-x86_64-pc-windows-msvc".to_string()));
}

#[test]
fn test_fs_ops_lifecycle() {
    use nbuild::core::{FsFilter, FsOps};
    let test_dir = std::env::temp_dir().join("nbuild_test_fs_ops");
    let _ = FsOps::delete(&test_dir, true, true);

    // 1. Mkdir
    FsOps::mkdir(&test_dir).unwrap();
    assert!(test_dir.exists());

    // 2. Touch
    let subfile = test_dir.join("sample.txt");
    FsOps::touch(&subfile).unwrap();
    assert!(subfile.exists());

    // 3. Search
    let results = FsOps::search(&test_dir, "*.txt", FsFilter::File).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], subfile);

    // 4. Copy
    let copied = test_dir.join("sample_copied.txt");
    FsOps::copy(&subfile, &copied, false).unwrap();
    assert!(copied.exists());

    // 5. Rename
    let renamed = FsOps::rename(&copied, "sample_renamed.txt").unwrap();
    assert!(renamed.exists());
    assert!(!copied.exists());

    // 6. Move
    let subfolder = test_dir.join("nested");
    FsOps::mkdir(&subfolder).unwrap();
    let moved_dest = subfolder.join("sample_renamed.txt");
    FsOps::move_path(&renamed, &moved_dest).unwrap();
    assert!(moved_dest.exists());
    assert!(!renamed.exists());

    // 7. Delete
    FsOps::delete(&test_dir, true, true).unwrap();
    assert!(!test_dir.exists());
}

#[test]
fn test_doctor_topology_check() {
    use nbuild::core::{Doctor, NeoConfig, TargetTriple};
    let host = TargetTriple::detect_host();
    let cfg = NeoConfig::new().with_toolchains_dir(r"C:\fake\toolchains");
    let (is_canonical, _current, canonical_bin) = Doctor::check_topology(&cfg, &host);
    assert!(!is_canonical);
    assert!(canonical_bin.to_string_lossy().contains("distro"));
    assert!(canonical_bin.to_string_lossy().contains("neo"));
}



