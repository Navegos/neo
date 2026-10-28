// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#[macro_export]
macro_rules! stack_guard {
    () => {
        if $crate::codegraph::kernel::stack::exhausted() {
            return ::core::default::Default::default();
        }
    };
}

pub mod buffers;
pub mod ccpp;
pub mod docstring;
pub mod go;
pub mod ids;
pub mod langs;
pub mod python;
pub mod rustlang;
pub mod stack;
pub mod textutil;
pub mod tsjs;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolNode {
    pub id: String,
    pub name: String,
    pub qualified_name: String,
    pub kind: String,
    pub visibility: Option<String>,
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub start_column: u32,
    pub end_column: u32,
    pub docstring: Option<String>,
    pub signature: Option<String>,
    pub return_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolEdge {
    pub source_id: String,
    pub target_id: String,
    pub kind: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolRef {
    pub from_node_id: String,
    pub name: String,
    pub kind: String,
    pub line: u32,
    pub column: u32,
    pub file_path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileExtraction {
    pub nodes: Vec<SymbolNode>,
    pub edges: Vec<SymbolEdge>,
    pub refs: Vec<SymbolRef>,
}

fn get_arena_str<'a>(arena: &'a [u8], offset: u32, len: u32) -> Option<&'a str> {
    if offset == buffers::NONE || (offset as usize + len as usize) > arena.len() {
        None
    } else {
        std::str::from_utf8(&arena[offset as usize..(offset + len) as usize]).ok()
    }
}

pub fn extract_file(file_path: &str, content: &str, language: &str) -> Result<FileExtraction, String> {
    let out = stack::run_guarded(|| match language {
        "rust" => rustlang::extract(file_path, content),
        "python" => python::extract(file_path, content),
        "go" => go::extract(file_path, content),
        "c" | "cpp" => ccpp::extract(file_path, content, language),
        "typescript" | "tsx" | "javascript" | "jsx" => tsjs::extract(file_path, content, language),
        _ => Err(format!("unsupported language: {}", language)),
    })?;

    decode_buffers(file_path, &out)
}

pub fn decode_buffers(file_path: &str, out: &buffers::EmitOut) -> Result<FileExtraction, String> {
    if out.meta.len() < buffers::META_SIZE {
        return Err("corrupt extraction meta".into());
    }

    let node_count = u32::from_le_bytes(out.meta[4..8].try_into().unwrap()) as usize;
    let edge_count = u32::from_le_bytes(out.meta[8..12].try_into().unwrap()) as usize;
    let ref_count = u32::from_le_bytes(out.meta[12..16].try_into().unwrap()) as usize;
    let arena = &out.arena;

    let mut nodes = Vec::with_capacity(node_count);
    for i in 0..node_count {
        let off = i * buffers::NODE_ROW_SIZE;
        if off + buffers::NODE_ROW_SIZE > out.nodes.len() {
            break;
        }
        let row = &out.nodes[off..off + buffers::NODE_ROW_SIZE];
        let kind_idx = row[0] as usize;
        let vis_code = row[1];
        let start_line = u32::from_le_bytes(row[4..8].try_into().unwrap());
        let end_line = u32::from_le_bytes(row[8..12].try_into().unwrap());
        let start_col = u32::from_le_bytes(row[12..16].try_into().unwrap());
        let end_col = u32::from_le_bytes(row[16..20].try_into().unwrap());

        let name_off = u32::from_le_bytes(row[20..24].try_into().unwrap());
        let name_len = u32::from_le_bytes(row[24..28].try_into().unwrap());
        let qn_off = u32::from_le_bytes(row[28..32].try_into().unwrap());
        let qn_len = u32::from_le_bytes(row[32..36].try_into().unwrap());
        let id_off = u32::from_le_bytes(row[36..40].try_into().unwrap());
        let id_len = u32::from_le_bytes(row[40..44].try_into().unwrap());
        let doc_off = u32::from_le_bytes(row[44..48].try_into().unwrap());
        let doc_len = u32::from_le_bytes(row[48..52].try_into().unwrap());
        let sig_off = u32::from_le_bytes(row[52..56].try_into().unwrap());
        let sig_len = u32::from_le_bytes(row[56..60].try_into().unwrap());
        let ret_off = u32::from_le_bytes(row[76..80].try_into().unwrap());
        let ret_len = u32::from_le_bytes(row[80..84].try_into().unwrap());

        let kind = buffers::NODE_KINDS.get(kind_idx).copied().unwrap_or("symbol").to_string();
        let visibility = match vis_code {
            1 => Some("public".to_string()),
            2 => Some("private".to_string()),
            3 => Some("protected".to_string()),
            4 => Some("internal".to_string()),
            _ => None,
        };

        let name = get_arena_str(arena, name_off, name_len).unwrap_or("").to_string();
        let qualified_name = get_arena_str(arena, qn_off, qn_len).unwrap_or(&name).to_string();
        let id = get_arena_str(arena, id_off, id_len).unwrap_or("").to_string();
        let docstring = get_arena_str(arena, doc_off, doc_len).map(|s| s.to_string());
        let signature = get_arena_str(arena, sig_off, sig_len).map(|s| s.to_string());
        let return_type = get_arena_str(arena, ret_off, ret_len).map(|s| s.to_string());

        nodes.push(SymbolNode {
            id,
            name,
            qualified_name,
            kind,
            visibility,
            file_path: file_path.to_string(),
            start_line,
            end_line,
            start_column: start_col,
            end_column: end_col,
            docstring,
            signature,
            return_type,
        });
    }

    let mut edges = Vec::with_capacity(edge_count);
    for i in 0..edge_count {
        let off = i * buffers::EDGE_ROW_SIZE;
        if off + buffers::EDGE_ROW_SIZE > out.edges.len() {
            break;
        }
        let row = &out.edges[off..off + buffers::EDGE_ROW_SIZE];
        let src_idx = u32::from_le_bytes(row[0..4].try_into().unwrap());
        let tgt_idx = u32::from_le_bytes(row[4..8].try_into().unwrap());
        let kind_idx = row[8] as usize;
        let line = u32::from_le_bytes(row[12..16].try_into().unwrap());
        let col = u32::from_le_bytes(row[16..20].try_into().unwrap());

        let src_str_off = u32::from_le_bytes(row[28..32].try_into().unwrap());
        let src_str_len = u32::from_le_bytes(row[32..36].try_into().unwrap());
        let tgt_str_off = u32::from_le_bytes(row[36..40].try_into().unwrap());
        let tgt_str_len = u32::from_le_bytes(row[40..44].try_into().unwrap());

        let source_id = if src_idx != buffers::NONE && (src_idx as usize) < nodes.len() {
            nodes[src_idx as usize].id.clone()
        } else {
            get_arena_str(arena, src_str_off, src_str_len).unwrap_or("").to_string()
        };

        let target_id = if tgt_idx != buffers::NONE && (tgt_idx as usize) < nodes.len() {
            nodes[tgt_idx as usize].id.clone()
        } else {
            get_arena_str(arena, tgt_str_off, tgt_str_len).unwrap_or("").to_string()
        };

        let kind = buffers::EDGE_KINDS.get(kind_idx).copied().unwrap_or("references").to_string();

        if !source_id.is_empty() && !target_id.is_empty() {
            edges.push(SymbolEdge {
                source_id,
                target_id,
                kind,
                line: if line != buffers::NONE { Some(line) } else { None },
                column: if col != buffers::NONE { Some(col) } else { None },
            });
        }
    }

    let mut refs = Vec::with_capacity(ref_count);
    for i in 0..ref_count {
        let off = i * buffers::REF_ROW_SIZE;
        if off + buffers::REF_ROW_SIZE > out.refs.len() {
            break;
        }
        let row = &out.refs[off..off + buffers::REF_ROW_SIZE];
        let from_idx = u32::from_le_bytes(row[0..4].try_into().unwrap());
        let kind_code = row[4];
        let line = u32::from_le_bytes(row[8..12].try_into().unwrap());
        let col = u32::from_le_bytes(row[12..16].try_into().unwrap());
        let name_off = u32::from_le_bytes(row[16..20].try_into().unwrap());
        let name_len = u32::from_le_bytes(row[20..24].try_into().unwrap());
        let from_str_off = u32::from_le_bytes(row[32..36].try_into().unwrap());
        let from_str_len = u32::from_le_bytes(row[36..40].try_into().unwrap());

        let from_node_id = if from_idx != buffers::NONE && (from_idx as usize) < nodes.len() {
            nodes[from_idx as usize].id.clone()
        } else {
            get_arena_str(arena, from_str_off, from_str_len).unwrap_or("").to_string()
        };

        let name = get_arena_str(arena, name_off, name_len).unwrap_or("").to_string();
        let kind = if kind_code == buffers::FUNCTION_REF_CODE {
            "function_ref".to_string()
        } else {
            buffers::EDGE_KINDS.get(kind_code as usize).copied().unwrap_or("references").to_string()
        };

        if !name.is_empty() && !from_node_id.is_empty() {
            refs.push(SymbolRef {
                from_node_id,
                name,
                kind,
                line,
                column: col,
                file_path: file_path.to_string(),
            });
        }
    }

    Ok(FileExtraction { nodes, edges, refs })
}

