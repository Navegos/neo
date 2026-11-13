// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/contextmode/error.rs
// created: 2026-11-13

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextModeError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Security violation: {0}")]
    Security(String),

    #[error("SSRF security guard blocked access to: {0}")]
    Ssrf(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Repository root not found: {0}")]
    RepoNotFound(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
