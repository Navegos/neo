// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/rules/embedded/mod.rs

//! Embedded in-tree rules & skills assets compiled directly into the tokless binary.

pub mod caveman;
pub mod ponytail;
pub mod principles;

pub use caveman as embedded_caveman;
pub use ponytail as embedded_ponytail;
pub use principles as embedded_principles;

