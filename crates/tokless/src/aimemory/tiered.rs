// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/aimemory/tiered.rs

use serde::{Deserialize, Serialize};

pub const BRIEF_L0_MAX_BYTES: usize = 256;
pub const BRIEF_L1_MAX_BYTES: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisclosureTier {
    /// L0: TOC, titles, high-level pointers (~100 tokens, always fits)
    L0,
    /// L1: Standing rules, active handoffs, core summaries (~400 tokens)
    L1,
    /// L2: Full documentation bodies, specs, raw histories (on-demand)
    L2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TieredEntry {
    pub key: String,
    pub title: String,
    pub abstract_text: Option<String>,
    pub summary_text: Option<String>,
    pub full_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackedBrief {
    pub tier: DisclosureTier,
    pub content: String,
    pub estimated_tokens: usize,
    pub entries_included: usize,
    pub entries_omitted: usize,
}

pub struct TieredPacker;

impl TieredPacker {
    /// Progressively pack entries based on max token budget
    pub fn pack(entries: &[TieredEntry], token_budget: usize) -> PackedBrief {
        // Try L1 first if budget allows (> 300 tokens)
        if token_budget >= 300 {
            let l1_brief = Self::pack_tier(entries, DisclosureTier::L1, token_budget);
            if l1_brief.estimated_tokens <= token_budget && l1_brief.entries_included > 0 {
                return l1_brief;
            }
        }

        // Fall back to L0
        Self::pack_tier(entries, DisclosureTier::L0, token_budget)
    }

    pub fn pack_tier(entries: &[TieredEntry], tier: DisclosureTier, token_budget: usize) -> PackedBrief {
        let mut out = String::new();
        let mut included = 0;
        let mut omitted = 0;

        match tier {
            DisclosureTier::L0 => {
                out.push_str("### Context Index (L0)\n");
                for entry in entries {
                    let desc = entry.abstract_text.as_deref()
                        .or(entry.summary_text.as_deref())
                        .unwrap_or("");
                    let short_desc = truncate_utf8_bytes(desc, BRIEF_L0_MAX_BYTES);
                    let line = if short_desc.is_empty() {
                        format!("- **{}**\n", entry.title)
                    } else {
                        format!("- **{}**: {}\n", entry.title, short_desc)
                    };

                    let next_tokens = estimate_tokens(&format!("{}{}", out, line));
                    if next_tokens > token_budget && included > 0 {
                        omitted += 1;
                        continue;
                    }
                    out.push_str(&line);
                    included += 1;
                }
            }
            DisclosureTier::L1 => {
                out.push_str("### Standing Knowledge & Rules (L1)\n");
                for entry in entries {
                    let summary = entry.summary_text.as_deref()
                        .or(entry.abstract_text.as_deref())
                        .unwrap_or(&entry.full_content);
                    let text = truncate_utf8_bytes(summary, BRIEF_L1_MAX_BYTES);
                    let block = format!("#### {}\n{}\n\n", entry.title, text);

                    let next_tokens = estimate_tokens(&format!("{}{}", out, block));
                    if next_tokens > token_budget && included > 0 {
                        omitted += 1;
                        continue;
                    }
                    out.push_str(&block);
                    included += 1;
                }
            }
            DisclosureTier::L2 => {
                out.push_str("### Detailed Knowledge (L2)\n");
                for entry in entries {
                    let block = format!("#### {}\n{}\n\n", entry.title, entry.full_content);
                    let next_tokens = estimate_tokens(&format!("{}{}", out, block));
                    if next_tokens > token_budget && included > 0 {
                        omitted += 1;
                        continue;
                    }
                    out.push_str(&block);
                    included += 1;
                }
            }
        }

        let total_tokens = estimate_tokens(&out);
        PackedBrief {
            tier,
            content: out.trim().to_string(),
            estimated_tokens: total_tokens,
            entries_included: included,
            entries_omitted: omitted,
        }
    }
}

pub fn truncate_utf8_bytes(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

pub fn estimate_tokens(text: &str) -> usize {
    (text.len() + 3) / 4
}

