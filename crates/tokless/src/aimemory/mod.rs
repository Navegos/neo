// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/aimemory/mod.rs

pub mod handoff;
pub mod tiered;
pub mod wiki;

pub use handoff::*;
pub use tiered::*;
pub use wiki::*;

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
            active_files: vec!["crates/tokless/src/icm/store.rs".into()],
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
    }
}

