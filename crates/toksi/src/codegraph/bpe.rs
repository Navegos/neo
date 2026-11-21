// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/bpe.rs
// created: 2026-11-21

//! Fast pure-Rust BPE-compatible token estimator for CodeGraph & Repomix packing.

#![deny(unsafe_code)]

use crate::headroom::tokenizer::BpeTokenizer;

/// Counts approximate BPE tokens for source code without network or Python calls.
pub fn estimate_code_tokens(code: &str) -> usize {
    BpeTokenizer::default().count_tokens(code)
}

/// Truncates `code` cleanly to stay within `max_tokens`.
pub fn truncate_to_token_budget(code: &str, max_tokens: usize) -> String {
    BpeTokenizer::default().truncate_to_budget(code, max_tokens)
}
