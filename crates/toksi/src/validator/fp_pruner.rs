// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/validator/fp_pruner.rs
// created: 2026-11-21

//! False-positive scrubber (`Safe-Synthesizer` + `corpus/benign/`) that strips
//! colliding `BLAKE3` n-grams and `sparse::DFA` entries and marks empty advisories `withdrawn`.

#![deny(unsafe_code)]

use std::collections::HashSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::security::blind_compiler::{compile_blake3_ngrams, decode_sparse_dfa_bytes, DEFAULT_BLIND_KEY};
use crate::security::schema::BlindSignatures;
use regex_automata::dfa::sparse::DFA as SparseDfa;
use regex_automata::dfa::Automaton;
use regex_automata::Input;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PruneReport {
    pub removed_ngrams: usize,
    pub removed_dfas: usize,
    pub withdrawn: bool,
}

/// Loads all text fixtures from `corpus/benign/` (recursively).
pub fn load_benign_corpus(corpus_dir: &Path) -> Vec<String> {
    let mut samples = Vec::new();
    if !corpus_dir.exists() {
        return samples;
    }
    let walker = ignore::WalkBuilder::new(corpus_dir).build();
    for entry in walker.flatten() {
        let p = entry.path();
        if p.is_file()
            && let Ok(raw) = std::fs::read_to_string(p) {
                samples.push(raw);
            }
    }
    samples
}

/// Scrubs any `BLAKE3` n-gram or `sparse::DFA` that matches benign corpus samples.
/// If no signatures remain, sets `withdrawn = true`.
pub fn prune_false_positives(sigs: &mut BlindSignatures, benign_samples: &[String]) -> PruneReport {
    let mut benign_digests: HashSet<String> = HashSet::new();
    for sample in benign_samples {
        for d in compile_blake3_ngrams(sample, &DEFAULT_BLIND_KEY) {
            benign_digests.insert(d);
        }
    }

    let before_ngrams = sigs.blake3_ngrams.len();
    sigs.blake3_ngrams.retain(|d| !benign_digests.contains(d));
    let removed_ngrams = before_ngrams - sigs.blake3_ngrams.len();

    let before_dfas = sigs.sparse_dfas_b64.len();
    sigs.sparse_dfas_b64.retain(|b64| {
        let Ok(bytes) = decode_sparse_dfa_bytes(b64) else {
            return false;
        };
        let Ok((dfa, _)) = SparseDfa::from_bytes(&bytes) else {
            return false;
        };
        !benign_samples.iter().any(|sample| {
            let input = Input::new(sample.as_bytes());
            dfa.try_search_fwd(&input).ok().flatten().is_some()
        })
    });
    let removed_dfas = before_dfas - sigs.sparse_dfas_b64.len();

    let withdrawn = sigs.blake3_ngrams.is_empty() && sigs.sparse_dfas_b64.is_empty();
    PruneReport {
        removed_ngrams,
        removed_dfas,
        withdrawn,
    }
}
