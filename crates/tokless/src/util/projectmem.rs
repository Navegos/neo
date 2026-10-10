use super::paths::which;
use super::exec::{run, RunOptions};

pub fn projectmem_installed() -> bool {
    !which("pjm-mcp").is_empty()
}

pub fn projectmem_installed_version() -> Option<String> {
    let uv = which("uv");
    if uv.is_empty() {
        return None;
    }
    let res = run(&uv, &["tool", "list"], RunOptions { capture: true, ..Default::default() });
    if res.code != 0 {
        return None;
    }
    for line in res.stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 && parts[0] == "projectmem" {
            return Some(parts[1].trim_start_matches('v').to_string());
        }
    }
    None
}

