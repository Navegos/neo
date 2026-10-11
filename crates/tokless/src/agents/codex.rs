use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::path::Path;
use std::time::{Duration, Instant};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::agents::proxy_protocol::proxy_endpoint_for;
use crate::util::byok::{
    BYOKRoute, byok_gateway_endpoint, byok_route_header, delete_byok_route as util_delete_byok_route,
    read_byok_route, upsert_byok_route,
};
use crate::util::headroom::headroom_paths_resolved;
use crate::util::jsonc::{OrderedMap, stringify_json, try_parse_jsonc};
use crate::util::logger::L;
use crate::util::mcpspawn::{mcp_spawn_for, pick_mcp_spawn, wrap_auto_index};
use crate::util::paths::{
    IS_WIN, codex_paths_resolved, ensure_dir, exists, home, persisted_tokless_command,
    read_file_safe, split_command, tokless_persisted_abs, write_file,
};
use crate::util::toml::{
    TomlBlock, block_text, get_toml_top_key, has_block, remove_block, remove_toml_top_key,
    render_block, set_toml_top_key, toml_dotted_table_header, upsert_block,
};

pub const CODEX_PROXY_PROVIDER: &str = "model_providers.headroom";
pub const CODEX_MODEL_PROVIDER_KEY: &str = "model_provider";
pub const CODEX_MARKER_START: &str = "# --- Headroom persistent provider ---";
pub const CODEX_MARKER_END: &str = "# --- end Headroom persistent provider ---";

const CODEX_BYOK_KEY_VAR: &str = "TOKLESS_CODEX_API_KEY";
const CODEX_BYOK_URL_VAR: &str = "TOKLESS_HEADROOM_BASE_URL";
const CODEX_BYOK_ROUTE_VAR: &str = "TOKLESS_CODEX_ROUTE";
const BYOK_ROUTE_HEADER_KEY: &str = "X-Tokless-Route";

const CODEX_HOOK_MATCHER: &str = "Bash|apply_patch|ctx_.*|codegraph_.*";
const CODEX_HOOK_TIMEOUT: i64 = 10;
const CODEX_PERM_HOOK_MATCHER: &str = "Bash|apply_patch";
const CODEX_PERM_HOOK_TIMEOUT: i64 = 5;
const CODEX_PROJECTMEM_HOOK_TIMEOUT: i64 = 15;

const CODEX_FEATURES_HOOK_MARKER: &str = "# tokless-managed: features.hooks\n";
const CODEX_APPROVAL_POLICY_MARKER: &str = "# tokless-managed: approval_policy\n";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexBYOK {
    pub id: String,
    pub file: String,
    pub base_url: String,
    pub api_key: String,
    pub protocol: String,
    pub route_header: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexStash {
    #[serde(default)]
    pub provider_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub openai_base_url: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub byok: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub env_key_existed: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub env_url_existed: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub provider_line: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url_line: String,
}

fn is_false(b: &bool) -> bool {
    !*b
}

// --- Internal shared helpers ---

fn tokless_command(args: &[&str]) -> String {
    persisted_tokless_command(&tokless_persisted_abs(), args)
}

fn tokless_managed_command(cmd: &str, args: &[&str]) -> bool {
    let parts = split_command(cmd.trim());
    if parts.len() != args.len() + 1 {
        return false;
    }
    let normalized = parts[0].replace('\\', "/");
    let exe = Path::new(&normalized)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if exe != "tokless" && exe != "tokless.exe" {
        return false;
    }
    for (i, a) in args.iter().enumerate() {
        if parts[i + 1] != *a {
            return false;
        }
    }
    true
}

fn write_file_mode_600(path: &str, content: &str) -> std::io::Result<()> {
    write_file(path, content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

fn chmod_600(path: &str) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).is_ok()
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        true
    }
}

fn is_absolute_http(u: &str) -> bool {
    let parsed = match url::Url::parse(u.trim()) {
        Ok(p) => p,
        Err(_) => return false,
    };
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return false;
    }
    match parsed.host_str() {
        Some(h) if !h.is_empty() => true,
        _ => false,
    }
}

fn same_proxy_base(a: &str, b: &str) -> bool {
    a.trim().trim_end_matches('/') == b.trim().trim_end_matches('/')
}

fn strip_v1_suffix(u: &str) -> String {
    let s = u.trim_end_matches('/');
    s.strip_suffix("/v1").unwrap_or(s).to_string()
}

fn byok_route_id(agent: &str, provider_id: &str) -> String {
    format!("{}:{}", agent, provider_id)
}

fn register_byok_route(
    agent: &str,
    provider_id: &str,
    api: &str,
    upstream: &str,
) -> (String, String, Option<BYOKRoute>, bool) {
    let protocol = crate::util::byok::byok_route_protocol(api);
    if protocol.is_empty() || upstream.trim().is_empty() {
        return (String::new(), String::new(), None, false);
    }
    match upsert_byok_route(&byok_route_id(agent, provider_id), protocol, upstream) {
        Ok((route, prev)) => (
            byok_gateway_endpoint(),
            byok_route_header(&route),
            prev,
            true,
        ),
        Err(_) => (String::new(), String::new(), None, false),
    }
}

fn delete_byok_route_for(agent: &str, provider_id: &str) -> Result<(), String> {
    util_delete_byok_route(&byok_route_id(agent, provider_id)).map_err(|e| e.to_string())
}

fn rollback_byok_route(agent: &str, provider_id: &str, previous: Option<&BYOKRoute>) -> Result<(), String> {
    if let Some(prev) = previous {
        if !prev.id.is_empty() {
            return upsert_byok_route(&byok_route_id(agent, provider_id), &prev.protocol, &prev.upstream)
                .map(|_| ())
                .map_err(|e| e.to_string());
        }
    }
    delete_byok_route_for(agent, provider_id)
}

fn rollback_byok_route_logged(agent: &str, provider_id: &str, previous: Option<&BYOKRoute>) {
    if let Err(err) = rollback_byok_route(agent, provider_id, previous) {
        L.err(&format!(
            "{} BYOK route rollback failed: {}",
            agent, err
        ));
    }
}

fn proxy_route_stash_lock_path() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("byok.stash.lock")
        .to_string_lossy()
        .to_string()
}

fn with_proxy_route_stash_lock<F>(f: F) -> Result<(), String>
where
    F: FnOnce() -> Result<(), String>,
{
    let root = headroom_paths_resolved().root;
    ensure_dir(&root).map_err(|e| e.to_string())?;
    let lock_path = proxy_route_stash_lock_path();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match OpenOptions::new().write(true).create_new(true).open(&lock_path) {
            Ok(file) => {
                drop(file);
                let res = f();
                let _ = fs::remove_file(&lock_path);
                return res;
            }
            Err(_) => {
                if let Ok(meta) = fs::metadata(&lock_path) {
                    if let Ok(modified) = meta.modified() {
                        if modified.elapsed().unwrap_or_default() > Duration::from_secs(30) {
                            let _ = fs::remove_file(&lock_path);
                            continue;
                        }
                    }
                }
                if Instant::now() >= deadline {
                    return Err(format!("timed out acquiring {}", lock_path));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

// Pure-Rust FIPS 180-4 SHA-256 implementation for Codex hook trust hashes.
fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = String::with_capacity(64);
    for word in h {
        out.push_str(&format!("{:08x}", word));
    }
    out
}

// --- Codex MCP ---

pub fn configure_codex_mcp(tool_id: &str) -> (bool, String) {
    let p = codex_paths_resolved();
    let _ = ensure_dir(&p.dir);
    let raw = read_file_safe(&p.config).unwrap_or_default();
    let raw = sweep_stale_hook_state_entries(&raw);
    let spawn = if tool_id == "codegraph" {
        wrap_auto_index("codex", pick_mcp_spawn("codegraph", &["serve", "--mcp"]))
    } else {
        mcp_spawn_for(tool_id)
    };
    let section = if tool_id == "context-mode" {
        "mcp_servers.context_mode".to_string()
    } else {
        format!("mcp_servers.{}", tool_id)
    };
    let mut block = TomlBlock::new(section);
    block.set("command", spawn.command);
    block.set("args", spawn.args);
    block.set("enabled", true);
    block.set("default_tools_approval_mode", "approve");
    let next = upsert_block(&raw, &block, false);
    let next = apply_codex_approval_policy(&next);
    if next == raw {
        return (false, p.config);
    }
    let _ = write_file(&p.config, &next);
    (true, p.config)
}

fn codex_hook_state_header(key: &str) -> String {
    toml_dotted_table_header("hooks.state", key)
}

fn split_after_newlines(s: &str) -> Vec<&str> {
    if s.is_empty() {
        return vec![""];
    }
    let mut out = Vec::new();
    let mut start = 0;
    for (idx, ch) in s.char_indices() {
        if ch == '\n' {
            out.push(&s[start..=idx]);
            start = idx + 1;
        }
    }
    out.push(&s[start..]);
    out
}

pub fn sweep_stale_hook_state_entries(raw: &str) -> String {
    let current = codex_hooks_file();
    let re = Regex::new(r#"^\[hooks\.state\.(?:'([^']*)'|"([^"]*)")\]\s*$"#).unwrap();
    let lines = split_after_newlines(raw);
    let mut out = String::new();
    let mut i = 0;
    while i < lines.len() {
        let line_no_nl = lines[i].trim_end_matches(['\r', '\n']);
        let Some(caps) = re.captures(line_no_nl) else {
            out.push_str(lines[i]);
            i += 1;
            continue;
        };
        let mut j = i + 1;
        while j < lines.len() && !lines[j].trim_start_matches([' ', '\t']).starts_with('[') {
            j += 1;
        }
        let m1 = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let m2 = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let key = if !m1.is_empty() { m1 } else { m2 };
        let legacy_double = !m2.is_empty();
        if key.starts_with(&format!("{}:", current)) && !legacy_double {
            while i < j {
                out.push_str(lines[i]);
                i += 1;
            }
            continue;
        }
        i = j;
    }
    out
}

// --- Codex headroom HTTP proxy ---

pub fn codex_proxy_block(endpoint: &str, byok: Option<&CodexBYOK>) -> TomlBlock {
    let mut block = TomlBlock::new(CODEX_PROXY_PROVIDER);
    block.set("name", "Headroom persistent proxy");
    block.set("base_url", endpoint);
    block.set("wire_api", "responses");
    let Some(byok) = byok else {
        block.set("supports_websockets", false);
        if codex_uses_chatgpt_auth() {
            block.set("requires_openai_auth", true);
        }
        return block;
    };
    block.set("env_key", CODEX_BYOK_KEY_VAR);
    let mut headers = HashMap::new();
    if !byok.route_header.is_empty() {
        headers.insert(
            BYOK_ROUTE_HEADER_KEY.to_string(),
            CODEX_BYOK_ROUTE_VAR.to_string(),
        );
    }
    block.set("env_http_headers", headers);
    block.set("supports_websockets", false);
    block
}

pub fn codex_byok_proxy_endpoint() -> String {
    format!("{}/v1", byok_gateway_endpoint())
}

pub fn codex_endpoints_for(primary: &str) -> Vec<String> {
    let byok = codex_byok_proxy_endpoint();
    if byok == primary {
        vec![primary.to_string()]
    } else {
        vec![primary.to_string(), byok]
    }
}

pub fn codex_is_managed_endpoint(have: &str) -> bool {
    codex_endpoints_for(&proxy_endpoint_for("codex"))
        .iter()
        .any(|ep| ep == have)
}

pub fn codex_uses_chatgpt_auth() -> bool {
    let auth_path = Path::new(&codex_paths_resolved().dir).join("auth.json");
    let Some(raw) = read_file_safe(auth_path) else {
        return false;
    };
    #[derive(Deserialize, Default)]
    struct AuthTokens {
        #[serde(default)]
        account_id: String,
    }
    #[derive(Deserialize, Default)]
    struct AuthFile {
        #[serde(default)]
        auth_mode: String,
        #[serde(default)]
        tokens: AuthTokens,
    }
    let Ok(auth) = serde_json::from_str::<AuthFile>(&raw) else {
        return false;
    };
    auth.auth_mode.eq_ignore_ascii_case("chatgpt") || !auth.tokens.account_id.trim().is_empty()
}

pub fn codex_proxy_block_with_bearer(endpoint: &str) -> TomlBlock {
    let mut block = TomlBlock::new(CODEX_PROXY_PROVIDER);
    block.set("name", "Headroom persistent proxy");
    block.set("base_url", endpoint);
    block.set("experimental_bearer_token", "tokless");
    block.set("supports_websockets", true);
    block
}

fn codex_legacy_proxy_block(endpoint: &str) -> TomlBlock {
    let mut block = TomlBlock::new(CODEX_PROXY_PROVIDER);
    block.set("name", "Headroom persistent proxy");
    block.set("base_url", endpoint);
    block.set("env_key", "OPENAI_API_KEY");
    block.set("supports_websockets", true);
    block
}

pub fn strip_codex_managed_block(raw: &str) -> String {
    let pat = format!(
        "(?s){}.*?{}",
        regex::escape(CODEX_MARKER_START),
        regex::escape(CODEX_MARKER_END)
    );
    let re = Regex::new(&pat).unwrap();
    re.replace_all(raw, "").into_owned()
}

pub fn strip_codex_root_provider_assignments(raw: &str) -> String {
    let lines = split_after_newlines(raw);
    let mut out = String::new();
    let mut in_root = true;
    let re_header =
        Regex::new(r"^[ \t]*(?:\[\[[^\]\r\n]+\]\]|\[[^\]\r\n]+\])[ \t]*(?:#.*)?$").unwrap();
    let re_model =
        Regex::new(r#"^[ \t]*(?:model_provider|"model_provider"|'model_provider')[ \t]*="#)
            .unwrap();
    let re_url =
        Regex::new(r#"^[ \t]*(?:openai_base_url|"openai_base_url"|'openai_base_url')[ \t]*="#)
            .unwrap();
    for line in lines {
        let no_nl = line.trim_end_matches(['\r', '\n']);
        if in_root && re_header.is_match(no_nl) {
            in_root = false;
        }
        if in_root && (re_model.is_match(line) || re_url.is_match(line)) {
            continue;
        }
        out.push_str(line);
    }
    out
}

fn codex_root_value(raw: &str, key: &str) -> String {
    let k = regex::escape(key);
    let pat = format!(
        r#"^[ \t]*(?:{k}|"{k}"|'{k}')[ \t]*=[ \t]*(?:"([^"]*)"|'((?:[^']|'')*)')"#
    );
    let re = Regex::new(&pat).unwrap();
    for line in raw.split('\n') {
        if line.trim().starts_with('[') {
            break;
        }
        if let Some(caps) = re.captures(line) {
            let m1 = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            if !m1.is_empty() {
                return m1.to_string();
            }
            let m2 = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            return m2.replace("''", "'");
        }
    }
    String::new()
}

fn codex_root_assignment_line(raw: &str, key: &str) -> String {
    let k = regex::escape(key);
    let pat = format!(r#"^[ \t]*(?:{k}|"{k}"|'{k}')[ \t]*="#);
    let re = Regex::new(&pat).unwrap();
    for line in raw.split('\n') {
        if line.trim().starts_with('[') {
            break;
        }
        if re.is_match(line) {
            return line.to_string();
        }
    }
    String::new()
}

fn codex_marked_proxy(raw: &str) -> bool {
    if raw.matches(CODEX_MARKER_START).count() != 1 || raw.matches(CODEX_MARKER_END).count() != 1 {
        return false;
    }
    match (raw.find(CODEX_MARKER_START), raw.find(CODEX_MARKER_END)) {
        (Some(s), Some(e)) => s < e,
        _ => false,
    }
}

fn codex_marked_current_proxy(raw: &str, endpoint: &str) -> bool {
    codex_matched_marked_section(raw, endpoint).2
}

fn codex_root_extras(section: &str) -> Vec<String> {
    let mut extras = Vec::new();
    let mut in_block = false;
    for line in section.split('\n') {
        let t = line.trim();
        if t.starts_with('[') {
            in_block = true;
            continue;
        }
        if in_block || t.is_empty() || t.starts_with('#') || !t.contains('=') {
            continue;
        }
        let Some(eq_idx) = t.find('=') else {
            continue;
        };
        let key = t[..eq_idx].trim();
        if key != "model_provider" && key != "openai_base_url" {
            extras.push(line.to_string());
        }
    }
    extras
}

fn codex_without_lines(section: &str, drop: &[String]) -> String {
    let mut kept = Vec::new();
    for line in section.split('\n') {
        if !drop.iter().any(|d| d == line) {
            kept.push(line);
        }
    }
    kept.join("\n")
}

fn codex_matched_marked_section(raw: &str, endpoint: &str) -> (String, Vec<String>, bool) {
    if !codex_marked_proxy(raw) {
        return (String::new(), Vec::new(), false);
    }
    let start = raw.find(CODEX_MARKER_START).unwrap();
    let end = raw.find(CODEX_MARKER_END).unwrap() + CODEX_MARKER_END.len();
    let section = raw[start..end].trim().to_string();
    let extras = codex_root_extras(&section);
    let mut core = codex_without_lines(&section, &extras).trim().to_string();
    let legacy_header = format!(r#"x-headroom-base-url = "{}""#, CODEX_BYOK_URL_VAR);
    core = core.replace(&format!(", {}", legacy_header), "");
    core = core.replace(&format!("{}, ", legacy_header), "");
    core = core.replace(&legacy_header, "");
    let mut byok = CodexBYOK::default();
    if core.contains(BYOK_ROUTE_HEADER_KEY) {
        byok.route_header = CODEX_BYOK_ROUTE_VAR.to_string();
    }
    for ep in codex_endpoints_for(endpoint) {
        let sec_nil = codex_proxy_section(&ep, None);
        let sec_byok = codex_proxy_section(&ep, Some(&byok));
        let sec_ws_true = sec_nil.replacen(
            "supports_websockets = false",
            "supports_websockets = true",
            1,
        );
        if core == sec_nil.trim() || core == sec_byok.trim() || core == sec_ws_true.trim() {
            return (section, extras, true);
        }
    }
    (section, extras, false)
}

fn codex_inject_root_extras(content: &str, extras: &[String]) -> String {
    let lines: Vec<&str> = content.split('\n').collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + extras.len());
    let mut in_span = false;
    let mut inserted = false;
    for line in lines {
        out.push(line.to_string());
        if inserted {
            continue;
        }
        if line.contains(CODEX_MARKER_START) {
            in_span = true;
            continue;
        }
        if line.contains(CODEX_MARKER_END) {
            in_span = false;
            continue;
        }
        if in_span && line.trim().starts_with("openai_base_url ") {
            out.extend(extras.iter().cloned());
            inserted = true;
        }
    }
    out.join("\n")
}

fn codex_marked_legacy_bearer_proxy(raw: &str, endpoint: &str) -> bool {
    if !codex_marked_proxy(raw) {
        return false;
    }
    let start = raw.find(CODEX_MARKER_START).unwrap();
    let end = raw.find(CODEX_MARKER_END).unwrap() + CODEX_MARKER_END.len();
    let re = Regex::new(r#"(?m)^experimental_bearer_token = "[^"]*"$"#).unwrap();
    let sec = codex_proxy_section_with_bearer(endpoint);
    let want = re.replace_all(
        sec.trim(),
        r#"experimental_bearer_token = "__TOKLESS_BEARER__""#,
    );
    let actual = re.replace_all(
        raw[start..end].trim(),
        r#"experimental_bearer_token = "__TOKLESS_BEARER__""#,
    );
    actual == want
}

fn codex_legacy_proxy(raw: &str, endpoint: &str) -> bool {
    let block = codex_provider_block(raw);
    if block.is_empty() {
        return false;
    }
    block.trim() == render_block(&codex_legacy_proxy_block(endpoint)).trim()
}

fn codex_provider_block(raw: &str) -> String {
    let header = format!("[{}]", CODEX_PROXY_PROVIDER);
    let mut start: Option<usize> = None;
    let mut offset = 0;
    for line in split_after_newlines(raw) {
        if line.trim() == header {
            start = Some(offset);
            break;
        }
        offset += line.len();
    }
    let Some(start) = start else {
        return String::new();
    };
    let mut block = &raw[start..];
    let re = Regex::new(r"(?m)^\[").unwrap();
    if let Some(m) = re.find(&block[header.len()..]) {
        block = &block[..header.len() + m.start()];
    }
    block.to_string()
}

fn codex_unmarked_current_proxy(raw: &str, endpoint: &str, byok: Option<&CodexBYOK>) -> bool {
    let block = codex_provider_block(raw);
    if block.is_empty() {
        return false;
    }
    block.trim() == render_block(&codex_proxy_block(endpoint, byok)).trim()
}

fn codex_unmarked_legacy_bearer_proxy(raw: &str, endpoint: &str) -> bool {
    let block = codex_provider_block(raw);
    if block.is_empty() {
        return false;
    }
    let re = Regex::new(r#"(?m)^experimental_bearer_token = "[^"]*"$"#).unwrap();
    let rendered = render_block(&codex_proxy_block_with_bearer(endpoint));
    let want = re.replace_all(
        rendered.trim(),
        r#"experimental_bearer_token = "__TOKLESS_BEARER__""#,
    );
    let actual = re.replace_all(
        block.trim(),
        r#"experimental_bearer_token = "__TOKLESS_BEARER__""#,
    );
    actual == want
}

pub fn codex_proxy_owned(raw: &str, endpoint: &str) -> bool {
    let flavor = codex_byok_flavor(raw);
    codex_marked_current_proxy(raw, endpoint)
        || codex_marked_legacy_bearer_proxy(raw, endpoint)
        || codex_legacy_proxy(raw, endpoint)
        || codex_unmarked_current_proxy(raw, endpoint, flavor.as_ref())
        || codex_unmarked_legacy_bearer_proxy(raw, endpoint)
}

pub fn codex_byok_flavor(raw: &str) -> Option<CodexBYOK> {
    let block = codex_provider_block(raw);
    if block.contains("env_http_headers") {
        let mut byok = CodexBYOK::default();
        if block.contains(BYOK_ROUTE_HEADER_KEY) {
            byok.route_header = "__tokless_route__".to_string();
        }
        return Some(byok);
    }
    None
}

fn insert_codex_block_at_root(content: &str, block: &str) -> String {
    let block = block.trim();
    let lines: Vec<&str> = content.split('\n').collect();
    let header_re =
        Regex::new(r"^[ \t]*(?:\[\[[^\]\r\n]+\]\]|\[[^\]\r\n]+\])[ \t]*(?:#.*)?$").unwrap();
    for (i, line) in lines.iter().enumerate() {
        if header_re.is_match(line) {
            let head = lines[..i].join("\n");
            let tail = lines[i..].join("\n");
            let head = head.trim_end_matches('\n');
            let tail = tail.trim_start_matches('\n');
            let prefix = if !head.is_empty() {
                format!("{}\n\n", head)
            } else {
                String::new()
            };
            let combined = format!("{}{}\n\n{}", prefix, block, tail);
            return format!("{}\n", combined.trim_end_matches('\n'));
        }
    }
    let trimmed = format!("{}\n\n{}\n", content.trim_end_matches('\n'), block);
    trimmed.trim_start_matches('\n').to_string()
}

fn codex_proxy_writable(raw: &str, endpoint: &str) -> bool {
    let have_model = codex_root_value(raw, "model_provider");
    if !have_model.is_empty() && have_model != "headroom" {
        return false;
    }
    let have_url = codex_root_value(raw, "openai_base_url");
    if !have_url.is_empty() && !codex_is_managed_endpoint(&have_url) {
        return false;
    }
    if !has_block(raw, CODEX_PROXY_PROVIDER) {
        return have_model.is_empty() && have_url.is_empty();
    }
    codex_proxy_owned(raw, endpoint)
}

pub fn codex_takeover_target(raw: &str) -> Option<CodexBYOK> {
    let id = codex_root_value(raw, "model_provider");
    if id.is_empty() || id == "headroom" {
        return None;
    }
    for b in discover_codex_byok() {
        if b.id == id
            && strip_v1_suffix(&codex_named_provider_base_url(raw, &id))
                == strip_v1_suffix(&b.base_url)
        {
            return Some(b);
        }
    }
    None
}

fn codex_named_provider_base_url(raw: &str, id: &str) -> String {
    let candidates = [
        id.to_string(),
        serde_json::to_string(id).unwrap_or_else(|_| format!("\"{}\"", id)),
        format!("'{}'", id.replace('\'', "''")),
    ];
    let mm = Regex::new(
        r#"(?m)^[ \t]*(?:base_url|"base_url"|'base_url')[ \t]*=[ \t]*(?:"([^"]*)"|'([^']*)')"#,
    )
    .unwrap();
    for header_id in candidates {
        let pat = format!(
            r"(?m)^\[model_providers\.{}\][ \t]*(?:#.*)?$([\s\S]*?)(?:\n\[|\z)",
            regex::escape(&header_id)
        );
        let Ok(re) = Regex::new(&pat) else {
            continue;
        };
        let Some(caps) = re.captures(raw) else {
            continue;
        };
        let body = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        if let Some(vcaps) = mm.captures(body) {
            let v1 = vcaps.get(1).map(|m| m.as_str()).unwrap_or("");
            if !v1.is_empty() {
                return v1.to_string();
            }
            let v2 = vcaps.get(2).map(|m| m.as_str()).unwrap_or("");
            return v2.to_string();
        }
    }
    String::new()
}

fn codex_pick_byok(raw: &str) -> Option<CodexBYOK> {
    let byoks = discover_codex_byok();
    if byoks.is_empty() {
        return None;
    }
    let id = codex_root_value(raw, "model_provider");
    if !id.is_empty() && id != "headroom" {
        return byoks.into_iter().find(|b| b.id == id);
    }
    let route = codex_dot_env_value(CODEX_BYOK_ROUTE_VAR);
    if !route.is_empty() {
        if let Some(found) = byoks
            .iter()
            .find(|b| route.starts_with(&format!("codex:{}.", b.id)))
        {
            return Some(found.clone());
        }
    }
    let url = codex_dot_env_value(CODEX_BYOK_URL_VAR);
    if !url.is_empty() {
        let want = strip_v1_suffix(&url);
        if let Some(found) = byoks.iter().find(|b| strip_v1_suffix(&b.base_url) == want) {
            return Some(found.clone());
        }
    }
    if codex_uses_chatgpt_auth() {
        return None;
    }
    if byoks.len() == 1 {
        return byoks.into_iter().next();
    }
    None
}

fn codex_dot_env_value(key: &str) -> String {
    codex_dot_env_value_present(key).0
}

fn codex_dot_env_value_present(key: &str) -> (String, bool) {
    let raw = read_file_safe(codex_dot_env_path()).unwrap_or_default();
    let prefix = format!("{}=", key);
    for line in raw.split('\n') {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            return (rest.trim().to_string(), true);
        }
    }
    (String::new(), false)
}

pub fn discover_codex_byok() -> Vec<CodexBYOK> {
    let config_path = codex_paths_resolved().config;
    let Some(raw) = read_file_safe(&config_path) else {
        return Vec::new();
    };
    if !codex_toml_well_formed(&raw) {
        return Vec::new();
    }
    let re = Regex::new(
        r#"(?m)^\[model_providers\.((?:"[^"]+")|(?:'(?:[^']|'')+')|[A-Za-z0-9_-]+)\][ \t]*(?:#.*)?$"#,
    )
    .unwrap();
    let mut providers = Vec::new();
    for caps in re.captures_iter(&raw) {
        let raw_id = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let mut id = raw_id.trim_matches(['"', '\'']).to_string();
        if raw_id.starts_with('\'') {
            id = id.replace("''", "'");
        }
        let base = codex_named_provider_field(&raw, raw_id, "base_url");
        if !is_absolute_http(&base) || same_proxy_base(&base, &proxy_endpoint_for("codex")) {
            continue;
        }
        if same_proxy_base(&base, &codex_byok_proxy_endpoint()) {
            continue;
        }
        let env_key = codex_named_provider_field(&raw, raw_id, "env_key");
        let mut api_key = codex_named_provider_field(&raw, raw_id, "api_key");
        let wire_api = codex_named_provider_field(&raw, raw_id, "wire_api");
        if !env_key.is_empty() {
            api_key = std::env::var(&env_key)
                .unwrap_or_default()
                .trim()
                .to_string();
            if api_key.is_empty() {
                api_key = codex_dot_env_value(&env_key);
            }
        }
        if api_key.is_empty() {
            continue;
        }
        let protocol = match wire_api.as_str() {
            "responses" => "openai-responses".to_string(),
            "chat" => "openai-chat".to_string(),
            _ => String::new(),
        };
        providers.push(CodexBYOK {
            id,
            file: config_path.clone(),
            base_url: base,
            api_key,
            protocol,
            route_header: String::new(),
        });
    }
    providers
}

fn codex_named_provider_field(raw: &str, header_id: &str, key: &str) -> String {
    let header = format!("[model_providers.{}]", header_id);
    let Some(start) = raw.find(&header) else {
        return String::new();
    };
    let mut block = &raw[start + header.len()..];
    let next_re = Regex::new(r"(?m)^\[").unwrap();
    if let Some(m) = next_re.find(block) {
        block = &block[..m.start()];
    }
    let k = regex::escape(key);
    let pat = format!(
        r#"(?m)^[ \t]*(?:{k}|"{k}"|'{k}')[ \t]*=[ \t]*(?:"([^"]*)"|'([^']*)')"#
    );
    let re = Regex::new(&pat).unwrap();
    let Some(caps) = re.captures(block) else {
        return String::new();
    };
    let m1 = caps.get(1).map(|m| m.as_str()).unwrap_or("");
    if !m1.is_empty() {
        return m1.to_string();
    }
    caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string()
}

pub fn codex_toml_well_formed(raw: &str) -> bool {
    let mut stack: Vec<u8> = Vec::new();
    let mut in_basic = false;
    let mut in_literal = false;
    let mut escaped = false;
    for line in raw.split('\n') {
        let trimmed = line.trim();
        if (trimmed.starts_with('[') && trimmed.ends_with(']'))
            || (trimmed.starts_with("[[") && trimmed.ends_with("]]"))
        {
            continue;
        }
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i];
            if in_basic {
                if escaped {
                    escaped = false;
                } else if c == b'\\' {
                    escaped = true;
                } else if c == b'"' {
                    in_basic = false;
                }
                i += 1;
                continue;
            }
            if in_literal {
                if c == b'\'' {
                    in_literal = false;
                }
                i += 1;
                continue;
            }
            match c {
                b'"' => in_basic = true,
                b'\'' => in_literal = true,
                b'[' | b'{' => stack.push(c),
                b']' | b'}' => {
                    let Some(top) = stack.pop() else {
                        return false;
                    };
                    if (c == b']' && top != b'[') || (c == b'}' && top != b'{') {
                        return false;
                    }
                }
                b'#' => break,
                _ => {}
            }
            i += 1;
        }
    }
    !in_basic && !in_literal && !escaped && stack.is_empty()
}

pub fn codex_dot_env_path() -> String {
    Path::new(&codex_paths_resolved().dir)
        .join(".env")
        .to_string_lossy()
        .to_string()
}

pub fn upsert_codex_dot_env(kv: &[(&str, &str)], remove: bool) -> bool {
    let path = codex_dot_env_path();
    let raw = read_file_safe(&path).unwrap_or_default();
    let mut out: Vec<String> = Vec::new();
    if !raw.is_empty() {
        for line in raw.trim_end_matches('\n').split('\n') {
            let trimmed = line.trim();
            let managed = kv
                .iter()
                .any(|(k, _)| trimmed.starts_with(&format!("{}=", k)));
            if managed && remove {
                continue;
            }
            if !managed {
                out.push(line.to_string());
            }
        }
    }
    if !remove {
        for (k, v) in kv {
            out.push(format!("{}={}", k, v));
        }
    }
    let mut next = out.join("\n");
    if !next.is_empty() {
        next.push('\n');
    }
    if next == raw {
        return true;
    }
    if write_file_mode_600(&path, &next).is_err() {
        return false;
    }
    chmod_600(&path)
}

pub fn write_codex_byok_dot_env(b: &CodexBYOK) -> bool {
    let stripped_url = strip_v1_suffix(&b.base_url);
    for (k, want) in [
        (CODEX_BYOK_KEY_VAR, b.api_key.as_str()),
        (CODEX_BYOK_URL_VAR, stripped_url.as_str()),
    ] {
        let (val, present) = codex_dot_env_value_present(k);
        if present && val != want {
            return false;
        }
    }
    let mut pairs = vec![
        (CODEX_BYOK_KEY_VAR, b.api_key.as_str()),
        (CODEX_BYOK_URL_VAR, stripped_url.as_str()),
    ];
    if !b.route_header.is_empty() {
        pairs.push((CODEX_BYOK_ROUTE_VAR, b.route_header.as_str()));
    }
    upsert_codex_dot_env(&pairs, false)
}

pub fn remove_codex_byok_dot_env(b: &CodexBYOK) -> bool {
    let stripped_url = strip_v1_suffix(&b.base_url);
    for (k, want) in [
        (CODEX_BYOK_KEY_VAR, b.api_key.as_str()),
        (CODEX_BYOK_URL_VAR, stripped_url.as_str()),
    ] {
        let (val, present) = codex_dot_env_value_present(k);
        if present && val != want {
            return false;
        }
    }
    let mut remove_pairs = vec![(CODEX_BYOK_KEY_VAR, ""), (CODEX_BYOK_URL_VAR, "")];
    if !b.route_header.is_empty() {
        remove_pairs.push((CODEX_BYOK_ROUTE_VAR, ""));
    }
    upsert_codex_dot_env(&remove_pairs, true)
}

fn remove_codex_byok_dot_env_owned(b: &CodexBYOK, stash: &CodexStash) -> bool {
    let mut remove_pairs = Vec::new();
    if !stash.env_key_existed {
        let (val, present) = codex_dot_env_value_present(CODEX_BYOK_KEY_VAR);
        if present && val != b.api_key {
            return false;
        }
        remove_pairs.push((CODEX_BYOK_KEY_VAR, ""));
    }
    if !stash.env_url_existed {
        let (val, present) = codex_dot_env_value_present(CODEX_BYOK_URL_VAR);
        if present && val != strip_v1_suffix(&b.base_url) {
            return false;
        }
        remove_pairs.push((CODEX_BYOK_URL_VAR, ""));
    }
    remove_pairs.is_empty() || upsert_codex_dot_env(&remove_pairs, true)
}

fn codex_stash_path() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("codex.proxy.stash.json")
        .to_string_lossy()
        .to_string()
}

pub fn load_codex_stash() -> (CodexStash, bool) {
    let Some(raw) = read_file_safe(codex_stash_path()) else {
        return (CodexStash::default(), true);
    };
    let Ok(s) = serde_json::from_str::<CodexStash>(&raw) else {
        return (CodexStash::default(), false);
    };
    if s.provider_id.is_empty() && !s.byok {
        return (CodexStash::default(), false);
    }
    (s, true)
}

pub fn save_codex_stash(s: &CodexStash) -> bool {
    let Ok(b) = serde_json::to_string(s) else {
        return false;
    };
    write_file_mode_600(&codex_stash_path(), &b).is_ok()
}

pub fn clear_codex_stash() -> std::io::Result<()> {
    match fs::remove_file(codex_stash_path()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn restore_codex_config(path: &str, raw: &str, existed: bool) -> std::io::Result<()> {
    if existed {
        return write_file(path, raw);
    }
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn configure_codex_proxy() -> (bool, String) {
    let mut changed = false;
    let mut file = codex_paths_resolved().config;
    if let Err(err) = with_proxy_route_stash_lock(|| {
        let (c, f) = configure_codex_proxy_locked();
        changed = c;
        file = f;
        Ok(())
    }) {
        L.err(&format!("codex proxy lock failed: {}", err));
    }
    (changed, file)
}

fn configure_codex_proxy_locked() -> (bool, String) {
    let p = codex_paths_resolved();
    let _ = ensure_dir(&p.dir);
    let raw_opt = read_file_safe(&p.config);
    let ok = raw_opt.is_some();
    if !ok && exists(&p.config) {
        return (false, p.config);
    }
    let mut raw = raw_opt.unwrap_or_default();
    let endpoint = proxy_endpoint_for("codex");
    let mut byok = codex_pick_byok(&raw);
    let dot_env_path = codex_dot_env_path();
    let dot_env_opt = read_file_safe(&dot_env_path);
    let dot_env_existed = dot_env_opt.is_some();
    let dot_env_raw = dot_env_opt.unwrap_or_default();
    let mut took_over = false;
    let stash_path = codex_stash_path();
    let stash_opt = read_file_safe(&stash_path);
    let stash_exists = stash_opt.is_some();
    let stash_raw = stash_opt.unwrap_or_default();
    let mut stash_created = false;
    let mut route_registered = false;
    let mut previous_route: Option<BYOKRoute> = None;

    if !codex_proxy_writable(&raw, &endpoint) {
        let Some(mut takeover) = codex_takeover_target(&raw) else {
            return (false, p.config);
        };
        let (_, key_existed) = codex_dot_env_value_present(CODEX_BYOK_KEY_VAR);
        let (_, url_existed) = codex_dot_env_value_present(CODEX_BYOK_URL_VAR);
        if !save_codex_stash(&CodexStash {
            provider_id: codex_root_value(&raw, "model_provider"),
            openai_base_url: codex_root_value(&raw, "openai_base_url"),
            byok: true,
            env_key_existed: key_existed,
            env_url_existed: url_existed,
            provider_line: codex_root_assignment_line(&raw, "model_provider"),
            url_line: codex_root_assignment_line(&raw, "openai_base_url"),
        }) {
            return (false, p.config);
        }
        stash_created = true;
        took_over = true;
        if !takeover.protocol.is_empty() && !takeover.base_url.is_empty() {
            let normalized_upstream = strip_v1_suffix(&takeover.base_url);
            let (_, route_header, prev, route_ok) = register_byok_route(
                "codex",
                &takeover.id,
                &takeover.protocol,
                &normalized_upstream,
            );
            if route_ok {
                takeover.route_header = route_header;
                route_registered = true;
                previous_route = prev;
            }
        }
        if !takeover.protocol.is_empty() && !takeover.base_url.is_empty() && !route_registered {
            let _ = restore_codex_config(&stash_path, &stash_raw, stash_exists);
            return (false, p.config);
        }
        byok = Some(takeover);
    }

    if let Some(b) = byok.as_mut() {
        if !took_over {
            if stash_exists {
                let (_, valid) = load_codex_stash();
                if !valid {
                    return (false, p.config);
                }
            } else {
                let (_, key_existed) = codex_dot_env_value_present(CODEX_BYOK_KEY_VAR);
                let (_, url_existed) = codex_dot_env_value_present(CODEX_BYOK_URL_VAR);
                if !save_codex_stash(&CodexStash {
                    byok: true,
                    env_key_existed: key_existed,
                    env_url_existed: url_existed,
                    ..Default::default()
                }) {
                    return (false, p.config);
                }
                stash_created = true;
            }
            if !b.protocol.is_empty() && !b.base_url.is_empty() {
                if let Some(route) = read_byok_route(&format!("codex:{}", b.id)) {
                    b.route_header = byok_route_header(&route);
                } else {
                    let (_, route_header, prev, route_ok) = register_byok_route(
                        "codex",
                        &b.id,
                        &b.protocol,
                        &strip_v1_suffix(&b.base_url),
                    );
                    if !route_ok {
                        return (false, p.config);
                    }
                    b.route_header = route_header;
                    previous_route = prev;
                    route_registered = true;
                }
            }
        }
    }

    let mut root_extras: Vec<String> = Vec::new();
    let original = raw.clone();
    if codex_marked_current_proxy(&raw, &endpoint) {
        let (_, extras, _) = codex_matched_marked_section(&raw, &endpoint);
        root_extras = extras;
        raw = strip_codex_managed_block(&raw);
    } else if codex_marked_legacy_bearer_proxy(&raw, &endpoint) {
        raw = strip_codex_managed_block(&raw);
    } else if codex_legacy_proxy(&raw, &endpoint)
        || codex_unmarked_current_proxy(&raw, &endpoint, codex_byok_flavor(&raw).as_ref())
        || codex_unmarked_legacy_bearer_proxy(&raw, &endpoint)
    {
        raw = strip_codex_root_provider_assignments(&raw);
        raw = remove_block(&raw, CODEX_PROXY_PROVIDER);
    }
    if took_over {
        raw = strip_codex_root_provider_assignments(&raw);
    }
    if has_block(&raw, CODEX_PROXY_PROVIDER) {
        return (false, p.config);
    }
    let target = if byok.is_some() {
        codex_byok_proxy_endpoint()
    } else {
        endpoint.clone()
    };
    let mut next = insert_codex_block_at_root(&raw, &codex_proxy_section(&target, byok.as_ref()));
    if !root_extras.is_empty() {
        next = codex_inject_root_extras(&next, &root_extras);
    }
    if next == original {
        if route_registered {
            if let Some(b) = byok.as_ref() {
                rollback_byok_route_logged("codex", &b.id, previous_route.as_ref());
            }
        }
        if stash_created {
            let _ = restore_codex_config(&stash_path, &stash_raw, stash_exists);
        }
        return (false, p.config);
    }
    if write_file(&p.config, &next).is_err() {
        if route_registered {
            if let Some(b) = byok.as_ref() {
                rollback_byok_route_logged("codex", &b.id, previous_route.as_ref());
            }
        }
        if stash_created {
            let _ = restore_codex_config(&stash_path, &stash_raw, stash_exists);
        }
        return (false, p.config);
    }
    if let Some(b) = byok.as_ref() {
        if !write_codex_byok_dot_env(b) {
            if route_registered {
                rollback_byok_route_logged("codex", &b.id, previous_route.as_ref());
            }
            if restore_codex_config(&p.config, &original, ok).is_err() {
                return (false, p.config);
            }
            let _ = restore_codex_config(&dot_env_path, &dot_env_raw, dot_env_existed);
            if stash_created {
                let _ = restore_codex_config(&stash_path, &stash_raw, stash_exists);
            }
            return (false, p.config);
        }
    }
    let _ = chmod_600(&p.config);
    (true, p.config)
}

pub fn remove_codex_proxy() -> bool {
    let mut removed = false;
    if let Err(err) = with_proxy_route_stash_lock(|| {
        removed = remove_codex_proxy_locked();
        Ok(())
    }) {
        L.err(&format!("codex proxy lock failed: {}", err));
    }
    removed
}

fn remove_codex_proxy_locked() -> bool {
    let p = codex_paths_resolved();
    let Some(raw) = read_file_safe(&p.config) else {
        return false;
    };
    let endpoint = proxy_endpoint_for("codex");
    if !codex_proxy_owned(&raw, &endpoint) {
        return false;
    }
    let mut byok = codex_pick_byok(&raw);
    let byok_proxy = codex_provider_block(&raw).contains("env_http_headers");
    if byok_proxy && byok.is_none() {
        byok = Some(CodexBYOK::default());
    }
    let mut next = raw.clone();
    if codex_marked_current_proxy(&next, &endpoint)
        || codex_marked_legacy_bearer_proxy(&next, &endpoint)
    {
        if codex_marked_current_proxy(&next, &endpoint) {
            let (_, root_extras, _) = codex_matched_marked_section(&next, &endpoint);
            let start = next.find(CODEX_MARKER_START).unwrap();
            let end = next.find(CODEX_MARKER_END).unwrap() + CODEX_MARKER_END.len();
            let replacement = if !root_extras.is_empty() {
                format!("{}\n", root_extras.join("\n"))
            } else {
                String::new()
            };
            next = format!("{}{}{}", &next[..start], replacement, &next[end..]);
        } else {
            next = strip_codex_managed_block(&next);
        }
    } else {
        next = strip_codex_root_provider_assignments(&next);
        next = remove_block(&next, CODEX_PROXY_PROVIDER);
    }
    let (stash, stash_ok) = load_codex_stash();
    if !stash_ok {
        return false;
    }
    if !stash.provider_id.is_empty() {
        next = set_toml_top_key(&next, "model_provider", &stash.provider_id);
        if !stash.provider_line.is_empty() {
            next =
                restore_codex_root_assignment_line(&next, "model_provider", &stash.provider_line);
        }
        if !stash.openai_base_url.is_empty() {
            next = set_toml_top_key(&next, "openai_base_url", &stash.openai_base_url);
            if !stash.url_line.is_empty() {
                next =
                    restore_codex_root_assignment_line(&next, "openai_base_url", &stash.url_line);
            }
        } else {
            next = remove_toml_top_key(&next, "openai_base_url");
        }
    }
    next = next.trim().to_string();
    if !next.is_empty() {
        next.push('\n');
    }
    if next == raw {
        return false;
    }
    let dot_env_path = Path::new(&p.dir)
        .join(".env")
        .to_string_lossy()
        .to_string();
    let dot_env_opt = read_file_safe(&dot_env_path);
    let dot_env_existed = dot_env_opt.is_some();
    let dot_env_raw = dot_env_opt.unwrap_or_default();
    if write_file(&p.config, &next).is_err() {
        return false;
    }
    if byok_proxy {
        if let Some(b) = byok.as_ref() {
            let dot_env_ok = if stash.byok {
                remove_codex_byok_dot_env_owned(b, &stash)
            } else {
                remove_codex_byok_dot_env(b)
            };
            if !dot_env_ok {
                let _ = restore_codex_config(&p.config, &raw, true);
                return false;
            }
        }
    }
    if let Some(b) = byok.as_ref() {
        if !b.id.is_empty() {
            if delete_byok_route_for("codex", &b.id).is_err() {
                let _ = restore_codex_config(&p.config, &raw, true);
                let _ = restore_codex_config(&dot_env_path, &dot_env_raw, dot_env_existed);
                return false;
            }
        }
    }
    if !stash.provider_id.is_empty() || stash.byok {
        if clear_codex_stash().is_err() {
            let _ = restore_codex_config(&p.config, &raw, true);
            let _ = restore_codex_config(&dot_env_path, &dot_env_raw, dot_env_existed);
            return false;
        }
    }
    true
}

fn restore_codex_root_assignment_line(raw: &str, key: &str, line: &str) -> String {
    let k = regex::escape(key);
    let pat = format!(r#"(?m)^[ \t]*(?:{k}|"{k}"|'{k}')[ \t]*=.*$"#);
    let re = Regex::new(&pat).unwrap();
    re.replace_all(raw, line).into_owned()
}

fn codex_proxy_section(endpoint: &str, byok: Option<&CodexBYOK>) -> String {
    format!(
        "{}\nmodel_provider = \"headroom\"\nopenai_base_url = \"{}\"\n\n{}\n{}\n",
        CODEX_MARKER_START,
        endpoint,
        render_block(&codex_proxy_block(endpoint, byok)).trim(),
        CODEX_MARKER_END
    )
}

fn codex_proxy_section_with_bearer(endpoint: &str) -> String {
    format!(
        "{}\nmodel_provider = \"headroom\"\nopenai_base_url = \"{}\"\n\n{}\n{}\n",
        CODEX_MARKER_START,
        endpoint,
        render_block(&codex_proxy_block_with_bearer(endpoint)).trim(),
        CODEX_MARKER_END
    )
}

pub fn codex_has_managed_headroom_block(raw: &str, endpoint: &str) -> bool {
    let flavor = codex_byok_flavor(raw);
    codex_marked_current_proxy(raw, endpoint)
        || codex_unmarked_current_proxy(raw, endpoint, flavor.as_ref())
}

pub fn codex_proxy_wired() -> bool {
    let Some(raw) = read_file_safe(codex_paths_resolved().config) else {
        return false;
    };
    let endpoint = proxy_endpoint_for("codex");
    if codex_root_value(&raw, "model_provider") != "headroom" {
        return false;
    }
    if !codex_is_managed_endpoint(&codex_root_value(&raw, "openai_base_url")) {
        return false;
    }
    codex_has_managed_headroom_block(&raw, &endpoint)
}

// --- Codex rtk PreToolUse hook ---

pub fn codex_hooks_file() -> String {
    Path::new(&codex_paths_resolved().dir)
        .join("hooks.json")
        .to_string_lossy()
        .to_string()
}

pub fn codex_hook_command() -> String {
    tokless_command(&["rtk-hook", "codex"])
}

pub fn codex_perm_hook_command() -> String {
    tokless_command(&["codex-perm", "codex"])
}

pub fn codex_projectmem_hook_command() -> String {
    tokless_command(&["projectmem-hook", "claude"])
}

pub fn codex_hook_trust_hash(command: &str) -> String {
    let handler = json!({
        "async": false,
        "command": command,
        "timeout": CODEX_HOOK_TIMEOUT,
        "type": "command",
    });
    let identity = json!({
        "event_name": "pre_tool_use",
        "hooks": [handler],
        "matcher": CODEX_HOOK_MATCHER,
    });
    let b = serde_json::to_vec(&identity).unwrap_or_default();
    format!("sha256:{}", sha256_hex(&b))
}

pub fn codex_perm_hook_trust_hash(command: &str) -> String {
    let handler = json!({
        "async": false,
        "command": command,
        "timeout": CODEX_PERM_HOOK_TIMEOUT,
        "type": "command",
    });
    let identity = json!({
        "event_name": "permission_request",
        "hooks": [handler],
        "matcher": CODEX_PERM_HOOK_MATCHER,
    });
    let b = serde_json::to_vec(&identity).unwrap_or_default();
    format!("sha256:{}", sha256_hex(&b))
}

pub fn codex_projectmem_hook_trust_hash(command: &str) -> String {
    let handler = json!({
        "async": false,
        "command": command,
        "timeout": CODEX_PROJECTMEM_HOOK_TIMEOUT,
        "type": "command",
    });
    let identity = json!({
        "event_name": "session_start",
        "hooks": [handler],
    });
    let b = serde_json::to_vec(&identity).unwrap_or_default();
    format!("sha256:{}", sha256_hex(&b))
}

fn codex_group_has_managed(group: &OrderedMap, args: &[&str]) -> bool {
    let Some(Value::Array(arr)) = group.get("hooks") else {
        return false;
    };
    for h in arr {
        let Some(hm) = h.as_object() else {
            continue;
        };
        let cmd = hm.get("command").and_then(|v| v.as_str()).unwrap_or("");
        if tokless_managed_command(cmd, args) {
            return true;
        }
    }
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CodexHookPos {
    group: isize,
    hook: isize,
}

fn codex_transform_managed_groups(
    groups: &[Value],
    matcher: &str,
    args: &[&str],
    desired: Option<Value>,
) -> (
    Vec<Value>,
    CodexHookPos,
    HashMap<CodexHookPos, CodexHookPos>,
    Vec<CodexHookPos>,
) {
    let mut out = Vec::with_capacity(groups.len() + 1);
    let mut desired_pos = CodexHookPos { group: -1, hook: -1 };
    let mut moved = HashMap::new();
    let mut removed = Vec::new();
    let desired_hook = desired
        .as_ref()
        .and_then(|d| d.as_object())
        .and_then(|m| m.get("hooks"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .cloned();

    for (old_group, g) in groups.iter().enumerate() {
        let Some(mut gm) = g.as_object().cloned() else {
            out.push(g.clone());
            continue;
        };
        let group_matcher = gm.get("matcher").and_then(|v| v.as_str()).unwrap_or("");
        let Some(hooks) = gm.get("hooks").and_then(|v| v.as_array()).cloned() else {
            out.push(g.clone());
            continue;
        };
        let mut kept = Vec::with_capacity(hooks.len());
        for (old_hook, h) in hooks.iter().enumerate() {
            let mut managed = false;
            if group_matcher == matcher {
                if let Some(hm) = h.as_object() {
                    let cmd = hm.get("command").and_then(|v| v.as_str()).unwrap_or("");
                    managed = tokless_managed_command(cmd, args);
                }
            }
            if managed {
                if let Some(dh) = desired_hook.as_ref() {
                    if desired_pos.group == -1 {
                        desired_pos = CodexHookPos {
                            group: out.len() as isize,
                            hook: kept.len() as isize,
                        };
                        kept.push(dh.clone());
                        continue;
                    }
                }
                removed.push(CodexHookPos {
                    group: old_group as isize,
                    hook: old_hook as isize,
                });
                continue;
            }
            let new_pos = CodexHookPos {
                group: out.len() as isize,
                hook: kept.len() as isize,
            };
            moved.insert(
                CodexHookPos {
                    group: old_group as isize,
                    hook: old_hook as isize,
                },
                new_pos,
            );
            kept.push(h.clone());
        }
        if !kept.is_empty() {
            gm.insert("hooks".to_string(), Value::Array(kept));
            out.push(Value::Object(gm));
        } else if hooks.is_empty() {
            out.push(Value::Object(gm));
        }
    }
    if desired_hook.is_some() && desired_pos.group == -1 {
        if let Some(d) = desired {
            desired_pos = CodexHookPos {
                group: out.len() as isize,
                hook: 0,
            };
            out.push(d);
        }
    }
    (out, desired_pos, moved, removed)
}

fn codex_rewrite_hook_state(
    mut raw: String,
    hooks_file: &str,
    event: &str,
    moved: &HashMap<CodexHookPos, CodexHookPos>,
    removed: &[CodexHookPos],
) -> String {
    for pos in removed {
        let key = format!("{}:{}:{}:{}", hooks_file, event, pos.group, pos.hook);
        raw = remove_block(&raw, &codex_hook_state_header(&key));
    }
    let mut moves = Vec::new();
    for (old_pos, new_pos) in moved {
        if old_pos == new_pos {
            continue;
        }
        let old_key = format!(
            "{}:{}:{}:{}",
            hooks_file, event, old_pos.group, old_pos.hook
        );
        let new_key = format!(
            "{}:{}:{}:{}",
            hooks_file, event, new_pos.group, new_pos.hook
        );
        let placeholder = format!("hooks.state.'__tokless_move_{}__'", moves.len());
        let old_header = format!("[{}]", codex_hook_state_header(&old_key));
        if raw.contains(&old_header) {
            raw = raw.replacen(&old_header, &format!("[{}]", placeholder), 1);
            moves.push((placeholder, codex_hook_state_header(&new_key)));
        }
    }
    for (placeholder, new_header) in moves {
        raw = raw.replacen(
            &format!("[{}]", placeholder),
            &format!("[{}]", new_header),
            1,
        );
    }
    raw
}

fn codex_rtk_group(command: &str) -> Value {
    json!({
        "matcher": CODEX_HOOK_MATCHER,
        "hooks": [{
            "type": "command",
            "command": command,
            "timeout": CODEX_HOOK_TIMEOUT,
        }],
    })
}

fn codex_perm_group(command: &str) -> Value {
    json!({
        "matcher": CODEX_PERM_HOOK_MATCHER,
        "hooks": [{
            "type": "command",
            "command": command,
            "timeout": CODEX_PERM_HOOK_TIMEOUT,
        }],
    })
}

fn codex_projectmem_group(command: &str) -> Value {
    json!({
        "hooks": [{
            "type": "command",
            "command": command,
            "timeout": CODEX_PROJECTMEM_HOOK_TIMEOUT,
        }],
    })
}

pub fn install_codex_rtk_hook() -> bool {
    let p = codex_paths_resolved();
    if ensure_dir(&p.dir).is_err() {
        return false;
    }
    let command = codex_hook_command();
    if command.is_empty() {
        return false;
    }
    let hooks_file = codex_hooks_file();
    let rules_file = codex_rules_file();
    let mut snapshots = Vec::with_capacity(3);
    for path in [&hooks_file, &p.config, &rules_file] {
        let raw_opt = read_file_safe(path);
        snapshots.push((
            path.clone(),
            raw_opt.clone().unwrap_or_default(),
            raw_opt.is_some(),
        ));
    }
    let rollback = || {
        for (path, raw, existed) in &snapshots {
            if *existed {
                let _ = write_file(path, raw);
            } else {
                let _ = fs::remove_file(path);
            }
        }
    };

    let raw_opt = read_file_safe(&hooks_file);
    let ok = raw_opt.is_some();
    let raw = raw_opt.unwrap_or_default();
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(v) if v.is_object() => v,
        _ => {
            if ok && !raw.trim().is_empty() {
                rollback();
                return false;
            }
            Value::Object(OrderedMap::new())
        }
    };
    {
        let root = cfg.as_object_mut().unwrap();
        if !root.contains_key("hooks") {
            root.insert("hooks".to_string(), Value::Object(OrderedMap::new()));
        }
        let Some(hooks) = root.get_mut("hooks").and_then(|v| v.as_object_mut()) else {
            rollback();
            return false;
        };
        let pre_arr = match hooks.get("PreToolUse") {
            Some(Value::Array(arr)) => arr.clone(),
            Some(_) => {
                rollback();
                return false;
            }
            None => Vec::new(),
        };
        let (new_pre, pos, moved, removed) = codex_transform_managed_groups(
            &pre_arr,
            CODEX_HOOK_MATCHER,
            &["rtk-hook", "codex"],
            Some(codex_rtk_group(&command)),
        );
        hooks.insert("PreToolUse".to_string(), Value::Array(new_pre));
        let next = stringify_json(&cfg);
        if next != raw {
            if write_file(&hooks_file, &next).is_err() {
                rollback();
                return false;
            }
        }

        let craw = read_file_safe(&p.config).unwrap_or_default();
        let craw_swept = sweep_stale_hook_state_entries(&craw);
        let craw_rewritten =
            codex_rewrite_hook_state(craw_swept, &hooks_file, "pre_tool_use", &moved, &removed);
        let key = format!("{}:pre_tool_use:{}:{}", hooks_file, pos.group, pos.hook);
        let mut block = TomlBlock::new(codex_hook_state_header(&key));
        block.set("trusted_hash", codex_hook_trust_hash(&command));
        let mut cnext = upsert_block(&craw_rewritten, &block, false);
        if get_toml_top_key(&cnext, "approval_policy").is_empty() {
            cnext = set_toml_top_key(&cnext, "approval_policy", "on-request");
            cnext = codex_add_marker(&cnext, CODEX_APPROVAL_POLICY_MARKER);
        }
        let mut features = TomlBlock::new("features");
        features.set("hooks", true);
        cnext = upsert_block(&cnext, &features, true);
        if !cnext.contains(CODEX_FEATURES_HOOK_MARKER) {
            if let Some(i) = cnext.find("[features]") {
                if let Some(rel_end) = cnext[i..].find('\n') {
                    let line_end = i + rel_end + 1;
                    cnext = format!(
                        "{}{}{}",
                        &cnext[..line_end],
                        CODEX_FEATURES_HOOK_MARKER,
                        &cnext[line_end..]
                    );
                }
            }
        }
        if cnext != craw {
            if write_file(&p.config, &cnext).is_err() {
                rollback();
                return false;
            }
        }
    }

    if !has_codex_rtk_hook() || !install_codex_permission_hook() || !install_codex_rules_allowlist()
    {
        rollback();
        return false;
    }
    true
}

pub fn remove_codex_rtk_hook() {
    let p = codex_paths_resolved();
    let hooks_file = codex_hooks_file();
    if let Some(raw) = read_file_safe(&hooks_file) {
        if let Some(mut cfg) = try_parse_jsonc(&raw) {
            if let Some(hooks) = cfg
                .as_object_mut()
                .and_then(|m| m.get_mut("hooks"))
                .and_then(|v| v.as_object_mut())
            {
                if let Some(pre_arr) = hooks
                    .get("PreToolUse")
                    .and_then(|v| v.as_array())
                    .cloned()
                {
                    let (kept, _, moved, removed) = codex_transform_managed_groups(
                        &pre_arr,
                        CODEX_HOOK_MATCHER,
                        &["rtk-hook", "codex"],
                        None,
                    );
                    if !removed.is_empty() {
                        hooks.insert("PreToolUse".to_string(), Value::Array(kept));
                        let _ = write_file(&hooks_file, &stringify_json(&cfg));
                        let craw = read_file_safe(&p.config).unwrap_or_default();
                        let cnext = codex_rewrite_hook_state(
                            craw.clone(),
                            &hooks_file,
                            "pre_tool_use",
                            &moved,
                            &removed,
                        );
                        if cnext != craw {
                            let _ = write_file(&p.config, &cnext);
                        }
                    }
                }
            }
        }
    }
    remove_codex_permission_hook();
    remove_codex_rules_allowlist();
    codex_cleanup_orphaned_config();
}

pub fn has_codex_rtk_hook() -> bool {
    let Some(raw) = read_file_safe(codex_hooks_file()) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(pre_arr) = cfg
        .as_object()
        .and_then(|m| m.get("hooks"))
        .and_then(|v| v.as_object())
        .and_then(|m| m.get("PreToolUse"))
        .and_then(|v| v.as_array())
    else {
        return false;
    };
    pre_arr
        .iter()
        .filter_map(|g| g.as_object())
        .any(|gm| codex_group_has_managed(gm, &["rtk-hook", "codex"]))
}

pub fn install_codex_permission_hook() -> bool {
    let p = codex_paths_resolved();
    if ensure_dir(&p.dir).is_err() {
        return false;
    }
    let command = codex_perm_hook_command();
    if command.is_empty() {
        return false;
    }
    let hooks_file = codex_hooks_file();
    let raw_opt = read_file_safe(&hooks_file);
    let ok = raw_opt.is_some();
    let raw = raw_opt.unwrap_or_default();
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(v) if v.is_object() => v,
        _ => {
            if ok && !raw.trim().is_empty() {
                return false;
            }
            Value::Object(OrderedMap::new())
        }
    };
    let root = cfg.as_object_mut().unwrap();
    if !root.contains_key("hooks") {
        root.insert("hooks".to_string(), Value::Object(OrderedMap::new()));
    }
    let Some(hooks) = root.get_mut("hooks").and_then(|v| v.as_object_mut()) else {
        return false;
    };
    let perm_arr = match hooks.get("PermissionRequest") {
        Some(Value::Array(arr)) => arr.clone(),
        Some(_) => return false,
        None => Vec::new(),
    };
    let (new_perm, pos, moved, removed) = codex_transform_managed_groups(
        &perm_arr,
        CODEX_PERM_HOOK_MATCHER,
        &["codex-perm", "codex"],
        Some(codex_perm_group(&command)),
    );
    hooks.insert("PermissionRequest".to_string(), Value::Array(new_perm));
    let next = stringify_json(&cfg);
    if next != raw {
        if write_file(&hooks_file, &next).is_err() {
            return false;
        }
    }
    let craw = read_file_safe(&p.config).unwrap_or_default();
    let craw_swept = sweep_stale_hook_state_entries(&craw);
    let craw_rewritten = codex_rewrite_hook_state(
        craw_swept,
        &hooks_file,
        "permission_request",
        &moved,
        &removed,
    );
    let key = format!("{}:permission_request:{}:{}", hooks_file, pos.group, pos.hook);
    let mut block = TomlBlock::new(codex_hook_state_header(&key));
    block.set("trusted_hash", codex_perm_hook_trust_hash(&command));
    let cnext = upsert_block(&craw_rewritten, &block, false);
    if cnext != craw {
        if write_file(&p.config, &cnext).is_err() {
            return false;
        }
    }
    has_codex_permission_hook()
}

pub fn remove_codex_permission_hook() {
    let p = codex_paths_resolved();
    let hooks_file = codex_hooks_file();
    if let Some(raw) = read_file_safe(&hooks_file) {
        if let Some(mut cfg) = try_parse_jsonc(&raw) {
            if let Some(hooks) = cfg
                .as_object_mut()
                .and_then(|m| m.get_mut("hooks"))
                .and_then(|v| v.as_object_mut())
            {
                if let Some(perm_arr) = hooks
                    .get("PermissionRequest")
                    .and_then(|v| v.as_array())
                    .cloned()
                {
                    let (kept, _, moved, removed) = codex_transform_managed_groups(
                        &perm_arr,
                        CODEX_PERM_HOOK_MATCHER,
                        &["codex-perm", "codex"],
                        None,
                    );
                    if !removed.is_empty() {
                        hooks.insert("PermissionRequest".to_string(), Value::Array(kept));
                        let _ = write_file(&hooks_file, &stringify_json(&cfg));
                        let craw = read_file_safe(&p.config).unwrap_or_default();
                        let cnext = codex_rewrite_hook_state(
                            craw.clone(),
                            &hooks_file,
                            "permission_request",
                            &moved,
                            &removed,
                        );
                        if cnext != craw {
                            let _ = write_file(&p.config, &cnext);
                        }
                    }
                }
            }
        }
    }
}

pub fn has_codex_permission_hook() -> bool {
    let Some(raw) = read_file_safe(codex_hooks_file()) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(perm_arr) = cfg
        .as_object()
        .and_then(|m| m.get("hooks"))
        .and_then(|v| v.as_object())
        .and_then(|m| m.get("PermissionRequest"))
        .and_then(|v| v.as_array())
    else {
        return false;
    };
    perm_arr
        .iter()
        .filter_map(|g| g.as_object())
        .any(|gm| codex_group_has_managed(gm, &["codex-perm", "codex"]))
}

pub fn codex_rules_file() -> String {
    Path::new(&codex_paths_resolved().dir)
        .join("rules")
        .join("default.rules")
        .to_string_lossy()
        .to_string()
}

pub fn install_codex_rules_allowlist() -> bool {
    let rules_file = codex_rules_file();
    if exists(&rules_file) {
        return true;
    }
    if let Some(parent) = Path::new(&rules_file).parent() {
        if ensure_dir(parent).is_err() {
            return false;
        }
    }
    let content = r#"# tokless-managed codex allowlist — our tools pre-approved, everything else prompts.

prefix_rule(pattern = ["rtk"], decision = "allow")
prefix_rule(pattern = ["tokless"], decision = "allow")
prefix_rule(pattern = ["git"], decision = "allow")
prefix_rule(pattern = ["cd"], decision = "allow")
prefix_rule(pattern = ["ls"], decision = "allow")
prefix_rule(pattern = ["node"], decision = "allow")
prefix_rule(pattern = ["npm"], decision = "allow")
prefix_rule(pattern = ["npx"], decision = "allow")
prefix_rule(pattern = ["context-mode"], decision = "allow")
prefix_rule(pattern = ["codegraph"], decision = "allow")
prefix_rule(pattern = ["cat"], decision = "allow")
prefix_rule(pattern = ["head"], decision = "allow")
prefix_rule(pattern = ["tail"], decision = "allow")
prefix_rule(pattern = ["grep"], decision = "allow")
prefix_rule(pattern = ["find"], decision = "allow")
prefix_rule(pattern = ["pwd"], decision = "allow")
prefix_rule(pattern = ["which"], decision = "allow")
prefix_rule(pattern = ["echo"], decision = "allow")
prefix_rule(pattern = ["bash"], decision = "allow")
prefix_rule(pattern = ["sh"], decision = "allow")
"#;
    if write_file(&rules_file, content).is_err() {
        return false;
    }
    has_codex_rules_allowlist()
}

pub fn remove_codex_rules_allowlist() {
    if !has_codex_rules_allowlist() {
        return;
    }
    let rules_file = codex_rules_file();
    let _ = fs::remove_file(&rules_file);
    if let Some(parent) = Path::new(&rules_file).parent() {
        let _ = fs::remove_dir(parent);
    }
}

pub fn has_codex_rules_allowlist() -> bool {
    let rules_file = codex_rules_file();
    if !exists(&rules_file) {
        return false;
    }
    let Some(raw) = read_file_safe(&rules_file) else {
        return false;
    };
    raw.contains("tokless-managed codex allowlist")
}

pub fn apply_codex_approval_policy(raw: &str) -> String {
    if get_toml_top_key(raw, "approval_policy").is_empty() {
        return set_toml_top_key(raw, "approval_policy", "on-request");
    }
    raw.to_string()
}

pub fn remove_codex_features_hook(raw: &str) -> (String, bool) {
    let Some(bt) = block_text(raw, "features") else {
        return (raw.to_string(), false);
    };
    let re_hook =
        Regex::new(r"(?m)^[ \t]*hooks[ \t]*=[ \t]*true\b([ \t]*#.*)?([ \t]*\r?\n|$)").unwrap();
    if !re_hook.is_match(&bt) {
        return (raw.to_string(), false);
    }
    let cleaned = re_hook.replace_all(&bt, "$1$2").into_owned();
    let re_any_key = Regex::new(r"(?m)^[ \t]*[A-Za-z0-9_-]+[ \t]*=").unwrap();
    if !re_any_key.is_match(&cleaned) {
        let re_child_table = Regex::new(r"(?m)^\[features\.[^\]]+\]").unwrap();
        if !re_child_table.is_match(&bt) {
            return (remove_block(raw, "features"), true);
        }
    }
    (raw.replacen(&bt, &cleaned, 1), true)
}

fn codex_add_marker(raw: &str, marker: &str) -> String {
    if raw.contains(marker) {
        return raw.to_string();
    }
    if let Some(i) = raw.find('\n') {
        return format!("{}{}{}", &raw[..=i], marker, &raw[i + 1..]);
    }
    format!("{}\n{}", raw, marker)
}

pub fn codex_cleanup_orphaned_config() {
    let p = codex_paths_resolved();
    let Some(mut raw) = read_file_safe(&p.config) else {
        return;
    };
    let mut changed = false;
    if !codex_has_any_tokless_hook() {
        if raw.contains(CODEX_FEATURES_HOOK_MARKER) {
            let (next, ok) = remove_codex_features_hook(&raw);
            if ok {
                raw = next.replacen(CODEX_FEATURES_HOOK_MARKER, "", 1);
                changed = true;
            }
        }
        if raw.contains(CODEX_APPROVAL_POLICY_MARKER) {
            raw = raw.replacen(CODEX_APPROVAL_POLICY_MARKER, "", 1);
            if get_toml_top_key(&raw, "approval_policy") == "on-request" {
                raw = remove_toml_top_key(&raw, "approval_policy");
            }
            changed = true;
        }
    }
    if changed {
        let _ = write_file(&p.config, &raw);
    }
}

pub fn codex_has_any_tokless_hook() -> bool {
    has_codex_rtk_hook() || has_codex_permission_hook()
}

pub fn codex_known_bin_dirs() -> Vec<String> {
    let mut dirs = Vec::new();
    if let Ok(d) = std::env::var("CODEX_INSTALL_DIR") {
        if !d.is_empty() {
            dirs.push(d);
        }
    }
    if IS_WIN {
        if let Ok(la) = std::env::var("LOCALAPPDATA") {
            if !la.is_empty() {
                dirs.push(
                    Path::new(&la)
                        .join("Programs")
                        .join("OpenAI")
                        .join("Codex")
                        .join("bin")
                        .to_string_lossy()
                        .to_string(),
                );
            }
        }
    }
    let h = home();
    dirs.push(
        Path::new(&h)
            .join(".local")
            .join("bin")
            .to_string_lossy()
            .to_string(),
    );
    dirs.push(
        Path::new(&h)
            .join(".cargo")
            .join("bin")
            .to_string_lossy()
            .to_string(),
    );
    dirs
}

// --- projectmem SessionStart hook ---

pub fn install_codex_projectmem_hook() -> bool {
    let p = codex_paths_resolved();
    if ensure_dir(&p.dir).is_err() {
        return false;
    }
    let command = codex_projectmem_hook_command();
    if command.is_empty() {
        return false;
    }
    let hooks_file = codex_hooks_file();
    let raw_opt = read_file_safe(&hooks_file);
    let ok = raw_opt.is_some();
    let raw = raw_opt.unwrap_or_default();
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(v) if v.is_object() => v,
        _ => {
            if ok && !raw.trim().is_empty() {
                return false;
            }
            Value::Object(OrderedMap::new())
        }
    };
    let root = cfg.as_object_mut().unwrap();
    if !root.contains_key("hooks") {
        root.insert("hooks".to_string(), Value::Object(OrderedMap::new()));
    }
    let Some(hooks) = root.get_mut("hooks").and_then(|v| v.as_object_mut()) else {
        return false;
    };
    let arr = match hooks.get("SessionStart") {
        Some(Value::Array(a)) => a.clone(),
        Some(_) => return false,
        None => Vec::new(),
    };
    let (new_arr, pos, moved, removed) = codex_transform_managed_groups(
        &arr,
        "",
        &["projectmem-hook", "claude"],
        Some(codex_projectmem_group(&command)),
    );
    hooks.insert("SessionStart".to_string(), Value::Array(new_arr));
    let next = stringify_json(&cfg);
    if next != raw {
        if write_file(&hooks_file, &next).is_err() {
            return false;
        }
    }
    let craw = read_file_safe(&p.config).unwrap_or_default();
    let craw_swept = sweep_stale_hook_state_entries(&craw);
    let craw_rewritten =
        codex_rewrite_hook_state(craw_swept, &hooks_file, "session_start", &moved, &removed);
    let key = format!("{}:session_start:{}:{}", hooks_file, pos.group, pos.hook);
    let mut block = TomlBlock::new(codex_hook_state_header(&key));
    block.set("trusted_hash", codex_projectmem_hook_trust_hash(&command));
    let mut cnext = upsert_block(&craw_rewritten, &block, false);
    let mut features = TomlBlock::new("features");
    features.set("hooks", true);
    cnext = upsert_block(&cnext, &features, true);
    if cnext != craw {
        if write_file(&p.config, &cnext).is_err() {
            return false;
        }
    }
    has_codex_projectmem_hook()
}

pub fn remove_codex_projectmem_hook() {
    let p = codex_paths_resolved();
    let hooks_file = codex_hooks_file();
    let Some(raw) = read_file_safe(&hooks_file) else {
        return;
    };
    let Some(mut cfg) = try_parse_jsonc(&raw) else {
        return;
    };
    let Some(hooks) = cfg
        .as_object_mut()
        .and_then(|m| m.get_mut("hooks"))
        .and_then(|v| v.as_object_mut())
    else {
        return;
    };
    let Some(arr) = hooks
        .get("SessionStart")
        .and_then(|v| v.as_array())
        .cloned()
    else {
        return;
    };
    let (kept, _, moved, removed) =
        codex_transform_managed_groups(&arr, "", &["projectmem-hook", "claude"], None);
    if removed.is_empty() {
        return;
    }
    if kept.is_empty() {
        hooks.remove("SessionStart");
    } else {
        hooks.insert("SessionStart".to_string(), Value::Array(kept));
    }
    let _ = write_file(&hooks_file, &stringify_json(&cfg));
    let craw = read_file_safe(&p.config).unwrap_or_default();
    let cnext =
        codex_rewrite_hook_state(craw.clone(), &hooks_file, "session_start", &moved, &removed);
    if cnext != craw {
        let _ = write_file(&p.config, &cnext);
    }
}

pub fn has_codex_projectmem_hook() -> bool {
    let hooks_file = codex_hooks_file();
    let Some(raw) = read_file_safe(&hooks_file) else {
        return false;
    };
    let Some(cfg) = try_parse_jsonc(&raw) else {
        return false;
    };
    let Some(arr) = cfg
        .as_object()
        .and_then(|m| m.get("hooks"))
        .and_then(|v| v.as_object())
        .and_then(|m| m.get("SessionStart"))
        .and_then(|v| v.as_array())
    else {
        return false;
    };
    for (gi, g) in arr.iter().enumerate() {
        let Some(gm) = g.as_object() else {
            continue;
        };
        let Some(hs) = gm.get("hooks").and_then(|v| v.as_array()) else {
            continue;
        };
        for (hi, h) in hs.iter().enumerate() {
            let Some(hm) = h.as_object() else {
                continue;
            };
            let cmd = hm.get("command").and_then(|v| v.as_str()).unwrap_or("");
            if !tokless_managed_command(cmd, &["projectmem-hook", "claude"]) {
                continue;
            }
            let craw = read_file_safe(codex_paths_resolved().config).unwrap_or_default();
            let key = format!("{}:session_start:{}:{}", hooks_file, gi, hi);
            return craw.contains(&format!("[{}]", codex_hook_state_header(&key)));
        }
    }
    false
}
