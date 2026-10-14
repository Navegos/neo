// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/version.rs
// created: 2026-10-05
// lastModified: 2026-10-14

pub fn tokless_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub use crate::util::versions::semver_compare;

pub fn semver_gte(a: &str, b: &str) -> bool {
    semver_compare(a, b) >= 0
}

