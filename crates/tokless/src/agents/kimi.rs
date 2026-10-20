// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/kimi.rs
// created: 2026-10-20
// lastModified: 2026-10-20

use std::path::Path;
use std::sync::Arc;
use regex::Regex;
use url::Url;

use crate::core::AgentManifest;
use crate::util::byok::{byok_route_header, delete_byok_route, read_byok_route, upsert_byok_route, BYOKRoute};
use crate::util::logger::L;
use crate::util::paths::{expected_bin_dirs, kimi_paths_resolved, read_file_safe, write_file, write_file_atomic};
use super::proxy_protocol::proxy_endpoint_for;
use super::proxy_provider::{
    detect_agent_with, is_valid_byok_upstream, load_proxy_route_stash, load_proxy_route_stash_locked,
    normalized_headroom_upstream, proxy_route_stash_path, proxy_route_stash_valid, register_byok_route,
    restore_proxy_route_stash_logged, rollback_byok_route_logged, same_proxy_base, save_proxy_route_stash,
    with_proxy_route_stash_lock, ByokRouteRollback, ProxyRouteStashEntry,
};

pub const KIMI_ROUTE_HEADER: &str = "X-Tokless-Route";
pub const KIMI_HEADROOM_BASE_URL_HEADER: &str = "x-headroom-base-url";
pub const KIMI_HEADROOM_ORIGINAL_PATH_HDR: &str = "x-headroom-original-path";

pub fn kimi_manifest() -> AgentManifest {
    AgentManifest {
        id: "kimi".to_string(),
        label: "Kimi Code".to_string(),
        homepage: "https://github.com/MoonshotAI/kimi-code".to_string(),
        cli_bin: "kimi".to_string(),
        config_dir: Some(Arc::new(|| kimi_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            let mut known = expected_bin_dirs();
            known.push(Path::new(&kimi_paths_resolved().dir).join("bin").to_string_lossy().to_string());
            detect_agent_with("kimi", &kimi_paths_resolved().dir, &known, &[])
        })),
    }
}

#[derive(Debug, Clone)]
struct KimiProviderBlock {
    id: String,
    start: usize,
    end: usize,
}

fn kimi_scan_providers(raw: &str) -> Vec<KimiProviderBlock> {
    let re_hdr = Regex::new(r#"(?m)^[ \t]*\[\s*providers\s*\.\s*(?:"([^"]+)"|'([^']+)'|([A-Za-z0-9_-]+))\s*\](?:[ \t]*#.*)?$"#).unwrap();
    let re_next = Regex::new(r#"(?m)^[ \t]*\["#).unwrap();
    let mut out = Vec::new();

    for caps in re_hdr.captures_iter(raw) {
        let m = caps.get(0).unwrap();
        let id = caps.get(1)
            .or_else(|| caps.get(2))
            .or_else(|| caps.get(3))
            .map(|m| m.as_str())
            .unwrap_or("")
            .to_string();
        let start = m.start();
        let after = &raw[m.end()..];
        let end = match re_next.find(after) {
            Some(nm) => m.end() + nm.start(),
            None => raw.len(),
        };
        out.push(KimiProviderBlock { id, start, end });
    }
    out
}

fn kimi_find_provider(raw: &str, id: &str) -> Option<KimiProviderBlock> {
    kimi_scan_providers(raw).into_iter().find(|p| p.id == id)
}

fn kimi_block_field(block: &str, key: &str) -> String {
    let escaped = regex::escape(key);
    let pattern = format!(r#"(?m)^[ \t]*(?:{}|"{}"|'{}')[ \t]*=[ \t]*(?:"((?:[^"\\]|\\.)*)"|'([^']*)')"#, escaped, escaped, escaped);
    let re = Regex::new(&pattern).unwrap();
    if let Some(caps) = re.captures(block) {
        if let Some(m) = caps.get(1) {
            return m.as_str().replace(r#"\""#, "\"").replace(r#"\\"#, "\\");
        }
        if let Some(m) = caps.get(2) {
            return m.as_str().replace("''", "'");
        }
    }
    String::new()
}

pub fn kimi_provider_protocol(kind: &str) -> String {
    match kind.trim().to_lowercase().as_str() {
        "" | "openai" | "openai-chat" | "openai-completions" | "openai_chat" | "openai_completions" => {
            "openai-completions".to_string()
        }
        "openai_responses" | "openai-responses" => "openai-responses".to_string(),
        _ => String::new(),
    }
}

fn kimi_provider_key(block: &str) -> String {
    let k = kimi_block_field(block, "api_key");
    if !k.is_empty() {
        return k;
    }
    let env = kimi_block_field(block, "api_key_env");
    if !env.is_empty() {
        return std::env::var(env).unwrap_or_default().trim().to_string();
    }
    String::new()
}

#[derive(Debug, Clone)]
pub struct KimiBYOK {
    pub id: String,
    pub file: String,
    pub base_url: String,
    pub api_key: String,
    pub protocol: String,
}

pub fn discover_kimi_byok() -> Vec<KimiBYOK> {
    let path = kimi_paths_resolved().config;
    let raw = match read_file_safe(&path) {
        Some(r) => r,
        None => return Vec::new(),
    };
    let proxy_base = proxy_endpoint_for("kimi").trim_end_matches('/').to_string();
    let gateway_base = crate::util::byok::byok_gateway_endpoint().trim_end_matches('/').to_string();

    struct Acc {
        base: String,
        key: String,
        proto: String,
    }

    let mut got: std::collections::BTreeMap<String, Acc> = std::collections::BTreeMap::new();
    let mut order = Vec::new();

    for p in kimi_scan_providers(&raw) {
        let block = &raw[p.start..p.end];
        let proto = kimi_provider_protocol(&kimi_block_field(block, "type"));
        if proto.is_empty() {
            continue;
        }
        let base = kimi_block_field(block, "base_url");
        let key = kimi_provider_key(block);
        if base.is_empty() && key.is_empty() {
            continue;
        }
        if !got.contains_key(&p.id) {
            got.insert(p.id.clone(), Acc { base: String::new(), key: String::new(), proto: proto.clone() });
            order.push(p.id.clone());
        }
        let a = got.get_mut(&p.id).unwrap();
        if !key.is_empty() {
            a.key = key;
        }
        if is_valid_byok_upstream(&base) && !same_proxy_base(&base, &proxy_base) && !same_proxy_base(&base, &gateway_base) {
            a.base = base;
        }
    }

    let stashed = load_proxy_route_stash("kimi");
    let mut out = Vec::new();
    for id in order {
        let a = got.get_mut(&id).unwrap();
        if a.base.is_empty() {
            if let Some(s) = stashed.get(&id) {
                if !s.base_url.is_empty() {
                    a.base = s.base_url.clone();
                }
            }
        }
        if a.base.is_empty() || a.key.is_empty() || a.proto.is_empty() {
            continue;
        }
        out.push(KimiBYOK {
            id,
            file: path.clone(),
            base_url: a.base.clone(),
            api_key: a.key.clone(),
            protocol: a.proto.clone(),
        });
    }
    out
}

pub fn kimi_headroom_upstream(base_url: &str) -> Option<(String, String)> {
    let u = Url::parse(base_url.trim()).ok()?;
    let host = u.host_str()?;
    let scheme = u.scheme();
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let port_str = u.port().map(|p| format!(":{}", p)).unwrap_or_default();
    let origin = format!("{}://{}{}", scheme, host, port_str);
    let path = u.path().trim_end_matches('/');
    let chat_path = format!("{}/chat/completions", path);
    Some((origin, chat_path))
}

pub fn kimi_headroom_base(base: &str) -> bool {
    same_proxy_base(base, &proxy_endpoint_for("kimi"))
}

pub fn kimi_gateway_base(base: &str) -> bool {
    let g = crate::util::byok::byok_gateway_endpoint();
    same_proxy_base(base, &g) || same_proxy_base(base, &format!("{}/v1", g))
}

pub fn kimi_proxy_base(base: &str) -> bool {
    kimi_headroom_base(base) || kimi_gateway_base(base)
}

fn toml_quoted(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn kimi_set_base_url(raw: &str, id: &str, base: &str) -> (String, bool) {
    let p = match kimi_find_provider(raw, id) {
        Some(p) => p,
        None => return (raw.to_string(), false),
    };
    let block = &raw[p.start..p.end];
    let re = Regex::new(r#"(?m)^([ \t]*(?:base_url|"base_url"|'base_url')[ \t]*=[ \t]*)(?:"(?:[^"\\]|\\.)*"|'[^']*')"#).unwrap();
    if let Some(m) = re.captures(block) {
        let whole = m.get(0).unwrap();
        let prefix = m.get(1).unwrap().as_str();
        let new_block = format!("{}{}{}{}", &block[..whole.start()], prefix, toml_quoted(base), &block[whole.end()..]);
        let next = format!("{}{}{}", &raw[..p.start], new_block, &raw[p.end..]);
        return (next, true);
    }
    (raw.to_string(), false)
}

fn kimi_sub_custom_re(id: &str) -> Regex {
    let q = regex::escape(id);
    Regex::new(&format!(r#"(?m)^[ \t]*\[\s*providers\s*\.\s*(?:"{}"|'{}'|{})\s*\.\s*custom_headers\s*\](?:[ \t]*#.*)?$"#, q, q, q)).unwrap()
}

fn kimi_header_line_re(key: &str) -> Regex {
    let q = regex::escape(key);
    Regex::new(&format!(r#"(?m)^([ \t]*)(?:{}|"{}"|'{}')[ \t]*=[ \t]*(?:"((?:[^"\\]|\\.)*)"|'([^']*)')"#, q, q, q)).unwrap()
}

pub fn kimi_custom_header_value(raw: &str, id: &str, key: &str) -> Option<String> {
    let re_inline = Regex::new(r#"(?m)^([ \t]*)custom_headers[ \t]*=[ \t]*\{([^}]*)\}[ \t]*(?:#.*)?$"#).unwrap();
    let re_pair = Regex::new(r#"(?:"((?:[^"\\]|\\.)*)"|'([^']*)')\s*=\s*(?:"((?:[^"\\]|\\.)*)"|'([^']*)')"#).unwrap();

    if let Some(p) = kimi_find_provider(raw, id) {
        let block = &raw[p.start..p.end];
        if let Some(caps) = re_inline.captures(block) {
            let inner = caps.get(2).unwrap().as_str();
            for m in re_pair.captures_iter(inner) {
                let k = m.get(1).or_else(|| m.get(2)).unwrap().as_str();
                let v = m.get(3).or_else(|| m.get(4)).unwrap().as_str();
                if k == key {
                    return Some(v.to_string());
                }
            }
        }
    }

    let re_sub = kimi_sub_custom_re(id);
    let loc = re_sub.find(raw)?;
    let re_next = Regex::new(r#"(?m)^[ \t]*\["#).unwrap();
    let after = &raw[loc.end()..];
    let end = match re_next.find(after) {
        Some(nm) => loc.end() + nm.start(),
        None => raw.len(),
    };
    let sub = &raw[loc.start()..end];
    let line_re = kimi_header_line_re(key);
    let caps = line_re.captures(sub)?;
    caps.get(2).or_else(|| caps.get(3)).map(|m| m.as_str().to_string())
}

pub fn kimi_provider_header_value(raw: &str, id: &str) -> (String, bool) {
    match kimi_custom_header_value(raw, id, KIMI_ROUTE_HEADER) {
        Some(v) => (v, true),
        None => (String::new(), false),
    }
}

pub fn kimi_set_custom_header(raw: &str, id: &str, key: &str, value: &str) -> (String, bool) {
    let p = match kimi_find_provider(raw, id) {
        Some(p) => p,
        None => return (raw.to_string(), false),
    };
    let block = &raw[p.start..p.end];
    let re_inline = Regex::new(r#"(?m)^([ \t]*)custom_headers[ \t]*=[ \t]*\{([^}]*)\}[ \t]*(?:#.*)?$"#).unwrap();
    if let Some(caps) = re_inline.captures(block) {
        let whole = caps.get(0).unwrap();
        let indent = caps.get(1).unwrap().as_str();
        let inner = caps.get(2).unwrap().as_str();
        let re_pair = Regex::new(r#"(?:"((?:[^"\\]|\\.)*)"|'([^']*)')\s*=\s*(?:"((?:[^"\\]|\\.)*)"|'([^']*)')"#).unwrap();
        let mut pairs: Vec<(String, String)> = Vec::new();
        let mut found = false;
        for m in re_pair.captures_iter(inner) {
            let k = m.get(1).or_else(|| m.get(2)).unwrap().as_str().to_string();
            let v = m.get(3).or_else(|| m.get(4)).unwrap().as_str().to_string();
            if k == key {
                pairs.push((k, value.to_string()));
                found = true;
            } else {
                pairs.push((k, v));
            }
        }
        if !found {
            pairs.push((key.to_string(), value.to_string()));
        }
        let rendered: Vec<String> = pairs.iter().map(|(k, v)| format!("\"{}\" = {}", k, toml_quoted(v))).collect();
        let line = format!("{}custom_headers = {{ {} }}", indent, rendered.join(", "));
        let new_block = format!("{}{}{}", &block[..whole.start()], line, &block[whole.end()..]);
        return (format!("{}{}{}", &raw[..p.start], new_block, &raw[p.end..]), true);
    }

    let re_sub = kimi_sub_custom_re(id);
    if let Some(m) = re_sub.find(raw) {
        let re_next = Regex::new(r#"(?m)^[ \t]*\["#).unwrap();
        let after = &raw[m.end()..];
        let end = match re_next.find(after) {
            Some(nm) => m.end() + nm.start(),
            None => raw.len(),
        };
        let sub = &raw[m.start()..end];
        let line_re = kimi_header_line_re(key);
        let new_sub = if let Some(lm) = line_re.find(sub) {
            let prefix = line_re.captures(sub).unwrap().get(1).unwrap().as_str();
            format!("{}{}{} = {}{}", &sub[..lm.start()], prefix, key, toml_quoted(value), &sub[lm.end()..])
        } else {
            format!("{}\n{} = {}", sub.trim_end(), key, toml_quoted(value))
        };
        return (format!("{}{}{}", &raw[..m.start()], new_sub, &raw[end..]), true);
    }

    let sub = format!("\n[providers.{}.custom_headers]\n{} = {}\n", toml_quoted(id), key, toml_quoted(value));
    (format!("{}{}{}", &raw[..p.end], sub, &raw[p.end..]), true)
}

fn kimi_drop_custom_header(raw: &str, id: &str, key: &str) -> (String, bool) {
    let p = match kimi_find_provider(raw, id) {
        Some(p) => p,
        None => return (raw.to_string(), false),
    };
    let block = &raw[p.start..p.end];
    let re_inline = Regex::new(r#"(?m)^([ \t]*)custom_headers[ \t]*=[ \t]*\{([^}]*)\}[ \t]*(?:#.*)?$"#).unwrap();
    if let Some(caps) = re_inline.captures(block) {
        let whole = caps.get(0).unwrap();
        let indent = caps.get(1).unwrap().as_str();
        let inner = caps.get(2).unwrap().as_str();
        let re_pair = Regex::new(r#"(?:"((?:[^"\\]|\\.)*)"|'([^']*)')\s*=\s*(?:"((?:[^"\\]|\\.)*)"|'([^']*)')"#).unwrap();
        let mut pairs: Vec<(String, String)> = Vec::new();
        let mut found = false;
        for m in re_pair.captures_iter(inner) {
            let k = m.get(1).or_else(|| m.get(2)).unwrap().as_str().to_string();
            let v = m.get(3).or_else(|| m.get(4)).unwrap().as_str().to_string();
            if k == key {
                found = true;
            } else {
                pairs.push((k, v));
            }
        }
        if !found {
            return (raw.to_string(), false);
        }
        let line = if pairs.is_empty() {
            String::new()
        } else {
            let rendered: Vec<String> = pairs.iter().map(|(k, v)| format!("\"{}\" = {}", k, toml_quoted(v))).collect();
            format!("{}custom_headers = {{ {} }}", indent, rendered.join(", "))
        };
        let new_block = format!("{}{}{}", &block[..whole.start()], line, &block[whole.end()..]);
        return (format!("{}{}{}", &raw[..p.start], new_block, &raw[p.end..]), true);
    }

    let re_sub = kimi_sub_custom_re(id);
    if let Some(m) = re_sub.find(raw) {
        let re_next = Regex::new(r#"(?m)^[ \t]*\["#).unwrap();
        let after = &raw[m.end()..];
        let end = match re_next.find(after) {
            Some(nm) => m.end() + nm.start(),
            None => raw.len(),
        };
        let sub = &raw[m.start()..end];
        let line_re = kimi_header_line_re(key);
        if let Some(lm) = line_re.find(sub) {
            let mut line_end = lm.end();
            while line_end < sub.len() && sub.as_bytes()[line_end] == b'\n' {
                line_end += 1;
            }
            let new_sub = format!("{}{}", &sub[..lm.start()], &sub[line_end..]);
            if let Some(nl) = new_sub.find('\n') {
                if new_sub[nl + 1..].trim().is_empty() {
                    let mut start = m.start();
                    if start > 0 && raw.as_bytes()[start - 1] == b'\n' {
                        start -= 1;
                    }
                    return (format!("{}{}", &raw[..start], &raw[end..]), true);
                }
            }
            return (format!("{}{}{}", &raw[..m.start()], new_sub, &raw[end..]), true);
        }
    }
    (raw.to_string(), false)
}

fn kimi_remove_header(raw: &str, id: &str, had: bool, header: &str) -> (String, bool) {
    let mut next = raw.to_string();
    for key in [KIMI_HEADROOM_BASE_URL_HEADER, KIMI_HEADROOM_ORIGINAL_PATH_HDR] {
        let (removed, ok) = kimi_drop_custom_header(&next, id, key);
        if ok {
            next = removed;
        }
    }
    if had {
        kimi_set_custom_header(&next, id, KIMI_ROUTE_HEADER, header)
    } else {
        kimi_drop_custom_header(&next, id, KIMI_ROUTE_HEADER)
    }
}

pub fn configure_kimi_proxy() -> (bool, String) {
    let mut changed = false;
    let mut file = kimi_paths_resolved().config;
    if let Err(err) = with_proxy_route_stash_lock(|| {
        let (c, f) = configure_kimi_proxy_locked();
        changed = c;
        file = f;
        Ok(())
    }) {
        L.err(&format!("kimi proxy lock failed: {err}"));
    }
    (changed, file)
}

pub fn configure_kimi_proxy_locked() -> (bool, String) {
    let file = kimi_paths_resolved().config;
    if !proxy_route_stash_valid("kimi") {
        return (false, file);
    }
    let raw = match read_file_safe(&file) {
        Some(r) => r,
        None => return (false, file),
    };
    let providers = discover_kimi_byok();
    if providers.is_empty() {
        return (false, file);
    }

    let mut stash = load_proxy_route_stash_locked("kimi");
    let stash_raw = read_file_safe(&proxy_route_stash_path("kimi"));
    let stash_exists = stash_raw.is_some();
    let mut route_changes: Vec<ByokRouteRollback> = Vec::new();

    let rollback = |route_changes: &[ByokRouteRollback]| {
        for rc in route_changes.iter().rev() {
            rollback_byok_route_logged("kimi", &rc.id, &rc.previous);
        }
        restore_proxy_route_stash_logged("kimi", stash_raw.as_deref().unwrap_or(""), stash_exists);
    };

    let mut next = raw;
    let mut changed_any = false;
    let headroom_base = proxy_endpoint_for("kimi");

    for provider in providers {
        if !is_valid_byok_upstream(&provider.base_url) {
            continue;
        }
        let (origin, chat_path) = match kimi_headroom_upstream(&provider.base_url) {
            Some(res) => res,
            None => {
                L.warn(&format!("kimi: skipping provider with unparseable base_url: {}", provider.id));
                continue;
            }
        };
        let upstream = normalized_headroom_upstream(&provider.base_url, &provider.protocol);
        let (_, route_header, previous, route_ok) = register_byok_route("kimi", &provider.id, &provider.protocol, &upstream);
        if !route_ok {
            rollback(&route_changes);
            return (false, file);
        }
        route_changes.push(ByokRouteRollback { id: provider.id.clone(), previous });

        let (header, had_header) = kimi_provider_header_value(&next, &provider.id);
        let (updated, ok) = kimi_set_base_url(&next, &provider.id, &headroom_base);
        if !ok {
            rollback(&route_changes);
            return (false, file);
        }
        let (updated, ok) = kimi_set_custom_header(&updated, &provider.id, KIMI_ROUTE_HEADER, &route_header);
        if !ok {
            rollback(&route_changes);
            return (false, file);
        }
        let (updated, ok) = kimi_set_custom_header(&updated, &provider.id, KIMI_HEADROOM_BASE_URL_HEADER, &origin);
        if !ok {
            rollback(&route_changes);
            return (false, file);
        }
        let (updated, ok) = kimi_set_custom_header(&updated, &provider.id, KIMI_HEADROOM_ORIGINAL_PATH_HDR, &chat_path);
        if !ok {
            rollback(&route_changes);
            return (false, file);
        }

        if updated != next {
            changed_any = true;
        }
        next = updated;

        stash.insert(provider.id.clone(), ProxyRouteStashEntry {
            file: file.clone(),
            provider: provider.id.clone(),
            base_url: provider.base_url.clone(),
            upstream,
            header,
            had_header,
            ..Default::default()
        });
    }

    if !changed_any {
        rollback(&route_changes);
        return (false, file);
    }
    if save_proxy_route_stash("kimi", &stash).is_err() {
        rollback(&route_changes);
        return (false, file);
    }
    if write_file_atomic(&file, &next).is_err() {
        rollback(&route_changes);
        return (false, file);
    }

    (true, file)
}

pub fn remove_kimi_proxy() -> bool {
    let mut removed = false;
    if let Err(err) = with_proxy_route_stash_lock(|| {
        removed = remove_kimi_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("kimi proxy lock failed: {err}"));
    }
    removed
}

pub fn remove_kimi_proxy_locked() -> bool {
    if !proxy_route_stash_valid("kimi") {
        return false;
    }
    let file = kimi_paths_resolved().config;
    let raw = match read_file_safe(&file) {
        Some(r) => r,
        None => return false,
    };
    let stash = load_proxy_route_stash_locked("kimi");
    if stash.is_empty() {
        return false;
    }

    let mut next = raw.clone();
    let mut remaining = std::collections::HashMap::new();
    let mut deleted_routes: Vec<BYOKRoute> = Vec::new();
    let mut deleted_ids: Vec<String> = Vec::new();

    for (id, entry) in &stash {
        let p = match kimi_find_provider(&next, id) {
            Some(p) => p,
            None => {
                remaining.insert(id.clone(), entry.clone());
                continue;
            }
        };
        let block = &next[p.start..p.end];
        let base = kimi_block_field(block, "base_url");
        let route = read_byok_route(&format!("kimi:{}", id));
        let (value, has) = kimi_provider_header_value(&next, id);
        let matches = route.is_some() && kimi_proxy_base(&base) && has && route.as_ref().map(|r| value == byok_route_header(r)).unwrap_or(false);

        if !matches {
            remaining.insert(id.clone(), entry.clone());
            continue;
        }

        let (mut updated, ok) = kimi_remove_header(&next, id, entry.had_header, &entry.header);
        if !ok {
            updated = next.clone();
        }
        let (updated, ok) = kimi_set_base_url(&updated, id, &entry.base_url);
        if !ok {
            remaining.insert(id.clone(), entry.clone());
            continue;
        }

        next = updated;
        if let Some(r) = route {
            deleted_routes.push(r);
        }
        deleted_ids.push(id.clone());
    }

    if deleted_ids.is_empty() {
        return false;
    }
    if write_file_atomic(&file, &next).is_err() {
        return false;
    }
    for id in &deleted_ids {
        if delete_byok_route(&format!("kimi:{}", id)).is_err() {
            let _ = write_file(&file, &raw);
            return false;
        }
    }
    if save_proxy_route_stash("kimi", &remaining).is_err() {
        let _ = write_file(&file, &raw);
        for r in deleted_routes {
            let _ = upsert_byok_route(&r.id, &r.protocol, &r.upstream);
        }
        return false;
    }

    true
}

pub fn kimi_proxy_wired() -> bool {
    let stash = load_proxy_route_stash("kimi");
    if stash.is_empty() {
        return false;
    }
    let raw = match read_file_safe(&kimi_paths_resolved().config) {
        Some(r) => r,
        None => return false,
    };
    for (id, entry) in &stash {
        let p = match kimi_find_provider(&raw, id) {
            Some(p) => p,
            None => return false,
        };
        let block = &raw[p.start..p.end];
        if !kimi_headroom_base(&kimi_block_field(block, "base_url")) {
            return false;
        }
        let route = match read_byok_route(&format!("kimi:{}", id)) {
            Some(r) => r,
            None => return false,
        };
        let (value, has) = kimi_provider_header_value(&raw, id);
        if !has || value != byok_route_header(&route) {
            return false;
        }
        let (origin, chat_path) = match kimi_headroom_upstream(&entry.base_url) {
            Some(res) => res,
            None => return false,
        };
        if kimi_custom_header_value(&raw, id, KIMI_HEADROOM_BASE_URL_HEADER).as_deref() != Some(&origin) {
            return false;
        }
        if kimi_custom_header_value(&raw, id, KIMI_HEADROOM_ORIGINAL_PATH_HDR).as_deref() != Some(&chat_path) {
            return false;
        }
    }
    true
}
