// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/rules/embedded/mod.rs
// created: 2026-10-29
// lastModified: 2026-10-31

//! Embedded in-tree rules & skills assets compiled directly into the toksi binary.

pub mod caveman;
pub mod ponytail;
pub mod principles;

pub use caveman as embedded_caveman;
pub use ponytail as embedded_ponytail;
pub use principles as embedded_principles;

