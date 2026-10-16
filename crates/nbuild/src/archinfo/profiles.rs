// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/profiles.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use crate::archinfo::arch::arm64::{self, Arm64CPUFeatures, TargetCpuArchitectureArm64};
use crate::archinfo::arch::x86_64::{self, TargetCpuArchitectureX64, X64CPUFeatures};
use crate::archinfo::platform::{Arch, Platform};

pub struct TargetProfile;

impl TargetProfile {
    /// Returns default CPU features for a given platform and architecture
    pub fn get_features(platform: Platform, arch: Arch) -> Result<(String, TargetCpuArchitectureX64, TargetCpuArchitectureArm64), String> {
        let platform = platform.resolve();
        let arch = arch.resolve();
        if !platform.is_arch_compatible(arch) {
            return Err(format!(
                "Architecture {} is incompatible with platform {}",
                arch, platform
            ));
        }

        match (platform, arch) {
            // Nintendo Switch 2 (Tegra T239 - Cortex-A78C)
            (Platform::Switch2, Arch::Arm64) => {
                let target = TargetCpuArchitectureArm64::Switch2;
                let ext = arm64::ClangTargetCpuArchitectureArm64ISANames::name(target).to_string();
                Ok((ext, TargetCpuArchitectureX64::None, target))
            }

            // Steam Deck (AMD Zen 2 APU)
            (Platform::Steamdeck, Arch::X86_64) => {
                let target = TargetCpuArchitectureX64::Steamdeck;
                let ext = x86_64::ClangTargetCpuArchitectureX64ISANames::name(target).to_string();
                Ok((ext, target, TargetCpuArchitectureArm64::None))
            }

            // Steam Machine (Generic X86_64_v3 / AMD Zen 4 baseline)
            (Platform::Steammachine, Arch::X86_64) => {
                let target = TargetCpuArchitectureX64::Steammachine;
                let ext = x86_64::ClangTargetCpuArchitectureX64ISANames::name(target).to_string();
                Ok((ext, target, TargetCpuArchitectureArm64::None))
            }

            // PS4 (AMD Jaguar / btver2)
            (Platform::Ps4, Arch::X86_64) => {
                let target = TargetCpuArchitectureX64::Ps4;
                let ext = x86_64::ClangTargetCpuArchitectureX64ISANames::name(target).to_string();
                Ok((ext, target, TargetCpuArchitectureArm64::None))
            }

            // PS5 (AMD Zen 2 / znver2 custom Oberon APU - deleted FP3 and stripped FP2 pipes, default vl128)
            (Platform::Ps5, Arch::X86_64) => {
                let target = TargetCpuArchitectureX64::Ps5;
                let ext = x86_64::ClangTargetCpuArchitectureX64ISANames::name(target).to_string();
                Ok((ext, target, TargetCpuArchitectureArm64::None))
            }

            // Xbox One (AMD Jaguar / btver2)
            (Platform::Xboxone, Arch::X86_64) => {
                let target = TargetCpuArchitectureX64::Xboxone;
                let ext = x86_64::ClangTargetCpuArchitectureX64ISANames::name(target).to_string();
                Ok((ext, target, TargetCpuArchitectureArm64::None))
            }

            // Xbox Series X/S (AMD Zen 2 / znver2)
            (Platform::Xboxxs, Arch::X86_64) => {
                let target = TargetCpuArchitectureX64::Xboxxs;
                let ext = x86_64::ClangTargetCpuArchitectureX64ISANames::name(target).to_string();
                Ok((ext, target, TargetCpuArchitectureArm64::None))
            }

            // Apple iOS / tvOS / xrOS (Apple A14 / A15 / A16 / M2 baseline)
            (Platform::Ios | Platform::Tvos | Platform::Xros, Arch::Arm64)
            | (Platform::Ios, Arch::Arm64E) => {
                let target = TargetCpuArchitectureArm64::Apple_A15;
                let ext = arm64::ClangTargetCpuArchitectureArm64ISANames::name(target).to_string();
                Ok((ext, TargetCpuArchitectureX64::None, target))
            }

            // macOS Apple Silicon
            (Platform::Macosx, Arch::Arm64 | Arch::Arm64E) => {
                let target = TargetCpuArchitectureArm64::Apple_M1;
                let ext = arm64::ClangTargetCpuArchitectureArm64ISANames::name(target).to_string();
                Ok((ext, TargetCpuArchitectureX64::None, target))
            }

            // macOS x86_64
            (Platform::Macosx, Arch::X86_64) => {
                let target = TargetCpuArchitectureX64::Haswell;
                let ext = x86_64::ClangTargetCpuArchitectureX64ISANames::name(target).to_string();
                Ok((ext, target, TargetCpuArchitectureArm64::None))
            }

            // Windows x86_64
            (Platform::Windows, Arch::X86_64) => {
                let target = TargetCpuArchitectureX64::Generic;
                let ext = x86_64::ClangTargetCpuArchitectureX64ISANames::name(target).to_string();
                Ok((ext, target, TargetCpuArchitectureArm64::None))
            }

            // Windows Arm64 / Arm64EC
            (Platform::Windows, Arch::Arm64 | Arch::Arm64EC) => {
                let target = TargetCpuArchitectureArm64::Cortex_A78;
                let ext = arm64::ClangTargetCpuArchitectureArm64ISANames::name(target).to_string();
                Ok((ext, TargetCpuArchitectureX64::None, target))
            }

            // Linux x86_64
            (Platform::Linux | Platform::Freebsd, Arch::X86_64) => {
                let target = TargetCpuArchitectureX64::Generic;
                let ext = x86_64::ClangTargetCpuArchitectureX64ISANames::name(target).to_string();
                Ok((ext, target, TargetCpuArchitectureArm64::None))
            }

            // Linux Arm64
            (Platform::Linux | Platform::Freebsd | Platform::Android, Arch::Arm64) => {
                let target = TargetCpuArchitectureArm64::Generic;
                let ext = arm64::ClangTargetCpuArchitectureArm64ISANames::name(target).to_string();
                Ok((ext, TargetCpuArchitectureX64::None, target))
            }

            // Linux / Android RISC-V 64
            (Platform::Linux | Platform::Freebsd | Platform::Android, Arch::Riscv64) => {
                let ext = "i+m+a+f+d+c".to_string();
                Ok((ext, TargetCpuArchitectureX64::None, TargetCpuArchitectureArm64::None))
            }

            _ => {
                let ext = match arch {
                    Arch::X86_64 => x86_64::ClangTargetCpuArchitectureX64ISANames::name(TargetCpuArchitectureX64::Generic).to_string(),
                    Arch::Arm64 | Arch::Arm64EC | Arch::Arm64E => arm64::ClangTargetCpuArchitectureArm64ISANames::name(TargetCpuArchitectureArm64::Generic).to_string(),
                    Arch::Riscv64 => "i+m+a+f+d+c".to_string(),
                    Arch::Native => unreachable!(),
                };
                Ok((ext, TargetCpuArchitectureX64::Generic, TargetCpuArchitectureArm64::Generic))
            }
        }
    }
}

