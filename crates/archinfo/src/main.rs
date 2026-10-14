// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/main.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use archinfo::{
    get_default_output_dir, Arch, ArchFeaturesReport, CpuArchitectureVectorLength,
    Platform, PlatformArchMatrixReport,
};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Target operating system platform (native, windows, linux, mac, ios, android, ps5, switch, xboxxs, etc.)
    #[arg(short = 'p', long)]
    platform: Option<String>,

    /// Target hardware architecture (native, x86_64, arm64, riscv64)
    #[arg(short = 'a', long)]
    arch: Option<String>,

    /// Target microarchitecture (native, generic, znver3, alderlake, apple-m4, cortex-a78c, etc.)
    #[arg(short = 't', long = "target", visible_alias = "target-cpu", alias = "target_cpu")]
    target: Option<String>,

    /// Target tune CPU microarchitecture (generic, znver3, alderlake, apple-m4, cortex-a78c, etc.)
    #[arg(short = 'u', long = "target-tune", visible_alias = "target-tune-cpu", alias = "tune-cpu", alias = "tune")]
    target_tune_cpu: Option<String>,

    /// Minimum CPU architecture baseline (AVX, AVX2, AVX512, AVX10.1, AVX10.2 for x86_64; ARMv8-A..ARMv9.7-A for arm64)
    #[arg(short = 'm', long = "min-cpu-arch", visible_alias = "min-arch", alias = "minimum-cpu-architecture")]
    min_cpu_arch: Option<String>,

    /// Enabled ISA extensions to include (Generic target only, e.g. "aes+sha2" or "avx512f,avx512vl")
    #[arg(short = 'e', long = "enable-extensions", visible_alias = "enabled-extensions", alias = "enable-ext")]
    enable_extensions: Option<String>,

    /// Disabled ISA extensions to exclude (Generic target only, e.g. "sse4.1" or "sve+sve2")
    #[arg(short = 'd', long = "disable-extensions", visible_alias = "disabled-extensions", alias = "disable-ext")]
    disable_extensions: Option<String>,

    /// Preferred vector length for SIMD code generation (128, 256, 512, vl128, vl256, vl512)
    #[arg(short = 'v', long = "vector-length", visible_alias = "vl", alias = "vector")]
    vector_length: Option<String>,

    /// Target GPU architecture family (native, cuda, rocm, xpu, metal). Accepts multi-values (e.g. cuda;rocm).
    #[arg(short = 'G', long = "gpu-arch", visible_alias = "gpu_arch", alias = "gpu", value_delimiter = ';', num_args = 1..)]
    gpu_arch: Option<Vec<String>>,

    /// Target CUDA GPU architecture(s) (75, 80, 86, 87, 89, 90, 100, 103, 110, 120, 121). Accepts multi-values (e.g. 75;86;89;120). Default: 75.
    #[arg(short = 'C', long = "cuda-arch", visible_alias = "cuda_arch", alias = "cuda", alias = "cu", value_delimiter = ';', num_args = 1..)]
    cuda_arch: Option<Vec<String>>,

    /// Target ROCm GPU architecture(s) (908, 90a, 940, 941, 942, 950, 1030, 1031, 1032, 1034, 1035, 1036, 1100, 1101, 1102, 1103, 1150, 1151, 1152, 1153, 1170, 1171, 1172, 1200, 1201). Accepts multi-values (e.g. 908;90a;950;1030). Default: 1030.
    #[arg(short = 'R', long = "rocm-arch", visible_alias = "rocm_arch", alias = "rocm", alias = "hip", alias = "ro", value_delimiter = ';', num_args = 1..)]
    rocm_arch: Option<Vec<String>>,

    /// Target Intel XPU architecture (bmg, lnl, ptl). Accepts multi-values. Default: bmg.
    #[arg(short = 'X', long = "xpu-arch", visible_alias = "xpu_arch", alias = "xpu", value_delimiter = ';', num_args = 1..)]
    xpu_arch: Option<Vec<String>>,

    /// Target Apple Metal GPU family (apple2..apple10). Accepts multi-values (e.g. apple7;apple8;apple9;apple10).
    #[arg(long = "metal-family", visible_alias = "metal_family", alias = "metal-gpu-family", alias = "metal_gpu_family", value_delimiter = ';', num_args = 1..)]
    metal_family: Option<Vec<String>>,

    /// Target Apple Metal version (metal, metal3, metal4).
    #[arg(long = "metal-version", visible_alias = "metal_version", alias = "metal")]
    metal_version: Option<String>,

    /// Path to output JSON file or directory. If omitted, uses standard Unreal Engine/ArchInfo directory.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Save result using canonical filename in default or selected output folder
    #[arg(short, long)]
    save: bool,

    /// Generates full compatibility matrix across all platforms and architectures
    #[arg(short = 'M', long)]
    matrix: bool,

    /// Force host CPU detection
    #[arg(short = 'D', long)]
    detect: bool,

    /// Target operating system level (e.g. Android NDK API level 24..30, FreeBSD OS level 13..15 / 13.0..15.3 / 13.0.0..15.3.9, macOS Sequoia 15 / 15.0..15.99 / 15.0.0..15.99.99, Tahoe 26 / 26.0..26.99 / 26.0.0..26.99.99, Golden Gate 27 / 27.0..27.99 / 27.0.0..27.99.99, iOS/tvOS/xrOS 26 / 26.0..26.99 / 26.0.0..26.99.99, 27 / 27.0..27.99 / 27.0.0..27.99.99)
    #[arg(short = 'l', long = "target-os-level", visible_alias = "os-level", alias = "target_os_level")]
    target_os_level: Option<String>,

    /// Target runtime level (e.g. Linux glibc 2.17..2.44 / 2.17.0..2.44.9, Windows MSVC 1930..1952 / 19.30..19.52.99999)
    #[arg(short = 'r', long = "target-runtime-level", visible_alias = "runtime-level", alias = "target_runtime_level")]
    target_runtime_level: Option<String>,

    /// Target is simulator flag for iOS/tvOS/xrOS (yes, true, 1)
    #[arg(short = 'i', long = "target-is-simulator", visible_alias = "is-simulator", alias = "target_is_simulator", alias = "simulator", num_args = 0..=1, default_missing_value = "true")]
    target_is_simulator: Option<String>,

    /// Format to print to console (json, extensions, clang, msvc, filename, dir, triple)
    #[arg(short, long, default_value = "json")]
    format: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let requested_vl = match cli.vector_length.as_deref() {
        Some(s) => Some(s.parse::<CpuArchitectureVectorLength>()?),
        None => None,
    };

    if cli.matrix {
        let matrix_report = PlatformArchMatrixReport::generate();
        let json_str = matrix_report.to_json()?;

        if cli.save {
            let out_dir = cli.output.unwrap_or_else(get_default_output_dir);
            let saved = matrix_report.save_all_to_dir(&out_dir)?;
            println!("Saved {} architecture profiles to {}", saved.len(), out_dir.display());
        } else if let Some(ref path) = cli.output {
            if path.is_dir() || path.to_string_lossy().ends_with('/') || path.to_string_lossy().ends_with('\\') {
                let saved = matrix_report.save_all_to_dir(path)?;
                println!("Saved {} architecture profiles to {}", saved.len(), path.display());
            } else {
                matrix_report.save_to_file(path)?;
                println!("Wrote platform-architecture matrix to {}", path.display());
            }
        } else {
            println!("{}", json_str);
        }
        return Ok(());
    }

    let platform = match cli.platform.as_deref() {
        Some(p) => Some(p.parse::<Platform>()?),
        None => None,
    };

    let gpu_arch_arg = cli.gpu_arch.as_ref().map(|v| v.join(";"));
    let cuda_arch_arg = cli.cuda_arch.as_ref().map(|v| v.join(";"));
    let rocm_arch_arg = cli.rocm_arch.as_ref().map(|v| v.join(";"));
    let xpu_arch_arg = cli.xpu_arch.as_ref().map(|v| v.join(";"));
    let metal_family_arg = cli.metal_family.as_ref().map(|v| v.join(";"));
    let metal_version_arg = cli.metal_version.as_deref();

    let arch = match cli.arch.as_deref() {
        Some(a) => Some(a.parse::<Arch>()?),
        None => None,
    };

    let target_opt = if cli.detect {
        Some("native")
    } else {
        cli.target.as_deref()
    };

    let target_is_simulator = match cli.target_is_simulator.as_deref() {
        Some(s) => match s.trim().to_ascii_lowercase().as_str() {
            "yes" | "true" | "1" => true,
            _ => false,
        },
        None => false,
    };

    // When no arguments are passed, evaluate defaults to native host platform, host arch, and live CPUFeatures probing
    let report = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        platform,
        arch,
        target_opt,
        cli.target_tune_cpu.as_deref(),
        cli.min_cpu_arch.as_deref(),
        cli.enable_extensions.as_deref(),
        cli.disable_extensions.as_deref(),
        requested_vl,
        cli.target_os_level.as_deref(),
        cli.target_runtime_level.as_deref(),
        target_is_simulator,
        gpu_arch_arg.as_deref(),
        cuda_arch_arg.as_deref(),
        rocm_arch_arg.as_deref(),
        xpu_arch_arg.as_deref(),
        metal_family_arg.as_deref(),
        metal_version_arg,
    )?;

    // Console output based on format
    match cli.format.to_lowercase().as_str() {
        "dir" | "folder" => {
            println!("{}", get_default_output_dir().display());
        }
        "filepath" | "path" => {
            println!("{}", report.default_filepath().display());
        }
        "filename" | "name" => {
            println!("{}", report.filename());
        }
        "extensions" | "ext" => {
            println!("{}", report.extensions);
        }
        "triple" | "target" => {
            if let Some(ref triple) = report.target_clang_triple {
                println!("{}", triple);
            }
        }
        "clang" => {
            let mut flags = Vec::new();
            if let Some(ref cpu) = report.target_cpu {
                flags.push(format!("-mcpu={}", cpu));
            }
            if !report.extensions.is_empty() {
                for ext in report.extensions.split('+') {
                    flags.push(format!("-target-feature +{}", ext));
                }
            }
            if let Some(ref vlen) = report.target_clang_vlen {
                if !vlen.is_empty() {
                    flags.push(vlen.clone());
                }
            }
            if let Some(ref cuda_sm) = report.cuda_sm_arch {
                if !cuda_sm.is_empty() {
                    for sm in cuda_sm.split(';') {
                        flags.push(format!("--cuda-gpu-arch={}", sm));
                    }
                }
            }
            if let Some(ref rocm_gfx) = report.rocm_gfx_arch {
                if !rocm_gfx.is_empty() {
                    for gfx in rocm_gfx.split(';') {
                        flags.push(format!("--offload-arch={}", gfx));
                    }
                }
            }
            println!("{}", flags.join(" "));
        }
        "msvc" => {
            let platform = report.platform.parse::<Platform>().unwrap_or(Platform::Native).resolve();
            let mut flags = Vec::new();
            if platform.uses_msvc() {
                if let Some(ref target_msvc) = report.target_msvc_arch {
                    if !target_msvc.is_empty() {
                        flags.push(target_msvc.clone());
                    }
                } else if let Some(ref min_arch) = report.min_cpu_arch {
                    if let Ok(arch_enum) = min_arch.parse::<archinfo::MinimumCpuArchitectureX64>() {
                        let msvc_name = archinfo::MSVCX64ISANames::name(arch_enum);
                        flags.push(format!("/arch:{}", msvc_name));
                    } else {
                        flags.push(format!("/arch:{}", min_arch.to_uppercase()));
                    }
                } else {
                    flags.push("/arch:AVX2".to_string());
                }
                if let Some(ref vlen) = report.target_msvc_vlen {
                    if !vlen.is_empty() {
                        flags.push(vlen.clone());
                    }
                }
            }
            println!("{}", flags.join(" "));
        }
        _ => {
            let json_str = report.to_json()?;
            println!("{}", json_str);
        }
    }

    // Determine destination filename / path
    let out_path = if let Some(ref p) = cli.output {
        if p.is_dir() || p.to_string_lossy().ends_with('/') || p.to_string_lossy().ends_with('\\') {
            Some(p.join(report.filename()))
        } else {
            Some(p.clone())
        }
    } else if cli.save {
        Some(report.default_filepath())
    } else {
        None
    };

    if let Some(path) = out_path {
        report.save_to_file(&path)?;
        println!("Wrote report to {}", path.display());
    }

    Ok(())
}
