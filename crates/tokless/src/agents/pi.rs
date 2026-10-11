use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::core::AgentManifest;
use crate::util::byok::{BYOKRoute, read_byok_routes};
use crate::util::exec::{RunOptions, run};
use crate::util::jsonc::{OrderedMap, Value, has_jsonc_comments, stringify_json};
use crate::util::logger::L;
use crate::util::mcpspawn::{mcp_spawn_for, pick_mcp_spawn, wrap_auto_index};
use crate::util::paths;

use super::proxy_detection::{ProxyConfigState, classify_pi_models_json_proxy};
use super::proxy_protocol::proxy_endpoint_for;
use super::proxy_provider::{
    ByokRouteRollback, ProxyRouteStashEntry, delete_byok_route, detect_agent_with,
    is_valid_byok_upstream, json_args_eq, json_equal, json_get_or_create_map, json_map_child,
    json_map_child_mut, load_proxy_route_stash, load_proxy_route_stash_locked,
    normalized_headroom_upstream, provider_spec_active, proxy_endpoint_for_api,
    proxy_route_stash_path, proxy_route_stash_valid, proxy_wire_key, register_byok_route,
    restore_deleted_byok_routes, restore_proxy_route_stash_logged, rollback_byok_route_logged,
    same_proxy_base, save_proxy_route_stash, strings_to_values,
    try_parse_jsonc_map as try_parse_jsonc, with_proxy_route_stash_lock,
};

pub const PI_SRC_MCP_ADAPTER: &str = "npm:pi-mcp-adapter";
pub const PI_PROXY_PROVIDER: &str = "tokless-headroom";

pub fn pi_agent_dir_resolved() -> String {
    paths::pi_paths_resolved().agent_dir
}

pub fn pi_manifest() -> AgentManifest {
    AgentManifest {
        id: "pi".to_string(),
        label: "Pi Coding Agent".to_string(),
        homepage: "https://github.com/badlogic/pi-mono".to_string(),
        cli_bin: "pi".to_string(),
        config_dir: Some(Arc::new(|| paths::pi_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            detect_agent_with(
                "pi",
                &paths::pi_paths_resolved().dir,
                &paths::expected_bin_dirs(),
                &[],
            )
        })),
    }
}

pub fn has_pi_rtk_extension() -> bool {
    paths::exists(&paths::pi_paths_resolved().rtk_extension)
}

pub fn pi_package_list() -> Vec<String> {
    let settings_file = paths::pi_paths_resolved().settings;
    let Some(raw) = paths::read_file_safe(&settings_file) else {
        return Vec::new();
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return Vec::new();
    };
    let Some(arr) = cfg.get("packages").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|v| {
            if let Some(s) = v.as_str() {
                Some(s.to_string())
            } else if let Some(m) = v.as_object() {
                m.get("source")
                    .and_then(|sv| sv.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect()
}

pub fn pi_source_has(source: &str) -> bool {
    let want = source.trim();
    pi_package_list()
        .iter()
        .any(|item| item == want || item.starts_with(&format!("{want}@")))
}

pub fn pi_package_settings_source(source: &str) -> String {
    let want = source.trim();
    for item in pi_package_list() {
        if item == want || item.starts_with(&format!("{want}@")) {
            return item;
        }
    }
    String::new()
}

pub fn pi_install_source(source: &str) -> bool {
    if pi_source_has(source) {
        return true;
    }
    let bin = paths::find_binary("pi", &paths::expected_bin_dirs());
    if !bin.is_empty() {
        let r = run(
            &bin,
            &["install", source],
            RunOptions {
                capture: true,
                ..Default::default()
            },
        );
        if r.code == 0 && pi_source_has(source) {
            return true;
        }
    }
    let settings_file = paths::pi_paths_resolved().settings;
    let raw_opt = paths::read_file_safe(&settings_file);
    let exists = raw_opt.is_some();
    let raw = raw_opt.unwrap_or_default();
    if exists && has_jsonc_comments(&raw) {
        return false;
    }
    let mut cfg = if raw.trim().is_empty() {
        OrderedMap::new()
    } else {
        let Some(c) = try_parse_jsonc(&raw) else {
            return false;
        };
        c
    };
    let mut pkgs = cfg
        .get("packages")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    pkgs.push(Value::String(source.to_string()));
    cfg.insert("packages".to_string(), Value::Array(pkgs));
    let next = stringify_json(&Value::Object(cfg));
    paths::write_file_atomic(&settings_file, &next).is_ok()
}

pub fn pi_remove_source(source: &str) -> bool {
    let actual = pi_package_settings_source(source);
    let target = if actual.is_empty() {
        source.to_string()
    } else {
        actual
    };
    let bin = paths::find_binary("pi", &paths::expected_bin_dirs());
    if !bin.is_empty() {
        let r = run(
            &bin,
            &["remove", &target],
            RunOptions {
                capture: true,
                ..Default::default()
            },
        );
        if r.code == 0 && !pi_source_has(source) {
            return true;
        }
    }
    let settings_file = paths::pi_paths_resolved().settings;
    let Some(raw) = paths::read_file_safe(&settings_file) else {
        return false;
    };
    if has_jsonc_comments(&raw) {
        return false;
    }
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(pkgs) = cfg.get("packages").and_then(|v| v.as_array()).cloned() else {
        return false;
    };
    let before = pkgs.len();
    let kept: Vec<Value> = pkgs
        .into_iter()
        .filter(|v| {
            let s = v
                .as_str()
                .or_else(|| {
                    v.as_object()
                        .and_then(|m| m.get("source"))
                        .and_then(|sv| sv.as_str())
                })
                .unwrap_or("");
            s != source && s != target && !s.starts_with(&format!("{source}@"))
        })
        .collect();
    if kept.len() == before {
        return false;
    }
    if kept.is_empty() {
        cfg.shift_remove("packages");
    } else {
        cfg.insert("packages".to_string(), Value::Array(kept));
    }
    let next = stringify_json(&Value::Object(cfg));
    paths::write_file_atomic(&settings_file, &next).is_ok()
}

pub fn pi_update_packages() {
    let bin = paths::find_binary("pi", &paths::expected_bin_dirs());
    if !bin.is_empty() {
        let _ = run(
            &bin,
            &["update"],
            RunOptions {
                capture: true,
                ..Default::default()
            },
        );
    }
}

pub fn pi_purge_context_mode_packages() -> usize {
    let mut removed = 0usize;
    for item in pi_package_list() {
        if item.contains("context-mode") && pi_remove_source(&item) {
            removed += 1;
        }
    }
    removed
}

pub fn pi_purge_codegraph_packages() -> usize {
    let mut removed = 0usize;
    for item in pi_package_list() {
        if item.contains("codegraph") && pi_remove_source(&item) {
            removed += 1;
        }
    }
    removed
}

pub fn pi_remove_mcp_adapter() -> bool {
    let p = paths::pi_paths_resolved().mcp_config;
    if let Some(raw) = paths::read_file_safe(&p)
        && let Some(cfg) = try_parse_jsonc(&raw)
        && json_map_child(&cfg, "mcpServers").is_some_and(|sm| !sm.is_empty())
    {
        return false;
    }
    pi_remove_source(PI_SRC_MCP_ADAPTER)
}

pub fn configure_pi_mcp(tool_id: &str) -> (bool, String) {
    let p = paths::pi_paths_resolved().mcp_config;
    let raw_opt = paths::read_file_safe(&p);
    let exists = raw_opt.is_some();
    if !exists && paths::exists(&p) {
        return (false, p);
    }
    let raw = raw_opt.unwrap_or_default();
    if exists && has_jsonc_comments(&raw) {
        return (false, p);
    }
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
    let desired = Value::Object(pi_mcp_desired(tool_id));
    let servers = json_map_child_mut(&mut cfg, "mcpServers").unwrap();
    if let Some(existing) = servers.get(tool_id) {
        if json_equal(existing, &desired) {
            let _ = pi_install_source(PI_SRC_MCP_ADAPTER);
            return (false, p);
        }
        if !pi_mcp_managed(tool_id, existing) {
            return (false, p);
        }
    }
    servers.insert(tool_id.to_string(), desired);
    let next = stringify_json(&Value::Object(cfg));
    if paths::write_file_atomic(&p, &next).is_err() {
        return (false, p);
    }
    let _ = pi_install_source(PI_SRC_MCP_ADAPTER);
    (true, p)
}

pub fn remove_pi_mcp(tool_id: &str) -> bool {
    let p = paths::pi_paths_resolved().mcp_config;
    let Some(raw) = paths::read_file_safe(&p) else {
        return false;
    };
    if has_jsonc_comments(&raw) {
        return false;
    }
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(servers) = json_map_child_mut(&mut cfg, "mcpServers") else {
        return false;
    };
    let Some(existing) = servers.get(tool_id) else {
        return false;
    };
    if !pi_mcp_managed(tool_id, existing) {
        return false;
    }
    servers.shift_remove(tool_id);
    if servers.is_empty() {
        cfg.shift_remove("mcpServers");
    }
    let next = stringify_json(&Value::Object(cfg));
    let ok = paths::write_file_atomic(&p, &next).is_ok();
    if ok {
        let _ = pi_remove_mcp_adapter();
    }
    ok
}

pub fn pi_mcp_bounded(tool_id: &str) -> bool {
    let p = paths::pi_paths_resolved().mcp_config;
    let Some(raw) = paths::read_file_safe(&p) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(servers) = json_map_child(&cfg, "mcpServers") else {
        return false;
    };
    let Some(existing) = servers.get(tool_id) else {
        return false;
    };
    pi_mcp_managed(tool_id, existing) && pi_source_has(PI_SRC_MCP_ADAPTER)
}

pub fn pi_projectmem_mcp_bounded() -> bool {
    pi_mcp_bounded("projectmem")
}

pub fn pi_mcp_has(tool_id: &str) -> bool {
    let p = paths::pi_paths_resolved().mcp_config;
    let Some(raw) = paths::read_file_safe(&p) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    json_map_child(&cfg, "mcpServers").is_some_and(|sm| sm.contains_key(tool_id))
}

fn pi_mcp_desired(tool_id: &str) -> OrderedMap {
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index("pi", pick_mcp_spawn("codegraph", &["serve", "--mcp"]))
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
    entry.insert("lifecycle".to_string(), Value::String("lazy".to_string()));
    entry.insert("directTools".to_string(), Value::Bool(true));
    entry
}

fn pi_mcp_managed(tool_id: &str, existing: &Value) -> bool {
    let Some(em) = existing.as_object() else {
        return false;
    };
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index("pi", pick_mcp_spawn("codegraph", &["serve", "--mcp"]))
    } else {
        mcp_spawn_for(tool_id)
    };
    em.get("command").and_then(|v| v.as_str()) == Some(spawn.command.as_str())
        && json_args_eq(em.get("args"), &spawn.args)
}

// --- pi_proxy.go ---

pub fn pi_models_file() -> String {
    Path::new(&pi_agent_dir_resolved())
        .join("models.json")
        .to_string_lossy()
        .into_owned()
}

pub fn pi_proxy_provider_entry(endpoint: &str) -> OrderedMap {
    let spec = provider_spec_active();
    let mut entry = OrderedMap::new();
    entry.insert("baseUrl".to_string(), Value::String(endpoint.to_string()));
    entry.insert(
        "api".to_string(),
        Value::String("openai-completions".to_string()),
    );
    entry.insert("apiKey".to_string(), Value::String(proxy_wire_key()));

    let mut models = Vec::with_capacity(spec.models.len());
    for m in &spec.models {
        let mut mv = OrderedMap::new();
        mv.insert("id".to_string(), Value::String(m.id.clone()));
        mv.insert("name".to_string(), Value::String(m.display.clone()));
        mv.insert("reasoning".to_string(), Value::Bool(m.reasoning));
        mv.insert(
            "input".to_string(),
            Value::Array(vec![Value::String("text".to_string())]),
        );
        if m.context > 0 {
            mv.insert("contextWindow".to_string(), Value::from(m.context));
        }
        if m.output > 0 {
            mv.insert("maxTokens".to_string(), Value::from(m.output));
        }
        models.push(Value::Object(mv));
    }
    entry.insert("models".to_string(), Value::Array(models));
    entry
}

pub fn configure_pi_proxy() -> (bool, String) {
    let mut out = (false, pi_models_file());
    if let Err(err) = with_proxy_route_stash_lock(|| {
        out = configure_pi_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("pi proxy lock failed: {err}"));
    }
    out
}

fn configure_pi_proxy_locked() -> (bool, String) {
    let file = pi_models_file();
    let raw_opt = paths::read_file_safe(&file);
    let exists = raw_opt.is_some();
    if !exists && paths::exists(&file) {
        return (false, file);
    }
    let raw = raw_opt.unwrap_or_default();
    if exists && has_jsonc_comments(&raw) {
        return (false, file);
    }
    if !proxy_route_stash_valid("pi") {
        return (false, file);
    }
    let mut stash = load_proxy_route_stash_locked("pi");
    let endpoint = proxy_endpoint_for("pi");
    let st = classify_pi_models_json_proxy(&raw, &endpoint, &stash);
    if matches!(st, ProxyConfigState::Unreadable | ProxyConfigState::Conflict) {
        return (false, file);
    }

    let mut cfg = if raw.trim().is_empty() {
        OrderedMap::new()
    } else {
        let Some(c) = try_parse_jsonc(&raw) else {
            return (false, file);
        };
        c
    };

    let stash_path = proxy_route_stash_path("pi");
    let prev_stash_raw = paths::read_file_safe(&stash_path);
    let prev_stash_exists = prev_stash_raw.is_some();

    let mut rollbacks: Vec<ByokRouteRollback> = Vec::new();
    let mut changed = false;

    let providers = json_get_or_create_map(&mut cfg, "providers");
    let provider_ids: Vec<String> = providers.keys().cloned().collect();
    let active_key = provider_spec_active().key;

    for id in provider_ids {
        if id == PI_PROXY_PROVIDER || id == active_key {
            continue;
        }
        let Some(entry_val) = providers.get(&id).cloned() else {
            continue;
        };
        let Some(mut entry_map) = entry_val.as_object().cloned() else {
            continue;
        };
        let base_key = if entry_map.contains_key("baseURL") && !entry_map.contains_key("baseUrl") {
            "baseURL".to_string()
        } else {
            "baseUrl".to_string()
        };
        let base = entry_map
            .get(&base_key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if base.is_empty() || same_proxy_base(&base, &endpoint) {
            continue;
        }
        let api = entry_map
            .get("api")
            .and_then(|v| v.as_str())
            .unwrap_or("openai-completions");
        let api = if api == "anthropic-messages" {
            "anthropic-messages"
        } else if api == "openai-responses" {
            "openai-responses"
        } else {
            "openai-completions"
        };
        let upstream = normalized_headroom_upstream(&base, api);
        if !is_valid_byok_upstream(&upstream) {
            for rb in &rollbacks {
                rollback_byok_route_logged("pi", &rb.id, &rb.previous);
            }
            return (false, file);
        }
        let (_, hdr, prev_route, ok) = register_byok_route("pi", &id, api, &upstream);
        if !ok {
            for rb in &rollbacks {
                rollback_byok_route_logged("pi", &rb.id, &rb.previous);
            }
            return (false, file);
        }
        rollbacks.push(ByokRouteRollback {
            id: id.clone(),
            previous: prev_route,
        });

        let target_ep = proxy_endpoint_for_api(api);
        entry_map.insert(base_key.clone(), Value::String(target_ep));
        let headers = json_get_or_create_map(&mut entry_map, "headers");
        if let Some((hk, hv)) = hdr.split_once(':') {
            headers.insert(hk.trim().to_string(), Value::String(hv.trim().to_string()));
        }
        let managed_val = Value::Object(entry_map);
        stash.insert(
            id.clone(),
            ProxyRouteStashEntry {
                file: file.clone(),
                provider: id.clone(),
                base_url: base,
                upstream,
                base_key,
                had_base_key: true,
                header: hdr,
                original: serde_json::to_vec(&entry_val).unwrap_or_default(),
                managed: serde_json::to_vec(&managed_val).unwrap_or_default(),
                ..Default::default()
            },
        );
        providers.insert(id, managed_val);
        changed = true;
    }

    let desired_entry = Value::Object(pi_proxy_provider_entry(&endpoint));
    match providers.get(PI_PROXY_PROVIDER) {
        Some(existing) if json_equal(existing, &desired_entry) => {}
        _ => {
            providers.insert(PI_PROXY_PROVIDER.to_string(), desired_entry);
            changed = true;
        }
    }

    if save_proxy_route_stash("pi", &stash).is_err() {
        for rb in &rollbacks {
            rollback_byok_route_logged("pi", &rb.id, &rb.previous);
        }
        return (false, file);
    }

    if !changed {
        return (false, file);
    }

    let next = stringify_json(&Value::Object(cfg));
    if paths::write_file_atomic(&file, &next).is_err() {
        restore_proxy_route_stash_logged(
            "pi",
            prev_stash_raw.as_deref().unwrap_or(""),
            prev_stash_exists,
        );
        for rb in &rollbacks {
            rollback_byok_route_logged("pi", &rb.id, &rb.previous);
        }
        return (false, file);
    }
    (true, file)
}

pub fn remove_pi_proxy() -> bool {
    let mut removed = false;
    if let Err(err) = with_proxy_route_stash_lock(|| {
        removed = remove_pi_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("pi proxy lock failed: {err}"));
    }
    removed
}

fn remove_pi_proxy_locked() -> bool {
    let file = pi_models_file();
    if !proxy_route_stash_valid("pi") {
        return false;
    }
    let mut stash = load_proxy_route_stash_locked("pi");
    let Some(raw) = paths::read_file_safe(&file) else {
        if !stash.is_empty() {
            for id in stash.keys() {
                let _ = delete_byok_route("pi", id);
            }
            stash.clear();
            let _ = save_proxy_route_stash("pi", &stash);
            return true;
        }
        return false;
    };
    if has_jsonc_comments(&raw) {
        return false;
    }
    let endpoint = proxy_endpoint_for("pi");
    if classify_pi_models_json_proxy(&raw, &endpoint, &stash) != ProxyConfigState::Managed {
        return false;
    }
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };

    let stash_path = proxy_route_stash_path("pi");
    let prev_stash_raw = paths::read_file_safe(&stash_path);
    let prev_stash_exists = prev_stash_raw.is_some();
    let existing_routes: Vec<BYOKRoute> = read_byok_routes()
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r.id.starts_with("pi:"))
        .collect();

    let mut changed = false;
    if let Some(providers) = json_map_child_mut(&mut cfg, "providers") {
        if providers.shift_remove(PI_PROXY_PROVIDER).is_some() {
            changed = true;
        }
        let active_key = provider_spec_active().key;
        if providers.shift_remove(&active_key).is_some() {
            changed = true;
        }
        let stashed_ids: Vec<String> = stash.keys().cloned().collect();
        for id in stashed_ids {
            let Some(entry) = stash.get(&id) else {
                continue;
            };
            if let Ok(orig_val) = serde_json::from_slice::<Value>(&entry.original) {
                providers.insert(id.clone(), orig_val);
                changed = true;
            }
            let _ = delete_byok_route("pi", &id);
            stash.remove(&id);
        }
        if providers.is_empty() {
            cfg.shift_remove("providers");
        }
    }

    if save_proxy_route_stash("pi", &stash).is_err() {
        restore_deleted_byok_routes(&existing_routes);
        return false;
    }

    if changed {
        if cfg.is_empty() {
            if fs::remove_file(&file).is_err() {
                restore_proxy_route_stash_logged(
                    "pi",
                    prev_stash_raw.as_deref().unwrap_or(""),
                    prev_stash_exists,
                );
                restore_deleted_byok_routes(&existing_routes);
                return false;
            }
        } else {
            let next = stringify_json(&Value::Object(cfg));
            if paths::write_file_atomic(&file, &next).is_err() {
                restore_proxy_route_stash_logged(
                    "pi",
                    prev_stash_raw.as_deref().unwrap_or(""),
                    prev_stash_exists,
                );
                restore_deleted_byok_routes(&existing_routes);
                return false;
            }
        }
    }
    changed
}

pub fn pi_proxy_wired() -> bool {
    let file = pi_models_file();
    let Some(raw) = paths::read_file_safe(&file) else {
        return false;
    };
    if !proxy_route_stash_valid("pi") {
        return false;
    }
    let stash = load_proxy_route_stash("pi");
    classify_pi_models_json_proxy(&raw, &proxy_endpoint_for("pi"), &stash)
        == ProxyConfigState::Managed
}
