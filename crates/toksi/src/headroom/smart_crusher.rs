// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/headroom/smart_crusher.rs
// created: 2026-11-13
// lastModified: 2026-11-13

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
        let raw = serde_json::to_string(&items).expect("serialize");
        let store = InMemoryCcrStore::default();
        let compressed = SmartCrusher::default()
            .compress_json_str(&raw, Some(&store))
            .expect("compressed");
        assert!(compressed.contains("<<ccr:"));
        assert!(compressed.contains("fatal panic occurred"));
        assert!(compressed.len() < raw.len());
        assert_eq!(store.len(), 1);
    }
}
