// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/headroom/compression/json.rs

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct JsonPrunerConfig {
    pub prune_nulls: bool,
    pub prune_empty_containers: bool,
    pub max_array_items: usize,
}

impl Default for JsonPrunerConfig {
    fn default() -> Self {
        Self {
            prune_nulls: true,
            prune_empty_containers: false,
            max_array_items: 4,
        }
    }
}

pub fn prune_value(val: &mut Value, config: &JsonPrunerConfig) {
    match val {
        Value::Object(map) => {
            let mut keys_to_remove = Vec::new();
            for (k, v) in map.iter_mut() {
                prune_value(v, config);

                if config.prune_nulls && v.is_null() {
                    keys_to_remove.push(k.clone());
                } else if config.prune_empty_containers {
                    match v {
                        Value::Object(sub) if sub.is_empty() => keys_to_remove.push(k.clone()),
                        Value::Array(arr) if arr.is_empty() => keys_to_remove.push(k.clone()),
                        _ => {}
                    }
                }
            }
            for k in keys_to_remove {
                map.remove(&k);
            }
        }
        Value::Array(arr) => {
            let orig_len = arr.len();
            for item in arr.iter_mut() {
                prune_value(item, config);
            }
            if config.max_array_items > 0 && orig_len > config.max_array_items {
                arr.truncate(config.max_array_items);
                let omitted = orig_len - config.max_array_items;
                arr.push(Value::String(format!("[... {} items omitted by tokless]", omitted)));
            }
        }
        _ => {}
    }
}

pub fn prune_json(input: &str, config: &JsonPrunerConfig) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let Ok(mut val) = serde_json::from_str::<Value>(trimmed) else {
        return input.to_string();
    };

    prune_value(&mut val, config);

    serde_json::to_string(&val).unwrap_or_else(|_| input.to_string())
}

/// Matches a balanced JSON span starting at `start` in `text`
pub fn match_json_span(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let open_char = bytes[start];
    let close_char = match open_char {
        b'{' => b'}',
        b'[' => b']',
        _ => return None,
    };

    let mut stack = 1;
    let mut in_str = false;
    let mut esc = false;
    let mut i = start + 1;

    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }

        if b == b'"' {
            in_str = true;
        } else if b == open_char {
            stack += 1;
        } else if b == close_char {
            stack -= 1;
            if stack == 0 {
                return Some(i + 1);
            }
        }
        i += 1;
    }

    None
}

/// Detects balanced JSON objects or arrays inside arbitrary markdown or text and prunes them
pub fn prune_embedded_json(text: &str, config: &JsonPrunerConfig) -> String {
    let mut out = String::with_capacity(text.len());
    let mut idx = 0;
    let bytes = text.as_bytes();

    while idx < bytes.len() {
        let b = bytes[idx];
        if (b == b'{' || b == b'[') && (idx == 0 || bytes[idx - 1] == b' ' || bytes[idx - 1] == b'\n' || bytes[idx - 1] == b'`') {
            if let Some(end) = match_json_span(text, idx) {
                let candidate = &text[idx..end];
                if let Ok(mut val) = serde_json::from_str::<Value>(candidate) {
                    prune_value(&mut val, config);
                    if let Ok(pruned) = serde_json::to_string(&val) {
                        out.push_str(&pruned);
                        idx = end;
                        continue;
                    }
                }
            }
        }

        out.push(bytes[idx] as char);
        idx += 1;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prune_json_whitespace_and_nulls() {
        let input = r#"{
            "status": "ok",
            "error": null,
            "data": {
                "id": 123,
                "debug": null
            }
        }"#;

        let res = prune_json(input, &JsonPrunerConfig::default());
        let val: Value = serde_json::from_str(&res).unwrap();
        assert_eq!(val["status"], "ok");
        assert_eq!(val["data"]["id"], 123);
        assert!(val.get("error").is_none());
        assert!(val["data"].get("debug").is_none());
        assert!(!res.contains(' '));
    }

    #[test]
    fn test_truncate_repetitive_arrays() {
        let input = r#"{"items": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]}"#;
        let mut cfg = JsonPrunerConfig::default();
        cfg.max_array_items = 3;

        let res = prune_json(input, &cfg);
        assert!(res.contains(r#""items":[1,2,3,"[... 7 items omitted by tokless]"]"#));
    }

    #[test]
    fn test_prune_embedded_json() {
        let input = "API response received:\n```json\n{\n  \"status\": 200,\n  \"null_field\": null\n}\n```\nDone.";
        let res = prune_embedded_json(input, &JsonPrunerConfig::default());
        assert!(res.contains(r#"{"status":200}"#));
        assert!(!res.contains(r#""null_field": null"#));
    }
}
