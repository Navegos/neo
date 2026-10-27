// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/jsonc.rs
// created: 2026-10-05
// lastModified: 2026-10-27

pub use serde_json::Value;

pub type OrderedMap = serde_json::Map<String, Value>;

pub fn strip_comments(s: &str) -> String {
    let mut out: Vec<u8> = Vec::with_capacity(s.len());
    let bytes = s.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    let mut in_str = false;
    let mut esc = false;
    let mut str_ch = b'"';

    while i < n {
        let ch = bytes[i];
        if in_str {
            out.push(ch);
            if esc {
                esc = false;
            } else if ch == b'\\' {
                esc = true;
            } else if ch == str_ch {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if ch == b'"' || ch == b'\'' {
            in_str = true;
            str_ch = ch;
            out.push(ch);
            i += 1;
            continue;
        }
        if ch == b'/' && i + 1 < n && bytes[i + 1] == b'/' {
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if ch == b'/' && i + 1 < n && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < n && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i += 2;
            continue;
        }
        if ch == b'}' || ch == b']' {
            let mut last = out.len();
            while last > 0
                && (out[last - 1] == b' '
                    || out[last - 1] == b'\t'
                    || out[last - 1] == b'\r'
                    || out[last - 1] == b'\n')
            {
                last -= 1;
            }
            if last > 0 && out[last - 1] == b',' {
                out[last - 1] = b' ';
            }
        }
        out.push(ch);
        i += 1;
    }

    String::from_utf8_lossy(&out).to_string()
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

pub fn try_parse_jsonc_map(text: &str) -> Option<OrderedMap> {
    match parse_jsonc(text).ok()? {
        Value::Object(m) => Some(m),
        _ => None,
    }
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

    #[test]
    fn test_strip_comments_preserves_strings_with_commas() {
        for val in [
            ",]",
            ",}",
            "a, ]",
            "a,\t}",
            "escaped quote: \",]",
            "escaped backslash: \\,}",
            "ends with backslash: \\",
            "https://example.test/a,] /* text,} */",
        ] {
            let json_input = serde_json::json!({ val: val }).to_string();
            let stripped = strip_comments(&json_input);
            let parsed: Value = serde_json::from_str(&stripped).unwrap();
            assert_eq!(parsed[val], val);
        }
    }
}
