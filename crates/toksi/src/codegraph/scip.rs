// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/scip.rs
// created: 2026-11-09
// lastModified: 2026-11-13

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util::paths::normalize_path;

#[derive(Error, Debug)]
pub enum ScipError {
    #[error("Malformed SCIP protobuf payload: {0}")]
    MalformedProtobuf(String),
    #[error("I/O error reading/writing SCIP index: {0}")]
    Io(#[from] std::io::Error),
}

/// SCIP Symbol Role bitflags (aligned with scip.proto `SymbolRole` enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum SymbolRole {
    Unspecified = 0x0,
    Definition = 0x1,
    Import = 0x2,
    WriteAccess = 0x4,
    ReadAccess = 0x8,
    Generated = 0x10,
    Test = 0x20,
    ForwardDefinition = 0x40,
}

/// SCIP occurrence inside a source file document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScipOccurrence {
    /// Range formatted as `[startLine, startCol, endCol]` (3-len) or `[startLine, startCol, endLine, endCol]` (4-len).
    pub range: Vec<i32>,
    pub symbol: String,
    pub symbol_roles: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enclosing_range: Vec<i32>,
}

/// SCIP relationship between symbols.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScipRelationship {
    pub symbol: String,
    pub is_reference: bool,
    pub is_implementation: bool,
    pub is_type_definition: bool,
    pub is_definition: bool,
}

/// SCIP symbol descriptor AST metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScipSymbolInformation {
    pub symbol: String,
    #[serde(default)]
    pub documentation: Vec<String>,
    #[serde(default)]
    pub relationships: Vec<ScipRelationship>,
    #[serde(default)]
    pub kind: i32,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub enclosing_symbol: String,
}

/// SCIP source document model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScipDocument {
    pub language: String,
    pub relative_path: String,
    #[serde(default)]
    pub occurrences: Vec<ScipOccurrence>,
    #[serde(default)]
    pub symbols: Vec<ScipSymbolInformation>,
}

/// SCIP tool info metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScipToolInfo {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub arguments: Vec<String>,
}

/// SCIP index header metadata (`scip.Metadata`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScipMetadata {
    pub version: i32,
    pub tool_info: ScipToolInfo,
    pub project_root: String,
    pub text_document_encoding: i32,
}

/// Complete SCIP Index container (`scip.Index`) supporting Protobuf binary + JSON.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScipIndex {
    pub metadata: ScipMetadata,
    pub documents: Vec<ScipDocument>,
    #[serde(default)]
    pub external_symbols: Vec<ScipSymbolInformation>,
}

impl ScipIndex {
    pub fn new(project_root: &str) -> Self {
        Self {
            metadata: ScipMetadata {
                version: 0,
                tool_info: ScipToolInfo {
                    name: "codegraph".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    arguments: vec![],
                },
                project_root: normalize_path(project_root),
                text_document_encoding: 1, // UTF-8
            },
            documents: Vec::new(),
            external_symbols: Vec::new(),
        }
    }

    pub fn add_document(&mut self, mut doc: ScipDocument) {
        doc.relative_path = normalize_path(&doc.relative_path);
        self.documents.push(doc);
    }

    /// Build a complete SCIP Index from an indexed CodeGraph workspace.
    pub fn from_codegraph(cg: &super::CodeGraph) -> Self {
        let root_str = normalize_path(&cg.root.to_string_lossy());
        let mut index = Self::new(&root_str);

        let all_nodes = cg.cache.get_all_nodes();
        let all_edges = cg.cache.get_all_edges();
        let all_refs = cg.cache.get_all_refs();
        let file_langs: HashMap<String, String> = cg
            .cache
            .get_indexed_files()
            .into_iter()
            .map(|(fp, lang, _)| (normalize_path(&fp), lang))
            .collect();

        let node_by_id: HashMap<&str, &super::kernel::SymbolNode> =
            all_nodes.iter().map(|n| (n.id.as_str(), n)).collect();

        let make_scip_sym = |n: &super::kernel::SymbolNode| -> String {
            let norm = normalize_path(&n.file_path);
            let lang = file_langs.get(&norm).map(|s| s.as_str()).unwrap_or("unknown");
            let sub = super::cache::derive_subproject_id(&norm);
            let suffix = match n.kind.as_str() {
                "function" | "method" => format!("{}().", n.qualified_name),
                "class" | "struct" | "interface" | "trait" | "enum" => {
                    format!("{}#", n.qualified_name)
                }
                _ => format!("{}.", n.qualified_name),
            };
            format_scip_symbol(lang, &sub, env!("CARGO_PKG_VERSION"), &suffix)
        };

        let mut sym_by_node_id: HashMap<&str, String> = HashMap::new();
        for n in &all_nodes {
            sym_by_node_id.insert(n.id.as_str(), make_scip_sym(n));
        }

        // Index edges as SCIP relationships
        let mut rels_by_source: HashMap<&str, Vec<ScipRelationship>> = HashMap::new();
        for e in &all_edges {
            if let Some(target_sym) = sym_by_node_id.get(e.target_id.as_str()) {
                rels_by_source.entry(e.source_id.as_str()).or_default().push(
                    ScipRelationship {
                        symbol: target_sym.clone(),
                        is_reference: e.kind == "calls" || e.kind == "references",
                        is_implementation: e.kind == "implements",
                        is_type_definition: e.kind == "extends",
                        is_definition: e.kind == "contains",
                    },
                );
            }
        }

        let mut docs_map: BTreeMap<String, ScipDocument> = BTreeMap::new();
        for n in &all_nodes {
            let norm = normalize_path(&n.file_path);
            let lang = file_langs
                .get(&norm)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            let doc = docs_map.entry(norm.clone()).or_insert_with(|| ScipDocument {
                language: lang,
                relative_path: norm.clone(),
                occurrences: Vec::new(),
                symbols: Vec::new(),
            });

            if n.kind == "file" {
                continue;
            }
            let sym = sym_by_node_id
                .get(n.id.as_str())
                .cloned()
                .unwrap_or_else(|| make_scip_sym(n));

            doc.occurrences.push(ScipOccurrence {
                range: vec![
                    n.start_line.saturating_sub(1) as i32,
                    n.start_column as i32,
                    n.end_line.saturating_sub(1) as i32,
                    n.end_column as i32,
                ],
                symbol: sym.clone(),
                symbol_roles: SymbolRole::Definition as i32,
                enclosing_range: vec![],
            });

            doc.symbols.push(ScipSymbolInformation {
                symbol: sym,
                documentation: n.docstring.iter().cloned().collect(),
                relationships: rels_by_source.remove(n.id.as_str()).unwrap_or_default(),
                kind: node_kind_to_scip_enum(&n.kind),
                display_name: n.name.clone(),
                enclosing_symbol: n.qualified_name.clone(),
            });
        }

        for r in &all_refs {
            let norm = normalize_path(&r.file_path);
            let lang = file_langs
                .get(&norm)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            let doc = docs_map.entry(norm.clone()).or_insert_with(|| ScipDocument {
                language: lang,
                relative_path: norm.clone(),
                occurrences: Vec::new(),
                symbols: Vec::new(),
            });
            let target_sym = node_by_id
                .get(r.name.as_str())
                .and_then(|n| sym_by_node_id.get(n.id.as_str()).cloned())
                .unwrap_or_else(|| format!("local {}", r.name));
            doc.occurrences.push(ScipOccurrence {
                range: vec![
                    r.line.saturating_sub(1) as i32,
                    r.column as i32,
                    (r.column as i32) + (r.name.len() as i32),
                ],
                symbol: target_sym,
                symbol_roles: SymbolRole::ReadAccess as i32,
                enclosing_range: vec![],
            });
        }

        for (_, doc) in docs_map {
            index.add_document(doc);
        }
        index
    }

    pub fn build_symbol_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for doc in &self.documents {
            for occ in &doc.occurrences {
                if (occ.symbol_roles & (SymbolRole::Definition as i32)) != 0 {
                    map.insert(occ.symbol.clone(), doc.relative_path.clone());
                }
            }
        }
        map
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Encode `ScipIndex` to canonical `scip.proto` Protobuf wire binary bytes.
    pub fn to_protobuf(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1024);

        let meta_bytes = encode_metadata(&self.metadata);
        write_len_field(&mut out, 1, &meta_bytes);

        for doc in &self.documents {
            let doc_bytes = encode_document(doc);
            write_len_field(&mut out, 2, &doc_bytes);
        }

        for ext in &self.external_symbols {
            let ext_bytes = encode_symbol_info(ext);
            write_len_field(&mut out, 3, &ext_bytes);
        }

        out
    }

    /// Decode canonical `scip.proto` Protobuf wire bytes into `ScipIndex`.
    pub fn from_protobuf(bytes: &[u8]) -> Result<Self, ScipError> {
        let mut idx = ScipIndex::new("");
        let mut pos = 0usize;
        while pos < bytes.len() {
            let (field_num, wire_type) = read_key(bytes, &mut pos)?;
            match (field_num, wire_type) {
                (1, 2) => {
                    let sub = read_bytes(bytes, &mut pos)?;
                    idx.metadata = decode_metadata(sub)?;
                }
                (2, 2) => {
                    let sub = read_bytes(bytes, &mut pos)?;
                    idx.documents.push(decode_document(sub)?);
                }
                (3, 2) => {
                    let sub = read_bytes(bytes, &mut pos)?;
                    idx.external_symbols.push(decode_symbol_info(sub)?);
                }
                (_, wt) => skip_field(bytes, &mut pos, wt)?,
            }
        }
        Ok(idx)
    }

    pub fn write_protobuf(&self, path: &Path) -> Result<(), ScipError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.to_protobuf())?;
        Ok(())
    }

    pub fn read_protobuf(path: &Path) -> Result<Self, ScipError> {
        let bytes = fs::read(path)?;
        Self::from_protobuf(&bytes)
    }
}

fn node_kind_to_scip_enum(kind: &str) -> i32 {
    match kind {
        "class" => 7,
        "constant" => 8,
        "enum" => 11,
        "field" | "property" => 15,
        "function" => 17,
        "interface" => 21,
        "method" => 26,
        "module" => 29,
        "struct" => 49,
        "trait" => 53,
        "type" => 54,
        "variable" => 61,
        _ => 0,
    }
}

/// Formats a canonical SCIP symbol string: `<scheme> <package> <version> <descriptors>`
pub fn format_scip_symbol(scheme: &str, package: &str, version: &str, descriptors: &str) -> String {
    format!("{} {} {} {}", scheme, package, version, descriptors)
}

/// Parses a SCIP symbol string back into structural components (`scheme`, `package`, `version`, `descriptors`).
pub fn parse_scip_symbol(symbol: &str) -> Option<(&str, &str, &str, &str)> {
    let parts: Vec<&str> = symbol.splitn(4, ' ').collect();
    if parts.len() == 4 {
        Some((parts[0], parts[1], parts[2], parts[3]))
    } else {
        None
    }
}

// --- Pure-Rust SCIP Protobuf wire codec (scip.proto field-exact) ----------------

fn write_varint(buf: &mut Vec<u8>, mut val: u64) {
    loop {
        let byte = (val & 0x7F) as u8;
        val >>= 7;
        if val == 0 {
            buf.push(byte);
            break;
        }
        buf.push(byte | 0x80);
    }
}

fn write_key(buf: &mut Vec<u8>, field_num: u32, wire_type: u8) {
    write_varint(buf, ((field_num as u64) << 3) | (wire_type as u64));
}

fn write_int32_field(buf: &mut Vec<u8>, field_num: u32, val: i32) {
    if val != 0 {
        write_key(buf, field_num, 0);
        write_varint(buf, val as i64 as u64);
    }
}

fn write_bool_field(buf: &mut Vec<u8>, field_num: u32, val: bool) {
    if val {
        write_key(buf, field_num, 0);
        write_varint(buf, 1);
    }
}

fn write_str_field(buf: &mut Vec<u8>, field_num: u32, s: &str) {
    if !s.is_empty() {
        write_key(buf, field_num, 2);
        write_varint(buf, s.len() as u64);
        buf.extend_from_slice(s.as_bytes());
    }
}

fn write_len_field(buf: &mut Vec<u8>, field_num: u32, data: &[u8]) {
    write_key(buf, field_num, 2);
    write_varint(buf, data.len() as u64);
    buf.extend_from_slice(data);
}

fn write_packed_i32(buf: &mut Vec<u8>, field_num: u32, vals: &[i32]) {
    if vals.is_empty() {
        return;
    }
    let mut inner = Vec::with_capacity(vals.len() * 2);
    for &v in vals {
        write_varint(&mut inner, v as i64 as u64);
    }
    write_len_field(buf, field_num, &inner);
}

fn encode_metadata(m: &ScipMetadata) -> Vec<u8> {
    let mut buf = Vec::new();
    write_int32_field(&mut buf, 1, m.version);
    let mut tool_buf = Vec::new();
    write_str_field(&mut tool_buf, 1, &m.tool_info.name);
    write_str_field(&mut tool_buf, 2, &m.tool_info.version);
    for arg in &m.tool_info.arguments {
        write_str_field(&mut tool_buf, 3, arg);
    }
    write_len_field(&mut buf, 2, &tool_buf);
    write_str_field(&mut buf, 3, &m.project_root);
    write_int32_field(&mut buf, 4, m.text_document_encoding);
    buf
}

fn encode_document(d: &ScipDocument) -> Vec<u8> {
    let mut buf = Vec::new();
    write_str_field(&mut buf, 1, &d.relative_path);
    for occ in &d.occurrences {
        let mut obuf = Vec::new();
        write_packed_i32(&mut obuf, 1, &occ.range);
        write_str_field(&mut obuf, 2, &occ.symbol);
        write_int32_field(&mut obuf, 3, occ.symbol_roles);
        write_packed_i32(&mut obuf, 7, &occ.enclosing_range);
        write_len_field(&mut buf, 2, &obuf);
    }
    for sym in &d.symbols {
        let sbuf = encode_symbol_info(sym);
        write_len_field(&mut buf, 3, &sbuf);
    }
    write_str_field(&mut buf, 4, &d.language);
    buf
}

fn encode_symbol_info(s: &ScipSymbolInformation) -> Vec<u8> {
    let mut buf = Vec::new();
    write_str_field(&mut buf, 1, &s.symbol);
    for doc in &s.documentation {
        write_str_field(&mut buf, 3, doc);
    }
    for rel in &s.relationships {
        let mut rbuf = Vec::new();
        write_str_field(&mut rbuf, 1, &rel.symbol);
        write_bool_field(&mut rbuf, 2, rel.is_reference);
        write_bool_field(&mut rbuf, 3, rel.is_implementation);
        write_bool_field(&mut rbuf, 4, rel.is_type_definition);
        write_bool_field(&mut rbuf, 5, rel.is_definition);
        write_len_field(&mut buf, 4, &rbuf);
    }
    write_int32_field(&mut buf, 5, s.kind);
    write_str_field(&mut buf, 6, &s.display_name);
    write_str_field(&mut buf, 8, &s.enclosing_symbol);
    buf
}

fn read_varint(bytes: &[u8], pos: &mut usize) -> Result<u64, ScipError> {
    let mut res = 0u64;
    let mut shift = 0u32;
    while *pos < bytes.len() {
        let b = bytes[*pos];
        *pos += 1;
        res |= ((b & 0x7F) as u64) << shift;
        if (b & 0x80) == 0 {
            return Ok(res);
        }
        shift += 7;
        if shift >= 64 {
            return Err(ScipError::MalformedProtobuf("varint overflow".into()));
        }
    }
    Err(ScipError::MalformedProtobuf("unexpected EOF in varint".into()))
}

fn read_key(bytes: &[u8], pos: &mut usize) -> Result<(u32, u8), ScipError> {
    let v = read_varint(bytes, pos)?;
    Ok(((v >> 3) as u32, (v & 0x7) as u8))
}

fn read_bytes<'a>(bytes: &'a [u8], pos: &mut usize) -> Result<&'a [u8], ScipError> {
    let len = read_varint(bytes, pos)? as usize;
    if *pos + len > bytes.len() {
        return Err(ScipError::MalformedProtobuf("length-delimited field exceeds buffer".into()));
    }
    let slice = &bytes[*pos..*pos + len];
    *pos += len;
    Ok(slice)
}

fn read_string(bytes: &[u8], pos: &mut usize) -> Result<String, ScipError> {
    let slice = read_bytes(bytes, pos)?;
    Ok(String::from_utf8_lossy(slice).into_owned())
}

fn skip_field(bytes: &[u8], pos: &mut usize, wt: u8) -> Result<(), ScipError> {
    match wt {
        0 => {
            read_varint(bytes, pos)?;
        }
        1 => {
            if *pos + 8 > bytes.len() {
                return Err(ScipError::MalformedProtobuf("EOF skipping fixed64".into()));
            }
            *pos += 8;
        }
        2 => {
            let _ = read_bytes(bytes, pos)?;
        }
        5 => {
            if *pos + 4 > bytes.len() {
                return Err(ScipError::MalformedProtobuf("EOF skipping fixed32".into()));
            }
            *pos += 4;
        }
        _ => {
            return Err(ScipError::MalformedProtobuf(format!("unsupported wire type {wt}")));
        }
    }
    Ok(())
}

fn decode_metadata(bytes: &[u8]) -> Result<ScipMetadata, ScipError> {
    let mut meta = ScipMetadata {
        version: 0,
        tool_info: ScipToolInfo {
            name: String::new(),
            version: String::new(),
            arguments: Vec::new(),
        },
        project_root: String::new(),
        text_document_encoding: 0,
    };
    let mut pos = 0;
    while pos < bytes.len() {
        let (f, wt) = read_key(bytes, &mut pos)?;
        match (f, wt) {
            (1, 0) => meta.version = read_varint(bytes, &mut pos)? as i32,
            (2, 2) => {
                let sub = read_bytes(bytes, &mut pos)?;
                let mut sp = 0;
                while sp < sub.len() {
                    let (tf, twt) = read_key(sub, &mut sp)?;
                    match (tf, twt) {
                        (1, 2) => meta.tool_info.name = read_string(sub, &mut sp)?,
                        (2, 2) => meta.tool_info.version = read_string(sub, &mut sp)?,
                        (3, 2) => meta.tool_info.arguments.push(read_string(sub, &mut sp)?),
                        (_, wt2) => skip_field(sub, &mut sp, wt2)?,
                    }
                }
            }
            (3, 2) => meta.project_root = read_string(bytes, &mut pos)?,
            (4, 0) => meta.text_document_encoding = read_varint(bytes, &mut pos)? as i32,
            (_, wt2) => skip_field(bytes, &mut pos, wt2)?,
        }
    }
    Ok(meta)
}

fn decode_document(bytes: &[u8]) -> Result<ScipDocument, ScipError> {
    let mut doc = ScipDocument {
        language: String::new(),
        relative_path: String::new(),
        occurrences: Vec::new(),
        symbols: Vec::new(),
    };
    let mut pos = 0;
    while pos < bytes.len() {
        let (f, wt) = read_key(bytes, &mut pos)?;
        match (f, wt) {
            (1, 2) => doc.relative_path = normalize_path(&read_string(bytes, &mut pos)?),
            (2, 2) => {
                let sub = read_bytes(bytes, &mut pos)?;
                doc.occurrences.push(decode_occurrence(sub)?);
            }
            (3, 2) => {
                let sub = read_bytes(bytes, &mut pos)?;
                doc.symbols.push(decode_symbol_info(sub)?);
            }
            (4, 2) => doc.language = read_string(bytes, &mut pos)?,
            (_, wt2) => skip_field(bytes, &mut pos, wt2)?,
        }
    }
    Ok(doc)
}

fn decode_occurrence(bytes: &[u8]) -> Result<ScipOccurrence, ScipError> {
    let mut occ = ScipOccurrence {
        range: Vec::new(),
        symbol: String::new(),
        symbol_roles: 0,
        enclosing_range: Vec::new(),
    };
    let mut pos = 0;
    while pos < bytes.len() {
        let (f, wt) = read_key(bytes, &mut pos)?;
        match (f, wt) {
            (1, 2) => {
                let sub = read_bytes(bytes, &mut pos)?;
                let mut sp = 0;
                while sp < sub.len() {
                    occ.range.push(read_varint(sub, &mut sp)? as i32);
                }
            }
            (1, 0) => occ.range.push(read_varint(bytes, &mut pos)? as i32),
            (2, 2) => occ.symbol = read_string(bytes, &mut pos)?,
            (3, 0) => occ.symbol_roles = read_varint(bytes, &mut pos)? as i32,
            (7, 2) => {
                let sub = read_bytes(bytes, &mut pos)?;
                let mut sp = 0;
                while sp < sub.len() {
                    occ.enclosing_range.push(read_varint(sub, &mut sp)? as i32);
                }
            }
            (7, 0) => occ.enclosing_range.push(read_varint(bytes, &mut pos)? as i32),
            (_, wt2) => skip_field(bytes, &mut pos, wt2)?,
        }
    }
    Ok(occ)
}

fn decode_symbol_info(bytes: &[u8]) -> Result<ScipSymbolInformation, ScipError> {
    let mut sym = ScipSymbolInformation {
        symbol: String::new(),
        documentation: Vec::new(),
        relationships: Vec::new(),
        kind: 0,
        display_name: String::new(),
        enclosing_symbol: String::new(),
    };
    let mut pos = 0;
    while pos < bytes.len() {
        let (f, wt) = read_key(bytes, &mut pos)?;
        match (f, wt) {
            (1, 2) => sym.symbol = read_string(bytes, &mut pos)?,
            (3, 2) => sym.documentation.push(read_string(bytes, &mut pos)?),
            (4, 2) => {
                let sub = read_bytes(bytes, &mut pos)?;
                let mut rel = ScipRelationship {
                    symbol: String::new(),
                    is_reference: false,
                    is_implementation: false,
                    is_type_definition: false,
                    is_definition: false,
                };
                let mut sp = 0;
                while sp < sub.len() {
                    let (rf, rwt) = read_key(sub, &mut sp)?;
                    match (rf, rwt) {
                        (1, 2) => rel.symbol = read_string(sub, &mut sp)?,
                        (2, 0) => rel.is_reference = read_varint(sub, &mut sp)? != 0,
                        (3, 0) => rel.is_implementation = read_varint(sub, &mut sp)? != 0,
                        (4, 0) => rel.is_type_definition = read_varint(sub, &mut sp)? != 0,
                        (5, 0) => rel.is_definition = read_varint(sub, &mut sp)? != 0,
                        (_, rwt2) => skip_field(sub, &mut sp, rwt2)?,
                    }
                }
                sym.relationships.push(rel);
            }
            (5, 0) => sym.kind = read_varint(bytes, &mut pos)? as i32,
            (6, 2) => sym.display_name = read_string(bytes, &mut pos)?,
            (8, 2) => sym.enclosing_symbol = read_string(bytes, &mut pos)?,
            (_, wt2) => skip_field(bytes, &mut pos, wt2)?,
        }
    }
    Ok(sym)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scip_symbol_formatting() {
        let sym = format_scip_symbol("rust-analyzer", "cargo toksi", "0.1.0", "codegraph/CodeGraph#");
        let parsed = parse_scip_symbol(&sym);
        assert!(parsed.is_some());
        let (scheme, pkg, ver, desc) = parsed.expect("parsed");
        assert_eq!(scheme, "rust-analyzer");
        assert_eq!(pkg, "cargo");
        assert_eq!(ver, "toksi");
        assert!(desc.starts_with("0.1.0 "));
    }

    #[test]
    fn test_scip_protobuf_roundtrip() {
        let mut index = ScipIndex::new("/workspace");
        index.add_document(ScipDocument {
            language: "rust".to_string(),
            relative_path: "src\\lib.rs".to_string(),
            occurrences: vec![ScipOccurrence {
                range: vec![10, 4, 10, 20],
                symbol: "rust-analyzer cargo app 1.0.0 lib/foo().".to_string(),
                symbol_roles: SymbolRole::Definition as i32,
                enclosing_range: vec![10, 0, 12, 1],
            }],
            symbols: vec![ScipSymbolInformation {
                symbol: "rust-analyzer cargo app 1.0.0 lib/foo().".to_string(),
                documentation: vec!["Computes foo.".to_string()],
                relationships: vec![ScipRelationship {
                    symbol: "rust-analyzer cargo app 1.0.0 lib/bar().".to_string(),
                    is_reference: true,
                    is_implementation: false,
                    is_type_definition: false,
                    is_definition: false,
                }],
                kind: 17,
                display_name: "foo".to_string(),
                enclosing_symbol: "lib::foo".to_string(),
            }],
        });

        let bin = index.to_protobuf();
        assert!(!bin.is_empty());
        let decoded = ScipIndex::from_protobuf(&bin).expect("decode scip protobuf");
        assert_eq!(decoded, index);
        let map = decoded.build_symbol_map();
        assert_eq!(
            map.get("rust-analyzer cargo app 1.0.0 lib/foo().").map(|s| s.as_str()),
            Some("src/lib.rs")
        );
    }

    #[test]
    fn test_scip_from_codegraph() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join(".git")).expect("git");
        fs::create_dir_all(dir.path().join("src")).expect("src");
        fs::write(dir.path().join("src/mod.rs"), "pub fn compute() -> i32 { 42 }\n").expect("write");

        let mut cg = super::super::CodeGraph::open(dir.path()).expect("open");
        cg.ensure_indexed().expect("index");
        let scip_idx = ScipIndex::from_codegraph(&cg);
        assert_eq!(scip_idx.documents.len(), 1);
        assert_eq!(scip_idx.documents[0].relative_path, "src/mod.rs");
        assert!(!scip_idx.documents[0].symbols.is_empty());
    }
}
