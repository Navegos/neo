// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/zcode.rs
// created: 2026-10-20
// lastModified: 2026-10-20

use std::path::Path;
use std::sync::Arc;
use serde_json::Value;
use url::Url;

use crate::core::AgentManifest;
use crate::util::byok::{byok_route_header, delete_byok_route, read_byok_route, upsert_byok_route, BYOKRoute};
use crate::util::jsonc::{has_jsonc_comments, stringify_json, OrderedMap};
use crate::util::logger::L;
use crate::util::paths::{expected_bin_dirs, home, read_file_safe, write_file, write_file_atomic, zcode_paths_resolved};
use super::proxy_protocol::proxy_endpoint_for;
use super::proxy_provider::{
    byok_route_header_key, detect_agent_with, is_valid_byok_upstream, load_proxy_route_stash,
    load_proxy_route_stash_locked, normalized_headroom_upstream, proxy_route_stash_path,
    proxy_route_stash_valid, register_byok_route, restore_proxy_route_stash_logged,
    rollback_byok_route_logged, route_upstream_to_base_url, same_proxy_base, save_proxy_route_stash,
    try_parse_jsonc_map as try_parse_jsonc, with_proxy_route_stash_lock, ByokRouteRollback,
    ProxyRouteStashEntry,
};

pub const ZCODE_HEADROOM_BASE_URL_HEADER: &str = "x-headroom-base-url";
pub const ZCODE_HEADROOM_ORIGINAL_PATH_HDR: &str = "x-headroom-original-path";

pub fn zcode_manifest() -> AgentManifest {
    AgentManifest {
        id: "zcode".to_string(),
        label: "ZCode".to_string(),
        homepage: "https://github.com/zai-org/ZCode".to_string(),
        cli_bin: "zcode".to_string(),
        config_dir: Some(Arc::new(|| zcode_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            detect_agent_with("zcode", &zcode_paths_resolved().dir, &expected_bin_dirs(), &[])
        })),
    }
}

pub fn zcode_provider_config_path() -> String {
    Path::new(&home())
        .join(".zcode")
        .join("v2")
        .join("provider_config.json")
        .to_string_lossy()
        .to_string()
}

pub fn zcode_provider_protocol(api_type: &str) -> String {
    match api_type.trim().to_lowercase().as_str() {
        "" | "openai-completions" | "openai-chat-completions" | "openai_chat_completions" => {
            "openai-completions".to_string()
        }
        "openai-responses" | "openai_responses" => "openai-responses".to_string(),
        "anthropic-messages" | "anthropic_messages" => "anthropic-messages".to_string(),
        _ => String::new(),
    }
}

fn zcode_string_field(m: &OrderedMap, key: &str) -> String {
    m.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn zcode_base_url(m: &OrderedMap) -> String {
    let s = zcode_string_field(m, "baseURL");
    if !s.is_empty() {
        return s;
    }
    zcode_string_field(m, "baseUrl")
}

pub fn zcode_headroom_base(base: &str) -> bool {
    same_proxy_base(base, &proxy_endpoint_for("zcode"))
}

pub fn zcode_gateway_base(base: &str) -> bool {
    let g = crate::util::byok::byok_gateway_endpoint();
    same_proxy_base(base, &g) || same_proxy_base(base, &format!("{}/v1", g))
}

pub fn zcode_proxy_base(base: &str) -> bool {
    zcode_headroom_base(base) || zcode_gateway_base(base)
}

pub fn zcode_headroom_upstream(base_url: &str) -> Option<(String, String)> {
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

fn zcode_headers<'a>(m: &'a OrderedMap) -> (Option<&'a OrderedMap>, bool, String, bool) {
    let Some(v) = m.get("headers") else {
        return (None, false, String::new(), true);
    };
    let Some(hm) = v.as_object() else {
        return (None, false, String::new(), false);
    };
    let Some(rv) = hm.get(byok_route_header_key()) else {
        return (Some(hm), false, String::new(), true);
    };
    let Some(s) = rv.as_str() else {
        return (None, false, String::new(), false);
    };
    (Some(hm), true, s.to_string(), true)
}

#[derive(Debug, Clone)]
pub struct ZcodeBYOK {
    pub id: String,
    pub file: String,
    pub base_url: String,
    pub api_key: String,
    pub protocol: String,
}

pub fn discover_zcode_byok() -> Vec<ZcodeBYOK> {
    let path = zcode_provider_config_path();
    let raw = match read_file_safe(&path) {
        Some(r) => r,
        None => return Vec::new(),
    };
    if has_jsonc_comments(&raw) {
        return Vec::new();
    }
    let cfg = match try_parse_jsonc(&raw) {
        Some(c) => c,
        None => return Vec::new(),
    };

    let proxy_base = proxy_endpoint_for("zcode").trim_end_matches('/').to_string();
    let gateway_base = crate::util::byok::byok_gateway_endpoint().trim_end_matches('/').to_string();
    let stashed = load_proxy_route_stash("zcode");
    let mut out = Vec::new();

    let rules = cfg.get("config")
        .and_then(|c| c.get("providerConfigRules"))
        .and_then(|r| r.get("providerRules"))
        .and_then(|pr| pr.as_array());

    let Some(rules_arr) = rules else {
        return Vec::new();
    };

    for entry in rules_arr {
        let Some(m) = entry.as_object() else { continue };
        let id = zcode_string_field(m, "providerId");
        if id.is_empty() {
            continue;
        }
        let config = match m.get("config").and_then(|c| c.as_object()) {
            Some(c) => c,
            None => continue,
        };
        let api = match config.get("api").and_then(|a| a.as_object()) {
            Some(a) => a,
            None => continue,
        };
        let proto = zcode_provider_protocol(&zcode_string_field(api, "type"));
        if proto.is_empty() {
            continue;
        }

        let mut base = zcode_base_url(api);
        if is_valid_byok_upstream(&base) && !same_proxy_base(&base, &proxy_base) && !same_proxy_base(&base, &gateway_base) {
            // keep on-disk original
        } else if let Some(s) = stashed.get(&id) {
            if !s.base_url.is_empty() {
                base = s.base_url.clone();
            }
        } else if let Some((_, _, value, _)) = Some(zcode_headers(api)) {
            if let Some(route_id) = zcode_route_id_from_header(&value) {
                if let Some(route) = read_byok_route(&route_id) {
                    if is_valid_byok_upstream(&route.upstream) {
                        base = route_upstream_to_base_url(&route.upstream, &proto);
                    } else {
                        base = String::new();
                    }
                } else {
                    base = String::new();
                }
            } else {
                base = String::new();
            }
        } else {
            base = String::new();
        }

        if base.is_empty() {
            continue;
        }

        let api_key = config.get("access")
            .and_then(|a| a.as_object())
            .map(|a| zcode_string_field(a, "apiKey"))
            .unwrap_or_default();

        out.push(ZcodeBYOK {
            id,
            file: path.clone(),
            base_url: base,
            api_key,
            protocol: proto,
        });
    }

    out
}

fn zcode_route_id_from_header(header: &str) -> Option<String> {
    let (prefix, token) = header.split_once('.')?;
    if !prefix.starts_with("zcode:") || token.is_empty() {
        return None;
    }
    let route = read_byok_route(prefix)?;
    if route.token != token {
        return None;
    }
    Some(prefix.to_string())
}

pub fn configure_zcode_proxy() -> (bool, String) {
    let mut changed = false;
    let mut file = zcode_provider_config_path();
    if let Err(err) = with_proxy_route_stash_lock(|| {
        let (c, f) = configure_zcode_proxy_locked();
        changed = c;
        file = f;
        Ok(())
    }) {
        L.err(&format!("zcode proxy lock failed: {err}"));
    }
    (changed, file)
}

pub fn configure_zcode_proxy_locked() -> (bool, String) {
    let file = zcode_provider_config_path();
    if !proxy_route_stash_valid("zcode") {
        return (false, file);
    }
    let raw = match read_file_safe(&file) {
        Some(r) => r,
        None => return (false, file),
    };
    if has_jsonc_comments(&raw) {
        return (false, file);
    }
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(c) => c,
        None => return (false, file),
    };
    let discovered = discover_zcode_byok();
    if discovered.is_empty() {
        return (false, file);
    }

    let mut stash = load_proxy_route_stash_locked("zcode");
    let stash_raw = read_file_safe(&proxy_route_stash_path("zcode"));
    let stash_exists = stash_raw.is_some();
    let mut route_changes: Vec<ByokRouteRollback> = Vec::new();

    let rollback = |route_changes: &[ByokRouteRollback]| {
        for rc in route_changes.iter().rev() {
            rollback_byok_route_logged("zcode", &rc.id, &rc.previous);
        }
        restore_proxy_route_stash_logged("zcode", stash_raw.as_deref().unwrap_or(""), stash_exists);
    };

    let headroom_base = proxy_endpoint_for("zcode");
    let mut changed_any = false;

    // Mutate cfg rules in place
    let Some(Value::Object(config_map)) = cfg.get_mut("config") else {
        return (false, file);
    };
    let Some(Value::Object(pcr_map)) = config_map.get_mut("providerConfigRules") else {
        return (false, file);
    };
    let Some(Value::Array(rules_vec)) = pcr_map.get_mut("providerRules") else {
        return (false, file);
    };

    for provider in discovered {
        let rule_opt = rules_vec.iter_mut().find(|r| {
            r.as_object().map(|obj| zcode_string_field(obj, "providerId") == provider.id).unwrap_or(false)
        });
        let Some(rule_val) = rule_opt else { continue };
        let Some(rule_obj) = rule_val.as_object_mut() else { continue };
        let Some(c_obj) = rule_obj.get_mut("config").and_then(|c| c.as_object_mut()) else { continue };
        let Some(api_obj) = c_obj.get_mut("api").and_then(|a| a.as_object_mut()) else { continue };

        let (origin, chat_path) = match zcode_headroom_upstream(&provider.base_url) {
            Some(res) => res,
            None => {
                L.warn(&format!("zcode: skipping provider with unparseable baseUrl: {}", provider.id));
                continue;
            }
        };

        let upstream = normalized_headroom_upstream(&provider.base_url, &provider.protocol);
        let (_, route_header, previous, route_ok) = register_byok_route("zcode", &provider.id, &provider.protocol, &upstream);
        if !route_ok {
            rollback(&route_changes);
            return (false, file);
        }
        route_changes.push(ByokRouteRollback { id: provider.id.clone(), previous });

        let (had_header, header_value) = {
            let (_, had, val, ok) = zcode_headers(api_obj);
            if !ok {
                rollback(&route_changes);
                return (false, file);
            }
            (had, val)
        };

        if zcode_base_url(api_obj) != headroom_base {
            if api_obj.contains_key("baseUrl") {
                api_obj.insert("baseUrl".to_string(), Value::String(headroom_base.clone()));
                api_obj.remove("baseURL");
            } else {
                api_obj.insert("baseURL".to_string(), Value::String(headroom_base.clone()));
            }
            changed_any = true;
        }

        if !api_obj.contains_key("headers") {
            api_obj.insert("headers".to_string(), Value::Object(OrderedMap::new()));
        }
        let headers_map = api_obj.get_mut("headers").unwrap().as_object_mut().unwrap();

        let byok_key = byok_route_header_key();
        if headers_map.get(byok_key).and_then(|v| v.as_str()) != Some(&route_header) {
            headers_map.insert(byok_key.to_string(), Value::String(route_header));
            changed_any = true;
        }
        if headers_map.get(ZCODE_HEADROOM_BASE_URL_HEADER).and_then(|v| v.as_str()) != Some(&origin) {
            headers_map.insert(ZCODE_HEADROOM_BASE_URL_HEADER.to_string(), Value::String(origin));
            changed_any = true;
        }
        if headers_map.get(ZCODE_HEADROOM_ORIGINAL_PATH_HDR).and_then(|v| v.as_str()) != Some(&chat_path) {
            headers_map.insert(ZCODE_HEADROOM_ORIGINAL_PATH_HDR.to_string(), Value::String(chat_path));
            changed_any = true;
        }

        stash.insert(provider.id.clone(), ProxyRouteStashEntry {
            file: file.clone(),
            provider: provider.id.clone(),
            base_url: provider.base_url.clone(),
            upstream,
            header: header_value,
            had_header,
            ..Default::default()
        });
    }

    if !changed_any {
        rollback(&route_changes);
        return (false, file);
    }
    if save_proxy_route_stash("zcode", &stash).is_err() {
        rollback(&route_changes);
        return (false, file);
    }
    let content = stringify_json(&Value::Object(cfg));
    if write_file_atomic(&file, &content).is_err() {
        rollback(&route_changes);
        return (false, file);
    }

    (true, file)
}

pub fn remove_zcode_proxy() -> bool {
    let mut removed = false;
    if let Err(err) = with_proxy_route_stash_lock(|| {
        removed = remove_zcode_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("zcode proxy lock failed: {err}"));
    }
    removed
}

pub fn remove_zcode_proxy_locked() -> bool {
    if !proxy_route_stash_valid("zcode") {
        return false;
    }
    let file = zcode_provider_config_path();
    let raw = match read_file_safe(&file) {
        Some(r) => r,
        None => return false,
    };
    if has_jsonc_comments(&raw) {
        return false;
    }
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(c) => c,
        None => return false,
    };
    let stash = load_proxy_route_stash_locked("zcode");
    if stash.is_empty() {
        return false;
    }

    let mut remaining = std::collections::HashMap::new();
    let mut deleted_routes: Vec<BYOKRoute> = Vec::new();
    let mut deleted_ids: Vec<String> = Vec::new();

    let Some(Value::Object(config_map)) = cfg.get_mut("config") else {
        return false;
    };
    let Some(Value::Object(pcr_map)) = config_map.get_mut("providerConfigRules") else {
        return false;
    };
    let Some(Value::Array(rules_vec)) = pcr_map.get_mut("providerRules") else {
        return false;
    };

    for (id, entry) in &stash {
        let rule_opt = rules_vec.iter_mut().find(|r| {
            r.as_object().map(|obj| zcode_string_field(obj, "providerId") == *id).unwrap_or(false)
        });
        let Some(rule_val) = rule_opt else {
            remaining.insert(id.clone(), entry.clone());
            continue;
        };
        let Some(rule_obj) = rule_val.as_object_mut() else {
            remaining.insert(id.clone(), entry.clone());
            continue;
        };
        let Some(c_obj) = rule_obj.get_mut("config").and_then(|c| c.as_object_mut()) else {
            remaining.insert(id.clone(), entry.clone());
            continue;
        };
        let Some(api_obj) = c_obj.get_mut("api").and_then(|a| a.as_object_mut()) else {
            remaining.insert(id.clone(), entry.clone());
            continue;
        };

        let route = read_byok_route(&format!("zcode:{}", id));
        let (headers_opt, _, value, hdr_ok) = zcode_headers(api_obj);
        let matches = route.is_some() && hdr_ok && headers_opt.is_some() && zcode_proxy_base(&zcode_base_url(api_obj))
            && route.as_ref().map(|r| value == byok_route_header(r)).unwrap_or(false);

        if !matches {
            remaining.insert(id.clone(), entry.clone());
            continue;
        }

        if api_obj.contains_key("baseUrl") {
            api_obj.insert("baseUrl".to_string(), Value::String(entry.base_url.clone()));
            api_obj.remove("baseURL");
        } else {
            api_obj.insert("baseURL".to_string(), Value::String(entry.base_url.clone()));
        }

        if let Some(Value::Object(hm)) = api_obj.get_mut("headers") {
            let byok_key = byok_route_header_key();
            if entry.had_header {
                hm.insert(byok_key.to_string(), Value::String(entry.header.clone()));
            } else {
                hm.remove(byok_key);
            }
            hm.remove(ZCODE_HEADROOM_BASE_URL_HEADER);
            hm.remove(ZCODE_HEADROOM_ORIGINAL_PATH_HDR);
        }
        if api_obj.get("headers").and_then(|h| h.as_object()).map(|hm| hm.is_empty()).unwrap_or(false) {
            api_obj.remove("headers");
        }

        if let Some(r) = route {
            deleted_routes.push(r);
        }
        deleted_ids.push(id.clone());
    }

    if deleted_ids.is_empty() {
        return false;
    }
    let content = stringify_json(&Value::Object(cfg));
    if write_file_atomic(&file, &content).is_err() {
        return false;
    }
    for id in &deleted_ids {
        if delete_byok_route(&format!("zcode:{}", id)).is_err() {
            let _ = write_file(&file, &raw);
            return false;
        }
    }
    if save_proxy_route_stash("zcode", &remaining).is_err() {
        let _ = write_file(&file, &raw);
        for r in deleted_routes {
            let _ = upsert_byok_route(&r.id, &r.protocol, &r.upstream);
        }
        return false;
    }

    true
}

pub fn zcode_proxy_wired() -> bool {
    let stash = load_proxy_route_stash("zcode");
    if stash.is_empty() {
        return false;
    }
    let raw = match read_file_safe(&zcode_provider_config_path()) {
        Some(r) => r,
        None => return false,
    };
    if has_jsonc_comments(&raw) {
        return false;
    }
    let cfg = match try_parse_jsonc(&raw) {
        Some(c) => c,
        None => return false,
    };

    let rules = cfg.get("config")
        .and_then(|c| c.get("providerConfigRules"))
        .and_then(|r| r.get("providerRules"))
        .and_then(|pr| pr.as_array());

    let Some(rules_arr) = rules else {
        return false;
    };

    for (id, entry) in &stash {
        let rule = rules_arr.iter().find(|r| {
            r.as_object().map(|obj| zcode_string_field(obj, "providerId") == *id).unwrap_or(false)
        });
        let Some(rule_val) = rule else { return false };
        let Some(rule_obj) = rule_val.as_object() else { return false };
        let Some(c_obj) = rule_obj.get("config").and_then(|c| c.as_object()) else { return false };
        let Some(api_obj) = c_obj.get("api").and_then(|a| a.as_object()) else { return false };

        if !zcode_headroom_base(&zcode_base_url(api_obj)) {
            return false;
        }
        let route = match read_byok_route(&format!("zcode:{}", id)) {
            Some(r) => r,
            None => return false,
        };
        let (headers_opt, _, value, hdr_ok) = zcode_headers(api_obj);
        if !hdr_ok || headers_opt.is_none() || value != byok_route_header(&route) {
            return false;
        }
        let (origin, chat_path) = match zcode_headroom_upstream(&entry.base_url) {
            Some(res) => res,
            None => return false,
        };
        let hm = headers_opt.unwrap();
        if hm.get(ZCODE_HEADROOM_BASE_URL_HEADER).and_then(|v| v.as_str()) != Some(&origin) {
            return false;
        }
        if hm.get(ZCODE_HEADROOM_ORIGINAL_PATH_HDR).and_then(|v| v.as_str()) != Some(&chat_path) {
            return false;
        }
    }

    true
}
