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
}

