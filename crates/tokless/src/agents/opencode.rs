use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::core::AgentManifest;
use crate::util::byok::{BYOKRoute, byok_route_header, read_byok_route, read_byok_routes};
use crate::util::headroom::headroom_paths_resolved;
use crate::util::jsonc::{OrderedMap, Value, has_jsonc_comments, stringify_json};
use crate::util::logger::L;
use crate::util::mcpspawn::{mcp_spawn_for, pick_mcp_spawn, wrap_auto_index};
use crate::util::paths;

use super::proxy_detection::{ProxyConfigState, classify_open_code_jsonc_proxy};
use super::proxy_protocol::proxy_endpoint_for;
use super::proxy_provider::{
    ByokRouteRollback, ProviderSpec, ProxyRouteStashEntry, delete_byok_route, detect_agent_with,
    is_valid_byok_upstream, json_args_eq, json_equal, json_get_or_create_map, json_map_child,
    json_map_child_mut, load_proxy_route_stash, load_proxy_route_stash_locked,
    normalized_headroom_upstream, open_code_protocol, provider_base_and_key, provider_spec_active,
    proxy_endpoint_for_api, proxy_route_stash_path, proxy_route_stash_valid, proxy_wire_key,
    register_byok_route, restore_deleted_byok_routes, restore_proxy_route_stash_logged,
    rollback_byok_route_logged, same_proxy_base, save_proxy_route_stash, strings_to_values,
    try_parse_jsonc_map as try_parse_jsonc, with_proxy_route_stash_lock,
};

pub const TOKLESS_OPENCODE_BYOK_PLUGIN: &str = r#"// tokless-opencode-byok-plugin: managed by tokless; do not edit by hand.
const HEADROOM_BYOK_HEADER = "x-headroom-byok-route";
const HEADROOM_BASE_URL_HEADER = "x-headroom-base-url";

function headerEntries(headers) {
  if (!headers) return [];
  if (typeof headers.entries === "function") return Array.from(headers.entries());
  if (Array.isArray(headers)) return headers;
  if (typeof headers === "object") return Object.entries(headers);
  return [];
}

function getHeader(headers, name) {
  const lower = String(name || "").toLowerCase();
  for (const [k, v] of headerEntries(headers)) {
    if (String(k).toLowerCase() === lower && v != null) return String(v);
  }
  return "";
}

function isHeadroomGatewayUrl(rawUrl) {
  if (!rawUrl || typeof rawUrl !== "string") return false;
  try {
    const u = new URL(rawUrl);
    if (u.protocol !== "http:" && u.protocol !== "https:") return false;
    const host = (u.hostname || "").toLowerCase();
    return host === "127.0.0.1" || host === "localhost" || host === "::1" || host === "[::1]";
  } catch {
    return false;
  }
}

function wrapFetchWithByokHeader(baseFetch, routeHeaderValue) {
  const f = typeof baseFetch === "function" ? baseFetch : globalThis.fetch;
  if (typeof f !== "function") return f;
  return async function toklessByokFetch(input, init) {
    const reqUrl = typeof input === "string"
      ? input
      : input && typeof input.url === "string"
        ? input.url
        : String(input || "");
    if (!isHeadroomGatewayUrl(reqUrl)) {
      return f(input, init);
    }
    const nextInit = Object.assign({}, init || {});
    const merged = new Headers(
      (input && typeof input === "object" && input.headers) || undefined,
    );
    for (const [k, v] of headerEntries(init && init.headers)) {
      if (v != null) merged.set(k, String(v));
    }
    if (routeHeaderValue && !merged.has(HEADROOM_BYOK_HEADER)) {
      merged.set(HEADROOM_BYOK_HEADER, routeHeaderValue);
    }
    nextInit.headers = merged;
    return f(input, nextInit);
  };
}

export const ToklessHeadroomTransportPlugin = async () => {
  return {
    "chat.params": async (input, output) => {
      try {
        const providerOpts =
          (input && input.provider && input.provider.options) ||
          (input && input.model && input.model.provider && input.model.provider.options) ||
          {};
        const hdrs = providerOpts.headers || {};
        const routeHdr = getHeader(hdrs, HEADROOM_BYOK_HEADER);
        const baseHdr = getHeader(hdrs, HEADROOM_BASE_URL_HEADER);
        if (!routeHdr && !baseHdr) return;
        if (output && typeof output === "object") {
          output.options = Object.assign({}, output.options || {});
          output.options.headers = Object.assign({}, output.options.headers || {}, hdrs);
          if (routeHdr) {
            output.options.fetch = wrapFetchWithByokHeader(output.options.fetch, routeHdr);
          }
        }
      } catch {
        // never break chat invocation on plugin inspection errors
      }
    },
  };
};

export default ToklessHeadroomTransportPlugin;
"#;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenCodeBYOK {
    pub id: String,
    pub file: String,
    pub base_url: String,
    pub api_key: String,
    pub npm: String,
    pub protocol: String,
    pub route_header: String,
}

pub fn opencode_manifest() -> AgentManifest {
    AgentManifest {
        id: "opencode".to_string(),
        label: "OpenCode".to_string(),
        homepage: "https://opencode.ai".to_string(),
        cli_bin: "opencode".to_string(),
        config_dir: Some(Arc::new(|| paths::opencode_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            detect_agent_with(
                "opencode",
                &paths::opencode_paths_resolved().dir,
                &paths::expected_bin_dirs(),
                &[],
            )
        })),
    }
}

pub fn configure_opencode_mcp(tool_id: &str) -> (bool, String) {
    let p = paths::opencode_paths_resolved().config;
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
    if cfg.get("mcp").is_some_and(|v| !v.is_object()) {
        return (false, p);
    }
    if !cfg.contains_key("$schema") {
        cfg.insert(
            "$schema".to_string(),
            Value::String("https://opencode.ai/config.json".to_string()),
        );
    }
    if !cfg.contains_key("mcp") {
        cfg.insert("mcp".to_string(), Value::Object(OrderedMap::new()));
    }
    let desired = Value::Object(opencode_mcp_desired(tool_id));
    let mcp = json_map_child_mut(&mut cfg, "mcp").unwrap();
    if let Some(existing) = mcp.get(tool_id) {
        if json_equal(existing, &desired) {
            return (false, p);
        }
        if !opencode_mcp_managed(tool_id, existing) {
            return (false, p);
        }
    }
    mcp.insert(tool_id.to_string(), desired);
    let next = stringify_json(&Value::Object(cfg));
    if next == raw {
        return (false, p);
    }
    if paths::write_file_atomic(&p, &next).is_err() {
        return (false, p);
    }
    (true, p)
}

pub fn remove_opencode_mcp(tool_id: &str) -> bool {
    let p = paths::opencode_paths_resolved().config;
    let Some(raw) = paths::read_file_safe(&p) else {
        return false;
    };
    if has_jsonc_comments(&raw) {
        return false;
    }
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(mcp) = json_map_child_mut(&mut cfg, "mcp") else {
        return false;
    };
    let Some(existing) = mcp.get(tool_id) else {
        return false;
    };
    if !opencode_mcp_managed(tool_id, existing) {
        return false;
    }
    mcp.shift_remove(tool_id);
    if mcp.is_empty() {
        cfg.shift_remove("mcp");
    }
    let next = stringify_json(&Value::Object(cfg));
    paths::write_file_atomic(&p, &next).is_ok()
}

pub fn opencode_mcp_has(tool_id: &str) -> bool {
    let p = paths::opencode_paths_resolved().config;
    let Some(raw) = paths::read_file_safe(&p) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    json_map_child(&cfg, "mcp").is_some_and(|m| m.contains_key(tool_id))
}

fn opencode_mcp_desired(tool_id: &str) -> OrderedMap {
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index(
            "opencode",
            pick_mcp_spawn("codegraph", &["serve", "--mcp"]),
        )
    } else {
        mcp_spawn_for(tool_id)
    };
    let mut cmd = Vec::with_capacity(1 + spawn.args.len());
    cmd.push(spawn.command);
    cmd.extend(spawn.args);
    let mut desired = OrderedMap::new();
    desired.insert("type".to_string(), Value::String("local".to_string()));
    desired.insert("command".to_string(), Value::Array(strings_to_values(&cmd)));
    desired.insert("enabled".to_string(), Value::Bool(true));
    desired
}

fn opencode_mcp_managed(tool_id: &str, existing: &Value) -> bool {
    let Some(em) = existing.as_object() else {
        return false;
    };
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index(
            "opencode",
            pick_mcp_spawn("codegraph", &["serve", "--mcp"]),
        )
    } else {
        mcp_spawn_for(tool_id)
    };
    let mut cmd = Vec::with_capacity(1 + spawn.args.len());
    cmd.push(spawn.command);
    cmd.extend(spawn.args);
    em.get("type").and_then(|v| v.as_str()) == Some("local")
        && json_args_eq(em.get("command"), &cmd)
}

// --- opencode_proxy.go ---

pub fn open_code_byok_plugin_path() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("opencode-byok-plugin.js")
        .to_string_lossy()
        .into_owned()
}

pub fn open_code_transport_plugin_path() -> String {
    open_code_byok_plugin_path()
}

pub fn is_open_code_transport_plugin_path_entry(v: &Value) -> bool {
    let Some(s) = v.as_str() else {
        return false;
    };
    let normalized = s.replace('\\', "/");
    normalized.ends_with("/opencode-byok-plugin.js")
        || normalized.contains("tokless-headroom-transport.js")
        || s == open_code_byok_plugin_path()
}

pub fn open_code_proxy_provider_block(base_url: &str) -> OrderedMap {
    open_code_proxy_provider_block_for(base_url, &provider_spec_active())
}

pub fn open_code_proxy_provider_block_for(base_url: &str, spec: &ProviderSpec) -> OrderedMap {
    let mut block = OrderedMap::new();
    block.insert("npm".to_string(), Value::String(spec.npm.clone()));
    block.insert("name".to_string(), Value::String(spec.name.clone()));

    let mut options = OrderedMap::new();
    options.insert("baseURL".to_string(), Value::String(base_url.to_string()));
    if !spec.key_env.is_empty() {
        options.insert(
            "apiKey".to_string(),
            Value::String(format!("{{env:{}}}", spec.key_env)),
        );
    } else {
        options.insert("apiKey".to_string(), Value::String(proxy_wire_key()));
    }
    block.insert("options".to_string(), Value::Object(options));

    let mut models = OrderedMap::new();
    for m in &spec.models {
        let mut model_entry = OrderedMap::new();
        model_entry.insert("name".to_string(), Value::String(m.display.clone()));
        if m.context > 0 || m.output > 0 {
            let mut limit = OrderedMap::new();
            if m.context > 0 {
                limit.insert("context".to_string(), Value::from(m.context));
            }
            if m.output > 0 {
                limit.insert("output".to_string(), Value::from(m.output));
            }
            model_entry.insert("limit".to_string(), Value::Object(limit));
        }
        if m.reasoning {
            model_entry.insert("reasoning".to_string(), Value::Bool(true));
        }
        if m.tool_call {
            model_entry.insert("tool_call".to_string(), Value::Bool(true));
        }
        models.insert(m.id.clone(), Value::Object(model_entry));
    }
    block.insert("models".to_string(), Value::Object(models));
    block
}

fn open_code_candidate_files() -> Vec<String> {
    let p = paths::opencode_paths_resolved();
    let mut out = vec![p.config.clone()];
    let jsonc = Path::new(&p.dir)
        .join("opencode.jsonc")
        .to_string_lossy()
        .into_owned();
    if jsonc != p.config && paths::exists(&jsonc) {
        out.push(jsonc);
    }
    out
}

pub fn discover_open_code_byok() -> Vec<OpenCodeBYOK> {
    let active_key = provider_spec_active().key;
    let stash = load_proxy_route_stash("opencode");
    let mut out = Vec::new();
    for file in open_code_candidate_files() {
        let Some(raw) = paths::read_file_safe(&file) else {
            continue;
        };
        let Some(cfg) = try_parse_jsonc(&raw) else {
            continue;
        };
        let Some(pm) = json_map_child(&cfg, "provider") else {
            continue;
        };
        for (id, entry_val) in pm {
            if id == &active_key || id == "tokless-headroom" {
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
            if base.is_empty() || same_proxy_base(&base, &proxy_endpoint_for("opencode")) {
                continue;
            }
            if !is_valid_byok_upstream(&base) {
                continue;
            }
            let route_header = read_byok_route(&format!("opencode:{id}"))
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
    }
    out
}

pub fn discover_opencode_byok() -> Vec<OpenCodeBYOK> {
    discover_open_code_byok()
}

pub fn sync_open_code_byok_routes() -> usize {
    let entries = discover_open_code_byok();
    let mut synced = 0usize;
    for entry in entries {
        let api = if entry.protocol == "anthropic-messages" {
            "anthropic-messages"
        } else {
            "openai-completions"
        };
        let upstream = normalized_headroom_upstream(&entry.base_url, api);
        if !is_valid_byok_upstream(&upstream) {
            continue;
        }
        let (_, _, _, ok) = register_byok_route("opencode", &entry.id, api, &upstream);
        if ok {
            synced += 1;
        }
    }
    synced
}

pub fn sync_opencode_byok_routes() -> usize {
    sync_open_code_byok_routes()
}

fn ensure_open_code_byok_plugin_file() -> Result<String, String> {
    let path = open_code_byok_plugin_path();
    if let Some(raw) = paths::read_file_safe(&path)
        && raw == TOKLESS_OPENCODE_BYOK_PLUGIN
    {
        return Ok(path);
    }
    paths::write_file_atomic(&path, TOKLESS_OPENCODE_BYOK_PLUGIN).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn configure_open_code_proxy() -> (bool, String) {
    let mut out = (false, paths::opencode_paths_resolved().config);
    if let Err(err) = with_proxy_route_stash_lock(|| {
        out = configure_open_code_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("opencode proxy lock failed: {err}"));
    }
    out
}

pub fn configure_opencode_proxy() -> (bool, String) {
    configure_open_code_proxy()
}

fn configure_open_code_proxy_locked() -> (bool, String) {
    let p = paths::opencode_paths_resolved().config;
    let raw_opt = paths::read_file_safe(&p);
    let exists = raw_opt.is_some();
    if !exists && paths::exists(&p) {
        return (false, p);
    }
    let raw = raw_opt.unwrap_or_default();
    if exists && has_jsonc_comments(&raw) {
        return (false, p);
    }
    if !proxy_route_stash_valid("opencode") {
        return (false, p);
    }
    let mut stash = load_proxy_route_stash_locked("opencode");
    let spec = provider_spec_active();
    let endpoint = proxy_endpoint_for("opencode");
    let st = classify_open_code_jsonc_proxy(
        &raw,
        &spec.key,
        &endpoint,
        &stash,
        is_open_code_transport_plugin_path_entry,
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

    let stash_path = proxy_route_stash_path("opencode");
    let prev_stash_raw = paths::read_file_safe(&stash_path);
    let prev_stash_exists = prev_stash_raw.is_some();

    let mut rollbacks: Vec<ByokRouteRollback> = Vec::new();
    let mut changed = false;

    if !cfg.contains_key("$schema") {
        cfg.insert(
            "$schema".to_string(),
            Value::String("https://opencode.ai/config.json".to_string()),
        );
        changed = true;
    }

    let mut need_plugin = false;
    {
        let provider_map = json_get_or_create_map(&mut cfg, "provider");
        let provider_ids: Vec<String> = provider_map.keys().cloned().collect();
        for id in provider_ids {
            if id == spec.key || id == "tokless-headroom" {
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
                if stash.contains_key(&id) {
                    need_plugin = true;
                }
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
                    rollback_byok_route_logged("opencode", &rb.id, &rb.previous);
                }
                return (false, p);
            }
            let (_, hdr, prev_route, ok) = register_byok_route("opencode", &id, api, &upstream);
            if !ok {
                for rb in &rollbacks {
                    rollback_byok_route_logged("opencode", &rb.id, &rb.previous);
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
            opts.insert(base_key.clone(), Value::String(target_ep));
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
                    header: hdr,
                    original: serde_json::to_vec(&entry_val).unwrap_or_default(),
                    managed: serde_json::to_vec(&managed_val).unwrap_or_default(),
                    ..Default::default()
                },
            );
            provider_map.insert(id, managed_val);
            need_plugin = true;
            changed = true;
        }

        let desired_block = Value::Object(open_code_proxy_provider_block_for(&endpoint, &spec));
        match provider_map.get(&spec.key) {
            Some(existing) if json_equal(existing, &desired_block) => {}
            _ => {
                provider_map.insert(spec.key.clone(), desired_block);
                changed = true;
            }
        }
    }

    if need_plugin {
        let Ok(plugin_path) = ensure_open_code_byok_plugin_file() else {
            for rb in &rollbacks {
                rollback_byok_route_logged("opencode", &rb.id, &rb.previous);
            }
            return (false, p);
        };
        let mut plugins = cfg
            .get("plugin")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        if !plugins.iter().any(is_open_code_transport_plugin_path_entry) {
            plugins.push(Value::String(plugin_path));
            cfg.insert("plugin".to_string(), Value::Array(plugins));
            changed = true;
        }
    }

    if save_proxy_route_stash("opencode", &stash).is_err() {
        for rb in &rollbacks {
            rollback_byok_route_logged("opencode", &rb.id, &rb.previous);
        }
        return (false, p);
    }

    if !changed {
        return (false, p);
    }

    let next = stringify_json(&Value::Object(cfg));
    if paths::write_file_atomic(&p, &next).is_err() {
        restore_proxy_route_stash_logged(
            "opencode",
            prev_stash_raw.as_deref().unwrap_or(""),
            prev_stash_exists,
        );
        for rb in &rollbacks {
            rollback_byok_route_logged("opencode", &rb.id, &rb.previous);
        }
        return (false, p);
    }
    (true, p)
}

pub fn remove_open_code_proxy() -> bool {
    let mut removed = false;
    if let Err(err) = with_proxy_route_stash_lock(|| {
        removed = remove_open_code_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("opencode proxy lock failed: {err}"));
    }
    removed
}

pub fn remove_opencode_proxy() -> bool {
    remove_open_code_proxy()
}

fn remove_open_code_proxy_locked() -> bool {
    let p = paths::opencode_paths_resolved().config;
    if !proxy_route_stash_valid("opencode") {
        return false;
    }
    let mut stash = load_proxy_route_stash_locked("opencode");
    let Some(raw) = paths::read_file_safe(&p) else {
        let _ = fs::remove_file(open_code_byok_plugin_path());
        if !stash.is_empty() {
            for id in stash.keys() {
                let _ = delete_byok_route("opencode", id);
            }
            stash.clear();
            let _ = save_proxy_route_stash("opencode", &stash);
            return true;
        }
        return false;
    };
    if has_jsonc_comments(&raw) {
        return false;
    }
    let spec = provider_spec_active();
    let endpoint = proxy_endpoint_for("opencode");
    let st = classify_open_code_jsonc_proxy(
        &raw,
        &spec.key,
        &endpoint,
        &stash,
        is_open_code_transport_plugin_path_entry,
    );
    if st != ProxyConfigState::Managed {
        return false;
    }
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return false;
    };

    let stash_path = proxy_route_stash_path("opencode");
    let prev_stash_raw = paths::read_file_safe(&stash_path);
    let prev_stash_exists = prev_stash_raw.is_some();
    let existing_routes: Vec<BYOKRoute> = read_byok_routes()
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r.id.starts_with("opencode:"))
        .collect();

    let mut changed = false;
    if let Some(pm) = json_map_child_mut(&mut cfg, "provider") {
        if pm.shift_remove(&spec.key).is_some() {
            changed = true;
        }
        if pm.shift_remove("tokless-headroom").is_some() {
            changed = true;
        }
        let stashed_ids: Vec<String> = stash.keys().cloned().collect();
        for id in stashed_ids {
            let Some(entry) = stash.get(&id) else {
                continue;
            };
            if let Ok(orig_val) = serde_json::from_slice::<Value>(&entry.original) {
                pm.insert(id.clone(), orig_val);
                changed = true;
            }
            let _ = delete_byok_route("opencode", &id);
            stash.remove(&id);
        }
        if pm.is_empty() {
            cfg.shift_remove("provider");
        }
    }

    if let Some(plugins) = cfg.get("plugin").and_then(|v| v.as_array()).cloned() {
        let kept: Vec<Value> = plugins
            .into_iter()
            .filter(|v| !is_open_code_transport_plugin_path_entry(v))
            .collect();
        if kept.is_empty() {
            cfg.shift_remove("plugin");
        } else {
            cfg.insert("plugin".to_string(), Value::Array(kept));
        }
        changed = true;
    }

    if cfg.len() == 1 && cfg.contains_key("$schema") {
        cfg.shift_remove("$schema");
    }

    if save_proxy_route_stash("opencode", &stash).is_err() {
        restore_deleted_byok_routes(&existing_routes);
        return false;
    }

    if changed {
        if cfg.is_empty() {
            if fs::remove_file(&p).is_err() {
                restore_proxy_route_stash_logged(
                    "opencode",
                    prev_stash_raw.as_deref().unwrap_or(""),
                    prev_stash_exists,
                );
                restore_deleted_byok_routes(&existing_routes);
                return false;
            }
        } else {
            let next = stringify_json(&Value::Object(cfg));
            if paths::write_file_atomic(&p, &next).is_err() {
                restore_proxy_route_stash_logged(
                    "opencode",
                    prev_stash_raw.as_deref().unwrap_or(""),
                    prev_stash_exists,
                );
                restore_deleted_byok_routes(&existing_routes);
                return false;
            }
        }
    }

    let _ = fs::remove_file(open_code_byok_plugin_path());
    changed
}

pub fn open_code_proxy_wired() -> bool {
    let p = paths::opencode_paths_resolved().config;
    let Some(raw) = paths::read_file_safe(&p) else {
        return false;
    };
    if !proxy_route_stash_valid("opencode") {
        return false;
    }
    let stash = load_proxy_route_stash("opencode");
    let spec = provider_spec_active();
    classify_open_code_jsonc_proxy(
        &raw,
        &spec.key,
        &proxy_endpoint_for("opencode"),
        &stash,
        is_open_code_transport_plugin_path_entry,
    ) == ProxyConfigState::Managed
}

pub fn opencode_proxy_wired() -> bool {
    open_code_proxy_wired()
}

pub fn open_code_proxy_satisfied() -> bool {
    open_code_proxy_wired()
}

pub fn opencode_proxy_satisfied() -> bool {
    open_code_proxy_wired()
}
