// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/mod.rs
// created: 2026-10-28
// lastModified: 2026-11-18

#![deny(unsafe_code)]

pub mod cli;
pub mod cognee_graph;
pub mod embeddings;
pub mod funes_recall;
pub mod handoff;
pub mod lance_store;
pub mod letta_memgpt;
pub mod mcp;
pub mod mem0_bridge;
pub mod message;
pub mod milvus_adapter;
pub mod mongo_bridge;
pub mod qdrant_store;
pub mod retriever;
pub mod slash;
pub mod temporal_graph;
pub mod tiered;
pub mod wiki;
pub mod wire;

pub use cli::*;
pub use cognee_graph::*;
pub use embeddings::*;
pub use funes_recall::*;
pub use handoff::*;
pub use lance_store::*;
pub use letta_memgpt::*;
pub use mcp::*;
pub use mem0_bridge::*;
pub use message::*;
pub use milvus_adapter::*;
pub use mongo_bridge::*;
pub use qdrant_store::*;
pub use retriever::*;
pub use slash::*;
pub use temporal_graph::*;
pub use tiered::*;
pub use wiki::*;
pub use wire::*;

use std::env;
use std::path::{Path, PathBuf};

/// Strictly resolve the repository root and monorepo subproject_id.
/// Storage in user home root '~' is strictly prohibited.
pub fn resolve_aimemory_root_and_subproject(start: Option<&Path>) -> Result<(PathBuf, String), String> {
    crate::icm::store::find_repo_root_and_subproject(start)
        .map_err(|e| e.to_string())
}

fn migrate_dir_non_destructive(src: &Path, dst: &Path) {
    let Ok(entries) = std::fs::read_dir(src) else {
        return;
    };
    let _ = std::fs::create_dir_all(dst);
    for entry in entries.flatten() {
        let s = entry.path();
        let d = dst.join(entry.file_name());
        if s.is_dir() {
            migrate_dir_non_destructive(&s, &d);
            let _ = std::fs::remove_dir(&s);
        } else if !d.exists() && std::fs::copy(&s, &d).is_ok() {
            let _ = std::fs::remove_file(&s);
        }
    }
    let _ = std::fs::remove_dir(src);
}

pub fn default_aimemory_dir() -> PathBuf {
    let toksi_dir = if let Ok((repo_root, _)) = resolve_aimemory_root_and_subproject(None) {
        repo_root.join(".toksi")
    } else if let Ok(root) = env::var("PROJECTMEM_ROOT") {
        if !root.trim().is_empty() {
            PathBuf::from(root).join(".toksi")
        } else {
            PathBuf::from(".toksi")
        }
    } else {
        PathBuf::from(".toksi")
    };

    let target = toksi_dir.join("aimemory");
    let legacy = toksi_dir.join("ai-memory");
    if legacy.is_dir() {
        migrate_dir_non_destructive(&legacy, &target);
    }
    target
}

pub fn default_handoffs_dir() -> PathBuf {
    if let Ok(p) = env::var("TOKSI_HANDOFF_DIR")
        && !p.trim().is_empty() {
            let norm = crate::util::paths::normalize_path(&p);
            return PathBuf::from(norm);
        }
    default_aimemory_dir().join("handoffs")
}

pub fn default_wiki_dir() -> PathBuf {
    if let Ok(p) = env::var("TOKSI_WIKI_DIR")
        && !p.trim().is_empty() {
            let norm = crate::util::paths::normalize_path(&p);
            return PathBuf::from(norm);
        }
    default_aimemory_dir().join("wiki")
}

pub fn default_messages_dir() -> PathBuf {
    if let Ok(p) = env::var("TOKSI_MESSAGES_DIR")
        && !p.trim().is_empty() {
            let norm = crate::util::paths::normalize_path(&p);
            return PathBuf::from(norm);
        }
    default_aimemory_dir().join("messages")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_claim_once_handoff_lifecycle() {
        let dir = tempdir().expect("tempdir");
        let store = HandoffStore::new(dir.path());

        let new_h = NewHandoff {
            title: "Migrate sqlite schema".into(),
            task: "Update FTS5 trigger definitions".into(),
            from_agent: "claude-code".into(),
            to_agent: Some("cursor".into()),
            active_files: vec!["crates/toksi/src/icm/store.rs".into()],
            unresolved_issues: vec!["fts syntax check".into()],
            next_steps: vec!["run cargo test".into()],
            context_notes: Some("tested manually".into()),
        };

        // 1. Offer
        let h = store.offer(new_h).expect("offer handoff");
        assert_eq!(h.state, HandoffState::Offered);
        assert_eq!(h.claimed_by, None);

        // 2. Reject claim by wrong agent
        let wrong_claim = store.claim(&h.id, "antigravity");
        assert!(wrong_claim.is_err(), "wrong agent claim must fail");

        // 3. Claim by designated agent
        let claimed = store.claim(&h.id, "cursor").expect("claim handoff");
        assert_eq!(claimed.state, HandoffState::Claimed);
        assert_eq!(claimed.claimed_by.as_deref(), Some("cursor"));

        // 4. Claim-once enforcement: second claim must fail
        let double_claim = store.claim(&h.id, "cursor");
        assert!(double_claim.is_err(), "double claim must fail");

        // 5. Resolve
        let resolved = store.resolve(&h.id, "cursor", Some("all tests pass")).expect("resolve handoff");
        assert_eq!(resolved.state, HandoffState::Resolved);
        assert!(resolved.context_notes.unwrap().contains("[Resolution]: all tests pass"));
    }

    #[test]
    fn test_tiered_disclosure_packer() {
        let entries = vec![
            TieredEntry {
                key: "rule-1".into(),
                title: "Build Rules".into(),
                abstract_text: Some("Always run cargo check before push.".into()),
                summary_text: Some("Build rule requires clean cargo check with zero warnings and all tests passing.".into()),
                full_content: "Long documentation about cargo check and rustc flags and lint levels across all workspace members.".into(),
            },
            TieredEntry {
                key: "rule-2".into(),
                title: "Safety Rules".into(),
                abstract_text: Some("Never bypass authentication checks.".into()),
                summary_text: Some("Safety checks require all API tokens and secret strings to be redacted prior to logging.".into()),
                full_content: "Comprehensive guidelines on security boundaries, permission elevation, and secret scanning.".into(),
            },
        ];

        // L0 packing (tight budget)
        let l0 = TieredPacker::pack_tier(&entries, DisclosureTier::L0, 100);
        assert_eq!(l0.tier, DisclosureTier::L0);
        assert!(l0.content.contains("### Context Index (L0)"));
        assert!(l0.content.contains("- **Build Rules**: Always run cargo check before push."));
        assert_eq!(l0.entries_included, 2);

        // L1 packing (moderate budget)
        let l1 = TieredPacker::pack_tier(&entries, DisclosureTier::L1, 400);
        assert_eq!(l1.tier, DisclosureTier::L1);
        assert!(l1.content.contains("### Standing Knowledge & Rules (L1)"));
        assert!(l1.content.contains("#### Build Rules"));
        assert!(l1.content.contains("Build rule requires clean cargo check"));
        assert_eq!(l1.entries_included, 2);

        // L2 packing (high budget)
        let l2 = TieredPacker::pack_tier(&entries, DisclosureTier::L2, 1000);
        assert_eq!(l2.tier, DisclosureTier::L2);
        assert!(l2.content.contains("### Detailed Knowledge (L2)"));
        assert!(l2.content.contains("Long documentation about cargo check and rustc flags"));
        assert_eq!(l2.entries_included, 2);
    }

    #[test]
    fn test_multi_agent_handoff_across_fleet_and_aliases() {
        let dir = tempdir().expect("tempdir");
        let store = HandoffStore::new(dir.path());

        // Verify all 17 agents are recognized
        for agent in SUPPORTED_AGENTS {
            assert!(is_supported_agent(agent));
        }

        // Test alias normalization
        assert_eq!(normalize_agent_id("claude-code"), "claude");
        assert_eq!(normalize_agent_id("opencode2"), "opencode");
        assert_eq!(normalize_agent_id("agy"), "antigravity");
        assert_eq!(normalize_agent_id("github-copilot"), "copilot");
        assert_eq!(normalize_agent_id("factory-droid"), "droid");

        // Offer from claude-code to agy (antigravity)
        let h1 = store.offer(NewHandoff {
            title: "Phase 4 implementation".into(),
            task: "Integrate ICM SQLite and ai-memory handoffs".into(),
            from_agent: "claude-code".into(),
            to_agent: Some("agy".into()),
            active_files: vec!["crates/toksi/src/icm/mod.rs".into()],
            unresolved_issues: vec![],
            next_steps: vec!["verify tests".into()],
            context_notes: None,
        }).expect("offer h1");

        assert_eq!(h1.from_agent, "claude");
        assert_eq!(h1.to_agent.as_deref(), Some("antigravity"));

        // Check pending for antigravity and for uninvited agent
        let agy_pending = store.pending_for_agent("antigravity").expect("pending");
        assert_eq!(agy_pending.len(), 1);
        assert_eq!(agy_pending[0].id, h1.id);

        let cursor_pending = store.pending_for_agent("cursor").expect("pending");
        assert_eq!(cursor_pending.len(), 0);

        // Claim by antigravity
        let claimed = store.claim(&h1.id, "google-antigravity").expect("claim");
        assert_eq!(claimed.state, HandoffState::Claimed);
        assert_eq!(claimed.claimed_by.as_deref(), Some("antigravity"));

        // Second claim attempt fails
        assert!(store.claim(&h1.id, "antigravity").is_err());
    }

    #[test]
    fn test_wiki_store_markdown_and_tiered_export() {
        let dir = tempdir().expect("tempdir");
        let store = WikiStore::new(dir.path());

        let page = WikiPage {
            slug: "architecture-principles".into(),
            title: "Core Architecture Principles".into(),
            doc_type: Some("Concept".into()),
            abstract_text: Some("Zero LLM token overhead on memory store and recall.".into()),
            summary_text: Some("Toksi adheres strictly to zero-LLM spend by utilizing SQLite FTS5 BM25 and deterministic progressive disclosure.".into()),
            tags: vec!["architecture".into(), "memory".into()],
            body: "# Core Architecture\n\nAll state machines are in-process and single-binary.".into(),
        };

        store.write_page(&page).expect("write page");

        let read = store.read_page("architecture-principles").expect("read").expect("found");
        assert_eq!(read.title, "Core Architecture Principles");
        assert_eq!(read.abstract_text.as_deref(), Some("Zero LLM token overhead on memory store and recall."));

        let tiered = store.to_tiered_entries().expect("to tiered");
        assert_eq!(tiered.len(), 1);
        assert_eq!(tiered[0].key, "architecture-principles");
    }

    #[test]
    fn test_wiki_batch_lint_consolidate_and_normalized_handoff() {
        let dir = tempdir().expect("tempdir");
        let wiki = WikiStore::new(dir.path().join("wiki"));

        let pages = vec![
            WikiPage {
                slug: "p1".into(),
                title: "Same Title".into(),
                doc_type: Some("Concept".into()),
                abstract_text: Some("Valid abstract".into()),
                summary_text: None,
                tags: vec!["tag".into()],
                body: "Non-empty body".into(),
            },
            WikiPage {
                slug: "p2".into(),
                title: "Same Title".into(),
                doc_type: Some("Concept".into()),
                abstract_text: None,
                summary_text: None,
                tags: vec![],
                body: "".into(),
            },
        ];

        assert_eq!(wiki.write_pages_batch(&pages).expect("batch"), 2);
        let issues = wiki.lint_pages().expect("lint");
        assert!(issues.iter().any(|i| i.slug == "p2" && i.severity == "error"));

        assert_eq!(wiki.consolidate_pages().expect("consolidate"), 1);
        assert_eq!(wiki.list_pages().expect("list").len(), 1);

        let handoff = HandoffStore::new(dir.path().join("handoffs"));
        let h = handoff
            .offer(NewHandoff {
                title: "Path norm".into(),
                task: "Check forward slashes".into(),
                from_agent: "claude".into(),
                to_agent: Some("cursor".into()),
                active_files: vec![r"crates\toksi\src\aimemory\mod.rs".into()],
                unresolved_issues: vec![],
                next_steps: vec![],
                context_notes: None,
            })
            .expect("offer");
        assert_eq!(h.active_files, vec!["crates/toksi/src/aimemory/mod.rs"]);
    }
}

