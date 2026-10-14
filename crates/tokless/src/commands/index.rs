// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/commands/index.rs
// created: 2026-10-05
// lastModified: 2026-10-14

use std::collections::HashSet;
use std::env;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::commands::helpers::InitOptions;
use crate::core::{list_tools, RunOpts};
use crate::tools::codegraph::ensure_codegraph_index;
use crate::tools::projectmem::{
    ensure_projectmem_project, projectmem_session_text, refresh_projectmem_rule_files,
};
use crate::util::colors::*;
use crate::util::instructions::install_cursor_project_rules;
use crate::util::jsonc::{stringify_json, try_parse_jsonc, OrderedMap, Value};
use crate::util::logger::L;
use crate::util::paths::{ensure_process_path, exists, is_win};

const PROJECT_MARKERS: &[&str] = &[
    ".git",
    "package.json",
    "go.mod",
    "Cargo.toml",
    "pyproject.toml",
    "pom.xml",
    "build.gradle",
    "tsconfig.json",
    "requirements.txt",
    ".projectmem",
];

pub fn looks_like_project(dir: &str) -> bool {
    if dir.is_empty() {
        return false;
    }
    let p = Path::new(dir);
    for m in PROJECT_MARKERS {
        if exists(p.join(m)) {
            return true;
        }
    }
    false
}

pub fn find_project_dir(dir: &str) -> String {
    let mut current = PathBuf::from(dir);
    loop {
        if looks_like_project(&current.to_string_lossy()) {
            return current.to_string_lossy().to_string();
        }
        if !current.pop() {
            break;
        }
    }
    dir.to_string()
}

pub fn cursor_project_dir(dir: &str) -> String {
    let p = find_project_dir(dir);
    if looks_like_project(&p) {
        p
    } else {
        String::new()
    }
}

pub fn normalize_cursor_hook_path(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches('"');
    if trimmed.is_empty() {
        return String::new();
    }
    if is_win() && trimmed.starts_with('/') && trimmed.len() >= 4 {
        let bytes = trimmed.as_bytes();
        if bytes[1].is_ascii_alphabetic() && (bytes[2] == b':' || bytes[2] == b'/') {
            if bytes[2] == b':' {
                return trimmed[1..].replace('/', "\\");
            }
            let drive = (bytes[1] as char).to_ascii_uppercase();
            return format!("{}:{}", drive, trimmed[2..].replace('/', "\\"));
        }
    }
    trimmed.to_string()
}

pub fn resolve_hook_project_dirs(raw_input: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    if !raw_input.trim().is_empty() {
        if let Some(Value::Object(map)) = try_parse_jsonc(raw_input) {
            for key in ["cwd", "workspace", "workspaceFolder", "projectDir", "rootPath"] {
                if let Some(Value::String(s)) = map.get(key) {
                    let norm = normalize_cursor_hook_path(s);
                    if !norm.is_empty() {
                        candidates.push(norm);
                    }
                }
            }
            if let Some(Value::Array(roots)) = map.get("workspace_roots").or_else(|| map.get("workspaceRoots")) {
                for r in roots {
                    if let Some(s) = r.as_str() {
                        let norm = normalize_cursor_hook_path(s);
                        if !norm.is_empty() {
                            candidates.push(norm);
                        }
                    }
                }
            }
        }
    }
    if candidates.is_empty() {
        if let Ok(cwd) = env::current_dir() {
            candidates.push(cwd.to_string_lossy().to_string());
        }
    }

    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for c in candidates {
        let root = find_project_dir(&c);
        if looks_like_project(&root) && seen.insert(root.clone()) {
            out.push(root);
        }
    }
    out
}

pub fn resolve_hook_project_dir_from_input(raw_input: &str) -> String {
    resolve_hook_project_dirs(raw_input)
        .into_iter()
        .next()
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| find_project_dir(&p.to_string_lossy()))
                .unwrap_or_else(|_| ".".into())
        })
}

pub fn run_index(opts: InitOptions, auto: bool) -> i32 {
    let cwd = match env::current_dir() {
        Ok(d) => d.to_string_lossy().to_string(),
        Err(e) => {
            L.err(&format!("cannot resolve current directory: {}", e));
            return if auto { 0 } else { 1 };
        }
    };

    let dir = if auto {
        let p = find_project_dir(&cwd);
        if !looks_like_project(&p) {
            return 0;
        }
        p
    } else {
        cwd
    };

    let all_tools = list_tools();
    let indexable: Vec<_> = all_tools.iter().filter(|t| t.index_project.is_some()).collect();

    if !auto {
        L.raw("");
        L.raw(&format!(
            "  {}  {}",
            C.bold(&C.cyan("tokless index")),
            C.gray(&format!("build per-project indexes in {}", dir))
        ));
        L.raw("");
    }

    if indexable.is_empty() {
        if !auto {
            L.raw(&format!("  {}", C.gray("no tools need a per-project index.")));
            L.raw("");
        }
        return 0;
    }

    let ro = RunOpts {
        dry_run: opts.dry_run,
        agent: opts.agent,
        ..Default::default()
    };

    let mut failed = 0;
    for t in indexable {
        if let Some(ref index_fn) = t.index_project {
            match index_fn(&dir, &ro) {
                Ok(true) => {
                    if !auto {
                        L.raw(&format!("  {} {}  {}", C.green("✔"), t.label, C.gray("indexed")));
                    }
                }
                Ok(false) => {
                    failed += 1;
                    if !auto {
                        L.raw(&format!(
                            "  {} {}  {}",
                            C.yellow("!"),
                            t.label,
                            C.gray("could not index")
                        ));
                    }
                }
                Err(e) => {
                    failed += 1;
                    if auto {
                        L.err(&format!("{} index: {}", t.label, e));
                    } else {
                        L.raw(&format!(
                            "  {} {}  {}",
                            C.red("✖"),
                            t.label,
                            C.gray(&e.to_string())
                        ));
                    }
                }
            }
        }
    }

    if auto {
        return 0;
    }

    L.raw("");
    if failed == 0 {
        L.raw(&format!("  {}", C.green("✔ Project indexed.")));
    } else {
        L.raw(&format!("  {} Some tools could not index.", C.yellow("⚠")));
    }
    L.raw("");

    if failed > 0 { 1 } else { 0 }
}

pub fn run_codegraph_index_hook() -> i32 {
    ensure_process_path();
    let mut input = Vec::new();
    let _ = io::stdin().read_to_end(&mut input);
    let raw = String::from_utf8_lossy(&input);
    for dir in resolve_hook_project_dirs(&raw) {
        let _ = ensure_codegraph_index(&dir, &RunOpts::default());
        let _ = ensure_projectmem_project(&dir);
        refresh_projectmem_rule_files(&dir);
    }
    0
}

pub fn run_cursor_project_rules_hook() -> i32 {
    ensure_process_path();
    let mut input = Vec::new();
    let _ = io::stdin().read_to_end(&mut input);
    let raw = String::from_utf8_lossy(&input);
    for dir in resolve_hook_project_dirs(&raw) {
        let _ = install_cursor_project_rules(&dir, false);
        let _ = ensure_projectmem_project(&dir);
        refresh_projectmem_rule_files(&dir);
    }
    0
}

pub fn run_grok_session_start_hook() -> i32 {
    ensure_process_path();
    let mut input = Vec::new();
    let _ = io::stdin().read_to_end(&mut input);
    let raw = String::from_utf8_lossy(&input);
    let root = resolve_hook_project_dir_from_input(&raw);
    if looks_like_project(&root) {
        let _ = ensure_codegraph_index(&root, &RunOpts::default());
        let _ = ensure_projectmem_project(&root);
        refresh_projectmem_rule_files(&root);
        let ctx = projectmem_session_text(&root);
        if !ctx.is_empty() {
            let mut hook_out = OrderedMap::new();
            hook_out.insert("hookEventName".into(), Value::String("SessionStart".into()));
            hook_out.insert("additionalContext".into(), Value::String(ctx));
            let mut resp = OrderedMap::new();
            resp.insert("hookSpecificOutput".into(), Value::Object(hook_out));
            println!("{}", stringify_json(&Value::Object(resp)));
        }
    }
    0
}
