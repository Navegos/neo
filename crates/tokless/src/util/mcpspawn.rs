use std::path::Path;
use serde::{Deserialize, Serialize};

use super::paths::{which, tokless_persisted_abs, IS_WIN};
use super::exec::resolve_codegraph_bin;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct McpSpawn {
    pub command: String,
    pub args: Vec<String>,
}

fn wrap_cmd_shim(s: McpSpawn) -> McpSpawn {
    if !IS_WIN {
        return s;
    }
    let mut p = s.command.clone();
    if !Path::new(&p).is_absolute() {
        p = which(&s.command);
    }
    let ext = Path::new(&p).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext == "cmd" || ext == "bat" {
        let mut new_args = vec!["/c".to_string(), p];
        new_args.extend(s.args);
        return McpSpawn {
            command: "cmd".to_string(),
            args: new_args,
        };
    }
    s
}

pub fn tokless_run_mcp_command() -> String {
    let self_exe = tokless_persisted_abs();
    if !self_exe.is_empty() {
        self_exe
    } else {
        "tokless".to_string()
    }
}

pub fn wrap_auto_index(agent: &str, s: McpSpawn) -> McpSpawn {
    let self_cmd = tokless_run_mcp_command();
    let mut args = vec!["run-mcp".to_string(), "--agent".to_string(), agent.to_string(), s.command];
    args.extend(s.args);
    McpSpawn {
        command: self_cmd,
        args,
    }
}

pub fn wrap_context_mode(s: McpSpawn) -> McpSpawn {
    let self_cmd = tokless_run_mcp_command();
    let mut args = vec!["run-mcp".to_string(), "--context-mode".to_string(), s.command];
    args.extend(s.args);
    McpSpawn {
        command: self_cmd,
        args,
    }
}

pub fn wrap_bounded_tool(tool_id: &str, s: McpSpawn) -> McpSpawn {
    let self_cmd = tokless_run_mcp_command();
    let mut args = vec!["run-mcp".to_string(), "--tool".to_string(), tool_id.to_string(), s.command];
    args.extend(s.args);
    McpSpawn {
        command: self_cmd,
        args,
    }
}

pub fn pick_codegraph_spawn(extra_args: &[&str]) -> Option<McpSpawn> {
    let p = resolve_codegraph_bin();
    if !p.is_empty() {
        let spawn = McpSpawn {
            command: p,
            args: extra_args.iter().map(|s| s.to_string()).collect(),
        };
        return Some(wrap_cmd_shim(spawn));
    }
    None
}

pub fn pick_mcp_spawn(bin: &str, extra_args: &[&str]) -> McpSpawn {
    let extra_vec: Vec<String> = extra_args.iter().map(|s| s.to_string()).collect();

    if bin == "codegraph" {
        if let Some(spawn) = pick_codegraph_spawn(extra_args) {
            return spawn;
        }
    } else if bin == "projectmem" {
        let mut spawn = wrap_cmd_shim(McpSpawn {
            command: "pjm-mcp".to_string(),
            args: extra_vec.clone(),
        });
        let p = which("pjm-mcp");
        if !p.is_empty() {
            spawn = wrap_cmd_shim(McpSpawn {
                command: p,
                args: extra_vec,
            });
        }
        return wrap_bounded_tool("projectmem", spawn);
    } else {
        let p = which(bin);
        if !p.is_empty() {
            let spawn = wrap_cmd_shim(McpSpawn {
                command: p,
                args: extra_vec,
            });
            if bin == "context-mode" {
                return wrap_context_mode(spawn);
            }
            return spawn;
        }
    }

    let pkg = match bin {
        "context-mode" => "context-mode",
        "codegraph" => "@colbymchenry/codegraph",
        _ => bin,
    };

    let mut args = vec!["--no-install".to_string(), pkg.to_string()];
    args.extend(extra_vec);
    let mut cmd = "npx".to_string();
    let p = which("npx");
    if !p.is_empty() {
        cmd = p;
    }
    let spawn = wrap_cmd_shim(McpSpawn { command: cmd, args });
    if bin == "context-mode" {
        return wrap_context_mode(spawn);
    }
    spawn
}
