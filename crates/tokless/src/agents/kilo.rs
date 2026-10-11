use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::AgentManifest;
use crate::util::byok::{BYOKRoute, byok_route_header, read_byok_route, read_byok_routes};
use crate::util::jsonc::{OrderedMap, Value, stringify_json};
use crate::util::logger::L;
use crate::util::paths;

use super::opencode::{
    OpenCodeBYOK, TOKLESS_OPENCODE_BYOK_PLUGIN, open_code_proxy_provider_block,
};
use super::proxy_detection::{ProxyConfigState, classify_open_code_jsonc_proxy};
use super::proxy_protocol::proxy_endpoint_for;
use super::proxy_provider::{
    ByokRouteRollback, ProxyRouteStashEntry, all_strings_nonempty, any_string_slice_equal,
    delete_byok_route, detect_agent_with, has_jsonc_comment, is_valid_byok_upstream, json_equal,
    json_get_or_create_map, json_map_child, json_map_child_mut, kilo_codegraph_server,
    kilo_context_server, kilo_is_tokless_command, kilo_projectmem_server,
    load_proxy_route_stash_locked, normalized_headroom_upstream, open_code_protocol,
    provider_base_and_key, provider_spec_active, proxy_endpoint_for_api, proxy_route_stash_path,
    proxy_route_stash_valid, register_byok_route, replace_kilo_file, restore_deleted_byok_routes,
    restore_proxy_route_stash_logged, rollback_byok_route_logged, same_proxy_base,
    save_proxy_route_stash, string_slices_equal, strings_to_values,
    try_parse_jsonc_map as try_parse_jsonc, with_proxy_route_stash_lock,
};

pub const KILO_PROXY_PROVIDER: &str = "tokless-headroom";

pub fn kilo_manifest() -> AgentManifest {
    AgentManifest {
        id: "kilo".to_string(),
        label: "Kilo CLI".to_string(),
        homepage: "https://kilo.ai/docs/cli".to_string(),
        cli_bin: "kilo".to_string(),
        config_dir: Some(Arc::new(|| paths::kilo_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            detect_agent_with("kilo", "", &paths::expected_bin_dirs(), &[])
        })),
    }
}

pub fn kilo_project_file(sub: &str, name: &str) -> String {
    let cwd = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let mut cur = cwd.clone();
    let root = loop {
        if paths::exists(cur.join(".git")) {
            break cur;
        }
        let Some(parent) = cur.parent().map(|p| p.to_path_buf()) else {
            break cwd;
        };
        if parent == cur {
            break cwd;
        }
        cur = parent;
    };
    root.join(".kilo")
        .join(sub)
        .join(name)
        .to_string_lossy()
        .into_owned()
}

pub fn kilo_instructions_path() -> String {
    paths::kilo_paths_resolved().instructions
}

pub fn configure_kilo_mcp_safe(
    tool_id: &str,
    command: &[String],
) -> Result<(bool, String), String> {
    let p = paths::kilo_paths_resolved().config;
    if tool_id == "headroom" {
        return Err("Headroom is HTTP proxy-only; MCP wiring is disabled".to_string());
    }
    if !kilo_expected_command(tool_id, command) {
        return Err(format!(
            "unrecognized Tokless MCP command for Kilo tool {tool_id:?}; refusing to adopt or overwrite entry"
        ));
    }
    let raw_opt = paths::read_file_safe(&p);
    let exists = raw_opt.is_some();
    let raw = raw_opt.unwrap_or_default();
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(m) => m,
        None => {
            if exists && !raw.is_empty() {
                return Err(format!(
                    "cannot parse Kilo config {p}; refusing to overwrite it"
                ));
            }
            OrderedMap::new()
        }
    };
    if cfg.get("mcp").is_some_and(|v| !v.is_object()) {
        return Err(
            "Kilo config field \"mcp\" is not an object; refusing to overwrite it".to_string(),
        );
    }
    if !cfg.contains_key("mcp") {
        cfg.insert("mcp".to_string(), Value::Object(OrderedMap::new()));
    }
    let mcp = json_map_child_mut(&mut cfg, "mcp").unwrap();
    let (state, state_exists) = kilo_state_read()?;
    let had_ledger = state.contains_key(tool_id);
    if let Some(existing) = mcp.get(tool_id) {
        let Some(owned) = state.get(tool_id) else {
            return Err(format!(
                "Kilo MCP {tool_id:?} has no Tokless ownership state; refusing same-name adoption"
            ));
        };
        if !state_exists {
            return Err(format!(
                "Kilo MCP {tool_id:?} has no Tokless ownership state; refusing same-name adoption"
            ));
        }
        if !kilo_managed_mcp_entry(existing, owned) || !kilo_expected_command(tool_id, owned) {
            return Err(format!(
                "Kilo MCP {tool_id:?} differs from Tokless ownership state; restore the owned entry or remove it manually"
            ));
        }
        if kilo_managed_mcp_entry(existing, command) {
            return Ok((false, p));
        }
    }
    if state_exists
        && let Some(owned) = state.get(tool_id)
        && !string_slices_equal(owned, command)
    {
        return Err(format!(
            "Kilo MCP {tool_id:?} ownership state differs from requested command"
        ));
    }
    if exists && has_jsonc_comment(&raw) {
        return Err(format!(
            "Kilo config {p} contains JSONC comments; edit it without relocating comments before wiring Tokless"
        ));
    }
    let mut desired = OrderedMap::new();
    desired.insert("type".to_string(), Value::String("local".to_string()));
    desired.insert(
        "command".to_string(),
        Value::Array(strings_to_values(command)),
    );
    desired.insert("enabled".to_string(), Value::Bool(true));
    mcp.insert(tool_id.to_string(), Value::Object(desired));
    let content = stringify_json(&Value::Object(cfg));
    kilo_state_set(tool_id, command)?;
    if let Err(err) = kilo_write_file_guarded(&p, &content, &raw, exists) {
        if !had_ledger {
            let _ = kilo_state_delete(tool_id);
        }
        return Err(err);
    }
    Ok((true, p))
}

pub fn remove_kilo_mcp(tool_id: &str) -> bool {
    let p = paths::kilo_paths_resolved().config;
    let Ok((state, state_exists)) = kilo_state_read() else {
        return false;
    };
    if !state_exists {
        return false;
    }
    let Some(owned_cmd) = state.get(tool_id) else {
        return false;
    };
    let Some(raw) = paths::read_file_safe(&p) else {
        return kilo_state_delete(tool_id).is_ok();
    };
    if has_jsonc_comment(&raw) {
        return false;
    }
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(mcp) = json_map_child_mut(&mut cfg, "mcp") else {
        return kilo_state_delete(tool_id).is_ok();
    };
    let Some(entry) = mcp.get(tool_id) else {
        return kilo_state_delete(tool_id).is_ok();
    };
    if !kilo_managed_mcp_entry(entry, owned_cmd) {
        return false;
    }
    mcp.shift_remove(tool_id);
    if mcp.is_empty() {
        cfg.shift_remove("mcp");
    }
    if kilo_write_file_guarded(&p, &stringify_json(&Value::Object(cfg)), &raw, true).is_err() {
        return false;
    }
    kilo_state_delete(tool_id).is_ok()
}

pub fn kilo_mcp_configured(tool_id: &str) -> bool {
    let p = paths::kilo_paths_resolved().config;
    let Some(raw) = paths::read_file_safe(&p) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(mcp) = json_map_child(&cfg, "mcp") else {
        return false;
    };
    let Some(em) = mcp.get(tool_id).and_then(|v| v.as_object()) else {
        return false;
    };
    if em.get("enabled").and_then(|v| v.as_bool()) == Some(false) {
        return false;
    }
    match em.get("type").and_then(|v| v.as_str()) {
        Some("remote") => em
            .get("url")
            .and_then(|v| v.as_str())
            .is_some_and(|s| !s.is_empty()),
        Some("local" | "") | None => em
            .get("command")
            .and_then(|v| v.as_array())
            .is_some_and(|arr| !arr.is_empty() && all_strings_nonempty(arr)),
        _ => false,
    }
}

pub fn kilo_mcp_matches(tool_id: &str, want: &[String]) -> bool {
    let p = paths::kilo_paths_resolved().config;
    let Some(raw) = paths::read_file_safe(&p) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(mcp) = json_map_child(&cfg, "mcp") else {
        return false;
    };
    let Some(em) = mcp.get(tool_id).and_then(|v| v.as_object()) else {
        return false;
    };
    kilo_mcp_configured(tool_id) && any_string_slice_equal(em.get("command"), want)
}

fn kilo_expected_command(tool_id: &str, command: &[String]) -> bool {
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
                && command[3] == "kilo"
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

fn kilo_state_path() -> String {
    Path::new(&paths::kilo_paths_resolved().dir)
        .join(".tokless-kilo-state.json")
        .to_string_lossy()
        .into_owned()
}

fn kilo_state_read() -> Result<(HashMap<String, Vec<String>>, bool), String> {
    let (state, exists, _) = kilo_state_read_raw()?;
    Ok((state, exists))
}

fn kilo_state_read_raw() -> Result<(HashMap<String, Vec<String>>, bool, String), String> {
    let Some(raw) = paths::read_file_safe(&kilo_state_path()) else {
        return Ok((HashMap::new(), false, String::new()));
    };
    let state: HashMap<String, Vec<String>> = serde_json::from_str(&raw).map_err(|e| {
        format!(
            "Kilo Tokless ownership state is unreadable; refusing to modify existing MCP entries: {e}"
        )
    })?;
    for (id, command) in &state {
        if command.is_empty() || !kilo_expected_command(id, command) {
            return Err(format!(
                "Kilo Tokless ownership state is malformed for {id:?}; refusing to modify existing MCP entries"
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

fn kilo_state_set(id: &str, command: &[String]) -> Result<(), String> {
    let (mut state, exists, raw) = kilo_state_read_raw()?;
    state.insert(id.to_string(), command.to_vec());
    let content = state_map_to_json(&state);
    kilo_write_file_guarded(&kilo_state_path(), &content, &raw, exists)
}

fn kilo_state_delete(id: &str) -> Result<(), String> {
    let (mut state, exists, raw) = kilo_state_read_raw()?;
    if !exists {
        return Ok(());
    }
    state.remove(id);
    if state.is_empty() {
        return match fs::remove_file(kilo_state_path()) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        };
    }
    let content = state_map_to_json(&state);
    kilo_write_file_guarded(&kilo_state_path(), &content, &raw, exists)
}

pub(crate) fn kilo_write_file_guarded(
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
        return Err(format!("Kilo file {path} changed during update; retry"));
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
        .join(format!(".tokless-kilo-{}-{nanos}", std::process::id()))
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

fn kilo_managed_mcp_entry(v: &Value, command: &[String]) -> bool {
    let Some(m) = v.as_object() else {
        return false;
    };
    if m.len() != 3 {
        return false;
    }
    if m.get("type").and_then(|v| v.as_str()) != Some("local") {
        return false;
    }
    if m.get("enabled").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }
    any_string_slice_equal(m.get("command"), command)
}

// --- kilo_proxy.go ---

pub fn discover_kilo_byok() -> Vec<OpenCodeBYOK> {
    let file = paths::kilo_paths_resolved().config;
    let Some(raw) = paths::read_file_safe(&file) else {
        return Vec::new();
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return Vec::new();
    };
    let Some(pm) = json_map_child(&cfg, "provider") else {
        return Vec::new();
    };
    let active_key = provider_spec_active().key;
    let stash = load_proxy_route_stash_locked("kilo");
    let mut out = Vec::new();
    for (id, entry_val) in pm {
        if id == KILO_PROXY_PROVIDER || id == &active_key {
            continue;
        }
        let Some(m) = entry_val.as_object() else {
            continue;
        };
        let npm = m
            .get("npm")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let protocol = open_code_protocol(&npm);
        let (mut base, key) = provider_base_and_key(m);
        if let Some(stashed) = stash.get(id)
            && !stashed.base_url.is_empty()
        {
            base = stashed.base_url.clone();
        }
        if base.is_empty() || same_proxy_base(&base, &proxy_endpoint_for("kilo")) {
            continue;
        }
        if !is_valid_byok_upstream(&base) {
            continue;
        }
        let route_header = read_byok_route(&format!("kilo:{id}"))
            .map(|r| byok_route_header(&r))
            .unwrap_or_default();
        out.push(OpenCodeBYOK {
            id: id.clone(),
            file: file.clone(),
            base_url: base,
            api_key: key,
            npm,
            protocol,
            route_header,
        });
    }
    out
}

pub fn kilo_transport_plugin_target() -> String {
    Path::new(&paths::kilo_paths_resolved().dir)
        .join("plugins")
        .join("tokless-headroom-transport.js")
        .to_string_lossy()
        .into_owned()
}

pub fn kilo_transport_plugin_wired() -> bool {
    paths::read_file_safe(&kilo_transport_plugin_target())
        .is_some_and(|raw| raw.trim() == TOKLESS_OPENCODE_BYOK_PLUGIN.trim())
}

fn ensure_kilo_transport_plugin() -> Result<bool, String> {
    let target = kilo_transport_plugin_target();
    if let Some(raw) = paths::read_file_safe(&target) {
        if raw.trim() == TOKLESS_OPENCODE_BYOK_PLUGIN.trim() {
            return Ok(false);
        }
        if !raw.contains("tokless-opencode-byok-plugin") && !raw.contains("HEADROOM_BYOK_HEADER") {
            return Err(format!("refusing to overwrite foreign Kilo plugin {target}"));
        }
    }
    paths::write_file_atomic(&target, TOKLESS_OPENCODE_BYOK_PLUGIN).map_err(|e| e.to_string())?;
    Ok(true)
}

fn remove_kilo_transport_plugin() -> bool {
    let target = kilo_transport_plugin_target();
    let Some(raw) = paths::read_file_safe(&target) else {
        return false;
    };
    if raw.trim() != TOKLESS_OPENCODE_BYOK_PLUGIN.trim()
        && !raw.contains("tokless-opencode-byok-plugin")
    {
        return false;
    }
    fs::remove_file(&target).is_ok()
}

pub fn configure_kilo_proxy() -> (bool, String) {
    let mut out = (false, paths::kilo_paths_resolved().config);
    if let Err(err) = with_proxy_route_stash_lock(|| {
        out = configure_kilo_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("kilo proxy lock failed: {err}"));
    }
    out
}

fn configure_kilo_proxy_locked() -> (bool, String) {
    let p = paths::kilo_paths_resolved().config;
    let raw_opt = paths::read_file_safe(&p);
    let exists = raw_opt.is_some();
    if !exists && paths::exists(&p) {
        return (false, p);
    }
    let raw = raw_opt.unwrap_or_default();
    if exists && has_jsonc_comment(&raw) {
        return (false, p);
    }
    if !proxy_route_stash_valid("kilo") {
        return (false, p);
    }
    let mut stash = load_proxy_route_stash_locked("kilo");
    let st = classify_open_code_jsonc_proxy(
        &raw,
        KILO_PROXY_PROVIDER,
        &proxy_endpoint_for("kilo"),
        &stash,
        is_kilo_transport_plugin_entry,
    );
    if matches!(st, ProxyConfigState::Unreadable | ProxyConfigState::Conflict) {
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

    let stash_path = proxy_route_stash_path("kilo");
    let prev_stash_raw = paths::read_file_safe(&stash_path);
    let prev_stash_exists = prev_stash_raw.is_some();

    let mut rollbacks: Vec<ByokRouteRollback> = Vec::new();
    let mut changed = false;

    if !cfg.contains_key("$schema") {
        cfg.insert(
            "$schema".to_string(),
            Value::String("https://kilo.ai/config.json".to_string()),
        );
        changed = true;
    }

    let endpoint = proxy_endpoint_for("kilo");
    let provider_map = json_get_or_create_map(&mut cfg, "provider");

    let provider_ids: Vec<String> = provider_map.keys().cloned().collect();
    let active_key = provider_spec_active().key;
    for id in provider_ids {
        if id == KILO_PROXY_PROVIDER || id == active_key {
            continue;
        }
        let Some(entry_val) = provider_map.get(&id).cloned() else {
            continue;
        };
        let Some(mut entry_map) = entry_val.as_object().cloned() else {
            continue;
        };
        let (base, _) = provider_base_and_key(&entry_map);
        if base.is_empty() || same_proxy_base(&base, &endpoint) {
            continue;
        }
        let npm = entry_map
            .get("npm")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let api = match open_code_protocol(&npm).as_str() {
            "anthropic-messages" => "anthropic-messages",
            _ => "openai-completions",
        };
        let upstream = normalized_headroom_upstream(&base, api);
        if !is_valid_byok_upstream(&upstream) {
            for rb in &rollbacks {
                rollback_byok_route_logged("kilo", &rb.id, &rb.previous);
            }
            return (false, p);
        }
        let (_, hdr, prev_route, ok) = register_byok_route("kilo", &id, api, &upstream);
        if !ok {
            for rb in &rollbacks {
                rollback_byok_route_logged("kilo", &rb.id, &rb.previous);
            }
            return (false, p);
        }
        rollbacks.push(ByokRouteRollback {
            id: id.clone(),
            previous: prev_route,
        });

        let had_top_base = entry_map.contains_key("baseURL");
        entry_map.shift_remove("baseURL");
        let opts = json_get_or_create_map(&mut entry_map, "options");
        let base_key = if opts.contains_key("baseUrl") && !opts.contains_key("baseURL") {
            "baseUrl".to_string()
        } else {
            "baseURL".to_string()
        };
        let target_ep = proxy_endpoint_for_api(api);
        opts.insert(base_key.clone(), Value::String(target_ep.clone()));
        let headers = json_get_or_create_map(opts, "headers");
        if let Some((hk, hv)) = hdr.split_once(':') {
            headers.insert(hk.trim().to_string(), Value::String(hv.trim().to_string()));
        }
        let managed_val = Value::Object(entry_map);
        stash.insert(
            id.clone(),
            ProxyRouteStashEntry {
                file: p.clone(),
                provider: id.clone(),
                base_url: base,
                upstream,
                base_key,
                had_base_key: true,
                moved_base: had_top_base,
                had_header: false,
                header: hdr,
                original: serde_json::to_vec(&entry_val).unwrap_or_default(),
                managed: serde_json::to_vec(&managed_val).unwrap_or_default(),
                ..Default::default()
            },
        );
        provider_map.insert(id, managed_val);
        changed = true;
    }

    let desired_block = Value::Object(open_code_proxy_provider_block(&endpoint));
    match provider_map.get(KILO_PROXY_PROVIDER) {
        Some(existing) if json_equal(existing, &desired_block) => {}
        _ => {
            provider_map.insert(KILO_PROXY_PROVIDER.to_string(), desired_block);
            changed = true;
        }
    }

    let plugin_changed = match ensure_kilo_transport_plugin() {
        Ok(c) => c,
        Err(_) => {
            for rb in &rollbacks {
                rollback_byok_route_logged("kilo", &rb.id, &rb.previous);
            }
            return (false, p);
        }
    };

    if save_proxy_route_stash("kilo", &stash).is_err() {
        for rb in &rollbacks {
            rollback_byok_route_logged("kilo", &rb.id, &rb.previous);
        }
        return (false, p);
    }

    if changed {
        let next = stringify_json(&Value::Object(cfg));
        if kilo_write_file_guarded(&p, &next, &raw, exists).is_err() {
            restore_proxy_route_stash_logged(
                "kilo",
                prev_stash_raw.as_deref().unwrap_or(""),
                prev_stash_exists,
            );
            for rb in &rollbacks {
                rollback_byok_route_logged("kilo", &rb.id, &rb.previous);
            }
            return (false, p);
        }
    }

    (changed || plugin_changed, p)
}

pub fn remove_kilo_proxy() -> bool {
    let mut removed = false;
    if let Err(err) = with_proxy_route_stash_lock(|| {
        removed = remove_kilo_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("kilo proxy lock failed: {err}"));
    }
    removed
}

fn remove_kilo_proxy_locked() -> bool {
    let p = paths::kilo_paths_resolved().config;
    if !proxy_route_stash_valid("kilo") {
        return false;
    }
    let mut stash = load_proxy_route_stash_locked("kilo");
    let Some(raw) = paths::read_file_safe(&p) else {
        let plugin_removed = remove_kilo_transport_plugin();
        if !stash.is_empty() {
            for id in stash.keys() {
                let _ = delete_byok_route("kilo", id);
            }
            stash.clear();
            let _ = save_proxy_route_stash("kilo", &stash);
            return true;
        }
        return plugin_removed;
    };
    if has_jsonc_comment(&raw) {
        return false;
    }
    let st = classify_open_code_jsonc_proxy(
        &raw,
        KILO_PROXY_PROVIDER,
        &proxy_endpoint_for("kilo"),
        &stash,
        is_kilo_transport_plugin_entry,
    );
    if st != ProxyConfigState::Managed {
        return false;
    }
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };

    let stash_path = proxy_route_stash_path("kilo");
    let prev_stash_raw = paths::read_file_safe(&stash_path);
    let prev_stash_exists = prev_stash_raw.is_some();
    let existing_routes: Vec<BYOKRoute> = read_byok_routes()
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r.id.starts_with("kilo:"))
        .collect();

    let mut changed = false;
    if let Some(pm) = json_map_child_mut(&mut cfg, "provider") {
        if pm.shift_remove(KILO_PROXY_PROVIDER).is_some() {
            changed = true;
        }
        let active_key = provider_spec_active().key;
        if pm.shift_remove(&active_key).is_some() {
            changed = true;
        }
        let stashed_ids: Vec<String> = stash.keys().cloned().collect();
        for id in stashed_ids {
            let Some(entry) = stash.get(&id) else {
                continue;
            };
            if let Some(orig_val) = serde_json::from_slice::<Value>(&entry.original).ok() {
                pm.insert(id.clone(), orig_val);
                changed = true;
            } else if let Some(m) = pm.get_mut(&id).and_then(|v| v.as_object_mut()) {
                if let Some(opts) = json_map_child_mut(m, "options") {
                    let bk = if entry.base_key.is_empty() {
                        "baseURL"
                    } else {
                        &entry.base_key
                    };
                    opts.insert(bk.to_string(), Value::String(entry.base_url.clone()));
                    if let Some(hdrs) = json_map_child_mut(opts, "headers") {
                        hdrs.shift_remove("x-headroom-byok-route");
                        if hdrs.is_empty() {
                            opts.shift_remove("headers");
                        }
                    }
                }
                changed = true;
            }
            let _ = delete_byok_route("kilo", &id);
            stash.remove(&id);
        }
        if pm.is_empty() {
            cfg.shift_remove("provider");
        }
    }

    if cfg.len() == 1 && cfg.contains_key("$schema") {
        cfg.shift_remove("$schema");
    }

    if save_proxy_route_stash("kilo", &stash).is_err() {
        restore_deleted_byok_routes(&existing_routes);
        return false;
    }

    if changed {
        if cfg.is_empty() {
            if fs::remove_file(&p).is_err() {
                restore_proxy_route_stash_logged(
                    "kilo",
                    prev_stash_raw.as_deref().unwrap_or(""),
                    prev_stash_exists,
                );
                restore_deleted_byok_routes(&existing_routes);
                return false;
            }
        } else {
            let next = stringify_json(&Value::Object(cfg));
            if kilo_write_file_guarded(&p, &next, &raw, true).is_err() {
                restore_proxy_route_stash_logged(
                    "kilo",
                    prev_stash_raw.as_deref().unwrap_or(""),
                    prev_stash_exists,
                );
                restore_deleted_byok_routes(&existing_routes);
                return false;
            }
        }
    }

    let plugin_removed = remove_kilo_transport_plugin();
    changed || plugin_removed
}

pub fn kilo_proxy_wired() -> bool {
    let p = paths::kilo_paths_resolved().config;
    let Some(raw) = paths::read_file_safe(&p) else {
        return false;
    };
    if !proxy_route_stash_valid("kilo") {
        return false;
    }
    let stash = load_proxy_route_stash_locked("kilo");
    classify_open_code_jsonc_proxy(
        &raw,
        KILO_PROXY_PROVIDER,
        &proxy_endpoint_for("kilo"),
        &stash,
        is_kilo_transport_plugin_entry,
    ) == ProxyConfigState::Managed
}

pub(crate) fn is_kilo_transport_plugin_entry(v: &Value) -> bool {
    v.as_str()
        .is_some_and(|s| s.contains("tokless-headroom-transport.js"))
}
