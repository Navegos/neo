// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/nbuild/src/pipeline/mod.rs
// created: 2026-10-14
// lastModified: 2026-10-15

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::core::config::NeoConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with: Option<BTreeMap<String, serde_yaml::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    #[serde(rename = "runs-on", default = "default_runs_on")]
    pub runs_on: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    pub steps: Vec<WorkflowStep>,
}

fn default_runs_on() -> String {
    if cfg!(windows) { "windows".to_string() } else { "linux".to_string() }
}

impl Workflow {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read workflow {}", path.as_ref().display()))?;
        let wf: Workflow = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse YAML workflow {}", path.as_ref().display()))?;
        Ok(wf)
    }

    /// Execute workflow steps locally
    pub fn run_local(&self, config: &NeoConfig, quiet: bool) -> Result<()> {
        println!("[nbuild] Starting workflow: {}", self.name);

        let host_usr_bin = config.host_usr_bin();

        for (idx, step) in self.steps.iter().enumerate() {
            println!("[{}/{}] Step: {}", idx + 1, self.steps.len(), step.name);

            // Handle "uses: toolchain/setup"
            if let Some(uses) = &step.uses {
                if uses.contains("toolchain/setup") {
                    if let Some(with_map) = &step.with {
                        if let Some(tools_val) = with_map.get("tools") {
                            println!("  [toolchain/setup] Requesting tools: {:?}", tools_val);
                        }
                    }
                    continue;
                }
            }

            // Handle "run: ..."
            if let Some(script) = &step.run {
                let mut cmd = if cfg!(windows) {
                    let mut c = Command::new("powershell");
                    c.args(["-NoProfile", "-NonInteractive", "-Command", script]);
                    c
                } else {
                    let mut c = Command::new("sh");
                    c.args(["-c", script]);
                    c
                };

                // Inject PATH with host_usr_bin prepended
                if let Ok(current_path) = std::env::var("PATH") {
                    let sep = if cfg!(windows) { ";" } else { ":" };
                    let new_path = format!("{}{}{}", host_usr_bin.display(), sep, current_path);
                    cmd.env("PATH", new_path);
                }

                // Inject TOOLCHAINS_DIR
                cmd.env("TOOLCHAINS_DIR", &config.toolchains_dir);

                // Inject workflow-level env
                for (k, v) in &self.env {
                    cmd.env(k, v);
                }

                // Inject step-level env
                for (k, v) in &step.env {
                    cmd.env(k, v);
                }

                let output = cmd.output().context("Failed to spawn step process")?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    eprintln!("[ERROR] Step '{}' failed with code {:?}", step.name, output.status.code());
                    eprintln!("--- stderr (tail) ---\n{}", tail_lines(&stderr, 20));
                    if !stdout.is_empty() {
                        eprintln!("--- stdout (tail) ---\n{}", tail_lines(&stdout, 20));
                    }
                    bail!("Workflow '{}' aborted at step '{}'", self.name, step.name);
                } else if !quiet {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if !stdout.trim().is_empty() {
                        println!("{}", stdout.trim());
                    }
                }
            }
        }

        println!("[nbuild] Workflow '{}' completed successfully.", self.name);
        Ok(())
    }
}

fn tail_lines(s: &str, n: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    if lines.len() <= n {
        s.to_string()
    } else {
        lines[lines.len() - n..].join("\n")
    }
}
