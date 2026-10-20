// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/cli.rs
// created: 2026-10-14
// lastModified: 2026-10-16

use std::path::PathBuf;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::agent::{AgentReporter, InstallReport};
use crate::core::{
    CatalogRegistry, DistroInstaller, Doctor, EnvManager, FsFilter, FsOps, NeoConfig,
    SysrootManager, TargetTriple,
};
use crate::pipeline::Workflow;

#[derive(Parser, Debug)]
#[command(name = "nbuild", author, version, about = "Universal toolchain manager, local workflow runner, and package engine for AI agents")]
pub struct Cli {
    #[arg(long, global = true, help = "Path to toolchains root directory (overrides TOOLCHAINS_DIR)")]
    pub toolchains_dir: Option<PathBuf>,

    #[arg(long, global = true, help = "Output machine-readable JSON for agents")]
    pub json: bool,

    #[arg(long, global = true, help = "Terse output; suppress verbose progress")]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Initialize toolchain root directory and GNU directory structure")]
    Init {
        #[arg(long, help = "Explicit directory path for toolchains root")]
        dir: Option<PathBuf>,
    },

    #[command(about = "Download and install prebuilt tool or SDK into distro/ and link into sysroot")]
    Get {
        #[arg(help = "Name of the package/tool to install (e.g. cmake, ninja, llvm)")]
        package: String,

        #[arg(long, help = "Target triple (defaults to host triple)")]
        triple: Option<String>,
    },

    #[command(about = "Check local tool installation vs latest remote GitHub release")]
    Check {
        #[arg(help = "Name of the package/tool to check (e.g. cmake, ninja)")]
        package: String,

        #[arg(long, help = "Target triple (defaults to host triple)")]
        triple: Option<String>,
    },

    #[command(about = "Update tool to latest release (prebuilt or self-build from source)")]
    Update {
        #[arg(help = "Name of the package/tool to update (e.g. cmake, ninja)")]
        package: String,

        #[arg(long, help = "Target triple (defaults to host triple)")]
        triple: Option<String>,

        #[arg(long, help = "Force reinstall/rebuild even if already up to date")]
        force: bool,
    },

    #[command(about = "Clone git source repository for package or from custom git address")]
    Clone {
        #[arg(help = "Package name from catalog or full git URL")]
        source: String,

        #[arg(long, help = "Git branch, tag or commit")]
        tag: Option<String>,

        #[arg(long, help = "Target destination directory")]
        dest: Option<PathBuf>,
    },

    #[command(about = "Build a package or recipe from source")]
    Build {
        #[arg(help = "Package name or recipe target")]
        package: Option<String>,

        #[arg(short, long, help = "Path to custom YAML recipe/workflow file")]
        file: Option<PathBuf>,

        #[arg(long, help = "Target triple (defaults to host triple)")]
        triple: Option<String>,
    },

    #[command(about = "List installed packages or available catalog recipes")]
    List {
        #[arg(long, help = "Show only installed packages")]
        installed: bool,

        #[arg(long, help = "Show only available catalog recipes")]
        available: bool,

        #[arg(long, help = "Target triple to inspect")]
        triple: Option<String>,
    },

    #[command(about = "Export toolchain environment variables")]
    Env {
        #[arg(long, help = "Target triple to export environment for")]
        triple: Option<String>,

        #[arg(long, default_value = "pwsh", help = "Shell format: pwsh | bash | json")]
        shell: String,
    },

    #[command(about = "Rebuild and repair symlinks for target triple sysroot and host usr")]
    Link {
        #[arg(long, help = "Target triple to re-link")]
        triple: Option<String>,
    },

    #[command(about = "Run local GitHub Actions-compatible workflow pipeline")]
    Run {
        #[arg(help = "Path to workflow YAML file")]
        workflow_file: PathBuf,
    },

    #[command(about = "Self-diagnosis and self-healing: verify structure, repair links, heal packages, run smoke tests")]
    Doctor {
        #[arg(long, help = "Automatically heal broken links, packages, and environment without prompting")]
        heal: bool,

        #[arg(long, help = "Run brief smoke test to verify functionality")]
        brief_test: bool,

        #[arg(long, help = "Target triple (defaults to host triple)")]
        triple: Option<String>,
    },

    #[command(about = "Check and configure TOOLCHAINS_DIR and user PATH environment variables")]
    EnvSetup {
        #[arg(long, help = "Explicit directory path for TOOLCHAINS_DIR")]
        dir: Option<PathBuf>,

        #[arg(long, help = "Run non-interactively using default path")]
        non_interactive: bool,
    },

    #[command(about = "Built-in file and directory operations (search, copy, move, rename, delete, mkdir, touch)")]
    Fs {
        #[command(subcommand)]
        op: FsCommands,
    },

    #[command(about = "Interactive CLI menu for toolchain status, self-healing, packages, and tools")]
    Menu,
}

#[derive(Subcommand, Debug)]
pub enum FsCommands {
    #[command(about = "Search for files and directories matching pattern")]
    Search {
        #[arg(help = "Search pattern or substring (e.g. *.dll, lib*, cmake)")]
        pattern: String,

        #[arg(short, long, help = "Root directory to search (defaults to current dir)")]
        dir: Option<PathBuf>,

        #[arg(short, long, help = "Filter type: all | file | dir")]
        filter: Option<String>,
    },

    #[command(about = "Copy file or directory")]
    Copy {
        #[arg(help = "Source path")]
        src: PathBuf,

        #[arg(help = "Destination path")]
        dest: PathBuf,

        #[arg(short, long, help = "Recursive copy for directories")]
        recursive: bool,
    },

    #[command(about = "Move file or directory")]
    Move {
        #[arg(help = "Source path")]
        src: PathBuf,

        #[arg(help = "Destination path")]
        dest: PathBuf,
    },

    #[command(about = "Rename file or directory")]
    Rename {
        #[arg(help = "Source path")]
        src: PathBuf,

        #[arg(help = "New file or directory name")]
        new_name: String,
    },

    #[command(about = "Delete file or directory")]
    Delete {
        #[arg(help = "Target path to delete")]
        path: PathBuf,

        #[arg(short, long, help = "Recursive deletion for directories")]
        recursive: bool,

        #[arg(short, long, help = "Force deletion")]
        force: bool,
    },

    #[command(about = "Create directory recursively (mkdir -p)")]
    Mkdir {
        #[arg(help = "Directory path to create")]
        dir: PathBuf,
    },

    #[command(about = "Create empty file or touch existing file")]
    Touch {
        #[arg(help = "File path to touch")]
        file: PathBuf,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    let mut config = NeoConfig::new();
    if let Some(dir) = cli.toolchains_dir {
        config = config.with_toolchains_dir(dir);
    }

    let host_triple = TargetTriple::detect_host();

    let (is_canonical, _current_exe, canonical_bin) = Doctor::check_topology(&config, &host_triple);
    if !is_canonical && !cli.quiet && !matches!(cli.command, Some(Commands::Doctor { .. })) {
        eprintln!("[WARN] nbuild is not running from canonical toolchain directory structure.");
        eprintln!("[WARN] Expected: {}", canonical_bin.display());
        eprintln!("[WARN] Run 'nbuild doctor --heal' to install into distro/neo and heal environment.");
    }

    let command = match cli.command {
        Some(cmd) => cmd,
        None => Commands::Menu,
    };

    match command {
        Commands::Menu => {
            run_interactive_menu(config)?;
        }

        Commands::Init { dir } => {
            if let Some(d) = dir {
                config = config.with_toolchains_dir(d);
            }
            let host_triple = TargetTriple::detect_host();
            let sysroot = SysrootManager::new(&config);
            sysroot.ensure_gnu_toolchain_dirs(&host_triple)?;

            println!("[nbuild] Toolchains directory initialized at: {}", config.toolchains_dir.display());
            println!("[nbuild] Host active prefix: {}", config.host_usr_dir().display());
            println!("[nbuild] Host target triple: {}", host_triple.raw);
        }

        Commands::Get { package, triple } => {
            let package = if package == "nbuild" { "neo".to_string() } else { package };
            let target = match triple {
                Some(t) => TargetTriple::parse(&t)?,
                None => TargetTriple::detect_host(),
            };

            let sysroot = SysrootManager::new(&config);
            sysroot.ensure_gnu_toolchain_dirs(&target)?;

            let registry = init_catalog_registry(&config);

            let cat = registry.get(&package)
                .with_context(|| format!("Package '{}' not found in catalog", package))?;

            let distro_path = config.pkg_distro_dir(target.platform.as_str(), &target.raw, &cat.name);
            let meta_file = config.pkg_meta_path(target.platform.as_str(), &target.raw, &cat.name);

            // Fast idempotency: check if already installed
            if meta_file.exists() {
                if let Ok(existing_meta) = crate::core::meta::PkgMeta::load_from_file(&meta_file) {
                    if !cli.quiet {
                        println!("[nbuild] {} {} is already installed at {}", cat.name, existing_meta.version, distro_path.display());
                    }
                    return Ok(());
                }
            }

            if cat.is_vcpkg() {
                let vcpkg = crate::core::VcpkgManager::new();
                let mut meta = vcpkg.install_package(&config, &target, cat)?;
                let symlinks = sysroot.link_package(&target, &mut meta)?;
                if !cli.quiet {
                    println!("[nbuild] Successfully provisioned {} v{} from vcpkg. Created {} symlinks.", cat.name, meta.version, symlinks.len());
                }
                if cli.json {
                    AgentReporter::success(
                        InstallReport {
                            package: cat.name.clone(),
                            version: meta.version.clone(),
                            triple: target.raw.clone(),
                            distro_path: distro_path.to_string_lossy().to_string(),
                            symlinks_count: symlinks.len(),
                            env_exports: meta.env_exports,
                        },
                        true,
                    );
                }
                return Ok(());
            }

            let recipe_opt = cat.targets.get(&target.raw)
                .or_else(|| cat.targets.get(target.platform.as_str()));

            let mut source_to_use = if let Some(rec) = recipe_opt {
                rec.source.clone()
            } else {
                crate::core::catalog::SourceSpec {
                    source_type: "api_release".to_string(),
                    url: String::new(),
                    sha256: None,
                    tag: None,
                    commit: None,
                    strip_prefix: None,
                }
            };

            let mut version_to_use = cat.version.clone();
            let hints = cat.get_asset_hints(target.platform.as_str(), &target.arch).map(|v| v.as_slice());

            // 1. Dynamic Release Resolution from API URL (GitHub, GitLab, Codeberg, Bitbucket)
            let api_url_opt = cat.get_releases_url().or_else(|| {
                if source_to_use.url.contains("/releases") {
                    Some(source_to_use.url.as_str())
                } else {
                    None
                }
            });

            if let Some(api_url) = api_url_opt {
                if let Ok(forge) = crate::core::forge::ForgeClient::new() {
                    if let Ok(latest_rel) = forge.fetch_release_by_url(api_url) {
                        if let Some(asset) = latest_rel.find_asset_with_hints(target.platform.as_str(), &target.arch, hints) {
                            if !cli.quiet {
                                println!("[nbuild] Resolved platform asset from releases API: {} (v{})", asset.name, latest_rel.clean_version());
                            }
                            source_to_use.url = asset.browser_download_url.clone();
                            source_to_use.sha256 = None; // Dynamically resolved release
                            version_to_use = latest_rel.clean_version();
                        } else {
                            anyhow::bail!("No matching asset for platform {} / arch {} found in releases API: {}", target.platform.as_str(), target.arch, api_url);
                        }
                    }
                }
            } else if let Some(repo) = cat.get_remote_repo() {
                if let Ok(forge) = crate::core::forge::ForgeClient::new() {
                    if let Ok(latest_rel) = forge.fetch_latest_release(repo) {
                        if let Some(asset) = latest_rel.find_asset_with_hints(target.platform.as_str(), &target.arch, hints) {
                            if !cli.quiet {
                                println!("[nbuild] Resolved latest release for {}: v{}", repo, latest_rel.clean_version());
                            }
                            source_to_use.url = asset.browser_download_url.clone();
                            source_to_use.sha256 = None;
                            version_to_use = latest_rel.clean_version();
                        }
                    }
                }
            }

            if source_to_use.url.is_empty() {
                anyhow::bail!("No download source or releases API URL resolved for package '{}' on target '{}'", package, target.raw);
            }

            let installer = DistroInstaller::new(&config);
            let mut meta = installer.install_prebuilt(
                &target,
                &cat.name,
                &version_to_use,
                &cat.category,
                &cat.target_level,
                &source_to_use,
            )?;

            // Apply environment exports from catalog if not empty
            if !cat.env_exports.is_empty() && meta.env_exports.is_empty() {
                meta.env_exports = cat.env_exports.clone();
            }

            let symlinks = sysroot.link_package(&target, &mut meta)?;

            if cli.json {
                AgentReporter::success(
                    InstallReport {
                        package: cat.name.clone(),
                        version: version_to_use,
                        triple: target.raw.clone(),
                        distro_path: distro_path.to_string_lossy().to_string(),
                        symlinks_count: symlinks.len(),
                        env_exports: meta.env_exports,
                    },
                    true,
                );
            } else {
                println!("[nbuild] Installed {} {} into {}", cat.name, meta.version, distro_path.display());
                println!("[nbuild] Generated {} symlinks into GNU sysroot.", symlinks.len());
            }
        }

        Commands::Check { package, triple } => {
            let package = if package == "nbuild" { "neo".to_string() } else { package };
            let target = match triple {
                Some(t) => TargetTriple::parse(&t)?,
                None => TargetTriple::detect_host(),
            };

            let registry = init_catalog_registry(&config);
            let cat = registry.get(&package)
                .with_context(|| format!("Package '{}' not found in catalog", package))?;

            let meta_file = config.pkg_meta_path(target.platform.as_str(), &target.raw, &cat.name);
            let local_version = if meta_file.exists() {
                crate::core::meta::PkgMeta::load_from_file(&meta_file).ok().map(|m| m.version)
            } else {
                None
            };

            let remote_version = if let Some(api_url) = cat.get_releases_url() {
                crate::core::forge::ForgeClient::new().ok()
                    .and_then(|forge| forge.fetch_release_by_url(api_url).ok())
                    .map(|r| r.clean_version())
            } else if let Some(repo) = cat.get_remote_repo() {
                crate::core::forge::ForgeClient::new().ok()
                    .and_then(|forge| forge.fetch_latest_release(repo).ok())
                    .map(|r| r.clean_version())
            } else {
                Some(cat.version.clone())
            };

            if cli.json {
                let mut map = std::collections::BTreeMap::new();
                map.insert("package", cat.name.clone());
                map.insert("installed", local_version.is_some().to_string());
                map.insert("local_version", local_version.unwrap_or_else(|| "none".to_string()));
                map.insert("latest_remote_version", remote_version.unwrap_or_else(|| "unknown".to_string()));
                println!("{}", serde_json::to_string_pretty(&map)?);
            } else {
                println!("=== Status: {} ({}) ===", cat.name, target.raw);
                match &local_version {
                    Some(v) => println!("  Installed: v{}", v),
                    None => println!("  Installed: false (not found in toolchain)"),
                }
                if let Some(rv) = &remote_version {
                    println!("  Latest Remote Release: v{}", rv);
                    if let Some(lv) = &local_version {
                        if lv == rv {
                            println!("  State: Up to date");
                        } else {
                            println!("  State: Outdated (can update or rebuild from source)");
                        }
                    } else {
                        println!("  State: Missing (can install prebuilt release)");
                    }
                }
            }
        }

        Commands::Update { package, triple, force } => {
            let package = if package == "nbuild" { "neo".to_string() } else { package };
            let target = match triple {
                Some(t) => TargetTriple::parse(&t)?,
                None => TargetTriple::detect_host(),
            };

            let registry = init_catalog_registry(&config);
            let cat = registry.get(&package)
                .with_context(|| format!("Package '{}' not found in catalog", package))?;

            let meta_file = config.pkg_meta_path(target.platform.as_str(), &target.raw, &cat.name);
            let local_version = if meta_file.exists() {
                crate::core::meta::PkgMeta::load_from_file(&meta_file).ok().map(|m| m.version)
            } else {
                None
            };

            let remote_release = if let Some(api_url) = cat.get_releases_url() {
                crate::core::forge::ForgeClient::new()?.fetch_release_by_url(api_url).ok()
            } else if let Some(repo) = cat.get_remote_repo() {
                crate::core::forge::ForgeClient::new()?.fetch_latest_release(repo).ok()
            } else {
                None
            };

            let remote_version = remote_release.as_ref().map(|r| r.clean_version()).unwrap_or_else(|| cat.version.clone());

            let needs_update = force || local_version.as_deref() != Some(&remote_version);

            if !needs_update && local_version.is_some() {
                println!("[nbuild] {} is already up to date (v{}). Use --force to reinstall.", cat.name, remote_version);
                return Ok(());
            }

            println!("[nbuild] Updating {} from {:?} to v{}...", cat.name, local_version, remote_version);

            let sysroot = SysrootManager::new(&config);
            sysroot.ensure_gnu_toolchain_dirs(&target)?;

            if cat.is_vcpkg() {
                let vcpkg = crate::core::VcpkgManager::new();
                let mut meta = vcpkg.install_package(&config, &target, cat)?;
                let symlinks = sysroot.link_package(&target, &mut meta)?;
                println!("[nbuild] Successfully updated {} to v{}. Generated {} symlinks.", cat.name, meta.version, symlinks.len());
                return Ok(());
            }

            let recipe_opt = cat.targets.get(&target.raw)
                .or_else(|| cat.targets.get(target.platform.as_str()));

            let mut source_to_use = if let Some(rec) = recipe_opt {
                rec.source.clone()
            } else {
                crate::core::catalog::SourceSpec {
                    source_type: "api_release".to_string(),
                    url: String::new(),
                    sha256: None,
                    tag: None,
                    commit: None,
                    strip_prefix: None,
                }
            };

            let hints = cat.get_asset_hints(target.platform.as_str(), &target.arch).map(|v| v.as_slice());

            if let Some(release) = &remote_release {
                if let Some(asset) = release.find_asset_with_hints(target.platform.as_str(), &target.arch, hints) {
                    source_to_use.url = asset.browser_download_url.clone();
                    source_to_use.sha256 = None;
                }
            }

            let installer = DistroInstaller::new(&config);
            let mut meta = installer.install_prebuilt(
                &target,
                &cat.name,
                &remote_version,
                &cat.category,
                &cat.target_level,
                &source_to_use,
            )?;

            let symlinks = sysroot.link_package(&target, &mut meta)?;
            println!("[nbuild] Successfully updated {} to v{}. Generated {} symlinks.", cat.name, remote_version, symlinks.len());
        }

        Commands::Clone { source, tag, dest } => {
            let registry = init_catalog_registry(&config);
            let repo_url = if let Some(cat) = registry.get(&source) {
                cat.git_url.as_deref().unwrap_or(&source)
            } else {
                &source
            };

            let target_dest = dest.unwrap_or_else(|| {
                let pkg_name = repo_url.trim_end_matches(".git").split('/').last().unwrap_or("repo");
                PathBuf::from(pkg_name)
            });

            println!("[nbuild] Cloning {} into {}...", repo_url, target_dest.display());
            crate::core::forge::ForgeClient::git_clone(repo_url, tag.as_deref(), &target_dest)?;
            println!("[nbuild] Successfully cloned into {}.", target_dest.display());
        }

        Commands::Build { package, file, triple: _ } => {
            if let Some(workflow_path) = file {
                let wf = Workflow::load_from_file(&workflow_path)?;
                wf.run_local(&config, cli.quiet)?;
            } else if let Some(pkg) = package {
                println!("[nbuild] Building package: {}", pkg);
            } else {
                eprintln!("[ERROR] Specify package name or -f <workflow.yaml>");
            }
        }

        Commands::List { installed: _, available: _, triple: _ } => {
            let registry = init_catalog_registry(&config);

            println!("=== Available Catalog Packages ===");
            for cat in registry.list() {
                println!("- {:<15} v{:<10} [{}] {}", cat.name, cat.version, cat.category, cat.description.as_deref().unwrap_or(""));
            }
        }

        Commands::Env { triple, shell } => {
            let target = match triple {
                Some(t) => TargetTriple::parse(&t)?,
                None => TargetTriple::detect_host(),
            };

            let host_bin = config.host_usr_bin();
            let triple_usr = config.triple_usr_dir(target.platform.as_str(), &target.raw);

            match shell.to_lowercase().as_str() {
                "json" => {
                    let mut map = std::collections::BTreeMap::new();
                    map.insert("TOOLCHAINS_DIR", config.toolchains_dir.to_string_lossy().to_string());
                    map.insert("PATH_PREPEND", host_bin.to_string_lossy().to_string());
                    map.insert("SYSROOT", config.triple_dir(target.platform.as_str(), &target.raw).to_string_lossy().to_string());
                    map.insert("CMAKE_PREFIX_PATH", triple_usr.to_string_lossy().to_string());
                    println!("{}", serde_json::to_string_pretty(&map)?);
                }
                "bash" => {
                    println!("export TOOLCHAINS_DIR=\"{}\"", config.toolchains_dir.display());
                    println!("export PATH=\"{}:$PATH\"", host_bin.display());
                    println!("export CMAKE_PREFIX_PATH=\"{}:$CMAKE_PREFIX_PATH\"", triple_usr.display());
                }
                _ => {
                    // Default pwsh
                    println!("$env:TOOLCHAINS_DIR = \"{}\"", config.toolchains_dir.display());
                    println!("$env:PATH = \"{};\" + $env:PATH", host_bin.display());
                    println!("$env:CMAKE_PREFIX_PATH = \"{}\"", triple_usr.display());
                }
            }
        }

        Commands::Link { triple } => {
            let target = match triple {
                Some(t) => TargetTriple::parse(&t)?,
                None => TargetTriple::detect_host(),
            };
            let sysroot = SysrootManager::new(&config);
            sysroot.ensure_gnu_toolchain_dirs(&target)?;
            println!("[nbuild] Rebuilt GNU toolchain directories for {}", target.raw);
        }

        Commands::Run { workflow_file } => {
            let wf = Workflow::load_from_file(&workflow_file)?;
            wf.run_local(&config, cli.quiet)?;
        }

        Commands::Doctor { heal, brief_test, triple } => {
            let target = match triple {
                Some(t) => TargetTriple::parse(&t)?,
                None => TargetTriple::detect_host(),
            };
            println!("=== Neo System Doctor ===");
            let report = Doctor::run_doctor(&mut config, &target, heal)?;
            println!("  Directory Structure: {}", report.topology_msg);
            println!("  Broken Links:        Found {}, Repaired {}", report.broken_links_found, report.broken_links_repaired);
            println!("  Package Integrity:   Found {} corrupted, Repaired {}", report.broken_packages_found, report.broken_packages_repaired);
            println!("  TOOLCHAINS_DIR Env:  {}", if report.env_toolchains_ok { "Configured" } else { "MISSING" });
            println!("  Host PATH:           {}", if report.env_path_ok { "Configured ($TOOLCHAINS_DIR/usr/bin)" } else { "MISSING" });
            if brief_test || report.smoke_test_ok {
                println!("  Smoke Test:          {}", if report.smoke_test_ok { "PASSED (OK to use)" } else { "FAILED" });
            }
        }

        Commands::EnvSetup { dir, non_interactive } => {
            if let Some(d) = dir {
                println!("[nbuild] Setting TOOLCHAINS_DIR to: {}", d.display());
                EnvManager::persist_toolchains_dir(&d)?;
                let cfg = NeoConfig::new().with_toolchains_dir(d);
                let healed = EnvManager::heal_path_env(&cfg)?;
                if healed {
                    println!("[nbuild] Added {} to user PATH.", cfg.host_usr_bin().display());
                }
            } else {
                let _ = EnvManager::ensure_environment(!non_interactive)?;
            }
        }

        Commands::Fs { op } => match op {
            FsCommands::Search { pattern, dir, filter } => {
                let search_dir = dir.unwrap_or_else(|| PathBuf::from("."));
                let f = match filter.as_deref() {
                    Some("file") => FsFilter::File,
                    Some("dir") => FsFilter::Dir,
                    _ => FsFilter::All,
                };
                let matches = FsOps::search(&search_dir, &pattern, f)?;
                if cli.json {
                    let list: Vec<String> = matches.iter().map(|p| p.to_string_lossy().to_string()).collect();
                    println!("{}", serde_json::to_string_pretty(&list)?);
                } else {
                    println!("[nbuild fs] Found {} items matching '{}':", matches.len(), pattern);
                    for m in matches {
                        println!("  {}", m.display());
                    }
                }
            }
            FsCommands::Copy { src, dest, recursive } => {
                FsOps::copy(&src, &dest, recursive)?;
                println!("[nbuild fs] Copied {} -> {}", src.display(), dest.display());
            }
            FsCommands::Move { src, dest } => {
                FsOps::move_path(&src, &dest)?;
                println!("[nbuild fs] Moved {} -> {}", src.display(), dest.display());
            }
            FsCommands::Rename { src, new_name } => {
                let dest = FsOps::rename(&src, &new_name)?;
                println!("[nbuild fs] Renamed {} -> {}", src.display(), dest.display());
            }
            FsCommands::Delete { path, recursive, force } => {
                FsOps::delete(&path, recursive, force)?;
                println!("[nbuild fs] Deleted {}", path.display());
            }
            FsCommands::Mkdir { dir } => {
                FsOps::mkdir(&dir)?;
                println!("[nbuild fs] Created directory {}", dir.display());
            }
            FsCommands::Touch { file } => {
                FsOps::touch(&file)?;
                println!("[nbuild fs] Created/touched {}", file.display());
            }
        },
    }

    Ok(())
}

fn run_interactive_menu(mut config: NeoConfig) -> Result<()> {
    use std::io::{stdin, stdout, Write};
    let host_triple = TargetTriple::detect_host();

    loop {
        let (is_canonical, _current, _canonical_bin) = Doctor::check_topology(&config, &host_triple);
        let has_toolchains = EnvManager::get_toolchains_dir_env().is_some();
        let in_path = EnvManager::is_usr_bin_in_path(&config);

        println!("\n========================================================");
        println!("       neo - Toolchain Manager & Agent Proxy (nbuild)   ");
        println!("========================================================");
        println!("  TOOLCHAINS_DIR: {} [{}]", config.toolchains_dir.display(), if has_toolchains { "OK" } else { "NOT SET" });
        println!("  Host Usr Bin:   {} [{}]", config.host_usr_bin().display(), if in_path { "IN PATH" } else { "MISSING FROM PATH" });
        println!("  Host Triple:    {}", host_triple.raw);
        println!("  Topology:       {}", if is_canonical { "Canonical (distro/neo/bin)" } else { "External (Not in distro/neo/bin)" });
        println!("--------------------------------------------------------");
        println!("  1. System Doctor & Self-Heal (repair links, packages, test)");
        println!("  2. Environment Setup (configure TOOLCHAINS_DIR & PATH)");
        println!("  3. Install / Update Neo (nbuild, tokless, companion tools)");
        println!("  4. Manage Packages (cmake, ninja, llvm, zlib, libpng, ...)");
        println!("  5. File Operations (search, copy, move, rename, delete, mkdir)");
        println!("  6. Run Workflow Pipeline");
        println!("  0. Exit");
        println!("--------------------------------------------------------");
        print!("  Select an option [0-6]: ");
        let _ = stdout().flush();

        let mut input = String::new();
        if stdin().read_line(&mut input).is_err() {
            break;
        }

        match input.trim() {
            "1" => {
                println!("\n[Doctor] Running diagnostic & self-healing suite...");
                let report = Doctor::run_doctor(&mut config, &host_triple, true)?;
                println!("  Topology:            {}", report.topology_msg);
                println!("  Broken links healed: {}/{}", report.broken_links_repaired, report.broken_links_found);
                println!("  Packages repaired:   {}/{}", report.broken_packages_repaired, report.broken_packages_found);
                println!("  Environment:         TOOLCHAINS_DIR: {}, PATH: {}", report.env_toolchains_ok, report.env_path_ok);
                println!("  Smoke test:          {}", if report.smoke_test_ok { "PASSED (OK to use)" } else { "FAILED" });
            }
            "2" => {
                println!("\n[Environment] Configuring TOOLCHAINS_DIR and PATH...");
                match EnvManager::ensure_environment(true) {
                    Ok(new_cfg) => {
                        config = new_cfg;
                        println!("[Environment] Configuration updated successfully.");
                    }
                    Err(e) => eprintln!("[Environment] Error: {e}"),
                }
            }
            "3" => {
                println!("\n[Neo] Installing / Updating Neo package from release...");
                let registry = init_catalog_registry(&config);
                if let Some(cat) = registry.get("neo") {
                    let installer = DistroInstaller::new(&config);
                    let sysroot = SysrootManager::new(&config);
                    let _ = sysroot.ensure_gnu_toolchain_dirs(&host_triple);
                    let distro_path = config.pkg_distro_dir(host_triple.platform.as_str(), &host_triple.raw, "neo");
                    if let Some(releases_url) = cat.get_releases_url() {
                        if let Ok(forge) = crate::core::forge::ForgeClient::new() {
                            if let Ok(release) = forge.fetch_release_by_url(releases_url) {
                                println!("Latest Neo release: v{}", release.clean_version());
                                let hints = cat.get_asset_hints(host_triple.platform.as_str(), &host_triple.arch).map(|v| v.as_slice());
                                if let Some(asset) = release.find_asset_with_hints(host_triple.platform.as_str(), &host_triple.arch, hints) {
                                    let source = crate::core::catalog::SourceSpec {
                                        source_type: "api_release".to_string(),
                                        url: asset.browser_download_url.clone(),
                                        sha256: None,
                                        tag: Some(release.tag_name.clone()),
                                        commit: None,
                                        strip_prefix: None,
                                    };
                                    if let Ok(mut meta) = installer.install_prebuilt(&host_triple, "neo", &release.clean_version(), &cat.category, &cat.target_level, &source) {
                                        let _ = sysroot.link_package(&host_triple, &mut meta);
                                        println!("[Neo] Updated successfully to {}", distro_path.display());
                                    }
                                } else {
                                    println!("[Neo] No matching release asset found for platform. Self-installing local binary...");
                                    let _ = Doctor::self_install(&config, &host_triple);
                                }
                            } else {
                                println!("[Neo] Forge query failed. Self-installing local binary...");
                                let _ = Doctor::self_install(&config, &host_triple);
                            }
                        }
                    } else {
                        let _ = Doctor::self_install(&config, &host_triple);
                    }
                }
            }
            "4" => {
                let registry = init_catalog_registry(&config);
                println!("\n[Catalog Packages]");
                for cat in registry.list() {
                    let installed = config.pkg_meta_path(host_triple.platform.as_str(), &host_triple.raw, &cat.name).exists();
                    println!("  - {:<10} {:<10} [{}]", cat.name, cat.category, if installed { "INSTALLED" } else { "AVAILABLE" });
                }
            }
            "5" => {
                println!("\n[File Operations]");
                println!("  1. Search");
                println!("  2. Create Directory (mkdir)");
                println!("  3. Back");
                print!("  Choice: ");
                let _ = stdout().flush();
                let mut f_in = String::new();
                let _ = stdin().read_line(&mut f_in);
                match f_in.trim() {
                    "1" => {
                        print!("  Pattern to search: ");
                        let _ = stdout().flush();
                        let mut pat = String::new();
                        let _ = stdin().read_line(&mut pat);
                        let results = FsOps::search(".", pat.trim(), FsFilter::All).unwrap_or_default();
                        println!("  Found {} matches:", results.len());
                        for r in results.iter().take(20) {
                            println!("    {}", r.display());
                        }
                    }
                    "2" => {
                        print!("  Directory path to create: ");
                        let _ = stdout().flush();
                        let mut dir_in = String::new();
                        let _ = stdin().read_line(&mut dir_in);
                        if let Err(e) = FsOps::mkdir(dir_in.trim()) {
                            eprintln!("  Error: {e}");
                        } else {
                            println!("  Directory created.");
                        }
                    }
                    _ => {}
                }
            }
            "6" => {
                print!("  Enter workflow YAML file path: ");
                let _ = stdout().flush();
                let mut wf_path = String::new();
                let _ = stdin().read_line(&mut wf_path);
                let p = PathBuf::from(wf_path.trim());
                if p.exists() {
                    match Workflow::load_from_file(&p) {
                        Ok(wf) => {
                            let _ = wf.run_local(&config, false);
                        }
                        Err(e) => eprintln!("  Workflow load error: {e}"),
                    }
                } else {
                    println!("  File does not exist: {}", p.display());
                }
            }
            "0" | "q" | "exit" => break,
            _ => println!("  Invalid selection. Please choose 0-6."),
        }
    }

    Ok(())
}

fn init_catalog_registry(config: &NeoConfig) -> CatalogRegistry {
    let mut registry = CatalogRegistry::new();
    registry.add_search_path("catalog");
    registry.add_search_path("crates/nbuild/catalog");
    registry.add_search_path(config.toolchains_dir.join("catalog"));
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            registry.add_search_path(parent.join("catalog"));
        }
    }
    for p in &config.catalog_extra_paths {
        registry.add_search_path(p);
    }
    let _ = registry.load_all();
    registry
}

