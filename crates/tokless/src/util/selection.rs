// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/selection.rs
// created: 2026-10-20
// lastModified: 2026-10-20

use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::util::paths::{ensure_dir, read_file_safe, tokless_data_dir, write_file};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectionStore {
    pub mode: String, // "full" or "selected"
    #[serde(default)]
    pub tools: Vec<String>,
}

pub fn selection_path() -> String {
    Path::new(&tokless_data_dir())
        .join("selection.json")
        .to_string_lossy()
        .to_string()
}

pub fn load_selection() -> Option<SelectionStore> {
    let raw = read_file_safe(&selection_path())?;
    let store: SelectionStore = serde_json::from_str(&raw).ok()?;
    if store.mode != "full" && store.mode != "selected" {
        return None;
    }
    Some(store)
}

pub fn save_selection(mode: &str, tools: &[String]) -> std::io::Result<()> {
    if mode != "full" && mode != "selected" {
        return Ok(());
    }
    let store = SelectionStore {
        mode: mode.to_string(),
        tools: tools.to_vec(),
    };
    let data = serde_json::to_string_pretty(&store).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let dir = tokless_data_dir();
    ensure_dir(&dir)?;
    write_file(&selection_path(), &data)
}

pub fn is_tool_selected(tool_id: &str) -> bool {
    let store = match load_selection() {
        Some(s) => s,
        None => return true,
    };
    if store.mode == "full" {
        return true;
    }
    store.tools.iter().any(|id| id == tool_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_store_serde() {
        let store = SelectionStore {
            mode: "selected".to_string(),
            tools: vec!["contextmode".to_string(), "projectmem".to_string()],
        };
        let serialized = serde_json::to_string(&store).expect("serialize");
        let parsed: SelectionStore = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(store, parsed);
    }

    #[test]
    fn test_invalid_selection_mode() {
        let invalid = r#"{"mode":"unknown","tools":[]}"#;
        let parsed: Result<SelectionStore, _> = serde_json::from_str(invalid);
        assert!(parsed.is_ok());
        // save_selection ignores invalid modes
        assert!(save_selection("invalid", &[]).is_ok());
    }
}
