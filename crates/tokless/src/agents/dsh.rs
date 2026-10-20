// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/dsh.rs
// created: 2026-10-20
// lastModified: 2026-10-20

use std::fs;
use std::path::Path;
use std::sync::Arc;
use url::Url;

use crate::core::AgentManifest;
use crate::util::byok::{byok_route_header, delete_byok_route, read_byok_route, upsert_byok_route, BYOKRoute};
use crate::util::logger::L;
use crate::util::paths::{ensure_dir, expected_bin_dirs, exists, home, read_file_safe, write_file, write_file_atomic};
use super::proxy_protocol::proxy_endpoint_for;
use super::proxy_provider::{
    byok_route_header_key, detect_agent_with, is_valid_byok_upstream, load_proxy_route_stash,
    load_proxy_route_stash_locked, normalized_headroom_upstream, proxy_route_stash_path,
    proxy_route_stash_valid, register_byok_route, restore_proxy_route_stash_logged,
    rollback_byok_route_logged, route_upstream_to_base_url, same_proxy_base, save_proxy_route_stash,
    with_proxy_route_stash_lock, ByokRouteRollback, ProxyRouteStashEntry,
};

pub const DSH_HEADROOM_BASE_URL_HEADER: &str = "x-headroom-base-url";
pub const DSH_HEADROOM_ORIGINAL_PATH_HDR: &str = "x-headroom-original-path";
pub const DSH_HOOKS_BRIDGE_ENTRY: &str = "hooks-claude-code";
pub const DSH_RTK_PLUGIN_NAME: &str = "tokless-rtk";

pub fn dsh_manifest() -> AgentManifest {
    AgentManifest {
        id: "dsh".to_string(),
        label: "DeepSeek Harness".to_string(),
        homepage: "https://github.com/deepseek-ai/deepseek-harness".to_string(),
        cli_bin: "dsh".to_string(),
        config_dir: Some(Arc::new(dsh_home)),
        detect: Some(Arc::new(|| {
            detect_agent_with("dsh", &dsh_home(), &expected_bin_dirs(), &[])
        })),
    }
}

pub fn dsh_home() -> String {
    let h = std::env::var("DSH_HOME").unwrap_or_default();
    if h == "~" {
        return home();
    }
    if h.starts_with("~/") {
        return Path::new(&home()).join(&h[2..]).to_string_lossy().to_string();
    }
    if !h.is_empty() {
        return h;
    }
    Path::new(&home()).join(".dsh").to_string_lossy().to_string()
}

pub fn dsh_patch_file() -> String {
    Path::new(&dsh_home()).join("cordis.patch.yml").to_string_lossy().to_string()
}

pub fn dsh_instructions_file() -> String {
    Path::new(&dsh_home()).join("AGENTS.md").to_string_lossy().to_string()
}

pub fn dsh_hooks_file() -> String {
    Path::new(&dsh_home()).join("hooks.json").to_string_lossy().to_string()
}

pub fn dsh_rtk_patch_file() -> String {
    dsh_patch_file()
}

pub fn dsh_rtk_plugin_file() -> String {
    Path::new(&dsh_home()).join("plugins").join("tokless-rtk.mjs").to_string_lossy().to_string()
}

pub fn dsh_rtk_plugin_source(exe: &str) -> String {
    let quoted = serde_json::to_string(exe).unwrap_or_else(|_| format!("\"{}\"", exe));
    format!(
r#"// tokless-rtk: transparent RTK rewrite for dsh bash tool calls.
const TOKLESS_BIN = {}

const name = "tokless-rtk"
const inject = []

async function apply(ctx) {{
  ctx.on("tools/pre-execute", async (exec, next) => {{
    const args = exec?.arguments
    if (exec?.name !== "bash" || typeof args?.command !== "string" || args.command.length === 0) {{
      return next()
    }}
    try {{
      const {{ spawnSync }} = await import("node:child_process")
      const r = spawnSync(TOKLESS_BIN, ["rtk-rewrite", "--", args.command], {{ encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }})
      if (r.status === 0 && typeof r.stdout === "string") {{
        const rewritten = r.stdout.trim()
        if (rewritten && rewritten !== args.command) {{
          exec.arguments = {{ ...args, command: rewritten }}
        }}
      }}
    }} catch {{}}
    return next()
  }})
}}

export default {{ name, inject, apply }}
"#,
        quoted
    )
}

fn yaml_quote(s: &str) -> String {
    if s.contains(':') || s.contains('#') || s.contains('\'') || s.contains('"') || s.starts_with(' ') {
        return format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""));
    }
    s.to_string()
}

pub fn dsh_rtk_plugin_entry() -> String {
    format!("\n- insert:\n    - id: {}\n      name: {}\n", DSH_RTK_PLUGIN_NAME, yaml_quote(&dsh_rtk_plugin_file()))
}

pub fn dsh_rtk_wired() -> bool {
    let raw = match read_file_safe(&dsh_patch_file()) {
        Some(r) => r,
        None => return false,
    };
    raw.contains(&format!("id: {}", DSH_RTK_PLUGIN_NAME)) && exists(&dsh_rtk_plugin_file())
}

pub fn dsh_bridge_wired() -> bool {
    let raw = match read_file_safe(&dsh_patch_file()) {
        Some(r) => r,
        None => return false,
    };
    raw.contains("name: '@deepseek-ai/dsh-hooks-claude-code'")
}

pub fn configure_dsh_rtk(exe: &str) -> (bool, String, Option<std::io::Error>) {
    if exe.is_empty() {
        return (false, String::new(), None);
    }
    let patched = dsh_patch_file();
    let plugin = dsh_rtk_plugin_file();
    let raw = read_file_safe(&patched).unwrap_or_default();
    let mut changed = false;

    if !raw.contains(&format!("id: {}", DSH_RTK_PLUGIN_NAME)) {
        if let Some(parent) = Path::new(&patched).parent() {
            if let Err(e) = ensure_dir(parent) {
                return (false, patched, Some(e));
            }
        }
        let updated = format!("{}{}", raw, dsh_rtk_plugin_entry());
        if let Err(e) = write_file(&patched, &updated) {
            return (false, patched, Some(e));
        }
        changed = true;
    }

    let want = dsh_rtk_plugin_source(exe);
    let cur = read_file_safe(&plugin);
    if cur.as_deref() != Some(&want) {
        if let Some(parent) = Path::new(&plugin).parent() {
            if let Err(e) = ensure_dir(parent) {
                return (false, plugin, Some(e));
            }
        }
        if let Err(e) = write_file(&plugin, &want) {
            return (false, plugin, Some(e));
        }
        changed = true;
    }

    if let Some(hooks) = read_file_safe(&dsh_hooks_file()) {
        if hooks.contains("rtk-hook dsh") {
            let _ = fs::remove_file(dsh_hooks_file());
            changed = true;
            remove_dsh_bridge_row();
        }
    }

    (changed, patched, None)
}

pub fn remove_dsh_bridge_row() -> bool {
    let patched = dsh_patch_file();
    let raw = match read_file_safe(&patched) {
        Some(r) => r,
        None => return false,
    };
    let marker = "name: '@deepseek-ai/dsh-hooks-claude-code'";
    if !raw.contains(marker) {
        return false;
    }
    let lines: Vec<&str> = raw.lines().collect();
    let mut out: Vec<&str> = Vec::new();
    let mut skipping = false;
    for l in lines {
        if l.contains(marker) {
            skipping = true;
            continue;
        }
        if skipping && (l.starts_with("    -") || l.starts_with("- insert:")) {
            skipping = false;
        }
        if !skipping {
            out.push(l);
        }
    }
    let next = out.join("\n");
    write_file(&patched, &next).is_ok()
}

pub fn remove_dsh_rtk() -> bool {
    let patched = dsh_patch_file();
    let raw = match read_file_safe(&patched) {
        Some(r) => r,
        None => return false,
    };
    let mut changed = false;
    if raw.contains(&format!("id: {}", DSH_RTK_PLUGIN_NAME)) {
        let lines: Vec<&str> = raw.lines().collect();
        let mut out = Vec::new();
        let mut skip = 0;
        for l in lines {
            if l.contains(&format!("id: {}", DSH_RTK_PLUGIN_NAME)) {
                skip = 2; // skip this and next name: line
                continue;
            }
            if skip > 0 {
                skip -= 1;
                continue;
            }
            out.push(l);
        }
        let next = out.join("\n");
        let _ = write_file(&patched, &next);
        changed = true;
    }
    let plugin = dsh_rtk_plugin_file();
    if exists(&plugin) {
        let _ = fs::remove_file(&plugin);
        changed = true;
    }
    changed
}

pub fn dsh_provider_protocol(api: &str) -> String {
    match api.trim().to_lowercase().as_str() {
        "" | "openai" | "openai-completions" | "openai-chat" | "openai_chat" | "openai_completions" => {
            "openai-completions".to_string()
        }
        "openai-responses" | "openai_responses" => "openai-responses".to_string(),
        "anthropic-messages" | "anthropic_messages" => "anthropic-messages".to_string(),
        _ => String::new(),
    }
}

pub fn dsh_headroom_upstream(base_url: &str) -> Option<(String, String)> {
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

pub fn dsh_headroom_base(base: &str) -> bool {
    same_proxy_base(base, &proxy_endpoint_for("dsh"))
}

pub fn dsh_gateway_base(base: &str) -> bool {
    let g = crate::util::byok::byok_gateway_endpoint();
    same_proxy_base(base, &g) || same_proxy_base(base, &format!("{}/v1", g))
}

pub fn dsh_proxy_base(base: &str) -> bool {
    dsh_headroom_base(base) || dsh_gateway_base(base)
}

fn yaml_key_line(line: &str) -> (i32, String, bool, String) {
    let trimmed = line.trim_start_matches(' ');
    let indent = (line.len() - trimmed.len()) as i32;
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return (-1, String::new(), false, String::new());
    }
    let colon = match trimmed.find(':') {
        Some(idx) if idx > 0 => idx,
        _ => return (-1, String::new(), false, String::new()),
    };
    let key = trimmed[..colon].to_string();
    let mut rest = trimmed[colon + 1..].trim().to_string();
    if let Some(i) = rest.find(" #") {
        rest = rest[..i].trim().to_string();
    }
    if rest.is_empty() {
        return (indent, key, false, String::new());
    }
    if rest.len() >= 2 && (rest.starts_with('\'') || rest.starts_with('"')) && rest.ends_with(rest.chars().next().unwrap()) {
        rest = rest[1..rest.len() - 1].to_string();
    }
    (indent, key, true, rest)
}

fn dsh_find_plugin_block(lines: &[String], plugin_id: &str) -> Option<(usize, usize)> {
    let want = format!("- id: {}", plugin_id);
    let mut start = None;
    for (i, l) in lines.iter().enumerate() {
        if l.trim() == want {
            start = Some(i);
            break;
        }
    }
    let s = start?;
    let mut end = lines.len();
    let block_indent = lines[s].len() - lines[s].trim_start_matches(' ').len();
    for i in s + 1..lines.len() {
        let trimmed = lines[i].trim_start_matches(' ');
        let indent = lines[i].len() - trimmed.len();
        if indent <= block_indent && trimmed.starts_with("- id: ") {
            end = i;
            break;
        }
    }
    Some((s, end))
}

fn dsh_provider_span(lines: &[String], start: usize, end: usize, provider_id: &str) -> Option<(usize, usize, usize)> {
    let mut prov_key = None;
    let mut prov_indent = -1;
    for i in start + 1..end {
        let (ind, key, _, _) = yaml_key_line(&lines[i]);
        if key == "providers" {
            prov_key = Some(i);
            prov_indent = ind;
            break;
        }
    }
    let pk = prov_key?;
    let mut direct_indent = -1;
    for i in pk + 1..end {
        let (ind, _, _, _) = yaml_key_line(&lines[i]);
        if ind > prov_indent {
            direct_indent = ind;
            break;
        }
    }
    if direct_indent < 0 {
        return None;
    }
    let mut prov_line = None;
    for i in pk + 1..end {
        let (ind, key, _, _) = yaml_key_line(&lines[i]);
        if ind == direct_indent && key == provider_id {
            prov_line = Some(i);
            break;
        }
    }
    let pl = prov_line?;
    let mut prov_end = end;
    for i in pl + 1..end {
        let (ind, _, _, _) = yaml_key_line(&lines[i]);
        if ind >= 0 && ind <= direct_indent {
            prov_end = i;
            break;
        }
    }
    Some((pk, pl, prov_end))
}

fn dsh_provider_field(lines: &[String], prov_line: usize, prov_end: usize, key: &str) -> (String, usize, bool) {
    for i in prov_line + 1..prov_end {
        let (_, k, hv, v) = yaml_key_line(&lines[i]);
        if k == key {
            return (v, i, hv);
        }
    }
    (String::new(), 0, false)
}

fn dsh_header_span(lines: &[String], prov_line: usize, prov_end: usize) -> Option<(usize, i32, usize)> {
    let mut hdr_key = None;
    let mut hdr_indent = -1;
    for i in prov_line + 1..prov_end {
        let (ind, key, has_val, _) = yaml_key_line(&lines[i]);
        if key == "headers" && !has_val {
            hdr_key = Some(i);
            hdr_indent = ind;
            break;
        }
    }
    let hk = hdr_key?;
    let mut hdr_end = prov_end;
    for i in hk + 1..prov_end {
        let (ind, _, _, _) = yaml_key_line(&lines[i]);
        if ind >= 0 && ind <= hdr_indent {
            hdr_end = i;
            break;
        }
    }
    Some((hk, hdr_indent, hdr_end))
}

fn dsh_header_value(lines: &[String], prov_line: usize, prov_end: usize, key: &str) -> (String, bool) {
    let (hdr_key, hdr_indent, hdr_end) = match dsh_header_span(lines, prov_line, prov_end) {
        Some(s) => s,
        None => return (String::new(), false),
    };
    for i in hdr_key + 1..hdr_end {
        let (ind, k, hv, v) = yaml_key_line(&lines[i]);
        if ind > hdr_indent && k == key {
            return (v, hv);
        }
    }
    (String::new(), false)
}

fn dsh_set_provider_field(lines: &mut [String], line: usize, key: &str, value: &str) {
    let (ind, _, _, _) = yaml_key_line(&lines[line]);
    let indent = if ind >= 0 { ind as usize } else { 0 };
    lines[line] = format!("{}{}: {}", " ".repeat(indent), key, value);
}

fn dsh_upsert_header(lines: &mut Vec<String>, prov_line: usize, prov_end: usize, key: &str, value: &str) -> usize {
    if let Some((hdr_key, hdr_indent, hdr_end)) = dsh_header_span(lines, prov_line, prov_end) {
        for i in hdr_key + 1..hdr_end {
            let (ind, k, _, old) = yaml_key_line(&lines[i]);
            if ind > hdr_indent && k == key {
                let indent = if ind >= 0 { ind as usize } else { 0 };
                lines[i] = format!("{}{}: {}", " ".repeat(indent), key, value);
                return if old == value { 0 } else { 1 };
            }
        }
        let indent = if hdr_indent >= 0 { (hdr_indent + 2) as usize } else { 2 };
        lines.insert(hdr_key + 1, format!("{}{}: {}", " ".repeat(indent), key, value));
        1
    } else {
        let (prov_ind, _, _, _) = yaml_key_line(&lines[prov_line]);
        let insert = if prov_ind >= 0 { (prov_ind + 2) as usize } else { 2 };
        lines.insert(prov_end, format!("{}headers:", " ".repeat(insert)));
        lines.insert(prov_end + 1, format!("{}{}: {}", " ".repeat(insert + 2), key, value));
        2
    }
}

fn dsh_remove_header(lines: &mut Vec<String>, prov_line: usize, prov_end: usize, key: &str) -> bool {
    let (hdr_key, _, hdr_end) = match dsh_header_span(lines, prov_line, prov_end) {
        Some(s) => s,
        None => return false,
    };
    for i in hdr_key + 1..hdr_end {
        let (_, k, _, _) = yaml_key_line(&lines[i]);
        if k == key {
            lines.remove(i);
            if let Some((hk, _, he)) = dsh_header_span(lines, prov_line, prov_end - 1) {
                let mut empty = true;
                for j in hk + 1..he {
                    let (_, kk, _, _) = yaml_key_line(&lines[j]);
                    if !kk.is_empty() {
                        empty = false;
                        break;
                    }
                }
                if empty {
                    lines.remove(hk);
                }
            }
            return true;
        }
    }
    false
}

fn dsh_provider_ids(lines: &[String], start: usize, end: usize) -> Vec<String> {
    let mut prov_key = None;
    let mut prov_indent = -1;
    for i in start + 1..end {
        let (ind, key, _, _) = yaml_key_line(&lines[i]);
        if key == "providers" {
            prov_key = Some(i);
            prov_indent = ind;
            break;
        }
    }
    let pk = match prov_key {
        Some(k) => k,
        None => return Vec::new(),
    };
    let mut direct_indent = -1;
    for i in pk + 1..end {
        let (ind, _, _, _) = yaml_key_line(&lines[i]);
        if ind > prov_indent {
            direct_indent = ind;
            break;
        }
    }
    if direct_indent < 0 {
        return Vec::new();
    }
    let mut ids = Vec::new();
    for i in pk + 1..end {
        let (ind, key, _, _) = yaml_key_line(&lines[i]);
        if ind == direct_indent && !key.is_empty() {
            ids.push(key);
        }
    }
    ids
}

#[derive(Debug, Clone)]
pub struct DshBYOK {
    pub id: String,
    pub file: String,
    pub base_url: String,
    pub api_key: String,
    pub protocol: String,
}

pub fn discover_dsh_byok() -> Vec<DshBYOK> {
    let path = dsh_patch_file();
    let raw = match read_file_safe(&path) {
        Some(r) => r,
        None => return Vec::new(),
    };
    let lines: Vec<String> = raw.lines().map(|s| s.to_string()).collect();
    let (start, end) = match dsh_find_plugin_block(&lines, "llm-pi-ai") {
        Some(s) => s,
        None => return Vec::new(),
    };

    let stashed = load_proxy_route_stash("dsh");
    let mut out = Vec::new();

    for id in dsh_provider_ids(&lines, start, end) {
        let (_, prov_line, prov_end) = match dsh_provider_span(&lines, start, end, &id) {
            Some(s) => s,
            None => continue,
        };
        let (api, _, _) = dsh_provider_field(&lines, prov_line, prov_end, "api");
        let proto = dsh_provider_protocol(&api);
        if proto.is_empty() {
            continue;
        }
        let (mut base, _, _) = dsh_provider_field(&lines, prov_line, prov_end, "baseURL");
        let proxy_base = proxy_endpoint_for("dsh").trim_end_matches('/').to_string();

        if is_valid_byok_upstream(&base) && !same_proxy_base(&base, &proxy_base) && !dsh_gateway_base(&base) {
            // keep on-disk original
        } else if let Some(s) = stashed.get(&id) {
            if !s.base_url.is_empty() {
                base = s.base_url.clone();
            }
        } else {
            let (hdr, has) = dsh_header_value(&lines, prov_line, prov_end, byok_route_header_key());
            if has {
                if let Some((prefix, token)) = hdr.split_once('.') {
                    if prefix.starts_with("dsh:") && !token.is_empty() {
                        if let Some(route) = read_byok_route(prefix) {
                            if route.token == token && is_valid_byok_upstream(&route.upstream) {
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
            } else {
                base = String::new();
            }
        }

        if base.is_empty() {
            continue;
        }

        let (api_key, _, _) = dsh_provider_field(&lines, prov_line, prov_end, "apiKey");
        out.push(DshBYOK {
            id,
            file: path.clone(),
            base_url: base,
            api_key,
            protocol: proto,
        });
    }

    out
}

pub fn configure_dsh_proxy() -> (bool, String) {
    let mut changed = false;
    let mut file = dsh_patch_file();
    if let Err(err) = with_proxy_route_stash_lock(|| {
        let (c, f) = configure_dsh_proxy_locked();
        changed = c;
        file = f;
        Ok(())
    }) {
        L.err(&format!("dsh proxy lock failed: {err}"));
    }
    (changed, file)
}

pub fn configure_dsh_proxy_locked() -> (bool, String) {
    let file = dsh_patch_file();
    if !proxy_route_stash_valid("dsh") {
        return (false, file);
    }
    let raw = match read_file_safe(&file) {
        Some(r) => r,
        None => return (false, file),
    };
    let discovered = discover_dsh_byok();
    if discovered.is_empty() {
        return (false, file);
    }

    let mut stash = load_proxy_route_stash_locked("dsh");
    let stash_raw = read_file_safe(&proxy_route_stash_path("dsh"));
    let stash_exists = stash_raw.is_some();
    let mut route_changes: Vec<ByokRouteRollback> = Vec::new();

    let rollback = |route_changes: &[ByokRouteRollback]| {
        for rc in route_changes.iter().rev() {
            rollback_byok_route_logged("dsh", &rc.id, &rc.previous);
        }
        restore_proxy_route_stash_logged("dsh", stash_raw.as_deref().unwrap_or(""), stash_exists);
    };

    let mut lines: Vec<String> = raw.lines().map(|s| s.to_string()).collect();
    let headroom_base = proxy_endpoint_for("dsh");
    let mut changed_any = false;

    for provider in discovered {
        let (start, end) = match dsh_find_plugin_block(&lines, "llm-pi-ai") {
            Some(s) => s,
            None => break,
        };
        let (_, prov_line, prov_end) = match dsh_provider_span(&lines, start, end, &provider.id) {
            Some(s) => s,
            None => continue,
        };
        let (origin, chat_path) = match dsh_headroom_upstream(&provider.base_url) {
            Some(res) => res,
            None => {
                L.warn(&format!("dsh: skipping provider with unparseable baseURL: {}", provider.id));
                continue;
            }
        };

        let upstream = normalized_headroom_upstream(&provider.base_url, &provider.protocol);
        let (_, route_header, previous, route_ok) = register_byok_route("dsh", &provider.id, &provider.protocol, &upstream);
        if !route_ok {
            rollback(&route_changes);
            return (false, file);
        }
        route_changes.push(ByokRouteRollback { id: provider.id.clone(), previous });

        let (hdr_val, had_header) = dsh_header_value(&lines, prov_line, prov_end, byok_route_header_key());
        let (base_val, base_line, has_base) = dsh_provider_field(&lines, prov_line, prov_end, "baseURL");

        if !has_base || base_val != headroom_base {
            if has_base {
                dsh_set_provider_field(&mut lines, base_line, "baseURL", &yaml_quote(&headroom_base));
            } else {
                let (prov_ind, _, _, _) = yaml_key_line(&lines[prov_line]);
                let indent = if prov_ind >= 0 { (prov_ind + 2) as usize } else { 2 };
                lines.insert(prov_line + 1, format!("{}baseURL: {}", " ".repeat(indent), yaml_quote(&headroom_base)));
            }
        }

        // Re-acquire span after possible insertion
        let (s2, e2) = dsh_find_plugin_block(&lines, "llm-pi-ai").unwrap();
        let (_, pl2, pe2) = dsh_provider_span(&lines, s2, e2, &provider.id).unwrap();

        let byok_key = byok_route_header_key();
        dsh_upsert_header(&mut lines, pl2, pe2, byok_key, &yaml_quote(&route_header));

        let (s3, e3) = dsh_find_plugin_block(&lines, "llm-pi-ai").unwrap();
        let (_, pl3, pe3) = dsh_provider_span(&lines, s3, e3, &provider.id).unwrap();
        dsh_upsert_header(&mut lines, pl3, pe3, DSH_HEADROOM_BASE_URL_HEADER, &yaml_quote(&origin));

        let (s4, e4) = dsh_find_plugin_block(&lines, "llm-pi-ai").unwrap();
        let (_, pl4, pe4) = dsh_provider_span(&lines, s4, e4, &provider.id).unwrap();
        dsh_upsert_header(&mut lines, pl4, pe4, DSH_HEADROOM_ORIGINAL_PATH_HDR, &yaml_quote(&chat_path));

        changed_any = true;

        stash.insert(provider.id.clone(), ProxyRouteStashEntry {
            file: file.clone(),
            provider: provider.id.clone(),
            base_url: provider.base_url.clone(),
            upstream,
            header: hdr_val,
            had_header,
            ..Default::default()
        });
    }

    if !changed_any {
        rollback(&route_changes);
        return (false, file);
    }
    if save_proxy_route_stash("dsh", &stash).is_err() {
        rollback(&route_changes);
        return (false, file);
    }
    let content = lines.join("\n");
    if write_file_atomic(&file, &content).is_err() {
        rollback(&route_changes);
        return (false, file);
    }

    (true, file)
}

pub fn remove_dsh_proxy() -> bool {
    let mut removed = false;
    if let Err(err) = with_proxy_route_stash_lock(|| {
        removed = remove_dsh_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("dsh proxy lock failed: {err}"));
    }
    removed
}

pub fn remove_dsh_proxy_locked() -> bool {
    if !proxy_route_stash_valid("dsh") {
        return false;
    }
    let file = dsh_patch_file();
    let raw = match read_file_safe(&file) {
        Some(r) => r,
        None => return false,
    };
    let stash = load_proxy_route_stash_locked("dsh");
    if stash.is_empty() {
        return false;
    }

    let mut lines: Vec<String> = raw.lines().map(|s| s.to_string()).collect();
    let mut remaining = std::collections::HashMap::new();
    let mut deleted_routes: Vec<BYOKRoute> = Vec::new();
    let mut deleted_ids: Vec<String> = Vec::new();

    for (id, entry) in &stash {
        let (start, end) = match dsh_find_plugin_block(&lines, "llm-pi-ai") {
            Some(s) => s,
            None => {
                remaining.insert(id.clone(), entry.clone());
                continue;
            }
        };
        let (_, prov_line, prov_end) = match dsh_provider_span(&lines, start, end, id) {
            Some(s) => s,
            None => {
                remaining.insert(id.clone(), entry.clone());
                continue;
            }
        };

        let route = read_byok_route(&format!("dsh:{}", id));
        let (value, has_hdr) = dsh_header_value(&lines, prov_line, prov_end, byok_route_header_key());
        let (base_val, base_line, _) = dsh_provider_field(&lines, prov_line, prov_end, "baseURL");

        let matches = route.is_some() && has_hdr && dsh_proxy_base(&base_val)
            && route.as_ref().map(|r| value == byok_route_header(r)).unwrap_or(false);

        if !matches {
            remaining.insert(id.clone(), entry.clone());
            continue;
        }

        dsh_set_provider_field(&mut lines, base_line, "baseURL", &yaml_quote(&entry.base_url));

        let byok_key = byok_route_header_key();
        if entry.had_header {
            dsh_upsert_header(&mut lines, prov_line, prov_end, byok_key, &yaml_quote(&entry.header));
        } else {
            dsh_remove_header(&mut lines, prov_line, prov_end, byok_key);
        }

        let (s2, e2) = dsh_find_plugin_block(&lines, "llm-pi-ai").unwrap();
        let (_, pl2, pe2) = dsh_provider_span(&lines, s2, e2, id).unwrap();
        dsh_remove_header(&mut lines, pl2, pe2, DSH_HEADROOM_BASE_URL_HEADER);

        let (s3, e3) = dsh_find_plugin_block(&lines, "llm-pi-ai").unwrap();
        let (_, pl3, pe3) = dsh_provider_span(&lines, s3, e3, id).unwrap();
        dsh_remove_header(&mut lines, pl3, pe3, DSH_HEADROOM_ORIGINAL_PATH_HDR);

        if let Some(r) = route {
            deleted_routes.push(r);
        }
        deleted_ids.push(id.clone());
    }

    if deleted_ids.is_empty() {
        return false;
    }
    let content = lines.join("\n");
    if write_file_atomic(&file, &content).is_err() {
        return false;
    }
    for id in &deleted_ids {
        if delete_byok_route(&format!("dsh:{}", id)).is_err() {
            let _ = write_file(&file, &raw);
            return false;
        }
    }
    if save_proxy_route_stash("dsh", &remaining).is_err() {
        let _ = write_file(&file, &raw);
        for r in deleted_routes {
            let _ = upsert_byok_route(&r.id, &r.protocol, &r.upstream);
        }
        return false;
    }

    true
}

pub fn dsh_proxy_wired() -> bool {
    let stash = load_proxy_route_stash("dsh");
    if stash.is_empty() {
        return false;
    }
    let raw = match read_file_safe(&dsh_patch_file()) {
        Some(r) => r,
        None => return false,
    };
    let lines: Vec<String> = raw.lines().map(|s| s.to_string()).collect();
    let (start, end) = match dsh_find_plugin_block(&lines, "llm-pi-ai") {
        Some(s) => s,
        None => return false,
    };

    for (id, entry) in &stash {
        let (_, prov_line, prov_end) = match dsh_provider_span(&lines, start, end, id) {
            Some(s) => s,
            None => return false,
        };
        let (base_val, _, _) = dsh_provider_field(&lines, prov_line, prov_end, "baseURL");
        if !dsh_headroom_base(&base_val) {
            return false;
        }
        let route = match read_byok_route(&format!("dsh:{}", id)) {
            Some(r) => r,
            None => return false,
        };
        let (value, has_hdr) = dsh_header_value(&lines, prov_line, prov_end, byok_route_header_key());
        if !has_hdr || value != byok_route_header(&route) {
            return false;
        }
        let (origin, chat_path) = match dsh_headroom_upstream(&entry.base_url) {
            Some(res) => res,
            None => return false,
        };
        let (h_base, has_hb) = dsh_header_value(&lines, prov_line, prov_end, DSH_HEADROOM_BASE_URL_HEADER);
        if !has_hb || h_base != origin {
            return false;
        }
        let (h_path, has_hp) = dsh_header_value(&lines, prov_line, prov_end, DSH_HEADROOM_ORIGINAL_PATH_HDR);
        if !has_hp || h_path != chat_path {
            return false;
        }
    }

    true
}
