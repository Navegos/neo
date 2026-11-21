// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/plugins/cli_bridge.rs
// created: 2026-11-21

//! Dynamic CLI command bridge (`toksi plugin run <cmd> [args...]`).

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

pub type CliPluginHandler = Arc<dyn Fn(&[String]) -> i32 + Send + Sync>;

#[derive(Default)]
pub struct PluginCliBridge {
    commands: RwLock<HashMap<String, (String, CliPluginHandler)>>,
}

impl PluginCliBridge {
    pub fn register(
        &self,
        cmd: impl Into<String>,
        plugin_id: impl Into<String>,
        handler: CliPluginHandler,
    ) {
        let key = cmd.into().trim().to_ascii_lowercase();
        if let Ok(mut g) = self.commands.write() {
            g.insert(key, (plugin_id.into(), handler));
        }
    }

    pub fn unregister_plugin(&self, plugin_id: &str) {
        if let Ok(mut g) = self.commands.write() {
            g.retain(|_, (pid, _)| pid != plugin_id);
        }
    }

    pub fn execute(&self, cmd: &str, args: &[String]) -> Result<i32, String> {
        let key = cmd.trim().to_ascii_lowercase();
        let handler = self
            .commands
            .read()
            .ok()
            .and_then(|g| g.get(&key).map(|(_, h)| Arc::clone(h)))
            .ok_or_else(|| format!("Plugin CLI command '{cmd}' not registered"))?;
        Ok((handler)(args))
    }

    pub fn list_commands(&self) -> Vec<(String, String)> {
        let mut out = self
            .commands
            .read()
            .map(|g| {
                g.iter()
                    .map(|(k, (pid, _))| (k.clone(), pid.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }
}

static GLOBAL_CLI_BRIDGE: OnceLock<Arc<PluginCliBridge>> = OnceLock::new();

pub fn global_plugin_cli_bridge() -> Arc<PluginCliBridge> {
    Arc::clone(GLOBAL_CLI_BRIDGE.get_or_init(|| Arc::new(PluginCliBridge::default())))
}
