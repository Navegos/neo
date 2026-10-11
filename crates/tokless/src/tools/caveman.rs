use std::sync::Arc;
use crate::core::{Channel, ToolManifest};
use crate::util::instructions::{cursor_project_rule_content, install_cursor_project_rules};
use crate::util::paths::{cursor_paths_resolved, read_file_safe};
use super::copilot_transaction::with_copilot_transaction;
use super::helpers::{
    has_owner, kilo_has_owner, kilo_remove_owner, kilo_write_owner, remove_owner_safe,
    sync_copilot_ide_instructions_safe, write_owner,
};

pub fn build_caveman() -> ToolManifest {
    let mut tool = ToolManifest::new("caveman", "Caveman");
    tool.description = "Compressed agent prompts through primitive English.".into();
    tool.homepage = "https://github.com/Navegos/caveman".into();
    tool.install_hint = "Instruction-only — no install needed.".into();
    tool.channel = Channel::GitHub;
    tool.not_trackable = true;
    tool.instruction_only = true;
    tool.install = Some(Arc::new(|_opts| Ok(true)));

    let standard_agents = [
        "claude", "opencode", "codex", "antigravity", "droid", "grok", "pi", "omp", "cline",
    ];

    for agent_id in standard_agents {
        let ag = agent_id.to_string();
        let ag_w = ag.clone();
        tool.wire_for.insert(
            ag.clone(),
            Arc::new(move |opts| {
                if opts.dry_run {
                    return Ok(true);
                }
                if !write_owner(&ag_w, "caveman") {
                    return Err(format!("failed to write caveman instructions for {}", ag_w).into());
                }
                Ok(has_owner(&ag_w, "caveman"))
            }),
        );

        let ag_u = ag.clone();
        tool.unwire_for.insert(
            ag.clone(),
            Arc::new(move |opts| {
                if opts.dry_run {
                    return Ok(true);
                }
                remove_owner_safe(&ag_u, "caveman")?;
                Ok(true)
            }),
        );

        let ag_v = ag.clone();
        tool.verify_for.insert(
            ag.clone(),
            Arc::new(move || Some(has_owner(&ag_v, "caveman"))),
        );
    }

    // Copilot
    tool.wire_for.insert(
        "copilot".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            with_copilot_transaction(&[], || {
                if !write_owner("copilot", "caveman") {
                    return Err("failed to write copilot caveman instructions".into());
                }
                sync_copilot_ide_instructions_safe()?;
                Ok(has_owner("copilot", "caveman"))
            })
        }),
    );
    tool.unwire_for.insert(
        "copilot".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            with_copilot_transaction(&[], || {
                remove_owner_safe("copilot", "caveman")?;
                sync_copilot_ide_instructions_safe()?;
                Ok(true)
            })
        }),
    );
    tool.verify_for.insert(
        "copilot".into(),
        Arc::new(|| Some(has_owner("copilot", "caveman"))),
    );

    // Kilo
    tool.wire_for.insert(
        "kilo".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            if !kilo_write_owner("caveman") {
                return Err("failed to write kilo caveman instructions".into());
            }
            Ok(kilo_has_owner("caveman"))
        }),
    );
    tool.unwire_for.insert(
        "kilo".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            Ok(kilo_remove_owner("caveman"))
        }),
    );
    tool.verify_for.insert(
        "kilo".into(),
        Arc::new(|| Some(kilo_has_owner("caveman"))),
    );

    // Cursor
    tool.wire_for.insert(
        "cursor".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            let _ = install_cursor_project_rules(".", false);
            Ok(true)
        }),
    );
    tool.unwire_for.insert(
        "cursor".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            let p = std::path::Path::new(&cursor_paths_resolved().rules_dir).join("caveman.mdc");
            if p.exists() {
                let _ = std::fs::remove_file(p);
            }
            Ok(true)
        }),
    );
    tool.verify_for.insert(
        "cursor".into(),
        Arc::new(|| {
            let p = std::path::Path::new(&cursor_paths_resolved().rules_dir)
                .join("caveman.mdc")
                .to_string_lossy()
                .to_string();
            if let Some(raw) = read_file_safe(&p) {
                let expected = cursor_project_rule_content("caveman");
                Some(!expected.is_empty() && raw.replace("\r\n", "\n") == expected.replace("\r\n", "\n"))
            } else {
                Some(true)
            }
        }),
    );

    tool
}
