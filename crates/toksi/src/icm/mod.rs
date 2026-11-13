// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/icm/mod.rs
// created: 2026-10-28
// lastModified: 2026-11-13

#![deny(unsafe_code)]

pub mod cli;
pub mod fusion;
pub mod mcp;
pub mod recall;
pub mod schema;
pub mod slash;
pub mod store;
pub mod temporal;
pub mod wire;

pub use store::*;
pub use schema::*;
pub use recall::*;
pub use cli::*;
pub use fusion::*;
pub use mcp::*;
pub use slash::*;
pub use temporal::*;
pub use wire::*;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_icm_store_crud_and_fts_bm25() {
        let mut store = IcmStore::open_in_memory().expect("in memory store open");

        let now = Utc::now().to_rfc3339();
        let mem1 = Memory {
            id: "mem_1".into(),
            created_at: now.clone(),
            updated_at: now.clone(),
            last_accessed: now.clone(),
            access_count: 0,
            weight: 1.0,
            topic: "database".into(),
            summary: "SQLite FTS5 supports BM25 ranking for fast text search".into(),
            raw_excerpt: Some("bm25 is fast".into()),
            keywords: vec!["sqlite".into(), "fts5".into(), "bm25".into()],
            importance: Importance::High,
            source_type: "manual".into(),
            source_data: None,
            related_ids: vec![],
            summary_hash: None,
            subproject_id: None,
        };

        store.insert_or_update_memory(&mem1).expect("insert mem1");

        let mem2 = Memory {
            id: "mem_2".into(),
            created_at: now.clone(),
            updated_at: now.clone(),
            last_accessed: now,
            access_count: 0,
            weight: 1.0,
            topic: "agents".into(),
            summary: "Multi-agent coordination requires atomic baton handoff protocol".into(),
            raw_excerpt: None,
            keywords: vec!["handoff".into(), "agents".into()],
            importance: Importance::Critical,
            source_type: "manual".into(),
            source_data: None,
            related_ids: vec![],
            summary_hash: None,
            subproject_id: None,
        };

        store.insert_or_update_memory(&mem2).expect("insert mem2");

        // Test list_topics
        let topics = store.list_topics().expect("list topics");
        assert_eq!(topics.len(), 2);

        // Test search_bm25
        let hits = store.search_bm25("SQLite ranking", 5).expect("search bm25");
        assert!(!hits.is_empty());
        assert_eq!(hits[0].memory.id, "mem_1");

        // Test recall_memories
        let opts = RecallOptions {
            query: "baton handoff".into(),
            limit: 5,
            topic: None,
            min_weight: None,
            max_tokens: Some(100),
        };
        let recall_res = recall_memories(&mut store, &opts).expect("recall");
        assert_eq!(recall_res.total, 1);
        assert_eq!(recall_res.hits[0].memory.id, "mem_2");

        // Test facts
        let fact_id = store.set_fact("postgres", "port", "5432", "test").expect("set fact");
        assert!(!fact_id.is_empty());

        let fact = store.get_fact("postgres", "port").expect("get fact").expect("found");
        assert_eq!(fact.value, "5432");

        // Supersede fact
        store.set_fact("postgres", "port", "5433", "test2").expect("update fact");
        let fact_updated = store.get_fact("postgres", "port").expect("get fact").expect("found");
        assert_eq!(fact_updated.value, "5433");

        let all_facts = store.list_facts(None).expect("list facts");
        assert_eq!(all_facts.len(), 1);
    }

    #[test]
    fn test_icm_fts5_query_sanitization_special_chars() {
        let mut store = IcmStore::open_in_memory().expect("in memory store open");
        let now = Utc::now().to_rfc3339();

        let mem = Memory {
            id: "mem_special".into(),
            created_at: now.clone(),
            updated_at: now.clone(),
            last_accessed: now,
            access_count: 0,
            weight: 1.0,
            topic: "syntax".into(),
            summary: "Error resolving foo:bar* in compiler parser".into(),
            raw_excerpt: None,
            keywords: vec!["foo".into(), "bar".into()],
            importance: Importance::Medium,
            source_type: "test".into(),
            source_data: None,
            related_ids: vec![],
            summary_hash: None,
            subproject_id: None,
        };
        store.insert_or_update_memory(&mem).expect("insert");

        // Query containing FTS5 control characters that would crash raw MATCH
        let hits = store.search_bm25("foo:bar* AND NOT (syntax^)", 5).expect("sanitized search");
        assert!(!hits.is_empty());
        assert_eq!(hits[0].memory.id, "mem_special");
    }

    #[test]
    fn test_icm_temporal_decay_and_pinning() {
        let mut store = IcmStore::open_in_memory().expect("in memory store open");
        let now = Utc::now().to_rfc3339();

        let mem_high = Memory {
            id: "mem_high".into(),
            created_at: now.clone(),
            updated_at: now.clone(),
            last_accessed: now.clone(),
            access_count: 0,
            weight: 1.0,
            topic: "general".into(),
            summary: "High importance note".into(),
            raw_excerpt: None,
            keywords: vec!["note".into()],
            importance: Importance::High,
            source_type: "test".into(),
            source_data: None,
            related_ids: vec![],
            summary_hash: None,
            subproject_id: None,
        };
        store.insert_or_update_memory(&mem_high).expect("insert");

        let mem_crit = Memory {
            id: "mem_crit".into(),
            created_at: now.clone(),
            updated_at: now.clone(),
            last_accessed: now,
            access_count: 0,
            weight: 1.0,
            topic: "general".into(),
            summary: "Critical pinned note".into(),
            raw_excerpt: None,
            keywords: vec!["pinned".into()],
            importance: Importance::Critical,
            source_type: "test".into(),
            source_data: None,
            related_ids: vec![],
            summary_hash: None,
            subproject_id: None,
        };
        store.insert_or_update_memory(&mem_crit).expect("insert");

        // Apply decay
        store.apply_decay(0.8).expect("apply decay");

        let after_high = store.get_memory("mem_high").expect("get").expect("found");
        let after_crit = store.get_memory("mem_crit").expect("get").expect("found");

        assert!(after_high.weight < 1.0, "High importance memory decayed");
        assert_eq!(after_crit.weight, 1.0, "Critical memory is immune to decay");

        // Pin high memory -> becomes critical and resets weight to 1.0
        let pinned = store.pin_memory("mem_high").expect("pin");
        assert_eq!(pinned.importance, Importance::Critical);
        assert_eq!(pinned.weight, 1.0);
    }

    #[test]
    fn test_icm_memoirs_and_feedback() {
        let mut store = IcmStore::open_in_memory().expect("in memory store open");

        // Memoir & concept
        let memoir = store.create_memoir("rust_compiler", "Rust AST and borrow checker domain", 50).expect("memoir");
        let concept = store.add_concept(
            &memoir.id,
            "BorrowChecker",
            "Validates reference lifetimes at compile time",
            &["lifetimes".into(), "aliasing".into()],
            0.9,
            &[],
        ).expect("concept");

        assert_eq!(concept.name, "BorrowChecker");

        let found = store.search_concepts(&memoir.id, "lifetimes", 5).expect("search");
        assert_eq!(found.len(), 1);

        // Feedback
        let fb = store.record_feedback(
            "cli",
            "cargo build",
            "cargo build --release",
            "cargo test --all",
            Some("Testing workflow requires test suite"),
            "user",
        ).expect("feedback");

        let fb_hits = store.search_feedback("cargo", None, 5).expect("search fb");
        assert_eq!(fb_hits.len(), 1);
        assert_eq!(fb_hits[0].id, fb.id);
    }

    #[test]
    fn test_icm_sub_10ms_latency_and_zero_llm_spend() {
        let mut store = IcmStore::open_in_memory().expect("in memory store open");
        let now = Utc::now().to_rfc3339();

        for i in 0..100 {
            let mem = Memory {
                id: format!("mem_{}", i),
                created_at: now.clone(),
                updated_at: now.clone(),
                last_accessed: now.clone(),
                access_count: 0,
                weight: 1.0,
                topic: if i % 2 == 0 { "database".into() } else { "compiler".into() },
                summary: format!("Memory item {} describing optimization techniques in rusqlite FTS5 BM25 index", i),
                raw_excerpt: Some(format!("raw snippet data for test item {}", i)),
                keywords: vec!["rusqlite".into(), "fts5".into(), "bm25".into()],
                importance: Importance::Medium,
                source_type: "benchmark".into(),
                source_data: None,
                related_ids: vec![],
                summary_hash: None,
                subproject_id: None,
            };
            store.insert_or_update_memory(&mem).expect("insert mem");
        }

        let opts = RecallOptions {
            query: "rusqlite optimization techniques".into(),
            limit: 10,
            topic: Some("database".into()),
            min_weight: None,
            max_tokens: Some(500),
        };

        let res = recall_memories(&mut store, &opts).expect("recall");
        assert!(!res.hits.is_empty());
        assert!(res.latency_ms < 10.0, "latency was {}ms, expected < 10ms", res.latency_ms);
    }
}
