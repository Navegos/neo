// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/byok_providers.rs
// created: 2026-10-20
// lastModified: 2026-10-20

use std::collections::{BTreeMap, HashSet};
use std::fs;
use regex::Regex;

use crate::util::byok::byok_routes_for;
use super::proxy_agent::{proxy_agent_config_files, proxy_spec_for};
use super::proxy_provider::load_proxy_route_stash;

pub fn strip_urls(s: &str) -> String {
    let re = Regex::new(r#"[a-zA-Z][a-zA-Z0-9+.-]*://[^\s"']*"#).unwrap();
    re.replace_all(s, " ").into_owned()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BYOKProvider {
    pub provider: String,
    pub upstream: String,
    pub file: String,
    pub stale: bool,
}

pub fn byok_providers_for(id: &str) -> Vec<BYOKProvider> {
    if proxy_spec_for(id).is_none() {
        return Vec::new();
    }
    let stash = load_proxy_route_stash(id);
    let mut routes = BTreeMap::new();
    let prefix = format!("{}:", id);
    for route in byok_routes_for(id) {
        if route.id.starts_with(&prefix) {
            routes.insert(route.id[prefix.len()..].to_string(), route);
        }
    }

    let mut keys = HashSet::new();
    for k in stash.keys() {
        keys.insert(k.clone());
    }
    for k in routes.keys() {
        keys.insert(k.clone());
    }

    let files = proxy_agent_config_files(id);
    let mut texts = BTreeMap::new();
    for f in &files {
        if let Ok(raw) = fs::read_to_string(f) {
            texts.insert(f.clone(), raw);
        }
    }

    let mut providers = Vec::with_capacity(keys.len());
    for key in keys {
        let mut provider = BYOKProvider {
            provider: key.clone(),
            upstream: String::new(),
            file: String::new(),
            stale: false,
        };
        let entry = stash.get(&key);
        let route = routes.get(&key);

        if let Some(e) = entry {
            if !e.base_url.is_empty() {
                provider.upstream = e.base_url.clone();
            } else if !e.upstream.is_empty() {
                provider.upstream = e.upstream.clone();
            } else if let Some(r) = route {
                provider.upstream = r.upstream.clone();
            }
        } else if let Some(r) = route {
            provider.upstream = r.upstream.clone();
        }

        for file in &files {
            if let Some(content) = texts.get(file) {
                if strip_urls(content).contains(&key) {
                    provider.file = file.clone();
                    break;
                }
            }
        }

        provider.stale = provider.file.is_empty();
        providers.push(provider);
    }

    providers.sort_by(|a, b| a.provider.cmp(&b.provider));
    providers
}

