// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/plugins/analyzer_plugin.rs
// created: 2026-11-21

//! Extensible `AnalyzerPlugin` trait and registry for `skill-scanner` / SAST plugins.

#![deny(unsafe_code)]

use std::sync::{Arc, OnceLock, RwLock};

use crate::security::sarif::UnifiedSecurityFinding;

pub trait AnalyzerPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn analyze(&self, file_path: &str, content: &str) -> Vec<UnifiedSecurityFinding>;
}

#[derive(Default)]
pub struct AnalyzerPluginRegistry {
    analyzers: RwLock<Vec<Arc<dyn AnalyzerPlugin>>>,
}

impl AnalyzerPluginRegistry {
    pub fn register(&self, plugin: Arc<dyn AnalyzerPlugin>) {
        if let Ok(mut g) = self.analyzers.write() {
            g.retain(|p| p.id() != plugin.id());
            g.push(plugin);
        }
    }

    pub fn run_all(&self, file_path: &str, content: &str) -> Vec<UnifiedSecurityFinding> {
        let list = self
            .analyzers
            .read()
            .map(|g| g.clone())
            .unwrap_or_default();
        let mut out = Vec::new();
        for a in list {
            out.extend(a.analyze(file_path, content));
        }
        out
    }

    pub fn ids(&self) -> Vec<String> {
        self.analyzers
            .read()
            .map(|g| g.iter().map(|a| a.id().to_string()).collect())
            .unwrap_or_default()
    }
}

static GLOBAL_ANALYZER_REGISTRY: OnceLock<Arc<AnalyzerPluginRegistry>> = OnceLock::new();

pub fn global_analyzer_plugin_registry() -> Arc<AnalyzerPluginRegistry> {
    Arc::clone(GLOBAL_ANALYZER_REGISTRY.get_or_init(|| Arc::new(AnalyzerPluginRegistry::default())))
}
