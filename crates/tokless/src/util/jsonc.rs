// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/jsonc.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use regex::Regex;
pub use serde_json::Value;

pub type OrderedMap = serde_json::Map<String, Value>;

pub fn strip_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut in_str = false;
    let mut esc = false;
    let mut str_ch = '"';

    while i < n {
        let ch = chars[i];
        if in_str {
            out.push(ch);
            if esc {
                esc = false;
            } else if ch == '\\' {
                esc = true;
            } else if ch == str_ch {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_str = true;
            str_ch = ch;
            out.push(ch);
            i += 1;
            continue;
        }
        if ch == '/' && i + 1 < n && chars[i + 1] == '/' {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if ch == '/' && i + 1 < n && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < n && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        }
        out.push(ch);
        i += 1;
    }

    let re_trailing_comma = Regex::new(r",(\s*[}\]])").unwrap();
    re_trailing_comma.replace_all(&out, "$1").to_string()
}

pub fn parse_jsonc(text: &str) -> Result<Value, serde_json::Error> {
    let stripped = strip_comments(text);
    if stripped.trim().is_empty() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    serde_json::from_str(&stripped)
}

pub fn try_parse_jsonc(text: &str) -> Option<Value> {
    parse_jsonc(text).ok()
}

pub fn has_jsonc_comments(raw: &str) -> bool {
    let chars: Vec<char> = raw.chars().collect();
    let mut in_str = false;
    let mut esc = false;

    for i in 0..chars.len() {
        let c = chars[i];
        if in_str {
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        if c == '"' {
            in_str = true;
            continue;
        }
        if c == '/' && i + 1 < chars.len() && (chars[i + 1] == '/' || chars[i + 1] == '*') {
            return true;
        }
    }
    false
}

pub fn stringify_json(v: &Value) -> String {
    let mut s = serde_json::to_string_pretty(v).unwrap_or_default();
    s.push('\n');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_comments() {
        let input = r#"{
            // comment
            "foo": "bar", /* inline */
            "arr": [1, 2, ],
        }"#;
        let stripped = strip_comments(input);
        let parsed: Value = serde_json::from_str(&stripped).unwrap();
        assert_eq!(parsed["foo"], "bar");
        assert_eq!(parsed["arr"].as_array().unwrap().len(), 2);
    }
}
