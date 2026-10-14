// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/neo-build/src/cli.rs
// created: 2026-10-14
// lastModified: 2026-10-14

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

            let recipe = cat.targets.get(&target.raw)
                .or_else(|| cat.targets.get(target.platform.as_str()))
                .with_context(|| format!("No target recipe for '{}' on target '{}'", package, target.raw))?;

            let installer = DistroInstaller::new(&config);
            let mut meta = installer.install_prebuilt(
                &target,
                &cat.name,
                &cat.version,
                &cat.category,
                &cat.target_level,
                &recipe.source,
            )?;

            let symlinks = sysroot.link_package(&target, &mut meta)?;

            let distro_path = config.pkg_distro_dir(target.platform.as_str(), &target.raw, &cat.name);

            if cli.json {
                AgentReporter::success(
                    InstallReport {
                        package: cat.name.clone(),
                        version: cat.version.clone(),
                        triple: target.raw.clone(),
                        distro_path: distro_path.to_string_lossy().to_string(),
                        symlinks_count: symlinks.len(),
                        env_exports: meta.env_exports,
                    },
                    true,
                );
            } else {
                println!("[nbuild] Installed {} {} into {}", cat.name, cat.version, distro_path.display());
                println!("[nbuild] Generated {} symlinks into GNU sysroot.", symlinks.len());
            }
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

