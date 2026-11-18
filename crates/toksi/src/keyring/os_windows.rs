// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/keyring/os_windows.rs
// created: 2026-11-04
// lastModified: 2026-11-18

#![deny(unsafe_code)]

pub fn read_credential(target_name: &str) -> Option<String> {
    toksi_sys::windows_cred::read_credential(target_name)
}

pub fn write_credential(target_name: &str, user_name: Option<&str>, secret: &str) -> bool {
    toksi_sys::windows_cred::write_credential(target_name, user_name, secret)
}

pub fn delete_credential(target_name: &str) -> bool {
    toksi_sys::windows_cred::delete_credential(target_name)
}
