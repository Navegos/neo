// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/projectmem/error.rs
// created: 2026-11-12

#![deny(unsafe_code)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectMemError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("File lock error: {0}")]
    Lock(String),

    #[error("Project memory directory not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Event reference error: {0}")]
    EventRef(String),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Execution error: {0}")]
    Execution(String),
}

pub type Result<T> = std::result::Result<T, ProjectMemError>;
