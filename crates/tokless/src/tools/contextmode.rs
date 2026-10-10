use std::sync::Arc;
use crate::core::{Channel, ToolManifest};
use crate::util::paths::which;
use super::helpers::{has_owner, remove_owner, write_owner};

pub fn build_context_mode() -> ToolManifest {
    let mut tool = ToolManifest::new("context-mode", "Context Mode");
    tool.description = "Sandboxed context tools with session memory and focused analysis.".into();
    tool.homepage = "https://github.com/mksglu/context-mode".into();
    tool.install_hint = "npm install -g context-mode".into();
    tool.channel = Channel::Npm;
    tool.needs_node = true;
    tool.min_node_major = 18;
    tool.install = Some(Arc::new(|_opts| {
        Ok(!which("context-mode").is_empty() || !which("npx").is_empty())
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
                write_owner(&ag_w, "context-mode");
                Ok(has_owner(&ag_w, "context-mode"))
            }),
        );

        let ag_u = ag.clone();
        tool.unwire_for.insert(
            ag.clone(),
            Arc::new(move |opts| {
                if opts.dry_run {
                    return Ok(true);
                }
                remove_owner(&ag_u, "context-mode");
                Ok(true)
            }),
        );

        let ag_v = ag.clone();
        tool.verify_for.insert(
            ag.clone(),
            Arc::new(move || Some(has_owner(&ag_v, "context-mode"))),
        );
    }

    tool
}

