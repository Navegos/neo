// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/tools/ponytail.rs
// created: 2026-10-05
// lastModified: 2026-10-31

use std::sync::Arc;
use crate::core::{Channel, ToolManifest};
use crate::util::instructions::{cursor_project_rule_content, install_cursor_project_rules};
use crate::util::paths::{cursor_paths_resolved, read_file_safe};
use super::copilot_transaction::with_copilot_transaction;
use super::helpers::{
    has_owner, kilo_has_owner, kilo_remove_owner, kilo_write_owner, remove_owner_safe,
    sync_copilot_ide_instructions_safe, write_owner,
};

pub fn build_ponytail() -> ToolManifest {
    let mut tool = ToolManifest::new("ponytail", "Ponytail");
    tool.description = "Minimal-code rules: reuse first, avoid speculative work.".into();
    tool.homepage = "in-tree".into();
    tool.install_hint = "Instruction-only — distributed inside tokless binary.".into();
    tool.channel = Channel::Binary;
    tool.not_trackable = true;
    tool.instruction_only = true;
    tool.install = Some(Arc::new(|_opts| Ok(true)));

    let standard_agents = [
        "claude", "opencode", "opencode2", "codex", "antigravity", "droid", "grok", "pi", "omp",
        "cline", "amp", "kimi", "zcode", "dsh",
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
                if !write_owner(&ag_w, "ponytail") {
                    return Err(format!("failed to write ponytail instructions for {}", ag_w).into());
                }
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
                remove_owner_safe(&ag_u, "ponytail")?;
                Ok(true)
            }),
        );

        let ag_v = ag.clone();
        tool.verify_for.insert(
            ag.clone(),
            Arc::new(move || Some(has_owner(&ag_v, "ponytail"))),
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
                if !write_owner("copilot", "ponytail") {
                    return Err("failed to write copilot ponytail instructions".into());
                }
                sync_copilot_ide_instructions_safe()?;
                Ok(has_owner("copilot", "ponytail"))
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
                remove_owner_safe("copilot", "ponytail")?;
                sync_copilot_ide_instructions_safe()?;
                Ok(true)
            })
        }),
    );
    tool.verify_for.insert(
        "copilot".into(),
        Arc::new(|| Some(has_owner("copilot", "ponytail"))),
    );

    // Kilo
    tool.wire_for.insert(
        "kilo".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            if !kilo_write_owner("ponytail") {
                return Err("failed to write kilo ponytail instructions".into());
            }
            Ok(kilo_has_owner("ponytail"))
        }),
    );
    tool.unwire_for.insert(
        "kilo".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            Ok(kilo_remove_owner("ponytail"))
        }),
    );
    tool.verify_for.insert(
        "kilo".into(),
        Arc::new(|| Some(kilo_has_owner("ponytail"))),
    );

    // Cursor
    tool.wire_for.insert(
        "cursor".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            let _ = install_cursor_project_rules(".", false);
            let global_dir = cursor_paths_resolved().rules_dir;
            if !global_dir.is_empty() {
                let _ = std::fs::create_dir_all(&global_dir);
                let p = std::path::Path::new(&global_dir).join("build-discipline.mdc");
                let _ = crate::util::paths::write_file(&p, &cursor_project_rule_content("ponytail"));
            }
            Ok(true)
        }),
    );
    tool.unwire_for.insert(
        "cursor".into(),
        Arc::new(|opts| {
            if opts.dry_run {
                return Ok(true);
            }
            for name in &["build-discipline.mdc", "ponytail.mdc"] {
                let p1 = std::path::Path::new(&cursor_paths_resolved().rules_dir).join(name);
                if p1.exists() {
                    let _ = std::fs::remove_file(p1);
                }
                let p2 = std::path::Path::new(".cursor").join("rules").join(name);
                if p2.exists() {
                    let _ = std::fs::remove_file(p2);
                }
            }
            Ok(true)
        }),
    );
    tool.verify_for.insert(
        "cursor".into(),
        Arc::new(|| {
            let dir = cursor_paths_resolved().rules_dir;
            for name in &["build-discipline.mdc", "ponytail.mdc"] {
                let p1 = std::path::Path::new(&dir).join(name);
                let p2 = std::path::Path::new(".cursor").join("rules").join(name);
                let target = if p1.exists() {
                    Some(p1)
                } else if p2.exists() {
                    Some(p2)
                } else {
                    None
                };
                if let Some(path) = target {
                    if let Some(raw) = read_file_safe(path.to_string_lossy().as_ref()) {
                        let expected = cursor_project_rule_content("ponytail");
                        if !expected.is_empty() && raw.replace("\r\n", "\n") == expected.replace("\r\n", "\n") {
                            return Some(true);
                        }
                    }
                }
            }
            Some(false)
        }),
    );

    tool
}
