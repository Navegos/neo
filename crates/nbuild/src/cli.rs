// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/cli.rs
// created: 2026-10-14
// lastModified: 2026-10-15

use std::path::PathBuf;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use crate::agent::{AgentReporter, InstallReport};
use crate::core::{
    CatalogRegistry, DistroInstaller, NeoConfig, SysrootManager, TargetTriple,
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
    pub command: Commands,
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
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    let mut config = NeoConfig::new();
    if let Some(dir) = cli.toolchains_dir {
        config = config.with_toolchains_dir(dir);
    }

    match cli.command {
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

