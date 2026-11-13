// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/icm/cli.rs
// created: 2026-10-31
// lastModified: 2026-11-13

#![deny(unsafe_code)]

use chrono::Utc;
use crate::icm::mcp::run_mcp_server;
use crate::icm::recall::{recall_memories, RecallOptions};
use crate::icm::schema::{Importance, Memory};
use crate::icm::store::{compute_summary_hash, IcmStore};

pub fn print_icm_help() {
    println!(
        "toksi icm — Instant Context Memory (ICM) SQLite FTS5 BM25 knowledge engine\n\n\
        Usage:\n\
          toksi icm recall <query> [--limit <n>] [--topic <topic>] [--max-tokens <n>]\n\
          toksi icm store <summary> --topic <topic> [--importance <level>] [--excerpt <text>] [--keywords <k1,k2>]\n\
          toksi icm pin <id>\n\
          toksi icm forget <id>\n\
          toksi icm decay [--factor <f>]\n\
          toksi icm consolidate <topic> [--threshold <n>]\n\
          toksi icm fact set <entity> <key> <value> [--source <text>]\n\
          toksi icm fact get <entity> <key>\n\
          toksi icm fact list [entity]\n\
          toksi icm topics\n\
          toksi icm stats\n\
          toksi icm wake_up\n\
          toksi icm sync\n\
          toksi icm mcp\n"
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
        "stats" => {
            let store = match IcmStore::open_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error opening ICM database: {}", e);
                    return 1;
                }
            };
            match store.stats() {
                Ok(s) => {
                    println!("=== ICM Global Telemetry ===");
                    println!("  Total Memories:       {}", s.total_memories);
                    println!("  Unique Topics:        {}", s.total_topics);
                    println!("  Active Facts:         {}", s.total_facts);
                    println!("  Memoirs:              {}", s.total_memoirs);
                    println!("  Concepts:             {}", s.total_concepts);
                    println!("  Feedback Entries:     {}", s.total_feedback);
                    println!("  Transcript Messages:  {}", s.total_transcripts);
                    println!("  Average Weight:       {:.2}", s.avg_weight);
                    0
                }
                Err(e) => {
                    eprintln!("Error reading stats: {}", e);
                    1
                }
            }
        }
        "pin" => {
            if rest.is_empty() {
                eprintln!("Usage: toksi icm pin <id>");
                return 1;
            }
            let id = &rest[0];
            let mut store = match IcmStore::open_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error opening ICM database: {}", e);
                    return 1;
                }
            };
            match store.pin_memory(id) {
                Ok(m) => {
                    println!("Pinned memory '{}' under topic '{}' (importance: critical, weight: 1.0)", m.id, m.topic);
                    0
                }
                Err(e) => {
                    eprintln!("Error pinning memory: {}", e);
                    1
                }
            }
        }
        "forget" => {
            if rest.is_empty() {
                eprintln!("Usage: toksi icm forget <id>");
                return 1;
            }
            let id = &rest[0];
            let mut store = match IcmStore::open_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error opening ICM database: {}", e);
                    return 1;
                }
            };
            match store.delete_memory(id) {
                Ok(true) => {
                    println!("Deleted memory '{}'", id);
                    0
                }
                Ok(false) => {
                    println!("Memory '{}' not found", id);
                    0
                }
                Err(e) => {
                    eprintln!("Error deleting memory: {}", e);
                    1
                }
            }
        }
        "decay" => {
            let mut factor = 0.95;
            let mut i = 0;
            while i < rest.len() {
                if rest[i] == "--factor" && i + 1 < rest.len() {
                    factor = rest[i + 1].parse().unwrap_or(0.95);
                    i += 2;
                } else {
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
            match store.apply_decay(factor) {
                Ok(count) => {
                    println!("Applied temporal decay ({:.2}) to {} memories", factor, count);
                    0
                }
                Err(e) => {
                    eprintln!("Error applying decay: {}", e);
                    1
                }
            }
        }
        "consolidate" => {
            if rest.is_empty() {
                eprintln!("Usage: toksi icm consolidate <topic> [--threshold <n>]");
                return 1;
            }
            let topic = &rest[0];
            let mut threshold = 10;
            if rest.len() >= 3 && rest[1] == "--threshold" {
                threshold = rest[2].parse().unwrap_or(10);
            }
            let mut store = match IcmStore::open_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error opening ICM database: {}", e);
                    return 1;
                }
            };
            match store.consolidate_topic(topic, threshold) {
                Ok(pruned) => {
                    println!("Consolidated topic '{}': pruned {} expendable memories (threshold: {})", topic, pruned, threshold);
                    0
                }
                Err(e) => {
                    eprintln!("Error consolidating: {}", e);
                    1
                }
            }
        }
        "wake_up" => {
            let store = match IcmStore::open_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error opening ICM database: {}", e);
                    return 1;
                }
            };
            match store.build_wake_up_brief(None) {
                Ok(brief) => {
                    println!("{}", brief);
                    0
                }
                Err(e) => {
                    eprintln!("Error generating wake up brief: {}", e);
                    1
                }
            }
        }
        "recall" => {
            if rest.is_empty() {
                eprintln!("Error: 'toksi icm recall <query>' requires a search query");
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
                    println!("=== ICM Recall: '{}' ({} hits, {:.2}ms) ===", res.query, res.total, res.latency_ms);
                    for hit in res.hits {
                        println!(
                            "  [{}] ({}) {} (weight: {:.2}, rank: {})",
                            hit.memory.id,
                            hit.memory.importance.as_str().to_uppercase(),
                            hit.memory.topic,
                            hit.memory.weight,
                            hit.rank
                        );
                        println!("      Summary: {}", hit.memory.summary);
                        if let Some(ref ex) = hit.memory.raw_excerpt {
                            println!("      Excerpt: {}", ex);
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
                eprintln!("Usage: toksi icm store <summary> --topic <topic> [--importance <level>]");
                return 1;
            }
            let mut summary = String::new();
            let mut topic = String::new();
            let mut importance = Importance::Medium;
            let mut raw_excerpt = None;
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
                    raw_excerpt = Some(rest[i + 1].clone());
                    i += 2;
                } else if rest[i] == "--keywords" && i + 1 < rest.len() {
                    keywords = rest[i + 1].split(',').map(|s| s.trim().to_string()).collect();
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
                eprintln!("Error: --topic is required for storing memory");
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
            let id = format!("mem_{:x}", rand_u64());
            if keywords.is_empty() {
                keywords.push(topic.clone());
            }

            let mem = Memory {
                id: id.clone(),
                created_at: now.clone(),
                updated_at: now.clone(),
                last_accessed: now,
                access_count: 0,
                weight: if importance == Importance::Critical { 1.0 } else { 0.8 },
                topic: topic.clone(),
                summary: summary.clone(),
                raw_excerpt,
                keywords,
                importance,
                source_type: "cli".into(),
                source_data: None,
                related_ids: Vec::new(),
                summary_hash: Some(compute_summary_hash(&topic, &summary)),
                subproject_id: Some(store.subproject_id().to_string()),
            };

            match store.insert_or_update_memory(&mem) {
                Ok(_) => {
                    println!("Stored memory '{}' under topic '{}'", id, topic);
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
                eprintln!("Usage: toksi icm fact [set|get|list]");
                return 1;
            }
            let sub = rest[0].as_str();
            let fact_args = &rest[1..];
            let mut store = match IcmStore::open_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error opening ICM database: {}", e);
                    return 1;
                }
            };

            match sub {
                "set" => {
                    if fact_args.len() < 3 {
                        eprintln!("Usage: toksi icm fact set <entity> <key> <value> [--source <text>]");
                        return 1;
                    }
                    let entity = &fact_args[0];
                    let key = &fact_args[1];
                    let value = &fact_args[2];
                    let source = if fact_args.len() >= 5 && fact_args[3] == "--source" {
                        &fact_args[4]
                    } else {
                        "cli"
                    };

                    match store.set_fact(entity, key, value, source) {
                        Ok(fid) => {
                            println!("Set fact '{}': {}::{} = {}", fid, entity, key, value);
                            0
                        }
                        Err(e) => {
                            eprintln!("Error setting fact: {}", e);
                            1
                        }
                    }
                }
                "get" => {
                    if fact_args.len() < 2 {
                        eprintln!("Usage: toksi icm fact get <entity> <key>");
                        return 1;
                    }
                    let entity = &fact_args[0];
                    let key = &fact_args[1];
                    match store.get_fact(entity, key) {
                        Ok(Some(f)) => {
                            println!("Fact: {}::{} = {} (source: {}, created: {})", f.entity, f.key, f.value, f.source, f.created_at);
                            0
                        }
                        Ok(None) => {
                            println!("Fact not found for {}::{}", entity, key);
                            0
                        }
                        Err(e) => {
                            eprintln!("Error getting fact: {}", e);
                            1
                        }
                    }
                }
                "list" => {
                    let entity = fact_args.first().map(|s| s.as_str());
                    match store.list_facts(entity) {
                        Ok(facts) => {
                            println!("=== ICM Active Facts ({}) ===", facts.len());
                            for f in facts {
                                println!("  {}::{} = {} (source: {})", f.entity, f.key, f.value, f.source);
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
                    eprintln!("Unknown fact subcommand: {}", sub);
                    1
                }
            }
        }
        "sync" => {
            let mut store = match IcmStore::open_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error opening ICM database: {}", e);
                    return 1;
                }
            };
            let events = match crate::projectmem::store::read_events(None) {
                Ok(evs) => evs,
                Err(e) => {
                    eprintln!("Error reading projectmem events: {}", e);
                    return 1;
                }
            };
            match store.sync_from_projectmem(&events) {
                Ok(count) => {
                    println!("Synchronized {} events from ProjectMem to ICM SQLite FTS5 database", count);
                    0
                }
                Err(e) => {
                    eprintln!("Error syncing to ICM: {}", e);
                    1
                }
            }
        }
        _ => {
            eprintln!("Unknown ICM command: '{}'. Run 'toksi icm help' for usage.", cmd);
            1
        }
    }
}

fn rand_u64() -> u64 {
    use std::time::SystemTime;
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    d.as_nanos() as u64
}
