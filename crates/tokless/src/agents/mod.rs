pub mod hooks;
pub mod proxy_agent;
pub mod proxy_protocol;

use std::path::Path;
use std::sync::Arc;
use crate::core::{register_agent, AgentManifest, Detection};
use crate::util::paths::*;

pub use hooks::*;
pub use proxy_agent::*;
pub use proxy_protocol::*;

pub fn omp_agent_dir_resolved() -> String {
    Path::new(&home()).join(".omp").to_string_lossy().to_string()
}

pub fn pi_agent_dir_resolved() -> String {
    Path::new(&home()).join(".pi").to_string_lossy().to_string()
}


pub fn register() {
    register_claude();
    register_opencode();
    register_codex();
    register_cursor();
    register_antigravity();
    register_copilot();
    register_droid();
    register_grok();
    register_kilo();
    register_omp();
    register_pi();
    register_cline();
}

fn register_claude() {
    register_agent(AgentManifest {
        id: "claude".into(),
        label: "Claude Code".into(),
        homepage: "https://github.com/anthropics/claude-code".into(),
        cli_bin: "claude".into(),
        config_dir: Some(Arc::new(|| claude_code_paths().dir)),
        detect: Some(Arc::new(|| {
            let cli = !which("claude").is_empty();
            let cfg = exists(claude_code_paths().dir);
            if cli && cfg {
                Detection { installed: true, source: "cli+config".into() }
            } else if cli {
                Detection { installed: true, source: "cli".into() }
            } else if cfg {
                Detection { installed: true, source: "config".into() }
            } else {
                Detection { installed: false, source: "".into() }
            }
        })),
    });
}

fn register_opencode() {
    register_agent(AgentManifest {
        id: "opencode".into(),
        label: "OpenCode".into(),
        homepage: "https://github.com/anomalyco/opencode".into(),
        cli_bin: "opencode".into(),
        config_dir: Some(Arc::new(|| opencode_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            let cli = !which("opencode").is_empty();
            let cfg = exists(opencode_paths_resolved().dir) || exists(opencode_paths_resolved().config);
            if cli && cfg {
                Detection { installed: true, source: "cli+config".into() }
            } else if cli {
                Detection { installed: true, source: "cli".into() }
            } else if cfg {
                Detection { installed: true, source: "config".into() }
            } else {
                Detection { installed: false, source: "".into() }
            }
        })),
    });
}

fn register_codex() {
    register_agent(AgentManifest {
        id: "codex".into(),
        label: "Codex".into(),
        homepage: "https://github.com/openai/codex".into(),
        cli_bin: "codex".into(),
        config_dir: Some(Arc::new(|| codex_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            let cli = !which("codex").is_empty();
            let cfg = exists(codex_paths_resolved().config) || exists(codex_paths_resolved().dir);
            if cli && cfg {
                Detection { installed: true, source: "cli+config".into() }
            } else if cli {
                Detection { installed: true, source: "cli".into() }
            } else if cfg {
                Detection { installed: true, source: "config".into() }
            } else {
                Detection { installed: false, source: "".into() }
            }
        })),
    });
}

fn register_cursor() {
    register_agent(AgentManifest {
        id: "cursor".into(),
        label: "Cursor".into(),
        homepage: "https://cursor.com".into(),
        cli_bin: "cursor".into(),
        config_dir: Some(Arc::new(|| cursor_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            let cli = !which("cursor").is_empty();
            let cfg = exists(cursor_paths_resolved().dir);
            if cli && cfg {
                Detection { installed: true, source: "cli+config".into() }
            } else if cli {
                Detection { installed: true, source: "cli".into() }
            } else if cfg {
                Detection { installed: true, source: "config".into() }
            } else {
                Detection { installed: false, source: "".into() }
            }
        })),
    });
}

fn register_antigravity() {
    register_agent(AgentManifest {
        id: "antigravity".into(),
        label: "Antigravity".into(),
        homepage: "https://antigravity.google".into(),
        cli_bin: "agy".into(),
        config_dir: Some(Arc::new(|| antigravity_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            let cli = !which("agy").is_empty();
            let cfg = exists(antigravity_paths_resolved().dir) || exists(antigravity_paths_resolved().settings);
            if cli && cfg {
                Detection { installed: true, source: "cli+config".into() }
            } else if cli {
                Detection { installed: true, source: "cli".into() }
            } else if cfg {
                Detection { installed: true, source: "config".into() }
            } else {
                Detection { installed: false, source: "".into() }
            }
        })),
    });
}

fn register_copilot() {
    register_agent(AgentManifest {
        id: "copilot".into(),
        label: "GitHub Copilot".into(),
        homepage: "https://github.com/github/copilot-cli".into(),
        cli_bin: "copilot".into(),
        config_dir: Some(Arc::new(|| copilot_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            let cli = !which("copilot").is_empty();
            let cfg = exists(copilot_paths_resolved().dir) || exists(copilot_paths_resolved().settings);
            if cli && cfg {
                Detection { installed: true, source: "cli+config".into() }
            } else if cli {
                Detection { installed: true, source: "cli".into() }
            } else if cfg {
                Detection { installed: true, source: "config".into() }
            } else {
                Detection { installed: false, source: "".into() }
            }
        })),
    });
}

fn register_droid() {
    let droid_dir = Path::new(&home()).join(".factory").to_string_lossy().to_string();
    let d_dir = droid_dir.clone();
    register_agent(AgentManifest {
        id: "droid".into(),
        label: "Factory Droid".into(),
        homepage: "https://factory.ai".into(),
        cli_bin: "droid".into(),
        config_dir: Some(Arc::new(move || droid_dir.clone())),
        detect: Some(Arc::new(move || {
            let cli = !which("droid").is_empty();
            let cfg = exists(&d_dir);
            if cli && cfg {
                Detection { installed: true, source: "cli+config".into() }
            } else if cli {
                Detection { installed: true, source: "cli".into() }
            } else if cfg {
                Detection { installed: true, source: "config".into() }
            } else {
                Detection { installed: false, source: "".into() }
            }
        })),
    });
}

fn register_grok() {
    let grok_dir = Path::new(&home()).join(".grok").to_string_lossy().to_string();
    let g_dir = grok_dir.clone();
    register_agent(AgentManifest {
        id: "grok".into(),
        label: "Grok Build".into(),
        homepage: "https://x.ai/cli".into(),
        cli_bin: "grok".into(),
        config_dir: Some(Arc::new(move || grok_dir.clone())),
        detect: Some(Arc::new(move || {
            let cli = !which("grok").is_empty();
            let cfg = exists(&g_dir);
            if cli && cfg {
                Detection { installed: true, source: "cli+config".into() }
            } else if cli {
                Detection { installed: true, source: "cli".into() }
            } else if cfg {
                Detection { installed: true, source: "config".into() }
            } else {
                Detection { installed: false, source: "".into() }
            }
        })),
    });
}

fn register_kilo() {
    register_agent(AgentManifest {
        id: "kilo".into(),
        label: "Kilo Code".into(),
        homepage: "https://kilo.ai/cli".into(),
        cli_bin: "kilo".into(),
        config_dir: Some(Arc::new(|| kilo_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            let cli = !which("kilo").is_empty();
            let cfg = exists(kilo_paths_resolved().dir) || exists(kilo_paths_resolved().config);
            if cli && cfg {
                Detection { installed: true, source: "cli+config".into() }
            } else if cli {
                Detection { installed: true, source: "cli".into() }
            } else if cfg {
                Detection { installed: true, source: "config".into() }
            } else {
                Detection { installed: false, source: "".into() }
            }
        })),
    });
}

fn register_omp() {
    let omp_dir = Path::new(&home()).join(".omp").to_string_lossy().to_string();
    let o_dir = omp_dir.clone();
    register_agent(AgentManifest {
        id: "omp".into(),
        label: "Oh My Pi".into(),
        homepage: "https://github.com/can1357/oh-my-pi".into(),
        cli_bin: "omp".into(),
        config_dir: Some(Arc::new(move || omp_dir.clone())),
        detect: Some(Arc::new(move || {
            let cli = !which("omp").is_empty();
            let cfg = exists(&o_dir);
            if cli && cfg {
                Detection { installed: true, source: "cli+config".into() }
            } else if cli {
                Detection { installed: true, source: "cli".into() }
            } else if cfg {
                Detection { installed: true, source: "config".into() }
            } else {
                Detection { installed: false, source: "".into() }
            }
        })),
    });
}

fn register_pi() {
    let pi_dir = Path::new(&home()).join(".pi").to_string_lossy().to_string();
    let p_dir = pi_dir.clone();
    register_agent(AgentManifest {
        id: "pi".into(),
        label: "Pi".into(),
        homepage: "https://pi.dev".into(),
        cli_bin: "pi".into(),
        config_dir: Some(Arc::new(move || pi_dir.clone())),
        detect: Some(Arc::new(move || {
            let cli = !which("pi").is_empty();
            let cfg = exists(&p_dir);
            if cli && cfg {
                Detection { installed: true, source: "cli+config".into() }
            } else if cli {
                Detection { installed: true, source: "cli".into() }
            } else if cfg {
                Detection { installed: true, source: "config".into() }
            } else {
                Detection { installed: false, source: "".into() }
            }
        })),
    });
}

fn register_cline() {
    register_agent(AgentManifest {
        id: "cline".into(),
        label: "Cline".into(),
        homepage: "https://cline.bot/".into(),
        cli_bin: "cline".into(),
        config_dir: Some(Arc::new(|| cline_paths_resolved().dir)),
        detect: Some(Arc::new(|| {
            let cli = !which("cline").is_empty();
            let cfg = exists(cline_paths_resolved().dir) || exists(cline_paths_resolved().mcp_config);
            if cli && cfg {
                Detection { installed: true, source: "cli+config".into() }
            } else if cli {
                Detection { installed: true, source: "cli".into() }
            } else if cfg {
                Detection { installed: true, source: "config".into() }
            } else {
                Detection { installed: false, source: "".into() }
            }
        })),
    });
}

pub fn run_copilot_cli(_args: &[String]) -> i32 {
    0
}

pub fn sync_copilot_ide_instructions() {}
