// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/error.rs
// created: 2026-11-13
// lastModified: 2026-11-13

use thiserror::Error;

/// Domain errors for pure-Rust Headroom compression, CCR storage, and proxy routing.
#[derive(Debug, Error)]
pub enum HeadroomError {
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("SQLite CCR storage error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Live-zone body splice failed: {0}")]
    LiveZoneSplice(String),

    #[error("HTTP proxy error: {0}")]
    Proxy(String),
}

pub type HeadroomResult<T> = Result<T, HeadroomError>;
