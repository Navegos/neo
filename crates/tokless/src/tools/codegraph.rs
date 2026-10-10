use std::path::Path;
use std::sync::Arc;
use crate::core::{Channel, ToolManifest};
use crate::util::paths::*;
use crate::util::mcpspawn::pick_codegraph_spawn;
use super::helpers::{has_owner, remove_owner, write_owner};

pub fn build_codegraph() -> ToolManifest {
    let mut tool = ToolManifest::new("codegraph", "CodeGraph");
    tool.description = "Indexed code graph for source, call paths, and impact.".into();
    tool.homepage = "https://github.com/colbymchenry/codegraph".into();
    tool.install_hint = "npm install -g @colbymchenry/codegraph".into();
    tool.channel = Channel::Npm;
    tool.needs_node = true;
    tool.min_node_major = 18;
    tool.install = Some(Arc::new(|_opts| {
        Ok(pick_codegraph_spawn(&[]).is_some())
    }));
    tool.index_ready = Some(Arc::new(|| {
        exists(".codegraph")
    }));
    tool.index_project = Some(Arc::new(|dir, _opts| {
        let p = Path::new(dir).join(".codegraph");
        Ok(exists(p))
    }));

    let agents = [
        "claude", "opencode", "codex", "antigravity", "copilot", "droid", "grok", "pi", "omp", "kilo", "cline", "cursor",
    ];

    for agent_id in agents {
        let ag = agent_id.to_string();
        let ag_w = ag.clone();
        tool.wire_for.insert(
            ag.clone(),
            Arc::new(move |opts| {
                if opts.dry_run {
                    return Ok(true);
                }
                write_owner(&ag_w, "codegraph");
                Ok(has_owner(&ag_w, "codegraph"))
            }),
        );

        let ag_u = ag.clone();
        tool.unwire_for.insert(
            ag.clone(),
            Arc::new(move |opts| {
                if opts.dry_run {
                    return Ok(true);
                }
                remove_owner(&ag_u, "codegraph");
                Ok(true)
            }),
        );

        let ag_v = ag.clone();
        tool.verify_for.insert(
            ag.clone(),
            Arc::new(move || Some(has_owner(&ag_v, "codegraph"))),
        );
    }

    tool
}

pub fn ensure_codegraph_index(dir: &str, _opts: &crate::core::RunOpts) -> Result<bool, String> {
    let p = Path::new(dir).join(".codegraph");
    if exists(&p) {
        return Ok(true);
    }
    if let Some(spawn) = pick_codegraph_spawn(&[]) {
        let mut cmd = std::process::Command::new(&spawn.command);
        cmd.args(&spawn.args);
        cmd.arg("index");
        cmd.arg("--path");
        cmd.arg(dir);
        if let Ok(st) = cmd.status() {
            return Ok(st.success());
        }
    }
    Ok(false)
}

