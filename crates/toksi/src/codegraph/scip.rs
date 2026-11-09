// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/codegraph/scip.rs
// created: 2026-11-09
// lastModified: 2026-11-09

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util::paths::normalize_path;

#[derive(Error, Debug)]
pub enum ScipError {
    #[error("SCIP serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid SCIP symbol string: '{0}'")]
    InvalidSymbol(String),
    #[error("Document not found: '{0}'")]
    DocumentNotFound(String),
}

/// Symbol roles conforming to the SCIP specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum SymbolRole {
    Unspecified = 0,
    Definition = 1,
    Import = 2,
    WriteAccess = 4,
    ReadAccess = 8,
    Test = 16,
}

/// A single symbol occurrence in a source code document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScipOccurrence {
    /// 3-element (start_line, start_col, end_col) or 4-element (start_line, start_col, end_line, end_col) range
    pub range: Vec<i32>,
    pub symbol: String,
    pub symbol_roles: i32,
    pub override_documentation: Option<Vec<String>>,
}

/// Symbol metadata and signature information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScipSymbolInformation {
    pub symbol: String,
    pub documentation: Vec<String>,
    pub relationships: Vec<ScipRelationship>,
    pub kind: String,
    pub display_name: String,
}

/// Cross-symbol relationships (e.g. implementation, reference).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScipRelationship {
    pub symbol: String,
    pub is_reference: bool,
    pub is_implementation: bool,
    pub is_type_definition: bool,
}

/// Indexed SCIP document containing occurrences and symbols for a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScipDocument {
    pub relative_path: String,
    pub language: String,
    pub occurrences: Vec<ScipOccurrence>,
    pub symbols: Vec<ScipSymbolInformation>,
}

/// Root SCIP Index structure containing all indexed documents and metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScipIndex {
    pub project_root: String,
    pub tool_name: String,
    pub tool_version: String,
    pub documents: Vec<ScipDocument>,
}

impl ScipIndex {
    pub fn new(project_root: &str) -> Self {
        Self {
            project_root: normalize_path(project_root),
            tool_name: "toksi-scip".to_string(),
            tool_version: "3.4.0".to_string(),
            documents: Vec::new(),
        }
    }

    pub fn add_document(&mut self, mut doc: ScipDocument) {
        doc.relative_path = normalize_path(&doc.relative_path);
        self.documents.push(doc);
    }

    /// Finds all definitions of a given symbol across all indexed documents.
    pub fn find_definitions(&self, symbol: &str) -> Vec<(&ScipDocument, &ScipOccurrence)> {
        let mut results = Vec::new();
        for doc in &self.documents {
            for occ in &doc.occurrences {
                if occ.symbol == symbol && (occ.symbol_roles & (SymbolRole::Definition as i32)) != 0 {
                    results.push((doc, occ));
                }
            }
        }
        results
    }

    /// Finds all references to a given symbol across all indexed documents.
    pub fn find_references(&self, symbol: &str) -> Vec<(&ScipDocument, &ScipOccurrence)> {
        let mut results = Vec::new();
        for doc in &self.documents {
            for occ in &doc.occurrences {
                if occ.symbol == symbol && (occ.symbol_roles & (SymbolRole::Definition as i32)) == 0 {
                    results.push((doc, occ));
                }
            }
        }
        results
    }

    /// Builds a cross-reference map mapping symbols to their (definition_count, reference_count).
    pub fn cross_reference_map(&self) -> HashMap<String, (usize, usize)> {
        let mut map: HashMap<String, (usize, usize)> = HashMap::new();
        for doc in &self.documents {
            for occ in &doc.occurrences {
                let entry = map.entry(occ.symbol.clone()).or_insert((0, 0));
                if (occ.symbol_roles & (SymbolRole::Definition as i32)) != 0 {
                    entry.0 += 1;
                } else {
                    entry.1 += 1;
                }
            }
        }
        map
    }
}

/// Formats a standardized SCIP symbol string:
/// `scip-<scheme> <manager> <pkg> <version> <descriptor>`
pub fn format_scip_symbol(
    scheme: &str,
    package_manager: &str,
    package_name: &str,
    version: &str,
    descriptor: &str,
) -> String {
    format!(
        "scip-{} {} {} {} {}",
        scheme, package_manager, package_name, version, descriptor
    )
}

/// Parses a SCIP symbol string into its constituent tuple elements.
pub fn parse_scip_symbol(
    symbol_str: &str,
) -> Result<(&str, &str, &str, &str, &str), ScipError> {
    let parts: Vec<&str> = symbol_str.split_whitespace().collect();
    if parts.len() < 5 {
        return Err(ScipError::InvalidSymbol(symbol_str.to_string()));
    }
    let scheme = parts[0].strip_prefix("scip-").unwrap_or(parts[0]);
    Ok((scheme, parts[1], parts[2], parts[3], parts[4]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scip_index_definitions_and_references() {
        let mut index = ScipIndex::new("/workspace");

        let sym = format_scip_symbol("rust", "cargo", "toksi", "3.4.0", "codegraph/scip/format_scip_symbol#");
        assert!(sym.contains("cargo toksi 3.4.0"));

        let doc1 = ScipDocument {
            relative_path: "src/scip.rs".to_string(),
            language: "rust".to_string(),
            occurrences: vec![
                ScipOccurrence {
                    range: vec![10, 4, 10, 22],
                    symbol: sym.clone(),
                    symbol_roles: SymbolRole::Definition as i32,
                    override_documentation: None,
                },
            ],
            symbols: vec![
                ScipSymbolInformation {
                    symbol: sym.clone(),
                    documentation: vec!["Format symbol helper".to_string()],
                    relationships: vec![],
                    kind: "Function".to_string(),
                    display_name: "format_scip_symbol".to_string(),
                },
            ],
        };

        let doc2 = ScipDocument {
            relative_path: "src/main.rs".to_string(),
            language: "rust".to_string(),
            occurrences: vec![
                ScipOccurrence {
                    range: vec![50, 8, 50, 26],
                    symbol: sym.clone(),
                    symbol_roles: SymbolRole::ReadAccess as i32,
                    override_documentation: None,
                },
            ],
            symbols: vec![],
        };

        index.add_document(doc1);
        index.add_document(doc2);

        let defs = index.find_definitions(&sym);
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].0.relative_path, "src/scip.rs");

        let refs = index.find_references(&sym);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].0.relative_path, "src/main.rs");

        let xref = index.cross_reference_map();
        assert_eq!(xref.get(&sym), Some(&(1, 1)));
    }
}
