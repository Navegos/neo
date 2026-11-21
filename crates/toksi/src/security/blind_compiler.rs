// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/blind_compiler.rs
// created: 2026-11-21

//! Zero-Plaintext Blind Signature Compiler (`toksi compile-blind`).
//!
//! Converts raw commands, AST token n-grams (`3, 4, 5`), and regex patterns into
//! keyed `BLAKE3` digests and little-endian Base64 `regex_automata::dfa::sparse::DFA`
//! automata before any disk, SQLite, or advisory write.

#![deny(unsafe_code)]

use base64::{engine::general_purpose::STANDARD as B64_STANDARD, Engine as _};
use regex_automata::dfa::dense::Builder as DenseDfaBuilder;
use regex_automata::dfa::sparse::DFA as SparseDfa;

use crate::commands::test_runner::blake3_keyed_hex;
use crate::security::polyglot::{extract_sliding_ngrams, normalize_tokens};
use crate::security::schema::BlindSignatures;

/// Domain-separated 32-byte key for Toksi blind n-gram hashing.
pub const DEFAULT_BLIND_KEY: [u8; 32] = *b"TOKSI-ZERO-PLAINTEXT-BLAKE3-KEY!";

/// Compiles raw command/snippet tokens into keyed BLAKE3 n-gram digests (`3, 4, 5`).
pub fn compile_blake3_ngrams(raw_input: &str, key: &[u8; 32]) -> Vec<String> {
    let tokens = normalize_tokens(raw_input);
    let ngrams = extract_sliding_ngrams(&tokens);
    let mut digests = Vec::with_capacity(ngrams.len());
    for ng in ngrams {
        let hex = blake3_keyed_hex(key, ng.as_bytes());
        if !digests.contains(&hex) {
            digests.push(hex);
        }
    }
    digests
}

/// Compiles a regex pattern offline into a little-endian `sparse::DFA` encoded as Base64.
/// Plaintext regex is never retained in the returned string.
pub fn compile_regex_to_sparse_dfa_b64(pattern: &str) -> Result<String, String> {
    let dense = DenseDfaBuilder::new()
        .build(pattern)
        .map_err(|e| format!("Failed to compile dense DFA: {e}"))?;
    let sparse = dense
        .to_sparse()
        .map_err(|e| format!("Failed to convert DFA to sparse: {e}"))?;
    let bytes = sparse.to_bytes_little_endian();
    Ok(B64_STANDARD.encode(&bytes))
}

/// Deserializes a Base64 little-endian `sparse::DFA` for zero-copy runtime matching.
pub fn load_sparse_dfa_from_b64(b64: &str) -> Result<(Vec<u8>, SparseDfa<&'static [u8]>), String> {
    let bytes = B64_STANDARD
        .decode(b64.trim())
        .map_err(|e| format!("Invalid Base64 sparse DFA: {e}"))?;
    // Verify alignment/validity without leaking `'static` lifetime: callers use `SparseDfa::from_bytes(&bytes)` directly.
    let _ = SparseDfa::from_bytes(&bytes)
        .map_err(|e| format!("Invalid sparse::DFA bytes: {e}"))?;
    Err("Use deserialize_sparse_dfa_bytes(&bytes) with owned Vec<u8>".to_string())
}

/// Decodes Base64 sparse DFA into owned byte buffer; pass `&buf` to `SparseDfa::from_bytes`.
pub fn decode_sparse_dfa_bytes(b64: &str) -> Result<Vec<u8>, String> {
    let bytes = B64_STANDARD
        .decode(b64.trim())
        .map_err(|e| format!("Invalid Base64 sparse DFA: {e}"))?;
    let _ = SparseDfa::from_bytes(&bytes)
        .map_err(|e| format!("Invalid sparse::DFA bytes: {e}"))?;
    Ok(bytes)
}

/// Compiles raw command/code samples and optional regex patterns into a `BlindSignatures` bundle.
pub fn compile_blind_bundle(
    samples: &[&str],
    regex_patterns: &[&str],
    taxonomy: &str,
    tlp_level: &str,
    key: &[u8; 32],
) -> Result<BlindSignatures, String> {
    let mut blake3_ngrams = Vec::new();
    for sample in samples {
        for d in compile_blake3_ngrams(sample, key) {
            if !blake3_ngrams.contains(&d) {
                blake3_ngrams.push(d);
            }
        }
    }
    let mut sparse_dfas_b64 = Vec::with_capacity(regex_patterns.len());
    for pat in regex_patterns {
        sparse_dfas_b64.push(compile_regex_to_sparse_dfa_b64(pat)?);
    }
    Ok(BlindSignatures {
        blake3_ngrams,
        sparse_dfas_b64,
        taxonomy: taxonomy.to_string(),
        tlp_level: tlp_level.to_string(),
    })
}
