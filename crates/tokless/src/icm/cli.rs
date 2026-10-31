// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/icm/cli.rs

use chrono::Utc;
use crate::icm::mcp::run_mcp_server;
use crate::icm::recall::{recall_memories, RecallOptions};
use crate::icm::schema::{Importance, Memory};
use crate::icm::store::{compute_summary_hash, IcmStore};

pub fn print_icm_help() {
    println!(
        "tokless icm — Instant Context Memory (ICM) SQLite FTS5 BM25 knowledge engine\n\n\
        Usage:\n\
          tokless icm recall <query> [--limit <n>] [--topic <topic>] [--max-tokens <n>]\n\
          tokless icm store <summary> --topic <topic> [--importance <level>] [--excerpt <text>] [--keywords <k1,k2>]\n\
          tokless icm fact set <entity> <key> <value> [--source <text>]\n\
          tokless icm fact get <entity> <key>\n\
          tokless icm fact list [entity]\n\
          tokless icm topics\n\
          tokless icm sync\n\
          tokless icm mcp\n"
    );
}

pub fn run_icm_cli(args: &[String]) -> i32 {
    if args.is_empty() {
        print_icm_help();
        return 0;
    }

    let cmd = args[0].as_str();
    let rest = &args[1..];

    match cmd {
        "help" | "-h" | "--help" => {
            print_icm_help();
            0
        }
        "mcp" => run_mcp_server(),
        "topics" => {
            let store = match IcmStore::open_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error opening ICM database: {}", e);
                    return 1;
                }
            };
            match store.list_topics() {
                Ok(topics) => {
                    println!("=== ICM Topics ({}) ===", topics.len());
                    for t in topics {
                        println!("  {} (count: {}, avg_weight: {:.2})", t.name, t.count, t.avg_weight);
                    }
                    0
                }
                Err(e) => {
                    eprintln!("Error listing topics: {}", e);
                    1
                }
            }
        }
        "recall" => {
            if rest.is_empty() {
                eprintln!("Error: 'tokless icm recall <query>' requires a search query");
                return 1;
            }
            let mut query = String::new();
            let mut limit = 5;
            let mut topic = None;
            let mut max_tokens = None;

            let mut i = 0;
            while i < rest.len() {
                if rest[i] == "--limit" && i + 1 < rest.len() {
                    limit = rest[i + 1].parse().unwrap_or(5);
                    i += 2;
                } else if rest[i] == "--topic" && i + 1 < rest.len() {
                    topic = Some(rest[i + 1].clone());
                    i += 2;
                } else if rest[i] == "--max-tokens" && i + 1 < rest.len() {
                    max_tokens = rest[i + 1].parse().ok();
                    i += 2;
                } else if query.is_empty() {
                    query = rest[i].clone();
                    i += 1;
                } else {
                    query.push(' ');
                    query.push_str(&rest[i]);
                    i += 1;
                }
            }

            let mut store = match IcmStore::open_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error opening ICM database: {}", e);
                    return 1;
                }
            };

            let opts = RecallOptions {
                query,
                limit,
                topic,
                min_weight: None,
                max_tokens,
            };

            match recall_memories(&mut store, &opts) {
                Ok(res) => {
                    println!("Recalled {} memories ({:.2}ms):", res.hits.len(), res.latency_ms);
                    for hit in res.hits {
                        println!(
                            "  [{}] [{}] {} (score: {:.2})",
                            hit.memory.id,
                            hit.memory.topic,
                            hit.memory.summary,
                            hit.score
                        );
                        if let Some(ref exc) = hit.memory.raw_excerpt {
                            println!("      Excerpt: {}", exc);
                        }
                    }
                    0
                }
                Err(e) => {
                    eprintln!("Error recalling memories: {}", e);
                    1
                }
            }
        }
        "store" => {
            if rest.is_empty() {
                eprintln!("Error: 'tokless icm store <summary> --topic <topic>' requires summary");
                return 1;
            }
            let mut summary = String::new();
            let mut topic = String::new();
            let mut importance = Importance::Medium;
            let mut excerpt = None;
            let mut keywords = Vec::new();

            let mut i = 0;
            while i < rest.len() {
                if rest[i] == "--topic" && i + 1 < rest.len() {
                    topic = rest[i + 1].clone();
                    i += 2;
                } else if rest[i] == "--importance" && i + 1 < rest.len() {
                    importance = rest[i + 1].parse().unwrap_or(Importance::Medium);
                    i += 2;
                } else if rest[i] == "--excerpt" && i + 1 < rest.len() {
                    excerpt = Some(rest[i + 1].clone());
                    i += 2;
                } else if rest[i] == "--keywords" && i + 1 < rest.len() {
                    keywords = rest[i + 1].split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    i += 2;
                } else if summary.is_empty() {
                    summary = rest[i].clone();
                    i += 1;
                } else {
                    summary.push(' ');
                    summary.push_str(&rest[i]);
                    i += 1;
                }
            }

            if topic.is_empty() {
                eprintln!("Error: 'tokless icm store' requires --topic <topic>");
                return 1;
            }

            let mut store = match IcmStore::open_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error opening ICM database: {}", e);
                    return 1;
                }
            };

            let now = Utc::now().to_rfc3339();
            let hash = compute_summary_hash(&topic, &summary);
            let id = format!("mem_{}", &hash[..8]);

            let mem = Memory {
                id: id.clone(),
                created_at: now.clone(),
                updated_at: now.clone(),
                last_accessed: now,
                access_count: 0,
                weight: match importance {
                    Importance::Critical => 2.0,
                    Importance::High => 1.5,
                    Importance::Medium => 1.0,
                    Importance::Low => 0.5,
                },
                topic,
                summary,
                raw_excerpt: excerpt,
                keywords,
                importance,
                source_type: "cli".into(),
                source_data: None,
                related_ids: vec![],
                summary_hash: Some(hash),
            };

            match store.insert_or_update_memory(&mem) {
                Ok(_) => {
                    println!("Stored memory [{}] in topic '{}'", id, mem.topic);
                    0
                }
                Err(e) => {
                    eprintln!("Error storing memory: {}", e);
                    1
                }
            }
        }
        "fact" => {
            if rest.is_empty() {
                eprintln!("Usage: tokless icm fact <set|get|list>");
                return 1;
            }
            let sub = rest[0].as_str();
            let mut store = match IcmStore::open_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error opening ICM database: {}", e);
                    return 1;
                }
            };
            match sub {
                "set" => {
                    if rest.len() < 4 {
                        eprintln!("Usage: tokless icm fact set <entity> <key> <value> [--source <text>]");
                        return 1;
                    }
                    let entity = &rest[1];
                    let key = &rest[2];
                    let value = &rest[3];
                    let mut source = "cli";
                    if rest.len() >= 6 && rest[4] == "--source" {
                        source = &rest[5];
                    }
                    match store.set_fact(entity, key, value, source) {
                        Ok(id) => {
                            println!("Set fact [{}] {}::{} = {}", id, entity, key, value);
                            0
                        }
                        Err(e) => {
                            eprintln!("Error setting fact: {}", e);
                            1
                        }
                    }
                }
                "get" => {
                    if rest.len() < 3 {
                        eprintln!("Usage: tokless icm fact get <entity> <key>");
                        return 1;
                    }
                    let entity = &rest[1];
                    let key = &rest[2];
                    match store.get_fact(entity, key) {
                        Ok(Some(fact)) => {
                            println!("{}::{} = {} (source: {}, id: {})", fact.entity, fact.key, fact.value, fact.source, fact.id);
                            0
                        }
                        Ok(None) => {
                            eprintln!("Fact {}::{} not found", entity, key);
                            1
                        }
                        Err(e) => {
                            eprintln!("Error getting fact: {}", e);
                            1
                        }
                    }
                }
                "list" => {
                    let entity = if rest.len() >= 2 { Some(rest[1].as_str()) } else { None };
                    match store.list_facts(entity) {
                        Ok(facts) => {
                            println!("=== Active Facts ({}) ===", facts.len());
                            for f in facts {
                                println!("  {}::{} = {} ({})", f.entity, f.key, f.value, f.source);
                            }
                            0
                        }
                        Err(e) => {
                            eprintln!("Error listing facts: {}", e);
                            1
                        }
                    }
                }
                _ => {
                    eprintln!("Unknown fact action: {}", sub);
                    1
                }
            }
        }
        "sync" => {
            let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let events = match crate::projectmem::read_events(Some(&root)) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Error reading projectmem events: {}", e);
                    return 1;
                }
            };
            let mut store = match IcmStore::open_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error opening ICM database: {}", e);
                    return 1;
                }
            };
            match store.sync_from_projectmem(&events) {
                Ok(count) => {
                    println!("Synced {} events from projectmem into SQLite ICM", count);
                    0
                }
                Err(e) => {
                    eprintln!("Error syncing to ICM: {}", e);
                    1
                }
            }
        }
        _ => {
            eprintln!("Unknown command 'tokless icm {}'. Run 'tokless icm help' for usage.", cmd);
            1
        }
    }
}

