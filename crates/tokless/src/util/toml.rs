// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/toml.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::collections::HashMap;
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum TomlValue {
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Array(Vec<String>),
    Map(HashMap<String, String>),
}

impl From<&str> for TomlValue {
    fn from(s: &str) -> Self {
        TomlValue::String(s.to_string())
    }
}

impl From<&String> for TomlValue {
    fn from(s: &String) -> Self {
        TomlValue::String(s.clone())
    }
}

impl From<String> for TomlValue {
    fn from(s: String) -> Self {
        TomlValue::String(s)
    }
}

impl From<&[String]> for TomlValue {
    fn from(v: &[String]) -> Self {
        TomlValue::Array(v.to_vec())
    }
}

impl From<bool> for TomlValue {
    fn from(b: bool) -> Self {
        TomlValue::Bool(b)
    }
}

impl From<i64> for TomlValue {
    fn from(i: i64) -> Self {
        TomlValue::Int(i)
    }
}

impl From<i32> for TomlValue {
    fn from(i: i32) -> Self {
        TomlValue::Int(i as i64)
    }
}

impl From<f64> for TomlValue {
    fn from(f: f64) -> Self {
        TomlValue::Float(f)
    }
}

impl From<Vec<String>> for TomlValue {
    fn from(v: Vec<String>) -> Self {
        TomlValue::Array(v)
    }
}

impl From<Vec<&str>> for TomlValue {
    fn from(v: Vec<&str>) -> Self {
        TomlValue::Array(v.into_iter().map(|s| s.to_string()).collect())
    }
}

impl From<HashMap<String, String>> for TomlValue {
    fn from(m: HashMap<String, String>) -> Self {
        TomlValue::Map(m)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TomlBlock {
    pub header: String,
    pub keys: Vec<String>,
    pub fields: HashMap<String, TomlValue>,
}

impl TomlBlock {
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            keys: Vec::new(),
            fields: HashMap::new(),
        }
    }

    pub fn set(&mut self, k: impl Into<String>, v: impl Into<TomlValue>) {
        let key = k.into();
        if !self.fields.contains_key(&key) {
            self.keys.push(key.clone());
        }
        self.fields.insert(key, v.into());
    }
}

pub fn toml_single_quoted(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

pub fn toml_escape_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

pub fn toml_quoted(s: &str) -> String {
    format!("\"{}\"", toml_escape_str(s))
}

pub fn toml_dotted_table_header(prefix: &str, path_key: &str) -> String {
    format!("{}.{}", prefix, toml_single_quoted(path_key))
}

fn fmt_toml_value(v: &TomlValue) -> String {
    match v {
        TomlValue::String(s) => format!("\"{}\"", toml_escape_str(s)),
        TomlValue::Bool(b) => b.to_string(),
        TomlValue::Int(i) => i.to_string(),
        TomlValue::Float(f) => f.to_string(),
        TomlValue::Array(arr) => {
            let parts: Vec<String> = arr.iter().map(|x| format!("\"{}\"", toml_escape_str(x))).collect();
            format!("[{}]", parts.join(", "))
        }
        TomlValue::Map(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let parts: Vec<String> = keys
                .into_iter()
                .map(|k| format!("{} = \"{}\"", k, toml_escape_str(&map[k])))
                .collect();
            format!("{{ {} }}", parts.join(", "))
        }
    }
}

pub fn render_block(block: &TomlBlock) -> String {
    let mut lines = vec![format!("[{}]", block.header)];
    for k in &block.keys {
        if let Some(v) = block.fields.get(k) {
            if k == "env" {
                if let TomlValue::Map(_) = v {
                    continue;
                }
            }
            lines.push(format!("{} = {}", k, fmt_toml_value(v)));
        }
    }
    if let Some(TomlValue::Map(env)) = block.fields.get("env") {
        lines.push(String::new());
        lines.push(format!("[{}.env]", block.header));
        let mut keys: Vec<&String> = env.keys().collect();
        keys.sort();
        for k in keys {
            lines.push(format!("{} = \"{}\"", k, toml_escape_str(&env[k])));
        }
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

#[derive(Debug, Clone, Copy)]
struct BlockRange {
    start: usize,
    end: usize,
}

pub fn strip_toml_comment(t: &str) -> String {
    let chars: Vec<char> = t.chars().collect();
    let mut in_str = false;
    let mut in_literal = false;
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
        if in_literal {
            if c == '\'' {
                in_literal = false;
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '\'' => in_literal = true,
            '#' => {
                let slice: String = chars[..i].iter().collect();
                return slice.trim_end_matches([' ', '\t']).to_string();
            }
            _ => {}
        }
    }
    t.to_string()
}

fn is_header_line(t: &str) -> bool {
    let clean = strip_toml_comment(t);
    if clean.starts_with("[[") {
        clean.ends_with("]]")
    } else {
        clean.starts_with('[') && clean.ends_with(']')
    }
}

fn find_block_range(src: &str, header: &str) -> Option<BlockRange> {
    let pattern = format!(r"(?m)^\[\s*{}\s*\](?:[ \t]*#.*)?$", regex::escape(header));
    let re_header = Regex::new(&pattern).ok()?;
    let loc = re_header.find(src)?;
    let start = loc.start();
    let rest = &src[loc.end()..];
    let offset = loc.end();
    let mut pos = 0;

    for line in rest.split_inclusive('\n') {
        let trimmed = line.trim();
        let t = strip_toml_comment(trimmed);
        if is_header_line(&t)
            && t != format!("[{}]", header)
            && t != format!("[[{}]]", header)
            && !t.starts_with(&format!("[{}.", header))
        {
            return Some(BlockRange {
                start,
                end: offset + pos,
            });
        }
        pos += line.len();
    }
    Some(BlockRange {
        start,
        end: src.len(),
    })
}

fn parse_scalar_fields(block_text: &str) -> TomlBlock {
    let mut b = TomlBlock::new("");
    let re_scalar = Regex::new(r#"^\s*(?:([A-Za-z0-9_-]+)|'([^']*)'|"([^"]*)")\s*=\s*(.+?)\s*$"#).unwrap();
    let re_num = Regex::new(r#"^-?\d+(\.\d+)?$"#).unwrap();

    for line in block_text.lines() {
        if let Some(caps) = re_scalar.captures(line) {
            let key = caps.get(1)
                .or_else(|| caps.get(2))
                .or_else(|| caps.get(3))
                .map(|m| m.as_str())
                .unwrap_or("");
            let raw_match = caps.get(4).map(|m| m.as_str()).unwrap_or("");
            let raw = strip_toml_comment(raw_match).trim().to_string();
            if raw.starts_with('[') || raw.starts_with('{') {
                continue;
            }
            if raw == "true" || raw == "false" {
                b.set(key, raw == "true");
            } else if re_num.is_match(&raw) {
                if let Ok(f) = raw.parse::<f64>() {
                    b.set(key, f);
                }
            } else if raw.starts_with('"') && raw.ends_with('"') && raw.len() >= 2 {
                let unescaped = raw[1..raw.len() - 1].replace("\\\"", "\"").replace("\\\\", "\\").replace("\\n", "\n");
                b.set(key, unescaped);
            } else if raw.starts_with('\'') && raw.ends_with('\'') && raw.len() >= 2 {
                b.set(key, raw[1..raw.len() - 1].replace("''", "'"));
            }
        }
    }
    b
}

pub fn upsert_block(src: &str, block: &TomlBlock, merge: bool) -> String {
    let mut effective = block.clone();
    if merge {
        if let Some(r) = find_block_range(src, &block.header) {
            let existing = parse_scalar_fields(&src[r.start..r.end]);
            let mut merged = TomlBlock::new(&block.header);
            for k in &existing.keys {
                if let Some(v) = existing.fields.get(k) {
                    merged.set(k.clone(), v.clone());
                }
            }
            for k in &block.keys {
                if let Some(v) = block.fields.get(k) {
                    merged.set(k.clone(), v.clone());
                }
            }
            effective = merged;
        }
    }
    let rendered = render_block(&effective);
    let r = match find_block_range(src, &block.header) {
        None => {
            let sep = if src.is_empty() || src.ends_with("\n\n") {
                ""
            } else if src.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            };
            return format!("{}{}{}", src, sep, rendered);
        }
        Some(range) => range,
    };

    let mut before = src[..r.start].to_string();
    let mut after = src[r.end..].to_string();

    if !before.is_empty() && !before.ends_with('\n') {
        before.push('\n');
    }
    if !after.starts_with('\n') && !after.is_empty() {
        after = format!("\n{}", after);
    }
    format!("{}{}{}", before, rendered, after)
}

pub fn remove_block(src: &str, header: &str) -> String {
    if let Some(r) = find_block_range(src, header) {
        format!("{}{}", &src[..r.start], &src[r.end..])
    } else {
        src.to_string()
    }
}

pub fn block_text(src: &str, header: &str) -> Option<String> {
    find_block_range(src, header).map(|r| src[r.start..r.end].to_string())
}

pub fn has_block(src: &str, header: &str) -> bool {
    find_block_range(src, header).is_some()
}

pub fn toml_block_field(src: &str, header: &str, key: &str) -> String {
    if let Some(r) = find_block_range(src, header) {
        let b = parse_scalar_fields(&src[r.start..r.end]);
        if let Some(TomlValue::String(s)) = b.fields.get(key) {
            return s.clone();
        }
    }
    String::new()
}

pub fn set_toml_top_key(src: &str, key: &str, value: &str) -> String {
    let line = format!("{} = \"{}\"", key, toml_escape_str(value));
    let pattern = format!(r#"(?m)^[ \t]*(?:{}|"{}"|'{}')[ \t]*=.*$"#, regex::escape(key), regex::escape(key), regex::escape(key));
    let re = Regex::new(&pattern).unwrap();
    if re.is_match(src) {
        return re.replace_all(src, line.as_str()).to_string();
    }
    if src.is_empty() {
        return format!("{}\n", line);
    }
    format!("{}\n{}", line, src)
}

pub fn get_toml_top_key(src: &str, key: &str) -> String {
    let pattern = format!(r#"(?m)^[ \t]*(?:{}|"{}"|'{}')[ \t]*=[ \t]*"([^"]*)""#, regex::escape(key), regex::escape(key), regex::escape(key));
    if let Ok(re) = Regex::new(&pattern) {
        if let Some(caps) = re.captures(src) {
            if let Some(m) = caps.get(1) {
                return m.as_str().to_string();
            }
        }
    }
    String::new()
}

pub fn remove_toml_top_key(src: &str, key: &str) -> String {
    let pattern = format!(r#"(?m)^[ \t]*(?:{}|"{}"|'{}')[ \t]*=.*\n?"#, regex::escape(key), regex::escape(key), regex::escape(key));
    if let Ok(re) = Regex::new(&pattern) {
        return re.replace_all(src, "").to_string();
    }
    src.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upsert_block_insert() {
        let mut b = TomlBlock::new("mcp_servers.codegraph");
        b.set("command", "codegraph");
        b.set("args", vec!["serve", "--mcp"]);

        let got = upsert_block("", &b, false);
        assert!(got.contains("[mcp_servers.codegraph]"));
        assert!(got.contains("command = \"codegraph\""));
        assert!(got.contains("args = [\"serve\", \"--mcp\"]"));
    }

    #[test]
    fn test_upsert_block_idempotent() {
        let mut b1 = TomlBlock::new("mcp_servers.codegraph");
        b1.set("command", "codegraph");
        b1.set("args", vec!["serve", "--mcp"]);

        let mut b2 = TomlBlock::new("features");
        b2.set("hooks", true);

        let mut b3 = TomlBlock::new("mcp_servers.context-mode");
        b3.set("command", "context-mode");

        let mut doc = upsert_block("", &b1, false);
        doc = upsert_block(&doc, &b2, true);
        doc = upsert_block(&doc, &b3, false);

        let mut doc2 = upsert_block(&doc, &b1, false);
        doc2 = upsert_block(&doc2, &b2, true);
        doc2 = upsert_block(&doc2, &b3, false);

        assert_eq!(doc, doc2);
    }

    #[test]
    fn test_remove_block() {
        let mut b1 = TomlBlock::new("mcp_servers.codegraph");
        b1.set("command", "codegraph");
        let mut b2 = TomlBlock::new("mcp_servers.context-mode");
        b2.set("command", "context-mode");

        let mut doc = upsert_block("", &b1, false);
        doc = upsert_block(&doc, &b2, false);

        let removed = remove_block(&doc, "mcp_servers.codegraph");
        assert!(!removed.contains("[mcp_servers.codegraph]"));
        assert!(removed.contains("[mcp_servers.context-mode]"));
    }
}
