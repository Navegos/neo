// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/plugins/slash_router.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! Fast (< 5ms) plugin slash command router.

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use serde::{Deserialize, Serialize};

pub type SlashHandler = Arc<dyn Fn(&str) -> String + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSlashEntry {
    pub command: String,
    pub plugin_id: String,
    pub description: String,
}

#[derive(Clone)]
struct RegisteredSlash {
    entry: PluginSlashEntry,
    handler: SlashHandler,
}

#[derive(Default)]
pub struct PluginSlashRouter {
    commands: RwLock<HashMap<String, RegisteredSlash>>,
}

impl PluginSlashRouter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &self,
        command: impl Into<String>,
        plugin_id: impl Into<String>,
        description: impl Into<String>,
        handler: SlashHandler,
    ) {
        let raw_cmd = command.into();
        let norm = normalize_slash_cmd(&raw_cmd);
        let entry = PluginSlashEntry {
            command: norm.clone(),
            plugin_id: plugin_id.into(),
            description: description.into(),
        };
        if let Ok(mut guard) = self.commands.write() {
            guard.insert(norm, RegisteredSlash { entry, handler });
        }
    }

    pub fn unregister_plugin(&self, plugin_id: &str) {
        if let Ok(mut guard) = self.commands.write() {
            guard.retain(|_, v| v.entry.plugin_id != plugin_id);
        }
    }

    pub fn dispatch(&self, input: &str) -> Option<String> {
        let trimmed = input.trim();
        if !trimmed.starts_with('/') {
            return None;
        }
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let cmd = normalize_slash_cmd(parts.next()?);
        let args = parts.next().unwrap_or("").trim();

        let handler = self
            .commands
            .read()
            .ok()
            .and_then(|g| g.get(&cmd).map(|r| Arc::clone(&r.handler)))?;
        Some((handler)(args))
    }

    pub fn list_entries(&self) -> Vec<PluginSlashEntry> {
        let mut out = self
            .commands
            .read()
            .map(|g| g.values().map(|v| v.entry.clone()).collect::<Vec<_>>())
            .unwrap_or_default();
        out.sort_by(|a, b| a.command.cmp(&b.command));
        out
    }
}

fn normalize_slash_cmd(cmd: &str) -> String {
    let trimmed = cmd.trim().to_ascii_lowercase();
    if trimmed.starts_with('/') {
        trimmed
    } else {
        format!("/{}", trimmed)
    }
}

static GLOBAL_PLUGIN_SLASH_ROUTER: OnceLock<Arc<PluginSlashRouter>> = OnceLock::new();

pub fn global_plugin_slash_router() -> Arc<PluginSlashRouter> {
    Arc::clone(GLOBAL_PLUGIN_SLASH_ROUTER.get_or_init(|| Arc::new(PluginSlashRouter::new())))
}
