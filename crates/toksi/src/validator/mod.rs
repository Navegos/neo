// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/validator/mod.rs
// created: 2026-11-21

//! Native CVE 5.1 / OSV 1.7.4 / `Navegos/advisory` validation & zero-plaintext hygiene.

#![deny(unsafe_code)]

pub mod validate;

pub use validate::*;
