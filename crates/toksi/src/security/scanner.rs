// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/scanner.rs
// created: 2026-11-21

//! Zero-Plaintext Runtime Blind Scanner (`BLAKE3` n-grams + `sparse::DFA::from_bytes`).

#![deny(unsafe_code)]

use std::collections::HashSet;

use regex_automata::dfa::sparse::DFA as SparseDfa;
use regex_automata::dfa::Automaton;
use regex_automata::Input;

use crate::security::blind_compiler::{
    compile_blake3_ngrams, decode_sparse_dfa_bytes, DEFAULT_BLIND_KEY,
};
use crate::security::sarif::UnifiedSecurityFinding;
use crate::security::schema::BlindSignatures;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlindMatch {
    pub kind: String,
    pub digest_or_index: String,
}

/// Executes zero-plaintext matching against keyed BLAKE3 n-grams and Base64 `sparse::DFA`s.
pub fn scan_with_blind_signatures(
    target_text: &str,
    sigs: &BlindSignatures,
    key: &[u8; 32],
) -> Vec<BlindMatch> {
    let mut hits = Vec::new();

    if !sigs.blake3_ngrams.is_empty() {
        let sig_set: HashSet<&str> = sigs.blake3_ngrams.iter().map(String::as_str).collect();
        for digest in compile_blake3_ngrams(target_text, key) {
            if sig_set.contains(digest.as_str()) {
                hits.push(BlindMatch {
                    kind: "blake3_ngram".to_string(),
                    digest_or_index: digest,
                });
            }
        }
    }

    for (idx, b64) in sigs.sparse_dfas_b64.iter().enumerate() {
        if let Ok(bytes) = decode_sparse_dfa_bytes(b64)
            && let Ok((dfa, _)) = SparseDfa::from_bytes(&bytes) {
                let input = Input::new(target_text.as_bytes());
                if dfa.try_search_fwd(&input).ok().flatten().is_some() {
                    hits.push(BlindMatch {
                        kind: "sparse_dfa".to_string(),
                        digest_or_index: format!("dfa[{idx}]"),
                    });
                }
            }
    }

    hits
}

/// Scans a file path + content against blind signatures and emits sanitized SARIF-ready findings.
pub fn scan_file_blind_to_findings(
    file_path: &str,
    content: &str,
    sigs: &BlindSignatures,
) -> Vec<UnifiedSecurityFinding> {
    scan_with_blind_signatures(content, sigs, &DEFAULT_BLIND_KEY)
        .into_iter()
        .map(|m| UnifiedSecurityFinding {
            rule_id: format!("toksi.blind.{}", m.kind),
            severity: "HIGH".to_string(),
            cwe: "CWE-94".to_string(),
            aitech: "AITech-9.1-BlindSignature".to_string(),
            file: file_path.to_string(),
            line: 1,
            message: format!("Matched zero-plaintext blind signature ({})", m.digest_or_index),
            remediation: "Quarantine or inspect matched artifact inside isolated sandbox.".to_string(),
        })
        .collect()
}
