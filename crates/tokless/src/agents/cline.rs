// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/cline.rs
// created: 2026-10-11
// lastModified: 2026-10-20

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::{AgentManifest, Detection};
use crate::util::byok::{BYOKRoute, read_byok_route};
use crate::util::jsonc::{OrderedMap, Value, parse_jsonc, stringify_json, try_parse_jsonc};
use crate::util::logger::L;
use crate::util::paths;

use super::proxy_detection::{ProxyCapability, ProxyDetection, ProxyState, proxy_detection};
use super::proxy_protocol::proxy_endpoint_for;
use super::proxy_provider::{
    HEADROOM_BASE_URL_HEADER, all_strings_nonempty, any_string_slice_equal, delete_byok_route,
    has_jsonc_comment, is_absolute_http, json_equal, kilo_codegraph_server, kilo_context_server,
    kilo_is_tokless_command, kilo_projectmem_server, normalized_headroom_upstream, proxy_wire_key,
    register_byok_route, replace_kilo_file, restore_deleted_byok_routes,
    rollback_byok_route_logged, same_proxy_base, string_slices_equal, strings_to_values,
    with_proxy_route_stash_lock,
};

use std::sync::Arc;

pub const CLINE_PROVIDER_NAME: &str = "openai-compatible";

pub fn cline_manifest() -> AgentManifest {
    AgentManifest {
        id: "cline".to_string(),
        label: "Cline".to_string(),
        homepage: "https://cline.bot".to_string(),
        cli_bin: "cline".to_string(),
        config_dir: Some(Arc::new(|| paths::cline_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            if !paths::find_binary::<&str>("cline", &[]).is_empty() {
                Detection {
                    installed: true,
                    source: "cli".to_string(),
                }
            } else {
                Detection::default()
            }
        })),
    }
}

pub fn cline_instructions_path() -> String {
    paths::cline_paths_resolved().instructions
}

pub fn configure_cline_mcp_safe(
    tool_id: &str,
    command: &[String],
) -> Result<(bool, String), String> {
    let p = paths::cline_paths_resolved().mcp_config;
    if tool_id == "headroom" {
        return Err("Headroom is HTTP proxy-only; MCP wiring is disabled".to_string());
    }
    if !cline_expected_command(tool_id, command) {
        return Err(format!(
            "unrecognized Tokless MCP command for Cline tool {tool_id:?}; refusing to adopt or overwrite entry"
        ));
    }
    let raw_opt = paths::read_file_safe(&p);
    let exists = raw_opt.is_some();
    let raw = raw_opt.unwrap_or_default();
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => {
            if exists && !raw.is_empty() {
                return Err(format!(
                    "cannot parse Cline MCP settings {p}; refusing to overwrite it"
                ));
            }
            OrderedMap::new()
        }
    };
    if let Some(v) = cfg.get("mcpServers")
        && !v.is_object()
    {
        return Err(
            "Cline config field \"mcpServers\" is not an object; refusing to overwrite it"
                .to_string(),
        );
    }
    if !cfg.contains_key("mcpServers") {
        cfg.insert("mcpServers".to_string(), Value::Object(OrderedMap::new()));
    }
    let mcp = cfg
        .get_mut("mcpServers")
        .and_then(|v| v.as_object_mut())
        .unwrap();

    let (state, state_exists) = cline_state_read()?;
    let had_ledger = state.contains_key(tool_id);
    if let Some(existing) = mcp.get(tool_id) {
        let Some(owned) = state.get(tool_id) else {
            return Err(format!(
                "Cline MCP {tool_id:?} has no Tokless ownership state; refusing same-name adoption"
            ));
        };
        if !state_exists {
            return Err(format!(
                "Cline MCP {tool_id:?} has no Tokless ownership state; refusing same-name adoption"
            ));
        }
        if !cline_managed_mcp_entry(existing, owned) || !cline_expected_command(tool_id, owned) {
            return Err(format!(
                "Cline MCP {tool_id:?} differs from Tokless ownership state; restore the owned entry or remove it manually"
            ));
        }
        if cline_managed_mcp_entry(existing, command) {
            return Ok((false, p));
        }
    }
    if state_exists
        && let Some(owned) = state.get(tool_id)
        && !string_slices_equal(owned, command)
    {
        return Err(format!(
            "Cline MCP {tool_id:?} ownership state differs from requested command"
        ));
    }
    if exists && has_jsonc_comment(&raw) {
        return Err(format!(
            "Cline MCP settings {p} contains JSONC comments; edit it without relocating comments before wiring Tokless"
        ));
    }
    let mut desired = OrderedMap::new();
    desired.insert("command".to_string(), Value::String(command[0].clone()));
    desired.insert(
        "args".to_string(),
        Value::Array(strings_to_values(&command[1..])),
    );
    desired.insert("env".to_string(), Value::Object(OrderedMap::new()));
    desired.insert("disabled".to_string(), Value::Bool(false));
    mcp.insert(tool_id.to_string(), Value::Object(desired));
    let content = stringify_json(&Value::Object(cfg));
    cline_state_set(tool_id, command)?;
    if let Err(err) = cline_write_file_guarded(&p, &content, &raw, exists) {
        if !had_ledger {
            let _ = cline_state_delete(tool_id);
        }
        return Err(err);
    }
    Ok((true, p))
}

pub fn remove_cline_mcp(tool_id: &str) -> bool {
    let p = paths::cline_paths_resolved().mcp_config;
    let Ok((state, state_exists)) = cline_state_read() else {
        return false;
    };
    if !state_exists {
        return false;
    }
    let Some(owned_cmd) = state.get(tool_id) else {
        return false;
    };
    let Some(raw) = paths::read_file_safe(&p) else {
        return cline_state_delete(tool_id).is_ok();
    };
    if has_jsonc_comment(&raw) {
        return false;
    }
    let Some(Value::Object(mut cfg)) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(mcp) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut()) else {
        return cline_state_delete(tool_id).is_ok();
    };
    let Some(entry) = mcp.get(tool_id) else {
        return cline_state_delete(tool_id).is_ok();
    };
    if !cline_managed_mcp_entry(entry, owned_cmd) {
        return false;
    }
    mcp.remove(tool_id);
    if mcp.is_empty() {
        cfg.remove("mcpServers");
    }
    if cline_write_file_guarded(&p, &stringify_json(&Value::Object(cfg)), &raw, true).is_err() {
        return false;
    }
    cline_state_delete(tool_id).is_ok()
}

pub fn cline_mcp_configured(tool_id: &str) -> bool {
    let p = paths::cline_paths_resolved().mcp_config;
    let Some(raw) = paths::read_file_safe(&p) else {
        return false;
    };
    let Some(Value::Object(cfg)) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(mcp) = cfg.get("mcpServers").and_then(|v| v.as_object()) else {
        return false;
    };
    let Some(em) = mcp.get(tool_id).and_then(|v| v.as_object()) else {
        return false;
    };
    let Some(command) = em.get("command").and_then(|v| v.as_str()) else {
        return false;
    };
    if command.is_empty() {
        return false;
    }
    if em.get("disabled").and_then(|v| v.as_bool()) == Some(true) {
        return false;
    }
    if let Some(a) = em.get("args") {
        let Some(args) = a.as_array() else {
            return false;
        };
        if !all_strings_nonempty(args) {
            return false;
        }
    }
    true
}

pub fn cline_mcp_matches(tool_id: &str, want: &[String]) -> bool {
    let p = paths::cline_paths_resolved().mcp_config;
    let Some(raw) = paths::read_file_safe(&p) else {
        return false;
    };
    let Some(Value::Object(cfg)) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(mcp) = cfg.get("mcpServers").and_then(|v| v.as_object()) else {
        return false;
    };
    let Some(entry) = mcp.get(tool_id) else {
        return false;
    };
    if !cline_mcp_configured(tool_id) {
        return false;
    }
    let Some(em) = entry.as_object() else {
        return false;
    };
    if want.is_empty() {
        return false;
    }
    let Some(command) = em.get("command").and_then(|v| v.as_str()) else {
        return false;
    };
    if command != want[0] {
        return false;
    }
    let Some(a) = em.get("args") else {
        return want.len() == 1;
    };
    any_string_slice_equal(Some(a), &want[1..])
}

fn cline_expected_command(tool_id: &str, command: &[String]) -> bool {
    if command.len() < 2 || !kilo_is_tokless_command(&command[0]) || command[1] != "run-mcp" {
        return false;
    }
    match tool_id {
        "context-mode" => {
            command.len() >= 4
                && command[2] == "--context-mode"
                && kilo_context_server(&command[3..])
        }
        "codegraph" => {
            command.len() >= 7
                && command[2] == "--agent"
                && command[3] == "cline"
                && kilo_codegraph_server(&command[4..])
        }
        "projectmem" => {
            command.len() >= 5
                && command[2] == "--tool"
                && command[3] == "projectmem"
                && kilo_projectmem_server(&command[4..])
        }
        _ => false,
    }
}

fn cline_state_path() -> String {
    Path::new(&paths::cline_paths_resolved().dir)
        .join(".tokless-cline-state.json")
        .to_string_lossy()
        .into_owned()
}

fn cline_state_read() -> Result<(HashMap<String, Vec<String>>, bool), String> {
    let (state, exists, _) = cline_state_read_raw()?;
    Ok((state, exists))
}

fn cline_state_read_raw() -> Result<(HashMap<String, Vec<String>>, bool, String), String> {
    let Some(raw) = paths::read_file_safe(&cline_state_path()) else {
        return Ok((HashMap::new(), false, String::new()));
    };
    let state: HashMap<String, Vec<String>> = serde_json::from_str(&raw).map_err(|e| {
        format!(
            "Cline Tokless ownership state is unreadable; refusing to modify existing MCP entries: {e}"
        )
    })?;
    for (id, command) in &state {
        if command.is_empty() || !cline_expected_command(id, command) {
            return Err(format!(
                "Cline Tokless ownership state is malformed for {id:?}; refusing to modify existing MCP entries"
            ));
        }
    }
    Ok((state, true, raw))
}

fn state_map_to_json(state: &HashMap<String, Vec<String>>) -> String {
    let mut keys: Vec<&String> = state.keys().collect();
    keys.sort();
    let mut map = OrderedMap::new();
    for k in keys {
        if let Some(cmds) = state.get(k) {
            map.insert(k.clone(), Value::Array(strings_to_values(cmds)));
        }
    }
    stringify_json(&Value::Object(map))
}

fn cline_state_set(id: &str, command: &[String]) -> Result<(), String> {
    let (mut state, exists, raw) = cline_state_read_raw()?;
    state.insert(id.to_string(), command.to_vec());
    let content = state_map_to_json(&state);
    cline_write_file_guarded(&cline_state_path(), &content, &raw, exists)
}

fn cline_state_delete(id: &str) -> Result<(), String> {
    let (mut state, exists, raw) = cline_state_read_raw()?;
    if !exists {
        return Ok(());
    }
    state.remove(id);
    if state.is_empty() {
        return match fs::remove_file(cline_state_path()) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        };
    }
    let content = state_map_to_json(&state);
    cline_write_file_guarded(&cline_state_path(), &content, &raw, exists)
}

pub(crate) fn cline_write_file_guarded(
    path: &str,
    content: &str,
    expected_raw: &str,
    expected_exists: bool,
) -> Result<(), String> {
    let parent = Path::new(path).parent().unwrap_or_else(|| Path::new("."));
    paths::ensure_dir(parent).map_err(|e| e.to_string())?;
    let current_opt = paths::read_file_safe(path);
    let exists = current_opt.is_some();
    let current = current_opt.unwrap_or_default();
    if exists != expected_exists || (exists && current != expected_raw) {
        return Err(format!("Cline file {path} changed during update; retry"));
    }
    #[cfg(unix)]
    let mode = fs::metadata(path)
        .ok()
        .map(|m| {
            use std::os::unix::fs::PermissionsExt;
            m.permissions().mode() & 0o777
        })
        .unwrap_or(0o644);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp_path = parent
        .join(format!(".tokless-cline-{}-{nanos}", std::process::id()))
        .to_string_lossy()
        .into_owned();

    let write_res = (|| -> Result<(), String> {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
            .map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            f.set_permissions(fs::Permissions::from_mode(mode))
                .map_err(|e| e.to_string())?;
        }
        f.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
        f.flush().map_err(|e| e.to_string())?;
        Ok(())
    })();

    if let Err(err) = write_res {
        let _ = fs::remove_file(&tmp_path);
        return Err(err);
    }

    let res = replace_kilo_file(&tmp_path, path);
    if res.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    res
}

fn cline_managed_mcp_entry(v: &Value, command: &[String]) -> bool {
    let Some(m) = v.as_object() else {
        return false;
    };
    if command.is_empty() {
        return false;
    }
    if m.get("command").and_then(|v| v.as_str()) != Some(command[0].as_str()) {
        return false;
    }
    if !any_string_slice_equal(m.get("args"), &command[1..]) {
        return false;
    }
    let Some(em) = m.get("env").and_then(|v| v.as_object()) else {
        return false;
    };
    if !em.is_empty() {
        return false;
    }
    m.get("disabled").and_then(|v| v.as_bool()) == Some(false)
}

// --- Cline Proxy ---

pub fn cline_providers_file() -> String {
    Path::new(&paths::cline_paths_resolved().data_dir)
        .join("settings")
        .join("providers.json")
        .to_string_lossy()
        .into_owned()
}

fn cline_provider_state_file() -> String {
    Path::new(&paths::tokless_data_dir())
        .join("cline-provider-prev")
        .to_string_lossy()
        .into_owned()
}

fn cline_desired_provider() -> OrderedMap {
    let mut env = OrderedMap::new();
    env.insert(
        "provider".to_string(),
        Value::String(CLINE_PROVIDER_NAME.to_string()),
    );
    env.insert("apiKey".to_string(), Value::String(proxy_wire_key()));
    env.insert(
        "model".to_string(),
        Value::String("deepseek-v4-flash".to_string()),
    );
    env.insert(
        "baseUrl".to_string(),
        Value::String(proxy_endpoint_for("cline")),
    );
    let mut m = OrderedMap::new();
    m.insert("settings".to_string(), Value::Object(env));
    m
}

fn cline_routed_provider(
    existing: &OrderedMap,
    register: bool,
) -> (Option<OrderedMap>, bool, Option<BYOKRoute>) {
    let Some(settings) = existing.get("settings").and_then(|v| v.as_object()) else {
        return (None, false, None);
    };
    let Some(base) = settings.get("baseUrl").and_then(|v| v.as_str()) else {
        return (None, false, None);
    };
    if !is_absolute_http(base) || same_proxy_base(base, &proxy_endpoint_for("cline")) {
        return (None, false, None);
    }
    let Ok(Value::Object(mut cloned)) = parse_jsonc(&stringify_json(&Value::Object(existing.clone())))
    else {
        return (None, false, None);
    };
    let Some(routed) = cloned.get_mut("settings").and_then(|v| v.as_object_mut()) else {
        return (None, false, None);
    };
    let mut headers = if let Some(value) = routed.get("headers") {
        let Some(h) = value.as_object() else {
            return (None, false, None);
        };
        h.clone()
    } else {
        OrderedMap::new()
    };

    let upstream = normalized_headroom_upstream(base, "openai-completions");
    let mut previous_route: Option<BYOKRoute> = None;

    if register && upstream.starts_with("https://") {
        let (_, _, prev, ok) = register_byok_route(
            "cline",
            CLINE_PROVIDER_NAME,
            "openai-completions",
            &upstream,
        );
        if !ok {
            return (None, false, None);
        }
        previous_route = Some(prev);
    } else if register {
        let (_, _, prev, ok) = register_byok_route(
            "cline",
            CLINE_PROVIDER_NAME,
            "openai-completions",
            &upstream,
        );
        if ok {
            previous_route = Some(prev);
        }
    }

    headers.insert(
        HEADROOM_BASE_URL_HEADER.to_string(),
        Value::String(upstream),
    );
    routed.insert(
        "provider".to_string(),
        Value::String(CLINE_PROVIDER_NAME.to_string()),
    );
    routed.insert(
        "baseUrl".to_string(),
        Value::String(proxy_endpoint_for("cline")),
    );
    routed.insert("headers".to_string(), Value::Object(headers));
    (Some(cloned), true, previous_route)
}

fn cline_provider_config(raw: &str) -> Result<OrderedMap, String> {
    let Value::Object(mut cfg) = parse_jsonc(raw).map_err(|e| e.to_string())? else {
        return Err("invalid config".to_string());
    };
    if let Some(v) = cfg.get("providers") {
        if !v.is_object() {
            return Err("invalid providers".to_string());
        }
    } else {
        cfg.insert("providers".to_string(), Value::Object(OrderedMap::new()));
    }
    Ok(cfg)
}

fn cline_managed_values(v: &Value) -> bool {
    let Some(m) = v.as_object() else {
        return false;
    };
    let Some(settings) = m.get("settings").and_then(|v| v.as_object()) else {
        return false;
    };
    if settings.get("provider").and_then(|v| v.as_str()) != Some(CLINE_PROVIDER_NAME) {
        return false;
    }
    if settings.get("baseUrl").and_then(|v| v.as_str()) != Some(&proxy_endpoint_for("cline")) {
        return false;
    }
    let Some(model) = settings.get("model") else {
        return false;
    };
    if model.as_str() == Some("") {
        return false;
    }
    if let Some(headers) = settings.get("headers") {
        let Some(h) = headers.as_object() else {
            return false;
        };
        if let Some(value) = h.get(HEADROOM_BASE_URL_HEADER) {
            return value.as_str().is_some_and(|s| !s.is_empty());
        }
    }
    model.as_str() == Some("deepseek-v4-flash")
}

pub fn configure_cline_proxy() -> (bool, String) {
    let mut changed = false;
    let mut file = cline_providers_file();
    if let Err(err) = with_proxy_route_stash_lock(|| {
        let (c, f) = configure_cline_proxy_locked();
        changed = c;
        file = f;
        Ok(())
    }) {
        L.err(&format!("cline proxy lock failed: {err}"));
    }
    (changed, file)
}

fn configure_cline_proxy_locked() -> (bool, String) {
    let file = cline_providers_file();
    let raw_opt = paths::read_file_safe(&file);
    let exists = raw_opt.is_some();
    if !exists && paths::exists(&file) {
        return (false, file);
    }
    let raw = raw_opt.unwrap_or_default();
    let state_path = cline_provider_state_file();
    let state_raw_opt = paths::read_file_safe(&state_path);
    let state_exists = state_raw_opt.is_some();
    let state_raw = state_raw_opt.unwrap_or_default();
    let mut state_created = false;

    let Ok(mut cfg) = cline_provider_config(&raw) else {
        return (false, file);
    };
    if let Some(selected) = cfg.get("lastUsedProvider")
        && selected.as_str() != Some(CLINE_PROVIDER_NAME)
    {
        return (false, file);
    }

    let desired = Value::Object(cline_desired_provider());
    let mut changed = false;
    let mut state_content = String::new();
    let mut rollback_prev: Option<BYOKRoute> = None;

    let existing_opt = cfg
        .get("providers")
        .and_then(|v| v.as_object())
        .and_then(|p| p.get(CLINE_PROVIDER_NAME))
        .cloned();

    let rollback = |prev: &Option<BYOKRoute>| {
        if let Some(p) = prev {
            rollback_byok_route_logged("cline", CLINE_PROVIDER_NAME, p);
        }
    };

    if let Some(existing) = existing_opt {
        let Some(existing_map) = existing.as_object() else {
            return (false, file);
        };
        let routed_val = if cline_managed_values(&existing) {
            if !state_exists {
                return (false, file);
            }
            existing.clone()
        } else {
            let (routed_opt, routed_ok, prev_route) = cline_routed_provider(existing_map, true);
            rollback_prev = prev_route;
            let Some(r) = routed_opt else {
                rollback(&rollback_prev);
                return (false, file);
            };
            if !routed_ok {
                rollback(&rollback_prev);
                return (false, file);
            }
            Value::Object(r)
        };

        if !state_exists {
            let mut state = OrderedMap::new();
            state.insert("provider".to_string(), existing.clone());
            if let Some(v) = cfg.get("lastUsedProvider") {
                state.insert("lastUsedProvider".to_string(), v.clone());
            } else {
                state.insert("lastUsedProvider".to_string(), Value::Null);
            }
            state_content = stringify_json(&Value::Object(state));
            if cline_write_file_guarded(&state_path, &state_content, &state_raw, state_exists)
                .is_err()
            {
                rollback(&rollback_prev);
                return (false, file);
            }
            state_created = true;
            changed = true;
        }

        let Some(current_state_raw) = paths::read_file_safe(&state_path) else {
            rollback(&rollback_prev);
            return (false, file);
        };
        if !cline_state_matches_route(&current_state_raw, &existing, &routed_val) {
            rollback(&rollback_prev);
            return (false, file);
        }
        if !json_equal(&existing, &routed_val) {
            if let Some(providers) = cfg.get_mut("providers").and_then(|v| v.as_object_mut()) {
                providers.insert(CLINE_PROVIDER_NAME.to_string(), routed_val);
            }
            changed = true;
        }
    } else {
        let mut state = OrderedMap::new();
        state.insert("provider".to_string(), Value::Null);
        if let Some(v) = cfg.get("lastUsedProvider") {
            state.insert("lastUsedProvider".to_string(), v.clone());
        } else {
            state.insert("lastUsedProvider".to_string(), Value::Null);
        }
        state_content = stringify_json(&Value::Object(state));
        if cline_write_file_guarded(&state_path, &state_content, &state_raw, state_exists).is_err()
        {
            return (false, file);
        }
        state_created = true;
        if let Some(providers) = cfg.get_mut("providers").and_then(|v| v.as_object_mut()) {
            providers.insert(CLINE_PROVIDER_NAME.to_string(), desired);
        }
        changed = true;
    }

    if cfg.get("lastUsedProvider").and_then(|v| v.as_str()) != Some(CLINE_PROVIDER_NAME) {
        cfg.insert(
            "lastUsedProvider".to_string(),
            Value::String(CLINE_PROVIDER_NAME.to_string()),
        );
        changed = true;
    }
    if !changed {
        rollback(&rollback_prev);
        return (false, file);
    }
    if cline_write_file_guarded(&file, &stringify_json(&Value::Object(cfg)), &raw, exists).is_err()
    {
        if state_created
            && let Some(current) = paths::read_file_safe(&state_path)
            && current == state_content
        {
            let _ = fs::remove_file(&state_path);
        }
        rollback(&rollback_prev);
        return (false, file);
    }
    (true, file)
}

fn cline_state_matches_route(raw: &str, original: &Value, current: &Value) -> bool {
    let Ok(Value::Object(state)) = parse_jsonc(raw) else {
        return false;
    };
    let Some(original_value) = state.get("provider") else {
        return cline_managed_values(current);
    };
    if original_value.is_null() {
        return cline_managed_values(current);
    }
    let Some(original_map) = original_value.as_object() else {
        return false;
    };
    if !json_equal(original_value, original) {
        return false;
    }
    let (routed_opt, ok, _) = cline_routed_provider(original_map, false);
    let Some(routed) = routed_opt else {
        return false;
    };
    ok && json_equal(&Value::Object(routed), current)
}

fn cline_state_owns_current(raw: &str, current: &Value) -> bool {
    let Ok(Value::Object(state)) = parse_jsonc(raw) else {
        return false;
    };
    let Some(original_value) = state.get("provider") else {
        return cline_managed_values(current);
    };
    if original_value.is_null() {
        return cline_managed_values(current);
    }
    let Some(original_map) = original_value.as_object() else {
        return false;
    };
    let (routed_opt, ok, _) = cline_routed_provider(original_map, false);
    let Some(routed) = routed_opt else {
        return false;
    };
    ok && json_equal(&Value::Object(routed), current)
}

pub fn remove_cline_proxy() -> bool {
    let mut removed = false;
    if let Err(err) = with_proxy_route_stash_lock(|| {
        removed = remove_cline_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("cline proxy lock failed: {err}"));
    }
    removed
}

fn remove_cline_proxy_locked() -> bool {
    let file = cline_providers_file();
    let Some(raw) = paths::read_file_safe(&file) else {
        return false;
    };
    let Ok(mut cfg) = cline_provider_config(&raw) else {
        return false;
    };
    let Some(existing) = cfg
        .get("providers")
        .and_then(|v| v.as_object())
        .and_then(|p| p.get(CLINE_PROVIDER_NAME))
        .cloned()
    else {
        return false;
    };
    if !cline_managed_values(&existing) {
        return false;
    }
    let Some(state_raw) = paths::read_file_safe(&cline_provider_state_file()) else {
        return false;
    };
    if !cline_state_owns_current(&state_raw, &existing) {
        return false;
    }
    let Ok(Value::Object(state)) = parse_jsonc(&state_raw) else {
        return false;
    };
    if let Some(providers) = cfg.get_mut("providers").and_then(|v| v.as_object_mut()) {
        if let Some(v) = state.get("provider")
            && !v.is_null()
        {
            providers.insert(CLINE_PROVIDER_NAME.to_string(), v.clone());
        } else {
            providers.remove(CLINE_PROVIDER_NAME);
        }
    }
    if let Some(v) = state.get("lastUsedProvider")
        && !v.is_null()
    {
        cfg.insert("lastUsedProvider".to_string(), v.clone());
    } else {
        cfg.remove("lastUsedProvider");
    }

    let prev_route = read_byok_route(&format!("cline:{CLINE_PROVIDER_NAME}"));
    if delete_byok_route("cline", CLINE_PROVIDER_NAME).is_err() {
        return false;
    }
    let updated_content = stringify_json(&Value::Object(cfg));
    if cline_write_file_guarded(&file, &updated_content, &raw, true).is_err() {
        if let Some(prev) = prev_route {
            restore_deleted_byok_routes(&[prev]);
        }
        return false;
    }
    if let Err(err) = fs::remove_file(cline_provider_state_file())
        && err.kind() != std::io::ErrorKind::NotFound
    {
        let _ = cline_write_file_guarded(&file, &raw, &updated_content, true);
        if let Some(prev) = prev_route {
            restore_deleted_byok_routes(&[prev]);
        }
        return false;
    }
    true
}

pub fn cline_proxy_wired() -> bool {
    let Some(raw) = paths::read_file_safe(&cline_providers_file()) else {
        return false;
    };
    let Ok(cfg) = cline_provider_config(&raw) else {
        return false;
    };
    let Some(v) = cfg
        .get("providers")
        .and_then(|v| v.as_object())
        .and_then(|p| p.get(CLINE_PROVIDER_NAME))
    else {
        return false;
    };
    cline_managed_values(v)
}

pub(crate) fn detect_cline_proxy(cap: &ProxyCapability) -> ProxyDetection {
    let raw = match fs::read_to_string(cline_providers_file()) {
        Ok(r) => r,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                return proxy_detection(&cap.id, "providers file absent", ProxyState::ProxyStateAbsent);
            }
            return proxy_detection(
                &cap.id,
                "providers unreadable",
                ProxyState::ProxyStateUnreadable,
            );
        }
    };
    let Ok(cfg) = cline_provider_config(&raw) else {
        return proxy_detection(
            &cap.id,
            "providers unreadable",
            ProxyState::ProxyStateUnreadable,
        );
    };
    let Some(providers) = cfg.get("providers").and_then(|v| v.as_object()) else {
        return proxy_detection(
            &cap.id,
            "providers unreadable",
            ProxyState::ProxyStateUnreadable,
        );
    };
    let Some(v) = providers.get(CLINE_PROVIDER_NAME) else {
        return proxy_detection(
            &cap.id,
            "reserved provider absent",
            ProxyState::ProxyStateNativeUnconfigured,
        );
    };
    if cline_managed_values(v) {
        return proxy_detection(
            &cap.id,
            "exact managed provider",
            ProxyState::ProxyStateManaged,
        );
    }
    proxy_detection(
        &cap.id,
        "reserved provider differs",
        ProxyState::ProxyStateConflict,
    )
}
