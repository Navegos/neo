// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/phase18_4_doc_and_checkpoint_hygiene.rs

#![deny(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use toksi::commands::doctor::{consolidate_nested_toksi_dirs, detect_loose_dot_folders};
use toksi::projectmem::store::{consolidate_nested_mem_dirs, read_events};
use toksi::rules::audit_docs;

fn repo_root() -> PathBuf {
    Path::new("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("../.."))
}

#[test]
fn test_phase18_4_brain_guides_37_files_and_prereads() {
    let root = repo_root();
    let brain_dir = root.join(".agents/brain");
    assert!(brain_dir.is_dir(), ".agents/brain must exist");

    let mut md_files: Vec<String> = fs::read_dir(&brain_dir)
        .expect("read .agents/brain")
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.is_file() && p.extension().and_then(|x| x.to_str()) == Some("md") {
                p.file_name().and_then(|n| n.to_str()).map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    md_files.sort();

    assert_eq!(
        md_files.len(),
        37,
        "Expected exact 37 .agents/brain/*.md files (INDEX_AND_EXECUTION_GUIDE.md + PHASE_01..PHASE_18_6)"
    );
    assert!(md_files.iter().any(|f| f == "INDEX_AND_EXECUTION_GUIDE.md"));

    let phase_files: Vec<&String> = md_files.iter().filter(|f| f.starts_with("PHASE_")).collect();
    assert_eq!(phase_files.len(), 36, "Expected 36 PHASE_*.md files");

    for pf in phase_files {
        let content = fs::read_to_string(brain_dir.join(pf)).expect("read phase file");
        assert!(
            content.contains(".toksi/projectmem/plan.md"),
            "{} missing .toksi/projectmem/plan.md pre-read",
            pf
        );
        assert!(
            content.contains(".toksi/projectmem/PROJECT_MAP.md"),
            "{} missing .toksi/projectmem/PROJECT_MAP.md pre-read",
            pf
        );
        assert!(
            content.contains(".toksi/projectmem/summary.md"),
            "{} missing .toksi/projectmem/summary.md pre-read",
            pf
        );
    }
}

#[test]
fn test_phase18_4_audit_docs_zero_stale_projectmem_prereads() {
    let root = repo_root();
    let report = audit_docs(&root).expect("audit_docs should succeed");
    assert!(report.files_scanned >= 43, "Expected >= 43 docs scanned, got {}", report.files_scanned);
    assert!(
        report.findings.is_empty(),
        "Stale .projectmem/ pre-read references found: {:?}",
        report.findings
    );
}

#[test]
fn test_phase18_4_master_docs_topology_and_parity() {
    let root = repo_root();
    let doc_paths = [
        "crates/toksi/README.md",
        "crates/toksi/PLAN.md",
        "crates/toksi/CONSOLIDATION_PLAN.md",
        "crates/toksi/research/RESEARCHPLAN.md",
        ".toksi/projectmem/plan.md",
        ".toksi/projectmem/PROJECT_MAP.md",
        ".toksi/projectmem/summary.md",
    ];

    for rel in doc_paths {
        let full = root.join(rel);
        assert!(full.is_file(), "Missing master doc {}", rel);
        let content = fs::read_to_string(&full).expect("read master doc");

        // No phantom crates/tokless/* usage outside explicit cleanup/prohibition notices
        for (idx, line) in content.lines().enumerate() {
            if line.contains("crates/tokless") {
                let allowed = line.contains("zero phantom")
                    || line.contains("Eliminate stale")
                    || line.contains("removing all stale")
                    || line.contains("legacy `crates/tokless/*`");
                assert!(
                    allowed,
                    "Unexpected phantom crates/tokless reference in {}:{} -> {}",
                    rel,
                    idx + 1,
                    line
                );
            }
        }

        // Core Wave 18 / v5.2 invariants present across all 7 master docs
        assert!(content.contains("295"), "{} missing 295 mandated vaults", rel);
        assert!(content.contains("<toksi-siagent>"), "{} missing <toksi-siagent>", rel);
        assert!(content.contains("<advisory>"), "{} missing <advisory> 3-repo topology", rel);
        assert!(content.contains("<skills>"), "{} missing <skills> 3-repo topology", rel);
    }

    let plan = fs::read_to_string(root.join("crates/toksi/PLAN.md")).expect("read PLAN.md");
    for port in ["8787", "8788", "8789", "8790", "9098", "9099", "9100", "18787"] {
        assert!(plan.contains(port), "PLAN.md missing port {}", port);
    }
    assert!(plan.contains("HeadroomEffortCompressor"));
    assert!(plan.contains("DirectWireBus"));
    assert!(plan.contains("src/plugins/"));
    assert!(plan.contains("plugins_registry"));
    assert!(plan.contains("src/validator/"));
    assert!(plan.contains("adr.rs"));
    assert!(plan.contains("sandbox.rs"));
    assert!(plan.contains("guard.rs"));
    assert!(plan.contains("defenseclaw"));
    assert!(plan.contains("skill-scanner"));
    for tok in ["opa", "cedar", "spire", "Guardrails", "llm-guard"] {
        assert!(plan.contains(tok), "PLAN.md missing {}", tok);
    }
    for slash in [
        "/byok", "/model", "/effort", "/byokmodel", "/projectmem", "/context",
        "/codegraph", "/icm", "/ai-memory", "/rtk", "/security", "/advisory",
        "/skills", "/sandbox", "/plugin", "/resume",
    ] {
        assert!(plan.contains(slash), "PLAN.md missing slash command {}", slash);
    }

    let readme = fs::read_to_string(root.join("crates/toksi/README.md")).expect("read README.md");
    assert!(readme.contains("DirectWireBus"));
    assert!(readme.contains("HeadroomEffortCompressor") || readme.contains("Headroom Effort Compressor"));
    assert!(readme.contains("src/plugins/"));
    assert!(readme.contains("src/validator/"));
    assert!(readme.contains("src/security/adr.rs"));
    assert!(readme.contains("src/agent_runtime/sandbox.rs"));
    assert!(readme.contains("src/commands/guard.rs"));
}

#[test]
fn test_phase18_4_repo_has_zero_loose_dot_folders() {
    let root = repo_root();
    let loose = detect_loose_dot_folders(&root);
    assert!(
        loose.is_empty(),
        "Found loose legacy/nested dot-folders outside <repo>/.toksi/: {:?}",
        loose
    );
}

#[test]
fn test_phase18_4_loose_dot_folder_migration_in_temp_sandbox() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path();
    fs::create_dir_all(root.join(".git")).expect("create .git");

    // Seed loose root .projectmem with events.jsonl and plan_checkpoints/
    let legacy_pm = root.join(".projectmem");
    fs::create_dir_all(legacy_pm.join("plan_checkpoints")).expect("mkdir plan_checkpoints");
    let mut ev = toksi::projectmem::schema::Event::new("note", "legacy note");
    ev.id = "0001".to_string();
    fs::write(
        legacy_pm.join("events.jsonl"),
        serde_json::to_string(&ev).expect("serialize event"),
    )
    .expect("write events.jsonl");
    fs::write(
        legacy_pm.join("plan_checkpoints/cp1.json"),
        r#"{"plan_id":"cp1","status":"paused"}"#,
    )
    .expect("write cp1.json");

    // Seed loose root & nested tool folders
    for loose in [
        ".codegraph",
        ".context-mode",
        ".icm",
        ".ai-memory",
        ".rtk",
        "crates/sub/.projectmem",
        "crates/sub/.rtk",
    ] {
        let dir = root.join(loose);
        fs::create_dir_all(&dir).expect("mkdir loose");
        fs::write(dir.join("marker.txt"), "ok").expect("write marker");
    }

    assert!(!detect_loose_dot_folders(root).is_empty());

    let mem_migrated = consolidate_nested_mem_dirs(root).expect("consolidate_nested_mem_dirs");
    assert!(mem_migrated >= 2, "Expected >= 2 projectmem items migrated, got {}", mem_migrated);

    let toksi_migrated = consolidate_nested_toksi_dirs(root).expect("consolidate_nested_toksi_dirs");
    assert!(toksi_migrated >= 5, "Expected >= 5 tool dirs migrated, got {}", toksi_migrated);

    assert!(
        detect_loose_dot_folders(root).is_empty(),
        "Expected zero loose dot-folders after migration"
    );
    assert!(root.join(".toksi/projectmem/plan_checkpoints/cp1.json").is_file());
    let events = read_events(Some(root)).expect("read migrated events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, "0001");
}
