// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/agent_cli/cursor.rs
// created: 2026-10-31
// lastModified: 2026-10-31

pub fn read_cursor_session() -> Option<String> {
    let config_dir = dirs::config_dir()?;
    let path = config_dir.join("Cursor").join("User").join("globalStorage").join("state.vscdb");
    if path.exists() {
        return Some("cursor-local-bridge-active".to_string());
    }
    None
}
