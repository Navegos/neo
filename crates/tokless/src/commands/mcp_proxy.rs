use std::collections::HashSet;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, RwLock};
use regex::Regex;
use serde_json::Value;

use crate::agents::hooks::PROJECTMEM_MCP_TOOL_NAMES;
use crate::tools::projectmem::PROJECTMEM_MCP_INSTRUCTIONS;
use crate::util::paths::{which, IS_WIN};

pub const CONTEXT_MODE_TOOLS: &[&str] = &[
    "ctx_batch_execute",
    "ctx_execute",
    "ctx_execute_file",
    "ctx_index",
    "ctx_search",
    "ctx_fetch_and_index",
];

const MAX_MCP_MESSAGE_SIZE: usize = 64 << 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpFraming {
    NdJson,
    ContentLength,
}

pub fn mcp_child_env(env: &[(String, String)]) -> Vec<(String, String)> {
    env.to_vec()
}

pub fn is_node_shebang_script(path: &str) -> bool {
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut buf = [0u8; 32];
    let n = match file.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return false,
    };
    let prefix = String::from_utf8_lossy(&buf[..n]);
    prefix.starts_with("#!/usr/bin/env node")
        || prefix.starts_with("#!/usr/bin/env -S node")
        || prefix.starts_with("#!/usr/bin/node")
}

fn path_base_lowercase(s: &str) -> String {
    let replaced = s.replace('\\', "/");
    let p = Path::new(&replaced);
    p.file_name().unwrap_or_default().to_string_lossy().to_lowercase()
}

pub fn normalized_cmd_batch_args(command: &str, args: &[String], windows: bool) -> Vec<String> {
    let mut out = args.to_vec();
    let base = path_base_lowercase(command);
    if !windows || (base != "cmd" && base != "cmd.exe") || out.len() < 2 || !out[0].eq_ignore_ascii_case("/c") {
        return out;
    }
    let norm_path = out[1].replace('\\', "/");
    let ext_p = Path::new(&norm_path);
    let ext = ext_p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext == "cmd" || ext == "bat" {
        out[1] = out[1].replace('/', "\\");
    }
    out
}

pub fn resolve_mcp_command(path: &str, argv: &[String]) -> (String, Vec<String>) {
    if is_node_shebang_script(path) {
        let node_path = which("node");
        if !node_path.is_empty() {
            let mut out = vec![path.to_string()];
            if argv.len() > 1 {
                out.extend_from_slice(&argv[1..]);
            }
            return (node_path, out);
        }
    }
    let rest = if argv.len() > 1 { &argv[1..] } else { &[] };
    (path.to_string(), normalized_cmd_batch_args(path, rest, IS_WIN))
}

pub fn run_mcp_proxy(
    _agent: &str,
    path: &str,
    argv: &[String],
    env: &[(String, String)],
    tool: &str,
) -> i32 {
    let (exe, args) = resolve_mcp_command(path, argv);
    let mut cmd = Command::new(&exe);
    if !args.is_empty() {
        cmd.args(&args);
    }
    for (k, v) in mcp_child_env(env) {
        cmd.env(k, v);
    }
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::inherit());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return 1,
    };

    let allowed_tools: Option<&[&str]> = match tool {
        "context-mode" => Some(CONTEXT_MODE_TOOLS),
        "projectmem" => Some(PROJECTMEM_MCP_TOOL_NAMES),
        _ => None,
    };

    if let Some(allowed) = allowed_tools {
        run_bounded_mcp_proxy(&mut child, allowed, tool == "projectmem")
    } else {
        run_unbounded_proxy(&mut child)
    }
}

fn run_unbounded_proxy(child: &mut Child) -> i32 {
    let mut child_in = child.stdin.take().unwrap();
    let mut child_out = child.stdout.take().unwrap();

    let t1 = std::thread::spawn(move || {
        let mut stdin = io::stdin();
        let _ = io::copy(&mut stdin, &mut child_in);
    });

    let t2 = std::thread::spawn(move || {
        let mut stdout = io::stdout();
        let _ = io::copy(&mut child_out, &mut stdout);
    });

    let _ = t2.join();
    let status = child.wait();
    let _ = t1.join();
    status.map(|s| s.code().unwrap_or(0)).unwrap_or(1)
}

struct McpRequestIDs {
    init_ids: HashSet<String>,
}

struct McpLockedWriter<W> {
    writer: Arc<Mutex<W>>,
}

impl<W: Write> McpLockedWriter<W> {
    fn new(w: W) -> Self {
        Self {
            writer: Arc::new(Mutex::new(w)),
        }
    }

    fn write_mcp_message(&self, framing: McpFraming, message: &[u8]) -> io::Result<()> {
        let mut guard = self.writer.lock().unwrap();
        write_mcp_message_unlocked(&mut *guard, framing, message)
    }
}

pub fn read_mcp_message<R: BufRead>(reader: &mut R) -> io::Result<Option<(Vec<u8>, McpFraming)>> {
    let line = match read_mcp_line(reader)? {
        Some(l) => l,
        None => return Ok(None),
    };
    let line_str = String::from_utf8_lossy(&line);
    let parts: Vec<&str> = line_str.splitn(2, ':').collect();
    if !parts[0].eq_ignore_ascii_case("Content-Length") {
        return Ok(Some((line, McpFraming::NdJson)));
    }
    if parts.len() != 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid Content-Length header"));
    }
    let length: usize = parts[1].trim().parse().map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidData, "invalid Content-Length")
    })?;
    if length > MAX_MCP_MESSAGE_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid Content-Length"));
    }
    loop {
        let header = match read_mcp_line(reader)? {
            Some(h) => h,
            None => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "unexpected EOF in header")),
        };
        if header.is_empty() {
            break;
        }
    }
    let mut message = vec![0u8; length];
    reader.read_exact(&mut message)?;
    Ok(Some((message, McpFraming::ContentLength)))
}

fn read_mcp_line<R: BufRead>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    let bytes_read = reader.read_until(b'\n', &mut line)?;
    if bytes_read == 0 {
        return Ok(None);
    }
    if line.ends_with(b"\n") {
        line.pop();
        if line.ends_with(b"\r") {
            line.pop();
        }
    }
    if line.len() > MAX_MCP_MESSAGE_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "MCP message exceeds limit"));
    }
    Ok(Some(line))
}

fn write_mcp_message_unlocked<W: Write>(writer: &mut W, framing: McpFraming, message: &[u8]) -> io::Result<()> {
    if message.len() > MAX_MCP_MESSAGE_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "MCP message exceeds limit"));
    }
    if framing == McpFraming::ContentLength {
        write!(writer, "Content-Length: {}\r\n\r\n", message.len())?;
        writer.write_all(message)?;
    } else {
        writer.write_all(message)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()
}

pub fn canonical_mcp_id(id: &Value) -> Option<String> {
    match id {
        Value::String(s) => Some(format!("string:{}", s)),
        Value::Number(n) => canonical_mcp_number(&n.to_string()),
        _ => None,
    }
}

pub fn canonical_mcp_number(mut number: &str) -> Option<String> {
    let mut sign = "";
    if number.starts_with('-') {
        sign = "-";
        number = &number[1..];
    }
    let mut exponent = 0i64;
    let mut num_str = number.to_string();
    if let Some(idx) = num_str.find(|c| c == 'e' || c == 'E') {
        let exp: i64 = num_str[idx + 1..].parse().ok()?;
        exponent = exp;
        num_str.truncate(idx);
    }
    if let Some(idx) = num_str.find('.') {
        exponent -= (num_str.len() - idx - 1) as i64;
        num_str = format!("{}{}", &num_str[..idx], &num_str[idx + 1..]);
    }
    let trimmed = num_str.trim_start_matches('0');
    if trimmed.is_empty() {
        return Some("number:0".to_string());
    }
    let mut s = trimmed.to_string();
    while s.ends_with('0') {
        s.pop();
        exponent += 1;
    }
    Some(format!("number:{}{}e{}", sign, s, exponent))
}

pub fn mcp_tool_denied_response(id: Option<&Value>, name: &str) -> Vec<u8> {
    let id_val = id.cloned().unwrap_or(Value::Null);
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id_val,
        "error": {
            "code": -32601,
            "message": format!("tool \"{}\" is not available", name)
        }
    });
    serde_json::to_vec(&resp).unwrap()
}

pub fn mcp_invalid_request_response() -> Vec<u8> {
    b"{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{\"code\":-32600,\"message\":\"JSON-RPC batches are not supported\"}}".to_vec()
}

pub fn mcp_parse_error_response(id: Option<&Value>) -> Vec<u8> {
    let id_val = id.cloned().unwrap_or(Value::Null);
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id_val,
        "error": {
            "code": -32700,
            "message": "Parse error"
        }
    });
    serde_json::to_vec(&resp).unwrap()
}

struct StaticUniverse {
    _tool: &'static str,
    names: &'static [&'static str],
}

const STATIC_TOOL_UNIVERSES: &[StaticUniverse] = &[
    StaticUniverse {
        _tool: "projectmem",
        names: &[
            "add_decision", "add_note", "current_project", "get_context",
            "get_global_gotchas", "get_instructions", "get_issue", "get_plan",
            "get_project_map", "get_score", "get_summary", "list_projects",
            "log_issue", "precheck_file", "record_attempt", "record_fix",
            "search_events",
        ],
    },
    StaticUniverse {
        _tool: "context-mode",
        names: &[
            "ctx_execute", "ctx_execute_file", "ctx_batch_execute", "ctx_search",
            "ctx_index", "ctx_fetch_and_index", "ctx_stats", "ctx_doctor",
            "ctx_upgrade", "ctx_insight", "ctx_purge",
        ],
    },
];

fn static_tool_universe(allowed: &[&str]) -> Option<&'static [&'static str]> {
    if allowed.is_empty() {
        return None;
    }
    for universe in STATIC_TOOL_UNIVERSES {
        let set: HashSet<&str> = universe.names.iter().copied().collect();
        if allowed.iter().all(|a| set.contains(a)) {
            return Some(universe.names);
        }
    }
    None
}

pub fn build_hidden_tool_names(
    by_name: &HashSet<String>,
    allowed: &HashSet<String>,
    universe: Option<&[&str]>,
) -> Vec<String> {
    let mut set = HashSet::new();
    for name in by_name {
        if !allowed.contains(name) {
            set.insert(name.clone());
        }
    }
    if let Some(u) = universe {
        for &name in u {
            if !allowed.contains(name) {
                set.insert(name.to_string());
            }
        }
    }
    let mut hidden: Vec<String> = set.into_iter().collect();
    hidden.sort_by(|a, b| b.len().cmp(&a.len()));
    hidden
}

fn hidden_tool_regexp(hidden: &[String]) -> Option<Regex> {
    if hidden.is_empty() {
        return None;
    }
    let parts: Vec<String> = hidden.iter().map(|n| format!(r"\b{}\b", regex::escape(n))).collect();
    Regex::new(&parts.join("|")).ok()
}

pub fn scrub_json_strings(node: &Value, re: &Regex, own_name: &str) -> Value {
    match node {
        Value::String(s) => Value::String(re.replace_all(s, "the memory system").to_string()),
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (key, val) in map {
                if key == "name" {
                    if let Value::String(s) = val {
                        if s == own_name {
                            out.insert(key.clone(), val.clone());
                            continue;
                        }
                    }
                }
                let clean_key = re.replace_all(key, "").to_string();
                out.insert(clean_key, scrub_json_strings(val, re, own_name));
            }
            Value::Object(out)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(|item| scrub_json_strings(item, re, own_name)).collect())
        }
        _ => node.clone(),
    }
}

pub fn scrub_hidden_tool_names(tool: &Value, own_name: &str, hidden: &[String]) -> Value {
    if let Some(re) = hidden_tool_regexp(hidden) {
        scrub_json_strings(tool, &re, own_name)
    } else {
        tool.clone()
    }
}

pub fn filter_mcp_tools_response(line: &[u8], allowed_tools: &[&str]) -> Vec<u8> {
    let mut response: Value = match serde_json::from_slice(line) {
        Ok(v) => v,
        Err(_) => return line.to_vec(),
    };
    let result = match response.get_mut("result").and_then(|v| v.as_object_mut()) {
        Some(r) => r,
        None => return line.to_vec(),
    };
    let tools = match result.get("tools").and_then(|v| v.as_array()) {
        Some(t) => t.clone(),
        None => return line.to_vec(),
    };

    let mut by_name = serde_json::Map::new();
    let mut names_set = HashSet::new();
    for tool in tools {
        if let Some(name) = tool.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()) {
            names_set.insert(name.clone());
            by_name.insert(name, tool);
        }
    }

    let allowed_set: HashSet<String> = allowed_tools.iter().map(|s| s.to_string()).collect();
    let hidden = build_hidden_tool_names(&names_set, &allowed_set, static_tool_universe(allowed_tools));

    let mut filtered = Vec::new();
    for &name in allowed_tools {
        if let Some(tool) = by_name.get(name) {
            let tool = if !hidden.is_empty() {
                scrub_hidden_tool_names(tool, name, &hidden)
            } else {
                tool.clone()
            };
            filtered.push(tool);
        }
    }

    result.insert("tools".to_string(), Value::Array(filtered));
    serde_json::to_vec(&response).unwrap_or_else(|_| line.to_vec())
}

pub fn rewrite_mcp_instructions(line: &[u8], instructions: &str) -> Option<Vec<u8>> {
    let mut response: Value = serde_json::from_slice(line).ok()?;
    let result = response.get_mut("result")?.as_object_mut()?;
    if !result.contains_key("protocolVersion") {
        return None;
    }
    result.insert("instructions".to_string(), Value::String(instructions.to_string()));
    serde_json::to_vec(&response).ok()
}

fn run_bounded_mcp_proxy(child: &mut Child, allowed: &[&str], is_projectmem: bool) -> i32 {
    let mut child_in = child.stdin.take().unwrap();
    let child_out = child.stdout.take().unwrap();

    let list_ids = Arc::new(RwLock::new(McpRequestIDs {
        init_ids: HashSet::new(),
    }));
    let locked_output = Arc::new(McpLockedWriter::new(io::stdout()));

    let allowed_vec: Vec<String> = allowed.iter().map(|s| s.to_string()).collect();
    let allowed_for_in = allowed_vec.clone();
    let ids_in = list_ids.clone();
    let out_for_in = locked_output.clone();

    // Client -> Child
    let t_in = std::thread::spawn(move || {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);
        loop {
            let (message, framing) = match read_mcp_message(&mut reader) {
                Ok(Some(pair)) => pair,
                Ok(None) => break,
                Err(_) => break,
            };
            let trimmed = message.trim_ascii();
            if trimmed.starts_with(b"[") {
                let _ = out_for_in.write_mcp_message(framing, &mcp_invalid_request_response());
                continue;
            }
            let parsed: Result<Value, _> = serde_json::from_slice(&message);
            match parsed {
                Ok(Value::Object(map)) => {
                    let id_val = map.get("id");
                    let method = map.get("method").and_then(|m| m.as_str());

                    if let Some(id) = id_val.and_then(canonical_mcp_id) {
                        if method == Some("initialize") {
                            ids_in.write().unwrap().init_ids.insert(id);
                        }
                    }

                    if method == Some("tools/call") {
                        let tool_name = map.get("params")
                            .and_then(|p| p.get("name"))
                            .and_then(|n| n.as_str())
                            .unwrap_or("");
                        if !allowed_for_in.contains(&tool_name.to_string()) {
                            let _ = out_for_in.write_mcp_message(
                                framing,
                                &mcp_tool_denied_response(id_val, tool_name),
                            );
                            continue;
                        }
                    }

                    if write_mcp_message_unlocked(&mut child_in, framing, &message).is_err() {
                        break;
                    }
                }
                _ => {
                    // Try to extract raw id for parse error response
                    let raw_id = serde_json::from_slice::<Value>(&message)
                        .ok()
                        .and_then(|v| v.get("id").cloned());
                    if raw_id.is_some() {
                        let _ = out_for_in.write_mcp_message(framing, &mcp_parse_error_response(raw_id.as_ref()));
                    }
                }
            }
        }
    });

    // Child -> Client
    let ids_out = list_ids.clone();
    let out_for_out = locked_output.clone();
    let allowed_for_out = allowed_vec.clone();
    let allowed_refs: Vec<&str> = allowed_for_out.iter().map(|s| s.as_str()).collect();

    let mut reader = BufReader::new(child_out);
    loop {
        let (mut message, framing) = match read_mcp_message(&mut reader) {
            Ok(Some(pair)) => pair,
            Ok(None) => break,
            Err(_) => break,
        };

        if let Ok(Value::Object(map)) = serde_json::from_slice::<Value>(&message) {
            if let Some(result) = map.get("result") {
                if result.get("tools").is_some() {
                    message = filter_mcp_tools_response(&message, &allowed_refs);
                }
                if is_projectmem {
                    if let Some(id) = map.get("id").and_then(canonical_mcp_id) {
                        if ids_out.read().unwrap().init_ids.contains(&id) {
                            if let Some(rewritten) = rewrite_mcp_instructions(&message, PROJECTMEM_MCP_INSTRUCTIONS) {
                                message = rewritten;
                            }
                        }
                    }
                }
            }
        }

        if out_for_out.write_mcp_message(framing, &message).is_err() {
            break;
        }
    }

    let status = child.wait();
    let _ = t_in.join();
    status.map(|s| s.code().unwrap_or(0)).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_cmd_batch_args() {
        let args = vec!["/c".into(), "C:/tools/codegraph.cmd".into(), "serve".into()];
        let norm = normalized_cmd_batch_args("cmd", &args, true);
        assert_eq!(norm, vec!["/c", r"C:\tools\codegraph.cmd", "serve"]);

        let norm_upper = normalized_cmd_batch_args(r"C:\Windows\System32\cmd.exe", &args, true);
        assert_eq!(norm_upper, vec!["/c", r"C:\tools\codegraph.cmd", "serve"]);

        let norm_non_win = normalized_cmd_batch_args("cmd.exe", &args, false);
        assert_eq!(norm_non_win, args);
    }

    #[test]
    fn test_canonical_mcp_id() {
        assert_eq!(canonical_mcp_id(&serde_json::json!("req-1")), Some("string:req-1".into()));
        assert_eq!(canonical_mcp_id(&serde_json::json!(42)), Some("number:42e0".into()));
        assert_eq!(canonical_mcp_id(&serde_json::json!(100)), Some("number:1e2".into()));
        assert_eq!(canonical_mcp_id(&serde_json::json!(0)), Some("number:0".into()));
        assert_eq!(canonical_mcp_id(&serde_json::json!(null)), None);
    }

    #[test]
    fn test_filter_mcp_tools_response() {
        let raw = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "tools": [
                    { "name": "log_issue", "description": "Log an issue using log_issue or get_issue" },
                    { "name": "get_issue", "description": "Get an issue" },
                    { "name": "add_note", "description": "Add a note" }
                ]
            }
        });
        let bytes = serde_json::to_vec(&raw).unwrap();
        let filtered_bytes = filter_mcp_tools_response(&bytes, &["log_issue", "add_note"]);
        let filtered: Value = serde_json::from_slice(&filtered_bytes).unwrap();
        let tools = filtered["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0]["name"], "log_issue");
        assert_eq!(tools[1]["name"], "add_note");
        // Hidden tool 'get_issue' should have been scrubbed from descriptions
        let desc = tools[0]["description"].as_str().unwrap();
        assert!(desc.contains("the memory system"));
        assert!(!desc.contains("get_issue"));
    }

    #[test]
    fn test_rewrite_mcp_instructions() {
        let raw = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "init-1",
            "result": {
                "protocolVersion": "2024-11-05",
                "serverInfo": { "name": "projectmem", "version": "1.0.0" }
            }
        });
        let bytes = serde_json::to_vec(&raw).unwrap();
        let rewritten_bytes = rewrite_mcp_instructions(&bytes, "Custom instructions").unwrap();
        let rewritten: Value = serde_json::from_slice(&rewritten_bytes).unwrap();
        assert_eq!(rewritten["result"]["instructions"], "Custom instructions");
    }

    #[test]
    fn test_mcp_invalid_request_response() {
        let resp = mcp_invalid_request_response();
        let val: Value = serde_json::from_slice(&resp).unwrap();
        assert_eq!(val["error"]["code"], -32600);
    }

    #[test]
    fn test_mcp_tool_denied_response() {
        let id = serde_json::json!(123);
        let resp = mcp_tool_denied_response(Some(&id), "forbidden_tool");
        let val: Value = serde_json::from_slice(&resp).unwrap();
        assert_eq!(val["error"]["code"], -32601);
        assert_eq!(val["id"], 123);
    }
}

