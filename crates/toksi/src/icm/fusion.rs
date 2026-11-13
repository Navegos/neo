// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/icm/fusion.rs
// created: 2026-11-13

#![deny(unsafe_code)]

use std::collections::HashMap;
use crate::icm::schema::FusedHit;

/// Standard RRF damping constant (Cormack et al., 2009).
pub const RRF_K: f32 = 60.0;

/// One ranked list of memory ids, best first.
#[derive(Debug, Clone, Copy)]
pub struct RankedList<'a> {
    pub name: &'static str,
    pub ids: &'a [String],
    pub weight: f32,
}

fn effective_weight(weight: f32) -> f64 {
    if weight.is_finite() && weight > 0.0 {
        f64::from(weight)
    } else {
        0.0
    }
}

/// Fuse ranked lists by Reciprocal Rank Fusion.
///
/// Raw score of an id = sum over lists of `weight / (k + rank)`, rank starting at 1.
/// The returned score is that sum divided by the best achievable one:
/// `sum(weight / (k + 1))` over non-empty lists.
///
/// Output order: score descending, then best rank ascending, then id ascending.
/// Fully deterministic and zero panics.
pub fn rrf_fuse(lists: &[RankedList<'_>], k: f32) -> Vec<FusedHit> {
    let k = if k.is_finite() && k > 0.0 {
        f64::from(k)
    } else {
        f64::from(RRF_K)
    };

    struct Acc<'a> {
        id: &'a str,
        raw: f64,
        ranks: Vec<Option<u32>>,
    }

    let mut index: HashMap<&str, usize> = HashMap::new();
    let mut accs: Vec<Acc<'_>> = Vec::new();
    let mut best_possible = 0.0_f64;

    for (list_idx, list) in lists.iter().enumerate() {
        if list.ids.is_empty() {
            continue;
        }
        let weight = effective_weight(list.weight);
        best_possible += weight / (k + 1.0);

        for (pos, id) in list.ids.iter().enumerate() {
            let rank = u32::try_from(pos + 1).unwrap_or(u32::MAX);
            let slot = *index.entry(id.as_str()).or_insert_with(|| {
                accs.push(Acc {
                    id: id.as_str(),
                    raw: 0.0,
                    ranks: vec![None; lists.len()],
                });
                accs.len() - 1
            });
            let acc = &mut accs[slot];
            if acc.ranks[list_idx].is_some() {
                continue;
            }
            acc.ranks[list_idx] = Some(rank);
            acc.raw += weight / (k + f64::from(rank));
        }
    }

    let mut hits: Vec<FusedHit> = accs
        .into_iter()
        .map(|acc| {
            let score = if best_possible > 0.0 {
                ((acc.raw / best_possible) as f32).min(1.0)
            } else {
                0.0
            };
            FusedHit {
                id: acc.id.to_string(),
                score,
                ranks: acc.ranks,
            }
        })
        .collect();

    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                let best_a = a.ranks.iter().flatten().min().copied().unwrap_or(u32::MAX);
                let best_b = b.ranks.iter().flatten().min().copied().unwrap_or(u32::MAX);
                best_a.cmp(&best_b)
            })
            .then_with(|| a.id.cmp(&b.id))
    });

    hits
}
