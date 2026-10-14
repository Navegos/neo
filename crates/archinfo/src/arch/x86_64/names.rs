// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/archinfo/src/arch/x86_64/names.rs
// created: 2026-10-14
// lastModified: 2026-10-14

use super::isa::X64ISA;
use super::targets::{MinimumCpuArchitectureX64, TargetCpuArchitectureX64};
use crate::vector_length::CpuArchitectureVectorLength;

/// Provides a mapping between the TargetCpuArchitectureX64 enum values and their default enabled X64ISA extensions string representations for Clang.
pub struct ClangTargetCpuArchitectureX64ISANames;

impl ClangTargetCpuArchitectureX64ISANames {
    pub fn name(target: TargetCpuArchitectureX64) -> &'static str {
        match target {
            TargetCpuArchitectureX64::None | TargetCpuArchitectureX64::Generic => {
                "sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf"
            }
            TargetCpuArchitectureX64::Native => "",
            TargetCpuArchitectureX64::X86_64_v2 | TargetCpuArchitectureX64::Corei7 |TargetCpuArchitectureX64::Nehalem | TargetCpuArchitectureX64::Westmere => {
                "sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf"
            }
            TargetCpuArchitectureX64::Slm | TargetCpuArchitectureX64::Silvermont => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd"
            }
            TargetCpuArchitectureX64::Goldmont => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+sha+rdseed+xsave+xsavec+xsaves+xsaveopt+clflushopt"
            }
            TargetCpuArchitectureX64::Goldmont_plus => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+sha+rdseed+xsave+xsavec+xsaves+xsaveopt+clflushopt+ptwrite+rdpid"
            }
            TargetCpuArchitectureX64::Tremont => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+sha+rdseed+xsave+xsavec+xsaves+xsaveopt+clflushopt+ptwrite+rdpid+clwb+gfni+movdiri+movdir64b+waitpkg"
            }
            TargetCpuArchitectureX64::Corei7_avx | TargetCpuArchitectureX64::Sandybridge => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+avx+xsave+vzeroupper"
            }
            TargetCpuArchitectureX64::Core_avx_i | TargetCpuArchitectureX64::Ivybridge => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+f16c+vzeroupper"
            }
            TargetCpuArchitectureX64::Bdver1 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+avx+xsave+lzcnt+sse4a+fma4+xop+lwp"
            }
            TargetCpuArchitectureX64::Bdver2 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+avx+xsave+lzcnt+fma+bmi+f16c+sse4a+fma4+xop+lwp+tbm"
            }
            TargetCpuArchitectureX64::Bdver3 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+avx+xsave+lzcnt+fma+bmi+f16c+sse4a+fma4+xop+lwp+tbm"
            }
            TargetCpuArchitectureX64::Btver2 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+avx+xsave+lzcnt+bmi+f16c+sse4a"
            }
            TargetCpuArchitectureX64::Xboxone => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+avx+xsave+lzcnt+bmi+f16c+sse4a"
            }
            TargetCpuArchitectureX64::Ps4 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+avx+xsave+lzcnt+bmi+f16c+sse4a"
            }
            TargetCpuArchitectureX64::X86_64_v3 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c"
            }
            TargetCpuArchitectureX64::Core_AVX2 | TargetCpuArchitectureX64::Haswell => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c"
            }
            TargetCpuArchitectureX64::Broadwell => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx"
            }
            TargetCpuArchitectureX64::Skylake => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt"
            }
            TargetCpuArchitectureX64::Alderlake | TargetCpuArchitectureX64::Raptorlake | TargetCpuArchitectureX64::Meteorlake | TargetCpuArchitectureX64::Gracemont => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+ptwrite+rdpid+clwb+gfni+movdiri+movdir64b+waitpkg+vaes+pku+vpclmulqdq+serialize+kl+widekl+avxvnni"
            }
            TargetCpuArchitectureX64::Sierraforest | TargetCpuArchitectureX64::Grandridge => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+ptwrite+rdpid+clwb+gfni+movdiri+movdir64b+waitpkg+vaes+pku+vpclmulqdq+serialize+kl+widekl+avxvnni+avxifma+avxvnniint8+avxneconvert+cmpccxadd+enqcmd+uintr"
            }
            TargetCpuArchitectureX64::Arrowlake => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+ptwrite+rdpid+clwb+gfni+movdiri+movdir64b+waitpkg+vaes+pku+vpclmulqdq+serialize+kl+widekl+avxvnni+avxifma+avxvnniint8+avxneconvert+cmpccxadd+uintr"
            }
            TargetCpuArchitectureX64::Arrowlake_s | TargetCpuArchitectureX64::Lunarlake => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+ptwrite+rdpid+clwb+gfni+movdiri+movdir64b+waitpkg+vaes+pku+vpclmulqdq+serialize+kl+widekl+avxvnni+avxifma+avxvnniint8+avxneconvert+cmpccxadd+avxvnniint16+uintr+sha512"
            }
            TargetCpuArchitectureX64::Pantherlake | TargetCpuArchitectureX64::Wildcatlake => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+ptwrite+rdpid+clwb+gfni+movdiri+movdir64b+waitpkg+vaes+pku+vpclmulqdq+serialize+avxvnni+avxifma+avxvnniint8+avxneconvert+cmpccxadd+avxvnniint16+uintr+sha512"
            }
            TargetCpuArchitectureX64::Clearwaterforest => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+ptwrite+rdpid+clwb+gfni+movdiri+movdir64b+waitpkg+vaes+pku+vpclmulqdq+serialize+avxvnni+avxifma+avxvnniint8+avxneconvert+cmpccxadd+avxvnniint16+enqcmd+uintr+sha512+prefetchi"
            }
            TargetCpuArchitectureX64::Bdver4 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c+sse4a+fma4+xop+lwp+tbm"
            }
            TargetCpuArchitectureX64::Znver1 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+sha+clzero+sse4a+mwaitx"
            }
            TargetCpuArchitectureX64::Znver2 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+sha+clzero+rdpid+clwb+sse4a+mwaitx"
            }
            TargetCpuArchitectureX64::Xboxxs => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+sha+clzero+rdpid+clwb+sse4a+mwaitx"
            }
            TargetCpuArchitectureX64::Ps5 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+sha+clzero+rdpid+clwb+sse4a+mwaitx"
            }
            TargetCpuArchitectureX64::Steamdeck => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+sha+clzero+rdpid+clwb+sse4a+mwaitx"
            }
            TargetCpuArchitectureX64::Znver3 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+sha+clzero+rdpid+clwb+vaes+pku+vpclmulqdq+sse4a+mwaitx"
            }
            TargetCpuArchitectureX64::X86_64_v4 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+avx512f+avx512bw+avx512cd+avx512dq+avx512vl"
            }
            TargetCpuArchitectureX64::Skx | TargetCpuArchitectureX64::Skylake_avx512 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+avx512f+avx512bw+avx512cd+avx512dq+avx512vl"
            }
            TargetCpuArchitectureX64::Cannonlake => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+sha+pku+avx512vbmi+avx512ifma"
            }
            TargetCpuArchitectureX64::Icelake_client | TargetCpuArchitectureX64::Icelake_server => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+sha+pku+avx512vbmi+avx512ifma+avx512vnni+gfni+avx512vbmi2+vpclmulqdq+avx512bitalg+avx512vpopcntdq+rdpid"
            }
            TargetCpuArchitectureX64::Cascadelake => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vnni"
            }
            TargetCpuArchitectureX64::Cooperlake => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vnni+avx512bf16"
            }
            TargetCpuArchitectureX64::Rocketlake => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+sha+pku+avx512vbmi+avx512ifma+avx512vnni+gfni+avx512vbmi2+vpclmulqdq+avx512bitalg+avx512vpopcntdq+rdpid"
            }
            TargetCpuArchitectureX64::Tigerlake => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+sha+pku+avx512vbmi+avx512ifma+avx512vnni+gfni+avx512vbmi2+vpclmulqdq+avx512bitalg+avx512vpopcntdq+rdpid+movdiri+movdir64b+avx512vp2intersect+kl+widekl"
            }
            TargetCpuArchitectureX64::Sapphirerapids | TargetCpuArchitectureX64::Emeraldrapids => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+sha+pku+avx512vbmi+avx512ifma+avx512vnni+gfni+avx512vbmi2+vpclmulqdq+avx512bitalg+avx512vpopcntdq+rdpid+movdiri+movdir64b+ptwrite+enqcmd+kl+widekl+waitpkg+serialize+uintr+amx_bf16+amx_tile+amx_int8+avxvnni+avx512fp16+avx512bf16"
            }
            TargetCpuArchitectureX64::Graniterapids => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+sha+pku+avx512vbmi+avx512ifma+avx512vnni+gfni+avx512vbmi2+vpclmulqdq+avx512bitalg+avx512vpopcntdq+rdpid+movdiri+movdir64b+ptwrite+enqcmd+kl+widekl+waitpkg+serialize+uintr+amx_bf16+amx_tile+amx_int8+avxvnni+avx512fp16+avx512bf16+amx_fp16+prefetchi"
            }
            TargetCpuArchitectureX64::Graniterapids_d => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+sha+pku+avx512vbmi+avx512ifma+avx512vnni+gfni+avx512vbmi2+vpclmulqdq+avx512bitalg+avx512vpopcntdq+rdpid+movdiri+movdir64b+ptwrite+enqcmd+kl+widekl+waitpkg+serialize+uintr+amx_bf16+amx_tile+amx_int8+avxvnni+avx512fp16+avx512bf16+amx_fp16+prefetchi+amx_complex"
            }
            TargetCpuArchitectureX64::Znver4 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+sha+clzero+rdpid+clwb+vaes+pku+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+gfni+avx512vbmi2+vpclmulqdq+avx512bitalg+avx512vpopcntdq+sse4a+mwaitx"
            }
            TargetCpuArchitectureX64::Steammachine => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+sha+clzero+rdpid+clwb+vaes+pku+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+gfni+avx512vbmi2+vpclmulqdq+avx512bitalg+avx512vpopcntdq+sse4a+mwaitx"
            }
            TargetCpuArchitectureX64::Znver5 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+sha+clzero+rdpid+clwb+vaes+pku+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+gfni+avx512vbmi2+vpclmulqdq+avx512bitalg+avx512vpopcntdq+avxvnni+movdiri+movdir64b+avx512vp2intersect+prefetchi+sse4a+mwaitx"
            }
            TargetCpuArchitectureX64::Znver6 => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+sha+clzero+rdpid+clwb+vaes+pku+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+gfni+avx512vbmi2+vpclmulqdq+avx512bitalg+avx512vpopcntdq+avxvnni+movdiri+movdir64b+avx512vp2intersect+prefetchi+avxvnniint8+avxifma+avx512fp16+avxneconvert+avx512bmm+sse4a+mwaitx"
            }
            TargetCpuArchitectureX64::Diamondrapids => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+pku+gfni+movdiri+movdir64b+sha+vpclmulqdq+avxifma+avxneconvert+avxvnni+avxvnniint8+prefetchi+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+rdpid+ptwrite+enqcmd+kl+widekl+waitpkg+serialize+uintr+amx_bf16+amx_tile+amx_int8+avx512fp16+avx512bf16+amx_fp16+amx_complex+avxvnniint16+cmpccxadd+sha512+apxf+amx_fp8+amx_tf32+movrs+amx_movrs+amx_avx512"
            }
            TargetCpuArchitectureX64::Novalake => {
				"sse+sse2+sse3+ssse3+sse4_1+sse4_2+popcnt+cx16+prfchw+pclmul+aes+crc32+fxsr+sahf+movbe+rdrnd+avx+xsave+vzeroupper+avx2+lzcnt+fma+bmi+bmi2+f16c+rdseed+adx+xsavec+xsaves+xsaveopt+clflushopt+clwb+vaes+pku+gfni+movdiri+movdir64b+sha+vpclmulqdq+avxifma+avxneconvert+avxvnni+avxvnniint8+prefetchi+avx10_1+avx10_2+rdpid+ptwrite+waitpkg+serialize+cmpccxadd+avxvnniint16+uintr+sha512+apxf+movrs"
            }
        }
    }
}

/// Provides a mapping between the TargetCpuArchitectureX64 enum values and their default disabled X64ISA extensions string representations for Clang.
pub struct ClangTargetCpuArchitectureX64NOISANames;

impl ClangTargetCpuArchitectureX64NOISANames {
    pub fn name(target: TargetCpuArchitectureX64) -> &'static str {
        match target {
            TargetCpuArchitectureX64::None | TargetCpuArchitectureX64::Generic => {
                ""
            }
            TargetCpuArchitectureX64::Native => "fsgsbase+rdpru",
            TargetCpuArchitectureX64::X86_64_v2 => {
				"avx512bmm+fsgsbase+rdpru+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+avx2+avx+tsxldtrk+sm4"
            }
            TargetCpuArchitectureX64::Corei7 | TargetCpuArchitectureX64::Nehalem | TargetCpuArchitectureX64::Westmere => {
				"avx512bmm+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+vaes+clwb+clflushopt+xsaveopt+xsaves+xsavec+adx+rdseed+f16c+bmi2+bmi+fma+lzcnt+avx2+xsave+avx+rdrnd+fsgsbase+movbe+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Slm | TargetCpuArchitectureX64::Silvermont => {
				"avx512bmm+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+vaes+clwb+clflushopt+xsaveopt+xsaves+xsavec+adx+rdseed+f16c+bmi2+bmi+fma+lzcnt+avx2+xsave+avx+fsgsbase+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Goldmont => {
				"avx512bmm+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+vaes+pku+clwb+adx+f16c+bmi2+bmi+fma+lzcnt+avx2+avx+fsgsbase+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Goldmont_plus => {
				"avx512bmm+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+movdir64b+movdiri+vpclmulqdq+gfni+vaes+pku+clwb+adx+f16c+bmi2+bmi+fma+lzcnt+avx2+avx+fsgsbase+tsxldtrk+sm4+rtm+hle+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Tremont => {
				"avx512bmm+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+vpclmulqdq+vaes+pku+adx+f16c+bmi2+bmi+fma+lzcnt+avx2+avx+fsgsbase+tsxldtrk+sm4+rtm+hle+raoint"
            }
            TargetCpuArchitectureX64::Corei7_avx | TargetCpuArchitectureX64::Sandybridge => {
				"avx512bmm+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+vaes+clwb+clflushopt+xsaveopt+xsaves+xsavec+adx+rdseed+f16c+bmi2+bmi+fma+lzcnt+avx2+rdrnd+fsgsbase+movbe+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Core_avx_i | TargetCpuArchitectureX64::Ivybridge => {
				"avx512bmm+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+vaes+clwb+clflushopt+xsaveopt+xsaves+xsavec+adx+rdseed+bmi2+bmi+fma+lzcnt+avx2+fsgsbase+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Bdver1 => {
				"avx512bmm+mwaitx+rdpru+tbm+clzero+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+vaes+clwb+clflushopt+xsaveopt+xsaves+xsavec+adx+rdseed+bmi2+bmi+fma+avx2+rdrnd+fsgsbase+movbe+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::Bdver2 => {
				"avx512bmm+mwaitx+rdpru+clzero+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+vaes+clwb+clflushopt+xsaveopt+xsaves+xsavec+adx+rdseed+bmi2+avx2+rdrnd+fsgsbase+movbe+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::Bdver3 => {
				"avx512bmm+mwaitx+rdpru+clzero+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+vaes+clwb+clflushopt+xsaveopt+xsaves+xsavec+adx+rdseed+bmi2+avx2+rdrnd+fsgsbase+movbe+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::Btver2 => {
				"avx512bmm+fma+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+vaes+clwb+clflushopt+xsaveopt+xsaves+xsavec+adx+rdseed+bmi2+avx2+rdrnd+fsgsbase+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::Xboxone => {
				"avx512bmm+fma+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+vaes+clwb+clflushopt+xsaveopt+xsaves+xsavec+adx+rdseed+bmi2+avx2+rdrnd+fsgsbase+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::Ps4 => {
				"avx512bmm+fma+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+vaes+clwb+clflushopt+xsaveopt+xsaves+xsavec+adx+rdseed+bmi2+avx2+rdrnd+fsgsbase+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::X86_64_v3 => {
				"avx512bmm+fsgsbase+rdpru+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+tsxldtrk"
            }
            TargetCpuArchitectureX64::Core_AVX2 | TargetCpuArchitectureX64::Haswell => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+vaes+clwb+clflushopt+xsaveopt+xsaves+xsavec+adx+rdseed+tsxldtrk+sm4+sgx+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Broadwell => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+vaes+clwb+clflushopt+xsaveopt+xsaves+xsavec+tsxldtrk+sm4+sgx+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Skylake => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+vaes+clwb+tsxldtrk+sm4+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Alderlake | TargetCpuArchitectureX64::Raptorlake | TargetCpuArchitectureX64::Meteorlake => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+prefetchi+uintr+enqcmd+sha+tsxldtrk+sm4+rtm+hle+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Gracemont => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+prefetchi+uintr+enqcmd+sha+tsxldtrk+sm4+rtm+hle+raoint"
            }
            TargetCpuArchitectureX64::Sierraforest => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+avxvnniint16+prefetchi+sha+tsxldtrk+sm4+rtm+hle+raoint"
            }
            TargetCpuArchitectureX64::Grandridge => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+avxvnniint16+prefetchi+sha+tsxldtrk+sm4+rtm+hle"
            }
            TargetCpuArchitectureX64::Arrowlake => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+avxvnniint16+prefetchi+enqcmd+sha+tsxldtrk+sm4+rtm+hle+cldemote"
            }
            TargetCpuArchitectureX64::Arrowlake_s | TargetCpuArchitectureX64::Lunarlake => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+prefetchi+enqcmd+sha+tsxldtrk+rtm+hle+cldemote"
            }
            TargetCpuArchitectureX64::Pantherlake | TargetCpuArchitectureX64::Wildcatlake => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+prefetchi+widekl+kl+enqcmd+sha+tsxldtrk+rtm+hle+cldemote"
            }
            TargetCpuArchitectureX64::Clearwaterforest => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+widekl+kl+sha+tsxldtrk+rtm+hle"
            }
            TargetCpuArchitectureX64::Bdver4 => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+clzero+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+vaes+clwb+clflushopt+xsaveopt+xsaves+xsavec+adx+rdseed+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::Znver1 => {
				"avx512bmm+fsgsbase+rdpru+fma4+xop+lwp+tbm+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+vaes+pku+clwb+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::Znver2 => {
				"avx512bmm+fsgsbase+rdpru+fma4+xop+lwp+tbm+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+movdir64b+movdiri+vpclmulqdq+gfni+vaes+pku+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::Xboxxs => {
				"avx512bmm+fsgsbase+rdpru+fma4+xop+lwp+tbm+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+movdir64b+movdiri+vpclmulqdq+gfni+vaes+pku+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::Ps5 => {
				"avx512bmm+fsgsbase+rdpru+fma4+xop+lwp+tbm+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+movdir64b+movdiri+vpclmulqdq+gfni+vaes+pku+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::Steamdeck => {
				"avx512bmm+fsgsbase+rdpru+fma4+xop+lwp+tbm+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+movdir64b+movdiri+vpclmulqdq+gfni+vaes+pku+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::Znver3 => {
				"avx512bmm+fsgsbase+rdpru+fma4+xop+lwp+tbm+avx10_1+avx10_2+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+movdir64b+movdiri+gfni+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::X86_64_v4 => {
				"fsgsbase+rdpru+fma4+xop+lwp+tbm"
            }
            TargetCpuArchitectureX64::Skx | TargetCpuArchitectureX64::Skylake_avx512 => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+tsxldtrk+sm4+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Cannonlake => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+tsxldtrk+sm4+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Icelake_client | TargetCpuArchitectureX64::Icelake_server => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+movdir64b+movdiri+tsxldtrk+sm4+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Cascadelake => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512vbmi+avx512ifma+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+tsxldtrk+sm4+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Cooperlake => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512vbmi+avx512ifma+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+vpclmulqdq+gfni+pku+sha+tsxldtrk+sm4+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Rocketlake => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+widekl+kl+enqcmd+ptwrite+movdir64b+movdiri+tsxldtrk+sm4+sgx+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Tigerlake => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+prefetchi+uintr+serialize+enqcmd+ptwrite+tsxldtrk+sm4+raoint+cldemote"
            }
            TargetCpuArchitectureX64::Sapphirerapids | TargetCpuArchitectureX64::Emeraldrapids => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512vp2intersect+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+prefetchi+sm4+raoint"
            }
            TargetCpuArchitectureX64::Graniterapids => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512vp2intersect+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+sm4"
            }
            TargetCpuArchitectureX64::Graniterapids_d => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx10_1+avx10_2+avx512vp2intersect+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+prefetchi+sm4"
            }
            TargetCpuArchitectureX64::Znver4 => {
				"avx512bmm+fsgsbase+rdpru+fma4+xop+lwp+tbm+avx10_1+avx10_2+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::Steammachine => {
				"avx512bmm+fsgsbase+rdpru+fma4+xop+lwp+tbm+avx10_1+avx10_2+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+avxvnni+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+movdir64b+movdiri+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::Znver5 => {
				"avx512bmm+fsgsbase+rdpru+fma4+xop+lwp+tbm+avx10_1+avx10_2+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint8+avxvnniint16+avxneconvert+avxifma+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::Znver6 => {
				"fsgsbase+rdpru+fma4+xop+lwp+tbm+avx10_1+avx10_2+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+apxf+movrs+sha512+waitpkg+cmpccxadd+avxvnniint16+uintr+serialize+widekl+kl+enqcmd+ptwrite+rdpid+usermsr+tsxldtrk+sm4+sgx+rtm+hle+raoint+cldemote+vzeroupper"
            }
            TargetCpuArchitectureX64::Diamondrapids => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx512vp2intersect"
            }
            TargetCpuArchitectureX64::Novalake => {
				"avx512bmm+fsgsbase+mwaitx+rdpru+fma4+xop+lwp+tbm+clzero+sse4a+avx512f+avx512bw+avx512cd+avx512dq+avx512vl+avx512vbmi+avx512ifma+avx512vnni+avx512vbmi2+avx512bitalg+avx512vpopcntdq+avx512vp2intersect+avx512fp16+avx512bf16+amx_bf16+amx_tile+amx_int8+amx_fp16+amx_complex+amx_tf32+amx_movrs+amx_fp8+amx_avx512+widekl+kl+enqcmd+tsxldtrk+rtm+hle+cldemote"
            }
        }
    }
}

/// Provides names for X64 ISA (Instruction Set Architecture) for Clang -m{extension} flags.
pub struct ClangX64ISANames;

impl ClangX64ISANames {
    pub fn name(isa: X64ISA) -> &'static str {
        match isa {
            X64ISA::None => "",
            X64ISA::Adx => "adx",
            X64ISA::Aes => "aes",
            X64ISA::AmxAvx512 => "amx-avx512",
            X64ISA::AmxBf16 => "amx-bf16",
            X64ISA::AmxComplex => "amx-complex",
            X64ISA::AmxFp16 => "amx-fp16",
            X64ISA::AmxFp8 => "amx-fp8",
            X64ISA::AmxInt8 => "amx-int8",
            X64ISA::AmxMovrs => "amx-movrs",
            X64ISA::AmxTf32 => "amx-tf32",
            X64ISA::AmxTile => "amx-tile",
            X64ISA::Apxf => "apxf",
            X64ISA::Avx => "avx",
            X64ISA::Avx10_1 => "avx10.1",
            X64ISA::Avx10_2 => "avx10.2",
            X64ISA::Avx2 => "avx2",
            X64ISA::Avx512bf16 => "avx512bf16",
            X64ISA::Avx512bitalg => "avx512bitalg",
            X64ISA::Avx512bw => "avx512bw",
            X64ISA::Avx512bmm => "avx512bmm",
            X64ISA::Avx512cd => "avx512cd",
            X64ISA::Avx512dq => "avx512dq",
            X64ISA::Avx512f => "avx512f",
            X64ISA::Avx512fp16 => "avx512fp16",
            X64ISA::Avx512ifma => "avx512ifma",
            X64ISA::Avx512vbmi => "avx512vbmi",
            X64ISA::Avx512vbmi2 => "avx512vbmi2",
            X64ISA::Avx512vl => "avx512vl",
            X64ISA::Avx512vnni => "avx512vnni",
            X64ISA::Avx512vp2intersect => "avx512vp2intersect",
            X64ISA::Avx512vpopcntdq => "avx512vpopcntdq",
            X64ISA::Avxifma => "avxifma",
            X64ISA::Avxneconvert => "avxneconvert",
            X64ISA::Avxvnni => "avxvnni",
            X64ISA::Avxvnniint16 => "avxvnniint16",
            X64ISA::Avxvnniint8 => "avxvnniint8",
            X64ISA::Bmi => "bmi",
            X64ISA::Bmi2 => "bmi2",
            X64ISA::Cldemote => "cldemote",
            X64ISA::Clflushopt => "clflushopt",
            X64ISA::Clwb => "clwb",
            X64ISA::Clzero => "clzero",
            X64ISA::Cmpccxadd => "cmpccxadd",
            X64ISA::Crc32 => "crc32",
            X64ISA::Cx16 => "cx16",
            X64ISA::Enqcmd => "enqcmd",
            X64ISA::F16c => "f16c",
            X64ISA::Fma => "fma",
            X64ISA::Fma4 => "fma4",
            X64ISA::Fsgsbase => "fsgsbase",
            X64ISA::Fxsr => "fxsr",
            X64ISA::Gfni => "gfni",
            X64ISA::Kl => "kl",
            X64ISA::Lwp => "lwp",
            X64ISA::Lzcnt => "lzcnt",
            X64ISA::Movbe => "movbe",
            X64ISA::Movdir64b => "movdir64b",
            X64ISA::Movdiri => "movdiri",
            X64ISA::Movrs => "movrs",
            X64ISA::Mwaitx => "mwaitx",
            X64ISA::Pclmul => "pclmul",
            X64ISA::Pku => "pku",
            X64ISA::Popcnt => "popcnt",
            X64ISA::Prefetchi => "prefetchi",
            X64ISA::Prfchw => "prfchw",
            X64ISA::Ptwrite => "ptwrite",
            X64ISA::Raoint => "raoint",
            X64ISA::Rdpid => "rdpid",
            X64ISA::Rdpru => "rdpru",
            X64ISA::Rdrnd => "rdrnd",
            X64ISA::Rdseed => "rdseed",
            X64ISA::Rtm => "rtm",
            X64ISA::Sahf => "sahf",
            X64ISA::Serialize => "serialize",
            X64ISA::Sgx => "sgx",
            X64ISA::Sha => "sha",
            X64ISA::Sha512 => "sha512",
            X64ISA::Sm4 => "sm4",
            X64ISA::Sse => "sse",
            X64ISA::Sse2 => "sse2",
            X64ISA::Sse3 => "sse3",
            X64ISA::Sse4_1 => "sse4.1",
            X64ISA::Sse4_2 => "sse4.2",
            X64ISA::Sse4a => "sse4a",
            X64ISA::Ssse3 => "ssse3",
            X64ISA::Tbm => "tbm",
            X64ISA::Tsxldtrk => "tsxldtrk",
            X64ISA::Uintr => "uintr",
            X64ISA::Usermsr => "usermsr",
            X64ISA::Vaes => "vaes",
            X64ISA::Vpclmulqdq => "vpclmulqdq",
            X64ISA::Vzeroupper => "vzeroupper",
            X64ISA::Waitpkg => "waitpkg",
            X64ISA::Aeskl => "",
            X64ISA::Widekl => "widekl",
            X64ISA::Xop => "xop",
            X64ISA::Xsave => "xsave",
            X64ISA::Xsavec => "xsavec",
            X64ISA::Xsaveopt => "xsaveopt",
            X64ISA::Xsaves => "xsaves",
        }
    }
}

/// Provides disabled names for X64 ISA (Instruction Set Architecture) for Clang -mno-{extension} flags.
pub struct ClangX64NOISANames;

impl ClangX64NOISANames {
    pub fn name(isa: X64ISA) -> &'static str {
        match isa {
            X64ISA::None => "",
            X64ISA::Adx => "no-adx",
            X64ISA::Aes => "no-aes",
            X64ISA::AmxAvx512 => "no-amx-avx512",
            X64ISA::AmxBf16 => "no-amx-bf16",
            X64ISA::AmxComplex => "no-amx-complex",
            X64ISA::AmxFp16 => "no-amx-fp16",
            X64ISA::AmxFp8 => "no-amx-fp8",
            X64ISA::AmxInt8 => "no-amx-int8",
            X64ISA::AmxMovrs => "no-amx-movrs",
            X64ISA::AmxTf32 => "no-amx-tf32",
            X64ISA::AmxTile => "no-amx-tile",
            X64ISA::Apxf => "no-apxf",
            X64ISA::Avx => "no-avx",
            X64ISA::Avx10_1 => "no-avx10.1",
            X64ISA::Avx10_2 => "no-avx10.2",
            X64ISA::Avx2 => "no-avx2",
            X64ISA::Avx512bf16 => "no-avx512bf16",
            X64ISA::Avx512bitalg => "no-avx512bitalg",
            X64ISA::Avx512bw => "no-avx512bw",
            X64ISA::Avx512bmm => "no-avx512bmm",
            X64ISA::Avx512cd => "no-avx512cd",
            X64ISA::Avx512dq => "no-avx512dq",
            X64ISA::Avx512f => "no-avx512f",
            X64ISA::Avx512fp16 => "no-avx512fp16",
            X64ISA::Avx512ifma => "no-avx512ifma",
            X64ISA::Avx512vbmi => "no-avx512vbmi",
            X64ISA::Avx512vbmi2 => "no-avx512vbmi2",
            X64ISA::Avx512vl => "no-avx512vl",
            X64ISA::Avx512vnni => "no-avx512vnni",
            X64ISA::Avx512vp2intersect => "no-avx512vp2intersect",
            X64ISA::Avx512vpopcntdq => "no-avx512vpopcntdq",
            X64ISA::Avxifma => "no-avxifma",
            X64ISA::Avxneconvert => "no-avxneconvert",
            X64ISA::Avxvnni => "no-avxvnni",
            X64ISA::Avxvnniint16 => "no-avxvnniint16",
            X64ISA::Avxvnniint8 => "no-avxvnniint8",
            X64ISA::Bmi => "no-bmi",
            X64ISA::Bmi2 => "no-bmi2",
            X64ISA::Cldemote => "no-cldemote",
            X64ISA::Clflushopt => "no-clflushopt",
            X64ISA::Clwb => "no-clwb",
            X64ISA::Clzero => "no-clzero",
            X64ISA::Cmpccxadd => "no-cmpccxadd",
            X64ISA::Crc32 => "no-crc32",
            X64ISA::Cx16 => "no-cx16",
            X64ISA::Enqcmd => "no-enqcmd",
            X64ISA::F16c => "no-f16c",
            X64ISA::Fma => "no-fma",
            X64ISA::Fma4 => "no-fma4",
            X64ISA::Fsgsbase => "no-fsgsbase",
            X64ISA::Fxsr => "no-fxsr",
            X64ISA::Gfni => "no-gfni",
            X64ISA::Kl => "no-kl",
            X64ISA::Lwp => "no-lwp",
            X64ISA::Lzcnt => "no-lzcnt",
            X64ISA::Movbe => "no-movbe",
            X64ISA::Movdir64b => "no-movdir64b",
            X64ISA::Movdiri => "no-movdiri",
            X64ISA::Movrs => "no-movrs",
            X64ISA::Mwaitx => "no-mwaitx",
            X64ISA::Pclmul => "no-pclmul",
            X64ISA::Pku => "no-pku",
            X64ISA::Popcnt => "no-popcnt",
            X64ISA::Prefetchi => "no-prefetchi",
            X64ISA::Prfchw => "no-prfchw",
            X64ISA::Ptwrite => "no-ptwrite",
            X64ISA::Raoint => "no-raoint",
            X64ISA::Rdpid => "no-rdpid",
            X64ISA::Rdpru => "no-rdpru",
            X64ISA::Rdrnd => "no-rdrnd",
            X64ISA::Rdseed => "no-rdseed",
            X64ISA::Rtm => "no-rtm",
            X64ISA::Sahf => "no-sahf",
            X64ISA::Serialize => "no-serialize",
            X64ISA::Sgx => "no-sgx",
            X64ISA::Sha => "no-sha",
            X64ISA::Sha512 => "no-sha512",
            X64ISA::Sm4 => "no-sm4",
            X64ISA::Sse => "no-sse",
            X64ISA::Sse2 => "no-sse2",
            X64ISA::Sse3 => "no-sse3",
            X64ISA::Sse4_1 => "no-sse4.1",
            X64ISA::Sse4_2 => "no-sse4.2",
            X64ISA::Sse4a => "no-sse4a",
            X64ISA::Ssse3 => "no-ssse3",
            X64ISA::Tbm => "no-tbm",
            X64ISA::Tsxldtrk => "no-tsxldtrk",
            X64ISA::Uintr => "no-uintr",
            X64ISA::Usermsr => "no-usermsr",
            X64ISA::Vaes => "no-vaes",
            X64ISA::Vpclmulqdq => "no-vpclmulqdq",
            X64ISA::Vzeroupper => "no-vzeroupper",
            X64ISA::Waitpkg => "no-waitpkg",
            X64ISA::Aeskl => "",
            X64ISA::Widekl => "no-widekl",
            X64ISA::Xop => "no-xop",
            X64ISA::Xsave => "no-xsave",
            X64ISA::Xsavec => "no-xsavec",
            X64ISA::Xsaveopt => "no-xsaveopt",
            X64ISA::Xsaves => "no-xsaves",
        }
    }
}

/// Provides names for X64 ISA for MSVC.
pub struct MSVCX64ISANames;

impl MSVCX64ISANames {
    pub fn name(min_arch: MinimumCpuArchitectureX64) -> &'static str {
        match min_arch {
            MinimumCpuArchitectureX64::None => "SSE4.2",
            MinimumCpuArchitectureX64::AVX => "AVX",
            MinimumCpuArchitectureX64::AVX2 => "AVX2",
            MinimumCpuArchitectureX64::AVX512 => "AVX512",
            MinimumCpuArchitectureX64::AVX10_1 => "AVX10.1",
            MinimumCpuArchitectureX64::AVX10_2 => "AVX10.2",
        }
    }
}

/// Provides MSVC target architecture compiler argument (/arch:...) for MinimumCpuArchitectureX64.
pub struct MSVCX64ArchTarget;

impl MSVCX64ArchTarget {
    pub fn name(min_arch: MinimumCpuArchitectureX64) -> &'static str {
        match min_arch {
            MinimumCpuArchitectureX64::None => "/arch:SSE4.2",
            MinimumCpuArchitectureX64::AVX => "/arch:AVX",
            MinimumCpuArchitectureX64::AVX2 => "/arch:AVX2",
            MinimumCpuArchitectureX64::AVX512 => "/arch:AVX512",
            MinimumCpuArchitectureX64::AVX10_1 => "/arch:AVX10.1",
            MinimumCpuArchitectureX64::AVX10_2 => "/arch:AVX10.2",
        }
    }
}

/// Provides MSVC vector length override compiler argument (/vlen=...) for MinimumCpuArchitectureX64 and vector length.
pub struct MSVCX64VLen;

impl MSVCX64VLen {
    pub fn name(min_arch: MinimumCpuArchitectureX64, vl: CpuArchitectureVectorLength) -> &'static str {
        match min_arch {
            MinimumCpuArchitectureX64::AVX512 => {
                if vl <= CpuArchitectureVectorLength::VL256 {
                    "/vlen=256"
                } else {
                    ""
                }
            }
            MinimumCpuArchitectureX64::AVX10_1 | MinimumCpuArchitectureX64::AVX10_2 => {
                if vl == CpuArchitectureVectorLength::VL512 {
                    "/vlen=512"
                } else {
                    ""
                }
            }
            _ => "",
        }
    }
}

/// Provides Clang vector length override compiler argument (-m'prefer-vector-width=...') for MinimumCpuArchitectureX64 and vector length.
pub struct ClangX64VLen;

impl ClangX64VLen {
    pub fn name(min_arch: MinimumCpuArchitectureX64, vl: CpuArchitectureVectorLength) -> &'static str {
        match min_arch {
            MinimumCpuArchitectureX64::AVX512 => {
                if vl == CpuArchitectureVectorLength::VL128 {
                  "-m'prefer-vector-width=128'"
                } else if vl == CpuArchitectureVectorLength::VL256 {
                  "-m'prefer-vector-width=256'"
              } else {
                  "-m'prefer-vector-width=512'"
                }
            }
            MinimumCpuArchitectureX64::AVX10_1 | MinimumCpuArchitectureX64::AVX10_2 => {
                if vl == CpuArchitectureVectorLength::VL512 {
                  "-m'prefer-vector-width=512'"
                } else if vl == CpuArchitectureVectorLength::VL128 {
                  "-m'prefer-vector-width=128'"
              } else {
                  "-m'prefer-vector-width=256'"
                }
            }
            MinimumCpuArchitectureX64::AVX2 => {
                if vl == CpuArchitectureVectorLength::VL128 {
                  "-m'prefer-vector-width=128'"
                } else {
                  "-m'prefer-vector-width=256'"
                }
            }
            _ => "-m'prefer-vector-width=128'",
        }
    }
}
