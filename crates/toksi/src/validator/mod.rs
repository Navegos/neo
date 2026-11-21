// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/validator/mod.rs
// created: 2026-11-21

//! Native CVE 5.1 / OSV 1.7.4 / STIX 2.1 / SARIF 2.1.0 / `Navegos/advisory`
//! validation, false-positive pruning, 3-tree shard builder, and Git/PR driver.

#![deny(unsafe_code)]

pub mod fp_pruner;
pub mod git_pr_driver;
pub mod shard_builder;
pub mod validate;

pub use fp_pruner::*;
pub use git_pr_driver::*;
pub use shard_builder::*;
pub use validate::*;
