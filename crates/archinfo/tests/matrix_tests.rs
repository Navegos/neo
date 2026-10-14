// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/tests/matrix_tests.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use archinfo::*;

#[test]
fn test_platform_architecture_matrix() {
    assert!(Platform::Windows.is_arch_compatible(Arch::X86_64));
    assert!(Platform::Windows.is_arch_compatible(Arch::Arm64));
    assert!(!Platform::Windows.is_arch_compatible(Arch::Riscv64));

    assert!(Platform::Linux.is_arch_compatible(Arch::X86_64));
    assert!(Platform::Linux.is_arch_compatible(Arch::Arm64));
    assert!(Platform::Linux.is_arch_compatible(Arch::Riscv64));

    assert!(Platform::Android.is_arch_compatible(Arch::X86_64));
    assert!(Platform::Android.is_arch_compatible(Arch::Arm64));
    assert!(Platform::Android.is_arch_compatible(Arch::Riscv64));

    assert!(Platform::Freebsd.is_arch_compatible(Arch::X86_64));
    assert!(Platform::Freebsd.is_arch_compatible(Arch::Arm64));
    assert!(Platform::Freebsd.is_arch_compatible(Arch::Riscv64));

    assert!(Platform::Macosx.is_arch_compatible(Arch::X86_64));
    assert!(Platform::Macosx.is_arch_compatible(Arch::Arm64));

    assert!(Platform::Ios.is_arch_compatible(Arch::Arm64));
    assert!(!Platform::Ios.is_arch_compatible(Arch::X86_64));

    assert!(Platform::Tvos.is_arch_compatible(Arch::Arm64));
    assert!(Platform::Xros.is_arch_compatible(Arch::Arm64));

    assert!(Platform::Nx2.is_arch_compatible(Arch::Arm64));
    assert!(Platform::Switch2.is_arch_compatible(Arch::Arm64));
    assert!(!Platform::Switch2.is_arch_compatible(Arch::X86_64));

    assert!(Platform::Ps4.is_arch_compatible(Arch::X86_64));
    assert!(!Platform::Ps4.is_arch_compatible(Arch::Arm64));

    assert!(Platform::Ps5.is_arch_compatible(Arch::X86_64));
    assert!(!Platform::Ps5.is_arch_compatible(Arch::Arm64));

    assert!(Platform::Xboxone.is_arch_compatible(Arch::X86_64));
    assert!(Platform::Xboxxs.is_arch_compatible(Arch::X86_64));

    assert!(Platform::Steamdeck.is_arch_compatible(Arch::X86_64));
    assert!(!Platform::Steamdeck.is_arch_compatible(Arch::Arm64));

    assert!(Platform::Steammachine.is_arch_compatible(Arch::X86_64));
}

#[test]
fn test_evaluate_zero_arguments() {
    // When no arguments are passed (None, None, None), ArchInfo defaults to native host detection
    let report = ArchFeaturesReport::evaluate(None, None, None).unwrap();
    assert_eq!(report.platform, Platform::current().to_string());
    assert_eq!(report.arch, Arch::current().to_string());
    assert_eq!(report.target_cpu, Some("native".to_string()));
    assert!(!report.extensions.is_empty());
    assert!(!report.features.is_empty());
}

#[test]
fn test_native_platform_and_arch() {
    // Parsing "native", "host", "current"
    assert_eq!("native".parse::<Platform>().unwrap(), Platform::Native);
    assert_eq!("host".parse::<Platform>().unwrap(), Platform::Native);
    assert_eq!("current".parse::<Platform>().unwrap(), Platform::Native);

    assert_eq!("native".parse::<Arch>().unwrap(), Arch::Native);
    assert_eq!("host".parse::<Arch>().unwrap(), Arch::Native);
    assert_eq!("current".parse::<Arch>().unwrap(), Arch::Native);

    // Resolving returns current host
    assert_eq!(Platform::Native.resolve(), Platform::current());
    assert_eq!(Arch::Native.resolve(), Arch::current());

    // Compatibility checks with Native
    assert!(Platform::Native.is_arch_compatible(Arch::Native));
    assert!(Platform::Native.is_arch_compatible(Arch::current()));
    assert!(Platform::current().is_arch_compatible(Arch::Native));

    // Both native: -p native -a native
    let report = ArchFeaturesReport::evaluate(Some(Platform::Native), Some(Arch::Native), None).unwrap();
    assert_eq!(report.platform, Platform::current().to_string());
    assert_eq!(report.arch, Arch::current().to_string());
    assert_eq!(report.target_cpu, Some("native".to_string()));

    // Native platform only: -p native
    let report_p = ArchFeaturesReport::evaluate(Some(Platform::Native), None, None).unwrap();
    assert_eq!(report_p.platform, Platform::current().to_string());
    assert_eq!(report_p.arch, Arch::current().to_string());

    // Native arch only: -a native
    let report_a = ArchFeaturesReport::evaluate(None, Some(Arch::Native), None).unwrap();
    assert_eq!(report_a.platform, Platform::current().to_string());
    assert_eq!(report_a.arch, Arch::current().to_string());

    // Host platform + native arch
    let report_hp = ArchFeaturesReport::evaluate(Some(Platform::current()), Some(Arch::Native), None).unwrap();
    assert_eq!(report_hp.platform, Platform::current().to_string());
    assert_eq!(report_hp.arch, Arch::current().to_string());

    // Native platform + host arch
    let report_ha = ArchFeaturesReport::evaluate(Some(Platform::Native), Some(Arch::current()), None).unwrap();
    assert_eq!(report_ha.platform, Platform::current().to_string());
    assert_eq!(report_ha.arch, Arch::current().to_string());

    // from_target with Native
    let report_target = ArchFeaturesReport::from_target(Platform::Native, Arch::Native).unwrap();
    assert_eq!(report_target.platform, Platform::current().to_string());
    assert_eq!(report_target.arch, Arch::current().to_string());
}

#[test]
fn test_default_output_dir() {
    let dir = get_default_output_dir();
    let dir_str = dir.to_string_lossy();
    assert!(dir_str.contains("ArchInfo"));

    let report = ArchFeaturesReport::evaluate(Some(Platform::Windows), Some(Arch::X86_64), Some("native")).unwrap();
    let file_path = report.default_filepath();
    let path_str = file_path.to_string_lossy();
    assert!(path_str.contains("ArchInfo"));
    assert!(path_str.ends_with(".json"));
}

#[test]
fn test_canonical_filenames() {
    let sw2 = ArchFeaturesReport::from_target(Platform::Switch2, Arch::Arm64).unwrap();
    assert_eq!(sw2.filename(), "switch2-aarch64-cortex-a78c-armv8.4-a-vl128.json");

    let ps5 = ArchFeaturesReport::from_target(Platform::Ps5, Arch::X86_64).unwrap();
    assert_eq!(ps5.filename(), "ps5-x86_64-znver2-avx2-vl128.json");

    let steamdeck = ArchFeaturesReport::from_target(Platform::Steamdeck, Arch::X86_64).unwrap();
    assert_eq!(steamdeck.filename(), "steamdeck-x86_64-znver2-avx2-vl128-glibc-2.17.json");

    let ps4 = ArchFeaturesReport::from_target(Platform::Ps4, Arch::X86_64).unwrap();
    assert_eq!(ps4.filename(), "ps4-x86_64-btver2-avx-vl128.json");

    let m4 = ArchFeaturesReport::evaluate(Some(Platform::Macosx), Some(Arch::Arm64), Some("apple-m4")).unwrap();
    assert_eq!(m4.filename(), "macosx-aarch64-apple-m4-armv9.2-a-vl128-macosx-15.0.json");
}

#[test]
fn test_evaluate_native_detection() {
    let report = ArchFeaturesReport::evaluate(Some(Platform::Windows), Some(Arch::X86_64), Some("native")).unwrap();
    assert_eq!(report.platform, "windows");
    assert_eq!(report.arch, "x86_64");
    assert_eq!(report.target_cpu, Some("native".to_string()));
    assert!(!report.extensions.is_empty());
    assert!(report.extensions.contains("sse"));
    assert!(!report.features.is_empty());
}

#[test]
fn test_evaluate_generic_and_known_targets_x64() {
    // Generic default (None / SSE4.2 -> x86-64-v2)
    let gen_report = ArchFeaturesReport::evaluate(Some(Platform::Linux), Some(Arch::X86_64), Some("generic")).unwrap();
    assert_eq!(gen_report.target_cpu, Some("x86-64-v2".to_string()));
    assert!(gen_report.extensions.contains("sse4.2") || gen_report.extensions.contains("sse4_2"));
    assert_eq!(gen_report.features.get("sse4_2"), Some(&true));
    assert_eq!(gen_report.min_cpu_arch, Some("sse4.2".to_string()));
    assert_eq!(gen_report.filename(), "linux-x86_64-generic-x86-64-v2-sse4.2-vl128-glibc-2.17.json");

    // Znver3
    let zen3_report = ArchFeaturesReport::evaluate(Some(Platform::Linux), Some(Arch::X86_64), Some("znver3")).unwrap();
    assert_eq!(zen3_report.target_cpu, Some("znver3".to_string()));
    assert!(zen3_report.extensions.contains("avx2"));
    assert!(zen3_report.extensions.contains("vaes"));
    assert_eq!(zen3_report.features.get("avx2"), Some(&true));
    assert_eq!(zen3_report.features.get("vaes"), Some(&true));
    assert_eq!(zen3_report.min_cpu_arch, Some("avx2".to_string()));
    assert_eq!(zen3_report.filename(), "linux-x86_64-znver3-avx2-vl256-glibc-2.17.json");
}

#[test]
fn test_evaluate_known_targets_arm64() {
    // Cortex-A78C (Switch 2)
    let a78c_report = ArchFeaturesReport::evaluate(Some(Platform::Switch2), Some(Arch::Arm64), Some("cortex-a78c")).unwrap();
    assert_eq!(a78c_report.target_cpu, Some("cortex-a78c".to_string()));
    assert_eq!(a78c_report.target_clang_cpu, Some("-m'cpu=cortex-a78c'".to_string()));
    assert_eq!(a78c_report.target_clang_isaarch, Some("".to_string()));
    assert_eq!(a78c_report.target_msvc_arch, Some("".to_string()));
    assert_eq!(a78c_report.target_msvc_vlen, Some("".to_string()));
    assert!(a78c_report.target_clan_arch.as_ref().unwrap().starts_with("-m'arch=armv8.4-a+"));
    assert!(a78c_report.extensions.contains("pauth"));
    assert!(a78c_report.extensions.contains("dotprod"));
    assert_eq!(a78c_report.features.get("pauth"), Some(&true));
    assert_eq!(a78c_report.features.get("dotprod"), Some(&true));
    assert_eq!(a78c_report.filename(), "switch2-aarch64-cortex-a78c-armv8.4-a-vl128.json");

    // Apple M4
    let m4_report = ArchFeaturesReport::evaluate(Some(Platform::Macosx), Some(Arch::Arm64), Some("apple-m4")).unwrap();
    assert_eq!(m4_report.target_cpu, Some("apple-m4".to_string()));
    assert_eq!(m4_report.target_clang_cpu, Some("-m'cpu=apple-m4'".to_string()));
    assert_eq!(m4_report.target_clang_isaarch, Some("".to_string()));
    assert_eq!(m4_report.target_msvc_arch, Some("".to_string()));
    assert_eq!(m4_report.target_msvc_vlen, Some("".to_string()));
    let m4_json = m4_report.to_json().unwrap();
    assert!(m4_json.contains("\"target_msvc_arch\": \"\""));
    assert!(m4_json.contains("\"target_msvc_vlen\": \"\""));
    assert_eq!(
        m4_report.target_clan_arch,
        Some("-m'arch=armv9.2-a+aes+bf16+bti+crc+crypto+dit+dotprod+fcma+flagm+fp+fp16+fp16fml+i8mm+jscvt+lse+pauth+pmuv3+predres+ras+rcpc+rdm+sb+sha2+sha3+simd+sme+sme-f64f64+sme-i16i64+sme2+wfxt+nosve+nosve2'".to_string())
    );
    assert_eq!(m4_report.filename(), "macosx-aarch64-apple-m4-armv9.2-a-vl128-macosx-15.0.json");

    // Ampere1B (Linux) - non-MSVC platform has empty MSVC output
    let amp1b_report = ArchFeaturesReport::evaluate(Some(Platform::Linux), Some(Arch::Arm64), Some("ampere1b")).unwrap();
    assert_eq!(amp1b_report.target_msvc_arch, Some("".to_string()));
    assert_eq!(amp1b_report.target_msvc_vlen, Some("".to_string()));

    // Apple A10 (iOS) - non-MSVC platform has empty MSVC output
    let a10_report = ArchFeaturesReport::evaluate(Some(Platform::Ios), Some(Arch::Arm64), Some("apple-a10")).unwrap();
    assert_eq!(a10_report.target_msvc_arch, Some("".to_string()));
    assert_eq!(a10_report.target_msvc_vlen, Some("".to_string()));

    // Windows Arm64 targets produce target_msvc_arch
    let win_x4_report = ArchFeaturesReport::evaluate(Some(Platform::Windows), Some(Arch::Arm64), Some("cortex-x4")).unwrap();
    assert_eq!(win_x4_report.target_msvc_arch, Some("/arch:armv9.2+lse+rcpc".to_string()));
    assert_eq!(win_x4_report.target_msvc_vlen, Some("".to_string()));

    let win_a78c_report = ArchFeaturesReport::evaluate(Some(Platform::Windows), Some(Arch::Arm64), Some("cortex-a78c")).unwrap();
    assert_eq!(win_a78c_report.target_msvc_arch, Some("/arch:armv8.4+lse+rcpc".to_string()));
    assert_eq!(win_a78c_report.target_msvc_vlen, Some("".to_string()));

    // Apple targets are incompatible with Windows
    assert!(ArchFeaturesReport::evaluate(Some(Platform::Windows), Some(Arch::Arm64), Some("apple-m4")).is_err());
}

#[test]
fn test_evaluate_known_targets_riscv64() {
    // SiFive P470
    let p470_report = ArchFeaturesReport::evaluate(Some(Platform::Linux), Some(Arch::Riscv64), Some("sifive-p470")).unwrap();
    assert_eq!(p470_report.target_cpu, Some("sifive-p470".to_string()));
    assert_eq!(p470_report.target_clang_cpu, Some("-m'cpu=sifive-p470'".to_string()));
    assert_eq!(p470_report.target_clang_isaarch, Some("".to_string()));
    assert!(p470_report.target_clan_arch.as_ref().unwrap().starts_with("-m'arch=rv64"));
    assert!(p470_report.target_clan_arch.as_ref().unwrap().contains("_v"));
    assert!(p470_report.target_clan_arch.as_ref().unwrap().contains("_zvbb"));
    assert!(p470_report.target_clan_arch.as_ref().unwrap().contains("_zvl32b"));
    assert!(p470_report.target_clan_arch.as_ref().unwrap().contains("_zvl64b"));
    assert!(p470_report.extensions.contains("v"));
    assert!(p470_report.extensions.contains("zvbb"));
    assert!(p470_report.extensions.contains("zvl32b"));
    assert!(p470_report.extensions.contains("zvl64b"));
    assert_eq!(p470_report.features.get("v"), Some(&true));
    assert_eq!(p470_report.features.get("zvbb"), Some(&true));
    assert_eq!(p470_report.features.get("zvl32b"), Some(&true));
    assert_eq!(p470_report.features.get("zvl64b"), Some(&true));
    assert_eq!(p470_report.filename(), "linux-riscv64-sifive-p470-none-vl128-glibc-2.17.json");
}

#[test]
fn test_switch_profiles() {
    // Platform::Nx2 is an alias to Platform::Switch2
    assert_eq!(Platform::Nx2, Platform::Switch2);
    assert_eq!("nx2".parse::<Platform>().unwrap(), Platform::Switch2);
    assert_eq!("switch2".parse::<Platform>().unwrap(), Platform::Switch2);
    assert_eq!(serde_json::from_str::<Platform>("\"nx2\"").unwrap(), Platform::Switch2);
    assert_eq!(serde_json::from_str::<Platform>("\"switch2\"").unwrap(), Platform::Switch2);

    let (sw2_ext, _, sw2_arm) = TargetProfile::get_features(Platform::Switch2, Arch::Arm64).unwrap();
    assert_eq!(sw2_arm, TargetCpuArchitectureArm64::Switch2);
    assert!(sw2_ext.contains("pauth"));
    assert!(sw2_ext.contains("lse"));
    assert!(sw2_ext.contains("dotprod"));

    // Verify evaluating with nx2 alias produces switch2 in report and json output
    let report_nx2 = ArchFeaturesReport::evaluate(Some("nx2".parse().unwrap()), Some(Arch::Arm64), None).unwrap();
    let report_sw2 = ArchFeaturesReport::evaluate(Some(Platform::Switch2), Some(Arch::Arm64), None).unwrap();
    assert_eq!(report_nx2.platform, "switch2");
    assert_eq!(report_sw2.platform, "switch2");
    assert_eq!(report_nx2.to_json().unwrap(), report_sw2.to_json().unwrap());
    assert!(report_nx2.to_json().unwrap().contains("\"platform\": \"switch2\""));
}

#[test]
fn test_steamdeck_profile() {
    let (sd_ext, sd_x64, _) = TargetProfile::get_features(Platform::Steamdeck, Arch::X86_64).unwrap();
    assert_eq!(sd_x64, TargetCpuArchitectureX64::Steamdeck);
    assert!(sd_ext.contains("avx2"));
    assert!(sd_ext.contains("fma"));
    assert!(sd_ext.contains("bmi2"));
    assert!(sd_ext.contains("sha"));
}

#[test]
fn test_ps5_and_xbox_profiles() {
    let (ps5_ext, ps5_x64, _) = TargetProfile::get_features(Platform::Ps5, Arch::X86_64).unwrap();
    assert_eq!(ps5_x64, TargetCpuArchitectureX64::Ps5);
    assert!(ps5_ext.contains("avx2"));

    let (xb_ext, xb_x64, _) = TargetProfile::get_features(Platform::Xboxxs, Arch::X86_64).unwrap();
    assert_eq!(xb_x64, TargetCpuArchitectureX64::Xboxxs);
    assert!(xb_ext.contains("avx2"));
}

#[test]
fn test_json_report_generation() {
    let report = ArchFeaturesReport::from_target(Platform::Windows, Arch::X86_64).unwrap();
    assert_eq!(report.platform, "windows");
    assert_eq!(report.arch, "x86_64");
    assert!(!report.extensions.is_empty());
    assert!(report.extensions.contains("sse"));
    assert!(report.extensions.contains("aes"));
    assert_eq!(report.target_msvc_arch, Some("/arch:SSE4.2".to_string()));
    assert_eq!(report.target_clan_arch, Some("-m'arch=x86-64-v2'".to_string()));
    assert_eq!(report.target_clang_cpu, Some("".to_string()));

    let json = report.to_json().unwrap();
    assert!(json.contains("\"platform\": \"windows\""));
    assert!(json.contains("\"arch\": \"x86_64\""));
    assert!(json.contains("\"extensions\":"));
    assert!(json.contains("\"target_msvc_arch\": \"/arch:SSE4.2\""));
    assert!(json.contains("\"target_clan_arch\": \"-m'arch=x86-64-v2'\""));
    assert!(json.contains("\"target_clang_cpu\": \"\""));

    // Test PS5 / Zen 2 target has empty MSVC arch/vlen, -m'arch=znver2', default vl128 (-m'prefer-vector-width=128')
    let ps5_report = ArchFeaturesReport::from_target(Platform::Ps5, Arch::X86_64).unwrap();
    assert_eq!(ps5_report.target_msvc_arch, Some("".to_string()));
    assert_eq!(ps5_report.target_msvc_vlen, Some("".to_string()));
    let ps5_json = ps5_report.to_json().unwrap();
    assert!(ps5_json.contains("\"target_msvc_arch\": \"\""));
    assert!(ps5_json.contains("\"target_msvc_vlen\": \"\""));
    assert_eq!(ps5_report.target_clan_arch, Some("-m'arch=znver2'".to_string()));
    assert_eq!(ps5_report.target_clang_cpu, Some("".to_string()));
    assert_eq!(ps5_report.vector_length, Some("vl128".to_string()));
    assert_eq!(ps5_report.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));

    // Xbox One and Xbox Series X/S targets DO produce MSVC arch/vlen
    let xb1_report = ArchFeaturesReport::from_target(Platform::Xboxone, Arch::X86_64).unwrap();
    assert_eq!(xb1_report.target_msvc_arch, Some("/arch:AVX".to_string()));
    assert_eq!(xb1_report.target_msvc_vlen, Some("".to_string()));

    let xs_report = ArchFeaturesReport::from_target(Platform::Xboxxs, Arch::X86_64).unwrap();
    assert_eq!(xs_report.target_msvc_arch, Some("/arch:AVX2".to_string()));
    assert_eq!(xs_report.target_msvc_vlen, Some("".to_string()));
}

#[test]
fn test_full_matrix_report() {
    let matrix = PlatformArchMatrixReport::generate();
    assert!(!matrix.matrix.is_empty());

    let json = matrix.to_json().unwrap();
    assert!(json.contains("\"matrix\":"));
    assert!(json.contains("\"platform\": \"switch2\""));
    assert!(json.contains("\"platform\": \"steamdeck\""));
    assert!(json.contains("\"platform\": \"ps5\""));
    assert!(json.contains("\"platform\": \"xboxxs\""));
}

#[test]
fn test_x64_vector_lengths() {
    // Generic / SSE4.2 -> VL128
    let gen_f = X64CPUFeatures::from_target(TargetCpuArchitectureX64::Generic);
    assert_eq!(gen_f.vector_length(), CpuArchitectureVectorLength::VL128);

    // AVX only (less than AVX2) -> VL128
    let avx_f = X64CPUFeatures::from_target(TargetCpuArchitectureX64::Sandybridge);
    assert_eq!(avx_f.vector_length(), CpuArchitectureVectorLength::VL128);

    // Jaguar / BTVER2 (PS4 / XboxOne) -> VL128
    let btver2_f = X64CPUFeatures::from_target(TargetCpuArchitectureX64::Btver2);
    assert_eq!(btver2_f.vector_length(), CpuArchitectureVectorLength::VL128);

    // AVX2 (Zen 2 / Steam Deck / Xbox Series X / Haswell) -> VL256
    let avx2_f = X64CPUFeatures::from_target(TargetCpuArchitectureX64::Znver2);
    assert_eq!(avx2_f.vector_length(), CpuArchitectureVectorLength::VL256);

    // AVX-512 -> VL512
    let avx512_f = X64CPUFeatures::from_target(TargetCpuArchitectureX64::Skylake_avx512);
    assert_eq!(avx512_f.vector_length(), CpuArchitectureVectorLength::VL512);

    // AVX10.1 / AVX10.2 normal -> VL256
    let avx10_f = X64CPUFeatures::from_target(TargetCpuArchitectureX64::Diamondrapids);
    assert_eq!(avx10_f.vector_length(), CpuArchitectureVectorLength::VL256);

    // Arm64 normal -> VL128
    let arm_f = Arm64CPUFeatures::from_target(TargetCpuArchitectureArm64::Cortex_A78C);
    assert_eq!(arm_f.vector_length(), CpuArchitectureVectorLength::VL128);

    // Riscv64 normal -> VL128
    let riscv_f = Riscv64CPUFeatures::from_target(TargetCpuArchitectureRiscv64::Sifive_P470);
    assert_eq!(riscv_f.vector_length(), CpuArchitectureVectorLength::VL128);

    // Clang preferred vector length flags (-m'prefer-vector-width=...')
    assert_eq!(archinfo::ClangCpuArchitecturePreferredVectorLength::name(CpuArchitectureVectorLength::None), "");
    assert_eq!(archinfo::ClangCpuArchitecturePreferredVectorLength::name(CpuArchitectureVectorLength::VL128), "-m'prefer-vector-width=128'");
    assert_eq!(archinfo::ClangCpuArchitecturePreferredVectorLength::name(CpuArchitectureVectorLength::VL256), "-m'prefer-vector-width=256'");
    assert_eq!(archinfo::ClangCpuArchitecturePreferredVectorLength::name(CpuArchitectureVectorLength::VL512), "-m'prefer-vector-width=512'");

    // Clang vector length suffix flags (for -mavx10.1-256 / -mavx10.1-512)
    assert_eq!(archinfo::ClangCpuArchitectureVectorLengthNames::name(CpuArchitectureVectorLength::None), "");
    assert_eq!(archinfo::ClangCpuArchitectureVectorLengthNames::name(CpuArchitectureVectorLength::VL128), "");
    assert_eq!(archinfo::ClangCpuArchitectureVectorLengthNames::name(CpuArchitectureVectorLength::VL256), "-256");
    assert_eq!(archinfo::ClangCpuArchitectureVectorLengthNames::name(CpuArchitectureVectorLength::VL512), "-512");

    // MSVC preferred vector length flags (/vlen=256 / /vlen=512)
    assert_eq!(archinfo::MSVCCpuArchitecturePreferredVectorLength::name(CpuArchitectureVectorLength::None), "");
    assert_eq!(archinfo::MSVCCpuArchitecturePreferredVectorLength::name(CpuArchitectureVectorLength::VL128), "");
    assert_eq!(archinfo::MSVCCpuArchitecturePreferredVectorLength::name(CpuArchitectureVectorLength::VL256), "/vlen=256");
    assert_eq!(archinfo::MSVCCpuArchitecturePreferredVectorLength::name(CpuArchitectureVectorLength::VL512), "/vlen=512");

    // MSVC /vlen override argument resolution (MSVCX64VLen)
    assert_eq!(archinfo::MSVCX64VLen::name(MinimumCpuArchitectureX64::None, CpuArchitectureVectorLength::VL128), "");
    assert_eq!(archinfo::MSVCX64VLen::name(MinimumCpuArchitectureX64::AVX, CpuArchitectureVectorLength::VL128), "");
    assert_eq!(archinfo::MSVCX64VLen::name(MinimumCpuArchitectureX64::AVX2, CpuArchitectureVectorLength::VL256), "");
    assert_eq!(archinfo::MSVCX64VLen::name(MinimumCpuArchitectureX64::AVX512, CpuArchitectureVectorLength::VL512), "");
    assert_eq!(archinfo::MSVCX64VLen::name(MinimumCpuArchitectureX64::AVX512, CpuArchitectureVectorLength::VL256), "/vlen=256");
    assert_eq!(archinfo::MSVCX64VLen::name(MinimumCpuArchitectureX64::AVX10_1, CpuArchitectureVectorLength::VL256), "");
    assert_eq!(archinfo::MSVCX64VLen::name(MinimumCpuArchitectureX64::AVX10_1, CpuArchitectureVectorLength::VL512), "/vlen=512");
    assert_eq!(archinfo::MSVCX64VLen::name(MinimumCpuArchitectureX64::AVX10_2, CpuArchitectureVectorLength::VL256), "");
    assert_eq!(archinfo::MSVCX64VLen::name(MinimumCpuArchitectureX64::AVX10_2, CpuArchitectureVectorLength::VL512), "/vlen=512");

    // Clang prefer-vector-width override argument resolution (ClangX64VLen)
    assert_eq!(archinfo::ClangX64VLen::name(MinimumCpuArchitectureX64::None, CpuArchitectureVectorLength::VL128), "-m'prefer-vector-width=128'");
    assert_eq!(archinfo::ClangX64VLen::name(MinimumCpuArchitectureX64::AVX, CpuArchitectureVectorLength::VL128), "-m'prefer-vector-width=128'");
    assert_eq!(archinfo::ClangX64VLen::name(MinimumCpuArchitectureX64::AVX2, CpuArchitectureVectorLength::VL256), "-m'prefer-vector-width=256'");
    assert_eq!(archinfo::ClangX64VLen::name(MinimumCpuArchitectureX64::AVX2, CpuArchitectureVectorLength::VL128), "-m'prefer-vector-width=128'");
    assert_eq!(archinfo::ClangX64VLen::name(MinimumCpuArchitectureX64::AVX512, CpuArchitectureVectorLength::VL512), "-m'prefer-vector-width=512'");
    assert_eq!(archinfo::ClangX64VLen::name(MinimumCpuArchitectureX64::AVX512, CpuArchitectureVectorLength::VL256), "-m'prefer-vector-width=256'");
    assert_eq!(archinfo::ClangX64VLen::name(MinimumCpuArchitectureX64::AVX10_1, CpuArchitectureVectorLength::VL256), "-m'prefer-vector-width=256'");
    assert_eq!(archinfo::ClangX64VLen::name(MinimumCpuArchitectureX64::AVX10_1, CpuArchitectureVectorLength::VL512), "-m'prefer-vector-width=512'");
    assert_eq!(archinfo::ClangX64VLen::name(MinimumCpuArchitectureX64::AVX10_2, CpuArchitectureVectorLength::VL256), "-m'prefer-vector-width=256'");
    assert_eq!(archinfo::ClangX64VLen::name(MinimumCpuArchitectureX64::AVX10_2, CpuArchitectureVectorLength::VL512), "-m'prefer-vector-width=512'");
}

#[test]
fn test_vector_length_argument_resolution() {
    // 1. AVX10: normal is 256. If user sets 512 -> VL512. If user sets 128 -> VL128. If user sets 256 -> VL256. Otherwise VL256.
    let avx10_default = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Linux), Some(Arch::X86_64), Some("diamondrapids"), None).unwrap();
    assert_eq!(avx10_default.vector_length, Some("vl256".to_string()));

    let avx10_512 = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Linux), Some(Arch::X86_64), Some("diamondrapids"), Some(CpuArchitectureVectorLength::VL512)).unwrap();
    assert_eq!(avx10_512.vector_length, Some("vl512".to_string()));

    let avx10_256 = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Linux), Some(Arch::X86_64), Some("diamondrapids"), Some(CpuArchitectureVectorLength::VL256)).unwrap();
    assert_eq!(avx10_256.vector_length, Some("vl256".to_string()));

    let avx10_128 = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Linux), Some(Arch::X86_64), Some("diamondrapids"), Some(CpuArchitectureVectorLength::VL128)).unwrap();
    assert_eq!(avx10_128.vector_length, Some("vl128".to_string()));

    // 2. AVX-512F: normal is 512. If user sets 512 -> VL512. If user sets 256 -> VL256. If user sets 128 -> VL128. Otherwise VL512.
    let avx512_default = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Linux), Some(Arch::X86_64), Some("skylake-avx512"), None).unwrap();
    assert_eq!(avx512_default.vector_length, Some("vl512".to_string()));

    let avx512_req256 = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Linux), Some(Arch::X86_64), Some("skylake-avx512"), Some(CpuArchitectureVectorLength::VL256)).unwrap();
    assert_eq!(avx512_req256.vector_length, Some("vl256".to_string()));

    let avx512_req128 = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Linux), Some(Arch::X86_64), Some("skylake-avx512"), Some(CpuArchitectureVectorLength::VL128)).unwrap();
    assert_eq!(avx512_req128.vector_length, Some("vl128".to_string()));

    let avx512_req512 = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Linux), Some(Arch::X86_64), Some("skylake-avx512"), Some(CpuArchitectureVectorLength::VL512)).unwrap();
    assert_eq!(avx512_req512.vector_length, Some("vl512".to_string()));

    // 3. AVX2: normal is 256. If user sets 128 -> VL128. If user sets 256 -> VL256. Cannot select higher (VL512 -> VL256).
    let avx2_default = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Linux), Some(Arch::X86_64), Some("znver3"), None).unwrap();
    assert_eq!(avx2_default.vector_length, Some("vl256".to_string()));

    let avx2_req128 = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Linux), Some(Arch::X86_64), Some("znver3"), Some(CpuArchitectureVectorLength::VL128)).unwrap();
    assert_eq!(avx2_req128.vector_length, Some("vl128".to_string()));

    let avx2_req256 = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Linux), Some(Arch::X86_64), Some("znver3"), Some(CpuArchitectureVectorLength::VL256)).unwrap();
    assert_eq!(avx2_req256.vector_length, Some("vl256".to_string()));

    let avx2_req512 = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Linux), Some(Arch::X86_64), Some("znver3"), Some(CpuArchitectureVectorLength::VL512)).unwrap();
    assert_eq!(avx2_req512.vector_length, Some("vl256".to_string()));

    // 4. AVX, SSE4.2, Arm64, Riscv64: fixed normal is 128, ignores user input
    let sse_report = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Linux), Some(Arch::X86_64), Some("generic"), Some(CpuArchitectureVectorLength::VL256)).unwrap();
    assert_eq!(sse_report.vector_length, Some("vl128".to_string()));

    let sse_req512 = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Linux), Some(Arch::X86_64), Some("generic"), Some(CpuArchitectureVectorLength::VL512)).unwrap();
    assert_eq!(sse_req512.vector_length, Some("vl128".to_string()));

    let avx_req512 = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::X86_64), Some("generic"), None, Some("avx"), None, None, Some(CpuArchitectureVectorLength::VL512)).unwrap();
    assert_eq!(avx_req512.vector_length, Some("vl128".to_string()));

    let arm_report = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Switch2), Some(Arch::Arm64), Some("cortex-a78c"), Some(CpuArchitectureVectorLength::VL512)).unwrap();
    assert_eq!(arm_report.vector_length, Some("vl128".to_string()));

    let riscv_report = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Linux), Some(Arch::Riscv64), Some("sifive-p470"), Some(CpuArchitectureVectorLength::VL512)).unwrap();
    assert_eq!(riscv_report.vector_length, Some("vl128".to_string()));

    // 5. PS5: defaults to vl128 (custom Zen 2 APU deleted FP3 and stripped FP2); user can explicitly select vl256 or vl128
    let ps5_default = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Ps5), Some(Arch::X86_64), None, None).unwrap();
    assert_eq!(ps5_default.vector_length, Some("vl128".to_string()));
    assert_eq!(ps5_default.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));
    assert_eq!(ps5_default.filename(), "ps5-x86_64-znver2-avx2-vl128.json");

    let ps5_none_vl = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Ps5), Some(Arch::X86_64), None, Some(CpuArchitectureVectorLength::None)).unwrap();
    assert_eq!(ps5_none_vl.vector_length, Some("vl128".to_string()));
    assert_eq!(ps5_none_vl.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));

    let ps5_req128 = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Ps5), Some(Arch::X86_64), None, Some(CpuArchitectureVectorLength::VL128)).unwrap();
    assert_eq!(ps5_req128.vector_length, Some("vl128".to_string()));
    assert_eq!(ps5_req128.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));

    let ps5_req256 = ArchFeaturesReport::evaluate_with_vl(Some(Platform::Ps5), Some(Arch::X86_64), None, Some(CpuArchitectureVectorLength::VL256)).unwrap();
    assert_eq!(ps5_req256.vector_length, Some("vl256".to_string()));
    assert_eq!(ps5_req256.target_clang_vlen, Some("-m'prefer-vector-width=256'".to_string()));
    assert_eq!(ps5_req256.filename(), "ps5-x86_64-znver2-avx2-vl256.json");
}

#[test]
fn test_minimum_cpu_architecture_vector_length_security() {
    let lengths = [
        None,
        Some(CpuArchitectureVectorLength::None),
        Some(CpuArchitectureVectorLength::VL128),
        Some(CpuArchitectureVectorLength::VL256),
        Some(CpuArchitectureVectorLength::VL512),
    ];

    // SSE4.2 (None) and AVX: fixed vl128, ignores user input
    for &req in &lengths {
        assert_eq!(MinimumCpuArchitectureX64::None.resolve_vector_length(req), CpuArchitectureVectorLength::VL128);
        assert_eq!(MinimumCpuArchitectureX64::AVX.resolve_vector_length(req), CpuArchitectureVectorLength::VL128);
    }

    // AVX2: vl256 or vl128; defaults to vl256; cannot select higher than 256 (vl512 -> vl256)
    assert_eq!(MinimumCpuArchitectureX64::AVX2.resolve_vector_length(None), CpuArchitectureVectorLength::VL256);
    assert_eq!(MinimumCpuArchitectureX64::AVX2.resolve_vector_length(Some(CpuArchitectureVectorLength::None)), CpuArchitectureVectorLength::VL256);
    assert_eq!(MinimumCpuArchitectureX64::AVX2.resolve_vector_length(Some(CpuArchitectureVectorLength::VL128)), CpuArchitectureVectorLength::VL128);
    assert_eq!(MinimumCpuArchitectureX64::AVX2.resolve_vector_length(Some(CpuArchitectureVectorLength::VL256)), CpuArchitectureVectorLength::VL256);
    assert_eq!(MinimumCpuArchitectureX64::AVX2.resolve_vector_length(Some(CpuArchitectureVectorLength::VL512)), CpuArchitectureVectorLength::VL256);

    // AVX512: vl512 or vl256 or vl128; defaults to vl512
    assert_eq!(MinimumCpuArchitectureX64::AVX512.resolve_vector_length(None), CpuArchitectureVectorLength::VL512);
    assert_eq!(MinimumCpuArchitectureX64::AVX512.resolve_vector_length(Some(CpuArchitectureVectorLength::None)), CpuArchitectureVectorLength::VL512);
    assert_eq!(MinimumCpuArchitectureX64::AVX512.resolve_vector_length(Some(CpuArchitectureVectorLength::VL128)), CpuArchitectureVectorLength::VL128);
    assert_eq!(MinimumCpuArchitectureX64::AVX512.resolve_vector_length(Some(CpuArchitectureVectorLength::VL256)), CpuArchitectureVectorLength::VL256);
    assert_eq!(MinimumCpuArchitectureX64::AVX512.resolve_vector_length(Some(CpuArchitectureVectorLength::VL512)), CpuArchitectureVectorLength::VL512);

    // AVX10.1 and AVX10.2: vl512 or vl256 or vl128; defaults to vl256
    for &avx10 in &[MinimumCpuArchitectureX64::AVX10_1, MinimumCpuArchitectureX64::AVX10_2] {
        assert_eq!(avx10.resolve_vector_length(None), CpuArchitectureVectorLength::VL256);
        assert_eq!(avx10.resolve_vector_length(Some(CpuArchitectureVectorLength::None)), CpuArchitectureVectorLength::VL256);
        assert_eq!(avx10.resolve_vector_length(Some(CpuArchitectureVectorLength::VL128)), CpuArchitectureVectorLength::VL128);
        assert_eq!(avx10.resolve_vector_length(Some(CpuArchitectureVectorLength::VL256)), CpuArchitectureVectorLength::VL256);
        assert_eq!(avx10.resolve_vector_length(Some(CpuArchitectureVectorLength::VL512)), CpuArchitectureVectorLength::VL512);
    }
}

#[test]
fn test_user_min_arch_selection_x64_generic() {
    // 1. Generic + AVX
    let avx_report = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::X86_64), Some("generic"), None, Some("avx"), None, None, None).unwrap();
    assert_eq!(avx_report.target_cpu, Some("x86-64-v2".to_string()));
    assert_eq!(avx_report.min_cpu_arch, Some("avx".to_string()));
    assert_eq!(avx_report.vector_length, Some("vl128".to_string()));
    assert_eq!(avx_report.target_msvc_arch, Some("".to_string()));
    assert_eq!(avx_report.target_msvc_vlen, Some("".to_string()));
    assert_eq!(avx_report.target_clan_arch, Some("-m'arch=x86-64-v2'".to_string()));
    assert!(avx_report.extensions.contains("avx"));
    assert!(avx_report.target_clang_isaarch.as_ref().unwrap().contains("-m'avx'"));
    assert_eq!(avx_report.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));
    assert_eq!(avx_report.filename(), "linux-x86_64-generic-x86-64-v2-avx-vl128-glibc-2.17.json");

    let win_avx_report = ArchFeaturesReport::evaluate_full(Some(Platform::Windows), Some(Arch::X86_64), Some("generic"), None, Some("avx"), None, None, None).unwrap();
    assert_eq!(win_avx_report.target_msvc_arch, Some("/arch:AVX".to_string()));
    assert_eq!(win_avx_report.target_msvc_vlen, Some("".to_string()));

    // 2. Generic + AVX2 (default VL256 -> target_clang_vlen: "-m'prefer-vector-width=256'")
    let avx2_report = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::X86_64), Some("generic"), None, Some("avx2"), None, None, None).unwrap();
    assert_eq!(avx2_report.target_cpu, Some("x86-64-v3".to_string()));
    assert_eq!(avx2_report.min_cpu_arch, Some("avx2".to_string()));
    assert_eq!(avx2_report.vector_length, Some("vl256".to_string()));
    assert_eq!(avx2_report.target_msvc_arch, Some("".to_string()));
    assert_eq!(avx2_report.target_msvc_vlen, Some("".to_string()));
    assert_eq!(avx2_report.target_clan_arch, Some("-m'arch=x86-64-v3'".to_string()));
    assert!(avx2_report.extensions.contains("avx2"));
    assert!(avx2_report.extensions.contains("fma"));
    assert!(avx2_report.target_clang_isaarch.as_ref().unwrap().contains("-m'avx2'"));
    assert_eq!(avx2_report.target_clang_vlen, Some("-m'prefer-vector-width=256'".to_string()));
    assert_eq!(avx2_report.filename(), "linux-x86_64-generic-x86-64-v3-avx2-vl256-glibc-2.17.json");

    let win_avx2_report = ArchFeaturesReport::evaluate_full(Some(Platform::Windows), Some(Arch::X86_64), Some("generic"), None, Some("avx2"), None, None, None).unwrap();
    assert_eq!(win_avx2_report.target_msvc_arch, Some("/arch:AVX2".to_string()));
    assert_eq!(win_avx2_report.target_msvc_vlen, Some("".to_string()));

    // AVX2 with VL128 override -> target_clang_vlen: "-m'prefer-vector-width=128'"
    let avx2_vl128_report = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::X86_64), Some("generic"), None, Some("avx2"), None, None, Some(CpuArchitectureVectorLength::VL128)).unwrap();
    assert_eq!(avx2_vl128_report.vector_length, Some("vl128".to_string()));
    assert_eq!(avx2_vl128_report.target_msvc_vlen, Some("".to_string()));
    assert_eq!(avx2_vl128_report.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));

    // 3. Generic + AVX512 (default VL512 -> target_msvc_vlen: "", target_clang_vlen: "-m'prefer-vector-width=512'")
    let avx512_report = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::X86_64), Some("generic"), None, Some("avx512"), None, None, None).unwrap();
    assert_eq!(avx512_report.target_cpu, Some("x86-64-v4".to_string()));
    assert_eq!(avx512_report.min_cpu_arch, Some("avx512".to_string()));
    assert_eq!(avx512_report.vector_length, Some("vl512".to_string()));
    assert_eq!(avx512_report.target_msvc_arch, Some("".to_string()));
    assert_eq!(avx512_report.target_msvc_vlen, Some("".to_string()));
    assert_eq!(avx512_report.target_clan_arch, Some("-m'arch=x86-64-v4'".to_string()));
    assert!(avx512_report.extensions.contains("avx512f"));
    assert!(avx512_report.target_clang_isaarch.as_ref().unwrap().contains("-m'avx512f'"));
    assert_eq!(avx512_report.target_clang_vlen, Some("-m'prefer-vector-width=512'".to_string()));
    assert_eq!(avx512_report.filename(), "linux-x86_64-generic-x86-64-v4-avx512-vl512-glibc-2.17.json");

    // AVX512 with user requested VL256 -> /arch:AVX512 /vlen=256, clang: -m'prefer-vector-width=256'
    let avx512_vl256_report = ArchFeaturesReport::evaluate_full(Some(Platform::Windows), Some(Arch::X86_64), Some("generic"), None, Some("avx512"), None, None, Some(CpuArchitectureVectorLength::VL256)).unwrap();
    assert_eq!(avx512_vl256_report.vector_length, Some("vl256".to_string()));
    assert_eq!(avx512_vl256_report.target_msvc_arch, Some("/arch:AVX512".to_string()));
    assert_eq!(avx512_vl256_report.target_msvc_vlen, Some("/vlen=256".to_string()));
    assert_eq!(avx512_vl256_report.target_clang_vlen, Some("-m'prefer-vector-width=256'".to_string()));

    // AVX512 with VL128 override -> target_clang_vlen: "-m'prefer-vector-width=128'"
    let avx512_vl128_report = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::X86_64), Some("generic"), None, Some("avx512"), None, None, Some(CpuArchitectureVectorLength::VL128)).unwrap();
    assert_eq!(avx512_vl128_report.vector_length, Some("vl128".to_string()));
    assert_eq!(avx512_vl128_report.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));
    assert_eq!(avx512_vl128_report.filename(), "linux-x86_64-generic-x86-64-v4-avx512-vl128-glibc-2.17.json");

    // 4. Generic + AVX10.1 (default VL256 -> target_msvc_vlen: "", target_clang_vlen: "-m'prefer-vector-width=256'", clang: -m'avx10.1')
    let avx10_1_report = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::X86_64), Some("generic"), None, Some("avx10.1"), None, None, None).unwrap();
    assert_eq!(avx10_1_report.target_cpu, Some("x86-64-v3".to_string()));
    assert_eq!(avx10_1_report.min_cpu_arch, Some("avx10.1".to_string()));
    assert_eq!(avx10_1_report.vector_length, Some("vl256".to_string()));
    assert_eq!(avx10_1_report.target_msvc_arch, Some("".to_string()));
    assert_eq!(avx10_1_report.target_msvc_vlen, Some("".to_string()));
    assert_eq!(avx10_1_report.target_clan_arch, Some("-m'arch=x86-64-v3'".to_string()));
    assert!(avx10_1_report.extensions.contains("avx10.1"));
    assert!(avx10_1_report.target_clang_isaarch.as_ref().unwrap().contains("-m'avx10.1'"));
    assert!(!avx10_1_report.target_clang_isaarch.as_ref().unwrap().contains("-m'avx10.1-256'"));
    assert_eq!(avx10_1_report.target_clang_vlen, Some("-m'prefer-vector-width=256'".to_string()));
    assert_eq!(avx10_1_report.filename(), "linux-x86_64-generic-x86-64-v3-avx10.1-vl256-glibc-2.17.json");

    // AVX10.1 with VL128 override -> target_clang_vlen: "-m'prefer-vector-width=128'"
    let avx10_1_vl128_report = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::X86_64), Some("generic"), None, Some("avx10.1"), None, None, Some(CpuArchitectureVectorLength::VL128)).unwrap();
    assert_eq!(avx10_1_vl128_report.vector_length, Some("vl128".to_string()));
    assert_eq!(avx10_1_vl128_report.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));
    assert_eq!(avx10_1_vl128_report.filename(), "linux-x86_64-generic-x86-64-v3-avx10.1-vl128-glibc-2.17.json");

    // 5. Generic + AVX10.2 with user requested VL512 -> /arch:AVX10.2 /vlen=512, clang: -m'avx10.2-512' -m'prefer-vector-width=512'
    let avx10_2_report = ArchFeaturesReport::evaluate_full(Some(Platform::Windows), Some(Arch::X86_64), Some("generic"), None, Some("avx10.2"), None, None, Some(CpuArchitectureVectorLength::VL512)).unwrap();
    assert_eq!(avx10_2_report.target_cpu, Some("x86-64-v3".to_string()));
    assert_eq!(avx10_2_report.min_cpu_arch, Some("avx10.2".to_string()));
    assert_eq!(avx10_2_report.vector_length, Some("vl512".to_string()));
    assert_eq!(avx10_2_report.target_msvc_arch, Some("/arch:AVX10.2".to_string()));
    assert_eq!(avx10_2_report.target_msvc_vlen, Some("/vlen=512".to_string()));
    assert_eq!(avx10_2_report.target_clan_arch, Some("-m'arch=x86-64-v3'".to_string()));
    assert!(avx10_2_report.extensions.contains("avx10.2"));
    assert!(avx10_2_report.target_clang_isaarch.as_ref().unwrap().contains("-m'avx10.2-512'"));
    assert_eq!(avx10_2_report.target_clang_vlen, Some("-m'prefer-vector-width=512'".to_string()));
    assert_eq!(avx10_2_report.filename(), "windows-x86_64-generic-x86-64-v3-avx10.2-vl512-msvc-19.51.json");

    // 6. Non-generic target ignores min_arch override
    let znver3_report = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::X86_64), Some("znver3"), None, Some("avx512"), None, None, None).unwrap();
    assert_eq!(znver3_report.target_cpu, Some("znver3".to_string()));
    assert_eq!(znver3_report.min_cpu_arch, Some("avx2".to_string()));
    assert_eq!(znver3_report.target_clan_arch, Some("-m'arch=znver3'".to_string()));
    assert_eq!(znver3_report.filename(), "linux-x86_64-znver3-avx2-vl256-glibc-2.17.json");
}

#[test]
fn test_user_min_arch_selection_arm64_generic() {
    // 1. Generic + ARMv8-A
    let v8_report = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::Arm64), Some("generic"), None, Some("armv8-a"), None, None, None).unwrap();
    assert_eq!(v8_report.target_cpu, Some("generic".to_string()));
    assert_eq!(v8_report.min_cpu_arch, Some("armv8-a".to_string()));
    assert_eq!(v8_report.vector_length, Some("vl128".to_string()));
    assert_eq!(v8_report.target_msvc_arch, Some("".to_string()));
    assert!(v8_report.target_clan_arch.as_ref().unwrap().starts_with("-m'arch=armv8-a+"));
    assert_eq!(v8_report.target_clang_isaarch, Some("".to_string()));
    assert_eq!(v8_report.target_clang_cpu, Some("-m'cpu=generic'".to_string()));
    assert_eq!(v8_report.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));
    assert_eq!(v8_report.filename(), "linux-aarch64-generic-armv8-a-vl128-glibc-2.17.json");

    // 2. Generic + ARMv8.2-A
    let v82_report = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::Arm64), Some("generic"), None, Some("armv8.2-a"), None, None, None).unwrap();
    assert_eq!(v82_report.target_cpu, Some("generic".to_string()));
    assert_eq!(v82_report.min_cpu_arch, Some("armv8.2-a".to_string()));
    assert_eq!(v82_report.vector_length, Some("vl128".to_string()));
    assert_eq!(v82_report.target_msvc_arch, Some("".to_string()));
    assert!(v82_report.target_clan_arch.as_ref().unwrap().starts_with("-m'arch=armv8.2-a+"));
    assert!(v82_report.target_clan_arch.as_ref().unwrap().contains("+dotprod"));
    assert_eq!(v82_report.target_clang_isaarch, Some("".to_string()));
    assert_eq!(v82_report.target_clang_cpu, Some("-m'cpu=generic'".to_string()));
    assert!(v82_report.extensions.contains("dotprod"));
    assert_eq!(v82_report.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));
    assert_eq!(v82_report.filename(), "linux-aarch64-generic-armv8.2-a-vl128-glibc-2.17.json");

    // 3. Generic + ARMv8-R
    let v8r_report = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::Arm64), Some("generic"), None, Some("armv8-r"), None, None, None).unwrap();
    assert_eq!(v8r_report.target_cpu, Some("generic".to_string()));
    assert_eq!(v8r_report.min_cpu_arch, Some("armv8-r".to_string()));
    assert_eq!(v8r_report.vector_length, Some("vl128".to_string()));
    assert_eq!(v8r_report.target_msvc_arch, Some("".to_string()));
    assert!(v8r_report.target_clan_arch.as_ref().unwrap().starts_with("-m'arch=armv8-r+"));
    assert_eq!(v8r_report.target_clang_isaarch, Some("".to_string()));
    assert_eq!(v8r_report.target_clang_cpu, Some("-m'cpu=generic'".to_string()));
    assert_eq!(v8r_report.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));
    assert_eq!(v8r_report.filename(), "linux-aarch64-generic-armv8-r-vl128-glibc-2.17.json");

    // 4. Generic + ARMv8.9-A
    let v89_report = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::Arm64), Some("generic"), None, Some("armv8.9-a"), None, None, None).unwrap();
    assert_eq!(v89_report.target_cpu, Some("generic".to_string()));
    assert_eq!(v89_report.min_cpu_arch, Some("armv8.9-a".to_string()));
    assert_eq!(v89_report.vector_length, Some("vl128".to_string()));
    assert_eq!(v89_report.target_msvc_arch, Some("".to_string()));
    assert!(v89_report.target_clan_arch.as_ref().unwrap().starts_with("-m'arch=armv8.9-a+"));
    assert_eq!(v89_report.target_clang_isaarch, Some("".to_string()));
    assert_eq!(v89_report.target_clang_cpu, Some("-m'cpu=generic'".to_string()));
    assert!(v89_report.extensions.contains("cssc"));
    assert_eq!(v89_report.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));
    assert_eq!(v89_report.filename(), "linux-aarch64-generic-armv8.9-a-vl128-glibc-2.17.json");

    // 5. Generic + ARMv9-A
    let v9_report = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::Arm64), Some("generic"), None, Some("armv9-a"), None, None, None).unwrap();
    assert_eq!(v9_report.target_cpu, Some("generic".to_string()));
    assert_eq!(v9_report.min_cpu_arch, Some("armv9-a".to_string()));
    assert_eq!(v9_report.vector_length, Some("vl128".to_string()));
    assert_eq!(v9_report.target_msvc_arch, Some("".to_string()));
    assert!(v9_report.target_clan_arch.as_ref().unwrap().starts_with("-m'arch=armv9-a+"));
    assert_eq!(v9_report.target_clang_isaarch, Some("".to_string()));
    assert_eq!(v9_report.target_clang_cpu, Some("-m'cpu=generic'".to_string()));
    assert!(v9_report.extensions.contains("sve"));
    assert_eq!(v9_report.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));
    assert_eq!(v9_report.filename(), "linux-aarch64-generic-armv9-a-vl128-glibc-2.17.json");

    // 6. Generic + ARMv9.4-A
    let v94_report = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::Arm64), Some("generic"), None, Some("armv9.4-a"), None, None, None).unwrap();
    assert_eq!(v94_report.target_cpu, Some("generic".to_string()));
    assert_eq!(v94_report.min_cpu_arch, Some("armv9.4-a".to_string()));
    assert_eq!(v94_report.vector_length, Some("vl128".to_string()));
    assert_eq!(v94_report.target_msvc_arch, Some("".to_string()));
    assert!(v94_report.target_clan_arch.as_ref().unwrap().starts_with("-m'arch=armv9.4-a+"));
    assert_eq!(v94_report.target_clang_isaarch, Some("".to_string()));
    assert_eq!(v94_report.target_clang_cpu, Some("-m'cpu=generic'".to_string()));
    assert!(v94_report.extensions.contains("lse128"));
    assert_eq!(v94_report.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));
    assert_eq!(v94_report.filename(), "linux-aarch64-generic-armv9.4-a-vl128-glibc-2.17.json");

    // 7. Generic + ARMv9.7-A
    let v97_report = ArchFeaturesReport::evaluate_full(Some(Platform::Windows), Some(Arch::Arm64), Some("generic"), None, Some("armv9.7-a"), None, None, None).unwrap();
    assert_eq!(v97_report.target_cpu, Some("generic".to_string()));
    assert_eq!(v97_report.min_cpu_arch, Some("armv9.7-a".to_string()));
    assert_eq!(v97_report.vector_length, Some("vl128".to_string()));
    assert_eq!(v97_report.target_msvc_arch, Some("/arch:armv9.4".to_string()));
    assert!(v97_report.target_clan_arch.as_ref().unwrap().starts_with("-m'arch=armv9.7-a+"));
    assert_eq!(v97_report.target_clang_isaarch, Some("".to_string()));
    assert_eq!(v97_report.target_clang_cpu, Some("-m'cpu=generic'".to_string()));
    assert!(v97_report.extensions.contains("d128"));
    assert_eq!(v97_report.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));
    assert_eq!(v97_report.filename(), "windows-aarch64-generic-armv9.7-a-vl128-msvc-19.51.json");
}

#[test]
fn test_user_min_arch_selection_riscv64_generic() {
    // Riscv64 ignores min_arch because march is processed differently
    let rv_report = ArchFeaturesReport::evaluate_full(Some(Platform::Linux), Some(Arch::Riscv64), Some("generic"), None, Some("avx512"), None, None, None).unwrap();
    assert_eq!(rv_report.min_cpu_arch, Some("none".to_string()));
    assert_eq!(rv_report.vector_length, Some("vl128".to_string()));
    assert_eq!(rv_report.target_msvc_arch, Some("".to_string()));
    assert!(rv_report.target_clan_arch.as_ref().unwrap().starts_with("-m'arch=rv64"));
    assert_eq!(rv_report.target_clang_isaarch, Some("".to_string()));
    assert_eq!(rv_report.target_clang_cpu, Some("-m'cpu=generic-rv64'".to_string()));
    assert_eq!(rv_report.target_clang_vlen, Some("-m'prefer-vector-width=128'".to_string()));
    assert_eq!(rv_report.filename(), "linux-riscv64-generic-rv64-none-vl128-glibc-2.17.json");
}

#[test]
fn test_apxf_os_support() {
    use archinfo::arch::x86_64::xcr0::Xcr0State;

    // Test Xcr0State without OSXSAVE
    let xcr0_disabled = Xcr0State::query(false);
    assert!(!xcr0_disabled.is_apx_usable());
    assert!(!xcr0_disabled.is_avx_usable());
    assert!(!xcr0_disabled.is_avx512_usable());

    // Test apxf feature enabled from target
    let dnr_report = ArchFeaturesReport::evaluate(Some(Platform::Linux), Some(Arch::X86_64), Some("diamondrapids")).unwrap();
    assert!(dnr_report.extensions.contains("apxf"));
    assert_eq!(dnr_report.features.get("apxf"), Some(&true));

    // Test from_extensions_str
    let f = X64CPUFeatures::from_extensions_str("sse+sse2+apxf");
    assert!(f.apxf);
    assert!(f.apxf_usable);
    assert!(f.has_feature(X64ISA::Apxf));
}

#[test]
fn test_x64_name_mappings() {
    assert_eq!(MSVCX64ISANames::name(MinimumCpuArchitectureX64::AVX2), "AVX2");
    assert_eq!(MSVCX64ISANames::name(MinimumCpuArchitectureX64::AVX512), "AVX512");
    assert_eq!(MSVCX64ISANames::name(MinimumCpuArchitectureX64::None), "SSE4.2");

    assert_eq!(TargetCpuArchitectureX64Names::name(TargetCpuArchitectureX64::Znver3), "znver3");
    assert_eq!(TargetCpuArchitectureX64Names::name(TargetCpuArchitectureX64::Alderlake), "alderlake");

    // MSVCX64ArchTarget
    assert_eq!(MSVCX64ArchTarget::name(MinimumCpuArchitectureX64::None), "/arch:SSE4.2");
    assert_eq!(MSVCX64ArchTarget::name(MinimumCpuArchitectureX64::AVX), "/arch:AVX");
    assert_eq!(MSVCX64ArchTarget::name(MinimumCpuArchitectureX64::AVX2), "/arch:AVX2");
    assert_eq!(MSVCX64ArchTarget::name(MinimumCpuArchitectureX64::AVX512), "/arch:AVX512");
    assert_eq!(MSVCX64ArchTarget::name(MinimumCpuArchitectureX64::AVX10_1), "/arch:AVX10.1");
    assert_eq!(MSVCX64ArchTarget::name(MinimumCpuArchitectureX64::AVX10_2), "/arch:AVX10.2");

    // ClangTargetCpuArchitectureX64NOISANames
    assert_eq!(ClangTargetCpuArchitectureX64NOISANames::name(TargetCpuArchitectureX64::None), "");
    assert_eq!(ClangTargetCpuArchitectureX64NOISANames::name(TargetCpuArchitectureX64::Generic), "");
    assert_eq!(ClangTargetCpuArchitectureX64NOISANames::name(TargetCpuArchitectureX64::X86_64_v4), "fsgsbase+rdpru+fma4+xop+lwp+tbm");
    assert_eq!(ClangTargetCpuArchitectureX64NOISANames::name(TargetCpuArchitectureX64::Diamondrapids), "avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx512vp2intersect");

    // ClangX64ISANames (-m{extension})
    assert_eq!(ClangX64ISANames::name(X64ISA::Sse4_1), "sse4.1");
    assert_eq!(ClangX64ISANames::name(X64ISA::Sse4_2), "sse4.2");
    assert_eq!(ClangX64ISANames::name(X64ISA::Avx10_1), "avx10.1");
    assert_eq!(ClangX64ISANames::name(X64ISA::Avx10_2), "avx10.2");
    assert_eq!(ClangX64ISANames::name(X64ISA::AmxAvx512), "amx-avx512");
    assert_eq!(ClangX64ISANames::name(X64ISA::AmxTile), "amx-tile");
    assert_eq!(ClangX64ISANames::name(X64ISA::Apxf), "apxf");

    // ClangX64NOISANames (-mno-{extension})
    assert_eq!(ClangX64NOISANames::name(X64ISA::Sse4_1), "no-sse4.1");
    assert_eq!(ClangX64NOISANames::name(X64ISA::Sse4_2), "no-sse4.2");
    assert_eq!(ClangX64NOISANames::name(X64ISA::Avx10_1), "no-avx10.1");
    assert_eq!(ClangX64NOISANames::name(X64ISA::Avx10_2), "no-avx10.2");
    assert_eq!(ClangX64NOISANames::name(X64ISA::AmxAvx512), "no-amx-avx512");
    assert_eq!(ClangX64NOISANames::name(X64ISA::AmxTile), "no-amx-tile");
    assert_eq!(ClangX64NOISANames::name(X64ISA::Apxf), "no-apxf");
}

#[test]
fn test_arm64_name_mappings() {
    assert_eq!(MinimumCpuArchitectureArm64ClangNames::name(MinimumCpuArchitectureArm64::ARMv8_2A), "armv8.2-a");
    assert_eq!(MinimumCpuArchitectureArm64MSVCNames::name(MinimumCpuArchitectureArm64::ARMv8_2A), "armv8.2");
    assert_eq!(TargetCpuArchitectureArm64Names::name(TargetCpuArchitectureArm64::Apple_M1), "apple-m1");
    assert_eq!(TargetCpuArchitectureArm64Names::name(TargetCpuArchitectureArm64::Cortex_A78C), "cortex-a78c");
}

#[test]
fn test_riscv64_name_mappings() {
    assert_eq!(TargetCpuArchitectureRiscv64Names::name(TargetCpuArchitectureRiscv64::Sifive_P450), "sifive-p450");
    assert_eq!(TargetCpuArchitectureRiscv64Names::name(TargetCpuArchitectureRiscv64::Xiangshan_Nanhu), "xiangshan-nanhu");
    assert_eq!(ClangRiscv64ISANames::name(Riscv64ISA::Zba), "zba");
    assert_eq!(ClangRiscv64NOISANames::name(Riscv64ISA::Zba), "-zba");
}

#[test]
fn test_riscv64_clang_target_extensions_are_valid_isas() {
    let targets = [
        TargetCpuArchitectureRiscv64::None,
        TargetCpuArchitectureRiscv64::Generic,
        TargetCpuArchitectureRiscv64::Generic_RV64,
        TargetCpuArchitectureRiscv64::Native,
        TargetCpuArchitectureRiscv64::MIPS_P8700,
        TargetCpuArchitectureRiscv64::Sifive_P450,
        TargetCpuArchitectureRiscv64::Sifive_P470,
        TargetCpuArchitectureRiscv64::Sifive_P550,
        TargetCpuArchitectureRiscv64::Sifive_P670,
        TargetCpuArchitectureRiscv64::Sifive_P870,
        TargetCpuArchitectureRiscv64::TT_Ascalon_X,
        TargetCpuArchitectureRiscv64::Veyron_V1,
        TargetCpuArchitectureRiscv64::Xiangshan_Kunminghu,
        TargetCpuArchitectureRiscv64::Xiangshan_Nanhu,
        TargetCpuArchitectureRiscv64::XT_C910v2,
        TargetCpuArchitectureRiscv64::XT_C920v2,
    ];

    for target in targets {
        let ext_str = ClangTargetCpuArchitectureRiscv64ISANames::name(target);
        if ext_str.is_empty() {
            continue;
        }
        for token in ext_str.split('+') {
            assert!(
                token.parse::<Riscv64ISA>().is_ok(),
                "Target {:?} contains unparseable/commented-out ISA extension '{}'",
                target,
                token
            );
            assert_ne!(token, "sha", "Target {:?} contains privileged extension 'sha'", target);
            assert_ne!(token, "h", "Target {:?} contains privileged extension 'h'", target);
            assert_ne!(token, "svinval", "Target {:?} contains privileged extension 'svinval'", target);
        }
    }
}

#[test]
fn test_user_enabled_disabled_extensions_x64_generic() {
    // Enable extensions with duplicates: "sha+sha+apxf", disable "sse4.1+sse4.2"
    let report = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::X86_64),
        Some("generic"),
        None,
        Some("avx2"),
        Some("sha+sha+apxf"),
        Some("sse4.1+sse4.2"),
        None,
    ).unwrap();

    assert!(report.extensions.contains("sha"));
    assert!(report.extensions.contains("apxf"));
    assert!(!report.extensions.contains("sse4_1"));
    assert!(!report.extensions.contains("sse4_2"));
    assert_eq!(report.features.get("sha"), Some(&true));
    assert_eq!(report.features.get("apxf"), Some(&true));
    assert_eq!(report.features.get("sse4_1"), Some(&false));
    assert_eq!(report.features.get("sse4_2"), Some(&false));

    let isaarch = report.target_clang_isaarch.unwrap();
    assert!(isaarch.contains("-m'sha'"));
    assert!(isaarch.contains("-m'apxf'"));
    assert!(isaarch.contains("-m'no-sse4.1'"));
    assert!(isaarch.contains("-m'no-sse4.2'"));
}

#[test]
fn test_user_enabled_disabled_extensions_arm64_generic() {
    // Enable "sve+sve+sve2", disable "sve2"
    let report = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::Arm64),
        Some("generic"),
        None,
        Some("armv8.2-a"),
        Some("sve+sve+sve2"),
        Some("sve2"),
        None,
    ).unwrap();

    assert!(report.extensions.contains("sve"));
    assert!(!report.extensions.contains("sve2"));
    assert_eq!(report.features.get("sve"), Some(&true));
    assert_eq!(report.features.get("sve2"), Some(&false));
    assert_eq!(report.target_clang_isaarch, Some("".to_string()));
    assert_eq!(report.target_clang_cpu, Some("-m'cpu=generic'".to_string()));

    let clan_arch = report.target_clan_arch.unwrap();
    assert!(clan_arch.starts_with("-m'arch=armv8.2-a+"));
    assert!(clan_arch.contains("+sve"));
    assert!(clan_arch.contains("+nosve2"));
}

#[test]
fn test_user_enabled_disabled_extensions_riscv64_generic() {
    // Enable "zba+zbb", disable "c"
    let report = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::Riscv64),
        Some("generic"),
        None,
        None,
        Some("zba+zbb"),
        Some("c"),
        None,
    ).unwrap();

    assert!(report.extensions.contains("zba"));
    assert!(report.extensions.contains("zbb"));
    assert!(!report.extensions.contains("+c+"));
    assert_eq!(report.features.get("zba"), Some(&true));
    assert_eq!(report.features.get("c"), Some(&false));
    assert_eq!(report.target_clang_isaarch, Some("-Xclang -target-feature -Xclang '-c'".to_string()));
    assert_eq!(report.target_clang_cpu, Some("-m'cpu=generic-rv64'".to_string()));

    let clan_arch = report.target_clan_arch.unwrap();
    assert!(clan_arch.starts_with("-m'arch=rv64"));
    assert!(clan_arch.contains("_zba"));
    assert!(clan_arch.contains("_zbb"));
    assert!(!clan_arch.contains("_c"));
    assert!(!clan_arch.contains("_no-c"));

    // Multiple disabled extensions: -Xclang -target-feature -Xclang '-ziccif,-zmmul'
    let report_multi = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::Riscv64),
        Some("generic"),
        None,
        None,
        None,
        Some("ziccif,zmmul"),
        None,
    ).unwrap();
    assert_eq!(report_multi.target_clang_isaarch, Some("-Xclang -target-feature -Xclang '-ziccif,-zmmul'".to_string()));
    let clan_arch_multi = report_multi.target_clan_arch.unwrap();
    assert!(!clan_arch_multi.contains("ziccif"));
    assert!(!clan_arch_multi.contains("zmmul"));
}

#[test]
fn test_non_generic_ignores_extensions() {
    // Non-generic target should ignore enable/disable extension arguments
    let report = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::X86_64),
        Some("znver3"),
        None,
        None,
        Some("apxf"),
        Some("avx2"),
        None,
    ).unwrap();

    assert_eq!(report.target_cpu, Some("znver3".to_string()));
    assert!(report.extensions.contains("avx2"));
    assert!(!report.extensions.contains("apxf"));
    assert_eq!(report.features.get("avx2"), Some(&true));
    assert_eq!(report.features.get("apxf"), Some(&false));

    // Arm64 non-generic target ignores enable/disable extensions
    let arm_report = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::Arm64),
        Some("cortex-a78"),
        None,
        None,
        Some("sve2"),
        Some("fp16"),
        None,
    ).unwrap();
    assert_eq!(arm_report.target_cpu, Some("cortex-a78".to_string()));
    assert_eq!(arm_report.features.get("sve2"), Some(&false));
    assert_eq!(arm_report.features.get("fp16"), Some(&true));

    // Riscv64 non-generic target ignores enable/disable extensions
    let rv_report = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::Riscv64),
        Some("sifive-p450"),
        None,
        None,
        Some("v"),
        Some("m"),
        None,
    ).unwrap();
    assert_eq!(rv_report.target_cpu, Some("sifive-p450".to_string()));
    assert_eq!(rv_report.features.get("v"), Some(&false));
    assert_eq!(rv_report.features.get("m"), Some(&true));
}

#[test]
fn test_invalid_extensions_return_error() {
    let res = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::X86_64),
        Some("generic"),
        None,
        None,
        Some("non_existent_isa_feature"),
        None,
        None,
    );
    assert!(res.is_err());
}

#[test]
fn test_target_clang_extraargs() {
    // x86_64 has empty string
    let x64 = ArchFeaturesReport::evaluate(Some(Platform::Linux), Some(Arch::X86_64), Some("generic")).unwrap();
    assert_eq!(x64.target_clang_extraargs, Some("".to_string()));
    assert!(x64.to_json().unwrap().contains("\"target_clang_extraargs\": \"\""));

    // arm64 has empty string
    let arm = ArchFeaturesReport::evaluate(Some(Platform::Linux), Some(Arch::Arm64), Some("generic")).unwrap();
    assert_eq!(arm.target_clang_extraargs, Some("".to_string()));

    // riscv64 generic without experimental extensions has empty string
    let rv = ArchFeaturesReport::evaluate(Some(Platform::Linux), Some(Arch::Riscv64), Some("generic")).unwrap();
    assert_eq!(rv.target_clang_extraargs, Some("".to_string()));

    // riscv64 generic with experimental extension enabled
    let rv_exp = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::Riscv64),
        Some("generic"),
        None,
        None,
        Some("zicfilp"),
        None,
        None,
    ).unwrap();
    assert_eq!(rv_exp.target_clang_extraargs, Some("-m'enable-experimental-extensions'".to_string()));
    assert!(rv_exp.to_json().unwrap().contains("\"target_clang_extraargs\": \"-m'enable-experimental-extensions'\""));

    // riscv64 generic with another experimental extension (p)
    let rv_p = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::Riscv64),
        Some("generic"),
        None,
        None,
        Some("p"),
        None,
        None,
    ).unwrap();
    assert_eq!(rv_p.target_clang_extraargs, Some("-m'enable-experimental-extensions'".to_string()));

    // riscv64 generic with experimental extension enabled BUT also disabled
    let rv_dis = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::Riscv64),
        Some("generic"),
        None,
        None,
        Some("zicfilp"),
        Some("zicfilp"),
        None,
    ).unwrap();
    assert_eq!(rv_dis.target_clang_extraargs, Some("".to_string()));
}

#[test]
fn test_target_tune_cpu() {
    // 1. User does not select target_cpu -> on native host, target_cpu is "native" and target_tune_cpu is empty ""
    let report_zero = ArchFeaturesReport::evaluate(None, None, None).unwrap();
    assert_eq!(report_zero.target_cpu, Some("native".to_string()));
    assert_eq!(report_zero.target_tune_cpu, Some("".to_string()));

    let report_no_target = ArchFeaturesReport::evaluate(Some(Platform::Windows), Some(Arch::X86_64), None).unwrap();
    assert_eq!(report_no_target.target_tune_cpu, Some("".to_string()));

    // When target_cpu defaults to a non-native target (e.g., Linux on Windows host), target_tune_cpu defaults to target_cpu
    let report_linux_default = ArchFeaturesReport::evaluate(Some(Platform::Linux), Some(Arch::X86_64), None).unwrap();
    assert_eq!(report_linux_default.target_cpu, Some("x86-64-v2".to_string()));
    assert_eq!(report_linux_default.target_tune_cpu, Some("x86-64-v2".to_string()));
    assert_eq!(report_linux_default.target_clang_tune_cpu, Some("-m'tune=x86-64-v2'".to_string()));

    let report_ps5_from_target = ArchFeaturesReport::from_target(Platform::Ps5, Arch::X86_64).unwrap();
    assert_eq!(report_ps5_from_target.target_cpu, Some("znver2".to_string()));
    assert_eq!(report_ps5_from_target.target_tune_cpu, Some("znver2".to_string()));
    assert_eq!(report_ps5_from_target.target_clang_tune_cpu, Some("-m'tune=znver2'".to_string()));

    // 2. User selects native target_cpu -> target_tune_cpu is empty ""
    let report_native = ArchFeaturesReport::evaluate(Some(Platform::Windows), Some(Arch::X86_64), Some("native")).unwrap();
    assert_eq!(report_native.target_cpu, Some("native".to_string()));
    assert_eq!(report_native.target_tune_cpu, Some("".to_string()));

    // 3. User selects generic x86_64 target_cpu without target_tune_cpu -> defaults to based target_cpu name
    let report_gen_x64 = ArchFeaturesReport::evaluate(Some(Platform::Linux), Some(Arch::X86_64), Some("generic")).unwrap();
    assert_eq!(report_gen_x64.target_cpu, Some("x86-64-v2".to_string()));
    assert_eq!(report_gen_x64.target_tune_cpu, Some("x86-64-v2".to_string()));

    // Generic x86_64 with min-arch avx2 -> based target_cpu is x86-64-v3
    let report_gen_avx2 = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::X86_64),
        Some("generic"),
        None,
        Some("avx2"),
        None,
        None,
        None,
    ).unwrap();
    assert_eq!(report_gen_avx2.target_cpu, Some("x86-64-v3".to_string()));
    assert_eq!(report_gen_avx2.target_tune_cpu, Some("x86-64-v3".to_string()));

    // 4. User selects known x86_64 target_cpu without target_tune_cpu -> defaults to based target_cpu name
    let report_znver3 = ArchFeaturesReport::evaluate(Some(Platform::Linux), Some(Arch::X86_64), Some("znver3")).unwrap();
    assert_eq!(report_znver3.target_cpu, Some("znver3".to_string()));
    assert_eq!(report_znver3.target_tune_cpu, Some("znver3".to_string()));

    // 5. User selects known x86_64 target_cpu with valid target_tune_cpu
    let report_tune = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::X86_64),
        Some("znver3"),
        Some("alderlake"),
        None,
        None,
        None,
        None,
    ).unwrap();
    assert_eq!(report_tune.target_cpu, Some("znver3".to_string()));
    assert_eq!(report_tune.target_tune_cpu, Some("alderlake".to_string()));

    // Generic x86_64 with explicit target_tune_cpu
    let report_gen_tune = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::X86_64),
        Some("generic"),
        Some("znver3"),
        None,
        None,
        None,
        None,
    ).unwrap();
    assert_eq!(report_gen_tune.target_cpu, Some("x86-64-v2".to_string()));
    assert_eq!(report_gen_tune.target_tune_cpu, Some("znver3".to_string()));

    // 6. Native target is not allowed for target_tune_cpu -> returns Err
    let err_native_tune_x64 = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::X86_64),
        Some("znver3"),
        Some("native"),
        None,
        None,
        None,
        None,
    );
    assert!(err_native_tune_x64.is_err());

    // 7. Arm64: generic and specific targets without and with tune
    let report_arm_gen = ArchFeaturesReport::evaluate(Some(Platform::Linux), Some(Arch::Arm64), Some("generic")).unwrap();
    assert_eq!(report_arm_gen.target_cpu, Some("generic".to_string()));
    assert_eq!(report_arm_gen.target_tune_cpu, Some("generic".to_string()));

    let report_arm_tune = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::Arm64),
        Some("cortex-a78"),
        Some("cortex-x1"),
        None,
        None,
        None,
        None,
    ).unwrap();
    assert_eq!(report_arm_tune.target_cpu, Some("cortex-a78".to_string()));
    assert_eq!(report_arm_tune.target_tune_cpu, Some("cortex-x1".to_string()));

    let report_mac_tune = ArchFeaturesReport::evaluate_full(
        Some(Platform::Macosx),
        Some(Arch::Arm64),
        Some("apple-m1"),
        Some("apple-m4"),
        None,
        None,
        None,
        None,
    ).unwrap();
    assert_eq!(report_mac_tune.target_cpu, Some("apple-m1".to_string()));
    assert_eq!(report_mac_tune.target_tune_cpu, Some("apple-m4".to_string()));
    assert_eq!(report_mac_tune.target_clang_tune_cpu, Some("-m'tune=apple-m4'".to_string()));

    let err_apple_tune_linux = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::Arm64),
        Some("cortex-a78"),
        Some("apple-m4"),
        None,
        None,
        None,
        None,
    );
    assert!(err_apple_tune_linux.is_err());

    let err_native_tune_arm = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::Arm64),
        Some("cortex-a78"),
        Some("native"),
        None,
        None,
        None,
        None,
    );
    assert!(err_native_tune_arm.is_err());

    // 8. Riscv64: generic and specific targets without and with tune
    let report_rv_gen = ArchFeaturesReport::evaluate(Some(Platform::Linux), Some(Arch::Riscv64), Some("generic")).unwrap();
    assert_eq!(report_rv_gen.target_cpu, Some("generic-rv64".to_string()));
    assert_eq!(report_rv_gen.target_tune_cpu, Some("generic-rv64".to_string()));

    let report_rv_tune = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::Riscv64),
        Some("sifive-p470"),
        Some("veyron-v1"),
        None,
        None,
        None,
        None,
    ).unwrap();
    assert_eq!(report_rv_tune.target_cpu, Some("sifive-p470".to_string()));
    assert_eq!(report_rv_tune.target_tune_cpu, Some("veyron-v1".to_string()));

    let err_native_tune_rv = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::Riscv64),
        Some("sifive-p470"),
        Some("native"),
        None,
        None,
        None,
        None,
    );
    assert!(err_native_tune_rv.is_err());

    // 9. Invalid target_tune_cpu returns error
    let err_invalid_tune = ArchFeaturesReport::evaluate_full(
        Some(Platform::Linux),
        Some(Arch::X86_64),
        Some("znver3"),
        Some("non_existent_cpu"),
        None,
        None,
        None,
        None,
    );
    assert!(err_invalid_tune.is_err());

    // 10. target_clang_tune_cpu tests
    assert_eq!(report_zero.target_clang_tune_cpu, Some("".to_string()));
    assert_eq!(report_native.target_clang_tune_cpu, Some("".to_string()));
    assert_eq!(report_gen_x64.target_clang_tune_cpu, Some("-m'tune=x86-64-v2'".to_string()));
    assert_eq!(report_znver3.target_clang_tune_cpu, Some("-m'tune=znver3'".to_string()));
    assert_eq!(report_tune.target_clang_tune_cpu, Some("-m'tune=alderlake'".to_string()));
    assert_eq!(report_arm_gen.target_clang_tune_cpu, Some("-m'tune=generic'".to_string()));
    assert_eq!(report_arm_tune.target_clang_tune_cpu, Some("-m'tune=cortex-x1'".to_string()));
    assert_eq!(report_rv_gen.target_clang_tune_cpu, Some("-m'tune=generic-rv64'".to_string()));
    assert_eq!(report_rv_tune.target_clang_tune_cpu, Some("-m'tune=veyron-v1'".to_string()));

    // 11. JSON output verifies target_tune_cpu is after target_cpu, target_clang_tune_cpu is after target_clang_cpu
    let json = report_tune.to_json().unwrap();
    let target_cpu_idx = json.find("\"target_cpu\"").unwrap();
    let target_tune_cpu_idx = json.find("\"target_tune_cpu\"").unwrap();
    assert!(target_cpu_idx < target_tune_cpu_idx);
    assert!(json.contains("\"target_tune_cpu\": \"alderlake\""));

    let target_clang_cpu_idx = json.find("\"target_clang_cpu\"").unwrap();
    let target_clang_tune_cpu_idx = json.find("\"target_clang_tune_cpu\"").unwrap();
    assert!(target_clang_cpu_idx < target_clang_tune_cpu_idx);
    assert!(json.contains("\"target_clang_tune_cpu\": \"-m'tune=alderlake'\""));

    let json_native = report_native.to_json().unwrap();
    assert!(json_native.contains("\"target_tune_cpu\": \"\""));
    assert!(json_native.contains("\"target_clang_tune_cpu\": \"\""));
}

#[test]
fn test_console_platforms_auto_target_cpu() {
    let consoles = [
        (Platform::Xboxone, Arch::X86_64, "xboxone", "btver2"),
        (Platform::Xboxxs, Arch::X86_64, "xboxxs", "znver2"),
        (Platform::Ps4, Arch::X86_64, "ps4", "btver2"),
        (Platform::Ps5, Arch::X86_64, "ps5", "znver2"),
        (Platform::Switch2, Arch::Arm64, "switch2", "cortex-a78c"),
        (Platform::Steamdeck, Arch::X86_64, "steamdeck", "znver2"),
        (Platform::Steammachine, Arch::X86_64, "steammachine", "znver4"),
    ];

    for (plat, arch, console_name, real_target) in consoles {
        // Verify is_console and console_target_cpu helpers
        assert!(plat.is_console());
        assert_eq!(plat.console_target_cpu(), Some(console_name));

        // 1. User does not set target_cpu (None) -> automatically set to console platform name, target_cpu to real target
        let rep_none = ArchFeaturesReport::evaluate(Some(plat), None, None).unwrap();
        assert_eq!(rep_none.platform, console_name);
        assert_eq!(rep_none.arch, arch.to_string());
        assert_eq!(rep_none.target_cpu, Some(real_target.to_string()));
        assert_eq!(rep_none.target_tune_cpu, Some(real_target.to_string()));
        assert_eq!(rep_none.target_clang_tune_cpu, Some(format!("-m'tune={}'", real_target)));

        // 2. User sets wrong target_cpu -> automatically set to console platform target
        let rep_wrong = ArchFeaturesReport::evaluate(Some(plat), None, Some("wrong_cpu_target")).unwrap();
        assert_eq!(rep_wrong.target_cpu, Some(real_target.to_string()));
        assert_eq!(rep_wrong.target_tune_cpu, Some(real_target.to_string()));
        assert_eq!(rep_wrong.target_clang_tune_cpu, Some(format!("-m'tune={}'", real_target)));

        let rep_generic = ArchFeaturesReport::evaluate(Some(plat), None, Some("generic")).unwrap();
        assert_eq!(rep_generic.target_cpu, Some(real_target.to_string()));
        assert_eq!(rep_generic.target_tune_cpu, Some(real_target.to_string()));
        assert_eq!(rep_generic.target_clang_tune_cpu, Some(format!("-m'tune={}'", real_target)));

        // 3. User sets correct target_cpu (console name) -> target_cpu is real target
        let rep_correct = ArchFeaturesReport::evaluate(Some(plat), None, Some(console_name)).unwrap();
        assert_eq!(rep_correct.target_cpu, Some(real_target.to_string()));
        assert_eq!(rep_correct.target_tune_cpu, Some(real_target.to_string()));
        assert_eq!(rep_correct.target_clang_tune_cpu, Some(format!("-m'tune={}'", real_target)));

        // 4. from_target_cpu with wrong target name also auto-resolves for console
        let rep_from_target_cpu = ArchFeaturesReport::from_target_cpu(plat, arch, "invalid_target").unwrap();
        assert_eq!(rep_from_target_cpu.target_cpu, Some(real_target.to_string()));
        assert_eq!(rep_from_target_cpu.target_tune_cpu, Some(real_target.to_string()));
        assert_eq!(rep_from_target_cpu.target_clang_tune_cpu, Some(format!("-m'tune={}'", real_target)));
    }

    // Non-console platforms should not be consoles
    assert!(!Platform::Windows.is_console());
    assert!(!Platform::Linux.is_console());
    assert!(!Platform::Macosx.is_console());
    assert!(!Platform::Android.is_console());
    assert_eq!(Platform::Windows.console_target_cpu(), None);
}

#[test]
fn test_target_clang_triple_all_platforms() {
    // Android: aarch64 & x86_64
    let android_arm = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Android), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("24"), None, false,
    ).unwrap();
    assert_eq!(android_arm.target_clang_triple, Some("--target='aarch64-none-linux-android24'".to_string()));
    assert_eq!(android_arm.target_os_level, Some("24".to_string()));
    assert_eq!(android_arm.target_runtime_level, Some("".to_string()));

    let android_arm_30 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Android), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("30.0.99999999"), None, false,
    ).unwrap();
    assert_eq!(android_arm_30.target_clang_triple, Some("--target='aarch64-none-linux-android30'".to_string()));
    assert_eq!(android_arm_30.target_os_level, Some("30.0.99999999".to_string()));

    let android_x64 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Android), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("28"), None, false,
    ).unwrap();
    assert_eq!(android_x64.target_clang_triple, Some("--target='x86_64-none-linux-android28'".to_string()));

    // Android out-of-range error
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Android), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("23"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Android), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("31"), None, false,
    ).is_err());

    // Linux: aarch64, x86_64, riscv64
    let linux_arm = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::Arm64), None, None, None, None, None, None,
        None, Some("2.17"), false,
    ).unwrap();
    assert_eq!(linux_arm.target_clang_triple, Some("--target='aarch64-unknown-linux-gnu2.17'".to_string()));
    assert_eq!(linux_arm.target_runtime_level, Some("2.17".to_string()));
    assert_eq!(linux_arm.target_os_level, Some("".to_string()));

    let linux_arm_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::Arm64), None, None, None, None, None, None,
        None, Some("2.17.0"), false,
    ).unwrap();
    assert_eq!(linux_arm_patch.target_clang_triple, Some("--target='aarch64-unknown-linux-gnu2.17.0'".to_string()));
    assert_eq!(linux_arm_patch.target_runtime_level, Some("2.17.0".to_string()));

    let linux_x64 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2.44"), false,
    ).unwrap();
    assert_eq!(linux_x64.target_clang_triple, Some("--target='x86_64-unknown-linux-gnu2.44'".to_string()));

    let linux_x64_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2.44.9"), false,
    ).unwrap();
    assert_eq!(linux_x64_patch.target_clang_triple, Some("--target='x86_64-unknown-linux-gnu2.44.9'".to_string()));
    assert_eq!(linux_x64_patch.target_runtime_level, Some("2.44.9".to_string()));

    let linux_rv = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::Riscv64), None, None, None, None, None, None,
        None, Some("2.34"), false,
    ).unwrap();
    assert_eq!(linux_rv.target_clang_triple, Some("--target='riscv64-unknown-linux-gnu2.34'".to_string()));

    let linux_rv_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::Riscv64), None, None, None, None, None, None,
        None, Some("2.34.1"), false,
    ).unwrap();
    assert_eq!(linux_rv_patch.target_clang_triple, Some("--target='riscv64-unknown-linux-gnu2.34.1'".to_string()));

    // Linux integer glibc representation
    let linux_int = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("234"), false,
    ).unwrap();
    assert_eq!(linux_int.target_clang_triple, Some("--target='x86_64-unknown-linux-gnu2.34'".to_string()));

    let linux_int_4digit_start = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2170"), false,
    ).unwrap();
    assert_eq!(linux_int_4digit_start.target_clang_triple, Some("--target='x86_64-unknown-linux-gnu2.17.0'".to_string()));

    let linux_int_4digit_end = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2449"), false,
    ).unwrap();
    assert_eq!(linux_int_4digit_end.target_clang_triple, Some("--target='x86_64-unknown-linux-gnu2.44.9'".to_string()));

    // Linux glibc out-of-range error
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2.16"), false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2.45"), false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2.16.9"), false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2.17.10"), false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2.44.10"), false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2.45.0"), false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2169"), false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2450"), false,
    ).is_err());

    // Steam Deck & Steam Machine
    let deck = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Steamdeck), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2.38"), false,
    ).unwrap();
    assert_eq!(deck.target_clang_triple, Some("--target='x86_64-unknown-linux-gnu2.38'".to_string()));

    let deck_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Steamdeck), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2.38.2"), false,
    ).unwrap();
    assert_eq!(deck_patch.target_clang_triple, Some("--target='x86_64-unknown-linux-gnu2.38.2'".to_string()));

    let steammachine = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Steammachine), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2.35"), false,
    ).unwrap();
    assert_eq!(steammachine.target_clang_triple, Some("--target='x86_64-unknown-linux-gnu2.35'".to_string()));

    let steammachine_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Steammachine), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2.35.0"), false,
    ).unwrap();
    assert_eq!(steammachine_patch.target_clang_triple, Some("--target='x86_64-unknown-linux-gnu2.35.0'".to_string()));

    // FreeBSD: aarch64, x86_64, riscv64
    let fbsd_arm = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("13.0"), None, false,
    ).unwrap();
    assert_eq!(fbsd_arm.target_clang_triple, Some("--target='aarch64-unknown-freebsd13.0'".to_string()));
    assert_eq!(fbsd_arm.target_os_level, Some("13.0".to_string()));
    assert_eq!(fbsd_arm.target_runtime_level, Some("".to_string()));

    let fbsd_arm_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("13.0.0"), None, false,
    ).unwrap();
    assert_eq!(fbsd_arm_patch.target_clang_triple, Some("--target='aarch64-unknown-freebsd13.0.0'".to_string()));
    assert_eq!(fbsd_arm_patch.target_os_level, Some("13.0.0".to_string()));

    let fbsd_arm_major = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("13"), None, false,
    ).unwrap();
    assert_eq!(fbsd_arm_major.target_clang_triple, Some("--target='aarch64-unknown-freebsd13.0'".to_string()));
    assert_eq!(fbsd_arm_major.target_os_level, Some("13.0".to_string()));

    let fbsd_x64 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("15.3"), None, false,
    ).unwrap();
    assert_eq!(fbsd_x64.target_clang_triple, Some("--target='x86_64-unknown-freebsd15.3'".to_string()));

    let fbsd_x64_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("15.3.9"), None, false,
    ).unwrap();
    assert_eq!(fbsd_x64_patch.target_clang_triple, Some("--target='x86_64-unknown-freebsd15.3.9'".to_string()));
    assert_eq!(fbsd_x64_patch.target_os_level, Some("15.3.9".to_string()));

    let fbsd_x64_major = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("15"), None, false,
    ).unwrap();
    assert_eq!(fbsd_x64_major.target_clang_triple, Some("--target='x86_64-unknown-freebsd15.0'".to_string()));
    assert_eq!(fbsd_x64_major.target_os_level, Some("15.0".to_string()));

    let fbsd_rv = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::Riscv64), None, None, None, None, None, None,
        Some("14.0"), None, false,
    ).unwrap();
    assert_eq!(fbsd_rv.target_clang_triple, Some("--target='riscv64-unknown-freebsd14.0'".to_string()));

    let fbsd_rv_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::Riscv64), None, None, None, None, None, None,
        Some("14.0.2"), None, false,
    ).unwrap();
    assert_eq!(fbsd_rv_patch.target_clang_triple, Some("--target='riscv64-unknown-freebsd14.0.2'".to_string()));

    // FreeBSD integer representation
    let fbsd_int_3digit = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("153"), None, false,
    ).unwrap();
    assert_eq!(fbsd_int_3digit.target_clang_triple, Some("--target='x86_64-unknown-freebsd15.3'".to_string()));

    let fbsd_int_4digit_start = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("1300"), None, false,
    ).unwrap();
    assert_eq!(fbsd_int_4digit_start.target_clang_triple, Some("--target='x86_64-unknown-freebsd13.0.0'".to_string()));

    let fbsd_int_4digit_end = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("1539"), None, false,
    ).unwrap();
    assert_eq!(fbsd_int_4digit_end.target_clang_triple, Some("--target='x86_64-unknown-freebsd15.3.9'".to_string()));

    // FreeBSD out of range
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("12"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("16"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("12.9"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("15.4"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("12.9.9"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("13.0.10"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("15.3.10"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("15.4.0"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("16.0.0"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("1299"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("1540"), None, false,
    ).is_err());

    // Windows, Xboxone, Xboxxs
    let win_x64_default = ArchFeaturesReport::evaluate(Some(Platform::Windows), Some(Arch::X86_64), None).unwrap();
    assert_eq!(win_x64_default.target_clang_triple, Some("--target='x86_64-pc-windows-msvc19.51.36231'".to_string()));
    assert_eq!(win_x64_default.target_runtime_level, Some("19.51.36231".to_string()));
    assert_eq!(win_x64_default.target_os_level, Some("".to_string()));

    let win_x64_1930 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Windows), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("1930"), false,
    ).unwrap();
    assert_eq!(win_x64_1930.target_clang_triple, Some("--target='x86_64-pc-windows-msvc19.30'".to_string()));
    assert_eq!(win_x64_1930.target_runtime_level, Some("19.30".to_string()));

    let win_x64_195299999 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Windows), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("195299999"), false,
    ).unwrap();
    assert_eq!(win_x64_195299999.target_clang_triple, Some("--target='x86_64-pc-windows-msvc19.52.99999'".to_string()));
    assert_eq!(win_x64_195299999.target_runtime_level, Some("19.52.99999".to_string()));

    let win_arm_1952 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Windows), Some(Arch::Arm64), None, None, None, None, None, None,
        None, Some("1952"), false,
    ).unwrap();
    assert_eq!(win_arm_1952.target_clang_triple, Some("--target='aarch64-pc-windows-msvc19.52'".to_string()));

    let win_armec = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Windows), Some(Arch::Arm64EC), None, None, None, None, None, None,
        None, Some("19.41.34120"), false,
    ).unwrap();
    assert_eq!(win_armec.target_clang_triple, Some("--target='arm64ec-pc-windows-msvc19.41.34120'".to_string()));

    let xbox1 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Xboxone), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("193030000"), false,
    ).unwrap();
    assert_eq!(xbox1.target_clang_triple, Some("--target='x86_64-pc-windows-msvc19.30.30000'".to_string()));

    let xboxxs = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Xboxxs), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("19.50"), false,
    ).unwrap();
    assert_eq!(xboxxs.target_clang_triple, Some("--target='x86_64-pc-windows-msvc19.50'".to_string()));

    // Windows MSVC out of range
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Windows), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("1929"), false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Windows), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("1953"), false,
    ).is_err());

    // macOS
    let mac_arm_15 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("15"), None, false,
    ).unwrap();
    assert_eq!(mac_arm_15.target_clang_triple, Some("--target='aarch64-apple-macosx15.0'".to_string()));
    assert_eq!(mac_arm_15.target_os_level, Some("15.0".to_string()));

    let mac_arm64e_26 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64E), None, None, None, None, None, None,
        Some("26.0"), None, false,
    ).unwrap();
    assert_eq!(mac_arm64e_26.target_clang_triple, Some("--target='arm64e-apple-macosx26.0'".to_string()));

    let mac_x64_26 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("Tahoe"), None, false,
    ).unwrap();
    assert_eq!(mac_x64_26.target_clang_triple, Some("--target='x86_64-apple-macosx26.0'".to_string()));

    // macOS Golden Gate (27.0) drops Intel support
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("27.0"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("Golden Gate"), None, false,
    ).is_err());

    // macOS Golden Gate on ARM is valid
    let mac_arm_27 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27"), None, false,
    ).unwrap();
    assert_eq!(mac_arm_27.target_clang_triple, Some("--target='aarch64-apple-macosx27.0'".to_string()));

    // macOS minor version range 0..99
    let mac_15_1 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("15.1"), None, false,
    ).unwrap();
    assert_eq!(mac_15_1.target_os_level, Some("15.1".to_string()));
    assert_eq!(mac_15_1.target_clang_triple, Some("--target='aarch64-apple-macosx15.1'".to_string()));

    let mac_15_99 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("15.99"), None, false,
    ).unwrap();
    assert_eq!(mac_15_99.target_os_level, Some("15.99".to_string()));

    let mac_26_99 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.99"), None, false,
    ).unwrap();
    assert_eq!(mac_26_99.target_os_level, Some("26.99".to_string()));

    let mac_27_99 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.99"), None, false,
    ).unwrap();
    assert_eq!(mac_27_99.target_os_level, Some("27.99".to_string()));

    // macOS patch versions (0..99)
    let mac_15_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("15.0.0"), None, false,
    ).unwrap();
    assert_eq!(mac_15_patch.target_os_level, Some("15.0.0".to_string()));
    assert_eq!(mac_15_patch.target_clang_triple, Some("--target='aarch64-apple-macosx15.0.0'".to_string()));

    let mac_15_patch_max = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("15.99.99"), None, false,
    ).unwrap();
    assert_eq!(mac_15_patch_max.target_os_level, Some("15.99.99".to_string()));

    let mac_26_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.0.0"), None, false,
    ).unwrap();
    assert_eq!(mac_26_patch.target_os_level, Some("26.0.0".to_string()));
    assert_eq!(mac_26_patch.target_clang_triple, Some("--target='aarch64-apple-macosx26.0.0'".to_string()));

    let mac_26_patch_max = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.99.99"), None, false,
    ).unwrap();
    assert_eq!(mac_26_patch_max.target_os_level, Some("26.99.99".to_string()));

    let mac_27_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.0.0"), None, false,
    ).unwrap();
    assert_eq!(mac_27_patch.target_os_level, Some("27.0.0".to_string()));
    assert_eq!(mac_27_patch.target_clang_triple, Some("--target='aarch64-apple-macosx27.0.0'".to_string()));

    let mac_27_patch_max = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.99.99"), None, false,
    ).unwrap();
    assert_eq!(mac_27_patch_max.target_os_level, Some("27.99.99".to_string()));

    // macOS out of range minor version (> 99) or patch version (> 99)
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("15.100"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("15.0.100"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.100"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.0.100"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.100"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.0.100"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("15.1.2.3"), None, false,
    ).is_err());

    // macOS out of range major version
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("14.0"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("16.0"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("25.0"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("28.0"), None, false,
    ).is_err());

    // iOS
    let ios_arm = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.0"), None, false,
    ).unwrap();
    assert_eq!(ios_arm.target_clang_triple, Some("--target='aarch64-apple-ios26.0'".to_string()));

    let ios_26_99 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.99"), None, false,
    ).unwrap();
    assert_eq!(ios_26_99.target_os_level, Some("26.99".to_string()));
    assert_eq!(ios_26_99.target_clang_triple, Some("--target='aarch64-apple-ios26.99'".to_string()));

    let ios_27_99 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.99"), None, false,
    ).unwrap();
    assert_eq!(ios_27_99.target_os_level, Some("27.99".to_string()));
    assert_eq!(ios_27_99.target_clang_triple, Some("--target='aarch64-apple-ios27.99'".to_string()));

    let ios_26_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.0.0"), None, false,
    ).unwrap();
    assert_eq!(ios_26_patch.target_os_level, Some("26.0.0".to_string()));
    assert_eq!(ios_26_patch.target_clang_triple, Some("--target='aarch64-apple-ios26.0.0'".to_string()));

    let ios_27_patch_max = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.99.99"), None, false,
    ).unwrap();
    assert_eq!(ios_27_patch_max.target_os_level, Some("27.99.99".to_string()));
    assert_eq!(ios_27_patch_max.target_clang_triple, Some("--target='aarch64-apple-ios27.99.99'".to_string()));

    // iOS out of range minor version (> 99), patch version (> 99), or major version
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.100"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.0.100"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.100"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.0.100"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("15.0"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("25.0"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("28.0"), None, false,
    ).is_err());

    let ios_sim = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27"), None, true,
    ).unwrap();
    assert_eq!(ios_sim.target_clang_triple, Some("--target='aarch64-apple-ios27.0-simulator'".to_string()));

    let ios_sim_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.1.2"), None, true,
    ).unwrap();
    assert_eq!(ios_sim_patch.target_clang_triple, Some("--target='aarch64-apple-ios26.1.2-simulator'".to_string()));

    let ios_arm64e = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64E), None, None, None, None, None, None,
        Some("26.0"), None, false,
    ).unwrap();
    assert_eq!(ios_arm64e.target_clang_triple, Some("--target='arm64e-apple-ios26.0'".to_string()));

    // iOS simulator not supported with arm64e
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64E), None, None, None, None, None, None,
        Some("26.0"), None, true,
    ).is_err());

    // tvOS
    let tvos_arm = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Tvos), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26"), None, false,
    ).unwrap();
    assert_eq!(tvos_arm.target_clang_triple, Some("--target='aarch64-apple-tvos26.0'".to_string()));

    let tvos_26_5 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Tvos), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.5"), None, false,
    ).unwrap();
    assert_eq!(tvos_26_5.target_os_level, Some("26.5".to_string()));
    assert_eq!(tvos_26_5.target_clang_triple, Some("--target='aarch64-apple-tvos26.5'".to_string()));

    let tvos_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Tvos), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.0.0"), None, false,
    ).unwrap();
    assert_eq!(tvos_patch.target_os_level, Some("26.0.0".to_string()));
    assert_eq!(tvos_patch.target_clang_triple, Some("--target='aarch64-apple-tvos26.0.0'".to_string()));

    let tvos_patch_max = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Tvos), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.99.99"), None, false,
    ).unwrap();
    assert_eq!(tvos_patch_max.target_os_level, Some("27.99.99".to_string()));

    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Tvos), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.100"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Tvos), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.0.100"), None, false,
    ).is_err());

    let tvos_sim = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Tvos), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.0"), None, true,
    ).unwrap();
    assert_eq!(tvos_sim.target_clang_triple, Some("--target='aarch64-apple-tvos27.0-simulator'".to_string()));

    let tvos_sim_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Tvos), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.1.4"), None, true,
    ).unwrap();
    assert_eq!(tvos_sim_patch.target_clang_triple, Some("--target='aarch64-apple-tvos27.1.4-simulator'".to_string()));

    // tvOS arm64e not supported
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Tvos), Some(Arch::Arm64E), None, None, None, None, None, None,
        None, None, false,
    ).is_err());

    // xrOS
    let xros_arm = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Xros), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.0"), None, false,
    ).unwrap();
    assert_eq!(xros_arm.target_clang_triple, Some("--target='aarch64-apple-xros26.0'".to_string()));

    let xros_27_12 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Xros), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.12"), None, false,
    ).unwrap();
    assert_eq!(xros_27_12.target_os_level, Some("27.12".to_string()));
    assert_eq!(xros_27_12.target_clang_triple, Some("--target='aarch64-apple-xros27.12'".to_string()));

    let xros_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Xros), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.0.0"), None, false,
    ).unwrap();
    assert_eq!(xros_patch.target_os_level, Some("26.0.0".to_string()));
    assert_eq!(xros_patch.target_clang_triple, Some("--target='aarch64-apple-xros26.0.0'".to_string()));

    let xros_patch_max = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Xros), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.99.99"), None, false,
    ).unwrap();
    assert_eq!(xros_patch_max.target_os_level, Some("27.99.99".to_string()));

    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Xros), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.100"), None, false,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Xros), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.0.100"), None, false,
    ).is_err());

    let xros_sim = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Xros), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27"), None, true,
    ).unwrap();
    assert_eq!(xros_sim.target_clang_triple, Some("--target='aarch64-apple-xros27.0-simulator'".to_string()));

    let xros_sim_patch = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Xros), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("26.5.10"), None, true,
    ).unwrap();
    assert_eq!(xros_sim_patch.target_clang_triple, Some("--target='aarch64-apple-xros26.5.10-simulator'".to_string()));

    // xrOS arm64e not supported
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Xros), Some(Arch::Arm64E), None, None, None, None, None, None,
        None, None, false,
    ).is_err());

    // Simulator rejected on non-mobile platforms
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Windows), Some(Arch::X86_64), None, None, None, None, None, None,
        None, None, true,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::X86_64), None, None, None, None, None, None,
        None, None, true,
    ).is_err());
    assert!(ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        None, None, true,
    ).is_err());

    // Consoles: PS4, PS5, Switch 2
    let ps4 = ArchFeaturesReport::evaluate(Some(Platform::Ps4), Some(Arch::X86_64), None).unwrap();
    assert_eq!(ps4.target_clang_triple, Some("--target='x86_64-sie-ps4'".to_string()));
    assert_eq!(ps4.target_os_level, Some("".to_string()));
    assert_eq!(ps4.target_runtime_level, Some("".to_string()));

    let ps5 = ArchFeaturesReport::evaluate(Some(Platform::Ps5), Some(Arch::X86_64), None).unwrap();
    assert_eq!(ps5.target_clang_triple, Some("--target='x86_64-sie-ps5'".to_string()));
    assert_eq!(ps5.target_os_level, Some("".to_string()));
    assert_eq!(ps5.target_runtime_level, Some("".to_string()));

    let switch2 = ArchFeaturesReport::evaluate(Some(Platform::Switch2), Some(Arch::Arm64), None).unwrap();
    assert_eq!(switch2.target_clang_triple, Some("--target='aarch64-nintendo-nx2'".to_string()));
    assert_eq!(switch2.target_os_level, Some("".to_string()));
    assert_eq!(switch2.target_runtime_level, Some("".to_string()));

    // JSON serialization test
    let json = android_arm.to_json().unwrap();
    assert!(json.contains("\"target_clang_triple\": \"--target='aarch64-none-linux-android24'\""));
    assert!(json.contains("\"target_os_level\": \"24\""));
    assert!(json.contains("\"target_runtime_level\": \"\""));
    assert!(json.contains("\"target_is_simulator\": false"));

    let json_sim = ios_sim.to_json().unwrap();
    assert!(json_sim.contains("\"target_is_simulator\": true"));

    let json_ps4 = ps4.to_json().unwrap();
    assert!(json_ps4.contains("\"target_clang_triple\": \"--target='x86_64-sie-ps4'\""));
    assert!(json_ps4.contains("\"target_os_level\": \"\""));
    assert!(json_ps4.contains("\"target_runtime_level\": \"\""));
    assert!(json_ps4.contains("\"target_is_simulator\": false"));
}

#[test]
fn test_macos_intel_drop_gte_27() {
    let mac_x64_15 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("15.0"), None, false,
    ).unwrap();
    assert_eq!(mac_x64_15.target_clang_triple, Some("--target='x86_64-apple-macosx15.0'".to_string()));

    let mac_x64_26 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("26.0"), None, false,
    ).unwrap();
    assert_eq!(mac_x64_26.target_clang_triple, Some("--target='x86_64-apple-macosx26.0'".to_string()));

    let err_27 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("27.0"), None, false,
    ).unwrap_err();
    assert_eq!(err_27, "macOS 27 (Golden Gate) and beyond completely drops Intel support");

    let err_golden_gate = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::X86_64), None, None, None, None, None, None,
        Some("Golden Gate"), None, false,
    ).unwrap_err();
    assert_eq!(err_golden_gate, "macOS 27 (Golden Gate) and beyond completely drops Intel support");

    let mac_arm_27 = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), None, None, None, None, None, None,
        Some("27.0"), None, false,
    ).unwrap();
    assert_eq!(mac_arm_27.target_clang_triple, Some("--target='aarch64-apple-macosx27.0'".to_string()));
}

#[test]
fn test_restructured_canonical_filenames_all_platforms() {
    // 1. Android: -ndk-{Major.Minor}.json
    let android = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Android), Some(Arch::Arm64), Some("cortex-a78"), None, None, None, None, None,
        Some("24"), None, false,
    ).unwrap();
    assert_eq!(android.filename(), "android-aarch64-cortex-a78-armv8.4-a-vl128-ndk-24.json");

    let android_minor = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Android), Some(Arch::Arm64), Some("cortex-a78"), None, None, None, None, None,
        Some("24.0"), None, false,
    ).unwrap();
    assert_eq!(android_minor.filename(), "android-aarch64-cortex-a78-armv8.4-a-vl128-ndk-24.0.json");

    // 2. FreeBSD: -freebsd-{Major.Minor}.json
    let freebsd = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Freebsd), Some(Arch::X86_64), Some("generic"), None, None, None, None, None,
        Some("14.0"), None, false,
    ).unwrap();
    assert_eq!(freebsd.filename(), "freebsd-x86_64-generic-x86-64-v2-sse4.2-vl128-freebsd-14.0.json");

    // 3. Linux | Steamdeck | Steammachine: -glibc-{Major.Minor}.json
    let linux = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Linux), Some(Arch::X86_64), Some("znver3"), None, None, None, None, None,
        None, Some("2.34"), false,
    ).unwrap();
    assert_eq!(linux.filename(), "linux-x86_64-znver3-avx2-vl256-glibc-2.34.json");

    let steamdeck = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Steamdeck), Some(Arch::X86_64), Some("znver2"), None, None, None, None, None,
        None, Some("2.17"), false,
    ).unwrap();
    assert_eq!(steamdeck.filename(), "steamdeck-x86_64-znver2-avx2-vl128-glibc-2.17.json");

    let steammachine = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Steammachine), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("2.17"), false,
    ).unwrap();
    assert_eq!(steammachine.filename(), "steammachine-x86_64-znver4-avx512-vl256-glibc-2.17.json");

    // 4. Windows | Xboxone | Xboxxs: -msvc-{Major.Minor}.json
    let windows = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Windows), Some(Arch::X86_64), Some("znver3"), None, None, None, None, None,
        None, Some("19.51.36231"), false,
    ).unwrap();
    assert_eq!(windows.filename(), "windows-x86_64-znver3-avx2-vl256-msvc-19.51.json");

    let xboxone = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Xboxone), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("19.30"), false,
    ).unwrap();
    assert_eq!(xboxone.filename(), "xboxone-x86_64-btver2-avx-vl128-msvc-19.30.json");

    let xboxxs = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Xboxxs), Some(Arch::X86_64), None, None, None, None, None, None,
        None, Some("19.30"), false,
    ).unwrap();
    assert_eq!(xboxxs.filename(), "xboxxs-x86_64-znver2-avx2-vl256-msvc-19.30.json");

    // 5. Macosx: -macosx-{Major.Minor}.json
    let macosx = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Macosx), Some(Arch::Arm64), Some("apple-m4"), None, None, None, None, None,
        Some("15.0"), None, false,
    ).unwrap();
    assert_eq!(macosx.filename(), "macosx-aarch64-apple-m4-armv9.2-a-vl128-macosx-15.0.json");

    // 6. IOS: -ios-{Major.Minor}.json and -ios-{Major.Minor}-simulator.json
    let ios_device = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), Some("apple-m4"), None, None, None, None, None,
        Some("26.0"), None, false,
    ).unwrap();
    assert_eq!(ios_device.filename(), "ios-aarch64-apple-m4-armv9.2-a-vl128-ios-26.0.json");

    let ios_sim = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Ios), Some(Arch::Arm64), Some("apple-m4"), None, None, None, None, None,
        Some("26.0"), None, true,
    ).unwrap();
    assert_eq!(ios_sim.filename(), "ios-aarch64-apple-m4-armv9.2-a-vl128-ios-26.0-simulator.json");

    // 7. TVOS: -tvos-{Major.Minor}.json and -tvos-{Major.Minor}-simulator.json
    let tvos_device = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Tvos), Some(Arch::Arm64), Some("apple-m4"), None, None, None, None, None,
        Some("26.0"), None, false,
    ).unwrap();
    assert_eq!(tvos_device.filename(), "tvos-aarch64-apple-m4-armv9.2-a-vl128-tvos-26.0.json");

    let tvos_sim = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Tvos), Some(Arch::Arm64), Some("apple-m4"), None, None, None, None, None,
        Some("26.0"), None, true,
    ).unwrap();
    assert_eq!(tvos_sim.filename(), "tvos-aarch64-apple-m4-armv9.2-a-vl128-tvos-26.0-simulator.json");

    // 8. XrOS: -xros-{Major.Minor}.json and -xros-{Major.Minor}-simulator.json
    let xros_device = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Xros), Some(Arch::Arm64), Some("apple-m4"), None, None, None, None, None,
        Some("26.0"), None, false,
    ).unwrap();
    assert_eq!(xros_device.filename(), "xros-aarch64-apple-m4-armv9.2-a-vl128-xros-26.0.json");

    let xros_sim = ArchFeaturesReport::evaluate_target_triple(
        Some(Platform::Xros), Some(Arch::Arm64), Some("apple-m4"), None, None, None, None, None,
        Some("26.0"), None, true,
    ).unwrap();
    assert_eq!(xros_sim.filename(), "xros-aarch64-apple-m4-armv9.2-a-vl128-xros-26.0-simulator.json");

    // 9. Ps4 | Ps5 | Switch2: .json without level suffix
    let ps4 = ArchFeaturesReport::from_target(Platform::Ps4, Arch::X86_64).unwrap();
    assert_eq!(ps4.filename(), "ps4-x86_64-btver2-avx-vl128.json");

    let ps5 = ArchFeaturesReport::from_target(Platform::Ps5, Arch::X86_64).unwrap();
    assert_eq!(ps5.filename(), "ps5-x86_64-znver2-avx2-vl128.json");

    let sw2 = ArchFeaturesReport::from_target(Platform::Switch2, Arch::Arm64).unwrap();
    assert_eq!(sw2.filename(), "switch2-aarch64-cortex-a78c-armv8.4-a-vl128.json");

    // 10. Empty or unset level falls back to standard base.json
    let mut empty_lvl_report = windows.clone();
    empty_lvl_report.target_runtime_level = Some("".to_string());
    assert_eq!(empty_lvl_report.filename(), "windows-x86_64-znver3-avx2-vl256.json");

    let mut none_lvl_report = windows.clone();
    none_lvl_report.target_runtime_level = None;
    assert_eq!(none_lvl_report.filename(), "windows-x86_64-znver3-avx2-vl256.json");
}

#[test]
fn test_msvc_arch_vlen_platforms_ignored_and_empty_in_json() {
    let non_msvc_platforms = [
        (Platform::Macosx, Arch::Arm64),
        (Platform::Linux, Arch::X86_64),
        (Platform::Android, Arch::Arm64),
        (Platform::Ios, Arch::Arm64),
        (Platform::Switch2, Arch::Arm64),
        (Platform::Ps4, Arch::X86_64),
        (Platform::Ps5, Arch::X86_64),
    ];

    for (p, a) in non_msvc_platforms {
        let report = ArchFeaturesReport::from_target(p, a).unwrap();
        assert_eq!(report.target_msvc_arch, Some("".to_string()));
        assert_eq!(report.target_msvc_vlen, Some("".to_string()));

        let json = report.to_json().unwrap();
        assert!(json.contains("\"target_msvc_arch\": \"\""), "JSON for platform {} must contain empty target_msvc_arch", p);
        assert!(json.contains("\"target_msvc_vlen\": \"\""), "JSON for platform {} must contain empty target_msvc_vlen", p);

        // Deserialization round-trip
        let round_trip: ArchFeaturesReport = serde_json::from_str(&json).unwrap();
        assert_eq!(round_trip.target_msvc_arch, Some("".to_string()));
        assert_eq!(round_trip.target_msvc_vlen, Some("".to_string()));
    }

    let msvc_platforms = [
        (Platform::Windows, Arch::X86_64),
        (Platform::Xboxone, Arch::X86_64),
        (Platform::Xboxxs, Arch::X86_64),
    ];

    for (p, a) in msvc_platforms {
        let report = ArchFeaturesReport::from_target(p, a).unwrap();
        assert!(report.target_msvc_arch.as_ref().map_or(false, |s| !s.is_empty()));
        let json = report.to_json().unwrap();
        assert!(!json.contains("\"target_msvc_arch\": \"\""));
    }
}

#[test]
fn test_apple_targets_platform_compatibility() {
    let apple_targets = [
        TargetCpuArchitectureArm64::Apple_A10,
        TargetCpuArchitectureArm64::Apple_A11,
        TargetCpuArchitectureArm64::Apple_A12,
        TargetCpuArchitectureArm64::Apple_A13,
        TargetCpuArchitectureArm64::Apple_A14,
        TargetCpuArchitectureArm64::Apple_M1,
        TargetCpuArchitectureArm64::Apple_A15,
        TargetCpuArchitectureArm64::Apple_A16,
        TargetCpuArchitectureArm64::Apple_A17,
        TargetCpuArchitectureArm64::Apple_M2,
        TargetCpuArchitectureArm64::Apple_M3,
        TargetCpuArchitectureArm64::Apple_A18,
        TargetCpuArchitectureArm64::Apple_A19,
        TargetCpuArchitectureArm64::Apple_M4,
        TargetCpuArchitectureArm64::Apple_M5,
    ];

    let apple_platforms = [
        Platform::Macosx,
        Platform::Ios,
        Platform::Tvos,
        Platform::Xros,
    ];

    let non_apple_platforms = [
        Platform::Android,
        Platform::Windows,
        Platform::Linux,
        Platform::Freebsd,
        Platform::Xboxone,
        Platform::Xboxxs,
        Platform::Ps4,
        Platform::Ps5,
        Platform::Switch2,
        Platform::Steamdeck,
        Platform::Steammachine,
    ];

    // Verify is_apple on targets
    for target in &apple_targets {
        assert!(target.is_apple(), "Target {:?} must be an Apple target", target);
    }
    assert!(!TargetCpuArchitectureArm64::Generic.is_apple());
    assert!(!TargetCpuArchitectureArm64::Cortex_A78.is_apple());
    assert!(!TargetCpuArchitectureArm64::Switch2.is_apple());
    assert!(!TargetCpuArchitectureArm64::Oryon_1.is_apple());

    // Verify is_apple on platforms
    for platform in &apple_platforms {
        assert!(platform.is_apple(), "Platform {:?} must be an Apple platform", platform);
    }
    for platform in &non_apple_platforms {
        assert!(!platform.is_apple(), "Platform {:?} must not be an Apple platform", platform);
    }

    // Verify compatibility across all combinations
    for target in &apple_targets {
        for platform in &apple_platforms {
            assert!(
                target.is_platform_compatible(*platform),
                "Target {:?} must be compatible with {:?}",
                target,
                platform
            );
            assert!(
                platform.is_target_arm64_compatible(*target),
                "Platform {:?} must be compatible with {:?}",
                platform,
                target
            );
        }

        for platform in &non_apple_platforms {
            assert!(
                !target.is_platform_compatible(*platform),
                "Target {:?} must NOT be compatible with {:?}",
                target,
                platform
            );
            assert!(
                !platform.is_target_arm64_compatible(*target),
                "Platform {:?} must NOT be compatible with {:?}",
                platform,
                target
            );
        }
    }

    // Test evaluate rejection on non-Apple Arm64-compatible platforms (non-console)
    let arm64_non_apple = [
        Platform::Windows,
        Platform::Linux,
        Platform::Android,
        Platform::Freebsd,
    ];

    for platform in arm64_non_apple {
        for target in &apple_targets {
            let target_name = TargetCpuArchitectureArm64Names::name(*target);
            let res = ArchFeaturesReport::evaluate(Some(platform), Some(Arch::Arm64), Some(target_name));
            assert!(
                res.is_err(),
                "Expected error evaluating Apple target {} on platform {}, got Ok",
                target_name,
                platform
            );
            let err_msg = res.unwrap_err();
            assert!(
                err_msg.contains("is incompatible with platform"),
                "Error message should indicate incompatibility, got: {}",
                err_msg
            );
        }
    }

    // Test evaluate success on Apple platforms
    for platform in apple_platforms {
        let res = ArchFeaturesReport::evaluate(Some(platform), Some(Arch::Arm64), Some("apple-m4"));
        assert!(res.is_ok(), "Expected Ok evaluating apple-m4 on {}, got: {:?}", platform, res.err());
    }
}

#[test]
fn test_freebsd_riscv64_probe_defaults() {
    use arch::riscv64::freebsd::FreeBsdRiscv64Probe;

    let default_probe = FreeBsdRiscv64Probe::default();
    assert_eq!(default_probe.hwcap, 0);
    assert_eq!(default_probe.hwcap2, 0);
    assert!(!default_probe.i_available);
    assert!(!default_probe.m_available);
    assert!(!default_probe.a_available);
    assert!(!default_probe.f_available);
    assert!(!default_probe.d_available);
    assert!(!default_probe.c_available);
    assert!(!default_probe.v_available);
    assert!(!default_probe.zba_available);
    assert!(!default_probe.zbb_available);
    assert!(!default_probe.zbs_available);

    let probe = FreeBsdRiscv64Probe::query();
    #[cfg(not(all(target_os = "freebsd", target_arch = "riscv64")))]
    {
        assert_eq!(probe, default_probe);
    }
    #[cfg(all(target_os = "freebsd", target_arch = "riscv64"))]
    {
        let _ = probe.hwcap;
    }
}

#[test]
fn test_cuda_arch_multivalue_formatting() {
    let parsed_semicolon = CudaArch::parse_multivalue("75;86;89;120").unwrap();
    assert_eq!(
        parsed_semicolon,
        vec![CudaArch::Sm_75, CudaArch::Sm_86, CudaArch::Sm_89, CudaArch::Sm_120]
    );
    assert_eq!(CudaArch::format_cuda_arch(&parsed_semicolon), "75;86;89;120");
    assert_eq!(CudaArch::format_cuda_sm_arch(&parsed_semicolon), "sm_75;sm_86;sm_89;sm_120");

    // Comma and plus delimiter support
    let parsed_comma = CudaArch::parse_multivalue("86,89,90").unwrap();
    assert_eq!(parsed_comma, vec![CudaArch::Sm_86, CudaArch::Sm_89, CudaArch::Sm_90]);
    assert_eq!(CudaArch::format_cuda_arch(&parsed_comma), "86;89;90");
    assert_eq!(CudaArch::format_cuda_sm_arch(&parsed_comma), "sm_86;sm_89;sm_90");

    let parsed_plus = CudaArch::parse_multivalue("86+89+90").unwrap();
    assert_eq!(parsed_plus, vec![CudaArch::Sm_86, CudaArch::Sm_89, CudaArch::Sm_90]);
    assert_eq!(CudaArch::format_cuda_arch(&parsed_plus), "86;89;90");
    assert_eq!(CudaArch::format_cuda_sm_arch(&parsed_plus), "sm_86;sm_89;sm_90");

    // Default when empty
    let empty = CudaArch::parse_multivalue("").unwrap();
    assert_eq!(empty, vec![CudaArch::Sm_75]);
    assert_eq!(CudaArch::format_cuda_arch(&empty), "75");
    assert_eq!(CudaArch::format_cuda_sm_arch(&empty), "sm_75");

    // Invalid arch fails
    assert!(CudaArch::parse_multivalue("999").is_err());
    assert!(CudaArch::parse_multivalue("invalid").is_err());
}

#[test]
fn test_cuda_arch_report_json() {
    // x86_64 target with custom cuda_arch
    let report_x64 = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Windows),
        Some(Arch::X86_64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        None,
        Some("86+89+90"),
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(report_x64.gpu_arch.as_deref(), Some("cuda"));
    assert_eq!(report_x64.cuda_arch.as_deref(), Some("86;89;90"));
    assert_eq!(report_x64.cuda_sm_arch.as_deref(), Some("sm_86;sm_89;sm_90"));
    let json_x64 = report_x64.to_json().unwrap();
    assert!(json_x64.contains("\"gpu_arch\": \"cuda\""));
    assert!(json_x64.contains("\"cuda_arch\": \"86;89;90\""));
    assert!(json_x64.contains("\"cuda_sm_arch\": \"sm_86;sm_89;sm_90\""));

    // arm64ec target with custom cuda_arch
    let report_arm = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Windows),
        Some(Arch::Arm64EC),
        Some("kryo"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        None,
        Some("86,89,90"),
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(report_arm.gpu_arch.as_deref(), Some("cuda"));
    assert_eq!(report_arm.cuda_arch.as_deref(), Some("86;89;90"));
    assert_eq!(report_arm.cuda_sm_arch.as_deref(), Some("sm_86;sm_89;sm_90"));

    // Default when no GPU args provided on x86_64: empty string
    let default_x64 = ArchFeaturesReport::evaluate(Some(Platform::Windows), Some(Arch::X86_64), Some("generic")).unwrap();
    assert_eq!(default_x64.gpu_arch.as_deref(), Some(""));
    assert_eq!(default_x64.cuda_arch.as_deref(), Some(""));
    assert_eq!(default_x64.cuda_sm_arch.as_deref(), Some(""));

    // Explicit gpu_arch = "cuda" gets desktop gaming default 75
    let gaming_cuda = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Windows),
        Some(Arch::X86_64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        Some("cuda"),
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(gaming_cuda.gpu_arch.as_deref(), Some("cuda"));
    assert_eq!(gaming_cuda.cuda_arch.as_deref(), Some("75"));
    assert_eq!(gaming_cuda.cuda_sm_arch.as_deref(), Some("sm_75"));

    // riscv64 is always empty string for cuda
    let report_riscv = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Linux),
        Some(Arch::Riscv64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        None,
        Some("86+89+90"),
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(report_riscv.cuda_arch.as_deref(), Some(""));
    assert_eq!(report_riscv.cuda_sm_arch.as_deref(), Some(""));
}

#[test]
fn test_rocm_arch_multivalue_formatting() {
    let parsed_semi = RocmArch::parse_multivalue("908;90a;950;1030").unwrap();
    assert_eq!(parsed_semi, vec![RocmArch::Gfx908, RocmArch::Gfx90a, RocmArch::Gfx950, RocmArch::Gfx1030]);
    assert_eq!(RocmArch::format_rocm_arch(&parsed_semi), "908;90a;950;1030");
    assert_eq!(RocmArch::format_rocm_gfx_arch(&parsed_semi), "gfx908;gfx90a;gfx950;gfx1030");

    let parsed_comma = RocmArch::parse_multivalue("908,90a,950,1030").unwrap();
    assert_eq!(parsed_comma, vec![RocmArch::Gfx908, RocmArch::Gfx90a, RocmArch::Gfx950, RocmArch::Gfx1030]);
    assert_eq!(RocmArch::format_rocm_arch(&parsed_comma), "908;90a;950;1030");
    assert_eq!(RocmArch::format_rocm_gfx_arch(&parsed_comma), "gfx908;gfx90a;gfx950;gfx1030");

    let parsed_plus = RocmArch::parse_multivalue("908+90a+950+1030").unwrap();
    assert_eq!(parsed_plus, vec![RocmArch::Gfx908, RocmArch::Gfx90a, RocmArch::Gfx950, RocmArch::Gfx1030]);
    assert_eq!(RocmArch::format_rocm_arch(&parsed_plus), "908;90a;950;1030");
    assert_eq!(RocmArch::format_rocm_gfx_arch(&parsed_plus), "gfx908;gfx90a;gfx950;gfx1030");

    // Default when empty is 1030
    let empty = RocmArch::parse_multivalue("").unwrap();
    assert_eq!(empty, vec![RocmArch::Gfx1030]);
    assert_eq!(RocmArch::format_rocm_arch(&empty), "1030");
    assert_eq!(RocmArch::format_rocm_gfx_arch(&empty), "gfx1030");

    // Invalid arch fails
    assert!(RocmArch::parse_multivalue("999").is_err());
    assert!(RocmArch::parse_multivalue("invalid").is_err());
}

#[test]
fn test_rocm_arch_report_json() {
    // x86_64 target with custom rocm_arch
    let report_x64 = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Windows),
        Some(Arch::X86_64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        None,
        None,
        Some("908+90a+950+1030"),
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(report_x64.gpu_arch.as_deref(), Some("rocm"));
    assert_eq!(report_x64.rocm_arch.as_deref(), Some("908;90a;950;1030"));
    assert_eq!(report_x64.rocm_gfx_arch.as_deref(), Some("gfx908;gfx90a;gfx950;gfx1030"));
    let json_x64 = report_x64.to_json().unwrap();
    assert!(json_x64.contains("\"rocm_arch\": \"908;90a;950;1030\""));
    assert!(json_x64.contains("\"rocm_gfx_arch\": \"gfx908;gfx90a;gfx950;gfx1030\""));

    // Default rocm arch for x86_64 when none provided: empty string
    let default_x64 = ArchFeaturesReport::evaluate(Some(Platform::Windows), Some(Arch::X86_64), Some("generic")).unwrap();
    assert_eq!(default_x64.rocm_arch.as_deref(), Some(""));
    assert_eq!(default_x64.rocm_gfx_arch.as_deref(), Some(""));

    // Explicit gpu_arch = "rocm" gets desktop gaming default 1030
    let gaming_rocm = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Windows),
        Some(Arch::X86_64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        Some("rocm"),
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(gaming_rocm.gpu_arch.as_deref(), Some("rocm"));
    assert_eq!(gaming_rocm.rocm_arch.as_deref(), Some("1030"));
    assert_eq!(gaming_rocm.rocm_gfx_arch.as_deref(), Some("gfx1030"));

    // arm64ec is empty string for rocm
    let report_arm = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Windows),
        Some(Arch::Arm64EC),
        Some("kryo"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        None,
        None,
        Some("908,90a,950,1030"),
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(report_arm.rocm_arch.as_deref(), Some(""));
    assert_eq!(report_arm.rocm_gfx_arch.as_deref(), Some(""));

    // riscv64 is always empty string for rocm
    let report_riscv = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Linux),
        Some(Arch::Riscv64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        None,
        None,
        Some("908+90a+950+1030"),
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(report_riscv.rocm_arch.as_deref(), Some(""));
    assert_eq!(report_riscv.rocm_gfx_arch.as_deref(), Some(""));
}

#[test]
fn test_gpu_arch_multivalue_formatting_and_native_rejection() {
    let parsed_multi = GPUArch::parse_multivalue("cuda;rocm;xpu;metal").unwrap();
    assert_eq!(
        parsed_multi,
        vec![GPUArch::Cuda, GPUArch::Rocm, GPUArch::Xpu, GPUArch::Metal]
    );
    assert_eq!(GPUArch::format_gpu_arch(&parsed_multi), "cuda;rocm;xpu;metal");

    let parsed_comma = GPUArch::parse_multivalue("cuda,xpu").unwrap();
    assert_eq!(parsed_comma, vec![GPUArch::Cuda, GPUArch::Xpu]);
    assert_eq!(GPUArch::format_gpu_arch(&parsed_comma), "cuda;xpu");

    let parsed_native = GPUArch::parse_multivalue("native").unwrap();
    assert_eq!(parsed_native, vec![GPUArch::Native]);

    // Native CANNOT be combined with any other GPU arch
    assert!(GPUArch::parse_multivalue("native;cuda").is_err());
    assert!(GPUArch::parse_multivalue("cuda,native").is_err());
    assert!(GPUArch::parse_multivalue("native+metal").is_err());
}

#[test]
fn test_xpu_arch_multivalue_formatting_and_defaults() {
    let parsed = XpuArch::parse_multivalue("bmg;lnl;ptl").unwrap();
    assert_eq!(parsed, vec![XpuArch::Bmg, XpuArch::Lnl, XpuArch::Ptl]);
    assert_eq!(XpuArch::format_xpu_arch(&parsed), "bmg;lnl;ptl");

    let empty = XpuArch::parse_multivalue("").unwrap();
    assert_eq!(empty, vec![XpuArch::Bmg]);
    assert_eq!(XpuArch::format_xpu_arch(&empty), "bmg");

    // Report with xpu_arch on x86_64
    let report = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Windows),
        Some(Arch::X86_64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        None,
        None,
        None,
        Some("bmg;lnl"),
        None,
        None,
    )
    .unwrap();
    assert_eq!(report.gpu_arch.as_deref(), Some("xpu"));
    assert_eq!(report.xpu_arch.as_deref(), Some("bmg;lnl"));

    // Explicit gpu_arch = "xpu" defaults to desktop gaming "bmg"
    let report_def = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Linux),
        Some(Arch::X86_64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        Some("xpu"),
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(report_def.xpu_arch.as_deref(), Some("bmg"));

    // arm64 / consoles / riscv64 is empty for xpu
    let report_arm = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Windows),
        Some(Arch::Arm64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        None,
        None,
        None,
        Some("bmg"),
        None,
        None,
    )
    .unwrap();
    assert_eq!(report_arm.xpu_arch.as_deref(), Some(""));
}

#[test]
fn test_metal_family_and_version_compatibility() {
    let families = AppleGpuFamily::parse_multivalue("apple7;apple8;apple9;apple10").unwrap();
    assert_eq!(
        families,
        vec![
            AppleGpuFamily::Apple7,
            AppleGpuFamily::Apple8,
            AppleGpuFamily::Apple9,
            AppleGpuFamily::Apple10
        ]
    );
    assert_eq!(
        AppleGpuFamily::format_metal_family(&families),
        "apple7;apple8;apple9;apple10"
    );

    // Apple2..Apple6 only accepts LegacyMetal ("metal")
    let rep_apple6_metal = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Macosx),
        Some(Arch::Arm64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        Some("15.0"),
        None,
        false,
        None,
        None,
        None,
        None,
        Some("apple6"),
        Some("metal"),
    );
    assert!(rep_apple6_metal.is_ok());

    let rep_apple6_metal3 = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Macosx),
        Some(Arch::Arm64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        Some("15.0"),
        None,
        false,
        None,
        None,
        None,
        None,
        Some("apple6"),
        Some("metal3"),
    );
    assert!(rep_apple6_metal3.is_err());

    // Apple7..Apple10 rejects metal (LegacyMetal)
    let rep_apple7_metal = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Macosx),
        Some(Arch::Arm64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        Some("15.0"),
        None,
        false,
        None,
        None,
        None,
        None,
        Some("apple7"),
        Some("metal"),
    );
    assert!(rep_apple7_metal.is_err());

    let rep_apple7_metal3 = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Macosx),
        Some(Arch::Arm64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        Some("15.0"),
        None,
        false,
        None,
        None,
        None,
        None,
        Some("apple7"),
        Some("metal3"),
    );
    assert!(rep_apple7_metal3.is_ok());

    // Apple10 prefers metal4 on macOS >= 26.0
    let rep_apple10 = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Macosx),
        Some(Arch::Arm64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        Some("26.0"),
        None,
        false,
        None,
        None,
        None,
        None,
        Some("apple10"),
        None,
    )
    .unwrap();
    assert_eq!(rep_apple10.metal_version.as_deref(), Some("metal4"));

    // Apple10 falls back to metal3 on macOS < 26.0
    let rep_apple10_old = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Macosx),
        Some(Arch::Arm64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        Some("15.0"),
        None,
        false,
        None,
        None,
        None,
        None,
        Some("apple10"),
        None,
    )
    .unwrap();
    assert_eq!(rep_apple10_old.metal_version.as_deref(), Some("metal3"));

    // Metal4 on macOS 15.0 should fail target_os_level validation
    let rep_metal4_old_os = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Macosx),
        Some(Arch::Arm64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        Some("15.0"),
        None,
        false,
        None,
        None,
        None,
        None,
        Some("apple10"),
        Some("metal4"),
    );
    assert!(rep_metal4_old_os.is_err());
}

#[test]
fn test_apple_platform_defaults() {
    // macOS default target_os_level is 15.0 (< 26.0), so metal_version defaults to metal3
    let report_mac = ArchFeaturesReport::evaluate(Some(Platform::Macosx), Some(Arch::Arm64), Some("generic")).unwrap();
    assert_eq!(report_mac.gpu_arch.as_deref(), Some("metal"));
    assert_eq!(
        report_mac.metal_family.as_deref(),
        Some("apple7;apple8;apple9;apple10")
    );
    assert_eq!(report_mac.metal_version.as_deref(), Some("metal3"));

    // iOS default target_os_level is 26.0 (>= 26.0), so metal_version defaults to metal4
    let report_ios = ArchFeaturesReport::evaluate(Some(Platform::Ios), Some(Arch::Arm64), Some("generic")).unwrap();
    assert_eq!(report_ios.gpu_arch.as_deref(), Some("metal"));
    assert_eq!(
        report_ios.metal_family.as_deref(),
        Some("apple7;apple8;apple9;apple10")
    );
    assert_eq!(report_ios.metal_version.as_deref(), Some("metal4"));

    // macOS with target_os_level 26.0 defaults to metal4
    let report_mac_26 = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Macosx),
        Some(Arch::Arm64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        Some("26.0"),
        None,
        false,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(report_mac_26.metal_version.as_deref(), Some("metal4"));

    // Metal on non-Apple platform is empty string
    let report_win = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Windows),
        Some(Arch::Arm64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        None,
        None,
        None,
        None,
        Some("apple10"),
        Some("metal4"),
    )
    .unwrap();
    assert_eq!(report_win.metal_family.as_deref(), Some(""));
    assert_eq!(report_win.metal_version.as_deref(), Some(""));
}

#[test]
fn test_multi_gpu_arch_selection() {
    // Specifying multiple GPU archs in gpu_arch selects desktop gaming for each
    let report = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Windows),
        Some(Arch::X86_64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        Some("cuda;rocm;xpu"),
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(report.gpu_arch.as_deref(), Some("cuda;rocm;xpu"));
    assert_eq!(report.cuda_arch.as_deref(), Some("75"));
    assert_eq!(report.rocm_arch.as_deref(), Some("1030"));
    assert_eq!(report.xpu_arch.as_deref(), Some("bmg"));

    // Inferring multiple gpu_arch from cuda_arch + rocm_arch
    let report_inferred = ArchFeaturesReport::evaluate_target_triple_with_gpu(
        Some(Platform::Windows),
        Some(Arch::X86_64),
        Some("generic"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        None,
        Some("86"),
        Some("1030"),
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(report_inferred.gpu_arch.as_deref(), Some("cuda;rocm"));
    assert_eq!(report_inferred.cuda_arch.as_deref(), Some("86"));
    assert_eq!(report_inferred.rocm_arch.as_deref(), Some("1030"));
}



