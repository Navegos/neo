use std::sync::Arc;
use crate::core::{Channel, ToolManifest};
use crate::util::projectmem::projectmem_installed;
use super::helpers::{has_owner, remove_owner, write_owner};

pub fn build_projectmem() -> ToolManifest {
    let mut tool = ToolManifest::new("projectmem", "Project Memory");
    tool.description = "Cross-session persistent project memory in .projectmem/".into();
    tool.homepage = "https://github.com/HoangP8/tokless".into();
    tool.install_hint = "uv tool install projectmem".into();
    tool.channel = Channel::UV;
    tool.install = Some(Arc::new(|_opts| {
        Ok(projectmem_installed())
    }));

    let agents = [
        "claude", "opencode", "codex", "antigravity", "copilot", "droid", "grok", "pi", "omp", "kilo", "cline",
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
                write_owner(&ag_w, "projectmem");
                Ok(has_owner(&ag_w, "projectmem"))
            }),
        );

        let ag_u = ag.clone();
        tool.unwire_for.insert(
            ag.clone(),
            Arc::new(move |opts| {
                if opts.dry_run {
                    return Ok(true);
                }
                remove_owner(&ag_u, "projectmem");
                Ok(true)
            }),
        );

        let ag_v = ag.clone();
        tool.verify_for.insert(
            ag.clone(),
            Arc::new(move || Some(has_owner(&ag_v, "projectmem"))),
        );
    }

    tool
}

pub const PROJECTMEM_MCP_INSTRUCTIONS: &str = "projectmem writer tools: log_issue, record_attempt, record_fix, add_decision, add_note.";

pub fn ensure_projectmem_project(root: &str) -> Result<(), String> {
    if root.is_empty() {
        return Ok(());
    }
    let p = std::path::Path::new(root).join(".projectmem");
    let _ = std::fs::create_dir_all(&p);
    Ok(())
}

pub fn refresh_projectmem_rule_files(_root: &str) {}

