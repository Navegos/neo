// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/agents/proxy_provider.rs
// created: 2026-10-11
// lastModified: 2026-10-14

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::core::Detection;
use crate::util::byok::{
    self, BYOKRoute, byok_gateway_endpoint, byok_route_header, byok_route_protocol,
    upsert_byok_route, valid_byok_upstream,
};
use crate::util::headroom::headroom_paths_resolved;
use crate::util::headroom_proxy::{headroom_proxy_openai_url, headroom_proxy_url};
use crate::util::jsonc::{OrderedMap, Value};
use crate::util::logger::L;
use crate::util::paths;

pub const PROXY_PROVIDER_ENV: &str = "TOKLESS_PROXY_PROVIDER";
pub const HEADROOM_BASE_URL_HEADER: &str = "x-headroom-base-url";
pub const ERR_NO_STABLE_TOKLESS: &str =
    "cannot resolve a stable tokless executable; refusing to persist an empty hook command";

#[derive(Debug, Clone)]
pub(crate) struct ByokRouteRollback {
    pub id: String,
    pub previous: BYOKRoute,
}

/// ProviderModel is the model metadata a provider-block spec carries.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProviderModel {
    pub id: String,
    pub display: String,
    pub context: i64,
    pub output: i64,
    pub reasoning: bool,
    pub tool_call: bool,
}

/// ProviderSpec is one selectable proxy backend identity.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProviderSpec {
    pub id: String,
    pub key: String,
    pub npm: String,
    pub name: String,
    pub key_env: String,
    pub models: Vec<ProviderModel>,
}

pub fn default_provider_spec() -> ProviderSpec {
    ProviderSpec {
        id: "headroom".to_string(),
        key: "headroom".to_string(),
        npm: "@ai-sdk/openai-compatible".to_string(),
        name: "Headroom Proxy".to_string(),
        key_env: String::new(),
        models: vec![
            ProviderModel {
                id: "gpt-4o".to_string(),
                display: "GPT-4o".to_string(),
                context: 128000,
                output: 16384,
                reasoning: false,
                tool_call: false,
            },
            ProviderModel {
                id: "gpt-4.1".to_string(),
                display: "GPT-4.1".to_string(),
                context: 1048576,
                output: 32768,
                reasoning: false,
                tool_call: false,
            },
        ],
    }
}

/// provider_spec_active returns the managed headroom proxy identity.
pub fn provider_spec_active() -> ProviderSpec {
    default_provider_spec()
}

/// proxy_wire_model returns the model id tokless pins into OpenAI-compatible agent configs.
pub fn proxy_wire_model() -> String {
    if let Ok(v) = std::env::var("TOKLESS_PROXY_MODEL") {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    "headroom".to_string()
}

/// proxy_wire_key returns the API key tokless pins into agent configs that have
/// no environment-key mechanism.
pub fn proxy_wire_key() -> String {
    if let Ok(v) = std::env::var("TOKLESS_PROXY_KEY") {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    "tokless".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProxyRouteStashEntry {
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub upstream: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub base_key: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub had_base_key: bool,
    #[serde(default, skip_serializing_if = "is_false", rename = "had_auth_token")]
    pub had_auth: bool,
    #[serde(default, skip_serializing_if = "is_false", rename = "moved_base_key")]
    pub moved_base: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub had_header: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub header: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub base_line: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub header_line: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub managed_native: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub original: Vec<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub managed: Vec<u8>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ProxyRouteStashFile {
    pub providers: Option<HashMap<String, ProxyRouteStashEntry>>,
}

pub fn proxy_route_stash_path(agent: &str) -> String {
    Path::new(&headroom_paths_resolved().root)
        .join(format!("{agent}.byok.stash.json"))
        .to_string_lossy()
        .into_owned()
}

pub fn proxy_route_stash_lock() -> String {
    Path::new(&headroom_paths_resolved().root)
        .join("byok.stash.lock")
        .to_string_lossy()
        .into_owned()
}

#[cfg(windows)]
pub fn proxy_process_alive(pid: u32) -> bool {
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    const STILL_ACTIVE: u32 = 259;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn OpenProcess(
            dwDesiredAccess: u32,
            bInheritHandle: i32,
            dwProcessId: u32,
        ) -> *mut std::ffi::c_void;
        fn GetExitCodeProcess(hProcess: *mut std::ffi::c_void, lpExitCode: *mut u32) -> i32;
        fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
    }

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return false;
        }
        let mut code: u32 = 0;
        let ret = GetExitCodeProcess(handle, &mut code as *mut u32);
        let _ = CloseHandle(handle);
        ret != 0 && code == STILL_ACTIVE
    }
}

#[cfg(not(windows))]
pub fn proxy_process_alive(pid: u32) -> bool {
    unsafe extern "C" {
        fn kill(pid: i32, sig: i32) -> i32;
    }
    if pid == 0 {
        return false;
    }
    unsafe { kill(pid as i32, 0) == 0 }
}

/// with_proxy_route_stash_lock runs f with an exclusive advisory lock held for the
/// duration of the call.
pub fn with_proxy_route_stash_lock<F>(f: F) -> Result<(), String>
where
    F: FnOnce() -> Result<(), String>,
{
    paths::ensure_dir(&headroom_paths_resolved().root).map_err(|e| e.to_string())?;
    let path = proxy_route_stash_lock();
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let token = format!("{}-{}", std::process::id(), now_nanos);

    loop {
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
                }
                if let Err(e) = file.write_all(token.as_bytes()) {
                    drop(file);
                    let _ = fs::remove_file(&path);
                    return Err(e.to_string());
                }
                drop(file);
                let res = f();
                if let Ok(content) = fs::read_to_string(&path)
                    && content == token
                {
                    let _ = fs::remove_file(&path);
                }
                return res;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let Ok(meta) = fs::metadata(&path) else {
                    continue;
                };
                let age = meta
                    .modified()
                    .ok()
                    .and_then(|m| SystemTime::now().duration_since(m).ok())
                    .unwrap_or_default();
                if age > Duration::from_secs(10) {
                    let content = fs::read_to_string(&path).ok();
                    let pid = content
                        .as_deref()
                        .and_then(|c| c.trim().split('-').next())
                        .and_then(|p| p.parse::<u32>().ok())
                        .unwrap_or(0);
                    if pid > 0 && pid == std::process::id() {
                        std::thread::sleep(Duration::from_millis(50));
                        continue;
                    }
                    let live = pid > 0 && proxy_process_alive(pid);
                    if !live {
                        let stale_nanos = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos();
                        let stale = format!("{path}.stale.{stale_nanos}");
                        if fs::rename(&path, &stale).is_ok() {
                            let _ = fs::remove_file(&stale);
                            continue;
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(e.to_string()),
        }
    }
}

pub fn stash_txn<F>(agent: &str, f: F) -> Result<(), String>
where
    F: FnOnce(&mut HashMap<String, ProxyRouteStashEntry>) -> Result<(), String>,
{
    with_proxy_route_stash_lock(|| {
        let mut all = load_proxy_route_stash_locked(agent);
        f(&mut all)
    })
}

pub fn load_proxy_route_stash_locked(agent: &str) -> HashMap<String, ProxyRouteStashEntry> {
    let Some(raw) = paths::read_file_safe(proxy_route_stash_path(agent)) else {
        return HashMap::new();
    };
    let Ok(f) = serde_json::from_str::<ProxyRouteStashFile>(&raw) else {
        return HashMap::new();
    };
    f.providers.unwrap_or_default()
}

pub fn proxy_route_stash_valid(agent: &str) -> bool {
    let Some(raw) = paths::read_file_safe(proxy_route_stash_path(agent)) else {
        return true;
    };
    let Ok(f) = serde_json::from_str::<ProxyRouteStashFile>(&raw) else {
        return false;
    };
    f.providers.is_some()
}

pub fn restore_proxy_route_stash(agent: &str, raw: &str, exists: bool) -> Result<(), String> {
    let path = proxy_route_stash_path(agent);
    if exists {
        return paths::write_file_mode(&path, raw, 0o600).map_err(|e| e.to_string());
    }
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn restore_proxy_route_stash_logged(agent: &str, raw: &str, exists: bool) -> bool {
    if let Err(err) = restore_proxy_route_stash(agent, raw, exists) {
        L.err(&format!("{agent} proxy stash rollback failed: {err}"));
        return false;
    }
    true
}

pub fn save_proxy_route_stash(
    agent: &str,
    providers: &HashMap<String, ProxyRouteStashEntry>,
) -> Result<(), String> {
    let path = proxy_route_stash_path(agent);
    if providers.is_empty() {
        if let Some(raw) = paths::read_file_safe(&path) {
            let valid = serde_json::from_str::<ProxyRouteStashFile>(&raw)
                .ok()
                .and_then(|f| f.providers)
                .is_some();
            if !valid {
                return Err(format!("refusing to replace malformed proxy stash {path}"));
            }
        }
        return match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        };
    }
    let file = ProxyRouteStashFile {
        providers: Some(providers.clone()),
    };
    let b = serde_json::to_string(&file).map_err(|e| e.to_string())?;
    paths::write_file_mode(&path, &b, 0o600).map_err(|e| e.to_string())
}

pub fn load_proxy_route_stash(agent: &str) -> HashMap<String, ProxyRouteStashEntry> {
    load_proxy_route_stash_locked(agent)
}

pub fn normalized_headroom_upstream(base_url: &str, api: &str) -> String {
    let trimmed = base_url.trim().trim_end_matches('/');
    let lower = trimmed.to_ascii_lowercase();
    if (api == "openai-completions" || api == "openai-responses" || api == "anthropic-messages")
        && lower.ends_with("/v1")
    {
        return trimmed[..trimmed.len() - 3].to_string();
    }
    trimmed.to_string()
}

pub fn proxy_endpoint_for_api(api: &str) -> String {
    if api == "anthropic-messages" {
        return headroom_proxy_url();
    }
    if api == "openai-completions" || api == "openai-responses" {
        return headroom_proxy_openai_url();
    }
    String::new()
}

pub fn register_byok_route(
    agent: &str,
    id: &str,
    api: &str,
    upstream: &str,
) -> (String, String, BYOKRoute, bool) {
    let protocol = byok_route_protocol(api);
    if protocol.is_empty() || upstream.trim().is_empty() {
        return (String::new(), String::new(), BYOKRoute::default(), false);
    }
    let namespaced_id = format!("{agent}:{id}");
    match upsert_byok_route(&namespaced_id, protocol, upstream) {
        Ok((route, previous)) => (
            byok_gateway_endpoint(),
            byok_route_header(&route),
            previous.unwrap_or_default(),
            true,
        ),
        Err(_) => (String::new(), String::new(), BYOKRoute::default(), false),
    }
}

pub fn rollback_byok_route(
    agent: &str,
    id: &str,
    previous: &BYOKRoute,
) -> Result<(), String> {
    let namespaced_id = format!("{agent}:{id}");
    if previous.id.is_empty() {
        return byok::delete_byok_route(&namespaced_id).map_err(|e| e.to_string());
    }
    upsert_byok_route(&namespaced_id, &previous.protocol, &previous.upstream)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub fn rollback_byok_route_logged(agent: &str, id: &str, previous: &BYOKRoute) -> bool {
    if let Err(err) = rollback_byok_route(agent, id, previous) {
        L.err(&format!("{agent} BYOK route rollback failed: {err}"));
        return false;
    }
    true
}

pub fn delete_byok_route(agent: &str, id: &str) -> Result<(), String> {
    let namespaced_id = format!("{agent}:{id}");
    byok::delete_byok_route(&namespaced_id).map_err(|e| e.to_string())
}

pub fn restore_deleted_byok_routes(routes: &[BYOKRoute]) {
    for route in routes {
        if let Err(err) = upsert_byok_route(&route.id, &route.protocol, &route.upstream) {
            L.err(&format!("BYOK route restore failed: {err}"));
        }
    }
}

// --- Shared internal helpers ---

pub(crate) fn tokless_command_ok(args: &[&str]) -> (String, bool) {
    let exe = paths::tokless_persisted_abs();
    if exe.is_empty() {
        return (String::new(), false);
    }
    (paths::persisted_tokless_command(&exe, args), true)
}

pub(crate) fn tokless_command(args: &[&str]) -> String {
    tokless_command_ok(args).0
}

pub(crate) fn tokless_managed_command(command: &str, args: &[&str]) -> bool {
    let fields = paths::split_command(command.trim());
    if fields.len() != args.len() + 1 {
        return false;
    }
    for (i, arg) in args.iter().enumerate() {
        if fields[i + 1] != *arg {
            return false;
        }
    }
    let exe = fields[0].trim_matches(|c| c == '"' || c == '\'');
    let normalized = exe.replace('\\', "/");
    let base = Path::new(&normalized)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    base == "tokless" || base == "tokless.exe" || base.starts_with("tokless-")
}

pub(crate) fn json_map_child<'a>(m: &'a OrderedMap, key: &str) -> Option<&'a OrderedMap> {
    m.get(key).and_then(|v| v.as_object())
}

pub(crate) fn json_map_child_mut<'a>(
    m: &'a mut OrderedMap,
    key: &str,
) -> Option<&'a mut OrderedMap> {
    m.get_mut(key).and_then(|v| v.as_object_mut())
}

pub(crate) fn json_get_or_create_map<'a>(
    m: &'a mut OrderedMap,
    key: &str,
) -> &'a mut OrderedMap {
    let is_obj = m.get(key).is_some_and(|v| v.is_object());
    if !is_obj {
        m.insert(key.to_string(), Value::Object(OrderedMap::new()));
    }
    m.get_mut(key).and_then(|v| v.as_object_mut()).unwrap()
}

pub(crate) fn json_args_eq(v: Option<&Value>, want: &[String]) -> bool {
    let Some(v) = v else {
        return want.is_empty();
    };
    let Some(arr) = v.as_array() else {
        return false;
    };
    if arr.len() != want.len() {
        return false;
    }
    for (item, w) in arr.iter().zip(want.iter()) {
        if item.as_str() != Some(w.as_str()) {
            return false;
        }
    }
    true
}

pub(crate) fn strings_to_values(ss: &[String]) -> Vec<Value> {
    ss.iter().map(|s| Value::String(s.clone())).collect()
}

pub(crate) fn json_equal(a: &Value, b: &Value) -> bool {
    canonical_json(a) == canonical_json(b)
}

pub(crate) fn canonical_json(v: &Value) -> String {
    fn sort_val(v: &Value) -> serde_json::Value {
        match v {
            Value::Null => serde_json::Value::Null,
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::Number(n) => serde_json::Value::Number(n.clone()),
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(sort_val).collect())
            }
            Value::Object(map) => {
                let mut btree = std::collections::BTreeMap::new();
                for (k, val) in map.iter() {
                    btree.insert(k.clone(), sort_val(val));
                }
                let mut out = serde_json::Map::new();
                for (k, val) in btree {
                    out.insert(k, val);
                }
                serde_json::Value::Object(out)
            }
        }
    }
    serde_json::to_string(&sort_val(v)).unwrap_or_default()
}

pub(crate) fn has_jsonc_comment(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if c == b'"' {
            in_string = true;
        } else if c == b'/' && i + 1 < bytes.len() && (bytes[i + 1] == b'/' || bytes[i + 1] == b'*')
        {
            return true;
        }
        i += 1;
    }
    false
}

pub(crate) fn any_string_slice_equal(v: Option<&Value>, want: &[String]) -> bool {
    let Some(arr) = v.and_then(|v| v.as_array()) else {
        return false;
    };
    if arr.len() != want.len() {
        return false;
    }
    for (x, w) in arr.iter().zip(want.iter()) {
        if x.as_str() != Some(w.as_str()) {
            return false;
        }
    }
    true
}

pub(crate) fn string_slices_equal(a: &[String], b: &[String]) -> bool {
    a == b
}

pub(crate) fn all_strings_nonempty(values: &[Value]) -> bool {
    values
        .iter()
        .all(|v| v.as_str().is_some_and(|s| !s.is_empty()))
}

#[cfg(windows)]
pub(crate) fn replace_kilo_file(tmp_path: &str, path: &str) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    const MOVEFILE_REPLACE_EXISTING: u32 = 0x00000001;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(
            lpExistingFileName: *const u16,
            lpNewFileName: *const u16,
            dwFlags: u32,
        ) -> i32;
    }

    let from: Vec<u16> = std::ffi::OsStr::new(tmp_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let to: Vec<u16> = std::ffi::OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let r = unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), MOVEFILE_REPLACE_EXISTING) };
    if r == 0 {
        Err(std::io::Error::last_os_error().to_string())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
pub(crate) fn replace_kilo_file(tmp_path: &str, path: &str) -> Result<(), String> {
    fs::rename(tmp_path, path).map_err(|e| e.to_string())
}

fn kilo_command_base(command: &str) -> String {
    let normalized = command.replace('\\', "/");
    let mut base = Path::new(&normalized)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    for ext in [".exe", ".cmd", ".bat"] {
        if let Some(stripped) = base.strip_suffix(ext) {
            base = stripped.to_string();
        }
    }
    base
}

fn kilo_cmd_shim_server(command: &[String], server: &str) -> bool {
    if command.len() != 3
        || kilo_command_base(&command[0]) != "cmd"
        || command[1] != "/c"
        || kilo_command_base(&command[2]) != server
    {
        return false;
    }
    let normalized = command[2].replace('\\', "/");
    let ext = Path::new(&normalized)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    ext == "cmd" || ext == "bat"
}

pub(crate) fn kilo_is_tokless_command(command: &str) -> bool {
    kilo_command_base(command) == "tokless"
}

pub(crate) fn kilo_projectmem_server(command: &[String]) -> bool {
    command.len() == 1 && kilo_command_base(&command[0]) == "pjm-mcp"
}

pub(crate) fn kilo_context_server(command: &[String]) -> bool {
    if command.len() == 1 {
        return kilo_command_base(&command[0]) == "context-mode";
    }
    if command.len() == 3
        && kilo_command_base(&command[0]) == "npx"
        && command[1] == "--no-install"
        && command[2] == "context-mode"
    {
        return true;
    }
    if command.len() == 5
        && kilo_cmd_shim_server(&command[..3], "npx")
        && command[3] == "--no-install"
        && command[4] == "context-mode"
    {
        return true;
    }
    command.len() == 3 && kilo_cmd_shim_server(command, "context-mode")
}

pub(crate) fn kilo_codegraph_server(command: &[String]) -> bool {
    if command.len() == 3 && command[1] == "serve" && command[2] == "--mcp" {
        return kilo_command_base(&command[0]) == "codegraph";
    }
    if command.len() == 5
        && kilo_command_base(&command[0]) == "npx"
        && command[1] == "--no-install"
        && command[2] == "@colbymchenry/codegraph"
        && command[3] == "serve"
        && command[4] == "--mcp"
    {
        return true;
    }
    if command.len() == 7
        && kilo_cmd_shim_server(&command[..3], "npx")
        && command[3] == "--no-install"
        && command[4] == "@colbymchenry/codegraph"
        && command[5] == "serve"
        && command[6] == "--mcp"
    {
        return true;
    }
    command.len() == 5
        && kilo_cmd_shim_server(&command[..3], "codegraph")
        && command[3] == "serve"
        && command[4] == "--mcp"
}

pub(crate) fn same_proxy_base(a: &str, b: &str) -> bool {
    let a = a.to_ascii_lowercase();
    let a = a.trim_end_matches('/');
    let b = b.to_ascii_lowercase();
    let b = b.trim_end_matches('/');
    !a.is_empty() && a == b
}

pub(crate) fn is_absolute_http(raw: &str) -> bool {
    let s = raw.trim();
    let rest = if let Some(r) = s.strip_prefix("http://") {
        r
    } else if let Some(r) = s.strip_prefix("https://") {
        r
    } else {
        return false;
    };
    if rest.is_empty() || rest.contains('?') || rest.contains('#') || rest.contains(char::is_whitespace) {
        return false;
    }
    let host_port = rest.split('/').next().unwrap_or("");
    if host_port.is_empty() || host_port.contains('@') {
        return false;
    }
    true
}

pub(crate) fn is_valid_byok_upstream(raw: &str) -> bool {
    is_absolute_http(raw) && valid_byok_upstream(raw)
}

pub(crate) fn resolve_api_key(reference: &str) -> String {
    if reference.is_empty() {
        return String::new();
    }
    if let Some(inner) = reference
        .strip_prefix("{env:")
        .and_then(|s| s.strip_suffix('}'))
    {
        return std::env::var(inner).unwrap_or_default().trim().to_string();
    }
    reference.to_string()
}

pub(crate) fn provider_base_and_key(m: &OrderedMap) -> (String, String) {
    let mut base = String::new();
    let mut key = String::new();
    if let Some(om) = json_map_child(m, "options") {
        if let Some(s) = om.get("baseURL").and_then(|v| v.as_str()) {
            base = s.to_string();
        }
        if base.is_empty()
            && let Some(s) = om.get("baseUrl").and_then(|v| v.as_str())
        {
            base = s.to_string();
        }
        if let Some(s) = om.get("apiKey").and_then(|v| v.as_str()) {
            key = s.to_string();
        }
    }
    if base.is_empty()
        && let Some(s) = m.get("baseURL").and_then(|v| v.as_str())
    {
        base = s.to_string();
    }
    if key.is_empty()
        && let Some(s) = m.get("apiKey").and_then(|v| v.as_str())
    {
        key = s.to_string();
    }
    let base = base.trim().to_string();
    let key = resolve_api_key(key.trim());
    (base, key)
}

pub(crate) fn open_code_protocol(npm: &str) -> String {
    match npm.trim() {
        "@ai-sdk/anthropic" => "anthropic-messages".to_string(),
        "@ai-sdk/openai-compatible" | "@ai-sdk/openai" => "openai-chat".to_string(),
        _ => String::new(),
    }
}

pub(crate) fn detect_agent_with(
    bin: &str,
    config_dir: &str,
    extra_dirs: &[String],
    app_paths: &[String],
) -> Detection {
    if !paths::find_binary(bin, extra_dirs).is_empty() {
        return Detection {
            installed: true,
            source: "cli".to_string(),
        };
    }
    for p in app_paths {
        if !p.is_empty() && paths::exists(p) {
            return Detection {
                installed: true,
                source: "app".to_string(),
            };
        }
    }
    if !config_dir.is_empty() && paths::exists(config_dir) {
        return Detection {
            installed: true,
            source: "config".to_string(),
        };
    }
    Detection::default()
}

pub(crate) fn is_vscode_extension_installed(extension_id: &str) -> bool {
    let prefix = format!("{}.", extension_id.to_ascii_lowercase());
    let h = paths::home();
    let dirs = [
        Path::new(&h).join(".vscode").join("extensions"),
        Path::new(&h).join(".vscode-insiders").join("extensions"),
        Path::new(&h).join(".vscode-oss").join("extensions"),
    ];
    for dir in dirs {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else {
                continue;
            };
            if !ft.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            if let Some(rest) = name.strip_prefix(&prefix)
                && rest.as_bytes().first().is_some_and(|c| c.is_ascii_digit())
            {
                return true;
            }
        }
    }
    false
}

pub(crate) fn detect_vscode_agent_with(
    bin: &str,
    config_dir: &str,
    extra_dirs: &[String],
    extension_id: &str,
) -> Detection {
    let d = detect_agent_with(bin, "", extra_dirs, &[]);
    if d.installed {
        return d;
    }
    if is_vscode_extension_installed(extension_id) {
        return Detection {
            installed: true,
            source: "vscode".to_string(),
        };
    }
    detect_agent_with("", config_dir, &[], &[])
}

pub(crate) fn try_parse_jsonc_map(text: &str) -> Option<OrderedMap> {
    match crate::util::jsonc::try_parse_jsonc(text)? {
        Value::Object(m) => Some(m),
        _ => None,
    }
}

pub(crate) fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = Vec::with_capacity(data.len() + 72);
    msg.extend_from_slice(data);
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
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

    format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]
    )
}

