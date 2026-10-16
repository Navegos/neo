// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/platform.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use crate::archinfo::arch::arm64::TargetCpuArchitectureArm64;
use crate::archinfo::vector_length::CpuArchitectureVectorLength;
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::fmt;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

/// Supported CUDA GPU Architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub enum CudaArch {
    #[default]
    Native,
    /// NVIDIA T4, QUADRO RTX 8000, QUADRO RTX 6000, QUADRO RTX 5000, QUADRO RTX 4000, QUADRO RTX 3000, QUADRO T2000, NVIDIA T1200, NVIDIA T1000,
    /// NVIDIA T600, NVIDIA T500, NVIDIA T400, GeForce GTX 1650 Ti, NVIDIA TITAN RTX, GeForce RTX 2080 Ti, GeForce RTX 2080, GeForce RTX 2070, GeForce RTX 2060
    #[serde(rename = "75")]
    Sm_75,
    /// NVIDIA A100, NVIDIA A30
    #[serde(rename = "80")]
    Sm_80,
    /// NVIDIA A40, NVIDIA A10, NVIDIA A16, NVIDIA A2, NVIDIA RTX A6000, NVIDIA RTX A5000, NVIDIA RTX A4000, NVIDIA RTX A3000, NVIDIA RTX A2000, GeForce RTX 3090 Ti,
    /// GeForce RTX 3090, GeForce RTX 3080 Ti, GeForce RTX 3080, GeForce RTX 3070 Ti, GeForce RTX 3070, GeForce RTX 3060 Ti, GeForce RTX 3060, GeForce RTX 3050 Ti, GeForce RTX 3050
    #[serde(rename = "86")]
    Sm_86,
    /// Jetson AGX Orin, Jetson Orin NX, Jetson Orin Nan
    #[serde(rename = "87")]
    Sm_87,
    /// NVIDIA L4, NVIDIA L40, NVIDIA L40S, NVIDIA RTX 6000 Ada, NVIDIA RTX 5000 Ada, NVIDIA RTX 4500 Ada, NVIDIA RTX 4000 Ada, NVIDIA RTX 4000 SFF Ada,
    /// NVIDIA RTX 2000 Ada, GeForce RTX 4090, GeForce RTX 4080, GeForce RTX 4070 Ti, GeForce RTX 4070, GeForce RTX 4060 Ti, GeForce RTX 4060, GeForce RTX 4050
    #[serde(rename = "89")]
    Sm_89,
    /// NVIDIA GH200, NVIDIA H200, NVIDIA H100
    #[serde(rename = "90")]
    Sm_90,
    /// NVIDIA GB200, NVIDIA B200
    #[serde(rename = "100")]
    Sm_100,
    /// NVIDIA GB300 NVIDIA B300, NVIDIA GB300 (DGX Station)
    #[serde(rename = "103")]
    Sm_103,
    /// Jetson T5000, Jetson T4000
    #[serde(rename = "110")]
    Sm_110,
    /// NVIDIA RTX PRO 6000, Blackwell Server Edition, NVIDIA RTX PRO 4500, Blackwell Server Edition, NVIDIA RTX PRO 6000 Blackwell Workstation Edition,
    /// NVIDIA RTX PRO 6000 Blackwell Max-Q Workstation Edition, NVIDIA RTX PRO 5000 Blackwell, NVIDIA RTX PRO 4500 Blackwell, NVIDIA RTX PRO 4000 Blackwell,
    /// NVIDIA RTX PRO 4000 Blackwell SFF Edition, NVIDIA RTX PRO 2000 Blackwell, GeForce RTX 5090, GeForce RTX 5080, GeForce RTX 5070 Ti, GeForce RTX 5070,
    /// GeForce RTX 5060 Ti, GeForce RTX 5060, GeForce RTX 5050
    #[serde(rename = "120")]
    Sm_120,
    /// NVIDIA GB10 (DGX Spark)
    #[serde(rename = "121")]
    Sm_121,
}

impl fmt::Display for CudaArch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CudaArch::Native => write!(f, "native"),
            CudaArch::Sm_75 => write!(f, "75"),
            CudaArch::Sm_80 => write!(f, "80"),
            CudaArch::Sm_86 => write!(f, "86"),
            CudaArch::Sm_87 => write!(f, "87"),
            CudaArch::Sm_89 => write!(f, "89"),
            CudaArch::Sm_90 => write!(f, "90"),
            CudaArch::Sm_100 => write!(f, "100"),
            CudaArch::Sm_103 => write!(f, "103"),
            CudaArch::Sm_110 => write!(f, "110"),
            CudaArch::Sm_120 => write!(f, "120"),
            CudaArch::Sm_121 => write!(f, "121"),
        }
    }
}

impl FromStr for CudaArch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().replace('-', "_").as_str() {
            "native" | "host" | "current" => Ok(CudaArch::Native),
            "compute75" | "compute_75" | "compute-75" | "sm75" | "sm_75" | "sm-75" | "75" => Ok(CudaArch::Sm_75),
            "compute80" | "compute_80" | "compute-80" | "sm80" | "sm_80" | "sm-80" | "80" => Ok(CudaArch::Sm_80),
            "compute86" | "compute_86" | "compute-86" | "sm86" | "sm_86" | "sm-86" | "86" => Ok(CudaArch::Sm_86),
            "compute87" | "compute_87" | "compute-87" | "sm87" | "sm_87" | "sm-87" | "87" => Ok(CudaArch::Sm_87),
            "compute89" | "compute_89" | "compute-89" | "sm89" | "sm_89" | "sm-89" | "89" => Ok(CudaArch::Sm_89),
            "compute90" | "compute_90" | "compute-90" | "sm90" | "sm_90" | "sm-90" | "90" => Ok(CudaArch::Sm_90),
            "compute100" | "compute_100" | "compute-100" | "sm100" | "sm_100" | "sm-100" | "100" => Ok(CudaArch::Sm_100),
            "compute103" | "compute_103" | "compute-103" | "sm103" | "sm_103" | "sm-103" | "103" => Ok(CudaArch::Sm_103),
            "compute110" | "compute_110" | "compute-110" | "sm110" | "sm_110" | "sm-110" | "110" => Ok(CudaArch::Sm_110),
            "compute120" | "compute_120" | "compute-120" | "sm120" | "sm_120" | "sm-120" | "120" => Ok(CudaArch::Sm_120),
            "compute121" | "compute_121" | "compute-121" | "sm121" | "sm_121" | "sm-121" | "121" => Ok(CudaArch::Sm_121),
            other => Err(format!("Unknown CUDA architecture: {}", other)),
        }
    }
}

impl CudaArch {
    pub fn arch_name(&self) -> &'static str {
        match self {
            CudaArch::Native => "native",
            CudaArch::Sm_75 => "75",
            CudaArch::Sm_80 => "80",
            CudaArch::Sm_86 => "86",
            CudaArch::Sm_87 => "87",
            CudaArch::Sm_89 => "89",
            CudaArch::Sm_90 => "90",
            CudaArch::Sm_100 => "100",
            CudaArch::Sm_103 => "103",
            CudaArch::Sm_110 => "110",
            CudaArch::Sm_120 => "120",
            CudaArch::Sm_121 => "121",
        }
    }

    pub fn sm_name(&self) -> &'static str {
        match self {
            CudaArch::Native => "native",
            CudaArch::Sm_75 => "sm_75",
            CudaArch::Sm_80 => "sm_80",
            CudaArch::Sm_86 => "sm_86",
            CudaArch::Sm_87 => "sm_87",
            CudaArch::Sm_89 => "sm_89",
            CudaArch::Sm_90 => "sm_90",
            CudaArch::Sm_100 => "sm_100",
            CudaArch::Sm_103 => "sm_103",
            CudaArch::Sm_110 => "sm_110",
            CudaArch::Sm_120 => "sm_120",
            CudaArch::Sm_121 => "sm_121",
        }
    }

    pub fn compute_name(&self) -> &'static str {
        match self {
            CudaArch::Native => "native",
            CudaArch::Sm_75 => "compute_75",
            CudaArch::Sm_80 => "compute_80",
            CudaArch::Sm_86 => "compute_86",
            CudaArch::Sm_87 => "compute_87",
            CudaArch::Sm_89 => "compute_89",
            CudaArch::Sm_90 => "compute_90",
            CudaArch::Sm_100 => "compute_100",
            CudaArch::Sm_103 => "compute_103",
            CudaArch::Sm_110 => "compute_110",
            CudaArch::Sm_120 => "compute_120",
            CudaArch::Sm_121 => "compute_121",
        }
    }

    /// Parses a string with potentially multi-value CUDA architectures separated by ';', ',', or '+'
    pub fn parse_multivalue(s: &str) -> Result<Vec<CudaArch>, String> {
        let mut list = Vec::new();
        for token in s.split(|c| c == ';' || c == ',' || c == '+').map(|t| t.trim()).filter(|t| !t.is_empty()) {
            let clean = token.trim_matches(|c| c == '(' || c == ')' || c == '[' || c == ']');
            let arch = clean.parse::<CudaArch>()?;
            if !list.contains(&arch) {
                list.push(arch);
            }
        }
        if list.is_empty() {
            list.push(CudaArch::Sm_75);
        }
        Ok(list)
    }

    /// Formats list of CUDA architectures as cuda_arch string (e.g. "75;86;89;120")
    pub fn format_cuda_arch(list: &[CudaArch]) -> String {
        list.iter().map(|a| a.arch_name()).collect::<Vec<_>>().join(";")
    }

    /// Formats list of CUDA architectures as cuda_sm_arch string (e.g. "sm_75;sm_86;sm_89;sm_120")
    pub fn format_cuda_sm_arch(list: &[CudaArch]) -> String {
        list.iter().map(|a| a.sm_name()).collect::<Vec<_>>().join(";")
    }
}

/// Supported AMD GPU Architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub enum RocmArch {
    #[default]
    Native,
    /* /// ROCm unsupported.
    #[serde(rename = "600")]
    Gfx600, // unsupported
    /// ROCm unsupported.
    #[serde(rename = "601")]
    Gfx601, // unsupported
    /// ROCm unsupported.
    #[serde(rename = "602")]
    Gfx602, // unsupported
    /// ROCm unsupported.
    #[serde(rename = "700")]
    Gfx700, // unsupported
    /// ROCm unsupported.
    #[serde(rename = "701")]
    Gfx701, // unsupported
    /// ROCm unsupported.
    #[serde(rename = "702")]
    Gfx702, // unsupported
    /// ROCm unsupported.
    #[serde(rename = "704")]
    Gfx704, // unsupported
    /// ROCm unsupported.
    #[serde(rename = "801")]
    Gfx801, // unsupported
    /// ROCm unsupported.
    #[serde(rename = "802")]
    Gfx802, // unsupported
    /// AMD Instinct GPUs, GCN4.0, MI6, GCN3.0, MI8.
    /// ROCm unsupported.
    #[serde(rename = "803")]
    Gfx803, // unsupported
    /// ROCm unsupported.
    #[serde(rename = "805")]
    Gfx805, // unsupported
    /// ROCm unsupported.
    #[serde(rename = "810")]
    Gfx810, // unsupported
    /// AMD Instinct GPUs, GCN5.0, MI25.
    /// AMD Consumer Desktop: Radeon RX Vega 64, Radeon RX Vega 56, and the dual-GPU Radeon RX Vega M (found in specialized Intel Kaby Lake-G processors).
    /// AMD Workstation & Server: Radeon Pro WX 9100, Radeon Pro SSG, and the early Radeon Instinct MI25 accelerator.
    /// ROCm unsupported.
    #[serde(rename = "900")]
    Gfx900, // unsupported
    /// AMD Radeon Desktop APUs: GCN5.0, Ryzen 5 2400G, Ryzen 3 2200G, Ryzen 5 3400G, and Ryzen 7 5700G.
    /// AMD Mobile Laptops: Ryzen 7 2700U, Ryzen 5 3500U, Ryzen 7 4700U, Ryzen 7 5800U, and Ryzen 7 5825U.
    /// AMD Integrated Graphics Tiers: Radeon Vega 11, Vega 10, Vega 8, and Vega 7.
    /// ROCm unsupported.
    #[serde(rename = "902")]
    Gfx902, // unsupported
    /// AMD Radeon Pro Vega GPUs, GCN5.0, Radeon Pro Vega 20, Radeon Pro Vega 16
    /// ROCm unsupported.
    #[serde(rename = "904")]
    Gfx904, // unsupported
    /// AMD Instinct GPUs, GCN5.1, MI50 (16GB), MI50 (32GB), MI60.
    /// AMD Consumer & Enthusiast: Radeon VII (the 16GB HBM2 desktop card).
    /// AMD Workstation: Radeon Pro Vega II and Radeon Pro Vega II Duo (exclusively deployed in Apple's Intel-based Mac Pro configurations).
    /// ROCm unsupported.
    #[serde(rename = "906")]
    Gfx906, // unsupported */
    /// AMD Instinct GPUs, CDNA, MI100.
    #[serde(rename = "908")]
    Gfx908,
    /// AMD Instinct GPUs, CDNA2, MI210, MI250, MI250X.
    #[serde(rename = "90a")]
    Gfx90a,
    /// AMD Instinct GPUs, CDNA3, MI300A.
    /// ROCm override to gfx942.
    #[serde(rename = "940")]
    Gfx940, // override by gfx942
    /// AMD Instinct GPUs, CDNA3, MI300X.
    /// ROCm override to gfx942.
    #[serde(rename = "941")]
    Gfx941, // override by gfx942
    /// AMD Instinct GPUs, CDNA3, MI300A, MI300X, MI325X.
    #[serde(rename = "942")]
    Gfx942,
    /// AMD Instinct GPUs, CDNA4, MI350P, MI350X, MI355X.
    #[serde(rename = "950")]
    Gfx950,
    /* /// AMD Radeon Desktop GPUs, RDNA1, Radeon RX 5700 XT, Radeon RX 5700, Radeon RX 5600 XT, Radeon RX 5600.
    /// AMD Radeon PRO Workstation GPUs, RDNA1, Radeon Pro W5700, Radeon Pro W5700X.
    /// ROCm unsupported.
    #[serde(rename = "1010")]
    Gfx1010, // unsupported
    /// AMD Radeon PRO Workstation GPUs, RDNA1, Radeon Pro V520, Radeon Pro 5600M.
    /// ROCm unsupported.
    #[serde(rename = "1011")]
    Gfx1011, // unsupported
    /// AMD Radeon Desktop GPUs, RDNA1, Radeon RX 5500 XT, Radeon RX 5500, Radeon RX 5300.
    /// AMD Radeon Mobile GPUs, RDNA1, Radeon RX 5500M, Radeon RX 5300M.
    /// AMD Radeon PRO Workstation GPUs, RDNA1, Radeon Pro 5500M, Radeon Pro 5300, Radeon Pro 5300M, Radeon Pro W5500.
    /// ROCm unsupported.
    #[serde(rename = "1012")]
    Gfx1012, // unsupported
    /// AMD custom hardware and embedded desktop kits, AMD BC-250 Mining Rig APU, AMD 4700S / 4800S Desktop Kits.
    /// ROCm unsupported.
    #[serde(rename = "1013")]
    Gfx1013, // unsupported */
    /// AMD Radeon Desktop GPUs, RDNA2, Radeon RX 6950 XT, Radeon RX 6900 XT, Radeon RX 6800 XT, Radeon RX 6800.
    /// AMD Radeon PRO Workstation GPUs, RDNA2, Radeon PRO W6800, Radeon PRO V620, Radeon PRO W6800X, Radeon PRO W6900X.
    #[serde(rename = "1030")]
    Gfx1030,
    /// AMD Radeon Desktop GPUs, RDNA2, Radeon RX 6750 XT, Radeon RX 6700 XT, Radeon RX 6700, Radeon RX 6750 GRE (10GB & 12GB variants).
    /// AMD Radeon Mobile GPUs, RDNA2, Radeon RX 6800M, Radeon RX 6850M XT, Radeon RX 6700M.
    /// ROCm override to gfx1030.
    #[serde(rename = "1031")]
    Gfx1031, // override by gfx1030
    /// AMD Radeon Desktop GPUs, RDNA2, Radeon RX 6650 XT, Radeon RX 6600 XT, Radeon RX 6600, Radeon RX 6600 LE.
    /// AMD Radeon Mobile GPUs, RDNA2, Radeon RX 6650M, Radeon RX 6650M XT, Radeon RX 6600M, Radeon RX 6600S, RX 6700S, Radeon RX 6800S.
    /// AMD Radeon PRO Workstation GPUs, RDNA2, Radeon PRO W6600, Radeon Pro W6600M, Radeon Pro W6600X
    /// ROCm override to gfx1030.
    #[serde(rename = "1032")]
    Gfx1032, // override by gfx1030
    /// AMD Radeon Desktop GPUs, RDNA2, Radeon RX 6500 XT, Radeon RX 6400, Radeon RX 6300.
    /// AMD Radeon Mobile GPUs, RDNA2, Radeon RX 6550M, Radeon RX 6550S, Radeon RX 6500M, Radeon RX 6450M, Radeon RX 6300M.
    /// ROCm override to gfx1030.
    #[serde(rename = "1034")]
    Gfx1034, // override by gfx1030
    /// AMD Radeon Mobile GPUs, RDNA2, Radeon 680M, Radeon 660M.
    /// ROCm override to gfx1030.
    #[serde(rename = "1035")]
    Gfx1035, // override by gfx1030
    /// Ryzen 9: 7950X3D, 7950X, 7900X3D, 7900X, 7900.
    /// Ryzen 7: 7800X3D, 7700X, 7700.
    /// Ryzen 5: 7600X, 7600.
    /// ROCm override to gfx1030.
    #[serde(rename = "1036")]
    Gfx1036, // override by gfx1030
    /// AMD Radeon GPUs, RDNA3, Radeon RX 7900 GRE, Radeon RX 7900 XT, Radeon RX 7900 XTX.
    /// AMD Radeon PRO GPUs, Radeon PRO W7800, Radeon PRO W7800 48GB, Radeon PRO W7900, Radeon PRO W7900 Dual Slot.
    #[serde(rename = "1100")]
    Gfx1100,
    /// AMD Radeon GPUs, RDNA3, Radeon RX 7700 XT, Radeon RX 7700, Radeon RX 7800 XT.
    /// AMD Radeon PRO GPUs, RDNA3, Radeon PRO W7700, Radeon PRO V710.
    #[serde(rename = "1101")]
    Gfx1101,
    /// AMD Radeon GPUs, RDNA3, Radeon RX 7600.
    #[serde(rename = "1102")]
    Gfx1102,
    /// AMD Ryzen APUs, RDNA3, Ryzen 3 210, Ryzen 3 PRO 210, Ryzen 5 220, Ryzen 5 230, Ryzen 5 240, Ryzen 5 PRO 215,
    /// Ryzen 5 PRO 220, Ryzen 5 PRO 230, Ryzen 7 250, Ryzen 7 260, Ryzen 7 PRO 250, Ryzen 9 270, Ryzen 7 7840U.
    #[serde(rename = "1103")]
    Gfx1103,
    /// AMD Ryzen APUs, RDNA3.5, Ryzen AI 9 365, Ryzen AI 9 HX 370, Ryzen AI 9 HX 375, Ryzen AI 9 HX PRO 370, Ryzen AI 9 HX PRO 375,
    /// Ryzen AI 7 450, Ryzen AI 9 465, Ryzen AI 9 HX 470, Ryzen AI 9 HX 475, Ryzen AI 9 PRO 465, Ryzen AI 9 HX PRO 470, Ryzen AI 9 HX PRO 475.
    #[serde(rename = "1150")]
    Gfx1150,
    /// AMD Ryzen APUs, RDNA3.5, Ryzen AI Max 385, Ryzen AI Max 390, Ryzen AI Max+ 388, Ryzen AI Max+ 392, Ryzen AI Max+ 395, Ryzen AI Max+ PRO 395,
    /// Ryzen AI Max PRO 380, Ryzen AI Max PRO 385, Ryzen AI Max PRO 390, Ryzen AI Max+ PRO 495, Ryzen AI Max PRO 485, Ryzen AI Max PRO 490.
    #[serde(rename = "1151")]
    Gfx1151,
    /// AMD Ryzen APUs, RDNA3.5, Ryzen AI 5 330, Ryzen AI 5 340, Ryzen AI 5 PRO 340, Ryzen AI 7 345, Ryzen AI 7 350,
    /// Ryzen AI 7 PRO 350, Ryzen AI 7 450, Ryzen AI 5 PRO 440, Ryzen AI 7 PRO 450.
    #[serde(rename = "1152")]
    Gfx1152,
    /// AMD Ryzen APUs, RDNA3.5, Ryzen AI 5 430, Ryzen AI 5 435, Ryzen AI 5 PRO 435, Ryzen AI 7 445.
    #[serde(rename = "1153")]
    Gfx1153,
    /// AMD Ryzen APUs, RDNA 4m, upcoming undesclosed Ryzen 500 Series (Codenamed "Medusa Point"): next-gen mobile.
    #[serde(rename = "1170")]
    Gfx1170,
    /// AMD Ryzen APUs, RDNA 4m, upcoming undesclosed AMD Ryzen 500 Series (Codenamed "Medusa Point"): next-gen mobile.
    #[serde(rename = "1171")]
    Gfx1171,
    /// AMD Ryzen APUs, RDNA 4m, upcoming undesclosed AMD Ryzen 500 Series (Codenamed "Medusa Point"): next-gen mobile.
    #[serde(rename = "1172")]
    Gfx1172,
    /// AMD Radeon GPUs, RDNA4, Radeon RX 9050 (4GB), Radeon RX 9050, Radeon RX 9060, Radeon RX 9060 XT, Radeon RX 9060 XT LP.
    #[serde(rename = "1200")]
    Gfx1200,
    /// AMD Radeon GPUs, RDNA4, Radeon RX 9070, Radeon RX 9070 GRE, Radeon RX 9070 XT.
    /// AMD Radeon PRO GPUs, RDNA4, Radeon AI PRO R9600D, Radeon AI PRO R9700, Radeon AI PRO R9700S.
    #[serde(rename = "1201")]
    Gfx1201,
}

impl fmt::Display for RocmArch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RocmArch::Native => write!(f, "native"),
            /* RocmArch::Gfx600 => write!(f, "600"),
            RocmArch::Gfx601 => write!(f, "601"),
            RocmArch::Gfx602 => write!(f, "602"),
            RocmArch::Gfx700 => write!(f, "700"),
            RocmArch::Gfx701 => write!(f, "701"),
            RocmArch::Gfx702 => write!(f, "702"),
            RocmArch::Gfx704 => write!(f, "704"),
            RocmArch::Gfx801 => write!(f, "801"),
            RocmArch::Gfx802 => write!(f, "802"),
            RocmArch::Gfx803 => write!(f, "803"),
            RocmArch::Gfx805 => write!(f, "805"),
            RocmArch::Gfx810 => write!(f, "810"),
            RocmArch::Gfx900 => write!(f, "900"),
            RocmArch::Gfx902 => write!(f, "902"),
            RocmArch::Gfx904 => write!(f, "904"),
            RocmArch::Gfx906 => write!(f, "906"), */
            RocmArch::Gfx908 => write!(f, "908"),
            RocmArch::Gfx90a => write!(f, "90a"),
            RocmArch::Gfx940 => write!(f, "940"),
            RocmArch::Gfx941 => write!(f, "941"),
            RocmArch::Gfx942 => write!(f, "942"),
            RocmArch::Gfx950 => write!(f, "950"),
            /* RocmArch::Gfx1010 => write!(f, "1010"),
            RocmArch::Gfx1011 => write!(f, "1011"),
            RocmArch::Gfx1012 => write!(f, "1012"),
            RocmArch::Gfx1013 => write!(f, "1013"), */
            RocmArch::Gfx1030 => write!(f, "1030"),
            RocmArch::Gfx1031 => write!(f, "1031"),
            RocmArch::Gfx1032 => write!(f, "1032"),
            RocmArch::Gfx1034 => write!(f, "1034"),
            RocmArch::Gfx1035 => write!(f, "1035"),
            RocmArch::Gfx1036 => write!(f, "1036"),
            RocmArch::Gfx1100 => write!(f, "1100"),
            RocmArch::Gfx1101 => write!(f, "1101"),
            RocmArch::Gfx1102 => write!(f, "1102"),
            RocmArch::Gfx1103 => write!(f, "1103"),
            RocmArch::Gfx1150 => write!(f, "1150"),
            RocmArch::Gfx1151 => write!(f, "1151"),
            RocmArch::Gfx1152 => write!(f, "1152"),
            RocmArch::Gfx1153 => write!(f, "1153"),
            RocmArch::Gfx1170 => write!(f, "1170"),
            RocmArch::Gfx1171 => write!(f, "1171"),
            RocmArch::Gfx1172 => write!(f, "1172"),
            RocmArch::Gfx1200 => write!(f, "1200"),
            RocmArch::Gfx1201 => write!(f, "1201"),
        }
    }
}

impl FromStr for RocmArch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().replace('-', "_").as_str() {
            "native" | "host" | "current" => Ok(RocmArch::Native),
            "gfx908" | "gfx_908" | "gfx-908" | "908" => Ok(RocmArch::Gfx908),
            "gfx90a" | "gfx_90a" | "gfx-90a" | "90a" => Ok(RocmArch::Gfx90a),
            "gfx940" | "gfx_940" | "gfx-940" | "940" => Ok(RocmArch::Gfx940),
            "gfx941" | "gfx_941" | "gfx-941" | "941" => Ok(RocmArch::Gfx941),
            "gfx942" | "gfx_942" | "gfx-942" | "942" => Ok(RocmArch::Gfx942),
            "gfx950" | "gfx_950" | "gfx-950" | "950" => Ok(RocmArch::Gfx950),
            "gfx1030" | "gfx_1030" | "gfx-1030" | "1030" => Ok(RocmArch::Gfx1030),
            "gfx1031" | "gfx_1031" | "gfx-1031" | "1031" => Ok(RocmArch::Gfx1031),
            "gfx1032" | "gfx_1032" | "gfx-1032" | "1032" => Ok(RocmArch::Gfx1032),
            "gfx1034" | "gfx_1034" | "gfx-1034" | "1034" => Ok(RocmArch::Gfx1034),
            "gfx1035" | "gfx_1035" | "gfx-1035" | "1035" => Ok(RocmArch::Gfx1035),
            "gfx1036" | "gfx_1036" | "gfx-1036" | "1036" => Ok(RocmArch::Gfx1036),
            "gfx1100" | "gfx_1100" | "gfx-1100" | "1100" => Ok(RocmArch::Gfx1100),
            "gfx1101" | "gfx_1101" | "gfx-1101" | "1101" => Ok(RocmArch::Gfx1101),
            "gfx1102" | "gfx_1102" | "gfx-1102" | "1102" => Ok(RocmArch::Gfx1102),
            "gfx1103" | "gfx_1103" | "gfx-1103" | "1103" => Ok(RocmArch::Gfx1103),
            "gfx1150" | "gfx_1150" | "gfx-1150" | "1150" => Ok(RocmArch::Gfx1150),
            "gfx1151" | "gfx_1151" | "gfx-1151" | "1151" => Ok(RocmArch::Gfx1151),
            "gfx1152" | "gfx_1152" | "gfx-1152" | "1152" => Ok(RocmArch::Gfx1152),
            "gfx1153" | "gfx_1153" | "gfx-1153" | "1153" => Ok(RocmArch::Gfx1153),
            "gfx1170" | "gfx_1170" | "gfx-1170" | "1170" => Ok(RocmArch::Gfx1170),
            "gfx1171" | "gfx_1171" | "gfx-1171" | "1171" => Ok(RocmArch::Gfx1171),
            "gfx1172" | "gfx_1172" | "gfx-1172" | "1172" => Ok(RocmArch::Gfx1172),
            "gfx1200" | "gfx_1200" | "gfx-1200" | "1200" => Ok(RocmArch::Gfx1200),
            "gfx1201" | "gfx_1201" | "gfx-1201" | "1201" => Ok(RocmArch::Gfx1201),
            other => Err(format!("Unknown or unsupported Rocm gfx architecture: {}", other)),
        }
    }
}

impl RocmArch {
    pub fn arch_name(&self) -> &'static str {
        match self {
            RocmArch::Native => "native",
            RocmArch::Gfx908 => "908",
            RocmArch::Gfx90a => "90a",
            RocmArch::Gfx940 => "940",
            RocmArch::Gfx941 => "941",
            RocmArch::Gfx942 => "942",
            RocmArch::Gfx950 => "950",
            RocmArch::Gfx1030 => "1030",
            RocmArch::Gfx1031 => "1031",
            RocmArch::Gfx1032 => "1032",
            RocmArch::Gfx1034 => "1034",
            RocmArch::Gfx1035 => "1035",
            RocmArch::Gfx1036 => "1036",
            RocmArch::Gfx1100 => "1100",
            RocmArch::Gfx1101 => "1101",
            RocmArch::Gfx1102 => "1102",
            RocmArch::Gfx1103 => "1103",
            RocmArch::Gfx1150 => "1150",
            RocmArch::Gfx1151 => "1151",
            RocmArch::Gfx1152 => "1152",
            RocmArch::Gfx1153 => "1153",
            RocmArch::Gfx1170 => "1170",
            RocmArch::Gfx1171 => "1171",
            RocmArch::Gfx1172 => "1172",
            RocmArch::Gfx1200 => "1200",
            RocmArch::Gfx1201 => "1201",
        }
    }

    pub fn gfx_name(&self) -> &'static str {
        match self {
            RocmArch::Native => "native",
            RocmArch::Gfx908 => "gfx908",
            RocmArch::Gfx90a => "gfx90a",
            RocmArch::Gfx940 => "gfx940",
            RocmArch::Gfx941 => "gfx941",
            RocmArch::Gfx942 => "gfx942",
            RocmArch::Gfx950 => "gfx950",
            RocmArch::Gfx1030 => "gfx1030",
            RocmArch::Gfx1031 => "gfx1031",
            RocmArch::Gfx1032 => "gfx1032",
            RocmArch::Gfx1034 => "gfx1034",
            RocmArch::Gfx1035 => "gfx1035",
            RocmArch::Gfx1036 => "gfx1036",
            RocmArch::Gfx1100 => "gfx1100",
            RocmArch::Gfx1101 => "gfx1101",
            RocmArch::Gfx1102 => "gfx1102",
            RocmArch::Gfx1103 => "gfx1103",
            RocmArch::Gfx1150 => "gfx1150",
            RocmArch::Gfx1151 => "gfx1151",
            RocmArch::Gfx1152 => "gfx1152",
            RocmArch::Gfx1153 => "gfx1153",
            RocmArch::Gfx1170 => "gfx1170",
            RocmArch::Gfx1171 => "gfx1171",
            RocmArch::Gfx1172 => "gfx1172",
            RocmArch::Gfx1200 => "gfx1200",
            RocmArch::Gfx1201 => "gfx1201",
        }
    }

    /// Parses a string with potentially multi-value ROCm architectures separated by ';', ',', or '+'
    pub fn parse_multivalue(s: &str) -> Result<Vec<RocmArch>, String> {
        let mut list = Vec::new();
        for token in s.split(|c| c == ';' || c == ',' || c == '+').map(|t| t.trim()).filter(|t| !t.is_empty()) {
            let clean = token.trim_matches(|c| c == '(' || c == ')' || c == '[' || c == ']');
            let arch = clean.parse::<RocmArch>()?;
            if !list.contains(&arch) {
                list.push(arch);
            }
        }
        if list.is_empty() {
            list.push(RocmArch::Gfx1030);
        }
        Ok(list)
    }

    /// Formats list of ROCm architectures as rocm_arch string (e.g. "908;90a;950;1030")
    pub fn format_rocm_arch(list: &[RocmArch]) -> String {
        list.iter().map(|a| a.arch_name()).collect::<Vec<_>>().join(";")
    }

    /// Formats list of ROCm architectures as rocm_gfx_arch string (e.g. "gfx908;gfx90a;gfx950;gfx1030")
    pub fn format_rocm_gfx_arch(list: &[RocmArch]) -> String {
        list.iter().map(|a| a.gfx_name()).collect::<Vec<_>>().join(";")
    }
}

/// Supported Intel GPU Architectures and Microarchitectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub enum XpuArch {
    #[default]
    Native,

    // ==========================================
    // DISCRETE GRAPHICS ARCHITECTURES
    // ==========================================

    /// Intel Data Center GPU Max Series (Ponte Vecchio). High-performance matrix XMX compute.
    #[serde(rename = "pvc")]
    Pvc,
    /// Intel Arc A580, A750, A770 Desktop GPUs, Arc Pro A60, and Data Center Flex 170. (Xe-HPG Alchemist High-tier)
    #[serde(rename = "acm-g10")]
    AcmG10,
    /// Intel Arc A530M, A570M Laptop GPUs. (Xe-HPG Alchemist Mid-tier)
    #[serde(rename = "acm-g12")]
    AcmG12,
    /// Intel Arc A310, A380 Desktop GPUs, Arc Pro A40/A50, Mobile A350M/A370M, and Data Center Flex 140. (Xe-HPG Alchemist Entry-tier)
    #[serde(rename = "acm-g11")]
    AcmG11,
    /// Intel Arc B580, B750, B770 Desktop GPUs and Battlemage Pro Workstation Cards. (Xe2-HPG Battlemage)
    #[serde(rename = "bmg")]
    Bmg,

    // ==========================================
    // MODERN CLIENT INTEGRATED ARCHITECTURES
    // ==========================================

    /// Intel Core Ultra (Series 1) "Meteor Lake" Processors with integrated Intel Arc Graphics. (Xe-LPG)
    #[serde(rename = "mtl")]
    Mtl,
    /// Intel Core Ultra "Arrow Lake" Desktop & Mobile Processors with integrated graphics. (Xe-LPG Plus)
    #[serde(rename = "arl")]
    Arl,
    /// Intel Core Ultra (Series 2) "Lunar Lake" Mobile Processors with dedicated XMX units. (Xe2-LPG)
    #[serde(rename = "lnl")]
    Lnl,
    /// Next-Generation Intel Core Ultra "Panther Lake" Client Processors. (Xe3-LPG)
    #[serde(rename = "ptl")]
    Ptl,
    /// Next-Generation Intel Ultra "Nova Lake" Core Platform.
    #[serde(rename = "nvl")]
    Nvl,

    // ==========================================
    // ABSTRACT FAMILIES & GENERIC TARGET GROUPS
    // ==========================================

    /// Broad target for all Intel Xe family architectures.
    #[serde(rename = "xe")]
    Xe,
    /// Broad target for all Intel Xe2 family architectures (Battlemage/Lunar Lake).
    #[serde(rename = "xe2")]
    Xe2,
    /// Broad target for all Intel Xe3 family architectures (Panther Lake).
    #[serde(rename = "xe3")]
    Xe3,
    /// Catch-all target grouping for High-Performance Datacenter Computes (Max Series/PVC).
    #[serde(rename = "xe-hpc")]
    XeHpc,
    /// Catch-all target grouping for Alchemist Discrete/Gaming Hardware (Arc/Flex).
    #[serde(rename = "xe-hpg")]
    XeHpg,
    /// Catch-all target grouping for client integrated setups (Meteor Lake/Arrow Lake).
    #[serde(rename = "xe-lpg")]
    XeLpg,
    /// Catch-all target grouping for secondary client generation integrated setups (Lunar Lake).
    #[serde(rename = "xe2-lpg")]
    Xe2Lpg,
}

impl fmt::Display for XpuArch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XpuArch::Native => write!(f, "native"),
            XpuArch::Pvc => write!(f, "pvc"),
            XpuArch::AcmG10 => write!(f, "acm-g10"),
            XpuArch::AcmG12 => write!(f, "acm-g12"),
            XpuArch::AcmG11 => write!(f, "acm-g11"),
            XpuArch::Bmg => write!(f, "bmg"),
            XpuArch::Mtl => write!(f, "mtl"),
            XpuArch::Arl => write!(f, "arl"),
            XpuArch::Lnl => write!(f, "lnl"),
            XpuArch::Ptl => write!(f, "ptl"),
            XpuArch::Nvl => write!(f, "nvl"),
            XpuArch::Xe => write!(f, "xe"),
            XpuArch::Xe2 => write!(f, "xe2"),
            XpuArch::Xe3 => write!(f, "xe3"),
            XpuArch::XeHpc => write!(f, "xe-hpc"),
            XpuArch::XeHpg => write!(f, "xe-hpg"),
            XpuArch::XeLpg => write!(f, "xe-lpg"),
            XpuArch::Xe2Lpg => write!(f, "xe2-lpg"),
        }
    }
}

impl FromStr for XpuArch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().replace('-', "_").as_str() {
            "native" | "host" | "current" => Ok(XpuArch::Native),
                                               "pvc" => Ok(XpuArch::Pvc),
                    "acmg10" | "acm_g10" | "acm-g10" => Ok(XpuArch::AcmG10),
                    "acmg12" | "acm_g12" | "acm-g12" => Ok(XpuArch::AcmG12),
                    "acmg11" | "acm_g11" | "acm-g11" => Ok(XpuArch::AcmG11),
                                               "bmg" => Ok(XpuArch::Bmg),
                                               "mtl" => Ok(XpuArch::Mtl),
                                               "arl" => Ok(XpuArch::Arl),
                                               "lnl" => Ok(XpuArch::Lnl),
                                               "ptl" => Ok(XpuArch::Ptl),
                                               "nvl" => Ok(XpuArch::Nvl),
                                                "xe" => Ok(XpuArch::Xe),
                                               "xe2" => Ok(XpuArch::Xe2),
                                               "xe3" => Ok(XpuArch::Xe3),
                       "xehpc" | "xe_hpc" | "xe-hpc" => Ok(XpuArch::XeHpc),
                       "xehpg" | "xe_hpg" | "xe-hpg" => Ok(XpuArch::XeHpg),
                       "xelpg" | "xe_lpg" | "xe-lpg" => Ok(XpuArch::XeLpg),
                    "xe2lpg" | "xe2_lpg" | "xe2-lpg" => Ok(XpuArch::Xe2Lpg),
            other => Err(format!("Unknown or unsupported Intel XPU architecture: {}", other)),
        }
    }
}

impl XpuArch {
    pub fn arch_name(&self) -> &'static str {
        match self {
            XpuArch::Native => "native",
            XpuArch::Pvc => "pvc",
            XpuArch::AcmG10 => "acm-g10",
            XpuArch::AcmG12 => "acm-g12",
            XpuArch::AcmG11 => "acm-g11",
            XpuArch::Bmg => "bmg",
            XpuArch::Mtl => "mtl",
            XpuArch::Arl => "arl",
            XpuArch::Lnl => "lnl",
            XpuArch::Ptl => "ptl",
            XpuArch::Nvl => "nvl",
            XpuArch::Xe => "xe",
            XpuArch::Xe2 => "xe2",
            XpuArch::Xe3 => "xe3",
            XpuArch::XeHpc => "xe-hpc",
            XpuArch::XeHpg => "xe-hpg",
            XpuArch::XeLpg => "xe-lpg",
            XpuArch::Xe2Lpg => "xe2-lpg",
        }
    }

    /// Parses a string with potentially multi-value Intel XPU architectures separated by ';', ',', or '+'
    pub fn parse_multivalue(s: &str) -> Result<Vec<XpuArch>, String> {
        let mut list = Vec::new();
        for token in s.split(|c| c == ';' || c == ',' || c == '+').map(|t| t.trim()).filter(|t| !t.is_empty()) {
            let clean = token.trim_matches(|c| c == '(' || c == ')' || c == '[' || c == ']');
            let arch = clean.parse::<XpuArch>()?;
            if !list.contains(&arch) {
                list.push(arch);
            }
        }
        if list.is_empty() {
            list.push(XpuArch::Bmg);
        }
        Ok(list)
    }

    /// Formats list of Intel XPU architectures as xpu_arch string (e.g. "arl;lnl;ptl;nvl")
    pub fn format_xpu_arch(list: &[XpuArch]) -> String {
        list.iter().map(|a| a.arch_name()).collect::<Vec<_>>().join(";")
    }
}

/// Represents the physical Apple Silicon GPU Hardware Architecture Family.
/// These values map directly to physical execution layers and limits in the hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AppleGpuFamily {
    /// MTLGPUFamily.apple2.
    /// Represents the Apple family 2 GPU features that correspond to the Apple A8 GPUs and later.
    /// Metal version: Metal.
    /// iOS 13.0+, iPadOS 13.0+, Mac Catalyst 13.1+, macOS 10.15+, tvOS 13.0+, visionOS 1.0+.
    #[serde(rename = "apple2")]
    Apple2,
    /// MTLGPUFamily.apple3.
    /// Represents the Apple family 3 GPU features that correspond to the Apple A9 and A10 GPUs and later.
    /// Metal version: Metal.
    /// iOS 13.0+, iPadOS 13.0+, Mac Catalyst 13.1+, macOS 10.15+, tvOS 13.0+, visionOS 1.0+.
    #[serde(rename = "apple3")]
    Apple3,
    /// MTLGPUFamily.apple4.
    /// Represents the Apple family 4 GPU features that correspond to the Apple A11 GPUs and later.
    /// Metal version: Metal.
    /// iOS 13.0+, iPadOS 13.0+, Mac Catalyst 13.1+, macOS 10.15+, tvOS 13.0+, visionOS 1.0+.
    #[serde(rename = "apple4")]
    Apple4,
    /// MTLGPUFamily.apple5.
    /// Represents the Apple family 5 GPU features that correspond to the Apple A12 GPUs and later.
    /// Metal version: Metal.
    /// iOS 13.0+, iPadOS 13.0+, Mac Catalyst 13.1+, macOS 10.15+, tvOS 13.0+, visionOS 1.0+.
    #[serde(rename = "apple5")]
    Apple5,
    /// MTLGPUFamily.apple6.
    /// Represents the Apple family 6 GPU features that correspond to the Apple A13 GPUs and later.
    /// Metal version: Metal.
    /// iOS 13.0+, iPadOS 13.0+, Mac Catalyst 13.1+, macOS 10.15+, tvOS 13.0+, visionOS 1.0+.
    #[serde(rename = "apple6")]
    Apple6,
    /// MTLGPUFamily.apple7.
    /// Represents the Apple family 7 GPU features that correspond to the Apple A14 and M1 GPUs and later.
    /// Metal version: Metal 3 & 4.
    /// Metal 3 support in iOS 16.0+, iPadOS 16.0+, Mac Catalyst 16.0+, macOS 13.0+, tvOS 16.0+, visionOS 1.0+.
    /// Metal 4 support in iOS 26.0+, iPadOS 26.0+, Mac Catalyst 26.0+, macOS 26.0+, tvOS 26.0+, visionOS 26.0+
    #[serde(rename = "apple7")]
    Apple7,
    /// MTLGPUFamily.apple8.
    /// Represents the Apple family 8 GPU features that correspond to the Apple A15, A16, M2 GPUs and later.
    /// Metal version: Metal 3 & 4.
    /// Metal 3 support in iOS 16.0+, iPadOS 16.0+, Mac Catalyst 16.0+, macOS 13.0+, tvOS 16.0+, visionOS 1.0+.
    /// Metal 4 support in iOS 26.0+, iPadOS 26.0+, Mac Catalyst 26.0+, macOS 26.0+, tvOS 26.0+, visionOS 26.0+
    #[serde(rename = "apple8")]
    Apple8,
    /// MTLGPUFamily.apple9.
    /// Represents the Apple family 9 GPU features that correspond to the Apple A17, A18, M3, M4 GPUs and later.
    /// Metal version: Metal 3 & 4.
    /// Metal 3 support in iOS 16.0+, iPadOS 16.0+, Mac Catalyst 16.0+, macOS 13.0+, tvOS 16.0+, visionOS 1.0+.
    /// Metal 4 support in iOS 26.0+, iPadOS 26.0+, Mac Catalyst 26.0+, macOS 26.0+, tvOS 26.0+, visionOS 26.0+
    #[serde(rename = "apple9")]
    Apple9,
    /// MTLGPUFamily.apple10.
    /// Represents the Apple family 10 GPU features that correspond to the Apple A19, M5 GPUs and later.
    /// Metal version: Metal 3 & 4.
    /// Metal 3 support in iOS 16.0+, iPadOS 16.0+, Mac Catalyst 16.0+, macOS 13.0+, tvOS 16.0+, visionOS 1.0+.
    /// Metal 4 support in iOS 26.0+, iPadOS 26.0+, Mac Catalyst 26.0+, macOS 26.0+, tvOS 26.0+, visionOS 26.0+
    #[serde(rename = "apple10")]
    Apple10,
}

impl fmt::Display for AppleGpuFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppleGpuFamily::Apple2 => write!(f, "apple2"),
            AppleGpuFamily::Apple3 => write!(f, "apple3"),
            AppleGpuFamily::Apple4 => write!(f, "apple4"),
            AppleGpuFamily::Apple5 => write!(f, "apple5"),
            AppleGpuFamily::Apple6 => write!(f, "apple6"),
            AppleGpuFamily::Apple7 => write!(f, "apple7"),
            AppleGpuFamily::Apple8 => write!(f, "apple8"),
            AppleGpuFamily::Apple9 => write!(f, "apple9"),
            AppleGpuFamily::Apple10 => write!(f, "apple10"),
        }
    }
}

impl FromStr for AppleGpuFamily {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().replace('-', "_").as_str() {
            "apple2" => Ok(AppleGpuFamily::Apple2),
            "apple3" => Ok(AppleGpuFamily::Apple3),
            "apple4" => Ok(AppleGpuFamily::Apple4),
            "apple5" => Ok(AppleGpuFamily::Apple5),
            "apple6" => Ok(AppleGpuFamily::Apple6),
            "apple7" => Ok(AppleGpuFamily::Apple7),
            "apple8" => Ok(AppleGpuFamily::Apple8),
            "apple9" => Ok(AppleGpuFamily::Apple9),
            "apple10" => Ok(AppleGpuFamily::Apple10),
            other => Err(format!("Unknown or unsupported Apple Metal Family: {}", other)),
        }
    }
}

impl AppleGpuFamily {
    pub fn arch_name(&self) -> &'static str {
        match self {
            AppleGpuFamily::Apple2 => "apple2",
            AppleGpuFamily::Apple3 => "apple3",
            AppleGpuFamily::Apple4 => "apple4",
            AppleGpuFamily::Apple5 => "apple5",
            AppleGpuFamily::Apple6 => "apple6",
            AppleGpuFamily::Apple7 => "apple7",
            AppleGpuFamily::Apple8 => "apple8",
            AppleGpuFamily::Apple9 => "apple9",
            AppleGpuFamily::Apple10 => "apple10",
        }
    }

    /// Parses a string with potentially multi-value Intel Metal architectures separated by ';', ',', or '+'
    pub fn parse_multivalue(s: &str) -> Result<Vec<AppleGpuFamily>, String> {
        let mut list = Vec::new();
        for token in s.split(|c| c == ';' || c == ',' || c == '+').map(|t| t.trim()).filter(|t| !t.is_empty()) {
            let clean = token.trim_matches(|c| c == '(' || c == ')' || c == '[' || c == ']');
            let arch = clean.parse::<AppleGpuFamily>()?;
            if !list.contains(&arch) {
                list.push(arch);
            }
        }
        if list.is_empty() {
            list.push(AppleGpuFamily::Apple7);
        }
        Ok(list)
    }

    /// Formats list of Metal architectures as metal_arch string (e.g. "apple7;apple8;apple9;apple10")
    pub fn format_metal_arch(list: &[AppleGpuFamily]) -> String {
        list.iter().map(|a| a.arch_name()).collect::<Vec<_>>().join(";")
    }

    /// Formats list of Metal families as metal_family string (e.g. "apple7;apple8;apple9;apple10")
    pub fn format_metal_family(list: &[AppleGpuFamily]) -> String {
        Self::format_metal_arch(list)
    }
}

/// Represents the software-facing Metal Programming Model (Feature Tier).
/// Used during early compilation phases to restrict or expose API feature targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MetalVersion {
    /// Legacy programming model tier for older devices.
    #[serde(rename = "metal")]
    LegacyMetal,
    /// Combined Metal 1, 2, and 3 API surface. Available on Apple A14, M1 and later GPU families.
    /// MTLGPUFamily.metal3
    /// Represents the Metal 3 features.
    /// iOS 16.0+, iPadOS 16.0+, Mac Catalyst 16.0+, macOS 13.0+, tvOS 16.0+, visionOS 1.0+.
    #[serde(rename = "metal3")]
    Metal3,
    /// Modern feature tier introducing native Machine Learning encoding and Tensors (Apple14+).
    /// Combined Metal 1, 2, 3 and 4 API surface. Available on Apple A14, M1 and later GPU families.
    /// iOS 26.0+, iPadOS 26.0+, Mac Catalyst 26.0+, macOS 26.0+, tvOS 26.0+, visionOS 26.0+.
    #[serde(rename = "metal4")]
    Metal4,
}

impl fmt::Display for MetalVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetalVersion::LegacyMetal => write!(f, "metal"),
            MetalVersion::Metal3 => write!(f, "metal3"),
            MetalVersion::Metal4 => write!(f, "metal4"),
        }
    }
}

impl FromStr for MetalVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().replace('-', "_").as_str() {
            "metal" => Ok(MetalVersion::LegacyMetal),
            "metal3" => Ok(MetalVersion::Metal3),
            "metal4" => Ok(MetalVersion::Metal4),
            other => Err(format!("Unknown or unsupported Metal Programming Model: {}", other)),
        }
    }
}

impl MetalVersion {
    pub fn arch_name(&self) -> &'static str {
        match self {
            MetalVersion::LegacyMetal => "metal",
            MetalVersion::Metal3 => "metal3",
            MetalVersion::Metal4 => "metal4",
        }
    }

    /// Parses a string into MetalVersion
    pub fn parse_multivalue(s: &str) -> Result<Vec<MetalVersion>, String> {
        let mut list = Vec::new();
        for token in s.split(|c| c == ';' || c == ',' || c == '+').map(|t| t.trim()).filter(|t| !t.is_empty()) {
            let clean = token.trim_matches(|c| c == '(' || c == ')' || c == '[' || c == ']');
            let arch = clean.parse::<MetalVersion>()?;
            if !list.contains(&arch) {
                list.push(arch);
            }
        }
        if list.is_empty() {
            list.push(MetalVersion::Metal4);
        }
        Ok(list)
    }
}

/// Supported CPU Architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub enum Arch {
    #[default]
    Native,
    #[serde(rename = "x86_64")]
    X86_64,
    #[serde(rename = "aarch64")]
    Arm64,
    #[serde(rename = "arm64ec")]
    Arm64EC,
    #[serde(rename = "arm64e")]
    Arm64E,
    #[serde(rename = "riscv64")]
    Riscv64,
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Arch::Native => write!(f, "native"),
            Arch::X86_64 => write!(f, "x86_64"),
            Arch::Arm64 => write!(f, "aarch64"),
            Arch::Arm64EC => write!(f, "arm64ec"),
            Arch::Arm64E => write!(f, "arm64e"),
            Arch::Riscv64 => write!(f, "riscv64"),
        }
    }
}

impl FromStr for Arch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().replace('-', "_").as_str() {
            "native" | "host" | "current" => Ok(Arch::Native),
            "x86_64" | "x64" | "amd64" => Ok(Arch::X86_64),
            "arm64" | "aarch64" => Ok(Arch::Arm64),
            "arm64ec" => Ok(Arch::Arm64EC),
            "arm64e" => Ok(Arch::Arm64E),
            "riscv64" | "riscv" | "rv64" => Ok(Arch::Riscv64),
            other => Err(format!("Unknown architecture: {}", other)),
        }
    }
}

impl Arch {
    /// Resolves Arch::Native to current host architecture
    pub fn resolve(self) -> Self {
        match self {
            Arch::Native => Arch::current(),
            other => other,
        }
    }

    /// Detects current host architecture at compile-time/runtime
    pub fn current() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            Arch::X86_64
        }
        #[cfg(target_arch = "aarch64")]
        {
            Arch::Arm64
        }
        #[cfg(target_arch = "riscv64")]
        {
            Arch::Riscv64
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
        {
            Arch::X86_64
        }
    }

    /// Returns the list of all concrete supported architectures
    pub fn all() -> &'static [Arch] {
        &[Arch::X86_64, Arch::Arm64, Arch::Riscv64]
    }
}

/// Supported GPU Architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub enum GPUArch {
    #[default]
    Native,
    /// NVIDIA CUDA
    #[serde(rename = "cuda")]
    Cuda,
    /// AMD ROCm
    #[serde(rename = "rocm")]
    Rocm,
    /// Intel GPU
    #[serde(rename = "xpu")]
    Xpu,
    /// APPLE Metal
    #[serde(rename = "metal")]
    Metal,
}

impl fmt::Display for GPUArch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GPUArch::Native => write!(f, "native"),
            GPUArch::Cuda => write!(f, "cuda"),
            GPUArch::Rocm => write!(f, "rocm"),
            GPUArch::Xpu => write!(f, "xpu"),
            GPUArch::Metal => write!(f, "metal")
        }
    }
}

impl FromStr for GPUArch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().replace('-', "_").as_str() {
            "native" | "host" | "current" => Ok(GPUArch::Native),
            "cuda" => Ok(GPUArch::Cuda),
            "rocm" => Ok(GPUArch::Rocm),
            "xpu" => Ok(GPUArch::Xpu),
            "metal" => Ok(GPUArch::Metal),
            other => Err(format!("Unknown architecture: {}", other)),
        }
    }
}

impl GPUArch {
    pub fn arch_name(&self) -> &'static str {
        match self {
            GPUArch::Native => "native",
            GPUArch::Cuda => "cuda",
            GPUArch::Rocm => "rocm",
            GPUArch::Xpu => "xpu",
            GPUArch::Metal => "metal",
        }
    }

    /// Parses a string with potentially multi-value GPU architectures separated by ';', ',', or '+'
    pub fn parse_multivalue(s: &str) -> Result<Vec<GPUArch>, String> {
        let mut list = Vec::new();
        for token in s.split(|c| c == ';' || c == ',' || c == '+').map(|t| t.trim()).filter(|t| !t.is_empty()) {
            let clean = token.trim_matches(|c| c == '(' || c == ')' || c == '[' || c == ']');
            let arch = clean.parse::<GPUArch>()?;
            if !list.contains(&arch) {
                list.push(arch);
            }
        }
        if list.contains(&GPUArch::Native) && list.len() > 1 {
            return Err("Native GPU architecture cannot be combined with other GPU architectures".to_string());
        }
        Ok(list)
    }

    /// Formats list of GPU architectures as gpu_arch string (e.g. "cuda;rocm")
    pub fn format_gpu_arch(list: &[GPUArch]) -> String {
        list.iter().map(|a| a.arch_name()).collect::<Vec<_>>().join(";")
    }

    /// Resolves `Native` using purely user-space dynamic loader checks
    pub fn resolve(self) -> Option<Self> {
        match self {
            GPUArch::Native => Self::detect(),
            other => Some(other),
        }
    }

    /// User-space detection: attempts to load vendor user-mode compute runtimes
    pub fn detect() -> Option<Self> {
        // 1. macOS Metal is pure user-space system framework
        #[cfg(target_os = "macos")]
        {
            return Some(GPUArch::Metal);
            /* if Self::probe_dylib("Metal.framework/Metal") {
                return Some(GPUArch::Metal);
            } */
        }

        // 2. CUDA user-mode driver API
        #[cfg(target_os = "windows")]
        let cuda_candidates = &["nvcuda.dll"];
        #[cfg(target_os = "linux")]
        let cuda_candidates = &[
            "libcuda.so.1",
            "libcuda.so",
            "/usr/lib/x86_64-linux-gnu/libcuda.so.1",
            "/usr/lib64/libcuda.so.1",
            "/usr/local/cuda/lib64/libcuda.so",
        ];

        #[cfg(any(target_os = "windows", target_os = "linux"))]
        if cuda_candidates.iter().any(|&lib| Self::probe_dylib(lib)) {
            return Some(GPUArch::Cuda);
        }

        // 3. AMD ROCm / HIP user-mode runtime
        #[cfg(target_os = "windows")]
        let rocm_candidates = &["amdhip64.dll", "amdhip64_6.dll"];
        #[cfg(target_os = "linux")]
        let rocm_candidates = &[
            "libamdhip64.so",
            "libamdhip64.so.6",
            "libamdhip64.so.5",
            "/opt/rocm/lib/libamdhip64.so",
            "/opt/rocm/hip/lib/libamdhip64.so",
        ];

        #[cfg(any(target_os = "windows", target_os = "linux"))]
        if rocm_candidates.iter().any(|&lib| Self::probe_dylib(lib)) {
            return Some(GPUArch::Rocm);
        }

        // 4. Intel OneAPI / Level Zero user-mode loader
        #[cfg(target_os = "windows")]
        let xpu_candidates = &["ze_loader.dll"];
        #[cfg(target_os = "linux")]
        let xpu_candidates = &[
            "libze_loader.so.1",
            "libze_loader.so",
            "/usr/lib/x86_64-linux-gnu/libze_loader.so.1",
        ];

        #[cfg(any(target_os = "windows", target_os = "linux"))]
        if xpu_candidates.iter().any(|&lib| Self::probe_dylib(lib)) {
            return Some(GPUArch::Xpu);
        }

        None
    }

    pub const fn all() -> &'static [GPUArch] {
        &[GPUArch::Cuda, GPUArch::Rocm, GPUArch::Xpu, GPUArch::Metal]
    }

    /// Checks if a dynamic library can be loaded in user space without crashing
    fn probe_dylib(name: &str) -> bool {
        #[cfg(target_family = "unix")]
        {
            let c_name = match CString::new(name) {
                Ok(s) => s,
                Err(_) => return false,
            };

            unsafe {
                // RTLD_LAZY: resolve symbols only on demand
                // RTLD_LOCAL: keep symbols private to this handle
                let handle = libc::dlopen(c_name.as_ptr(), libc::RTLD_LAZY | libc::RTLD_LOCAL);
                if !handle.is_null() {
                    libc::dlclose(handle);
                    true
                } else {
                    false
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::ffi::OsStrExt;
            let wide: Vec<u16> = std::ffi::OsStr::new(name)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            unsafe extern "system" {
                fn LoadLibraryW(lpLibFileName: *const u16) -> *mut std::ffi::c_void;
                fn FreeLibrary(hLibModule: *mut std::ffi::c_void) -> i32;
                fn SetErrorMode(uMode: u32) -> u32;
            }

            const SEM_FAILCRITICALERRORS: u32 = 0x0001;

            unsafe {
                // Suppress Windows modal error dialogs if missing indirect DLL dependencies
                let prev_mode = SetErrorMode(SEM_FAILCRITICALERRORS);
                let handle = LoadLibraryW(wide.as_ptr());
                SetErrorMode(prev_mode);

                if !handle.is_null() {
                    FreeLibrary(handle);
                    true
                } else {
                    false
                }
            }
        }

        #[cfg(not(any(unix, windows)))]
        {
            let _ = name;
            false
        }
    }
}

/// Target Platforms supported by Unreal Engine and modern gaming / runtime environments
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    #[default]
    Native,
    Android,
    Windows,
    Linux,
    Freebsd,
    Macosx,
    Ios,
    Tvos,
    Xros,
    Xboxone,
    Xboxxs,
    Ps4,
    Ps5,
    #[serde(alias = "nx2")]
    Switch2,
    Steamdeck,
    Steammachine,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::Native => write!(f, "native"),
            Platform::Android => write!(f, "android"),
            Platform::Windows => write!(f, "windows"),
            Platform::Linux => write!(f, "linux"),
            Platform::Freebsd => write!(f, "freebsd"),
            Platform::Macosx => write!(f, "macosx"),
            Platform::Ios => write!(f, "ios"),
            Platform::Tvos => write!(f, "tvos"),
            Platform::Xros => write!(f, "xros"),
            Platform::Xboxone => write!(f, "xboxone"),
            Platform::Xboxxs => write!(f, "xboxxs"),
            Platform::Ps4 => write!(f, "ps4"),
            Platform::Ps5 => write!(f, "ps5"),
            Platform::Switch2 => write!(f, "switch2"),
            Platform::Steamdeck => write!(f, "steamdeck"),
            Platform::Steammachine => write!(f, "steammachine"),
        }
    }
}

impl FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "native" | "host" | "current" => Ok(Platform::Native),
            "android" => Ok(Platform::Android),
            "windows" | "win64" | "win" => Ok(Platform::Windows),
            "linux" => Ok(Platform::Linux),
            "freebsd" => Ok(Platform::Freebsd),
            "macosx" | "macos" | "mac" | "darwin" => Ok(Platform::Macosx),
            "ios" => Ok(Platform::Ios),
            "tvos" => Ok(Platform::Tvos),
            "xros" | "visionos" => Ok(Platform::Xros),
            "xboxone" | "xb1" => Ok(Platform::Xboxone),
            "xboxxs" | "xboxseriesx" | "xboxseries" => Ok(Platform::Xboxxs),
            "ps4" | "playstation4" => Ok(Platform::Ps4),
            "ps5" | "playstation5" => Ok(Platform::Ps5),
            "switch2" | "nx2" => Ok(Platform::Switch2),
            "steamdeck" => Ok(Platform::Steamdeck),
            "steammachine" => Ok(Platform::Steammachine),
            other => Err(format!("Unknown platform: {}", other)),
        }
    }
}

impl Platform {
    /// Alias for `Platform::Switch2`
    #[allow(non_upper_case_globals)]
    pub const Nx2: Platform = Platform::Switch2;

    /// Resolves Platform::Native to current host platform
    pub fn resolve(self) -> Self {
        match self {
            Platform::Native => Platform::current(),
            other => other,
        }
    }

    /// Detects current host platform
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        {
            Platform::Windows
        }
        #[cfg(target_os = "linux")]
        {
            Platform::Linux
        }
        #[cfg(target_os = "macos")]
        {
            Platform::Macosx
        }
        #[cfg(target_os = "freebsd")]
        {
            Platform::Freebsd
        }
        #[cfg(target_os = "android")]
        {
            Platform::Android
        }
        #[cfg(target_os = "ios")]
        {
            Platform::Ios
        }
        #[cfg(not(any(
            target_os = "windows",
            target_os = "linux",
            target_os = "macos",
            target_os = "freebsd",
            target_os = "android",
            target_os = "ios"
        )))]
        {
            Platform::Windows
        }
    }

    /// Returns whether the specified architecture is compatible with this platform
    pub fn is_arch_compatible(&self, arch: Arch) -> bool {
        let p = self.resolve();
        let a = arch.resolve();
        match p {
            Platform::Native => unreachable!(),
            Platform::Xboxone | Platform::Xboxxs | Platform::Ps4 | Platform::Ps5
            | Platform::Steamdeck | Platform::Steammachine => a == Arch::X86_64,
            Platform::Switch2 | Platform::Tvos | Platform::Xros => a == Arch::Arm64,
            Platform::Ios => matches!(a, Arch::Arm64 | Arch::Arm64E),
            Platform::Windows => {
                matches!(a, Arch::X86_64 | Arch::Arm64 | Arch::Arm64EC)
            }
            Platform::Macosx => {
                matches!(a, Arch::X86_64 | Arch::Arm64 | Arch::Arm64E)
            }

            Platform::Android | Platform::Linux | Platform::Freebsd => {
                matches!(a, Arch::X86_64 | Arch::Arm64 | Arch::Riscv64)
            }
        }
    }

    /// Returns whether this platform is an Apple platform (macOS, iOS, tvOS, xrOS)
    pub fn is_apple(&self) -> bool {
        matches!(
            self.resolve(),
            Platform::Macosx | Platform::Ios | Platform::Tvos | Platform::Xros
        )
    }

    /// Returns whether the specified Arm64 target is compatible with this platform
    pub fn is_target_arm64_compatible(&self, target: TargetCpuArchitectureArm64) -> bool {
        target.is_platform_compatible(*self)
    }

    /// Returns the list of all supported platforms
    pub fn all() -> &'static [Platform] {
        &[
            Platform::Android,
            Platform::Windows,
            Platform::Linux,
            Platform::Freebsd,
            Platform::Macosx,
            Platform::Ios,
            Platform::Tvos,
            Platform::Xros,
            Platform::Xboxone,
            Platform::Xboxxs,
            Platform::Ps4,
            Platform::Ps5,
            Platform::Switch2,
            Platform::Steamdeck,
            Platform::Steammachine,
        ]
    }

    /// Returns default preferred vector length for platforms with dedicated hardware vector width characteristics.
    /// PS5 (AMD Zen 2 custom APU) defaults to 128-bit vector length (vl128) because FP3 was deleted and FP2 stripped.
    pub fn default_vector_length(&self) -> Option<CpuArchitectureVectorLength> {
        match self.resolve() {
            Platform::Macosx | Platform::Ios | Platform::Tvos | Platform::Xros | Platform::Xboxone | Platform::Ps4
            | Platform::Ps5 | Platform::Switch2 | Platform::Steamdeck => Some(CpuArchitectureVectorLength::VL128),
            Platform::Xboxxs | Platform::Steammachine => Some(CpuArchitectureVectorLength::VL256),
            _ => None,
        }
    }

    /// Returns whether this platform is a console platform
    pub fn is_console(&self) -> bool {
        matches!(
            self.resolve(),
            Platform::Xboxone
                | Platform::Xboxxs
                | Platform::Ps4
                | Platform::Ps5
                | Platform::Switch2
                | Platform::Steamdeck
                | Platform::Steammachine
        )
    }

    /// Returns the target CPU for console platforms, which is the same name as the console platform
    pub fn console_target_cpu(&self) -> Option<&'static str> {
        match self.resolve() {
            Platform::Xboxone => Some("xboxone"),
            Platform::Xboxxs => Some("xboxxs"),
            Platform::Ps4 => Some("ps4"),
            Platform::Ps5 => Some("ps5"),
            Platform::Switch2 => Some("switch2"),
            Platform::Steamdeck => Some("steamdeck"),
            Platform::Steammachine => Some("steammachine"),
            _ => None,
        }
    }

    /// Returns whether this platform uses MSVC tooling (Windows, Xbox One, Xbox Series X/S)
    pub fn uses_msvc(&self) -> bool {
        matches!(
            self.resolve(),
            Platform::Windows | Platform::Xboxone | Platform::Xboxxs
        )
    }

    /// Returns whether NVIDIA CUDA is compatible with this platform and CPU architecture
    pub fn is_cuda_compatible(&self, arch: Arch) -> bool {
        is_cuda_compatible(*self, arch)
    }

    /// Returns whether AMD ROCm is compatible with this platform and CPU architecture
    pub fn is_rocm_compatible(&self, arch: Arch) -> bool {
        is_rocm_compatible(*self, arch)
    }

    /// Returns whether Intel XPU is compatible with this platform and CPU architecture
    pub fn is_xpu_compatible(&self, arch: Arch) -> bool {
        is_xpu_compatible(*self, arch)
    }

    /// Returns whether Apple Metal is compatible with this platform and CPU architecture
    pub fn is_metal_compatible(&self, arch: Arch) -> bool {
        is_metal_compatible(*self, arch)
    }
}

/// Returns whether NVIDIA CUDA is compatible with the specified platform and CPU architecture.
/// CUDA is only available for Windows and Linux in x86_64 and or aarch64 (not consoles).
pub fn is_cuda_compatible(platform: Platform, arch: Arch) -> bool {
    let p = platform.resolve();
    let a = arch.resolve();
    matches!(p, Platform::Windows | Platform::Linux)
        && !p.is_console()
        && matches!(a, Arch::X86_64 | Arch::Arm64 | Arch::Arm64EC)
}

/// Returns whether AMD ROCm is compatible with the specified platform and CPU architecture.
/// ROCm is only available for Windows and Linux in x86_64 (not consoles).
pub fn is_rocm_compatible(platform: Platform, arch: Arch) -> bool {
    let p = platform.resolve();
    let a = arch.resolve();
    matches!(p, Platform::Windows | Platform::Linux)
        && !p.is_console()
        && a == Arch::X86_64
}

/// Returns whether Intel XPU is compatible with the specified platform and CPU architecture.
/// XPU is only available for Windows and Linux in x86_64 (not consoles).
pub fn is_xpu_compatible(platform: Platform, arch: Arch) -> bool {
    let p = platform.resolve();
    let a = arch.resolve();
    matches!(p, Platform::Windows | Platform::Linux)
        && !p.is_console()
        && a == Arch::X86_64
}

/// Returns whether Apple Metal is compatible with the specified platform and CPU architecture.
/// Metal is only available for Apple products (Macosx, Ios, Tvos, Xros) in aarch64.
pub fn is_metal_compatible(platform: Platform, arch: Arch) -> bool {
    let p = platform.resolve();
    let a = arch.resolve();
    p.is_apple() && matches!(a, Arch::Arm64 | Arch::Arm64E)
}

