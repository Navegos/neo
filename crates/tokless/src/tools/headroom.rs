use std::sync::Arc;
use crate::core::{Channel, ToolManifest};
use crate::util::headroom::headroom_installed;

pub fn build_headroom() -> ToolManifest {
    let mut tool = ToolManifest::new("headroom", "Headroom");
    tool.description = "Universal HTTP caching and routing proxy daemon for coding agents.".into();
    tool.homepage = "https://github.com/headroom-ai/headroom".into();
    tool.install_hint = "uv tool install headroom-ai[proxy]".into();
    tool.channel = Channel::UV;
    tool.install = Some(Arc::new(|_opts| {
        Ok(headroom_installed())
    }));

    let agents = [
        "claude", "opencode", "codex", "antigravity", "copilot", "droid", "pi", "omp", "kilo", "cline",
    ];

    for agent_id in agents {
        let ag = agent_id.to_string();
        tool.wire_for.insert(
            ag.clone(),
            Arc::new(|opts| {
                if opts.dry_run {
                    return Ok(true);
                }
                Ok(true)
            }),
        );
        tool.unwire_for.insert(
            ag.clone(),
            Arc::new(|opts| {
                if opts.dry_run {
                    return Ok(true);
                }
                Ok(true)
            }),
        );
        tool.verify_for.insert(
            ag.clone(),
            Arc::new(|| Some(true)),
        );
    }

    tool
}

