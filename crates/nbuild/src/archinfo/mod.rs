// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/lib.rs
// created: 2026-10-14
// lastModified: 2026-10-14

#![allow(non_camel_case_types, non_snake_case, dead_code, unused_imports)]

pub mod arch;
pub mod json_output;
pub mod platform;
pub mod profiles;
pub mod vector_length;

pub use arch::arm64::{
    Arm64CPUFeatures, Arm64ISA, ClangArm64ISANames, ClangArm64NOISANames,
    ClangTargetCpuArchitectureArm64ISANames, ClangTargetCpuArchitectureArm64NOISANames,
    MSVCArm64ISANames, MSVCTargetCpuArchitectureArm64ISANames,
    MinimumCpuArchitectureArm64, MinimumCpuArchitectureArm64ClangNames,
    MinimumCpuArchitectureArm64MSVCNames, TargetCpuArchitectureArm64,
    TargetCpuArchitectureArm64Names,
};
pub use arch::riscv64::{
    ClangRiscv64ISANames, ClangRiscv64NOISANames, ClangTargetCpuArchitectureRiscv64ISANames,
    Riscv64CPUFeatures, Riscv64ISA, TargetCpuArchitectureRiscv64, TargetCpuArchitectureRiscv64Names,
};
pub use arch::x86_64::{
    ClangTargetCpuArchitectureX64ISANames, ClangTargetCpuArchitectureX64NOISANames,
    ClangX64ISANames, ClangX64NOISANames, ClangX64VLen, MSVCX64ArchTarget, MSVCX64ISANames,
    MSVCX64VLen, MinimumCpuArchitectureX64, TargetCpuArchitectureX64,
    TargetCpuArchitectureX64Names, X64CPUFeatures, X64ISA,
};
pub use arch::CPUFeatures;
pub use json_output::{get_default_output_dir, ArchFeaturesReport, PlatformArchMatrixReport};
pub use platform::{
    is_cuda_compatible, is_metal_compatible, is_rocm_compatible, is_xpu_compatible, AppleGpuFamily,
    Arch, CudaArch, GPUArch, MetalVersion, Platform, RocmArch, XpuArch,
};
pub use profiles::TargetProfile;
pub use vector_length::{
    ClangCpuArchitecturePreferredVectorLength, ClangCpuArchitectureVectorLengthNames,
    CpuArchitectureVectorLength, MSVCCpuArchitecturePreferredVectorLength,
};

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// C-FFI: Get default output folder path
#[unsafe(no_mangle)]
pub extern "C" fn archinfo_get_default_output_dir() -> *mut c_char {
    let dir = get_default_output_dir();
    let s = dir.to_string_lossy();
    CString::new(s.as_bytes()).unwrap().into_raw()
}

/// C-FFI: Detect host features and return JSON string
#[unsafe(no_mangle)]
pub extern "C" fn archinfo_detect_host_json() -> *mut c_char {
    let features = CPUFeatures::detect_host();
    let report = ArchFeaturesReport::from_host(&features);
    match report.to_json() {
        Ok(json) => CString::new(json).unwrap().into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// C-FFI: Query target platform + architecture features JSON
#[unsafe(no_mangle)]
pub extern "C" fn archinfo_query_target_json(platform_str: *const c_char, arch_str: *const c_char) -> *mut c_char {
    if platform_str.is_null() || arch_str.is_null() {
        return std::ptr::null_mut();
    }

    let p_str = unsafe { CStr::from_ptr(platform_str).to_string_lossy() };
    let a_str = unsafe { CStr::from_ptr(arch_str).to_string_lossy() };

    let platform = match p_str.parse::<Platform>() {
        Ok(p) => p,
        Err(_) => return std::ptr::null_mut(),
    };

    let arch = match a_str.parse::<Arch>() {
        Ok(a) => a,
        Err(_) => return std::ptr::null_mut(),
    };

    match ArchFeaturesReport::from_target(platform, arch) {
        Ok(report) => match report.to_json() {
            Ok(json) => CString::new(json).unwrap().into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// C-FFI: Evaluate platform, arch, and target_cpu JSON
#[unsafe(no_mangle)]
pub extern "C" fn archinfo_evaluate_json(
    platform_str: *const c_char,
    arch_str: *const c_char,
    target_cpu_str: *const c_char,
) -> *mut c_char {
    let platform = if platform_str.is_null() {
        None
    } else {
        let s = unsafe { CStr::from_ptr(platform_str).to_string_lossy() };
        s.parse::<Platform>().ok()
    };

    let arch = if arch_str.is_null() {
        None
    } else {
        let s = unsafe { CStr::from_ptr(arch_str).to_string_lossy() };
        s.parse::<Arch>().ok()
    };

    let target_cpu = if target_cpu_str.is_null() {
        None
    } else {
        let s = unsafe { CStr::from_ptr(target_cpu_str).to_string_lossy() };
        Some(s.to_string())
    };

    match ArchFeaturesReport::evaluate(platform, arch, target_cpu.as_deref()) {
        Ok(report) => match report.to_json() {
            Ok(json) => CString::new(json).unwrap().into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// C-FFI: Free string allocated by ArchInfo
#[unsafe(no_mangle)]
pub extern "C" fn archinfo_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}
