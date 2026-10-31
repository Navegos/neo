// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/tests/phase4_icm_aimemory_verification.rs

use chrono::Utc;
use serde_json::json;
use tempfile::tempdir;

use tokless::aimemory::{
    is_supported_agent, normalize_agent_id, DisclosureTier, HandoffState, HandoffStore,
    NewHandoff, TieredEntry, TieredPacker, WikiPage, WikiStore, SUPPORTED_AGENTS,
};
use tokless::headroom::adaptive_mem::{
    calculate_headroom, model_context_limit, select_tier_for_headroom,
};
use tokless::icm::mcp::handle_mcp_request;
use tokless::icm::recall::{recall_memories, RecallOptions};
use tokless::icm::schema::{Importance, Memory};
use tokless::icm::store::{IcmStore, DEFAULT_DB_FILENAME};

#[test]
fn test_phase4_icm_sqlite_fts5_bm25_and_sub_10ms_latency() {
    let tmp = tempdir().expect("tempdir");
    let db_path = tmp.path().join(".tokless").join(DEFAULT_DB_FILENAME);

    // 1. Embed single-file SQLite database (.tokless/memory.db)
    let mut store = IcmStore::open(&db_path).expect("open icm db");
    assert!(db_path.exists(), "database file must exist at {}", db_path.display());

    let now = Utc::now().to_rfc3339();

    // 2. Insert memories
    let mem1 = Memory {
        id: "mem_fts_01".into(),
        created_at: now.clone(),
        updated_at: now.clone(),
        last_accessed: now.clone(),
        access_count: 0,
        weight: 1.5,
        topic: "fts5_indexing".into(),
        summary: "SQLite FTS5 provides ultra-fast full-text indexing with BM25 ranking".into(),
        raw_excerpt: Some("bm25 ranking formula evaluates term frequency and inverse document frequency".into()),
        keywords: vec!["sqlite".into(), "fts5".into(), "bm25".into()],
        importance: Importance::High,
        source_type: "unit_test".into(),
        source_data: None,
        related_ids: vec![],
        summary_hash: None,
    };
    store.insert_or_update_memory(&mem1).expect("insert mem1");

    let mem2 = Memory {
        id: "mem_fts_02".into(),
        created_at: now.clone(),
        updated_at: now.clone(),
        last_accessed: now.clone(),
        access_count: 0,
        weight: 1.0,
        topic: "concurrency".into(),
        summary: "SQLite WAL mode allows multiple concurrent readers with one writer".into(),
        raw_excerpt: Some("PRAGMA journal_mode = WAL".into()),
        keywords: vec!["sqlite".into(), "wal".into(), "concurrency".into()],
        importance: Importance::Medium,
        source_type: "unit_test".into(),
        source_data: None,
        related_ids: vec![],
        summary_hash: None,
    };
    store.insert_or_update_memory(&mem2).expect("insert mem2");

    // 3. FTS5 BM25 search
    let hits = store.search_bm25("BM25 ranking formula", 5).expect("bm25 search");
    assert!(!hits.is_empty(), "hits must not be empty");
    assert_eq!(hits[0].memory.id, "mem_fts_01");

    // 4. Test latency and zero LLM spend across 150 items
    for i in 0..150 {
        let mem = Memory {
            id: format!("mem_bulk_{:03}", i),
            created_at: now.clone(),
            updated_at: now.clone(),
            last_accessed: now.clone(),
            access_count: 0,
            weight: 1.0,
            topic: if i % 2 == 0 { "database".into() } else { "networking".into() },
            summary: format!("High performance SQLite index test item {} with BM25 retrieval", i),
            raw_excerpt: Some(format!("raw trace log snippet for test {}", i)),
            keywords: vec!["sqlite".into(), "benchmark".into()],
            importance: Importance::Medium,
            source_type: "benchmark".into(),
            source_data: None,
            related_ids: vec![],
            summary_hash: None,
        };
        store.insert_or_update_memory(&mem).expect("insert bulk");
    }

    let opts = RecallOptions {
        query: "SQLite index BM25 retrieval".into(),
        limit: 10,
        topic: Some("database".into()),
        min_weight: None,
        max_tokens: Some(500),
    };

    let recall_res = recall_memories(&mut store, &opts).expect("recall memories");
    assert!(!recall_res.hits.is_empty());
    // Sub-10ms latency requirement
    assert!(
        recall_res.latency_ms < 10.0,
        "ICM recall latency must be < 10ms, got {}ms",
        recall_res.latency_ms
    );

    // 5. Atomic fact slot supersession
    let fact_id = store.set_fact("postgres", "pool_size", "20", "config.rs").expect("set fact");
    assert!(!fact_id.is_empty());
    let fact = store.get_fact("postgres", "pool_size").expect("get fact").expect("found");
    assert_eq!(fact.value, "20");

    // Supersede fact
    store.set_fact("postgres", "pool_size", "50", "tuning.rs").expect("supersede fact");
    let superseded = store.get_fact("postgres", "pool_size").expect("get fact").expect("found");
    assert_eq!(superseded.value, "50");

    let all_facts = store.list_facts(Some("postgres")).expect("list facts");
    assert_eq!(all_facts.len(), 1, "only active non-superseded facts returned");
}

#[test]
fn test_phase4_claim_once_multi_agent_handoff_across_17_agents() {
    let tmp = tempdir().expect("tempdir");
    let store = HandoffStore::new(tmp.path());

    // 1. Verify 17+ agent fleet recognition & alias normalization
    assert!(SUPPORTED_AGENTS.len() >= 17);
    for agent in SUPPORTED_AGENTS {
        assert!(is_supported_agent(agent), "agent {} must be recognized", agent);
    }
    assert_eq!(normalize_agent_id("claude-code"), "claude");
    assert_eq!(normalize_agent_id("opencode2"), "opencode");
    assert_eq!(normalize_agent_id("factory-droid"), "droid");
    assert_eq!(normalize_agent_id("github-copilot"), "copilot");
    assert_eq!(normalize_agent_id("agy"), "antigravity");
    assert_eq!(normalize_agent_id("kiro-cli"), "kilo");
    assert_eq!(normalize_agent_id("command-code"), "codex");

    // 2. Create handoff
    let new_h = NewHandoff {
        title: "Cross-agent session baton".into(),
        task: "Implement SQLite FTS5 migration in tokless".into(),
        from_agent: "claude".into(),
        to_agent: Some("cursor".into()),
        active_files: vec!["crates/tokless/src/icm/store.rs".into()],
        unresolved_issues: vec!["FTS5 trigger sync".into()],
        next_steps: vec!["Run cargo test".into()],
        context_notes: Some("Initial schema applied".into()),
    };

    let offered = store.offer(new_h).expect("offer handoff");
    assert_eq!(offered.state, HandoffState::Offered);
    assert_eq!(offered.claimed_by, None);

    // 3. Reject claim by wrong agent
    let wrong_claim = store.claim(&offered.id, "antigravity");
    assert!(wrong_claim.is_err(), "claim by non-designated agent must fail");

    // 4. Claim by designated agent
    let claimed = store.claim(&offered.id, "cursor").expect("claim handoff");
    assert_eq!(claimed.state, HandoffState::Claimed);
    assert_eq!(claimed.claimed_by.as_deref(), Some("cursor"));

    // 5. Claim-once enforcement: second claim must fail
    let double_claim = store.claim(&offered.id, "cursor");
    assert!(double_claim.is_err(), "double claim must fail");

    // 6. Resolve handoff
    let resolved = store
        .resolve(&offered.id, "cursor", Some("Triggers verified and unit tests pass"))
        .expect("resolve handoff");
    assert_eq!(resolved.state, HandoffState::Resolved);
    assert!(resolved.context_notes.unwrap().contains("[Resolution]: Triggers verified"));

    // 7. Verify resolved handoff cannot be claimed or resolved again
    assert!(store.claim(&offered.id, "cursor").is_err());
    assert!(store.resolve(&offered.id, "cursor", None).is_err());
}

#[test]
fn test_phase4_tiered_progressive_disclosure_packer() {
    let entries = vec![
        TieredEntry {
            key: "rule-principles".into(),
            title: "Karpathy Principles".into(),
            abstract_text: Some("Think, simplify, surgical edits, verify.".into()),
            summary_text: Some("Core engineering discipline: minimize abstractions, verify with runnable checks.".into()),
            full_content: "Detailed markdown document describing first principles engineering and surgical diffs.".into(),
        },
        TieredEntry {
            key: "rule-caveman".into(),
            title: "Caveman Prose Style".into(),
            abstract_text: Some("Terse style, ASD-STE100 technical precision.".into()),
            summary_text: Some("Drop fluff, retain technical accuracy, standard acronyms, zero unnecessary tokens.".into()),
            full_content: "Comprehensive compression guidelines for LLM agent prompt responses.".into(),
        },
        TieredEntry {
            key: "rule-ponytail".into(),
            title: "Ponytail Build Discipline".into(),
            abstract_text: Some("Lazy senior developer ladder: reuse first.".into()),
            summary_text: Some("Check existing utils, stdlib, native features, and installed dependencies before adding code.".into()),
            full_content: "Rigorous 7-rung ladder preventing speculative scaffolding and code bloat.".into(),
        },
    ];

    // L0: Context Index (TOC) with tight budget (~100 tokens)
    let l0 = TieredPacker::pack_tier(&entries, DisclosureTier::L0, 100);
    assert_eq!(l0.tier, DisclosureTier::L0);
    assert!(l0.content.contains("### Context Index (L0)"));
    assert!(l0.content.contains("Karpathy Principles"));
    assert!(l0.content.contains("Caveman Prose Style"));
    assert!(l0.content.contains("Ponytail Build Discipline"));
    assert_eq!(l0.entries_included, 3);

    // L1: Standing Knowledge & Rules (~400 tokens)
    let l1 = TieredPacker::pack_tier(&entries, DisclosureTier::L1, 400);
    assert_eq!(l1.tier, DisclosureTier::L1);
    assert!(l1.content.contains("### Standing Knowledge & Rules (L1)"));
    assert!(l1.content.contains("#### Karpathy Principles"));
    assert!(l1.content.contains("Core engineering discipline"));
    assert_eq!(l1.entries_included, 3);

    // L2: Full Knowledge Bodies (~1500 tokens)
    let l2 = TieredPacker::pack_tier(&entries, DisclosureTier::L2, 1500);
    assert_eq!(l2.tier, DisclosureTier::L2);
    assert!(l2.content.contains("### Detailed Knowledge (L2)"));
    assert!(l2.content.contains("Detailed markdown document describing first principles"));
    assert_eq!(l2.entries_included, 3);

    // Dynamic budget adaptation
    let adaptive_tight = TieredPacker::pack(&entries, 120);
    assert_eq!(adaptive_tight.tier, DisclosureTier::L0);

    let adaptive_spacious = TieredPacker::pack(&entries, 600);
    assert_eq!(adaptive_spacious.tier, DisclosureTier::L1);
}

#[test]
fn test_phase4_okf_wiki_store() {
    let tmp = tempdir().expect("tempdir");
    let wiki_dir = tmp.path().join("wiki");
    let store = WikiStore::new(&wiki_dir);

    let page = WikiPage {
        slug: "architecture-patterns".into(),
        title: "Architecture Patterns".into(),
        abstract_text: Some("Monolith vs Microservices patterns".into()),
        summary_text: Some("Guidance on module isolation in Rust workspaces.".into()),
        tags: vec!["architecture".into(), "rust".into()],
        body: "# Architecture Patterns\n\nUse workspace crates with clean boundaries.".into(),
    };

    store.write_page(&page).expect("write wiki page");

    let read_back = store.read_page("architecture-patterns").expect("read page").expect("found");
    assert_eq!(read_back.slug, "architecture-patterns");
    assert_eq!(read_back.title, "Architecture Patterns");
    assert_eq!(read_back.tags, vec!["architecture", "rust"]);
    assert!(read_back.body.contains("Use workspace crates"));

    let pages = store.list_pages().expect("list pages");
    assert_eq!(pages.len(), 1);

    let tiered = store.to_tiered_entries().expect("to tiered");
    assert_eq!(tiered.len(), 1);
    assert_eq!(tiered[0].key, "architecture-patterns");
}

#[test]
fn test_phase4_projectmem_coordination_and_headroom_hooks() {
    let _tmp = tempdir().expect("tempdir");
    let mut store = IcmStore::open_in_memory().expect("open store");

    // 1. Sync projectmem events into ICM SQLite
    let mut ev_dec = tokless::projectmem::schema::Event::new(
        "decision",
        "Use single-file SQLite database (.tokless/memory.db) with bundled FTS5 BM25",
    );
    ev_dec.id = "0001".into();

    let mut ev_gotcha = tokless::projectmem::schema::Event::new(
        "note",
        "gotcha: SQLite requires WAL mode for concurrent multi-agent reading",
    );
    ev_gotcha.id = "0002".into();

    let synced = store.sync_from_projectmem(&[ev_dec, ev_gotcha]).expect("sync");
    assert_eq!(synced, 2);

    // Verify FTS5 BM25 retrieval of synced events
    let hits = store.search_bm25("SQLite WAL mode concurrent", 5).expect("bm25");
    assert!(!hits.is_empty());
    assert_eq!(hits[0].memory.topic, "gotcha");

    // 2. Headroom dynamic memory governor hooks
    assert_eq!(model_context_limit("gemini-1.5-pro"), 1_000_000);
    assert_eq!(model_context_limit("claude-3-5-sonnet"), 200_000);
    assert_eq!(model_context_limit("gpt-4o"), 128_000);

    // Headroom ratio calculations
    let full_headroom = calculate_headroom(10_000, 200_000); // 95% free
    assert!(full_headroom > 0.92);
    let (tier_full, cap_full) = select_tier_for_headroom(full_headroom);
    assert_eq!(tier_full, DisclosureTier::L2);
    assert_eq!(cap_full, 1500);

    let spacious_headroom = calculate_headroom(40_000, 200_000); // 80% free (> 70%)
    let (tier_space, cap_space) = select_tier_for_headroom(spacious_headroom);
    assert_eq!(tier_space, DisclosureTier::L1);
    assert_eq!(cap_space, 600);

    let moderate_headroom = calculate_headroom(100_000, 200_000); // 50% free (30%..70%)
    let (tier_mod, cap_mod) = select_tier_for_headroom(moderate_headroom);
    assert_eq!(tier_mod, DisclosureTier::L1);
    assert_eq!(cap_mod, 300);

    let tight_headroom = calculate_headroom(180_000, 200_000); // 10% free (< 30%)
    let (tier_tight, cap_tight) = select_tier_for_headroom(tight_headroom);
    assert_eq!(tier_tight, DisclosureTier::L0);
    assert_eq!(cap_tight, 100);
}

#[test]
fn test_phase4_in_process_mcp_server_handlers() {
    let mut store = IcmStore::open_in_memory().expect("open store");

    // 1. initialize
    let init_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let init_resp = handle_mcp_request(&mut store, &init_req).expect("init resp");
    assert_eq!(init_resp["result"]["serverInfo"]["name"], "tokless-icm");

    // 2. tools/list
    let list_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    let list_resp = handle_mcp_request(&mut store, &list_req).expect("list resp");
    let tools = list_resp["result"]["tools"].as_array().expect("tools array");
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(tool_names.contains(&"icm_memory_store"));
    assert!(tool_names.contains(&"icm_memory_recall"));
    assert!(tool_names.contains(&"icm_set_fact"));
    assert!(tool_names.contains(&"icm_get_fact"));
    assert!(tool_names.contains(&"icm_list_topics"));
    assert!(tool_names.contains(&"memory_handoff_offer"));
    assert!(tool_names.contains(&"memory_handoff_claim"));
    assert!(tool_names.contains(&"memory_handoff_resolve"));
    assert!(tool_names.contains(&"memory_handoff_list"));
    assert!(tool_names.contains(&"memory_wiki_read"));
    assert!(tool_names.contains(&"memory_wiki_write"));
    assert!(tool_names.contains(&"memory_wiki_list"));

    // 3. icm_memory_store
    let store_req = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "icm_memory_store",
            "arguments": {
                "topic": "testing",
                "summary": "In-process MCP server handles memory storage seamlessly",
                "keywords": ["mcp", "tokless", "testing"],
                "importance": "high"
            }
        }
    });
    let store_resp = handle_mcp_request(&mut store, &store_req).expect("store resp");
    let store_text = store_resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(store_text.contains("Memory stored successfully"));

    // 4. icm_memory_recall
    let recall_req = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "icm_memory_recall",
            "arguments": {
                "query": "In-process MCP server"
            }
        }
    });
    let recall_resp = handle_mcp_request(&mut store, &recall_req).expect("recall resp");
    let recall_text = recall_resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(recall_text.contains("\"total\": 1"));

    // 5. icm_set_fact and icm_get_fact
    let set_fact_req = json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "icm_set_fact",
            "arguments": {
                "entity": "redis",
                "key": "port",
                "value": "6379",
                "source": "mcp_test"
            }
        }
    });
    let _ = handle_mcp_request(&mut store, &set_fact_req);

    let get_fact_req = json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "icm_get_fact",
            "arguments": {
                "entity": "redis",
                "key": "port"
            }
        }
    });
    let get_fact_resp = handle_mcp_request(&mut store, &get_fact_req).expect("get fact resp");
    let fact_text = get_fact_resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(fact_text.contains("\"value\": \"6379\""));
}

#[test]
fn test_phase4_cli_integration() {
    // 1. ICM CLI help
    assert_eq!(tokless::icm::run_icm_cli(&["help".into()]), 0);

    // 2. Handoff CLI help
    assert_eq!(tokless::aimemory::run_handoff_cli(&["help".into()]), 0);

    // 3. Projectmem sync CLI
    assert_eq!(tokless::projectmem::run_mem_cli(&["help".into()]), 0);
}
