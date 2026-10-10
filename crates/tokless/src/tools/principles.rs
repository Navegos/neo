use std::sync::Arc;
use crate::core::{Channel, ToolManifest};
use super::helpers::{has_owner, remove_owner, write_owner};

pub fn build_principles() -> ToolManifest {
    let mut tool = ToolManifest::new("principles", "Principles");
    tool.description = "Coding principles: think first, simplify, edit surgically, verify.".into();
    tool.homepage = "https://github.com/multica-ai/andrej-karpathy-skills".into();
    tool.install_hint = "Instruction-only — no install needed.".into();
    tool.channel = Channel::GitHub;
    tool.not_trackable = true;
    tool.instruction_only = true;
    tool.install = Some(Arc::new(|_opts| Ok(true)));

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
                write_owner(&ag_w, "principles");
                Ok(has_owner(&ag_w, "principles"))
            }),
        );

        let ag_u = ag.clone();
        tool.unwire_for.insert(
            ag.clone(),
            Arc::new(move |opts| {
                if opts.dry_run {
                    return Ok(true);
                }
                remove_owner(&ag_u, "principles");
                Ok(true)
            }),
        );

        let ag_v = ag.clone();
        tool.verify_for.insert(
            ag.clone(),
            Arc::new(move || Some(has_owner(&ag_v, "principles"))),
        );
    }

    tool
}

