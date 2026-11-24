// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase19_1_ctxmode_rename_verification.rs
// created: 2026-11-24

use std::fs;
use toksi::agent_runtime::global_slash_engine;
use toksi::agents::hooks::{agent_mcp_to_skills_path, standard_hook_command};
use toksi::agents::mcp_adapters::normalize_mcp_tool_id;
use toksi::commands::doctor::consolidate_nested_toksi_dirs;
use toksi::contextmode::fts::{migrate_legacy_ctxmode_db, resolve_content_store_path, ContentStore};
use toksi::contextmode::store::{resolve_context_dir, DB_NAME};
use toksi::tools::multi_protocol::ToolTarget;

#[test]
fn test_phase19_1_canonical_ctxmode_and_aimemory_naming() {
    assert_eq!("ctxmode".parse::<ToolTarget>().unwrap(), ToolTarget::Ctxmode);
    assert_eq!("context".parse::<ToolTarget>().unwrap(), ToolTarget::Ctxmode);
    assert_eq!("context-mode".parse::<ToolTarget>().unwrap(), ToolTarget::Ctxmode);
    assert_eq!("aimemory".parse::<ToolTarget>().unwrap(), ToolTarget::Aimemory);
    assert_eq!("ai-memory".parse::<ToolTarget>().unwrap(), ToolTarget::Aimemory);

    assert_eq!(ToolTarget::Ctxmode.as_str(), "ctxmode");
    assert_eq!(ToolTarget::Aimemory.as_str(), "aimemory");

    assert_eq!(standard_hook_command("ctxmode", "cursor"), vec!["ctxmode-hook", "cursor"]);
    assert_eq!(standard_hook_command("context-mode", "agy"), vec!["ctxmode-hook", "agy"]);
    assert_eq!(standard_hook_command("ai-memory", "copilot"), vec!["aimemory-hook", "copilot"]);

    assert_eq!(normalize_mcp_tool_id("context-mode"), "ctxmode");
    assert_eq!(normalize_mcp_tool_id("toksi-context-mode"), "ctxmode");
    assert_eq!(normalize_mcp_tool_id("ai-memory"), "aimemory");
    assert_eq!(agent_mcp_to_skills_path(".gemini", "ctxmode"), ".gemini/skills/ctxmode");
    assert_eq!(agent_mcp_to_skills_path(".copilot", "context-mode"), ".copilot/skills/ctxmode");
}

#[test]
fn test_phase19_1_transactional_ctxmode_sqlite_migration() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path();
    fs::create_dir_all(repo.join(".git")).expect("mkdir .git");

    let legacy_ctx_dir = repo.join(".toksi").join("context");
    fs::create_dir_all(&legacy_ctx_dir).expect("mkdir legacy context");
    let legacy_db = legacy_ctx_dir.join("context.db");
    {
        let mut store = ContentStore::open(&legacy_db).expect("open legacy db");
        store
            .index("Important legacy FTS5 content chunk about quantum widgets.", "legacy_doc.md", Some("markdown"), Some("sess-1"))
            .expect("index legacy chunk");
    }
    assert!(legacy_db.is_file());

    let resolved_dir = resolve_context_dir(Some(repo)).expect("resolve ctxmode dir");
    assert!(resolved_dir.ends_with(".toksi/ctxmode") || resolved_dir.ends_with(".toksi\\ctxmode"));
    let target_db = resolved_dir.join(DB_NAME);
    assert!(target_db.is_file());
    assert!(!legacy_ctx_dir.exists(), "legacy .toksi/context dir should be removed after migration");

    let migrated_store = ContentStore::open(&target_db).expect("open migrated db");
    let sources = migrated_store.list_sources().expect("list migrated sources");
    assert!(sources.iter().any(|s| s.label == "legacy_doc.md"));

    // Also test direct helper migration from another legacy DB file
    let second_legacy = repo.join("extra_legacy.db");
    {
        let mut s2 = ContentStore::open(&second_legacy).expect("open extra db");
        s2.index("Extra chunk content for lossless merge", "extra_doc.md", None, None)
            .expect("index extra");
    }
    let count = migrate_legacy_ctxmode_db(&target_db, &second_legacy).expect("merge extra");
    assert_eq!(count, 1);
    assert!(!second_legacy.exists());

    let resolved_default = resolve_content_store_path(Some(repo));
    assert_eq!(resolved_default, target_db);
}

#[test]
fn test_phase19_1_doctor_consolidates_legacy_context_and_ai_memory() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path();
    fs::create_dir_all(repo.join(".git")).expect("mkdir .git");

    let legacy_ctx = repo.join(".context-mode");
    fs::create_dir_all(&legacy_ctx).expect("mkdir .context-mode");
    {
        let mut store = ContentStore::open(&legacy_ctx.join("context.db")).expect("open .context-mode db");
        store.index("Consolidate me safely", "doc.txt", Some("prose"), None).expect("index");
    }

    let legacy_ai = repo.join(".toksi").join("ai-memory");
    fs::create_dir_all(&legacy_ai).expect("mkdir legacy ai-memory");
    fs::write(legacy_ai.join("handoff.json"), "{}").expect("write handoff");

    let consolidated = consolidate_nested_toksi_dirs(repo).expect("consolidate");
    assert!(consolidated >= 2);
    assert!(!legacy_ctx.exists());
    assert!(!legacy_ai.exists());
    assert!(repo.join(".toksi").join("ctxmode").join("ctxmode.db").is_file());
    assert!(repo.join(".toksi").join("aimemory").join("handoff.json").is_file());
}

#[test]
fn test_phase19_1_slash_commands_and_deprecation_aliases() {
    let engine = global_slash_engine();

    let ctxmode_res = engine.execute("/ctxmode status", "tester");
    assert!(ctxmode_res.handled);
    assert!(ctxmode_res.output.contains("CtxMode") || ctxmode_res.output.contains("ctxmode"));

    let legacy_ctx_res = engine.execute("/context status", "tester");
    assert!(legacy_ctx_res.handled);
    assert!(legacy_ctx_res.output.contains("[deprecated: `/context` is renamed to `/ctxmode`]"));

    let aimem_res = engine.execute("/aimemory help", "tester");
    assert!(aimem_res.handled);
    assert!(aimem_res.output.contains("aimemory"));

    let legacy_aimem_res = engine.execute("/ai-memory help", "tester");
    assert!(legacy_aimem_res.handled);
    assert!(legacy_aimem_res.output.contains("[deprecated: `/ai-memory` is renamed to `/aimemory`]"));
}
