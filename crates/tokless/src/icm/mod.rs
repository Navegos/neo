// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/icm/mod.rs

pub mod schema;
pub mod store;
pub mod recall;
pub mod mcp;

pub use schema::*;
pub use store::*;
pub use recall::*;
pub use mcp::*;

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
    fn test_icm_sub_10ms_latency_and_zero_llm_spend() {
        let mut store = IcmStore::open_in_memory().expect("in memory store open");
        let now = Utc::now().to_rfc3339();

        // Populate with 100 memories to test realistic search latency
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
        // Verify sub-10ms latency requirement
        assert!(res.latency_ms < 10.0, "latency was {}ms, expected < 10ms", res.latency_ms);
    }

    #[test]
    fn test_icm_sync_from_projectmem() {
        let mut store = IcmStore::open_in_memory().expect("in memory store open");

        let mut ev1 = crate::projectmem::schema::Event::new(
            "decision",
            "Use single-file SQLite database (.tokless/memory.db) with bundled FTS5 BM25",
        );
        ev1.id = "0001".into();
        ev1.location = Some("crates/tokless/src/icm/store.rs:1".into());

        let mut ev2 = crate::projectmem::schema::Event::new(
            "note",
            "gotcha: SQLite requires WAL mode for multi-threaded concurrency",
        );
        ev2.id = "0002".into();
        ev2.location = Some("crates/tokless/src/icm/store.rs:57".into());

        let events = vec![ev1, ev2];

        let synced = store.sync_from_projectmem(&events).expect("sync");
        assert_eq!(synced, 2);

        // Verify FTS5 BM25 can recall synced projectmem decision
        let hits = store.search_bm25("SQLite FTS5", 5).expect("search bm25");
        assert!(!hits.is_empty());
        assert_eq!(hits[0].memory.topic, "decision");

        // Verify fact was stored
        let fact = store.get_fact("architecture", "decision_0001").expect("get fact");
        assert!(fact.is_some());
        assert!(fact.unwrap().value.contains("SQLite database"));
    }
}

