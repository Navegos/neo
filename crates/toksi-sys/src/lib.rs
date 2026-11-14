// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi-sys/src/lib.rs
// created: 2026-11-13
// lastModified: 2026-11-13

//! Isolated leaf FFI crate for platform-specific system calls.
//!
//! Every `unsafe` FFI call in this crate must be accompanied by an explicit
//! `// SAFETY:` comment proving pointer validity, null-termination, alignment,
//! and buffer size invariants.

pub mod process;
pub mod windows_env;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegEnvEntry {
    pub value: String,
    pub reg_type: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum SysError {
    #[error("Win32 registry error in {op} (code {code})")]
    Registry { op: &'static str, code: u32 },

    #[error("Win32 registry value '{name}' has unexpected type {reg_type}")]
    UnexpectedValueType { name: String, reg_type: u32 },

    #[error("Win32 registry value '{name}' exceeds u32 byte length ({len} bytes)")]
    ValueTooLarge { name: String, len: usize },

    #[error("Unsupported platform operation: {0}")]
    UnsupportedPlatform(&'static str),
}

impl SysError {
    pub fn is_access_denied(&self) -> bool {
        matches!(self, Self::Registry { code: 5, .. })
    }
}
