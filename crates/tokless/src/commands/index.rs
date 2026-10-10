use std::env;
use std::path::{Path, PathBuf};
use crate::commands::helpers::InitOptions;
use crate::core::{list_tools, RunOpts};
use crate::tools::codegraph::ensure_codegraph_index;
use crate::tools::projectmem::{ensure_projectmem_project, refresh_projectmem_rule_files};
use crate::util::colors::*;
use crate::util::logger::L;
use crate::util::paths::exists;

const PROJECT_MARKERS: &[&str] = &[
    ".git", "package.json", "go.mod", "Cargo.toml",
    "pyproject.toml", "pom.xml", "build.gradle",
    "tsconfig.json", "requirements.txt", ".projectmem",
];

pub fn looks_like_project(dir: &str) -> bool {
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
                        L.raw(&format!("  {} {}  {}", C.yellow("!"), t.label, C.gray("could not index")));
                    }
                }
                Err(e) => {
                    failed += 1;
                    if auto {
                        L.err(&format!("{} index: {}", t.label, e));
                    } else {
                        L.raw(&format!("  {} {}  {}", C.red("✖"), t.label, C.gray(&e.to_string())));
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
    let cwd = env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| ".".into());
    let root = find_project_dir(&cwd);
    if looks_like_project(&root) {
        let _ = ensure_codegraph_index(&root, &RunOpts::default());
    }
    0
}

pub fn run_cursor_project_rules_hook() -> i32 {
    0
}

pub fn run_grok_session_start_hook() -> i32 {
    0
}

pub fn run_projectmem_hook(_agent: &str) -> i32 {
    let cwd = env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| ".".into());
    let root = find_project_dir(&cwd);
    let _ = ensure_projectmem_project(&root);
    refresh_projectmem_rule_files(&root);
    0
}
