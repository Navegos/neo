use std::sync::Arc;
use crate::core::{Channel, ToolManifest};
use super::helpers::{has_owner, remove_owner, write_owner};

pub fn build_ponytail() -> ToolManifest {
    let mut tool = ToolManifest::new("ponytail", "Ponytail");
    tool.description = "Minimal-code rules: reuse first, avoid speculative work.".into();
    tool.homepage = "https://github.com/DietrichGebert/ponytail".into();
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
                write_owner(&ag_w, "ponytail");
                Ok(has_owner(&ag_w, "ponytail"))
            }),
        );

        let ag_u = ag.clone();
        tool.unwire_for.insert(
            ag.clone(),
            Arc::new(move |opts| {
                if opts.dry_run {
                    return Ok(true);
                }
                remove_owner(&ag_u, "ponytail");
                Ok(true)
            }),
        );

        let ag_v = ag.clone();
        tool.verify_for.insert(
            ag.clone(),
            Arc::new(move || Some(has_owner(&ag_v, "ponytail"))),
        );
    }

    tool
}

