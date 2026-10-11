use std::env;
use std::fs;
use std::path::Path;

use crate::agents::{ide_project_root, omp_agent_dir_resolved, pi_agent_dir_resolved};
use crate::util::jsonc::{stringify_json, try_parse_jsonc, OrderedMap, Value};
use crate::util::mcpspawn::McpSpawn;
use crate::util::paths::{
    antigravity_paths_resolved, codex_paths_resolved, copilot_paths_resolved,
    cursor_global_mcp_path, cursor_paths_resolved, ensure_dir, exists, home,
    persisted_tokless_command, read_file_safe, tokless_persisted_abs, write_file_atomic,
};
use crate::util::toml::{has_block, remove_block, upsert_block, TomlBlock};

pub use super::tokless_block::{
    configure_instruction_conflicts, has_owner, has_owner_at_path, instruction_path,
    kilo_has_owner, kilo_instructions_path, kilo_remove_owner, kilo_write_owner, remove_owner,
    remove_owner_at_path, remove_owner_at_path_safe, remove_owner_safe, strip_legacy,
    strip_legacy_heading, write_owner, write_owner_at_path,
};

pub fn write_if_missing(p: &str, content: &str, dry_run: bool) -> bool {
    if exists(p) {
        return false;
    }
    if dry_run {
        return true;
    }
    let _ = write_file_atomic(p, content);
    true
}

pub fn get_or_create_map<'a>(parent: &'a mut OrderedMap, key: &str) -> &'a mut OrderedMap {
    if !matches!(parent.get(key), Some(Value::Object(_))) {
        parent.insert(key.to_string(), Value::Object(OrderedMap::new()));
    }
    match parent.get_mut(key) {
        Some(Value::Object(m)) => m,
        _ => unreachable!(),
    }
}

pub fn clip(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.len() <= max {
        s.to_string()
    } else {
        s[s.len() - max..].to_string()
    }
}

pub fn is_test() -> bool {
    env::var("TOKLESS_TEST").as_deref() == Ok("1")
}

pub fn load_ordered(path: &str) -> OrderedMap {
    let raw = match read_file_safe(path) {
        Some(r) => r,
        None => return OrderedMap::new(),
    };
    match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => OrderedMap::new(),
    }
}

pub fn get_arr(m: &OrderedMap, key: &str) -> Vec<Value> {
    match m.get(key) {
        Some(Value::Array(arr)) => arr.clone(),
        _ => Vec::new(),
    }
}

pub fn mcp_str_arr_eq(v: Option<&Value>, want: &[String]) -> bool {
    let arr = match v {
        Some(Value::Array(a)) => a,
        _ => return false,
    };
    if arr.len() != want.len() {
        return false;
    }
    for (i, item) in arr.iter().enumerate() {
        match item {
            Value::String(s) if s == &want[i] => {}
            _ => return false,
        }
    }
    true
}

pub fn tokless_cmd(args: &[&str]) -> String {
    let exe = tokless_persisted_abs();
    if exe.is_empty() {
        return String::new();
    }
    persisted_tokless_command(&exe, args)
}

// --- Generic JSON MCP helpers (`mcpServers.<name> = { command, args, ... }`) ---

pub fn upsert_json_mcp_entry(
    path: &str,
    servers_key: &str,
    name: &str,
    spawn: &McpSpawn,
    extra_fields: &[(&str, Value)],
) -> bool {
    if path.is_empty() || spawn.command.is_empty() {
        return false;
    }
    if let Some(parent) = Path::new(path).parent() {
        let _ = ensure_dir(parent);
    }
    let mut cfg = load_ordered(path);
    let servers = get_or_create_map(&mut cfg, servers_key);
    let mut entry = match servers.get(name) {
        Some(Value::Object(m)) => m.clone(),
        _ => OrderedMap::new(),
    };
    entry.insert("command".into(), Value::String(spawn.command.clone()));
    let args_val: Vec<Value> = spawn.args.iter().map(|s| Value::String(s.clone())).collect();
    entry.insert("args".into(), Value::Array(args_val));
    for (k, v) in extra_fields {
        entry.insert((*k).to_string(), v.clone());
    }
    servers.insert(name.to_string(), Value::Object(entry));
    write_file_atomic(path, &stringify_json(&Value::Object(cfg))).is_ok()
}

pub fn remove_json_mcp_entry_safe(path: &str, servers_key: &str, name: &str) -> Result<(), String> {
    if path.is_empty() || !exists(path) {
        return Ok(());
    }
    let raw = match read_file_safe(path) {
        Some(r) => r,
        None => return Ok(()),
    };
    let mut cfg = match try_parse_jsonc(&raw) {
        Some(Value::Object(m)) => m,
        _ => return Ok(()),
    };
    let changed = if let Some(Value::Object(servers)) = cfg.get_mut(servers_key) {
        let had = servers.remove(name).is_some();
        if servers.is_empty() {
            cfg.remove(servers_key);
        }
        had
    } else {
        false
    };
    if !changed {
        return Ok(());
    }
    write_file_atomic(path, &stringify_json(&Value::Object(cfg))).map_err(|e| e.to_string())
}

pub fn remove_json_mcp_entry(path: &str, servers_key: &str, name: &str) -> bool {
    remove_json_mcp_entry_safe(path, servers_key, name).is_ok()
}

pub fn has_mcp_entry_at_file(path: &str, servers_key: &str, name: &str, expected: Option<&McpSpawn>) -> bool {
    if path.is_empty() {
        return false;
    }
    let cfg = load_ordered(path);
    let servers = match cfg.get(servers_key) {
        Some(Value::Object(m)) => m,
        _ => return false,
    };
    let entry = match servers.get(name) {
        Some(Value::Object(m)) => m,
        _ => return false,
    };
    let cmd = match entry.get("command") {
        Some(Value::String(s)) if !s.is_empty() => s,
        _ => return false,
    };
    if let Some(exp) = expected {
        if cmd != &exp.command {
            return false;
        }
        if !mcp_str_arr_eq(entry.get("args"), &exp.args) {
            return false;
        }
    }
    true
}

// --- OpenCode / Kilo array-command MCP helpers (`mcp.<name> = { type: "local", command: [...], enabled: true }`) ---

pub fn upsert_array_mcp_entry(path: &str, name: &str, spawn: &McpSpawn) -> bool {
    if path.is_empty() || spawn.command.is_empty() {
        return false;
    }
    if let Some(parent) = Path::new(path).parent() {
        let _ = ensure_dir(parent);
    }
    let mut cfg = load_ordered(path);
    let mcp = get_or_create_map(&mut cfg, "mcp");
    let mut cmd_arr = Vec::with_capacity(1 + spawn.args.len());
    cmd_arr.push(Value::String(spawn.command.clone()));
    for a in &spawn.args {
        cmd_arr.push(Value::String(a.clone()));
    }
    let mut entry = OrderedMap::new();
    entry.insert("type".into(), Value::String("local".into()));
    entry.insert("command".into(), Value::Array(cmd_arr));
    entry.insert("enabled".into(), Value::Bool(true));
    mcp.insert(name.to_string(), Value::Object(entry));
    write_file_atomic(path, &stringify_json(&Value::Object(cfg))).is_ok()
}

pub fn remove_array_mcp_entry(path: &str, name: &str) -> bool {
    remove_json_mcp_entry(path, "mcp", name)
}

pub fn has_array_mcp_entry(path: &str, name: &str, expected: Option<&McpSpawn>) -> bool {
    if path.is_empty() {
        return false;
    }
    let cfg = load_ordered(path);
    let mcp = match cfg.get("mcp") {
        Some(Value::Object(m)) => m,
        _ => return false,
    };
    let entry = match mcp.get(name) {
        Some(Value::Object(m)) => m,
        _ => return false,
    };
    if let Some(Value::Bool(false)) = entry.get("enabled") {
        return false;
    }
    if let Some(exp) = expected {
        let mut want = Vec::with_capacity(1 + exp.args.len());
        want.push(exp.command.clone());
        want.extend(exp.args.iter().cloned());
        if !mcp_str_arr_eq(entry.get("command"), &want) {
            return false;
        }
    } else {
        match entry.get("command") {
            Some(Value::Array(a)) if !a.is_empty() => {}
            _ => return false,
        }
    }
    true
}

// --- Codex TOML MCP helpers ---

pub fn upsert_codex_mcp_entry(name: &str, spawn: &McpSpawn) -> bool {
    if spawn.command.is_empty() {
        return false;
    }
    let p = codex_paths_resolved();
    let _ = ensure_dir(&p.dir);
    let existing = read_file_safe(&p.config).unwrap_or_default();
    let header = format!("mcp_servers.{}", name);
    let mut block = TomlBlock::new(header);
    block.set("command", spawn.command.clone());
    block.set("args", spawn.args.clone());
    let updated = upsert_block(&existing, &block, false);
    write_file_atomic(&p.config, &updated).is_ok()
}

pub fn remove_codex_mcp_entry(name: &str) -> bool {
    let p = codex_paths_resolved();
    let existing = match read_file_safe(&p.config) {
        Some(r) => r,
        None => return true,
    };
    let header = format!("mcp_servers.{}", name);
    if !has_block(&existing, &header) {
        return true;
    }
    let updated = remove_block(&existing, &header);
    write_file_atomic(&p.config, &updated).is_ok()
}

pub fn has_codex_mcp_entry(name: &str, _expected: Option<&McpSpawn>) -> bool {
    let p = codex_paths_resolved();
    let existing = match read_file_safe(&p.config) {
        Some(r) => r,
        None => return false,
    };
    let header = format!("mcp_servers.{}", name);
    has_block(&existing, &header)
}

// --- Agent-specific MCP paths ---

pub fn droid_mcp_path() -> String {
    Path::new(&home()).join(".factory").join("mcp.json").to_string_lossy().to_string()
}

pub fn pi_mcp_path() -> String {
    Path::new(&pi_agent_dir_resolved()).join("mcp.json").to_string_lossy().to_string()
}

pub fn omp_mcp_path() -> String {
    Path::new(&omp_agent_dir_resolved()).join("mcp.json").to_string_lossy().to_string()
}

pub fn grok_mcp_path() -> String {
    let dir = env::var("GROK_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| Path::new(&home()).join(".grok").to_string_lossy().to_string());
    Path::new(&dir).join("settings.json").to_string_lossy().to_string()
}

pub fn copilot_ide_mcp_path() -> String {
    Path::new(&ide_project_root())
        .join(".vscode")
        .join("mcp.json")
        .to_string_lossy()
        .to_string()
}

pub fn copilot_ide_instructions_path() -> String {
    Path::new(&ide_project_root())
        .join(".github")
        .join("copilot-instructions.md")
        .to_string_lossy()
        .to_string()
}

// --- Claude / Antigravity MCP permissions ---

pub fn allow_claude_mcp_server(name: &str) {
    crate::agents::allow_claude_bash_pattern(&format!("mcp__{}__*", name));
}

pub fn disallow_claude_mcp_server(name: &str) {
    crate::agents::disallow_claude_bash_pattern(&format!("mcp__{}__*", name));
}

pub fn allow_antigravity_mcp_server(name: &str) {
    crate::agents::allow_antigravity_entry(&format!("mcp({}:*)", name));
}

pub fn remove_antigravity_mcp_server(name: &str) {
    crate::agents::remove_antigravity_entry(&format!("mcp({}:*)", name));
}

// --- Antigravity dual MCP config (IDE + CLI) ---

pub fn upsert_antigravity_mcp(name: &str, spawn: &McpSpawn) -> bool {
    let p = antigravity_paths_resolved();
    let ok1 = upsert_json_mcp_entry(&p.mcp_config_ide, "mcpServers", name, spawn, &[]);
    let ok2 = upsert_json_mcp_entry(&p.mcp_config_cli, "mcpServers", name, spawn, &[]);
    allow_antigravity_mcp_server(name);
    ok1 && ok2
}

pub fn remove_antigravity_mcp(name: &str) -> bool {
    let p = antigravity_paths_resolved();
    let ok1 = remove_json_mcp_entry(&p.mcp_config_ide, "mcpServers", name);
    let ok2 = remove_json_mcp_entry(&p.mcp_config_cli, "mcpServers", name);
    remove_antigravity_mcp_server(name);
    ok1 && ok2
}

pub fn has_antigravity_mcp(name: &str, expected: Option<&McpSpawn>) -> bool {
    let p = antigravity_paths_resolved();
    has_mcp_entry_at_file(&p.mcp_config_cli, "mcpServers", name, expected)
}

// --- Copilot CLI + IDE MCP & instructions ---

pub fn upsert_copilot_mcp_safe(name: &str, cli_spawn: &McpSpawn, ide_spawn: &McpSpawn) -> Result<(), String> {
    let p = copilot_paths_resolved();
    if !upsert_json_mcp_entry(&p.mcp_config, "mcpServers", name, cli_spawn, &[("tools", Value::Array(vec![Value::String("*".into())]))]) {
        return Err("failed to write copilot CLI mcp config".into());
    }
    let ide_path = copilot_ide_mcp_path();
    let extra = [("type", Value::String("stdio".into()))];
    if !upsert_json_mcp_entry(&ide_path, "servers", name, ide_spawn, &extra) {
        return Err("failed to write copilot IDE mcp config".into());
    }
    Ok(())
}

pub fn remove_copilot_mcp_safe(name: &str) -> Result<(), String> {
    let p = copilot_paths_resolved();
    remove_json_mcp_entry_safe(&p.mcp_config, "mcpServers", name)?;
    let ide_path = copilot_ide_mcp_path();
    remove_json_mcp_entry_safe(&ide_path, "servers", name)?;
    Ok(())
}

pub fn has_copilot_mcp(name: &str, cli_expected: Option<&McpSpawn>, ide_expected: Option<&McpSpawn>) -> bool {
    let p = copilot_paths_resolved();
    let ide_path = copilot_ide_mcp_path();
    has_mcp_entry_at_file(&p.mcp_config, "mcpServers", name, cli_expected)
        && has_mcp_entry_at_file(&ide_path, "servers", name, ide_expected)
}

pub fn sync_copilot_ide_instructions_safe() -> Result<(), String> {
    let p = copilot_paths_resolved();
    let ide_inst = copilot_ide_instructions_path();
    if let Some(content) = read_file_safe(&p.instructions) {
        if content.trim().is_empty() {
            if exists(&ide_inst) {
                let _ = fs::remove_file(&ide_inst);
            }
            return Ok(());
        }
        if let Some(parent) = Path::new(&ide_inst).parent() {
            let _ = ensure_dir(parent);
        }
        write_file_atomic(&ide_inst, &content).map_err(|e| e.to_string())
    } else {
        if exists(&ide_inst) {
            let _ = fs::remove_file(&ide_inst);
        }
        Ok(())
    }
}

// --- Pi MCP adapter helpers ---

pub fn pi_mcp_adapter_ready() -> bool {
    let p = pi_mcp_path();
    exists(&p) || exists(&pi_agent_dir_resolved())
}

pub fn pi_ensure_mcp_adapter(_dry_run: bool) -> bool {
    let _ = ensure_dir(&pi_agent_dir_resolved());
    true
}

pub fn pi_remove_mcp_adapter() {
    let p = pi_mcp_path();
    if let Some(raw) = read_file_safe(&p) {
        if let Some(Value::Object(cfg)) = try_parse_jsonc(&raw) {
            let empty = match cfg.get("mcpServers") {
                Some(Value::Object(s)) => s.is_empty(),
                None => true,
                _ => false,
            };
            if empty && cfg.len() <= 1 {
                let _ = fs::remove_file(&p);
            }
        }
    }
}

// --- Codex Perm Hook & CodeGraph Index Hook ---

pub fn install_codex_perm_hook() -> bool {
    let cmd = tokless_cmd(&["codex-perm"]);
    if cmd.is_empty() {
        return false;
    }
    let p = Path::new(&codex_paths_resolved().dir).join("hooks.json").to_string_lossy().to_string();
    upsert_hook_group_in_hooks_obj(&p, "PermissionRequest", "Bash", &cmd, Some(10))
}

pub fn remove_codex_perm_hook() {
    let p = Path::new(&codex_paths_resolved().dir).join("hooks.json").to_string_lossy().to_string();
    remove_hook_group_in_hooks_obj(&p, "PermissionRequest", "codex-perm");
}

pub fn install_codex_codegraph_index_hook() -> bool {
    let cmd = tokless_cmd(&["index", "--codegraph-hook"]);
    if cmd.is_empty() {
        return false;
    }
    let p = Path::new(&codex_paths_resolved().dir).join("hooks.json").to_string_lossy().to_string();
    upsert_hook_group_in_hooks_obj(&p, "SessionStart", "", &cmd, Some(30))
}

pub fn has_codex_codegraph_index_hook() -> bool {
    let p = Path::new(&codex_paths_resolved().dir).join("hooks.json").to_string_lossy().to_string();
    read_file_safe(&p)
        .map(|r| r.contains("--codegraph-hook"))
        .unwrap_or(false)
}

pub fn remove_codex_codegraph_index_hook() {
    let p = Path::new(&codex_paths_resolved().dir).join("hooks.json").to_string_lossy().to_string();
    remove_hook_group_in_hooks_obj(&p, "SessionStart", "--codegraph-hook");
}

// --- Cursor CodeGraph permissions & hooks ---

pub fn configure_cursor_codegraph_permissions() -> bool {
    let p = cursor_paths_resolved().cli_config;
    if p.is_empty() {
        return true;
    }
    let _ = ensure_dir(&cursor_paths_resolved().dir);
    let mut cfg = load_ordered(&p);
    let perms = get_or_create_map(&mut cfg, "permissions");
    let mut allow = get_arr(perms, "allow");
    let entry = "Mcp(codegraph:*)";
    if !allow.iter().any(|v| v.as_str() == Some(entry)) {
        allow.push(Value::String(entry.into()));
    }
    perms.insert("allow".into(), Value::Array(allow));
    write_file_atomic(&p, &stringify_json(&Value::Object(cfg))).is_ok()
}

pub fn has_cursor_codegraph_permissions() -> bool {
    let p = cursor_paths_resolved().cli_config;
    if p.is_empty() || !exists(&p) {
        return true;
    }
    read_file_safe(&p)
        .map(|r| r.contains("Mcp(codegraph:*)") || !r.contains("\"permissions\""))
        .unwrap_or(true)
}

pub fn remove_cursor_codegraph_permissions() -> bool {
    let p = cursor_paths_resolved().cli_config;
    if p.is_empty() || !exists(&p) {
        return true;
    }
    let mut cfg = load_ordered(&p);
    if let Some(Value::Object(perms)) = cfg.get_mut("permissions") {
        if let Some(Value::Array(allow)) = perms.get_mut("allow") {
            allow.retain(|v| v.as_str() != Some("Mcp(codegraph:*)"));
        }
    }
    write_file_atomic(&p, &stringify_json(&Value::Object(cfg))).is_ok()
}

pub fn install_cursor_codegraph_index_hook() -> bool {
    let cmd = tokless_cmd(&["index", "--codegraph-hook"]);
    if cmd.is_empty() {
        return false;
    }
    let p = Path::new(&cursor_paths_resolved().dir).join("hooks.json").to_string_lossy().to_string();
    let _ = ensure_dir(&cursor_paths_resolved().dir);
    let mut cfg = load_ordered(&p);
    let hooks = get_or_create_map(&mut cfg, "hooks");
    let mut arr = get_arr(hooks, "sessionStart");
    arr.retain(|v| {
        v.as_object()
            .and_then(|o| o.get("command"))
            .and_then(|c| c.as_str())
            .map(|s| !s.contains("--codegraph-hook"))
            .unwrap_or(true)
    });
    let mut entry = OrderedMap::new();
    entry.insert("command".into(), Value::String(cmd));
    arr.push(Value::Object(entry));
    hooks.insert("sessionStart".into(), Value::Array(arr));
    write_file_atomic(&p, &stringify_json(&Value::Object(cfg))).is_ok()
}

pub fn has_cursor_codegraph_index_hook() -> bool {
    let p = Path::new(&cursor_paths_resolved().dir).join("hooks.json").to_string_lossy().to_string();
    read_file_safe(&p)
        .map(|r| r.contains("--codegraph-hook"))
        .unwrap_or(false)
}

pub fn remove_cursor_codegraph_index_hook() -> bool {
    let p = Path::new(&cursor_paths_resolved().dir).join("hooks.json").to_string_lossy().to_string();
    if !exists(&p) {
        return true;
    }
    let mut cfg = load_ordered(&p);
    if let Some(Value::Object(hooks)) = cfg.get_mut("hooks") {
        if let Some(Value::Array(arr)) = hooks.get_mut("sessionStart") {
            arr.retain(|v| {
                v.as_object()
                    .and_then(|o| o.get("command"))
                    .and_then(|c| c.as_str())
                    .map(|s| !s.contains("--codegraph-hook"))
                    .unwrap_or(true)
            });
            if arr.is_empty() {
                hooks.remove("sessionStart");
            }
        }
        if hooks.is_empty() {
            cfg.remove("hooks");
        }
    }
    write_file_atomic(&p, &stringify_json(&Value::Object(cfg))).is_ok()
}

// --- Copilot CodeGraph Index Hooks ---

pub fn install_copilot_codegraph_index_hook_safe() -> Result<(), String> {
    let cmd = tokless_cmd(&["index", "--codegraph-hook"]);
    if cmd.is_empty() {
        return Err("no tokless command".into());
    }
    let p = Path::new(&copilot_paths_resolved().hooks_dir)
        .join("tokless-codegraph-index.json")
        .to_string_lossy()
        .to_string();
    if let Some(parent) = Path::new(&p).parent() {
        let _ = ensure_dir(parent);
    }
    let mut obj = OrderedMap::new();
    obj.insert("command".into(), Value::String(cmd));
    write_file_atomic(&p, &stringify_json(&Value::Object(obj))).map_err(|e| e.to_string())
}

pub fn install_copilot_ide_codegraph_index_hook_safe() -> Result<(), String> {
    let cmd = tokless_cmd(&["index", "--codegraph-hook"]);
    if cmd.is_empty() {
        return Err("no tokless command".into());
    }
    let p = Path::new(&ide_project_root())
        .join(".github")
        .join("hooks")
        .join("tokless-codegraph-index.json")
        .to_string_lossy()
        .to_string();
    if let Some(parent) = Path::new(&p).parent() {
        let _ = ensure_dir(parent);
    }
    let mut obj = OrderedMap::new();
    obj.insert("command".into(), Value::String(cmd));
    write_file_atomic(&p, &stringify_json(&Value::Object(obj))).map_err(|e| e.to_string())
}

pub fn has_copilot_codegraph_index_hook() -> bool {
    let p = Path::new(&copilot_paths_resolved().hooks_dir)
        .join("tokless-codegraph-index.json")
        .to_string_lossy()
        .to_string();
    exists(&p)
}

pub fn has_copilot_ide_codegraph_index_hook() -> bool {
    let p = Path::new(&ide_project_root())
        .join(".github")
        .join("hooks")
        .join("tokless-codegraph-index.json")
        .to_string_lossy()
        .to_string();
    exists(&p)
}

pub fn remove_copilot_codegraph_index_hook_safe() -> Result<(), String> {
    let p = Path::new(&copilot_paths_resolved().hooks_dir)
        .join("tokless-codegraph-index.json")
        .to_string_lossy()
        .to_string();
    if exists(&p) {
        let _ = fs::remove_file(&p);
    }
    Ok(())
}

pub fn remove_copilot_ide_codegraph_index_hook_safe() -> Result<(), String> {
    let p = Path::new(&ide_project_root())
        .join(".github")
        .join("hooks")
        .join("tokless-codegraph-index.json")
        .to_string_lossy()
        .to_string();
    if exists(&p) {
        let _ = fs::remove_file(&p);
    }
    Ok(())
}

// --- Shared Hook Object Upsert/Remove ---

pub fn upsert_hook_group_in_hooks_obj(
    path: &str,
    event: &str,
    matcher: &str,
    cmd: &str,
    timeout: Option<i64>,
) -> bool {
    if path.is_empty() || cmd.is_empty() {
        return false;
    }
    if let Some(parent) = Path::new(path).parent() {
        let _ = ensure_dir(parent);
    }
    let mut cfg = load_ordered(path);
    let hooks_obj = get_or_create_map(&mut cfg, "hooks");
    let mut groups = get_arr(hooks_obj, event);
    groups.retain(|g| {
        if let Value::Object(gm) = g {
            if let Some(Value::Array(ha)) = gm.get("hooks") {
                for h in ha {
                    if let Value::Object(hm) = h {
                        if let Some(Value::String(c)) = hm.get("command") {
                            if c == cmd || (cmd.contains("projectmem-hook") && c.contains("projectmem-hook"))
                                || (cmd.contains("--codegraph-hook") && c.contains("--codegraph-hook"))
                                || (cmd.contains("codex-perm") && c.contains("codex-perm"))
                            {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        true
    });

    let mut hm = OrderedMap::new();
    hm.insert("type".into(), Value::String("command".into()));
    hm.insert("command".into(), Value::String(cmd.to_string()));
    if let Some(t) = timeout {
        hm.insert("timeout".into(), Value::from(t));
    }

    let mut gm = OrderedMap::new();
    if !matcher.is_empty() {
        gm.insert("matcher".into(), Value::String(matcher.to_string()));
    }
    gm.insert("hooks".into(), Value::Array(vec![Value::Object(hm)]));
    groups.push(Value::Object(gm));
    hooks_obj.insert(event.to_string(), Value::Array(groups));
    write_file_atomic(path, &stringify_json(&Value::Object(cfg))).is_ok()
}

pub fn remove_hook_group_in_hooks_obj(path: &str, event: &str, needle: &str) {
    if path.is_empty() || !exists(path) {
        return;
    }
    let mut cfg = load_ordered(path);
    let changed = if let Some(Value::Object(hooks_obj)) = cfg.get_mut("hooks") {
        if let Some(Value::Array(groups)) = hooks_obj.get_mut(event) {
            let before = groups.len();
            groups.retain(|g| {
                if let Value::Object(gm) = g {
                    if let Some(Value::Array(ha)) = gm.get("hooks") {
                        for h in ha {
                            if let Value::Object(hm) = h {
                                if let Some(Value::String(c)) = hm.get("command") {
                                    if c.contains(needle) {
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                }
                true
            });
            let after = groups.len();
            if groups.is_empty() {
                hooks_obj.remove(event);
            }
            before != after
        } else {
            false
        }
    } else {
        false
    };
    if changed {
        let _ = write_file_atomic(path, &stringify_json(&Value::Object(cfg)));
    }
}

pub fn cursor_mcp_path() -> String {
    cursor_global_mcp_path()
}
