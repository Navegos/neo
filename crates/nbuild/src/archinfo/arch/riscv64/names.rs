// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/riscv64/names.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use super::isa::Riscv64ISA;
use super::targets::TargetCpuArchitectureRiscv64;

/// Provides a mapping between the TargetCpuArchitectureRiscv64 enum values and their default enabled Riscv64ISA enum extensions string representations for Clang.
pub struct ClangTargetCpuArchitectureRiscv64ISANames;

impl ClangTargetCpuArchitectureRiscv64ISANames {
    pub fn name(target: TargetCpuArchitectureRiscv64) -> &'static str {
        match target {
            TargetCpuArchitectureRiscv64::None | TargetCpuArchitectureRiscv64::Generic | TargetCpuArchitectureRiscv64::Generic_RV64 => "i",
            TargetCpuArchitectureRiscv64::Native => "",
            /* TargetCpuArchitectureRiscv64::Andes_AX25 | TargetCpuArchitectureRiscv64::Andes_AX45 | TargetCpuArchitectureRiscv64::Andes_NX45 => {
                "i+m+a+f+d+c+zicsr+zifencei+zmmul+zaamo+zalrsc+zca+zcd+xandesperf"
            }
            TargetCpuArchitectureRiscv64::Andes_AX45MPV => {
                "i+m+a+f+d+c+v+zicsr+zifencei+zmmul+zaamo+zalrsc+zca+zcd+zve32f+zve32x+zve64d+zve64f+zve64x+zvl128b+zvl32b+zvl64b+xandesperf"
            } */
            TargetCpuArchitectureRiscv64::MIPS_P8700 => {
                "i+m+a+f+d+c+zicsr+zifencei+zmmul+zaamo+zalrsc+zca+zcd+zba+zbb+xmipscbop+xmipscmov+xmipsexectl+xmipslsp"
            }
            /* TargetCpuArchitectureRiscv64::Rocket_RV64 => {
				"i+zicsr+zifencei"
			} */
            TargetCpuArchitectureRiscv64::Sifive_P450 => {
                "i+m+a+f+d+c+b+zic64b+zicbom+zicbop+zicboz+ziccamoa+ziccif+zicclsm+ziccrse+zicsr+zifencei+zihintntl+zihintpause+zihpm+zmmul+za64rs+zaamo+zalrsc+zfhmin+zca+zcd+zba+zbb+zbs+zkt"
            }
            TargetCpuArchitectureRiscv64::Sifive_P470 => {
                "i+m+a+f+d+c+b+v+zic64b+zicbom+zicbop+zicboz+ziccamoa+ziccif+zicclsm+ziccrse+zicsr+zifencei+zihintntl+zihintpause+zihpm+zmmul+za64rs+zaamo+zalrsc+zfhmin+zca+zcd+zba+zbb+zbs+zkt+zvbb+zvbc+zve32f+zve32x+zve64d+zve64f+zve64x+zvkb+zvkg+zvkn+zvknc+zvkned+zvkng+zvknha+zvknhb+zvks+zvksc+zvksed+zvksg+zvksh+zvkt+zvl128b+zvl32b+zvl64b+xsifivecdiscarddlone+xsifivecflushdlone"
            }
            TargetCpuArchitectureRiscv64::Sifive_P550 => {
                "i+m+a+f+d+c+zicsr+zifencei+zmmul+zaamo+zalrsc+zca+zcd+zba+zbb"
            }
            TargetCpuArchitectureRiscv64::Sifive_P670 => {
                "i+m+a+f+d+c+b+v+zic64b+zicbom+zicbop+zicboz+ziccamoa+ziccif+zicclsm+ziccrse+zicsr+zifencei+zihintntl+zihintpause+zihpm+zmmul+za64rs+zaamo+zalrsc+zfhmin+zca+zcd+zba+zbb+zbs+zkt+zvbb+zvbc+zve32f+zve32x+zve64d+zve64f+zve64x+zvkb+zvkg+zvkn+zvknc+zvkned+zvkng+zvknha+zvknhb+zvks+zvksc+zvksed+zvksg+zvksh+zvkt+zvl128b+zvl32b+zvl64b"
            }
            TargetCpuArchitectureRiscv64::Sifive_P870 => {
                "i+m+a+f+d+c+b+v+zic64b+zicbom+zicbop+zicboz+ziccamoa+ziccif+zicclsm+ziccrse+zicntr+zicond+zicsr+zifencei+zihintntl+zihintpause+zihpm+zimop+zmmul+za64rs+zaamo+zalrsc+zama16b+zawrs+zfa+zfbfmin+zfh+zfhmin+zca+zcb+zcd+zcmop+zba+zbb+zbs+zkr+zkt+zvbb+zvbc+zve32f+zve32x+zve64d+zve64f+zve64x+zvfbfmin+zvfbfwma+zvfh+zvfhmin+zvkb+zvkg+zvkn+zvknc+zvkned+zvkng+zvknha+zvknhb+zvks+zvksc+zvksed+zvksg+zvksh+zvkt+zvl128b+zvl32b+zvl64b+supm"
            }
            /* TargetCpuArchitectureRiscv64::Sifive_S21 | TargetCpuArchitectureRiscv64::Sifive_S51 => {
                "i+m+a+c+zicsr+zifencei+zmmul+zaamo+zalrsc+zca"
            }
            TargetCpuArchitectureRiscv64::Sifive_S54 | TargetCpuArchitectureRiscv64::Sifive_U54 | TargetCpuArchitectureRiscv64::Sifive_U74 => {
                "i+m+a+f+d+c+zicsr+zifencei+zmmul+zaamo+zalrsc+zca+zcd"
            }
            TargetCpuArchitectureRiscv64::Sifive_S76 => {
                "i+m+a+f+d+c+zicsr+zifencei+zihintpause+zmmul+zaamo+zalrsc+zca+zcd"
            }
            TargetCpuArchitectureRiscv64::Sifive_X180 => {
                "i+m+a+f+d+c+b+v+zic64b+zicbom+zicbop+zicboz+ziccamoa+ziccif+ziccrse+zicfilp+zicond+zicsr+zifencei+zihintntl+zihintpause+zihpm+zimop+zmmul+za64rs+zaamo+zalrsc+zawrs+zfa+zfbfmin+zfh+zfhmin+zca+zcb+zcd+zcmop+zba+zbb+zbs+zkt+zvbb+zvdot4a8i+zve32f+zve32x+zve64d+zve64f+zve64x+zvfbfa+zvfbfmin+zvfbfwma+zvfh+zvfhmin+zvkb+zvkt+zvl128b+zvl32b+zvl64b+xsfcease+xsfvfbfexp16e+xsfvfexp16e+xsfvfexp32e+xsfvfexpa+xsifivecflushdlone"
            }
            TargetCpuArchitectureRiscv64::Sifive_X280 => {
                "i+m+a+f+d+c+v+zicsr+zifencei+zmmul+zaamo+zalrsc+zfh+zfhmin+zca+zcd+zba+zbb+zve32f+zve32x+zve64d+zve64f+zve64x+zvfh+zvfhmin+zvl128b+zvl256b+zvl32b+zvl512b+zvl64b"
            }
            TargetCpuArchitectureRiscv64::Sifive_X390 => {
                "i+m+a+f+d+c+b+v+zic64b+zicbom+zicbop+zicboz+ziccamoa+ziccif+ziccrse+zicfilp+zicfiss+zicntr+zicond+zicsr+zifencei+zihintntl+zihintpause+zihpm+zimop+zmmul+za64rs+zaamo+zalrsc+zawrs+zfa+zfbfmin+zfh+zfhmin+zca+zcb+zcd+zcmop+zba+zbb+zbs+zkr+zkt+zvbb+zve32f+zve32x+zve64d+zve64f+zve64x+zvfbfmin+zvfbfwma+zvfh+zvfhmin+zvkb+zvkt+zvl1024b+zvl128b+zvl256b+zvl32b+zvl512b+zvl64b+xsifivecdiscarddlone+xsifivecflushdlone"
            }
            TargetCpuArchitectureRiscv64::Spacemit_A100 => {
                "i+m+a+f+d+c+b+v+zic64b+zicbom+zicbop+zicboz+ziccamoa+ziccif+zicclsm+ziccrse+zicntr+zicond+zicsr+zifencei+zihintntl+zihintpause+zihpm+zimop+zmmul+za64rs+zaamo+zalrsc+zawrs+zfa+zfbfmin+zfh+zfhmin+zca+zcb+zcd+zcmop+zba+zbb+zbc+zbkc+zbs+zkt+zvbb+zvbc+zve32f+zve32x+zve64d+zve64f+zve64x+zvfbfmin+zvfbfwma+zvfh+zvfhmin+zvkb+zvkg+zvkn+zvknc+zvkned+zvkng+zvknha+zvknhb+zvks+zvksc+zvksed+zvksg+zvksh+zvkt+zvl1024b+zvl128b+zvl256b+zvl32b+zvl512b+zvl64b+supm+xsmtvdotii"
            }
            TargetCpuArchitectureRiscv64::Spacemit_X60 => {
                "i+m+a+f+d+c+b+v+zic64b+zicbom+zicbop+zicboz+ziccamoa+ziccif+zicclsm+ziccrse+zicntr+zicond+zicsr+zifencei+zihintpause+zihpm+zmmul+za64rs+zaamo+zalrsc+zfh+zfhmin+zca+zcd+zba+zbb+zbc+zbkc+zbs+zkt+zve32f+zve32x+zve64d+zve64f+zve64x+zvfh+zvfhmin+zvkt+zvl128b+zvl256b+zvl32b+zvl64b+xsmtvdot"
            }
            TargetCpuArchitectureRiscv64::Spacemit_X100 => {
                "i+m+a+f+d+c+b+v+zic64b+zicbom+zicbop+zicboz+ziccamoa+ziccif+zicclsm+ziccrse+zicntr+zicond+zicsr+zifencei+zihintntl+zihintpause+zihpm+zimop+zmmul+za64rs+zaamo+zalrsc+zawrs+zfa+zfbfmin+zfh+zfhmin+zca+zcb+zcd+zcmop+zba+zbb+zbc+zbkc+zbs+zkt+zvbb+zvbc+zve32f+zve32x+zve64d+zve64f+zve64x+zvfbfmin+zvfbfwma+zvfh+zvfhmin+zvkb+zvkg+zvkn+zvknc+zvkned+zvkng+zvknha+zvknhb+zvks+zvksc+zvksed+zvksg+zvksh+zvkt+zvl128b+zvl256b+zvl32b+zvl64b+supm+xsmtvdot"
            }
            TargetCpuArchitectureRiscv64::Syntacore_SCR3_RV64 => {
                "i+m+a+c+zicsr+zifencei+zmmul+zaamo+zalrsc+zca"
            }
            TargetCpuArchitectureRiscv64::Syntacore_SCR4_RV64 | TargetCpuArchitectureRiscv64::Syntacore_SCR5_RV64=> {
                "i+m+a+f+d+c+zicsr+zifencei+zmmul+zaamo+zalrsc+zca+zcd"
            }
            TargetCpuArchitectureRiscv64::Syntacore_SCR7 => {
                "i+m+a+f+d+c+b+v+zicsr+zifencei+zmmul+zaamo+zalrsc+zca+zcd+zba+zbb+zbc+zbkb+zbkc+zbkx+zbs+zkn+zknd+zkne+zknh+zve32f+zve32x+zve64d+zve64f+zve64x+zvl128b+zvl32b+zvl64b"
            } */
            TargetCpuArchitectureRiscv64::TT_Ascalon_X => {
                "i+m+a+f+d+c+b+v+zic64b+zicbom+zicbop+zicboz+ziccamoa+ziccif+zicclsm+ziccrse+zicntr+zicond+zicsr+zifencei+zihintntl+zihintpause+zihpm+zimop+zmmul+za64rs+zaamo+zalrsc+zawrs+zfa+zfbfmin+zfh+zfhmin+zca+zcb+zcd+zcmop+zba+zbb+zbs+zkr+zkt+zvbb+zvbc+zve32f+zve32x+zve64d+zve64f+zve64x+zvfbfmin+zvfbfwma+zvfh+zvfhmin+zvkb+zvkg+zvkn+zvknc+zvkned+zvkng+zvknha+zvknhb+zvkt+zvl128b+zvl256b+zvl32b+zvl64b+supm"
            }
            TargetCpuArchitectureRiscv64::Veyron_V1 => {
                "i+m+a+f+d+c+b+zicbom+zicbop+zicboz+zicntr+zicsr+zifencei+zihintpause+zihpm+zmmul+zaamo+zalrsc+zca+zcd+zba+zbb+zbc+zbkc+zbs+xventanacondops"
            }
            TargetCpuArchitectureRiscv64::Xiangshan_Kunminghu => {
                "i+m+a+f+d+c+b+v+zic64b+zicbom+zicbop+zicboz+ziccamoa+ziccif+zicclsm+ziccrse+zicntr+zicond+zicsr+zifencei+zihintntl+zihintpause+zihpm+zimop+zmmul+za64rs+zaamo+zacas+zalrsc+zawrs+zfa+zfh+zfhmin+zca+zcb+zcd+zcmop+zba+zbb+zbc+zbkb+zbkc+zbkx+zbs+zkn+zknd+zkne+zknh+zks+zksed+zksh+zkt+zvbb+zve32f+zve32x+zve64d+zve64f+zve64x+zvfh+zvfhmin+zvkb+zvkt+zvl128b+zvl32b+zvl64b+supm"
            }
            TargetCpuArchitectureRiscv64::Xiangshan_Nanhu => {
                "i+m+a+f+d+c+b+zicbom+zicboz+zicsr+zifencei+zmmul+zaamo+zalrsc+zca+zcd+zba+zbb+zbc+zbkb+zbkc+zbkx+zbs+zkn+zknd+zkne+zknh+zks+zksed+zksh"
            }
            TargetCpuArchitectureRiscv64::XT_C910v2 => {
                "i+m+a+f+d+c+b+zic64b+zicbom+zicbop+zicboz+ziccamoa+ziccif+zicclsm+ziccrse+zicntr+zicond+zicsr+zifencei+zihintntl+zihintpause+zihpm+zimop+zmmul+za64rs+zaamo+zalrsc+zawrs+zfa+zfbfmin+zfh+zfhmin+zca+zcb+zcd+zcmop+zba+zbb+zbc+zbkc+zbs+zkt+xtheadba+xtheadbb+xtheadbs+xtheadcmo+xtheadcondmov+xtheadfmemidx+xtheadmac+xtheadmemidx+xtheadmempair+xtheadsync"
            }
            TargetCpuArchitectureRiscv64::XT_C920v2 => {
                "i+m+a+f+d+c+b+v+zic64b+zicbom+zicbop+zicboz+ziccamoa+ziccif+ziccrse+zicntr+zicond+zicsr+zifencei+zihintntl+zihintpause+zihpm+zimop+zmmul+za64rs+zaamo+zalrsc+zawrs+zfa+zfbfmin+zfh+zfhmin+zca+zcb+zcd+zcmop+zba+zbb+zbc+zbkc+zbs+zkt+zve32f+zve32x+zve64d+zve64f+zve64x+zvfbfmin+zvfbfwma+zvfh+zvfhmin+zvl128b+zvl32b+zvl64b+xtheadba+xtheadbb+xtheadbs+xtheadcmo+xtheadcondmov+xtheadfmemidx+xtheadmac+xtheadmemidx+xtheadmempair+xtheadsync+xtheadvdot"
            }
            //_ => "i+m+a+f+d+c",
        }
    }
}

/// Provides names for Riscv64 ISA for Clang.
pub struct ClangRiscv64ISANames;

impl ClangRiscv64ISANames {
    pub fn name(isa: Riscv64ISA) -> &'static str {
        isa.as_str()
    }
}

/// Provides names for Riscv64 ISA to disable for Clang.
pub struct ClangRiscv64NOISANames;

impl ClangRiscv64NOISANames {
    pub fn name(isa: Riscv64ISA) -> String {
        match isa {
            Riscv64ISA::None => "".into(),
            other => format!("-{}", other.as_str()),
        }
    }
}

