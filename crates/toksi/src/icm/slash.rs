// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/icm/slash.rs
// created: 2026-11-13

#![deny(unsafe_code)]

use std::path::Path;
use crate::icm::recall::{recall_memories, RecallOptions};
use crate::icm::store::{resolve_icm_db_path_from, IcmStore};

pub fn handle_slash_command(input: &str, root: Option<&Path>) -> String {
    let trimmed = input.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    if parts.is_empty() || parts[0] != "/icm" {
        return "Unknown command. Use `/icm help` for available options.".to_string();
    }

    let sub = parts.get(1).copied().unwrap_or("help");

    let db_path = match resolve_icm_db_path_from(root) {
        Ok(p) => p,
        Err(e) => return format!("Error locating ICM store: {e}"),
    };

    let mut store = match IcmStore::open(&db_path) {
        Ok(s) => s,
        Err(e) => return format!("Error opening ICM database: {e}"),
    };

    match sub {
        "help" => {
            "**ICM Slash Commands:**\n\
             - `/icm status` - Show ICM database location and global statistics\n\
             - `/icm recall <query>` - Hybrid FTS5 BM25 and temporal recall\n\
             - `/icm pin <id>` - Pin a memory entry (makes it critical and decay-immune)\n\
             - `/icm decay [factor]` - Trigger temporal decay on non-critical memories\n\
             - `/icm topics` - List all active knowledge topics and average weights\n\
             - `/icm facts [entity]` - List active facts\n\
             - `/icm stats` - Show detailed database metrics\n\
             - `/icm wake_up` - Generate agent standing context brief\n\
             - `/icm help` - Show this help message"
                .to_string()
        }
        "status" | "stats" => {
            match store.stats() {
                Ok(s) => format!(
                    "**ICM Status & Telemetry:**\n\
                     - Location: `{}`\n\
                     - Total Memories: {}\n\
                     - Topics: {}\n\
                     - Active Facts: {}\n\
                     - Memoirs: {}\n\
                     - Concepts: {}\n\
                     - Feedback Entries: {}\n\
                     - Transcript Messages: {}\n\
                     - Avg Weight: {:.2}",
                    db_path.display(),
                    s.total_memories,
                    s.total_topics,
                    s.total_facts,
                    s.total_memoirs,
                    s.total_concepts,
                    s.total_feedback,
                    s.total_transcripts,
                    s.avg_weight
                ),
                Err(e) => format!("Error reading stats: {e}"),
            }
        }
        "recall" => {
            let query = parts[2..].join(" ");
            if query.is_empty() {
                return "Error: `/icm recall <query>` requires a search query.".to_string();
            }
            let opts = RecallOptions {
                query,
                limit: 5,
                ..Default::default()
            };
            match recall_memories(&mut store, &opts) {
                Ok(res) => {
                    if res.hits.is_empty() {
                        return format!("No memories matched '{}'.", res.query);
                    }
                    let mut lines = vec![format!("**ICM Recall Results ({} hits, {:.2}ms):**", res.total, res.latency_ms)];
                    for h in &res.hits {
                        lines.push(format!("- [{}] `{}`: {}", h.memory.importance.as_str().to_uppercase(), h.memory.topic, h.memory.summary));
                    }
                    lines.join("\n")
                }
                Err(e) => format!("Error recalling memories: {e}"),
            }
        }
        "pin" => {
            let id = match parts.get(2) {
                Some(id) => *id,
                None => return "Usage: `/icm pin <memory_id>`".to_string(),
            };
            match store.pin_memory(id) {
                Ok(m) => format!("Pinned memory `{}` (topic: {}, importance: critical, weight: 1.0)", m.id, m.topic),
                Err(e) => format!("Error pinning memory: {e}"),
            }
        }
        "decay" => {
            let factor = parts.get(2).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.95);
            match store.apply_decay(factor) {
                Ok(count) => format!("Applied temporal decay ({:.2}) to {} non-critical memories.", factor, count),
                Err(e) => format!("Error applying decay: {e}"),
            }
        }
        "topics" => {
            match store.list_topics() {
                Ok(topics) => {
                    let mut lines = vec![format!("**ICM Topics ({}):**", topics.len())];
                    for t in topics {
                        lines.push(format!("- **{}** (count: {}, avg_weight: {:.2})", t.name, t.count, t.avg_weight));
                    }
                    lines.join("\n")
                }
                Err(e) => format!("Error listing topics: {e}"),
            }
        }
        "facts" => {
            let entity = parts.get(2).copied();
            match store.list_facts(entity) {
                Ok(facts) => {
                    let mut lines = vec![format!("**ICM Active Facts ({}):**", facts.len())];
                    for f in facts {
                        lines.push(format!("- `{}::{}` = `{}` ({})", f.entity, f.key, f.value, f.source));
                    }
                    lines.join("\n")
                }
                Err(e) => format!("Error listing facts: {e}"),
            }
        }
        "wake_up" => {
            match store.build_wake_up_brief(None) {
                Ok(brief) => brief,
                Err(e) => format!("Error generating wake up brief: {e}"),
            }
        }
        _ => "Unknown ICM subcommand. Use `/icm help` for available options.".to_string(),
    }
}
