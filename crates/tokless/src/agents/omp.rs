// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/omp.rs
// created: 2026-10-11
// lastModified: 2026-10-14

use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::core::AgentManifest;
use crate::util::byok::{BYOKRoute, read_byok_routes};
use crate::util::jsonc::{OrderedMap, Value, stringify_json};
use crate::util::logger::L;
use crate::util::mcpspawn::{mcp_spawn_for, pick_mcp_spawn, wrap_auto_index};
use crate::util::paths;

use super::proxy_detection::{ProxyConfigState, classify_omp_yaml_proxy};
use super::proxy_protocol::proxy_endpoint_for;
use super::proxy_provider::{
    ByokRouteRollback, ProxyRouteStashEntry, delete_byok_route, detect_agent_with,
    is_valid_byok_upstream, json_args_eq, json_equal, json_map_child, json_map_child_mut,
    load_proxy_route_stash, load_proxy_route_stash_locked, normalized_headroom_upstream,
    provider_spec_active, proxy_endpoint_for_api, proxy_route_stash_path, proxy_route_stash_valid,
    proxy_wire_key, register_byok_route, restore_deleted_byok_routes,
    restore_proxy_route_stash_logged, rollback_byok_route_logged, same_proxy_base,
    save_proxy_route_stash, strings_to_values, try_parse_jsonc_map as try_parse_jsonc,
    with_proxy_route_stash_lock,
};

pub const OMP_ROLE_MODEL: &str = "deepseek-v4-flash";
pub const OMP_DISCOVERY_TYPE: &str = "openai-models-list";

pub fn omp_agent_dir_resolved() -> String {
    paths::omp_paths_resolved().agent_dir
}

pub fn omp_manifest() -> AgentManifest {
    AgentManifest {
        id: "omp".to_string(),
        label: "Oh My Pi".to_string(),
        homepage: "https://github.com/can1357/oh-my-pi".to_string(),
        cli_bin: "omp".to_string(),
        config_dir: Some(Arc::new(|| paths::omp_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            detect_agent_with(
                "omp",
                &paths::omp_paths_resolved().dir,
                &paths::expected_bin_dirs(),
                &[],
            )
        })),
    }
}

pub fn has_omp_rtk_extension() -> bool {
    paths::exists(&paths::omp_paths_resolved().rtk_extension)
}

pub fn configure_omp_mcp(tool_id: &str) -> (bool, String) {
    let p = paths::omp_paths_resolved().config;
    let raw_opt = paths::read_file_safe(&p);
    let exists = raw_opt.is_some();
    if !exists && paths::exists(&p) {
        return (false, p);
    }
    let raw = raw_opt.unwrap_or_default();
    let mut cfg = if raw.trim().is_empty() {
        OrderedMap::new()
    } else {
        let Some(c) = try_parse_jsonc(&raw) else {
            return (false, p);
        };
        c
    };
    if cfg.get("mcpServers").is_some_and(|v| !v.is_object()) {
        return (false, p);
    }
    if !cfg.contains_key("mcpServers") {
        cfg.insert("mcpServers".to_string(), Value::Object(OrderedMap::new()));
    }
    let desired = Value::Object(omp_mcp_desired(tool_id));
    let servers = json_map_child_mut(&mut cfg, "mcpServers").unwrap();
    if let Some(existing) = servers.get(tool_id) {
        if json_equal(existing, &desired) {
            return (false, p);
        }
        if !omp_mcp_managed(tool_id, existing) {
            return (false, p);
        }
    }
    servers.insert(tool_id.to_string(), desired);
    let next = stringify_json(&Value::Object(cfg));
    if next == raw {
        return (false, p);
    }
    if paths::write_file_atomic(&p, &next).is_err() {
        return (false, p);
    }
    (true, p)
}

pub fn remove_omp_mcp(tool_id: &str) -> bool {
    let p = paths::omp_paths_resolved().config;
    let Some(raw) = paths::read_file_safe(&p) else {
        return false;
    };
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(servers) = json_map_child_mut(&mut cfg, "mcpServers") else {
        return false;
    };
    let Some(existing) = servers.get(tool_id) else {
        return false;
    };
    if !omp_mcp_managed(tool_id, existing) {
        return false;
    }
    servers.shift_remove(tool_id);
    if servers.is_empty() {
        cfg.shift_remove("mcpServers");
    }
    let next = stringify_json(&Value::Object(cfg));
    paths::write_file_atomic(&p, &next).is_ok()
}

pub fn omp_mcp_has(tool_id: &str) -> bool {
    let p = paths::omp_paths_resolved().config;
    let Some(raw) = paths::read_file_safe(&p) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    json_map_child(&cfg, "mcpServers").is_some_and(|sm| sm.contains_key(tool_id))
}

fn omp_mcp_desired(tool_id: &str) -> OrderedMap {
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index("omp", pick_mcp_spawn("codegraph", &["serve", "--mcp"]))
    } else {
        mcp_spawn_for(tool_id)
    };
    let mut entry = OrderedMap::new();
    entry.insert("command".to_string(), Value::String(spawn.command));
    if !spawn.args.is_empty() {
        entry.insert(
            "args".to_string(),
            Value::Array(strings_to_values(&spawn.args)),
        );
    }
    entry
}

fn omp_mcp_managed(tool_id: &str, existing: &Value) -> bool {
    let Some(em) = existing.as_object() else {
        return false;
    };
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index("omp", pick_mcp_spawn("codegraph", &["serve", "--mcp"]))
    } else {
        mcp_spawn_for(tool_id)
    };
    em.get("command").and_then(|v| v.as_str()) == Some(spawn.command.as_str())
        && json_args_eq(em.get("args"), &spawn.args)
}

// --- omp_proxy.go ---

pub fn omp_models_file() -> String {
    Path::new(&omp_agent_dir_resolved())
        .join("models.yml")
        .to_string_lossy()
        .into_owned()
}

#[derive(Debug, Clone, Default)]
pub struct OmpYamlProviderInfo {
    pub name: String,
    pub base_url: String,
    pub base_line: String,
    pub api: String,
    pub api_key: String,
    pub header_line: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Default)]
pub struct OmpYamlScan {
    pub valid: bool,
    pub has_providers_root: bool,
    pub providers: Vec<OmpYamlProviderInfo>,
}

pub fn omp_yaml_scan(raw: &str) -> OmpYamlScan {
    if raw.trim().is_empty() {
        return OmpYamlScan {
            valid: true,
            has_providers_root: false,
            providers: Vec::new(),
        };
    }
    let lines: Vec<&str> = raw.split('\n').collect();
    let mut scan = OmpYamlScan {
        valid: true,
        has_providers_root: false,
        providers: Vec::new(),
    };
    let mut in_providers = false;
    let mut current: Option<OmpYamlProviderInfo> = None;

    for (idx, raw_line) in lines.iter().enumerate() {
        let line = raw_line.trim_end_matches('\r');
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = line.len() - line.trim_start_matches(' ').len();
        if line.contains('\t') {
            scan.valid = false;
            return scan;
        }
        if indent == 0 {
            if let Some(mut cur) = current.take() {
                cur.end_line = idx;
                scan.providers.push(cur);
            }
            if trimmed == "providers:" {
                scan.has_providers_root = true;
                in_providers = true;
            } else if trimmed.ends_with(':') || trimmed.contains(':') {
                in_providers = false;
            } else {
                scan.valid = false;
                return scan;
            }
            continue;
        }
        if !in_providers {
            continue;
        }
        if indent == 2 && trimmed.ends_with(':') {
            if let Some(mut cur) = current.take() {
                cur.end_line = idx;
                scan.providers.push(cur);
            }
            let name = trimmed.trim_end_matches(':').trim().to_string();
            current = Some(OmpYamlProviderInfo {
                name,
                start_line: idx,
                end_line: lines.len(),
                ..Default::default()
            });
            continue;
        }
        if let Some(cur) = current.as_mut() {
            if indent >= 4
                && let Some((k, v)) = trimmed.split_once(':')
            {
                let key = k.trim();
                let val = unquote_yaml(v.trim());
                match key {
                    "baseUrl" | "baseURL" | "base_url" => {
                        cur.base_url = val;
                        cur.base_line = line.to_string();
                    }
                    "api" => cur.api = val,
                    "apiKey" | "api_key" => cur.api_key = val,
                    "x-headroom-byok-route" => cur.header_line = line.to_string(),
                    _ => {}
                }
            }
        }
    }
    if let Some(mut cur) = current.take() {
        cur.end_line = lines.len();
        scan.providers.push(cur);
    }
    scan
}

fn unquote_yaml(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.len() >= 2
        && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
    {
        return trimmed[1..trimmed.len() - 1].to_string();
    }
    trimmed.to_string()
}

pub fn omp_yaml_headroom_managed(raw: &str) -> bool {
    let provider_key = provider_spec_active().key;
    let endpoint = proxy_endpoint_for("omp");
    let scan = omp_yaml_scan(raw);
    if !scan.valid {
        return false;
    }
    scan.providers.iter().any(|p| {
        (p.name == provider_key || p.name == "headroom") && same_proxy_base(&p.base_url, &endpoint)
    })
}

fn render_omp_managed_yaml(endpoint: &str) -> String {
    let spec = provider_spec_active();
    let key = proxy_wire_key();
    format!(
        "providers:\n\
  {}:\n\
    baseUrl: {endpoint}\n\
    apiKey: {key}\n\
    api: openai-completions\n\
    discovery:\n\
      type: {OMP_DISCOVERY_TYPE}\n\
    roleModel: {OMP_ROLE_MODEL}\n",
        spec.key
    )
}

pub fn configure_omp_proxy() -> (bool, String) {
    let mut out = (false, omp_models_file());
    if let Err(err) = with_proxy_route_stash_lock(|| {
        out = configure_omp_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("omp proxy lock failed: {err}"));
    }
    out
}

fn configure_omp_proxy_locked() -> (bool, String) {
    let file = omp_models_file();
    let raw_opt = paths::read_file_safe(&file);
    let exists = raw_opt.is_some();
    if !exists && paths::exists(&file) {
        return (false, file);
    }
    let raw = raw_opt.unwrap_or_default();
    if !proxy_route_stash_valid("omp") {
        return (false, file);
    }
    let mut stash = load_proxy_route_stash_locked("omp");
    let endpoint = proxy_endpoint_for("omp");
    let st = classify_omp_yaml_proxy(&raw, &endpoint, &stash);

    match st {
        ProxyConfigState::Unreadable
        | ProxyConfigState::Conflict
        | ProxyConfigState::Unknown => return (false, file),
        ProxyConfigState::Managed => return (false, file),
        ProxyConfigState::Absent | ProxyConfigState::NativeUnconfigured => {
            let rendered = render_omp_managed_yaml(&endpoint);
            let next = if raw.trim().is_empty() {
                rendered
            } else {
                let scan = omp_yaml_scan(&raw);
                if scan.has_providers_root {
                    let block: String = rendered
                        .lines()
                        .skip(1)
                        .map(|l| format!("{l}\n"))
                        .collect();
                    format!("{}\n{block}", raw.trim_end_matches('\n'))
                } else {
                    format!("{}\n{rendered}", raw.trim_end_matches('\n'))
                }
            };
            stash.insert(
                provider_spec_active().key,
                ProxyRouteStashEntry {
                    file: file.clone(),
                    provider: provider_spec_active().key,
                    managed_native: true,
                    original: raw.as_bytes().to_vec(),
                    managed: next.as_bytes().to_vec(),
                    ..Default::default()
                },
            );
            if save_proxy_route_stash("omp", &stash).is_err() {
                return (false, file);
            }
            if paths::write_file_atomic(&file, &next).is_err() {
                return (false, file);
            }
            return (true, file);
        }
        ProxyConfigState::ForeignBYOK => {}
    }

    let scan = omp_yaml_scan(&raw);
    if !scan.valid {
        return (false, file);
    }
    let stash_path = proxy_route_stash_path("omp");
    let prev_stash_raw = paths::read_file_safe(&stash_path);
    let prev_stash_exists = prev_stash_raw.is_some();

    let mut rollbacks: Vec<ByokRouteRollback> = Vec::new();
    let mut updated = raw.clone();
    let active_key = provider_spec_active().key;

    for p in &scan.providers {
        if p.name == active_key || p.name == "headroom" {
            continue;
        }
        if p.base_url.is_empty() || same_proxy_base(&p.base_url, &endpoint) {
            continue;
        }
        let api = if p.api == "anthropic-messages" {
            "anthropic-messages"
        } else {
            "openai-completions"
        };
        let upstream = normalized_headroom_upstream(&p.base_url, api);
        if !is_valid_byok_upstream(&upstream) {
            for rb in &rollbacks {
                rollback_byok_route_logged("omp", &rb.id, &rb.previous);
            }
            return (false, file);
        }
        let (_, hdr, prev_route, ok) = register_byok_route("omp", &p.name, api, &upstream);
        if !ok {
            for rb in &rollbacks {
                rollback_byok_route_logged("omp", &rb.id, &rb.previous);
            }
            return (false, file);
        }
        rollbacks.push(ByokRouteRollback {
            id: p.name.clone(),
            previous: prev_route,
        });
        let target_ep = proxy_endpoint_for_api(api);
        let new_base_line = format!("    baseUrl: {target_ep}");
        let hdr_line = if let Some((hk, hv)) = hdr.split_once(':') {
            format!("    headers:\n      {}: {}", hk.trim(), hv.trim())
        } else {
            String::new()
        };
        if !p.base_line.is_empty() {
            let replacement = if hdr_line.is_empty() {
                new_base_line.clone()
            } else {
                format!("{new_base_line}\n{hdr_line}")
            };
            updated = updated.replacen(&p.base_line, &replacement, 1);
        }
        stash.insert(
            p.name.clone(),
            ProxyRouteStashEntry {
                file: file.clone(),
                provider: p.name.clone(),
                base_url: p.base_url.clone(),
                upstream,
                base_key: "baseUrl".to_string(),
                had_base_key: true,
                base_line: p.base_line.clone(),
                header: hdr,
                header_line: hdr_line,
                original: raw.as_bytes().to_vec(),
                managed: updated.as_bytes().to_vec(),
                ..Default::default()
            },
        );
    }

    if save_proxy_route_stash("omp", &stash).is_err() {
        for rb in &rollbacks {
            rollback_byok_route_logged("omp", &rb.id, &rb.previous);
        }
        return (false, file);
    }
    if paths::write_file_atomic(&file, &updated).is_err() {
        restore_proxy_route_stash_logged(
            "omp",
            prev_stash_raw.as_deref().unwrap_or(""),
            prev_stash_exists,
        );
        for rb in &rollbacks {
            rollback_byok_route_logged("omp", &rb.id, &rb.previous);
        }
        return (false, file);
    }
    (true, file)
}

pub fn remove_omp_proxy() -> bool {
    let mut removed = false;
    if let Err(err) = with_proxy_route_stash_lock(|| {
        removed = remove_omp_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("omp proxy lock failed: {err}"));
    }
    removed
}

fn remove_omp_proxy_locked() -> bool {
    let file = omp_models_file();
    if !proxy_route_stash_valid("omp") {
        return false;
    }
    let mut stash = load_proxy_route_stash_locked("omp");
    let Some(raw) = paths::read_file_safe(&file) else {
        if !stash.is_empty() {
            for id in stash.keys() {
                let _ = delete_byok_route("omp", id);
            }
            stash.clear();
            let _ = save_proxy_route_stash("omp", &stash);
            return true;
        }
        return false;
    };
    let endpoint = proxy_endpoint_for("omp");
    if classify_omp_yaml_proxy(&raw, &endpoint, &stash) != ProxyConfigState::Managed {
        return false;
    }
    let existing_routes: Vec<BYOKRoute> = read_byok_routes()
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r.id.starts_with("omp:"))
        .collect();
    let stash_path = proxy_route_stash_path("omp");
    let prev_stash_raw = paths::read_file_safe(&stash_path);
    let prev_stash_exists = prev_stash_raw.is_some();

    let mut original_bytes: Option<Vec<u8>> = None;
    for entry in stash.values() {
        if !entry.managed.is_empty() && entry.managed.as_slice() == raw.as_bytes() {
            original_bytes = Some(entry.original.clone());
            break;
        }
    }

    let restored = if let Some(orig) = original_bytes {
        String::from_utf8_lossy(&orig).into_owned()
    } else if raw.trim() == render_omp_managed_yaml(&endpoint).trim() {
        String::new()
    } else {
        return false;
    };

    for id in stash.keys() {
        let _ = delete_byok_route("omp", id);
    }
    stash.clear();
    if save_proxy_route_stash("omp", &stash).is_err() {
        restore_deleted_byok_routes(&existing_routes);
        return false;
    }

    if restored.trim().is_empty() {
        if fs::remove_file(&file).is_err() && paths::exists(&file) {
            restore_proxy_route_stash_logged(
                "omp",
                prev_stash_raw.as_deref().unwrap_or(""),
                prev_stash_exists,
            );
            restore_deleted_byok_routes(&existing_routes);
            return false;
        }
    } else if paths::write_file_atomic(&file, &restored).is_err() {
        restore_proxy_route_stash_logged(
            "omp",
            prev_stash_raw.as_deref().unwrap_or(""),
            prev_stash_exists,
        );
        restore_deleted_byok_routes(&existing_routes);
        return false;
    }
    true
}

pub fn omp_proxy_wired() -> bool {
    let file = omp_models_file();
    let Some(raw) = paths::read_file_safe(&file) else {
        return false;
    };
    if !proxy_route_stash_valid("omp") {
        return false;
    }
    let stash = load_proxy_route_stash("omp");
    classify_omp_yaml_proxy(&raw, &proxy_endpoint_for("omp"), &stash) == ProxyConfigState::Managed
}
