// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/smart_crusher.rs
// created: 2026-11-13
// lastModified: 2026-11-16

//! SmartCrusher — statistical JSON array compressor that preserves first/last items,
//! error/anomaly records, and stores the full original array in `CcrStore` under `<<ccr:HASH>>`.

use serde_json::{json, Value};

use super::ccr::{compute_key, marker_for, CcrStore};

#[derive(Debug, Clone)]
pub struct SmartCrusherConfig {
    pub min_items: usize,
    pub keep_first: usize,
    pub keep_last: usize,
    pub max_anomalies: usize,
}

impl Default for SmartCrusherConfig {
    fn default() -> Self {
        Self {
            min_items: 8,
            keep_first: 2,
            keep_last: 3,
            max_anomalies: 6,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SmartCrusher {
    config: SmartCrusherConfig,
}

impl Default for SmartCrusher {
    fn default() -> Self {
        Self::new(SmartCrusherConfig::default())
    }
}

impl SmartCrusher {
    pub fn new(config: SmartCrusherConfig) -> Self {
        Self { config }
    }

    pub fn compress_json_str(&self, raw: &str, store: Option<&dyn CcrStore>) -> Option<String> {
        self.compress_json_str_with_query(raw, "", store)
    }

    /// Compresses a JSON array string while unconditionally preserving rows named by selective
    /// query anchors (e.g. `#4475`, UUIDs, emails, quoted names, IDs) alongside first/last/anomalies.
    pub fn compress_json_str_with_query(
        &self,
        raw: &str,
        query: &str,
        store: Option<&dyn CcrStore>,
    ) -> Option<String> {
        let parsed: Value = serde_json::from_str(raw.trim()).ok()?;
        let arr = parsed.as_array()?;
        let total = arr.len();
        if total < self.config.min_items {
            return None;
        }

        let mut keep_indices = vec![false; total];
        for flag in keep_indices.iter_mut().take(self.config.keep_first.min(total)) {
            *flag = true;
        }
        let tail_start = total.saturating_sub(self.config.keep_last);
        for flag in keep_indices.iter_mut().skip(tail_start) {
            *flag = true;
        }

        // Preserve query-named rows when the anchors are selective (<= budget).
        let budget = self
            .config
            .keep_first
            .saturating_add(self.config.keep_last)
            .saturating_add(self.config.max_anomalies);
        let anchors = extract_query_anchors(query);
        if !anchors.is_empty() {
            let anchor_matches: Vec<usize> = arr
                .iter()
                .enumerate()
                .filter_map(|(idx, item)| {
                    if item_matches_anchors(item, &anchors) {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();
            if !anchor_matches.is_empty() && anchor_matches.len() <= budget {
                for idx in anchor_matches {
                    keep_indices[idx] = true;
                }
            }
        }

        let mut anomalies_kept = 0usize;
        for (idx, item) in arr.iter().enumerate() {
            if keep_indices[idx] || anomalies_kept >= self.config.max_anomalies {
                continue;
            }
            if is_anomaly_item(item) {
                keep_indices[idx] = true;
                anomalies_kept += 1;
            }
        }

        let kept: Vec<&Value> = arr
            .iter()
            .enumerate()
            .filter_map(|(i, v)| if keep_indices[i] { Some(v) } else { None })
            .collect();
        let dropped = total.saturating_sub(kept.len());
        if dropped == 0 {
            return None;
        }

        let marker = if let Some(ccr) = store {
            let key = compute_key(raw.as_bytes());
            ccr.put(&key, raw);
            Some(marker_for(&key))
        } else {
            None
        };

        let mut summary = json!({
            "_headroom_crushed": true,
            "total_items": total,
            "kept_items": kept.len(),
            "dropped_items": dropped,
            "samples": kept,
        });
        if let (Some(m), Some(obj)) = (marker, summary.as_object_mut()) {
            obj.insert("ccr_marker".to_string(), Value::String(m));
        }
        serde_json::to_string(&summary).ok()
    }
}

/// Extracts selective anchor identifiers (`#1234`, IDs, UUIDs, emails, quoted phrases, mixed identifiers).
pub fn extract_query_anchors(query: &str) -> Vec<String> {
    let mut anchors: Vec<String> = Vec::new();
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return anchors;
    }

    // Extract quoted spans `"..."` or `'...'`
    for quote in ['"', '\''] {
        let mut rest = trimmed;
        while let Some(start) = rest.find(quote) {
            let after = &rest[start + 1..];
            if let Some(end) = after.find(quote) {
                let inner = after[..end].trim();
                if inner.len() >= 2 {
                    let low = inner.to_ascii_lowercase();
                    if !anchors.contains(&low) {
                        anchors.push(low);
                    }
                }
                rest = &after[end + 1..];
            } else {
                break;
            }
        }
    }

    for raw_tok in trimmed.split_whitespace() {
        let tok = raw_tok.trim_matches(|c: char| matches!(c, ',' | '?' | '!' | ';' | '(' | ')' | '[' | ']' | '.'));
        if tok.is_empty() {
            continue;
        }
        if let Some(num) = tok.strip_prefix('#') {
            if !num.is_empty() && num.chars().all(|c| c.is_ascii_digit()) {
                let num_str = num.to_string();
                if !anchors.contains(&num_str) {
                    anchors.push(num_str);
                }
                continue;
            }
        }
        let low = tok.to_ascii_lowercase();
        let all_digits = low.len() >= 2 && low.chars().all(|c| c.is_ascii_digit());
        let has_email_or_host = low.contains('@') || (low.contains('.') && low.len() >= 5);
        let has_mixed_id = low.len() >= 3
            && (low.contains('-') || low.contains('_'))
            && low.chars().any(|c| c.is_ascii_alphanumeric());
        let has_alpha_and_digit =
            low.len() >= 3 && low.chars().any(|c| c.is_ascii_alphabetic()) && low.chars().any(|c| c.is_ascii_digit());

        if (all_digits || has_email_or_host || has_mixed_id || has_alpha_and_digit)
            && !anchors.contains(&low)
        {
            anchors.push(low);
        }
    }

    anchors
}

/// Returns true if the serialized JSON item matches any selective query anchor.
pub fn item_matches_anchors(item: &Value, anchors: &[String]) -> bool {
    if anchors.is_empty() {
        return false;
    }
    let text = item.to_string().to_ascii_lowercase();
    anchors.iter().any(|a| text.contains(a.as_str()))
}

fn is_anomaly_item(item: &Value) -> bool {
    if let Some(obj) = item.as_object() {
        if let Some(status) = obj.get("status").and_then(|v| v.as_u64()) {
            if status >= 400 {
                return true;
            }
        }
        if let Some(code) = obj.get("exit_code").or_else(|| obj.get("code")).and_then(|v| v.as_i64()) {
            if code != 0 {
                return true;
            }
        }
    }
    let text = item.to_string().to_ascii_lowercase();
    text.contains("error")
        || text.contains("fatal")
        || text.contains("panic")
        || text.contains("exception")
        || text.contains("failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headroom::ccr::InMemoryCcrStore;

    #[test]
    fn test_smart_crusher_preserves_anomalies_and_ccr() {
        let mut items = Vec::new();
        for i in 0..40 {
            if i == 17 {
                items.push(json!({"id": i, "status": 500, "msg": "fatal panic occurred"}));
            } else {
                items.push(json!({"id": i, "status": 200, "msg": "ok"}));
            }
        }
        let raw = serde_json::to_string(&items).unwrap_or_default();
        let store = InMemoryCcrStore::default();
        let compressed = SmartCrusher::default().compress_json_str(&raw, Some(&store));
        assert!(compressed.is_some());
        if let Some(c) = compressed {
            assert!(c.contains("<<ccr:"));
            assert!(c.contains("fatal panic occurred"));
            assert!(c.len() < raw.len());
            assert_eq!(store.len(), 1);
        }
    }

    #[test]
    fn test_smart_crusher_keeps_query_named_row_when_errors_fill_budget() {
        let items: Vec<Value> = (0..40)
            .map(|i| {
                let outcome = if i == 25 { "ok" } else { "failed" };
                json!({"number": 4450 + i, "title": format!("parser {} on case {}", outcome, i)})
            })
            .collect();
        let raw = serde_json::to_string(&items).unwrap_or_default();
        let store = InMemoryCcrStore::default();
        let compressed = SmartCrusher::default().compress_json_str_with_query(
            &raw,
            "Who opened issue #4475?",
            Some(&store),
        );
        assert!(compressed.is_some());
        if let Some(c) = compressed {
            assert!(c.contains("4475"));
        }
    }
}
