// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/headroom/adaptive_mem.rs

use std::path::Path;
use serde_json::{json, Value};

use crate::aimemory::tiered::{DisclosureTier, TieredEntry, TieredPacker};
use crate::headroom::cache_align::estimate_tokens;

pub const DEFAULT_MAX_CONTEXT_TOKENS: usize = 200_000;

/// Returns the nominal context window size for a given model identifier
pub fn model_context_limit(model: &str) -> usize {
    let lower = model.to_lowercase();
    if lower.contains("gemini") {
        1_000_000
    } else if lower.contains("claude-3") || lower.contains("claude-sonnet") || lower.contains("claude-opus") {
        200_000
    } else if lower.contains("gpt-4o") || lower.contains("o1") || lower.contains("o3") {
        128_000
    } else if lower.contains("gpt-4-32k") {
        32_768
    } else if lower.contains("gpt-4") {
        8_192
    } else {
        128_000
    }
}

/// Computes the ratio of free context tokens (0.0 = full/exhausted, 1.0 = empty)
pub fn calculate_headroom(current_tokens: usize, max_context: usize) -> f32 {
    if max_context == 0 {
        return 0.0;
    }
    let free = max_context.saturating_sub(current_tokens);
    free as f32 / max_context as f32
}

/// Scales memory disclosure tier strictly based on headroom percentage:
/// - Context Free > 92%: L2 full detail & raw history (~1500 tokens)
/// - Context Free 70%..=92%: L1 full brief + active handoffs (~400-600 tokens)
/// - Context Free 30%..=70%: L1 compressed (~200-300 tokens)
/// - Context Free < 30%: L0 titles only (~50-100 tokens), reserving context for code generation
pub fn select_tier_for_headroom(headroom_ratio: f32) -> (DisclosureTier, usize) {
    if headroom_ratio > 0.92 {
        (DisclosureTier::L2, 1500)
    } else if headroom_ratio > 0.70 {
        (DisclosureTier::L1, 600)
    } else if headroom_ratio >= 0.30 {
        (DisclosureTier::L1, 300)
    } else {
        (DisclosureTier::L0, 100)
    }
}

/// Collects available project memory entries from projectmem and aimemory
pub fn collect_memory_entries() -> Vec<TieredEntry> {
    let mut entries = Vec::new();

    // 1. Check standing gotchas and decisions in .projectmem/events.jsonl
    let events_path = Path::new(".projectmem").join("events.jsonl");
    if events_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&events_path) {
            let mut gotcha_count = 0;
            let mut decision_count = 0;

            for line in content.lines().rev() {
                if let Ok(val) = serde_json::from_str::<Value>(line) {
                    let ev_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    let summary = val.get("summary").and_then(|v| v.as_str()).unwrap_or("");
                    if ev_type == "note" && (summary.starts_with("gotcha:") || summary.starts_with("Gotcha:")) {
                        if gotcha_count < 5 {
                            entries.push(TieredEntry {
                                key: format!("gotcha_{}", gotcha_count),
                                title: "Standing Gotcha".to_string(),
                                abstract_text: Some(summary.to_string()),
                                summary_text: Some(summary.to_string()),
                                full_content: summary.to_string(),
                            });
                            gotcha_count += 1;
                        }
                    } else if ev_type == "decision" && decision_count < 5 {
                        entries.push(TieredEntry {
                            key: format!("decision_{}", decision_count),
                            title: "Architecture Decision".to_string(),
                            abstract_text: Some(summary.to_string()),
                            summary_text: Some(summary.to_string()),
                            full_content: summary.to_string(),
                        });
                        decision_count += 1;
                    }
                }
            }
        }
    }

    // 2. Check summary.md if present
    let summary_file = Path::new(".projectmem").join("summary.md");
    if summary_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&summary_file) {
            let first_para = content.lines().take(6).collect::<Vec<&str>>().join("\n");
            entries.push(TieredEntry {
                key: "project_summary".to_string(),
                title: "Project Purpose & Architecture".to_string(),
                abstract_text: Some(first_para.clone()),
                summary_text: Some(first_para.clone()),
                full_content: content,
            });
        }
    }

    // 3. Check active multi-agent handoffs from aimemory
    let handoffs_dir = crate::aimemory::default_handoffs_dir();
    let handoff_store = crate::aimemory::HandoffStore::new(&handoffs_dir);
    if let Ok(active_handoffs) = handoff_store.list_active() {
        for h in active_handoffs.iter().take(3) {
            let desc = format!("Handoff from {} to {}: {}", h.from_agent, h.to_agent.as_deref().unwrap_or("any"), h.task);
            let full = format!(
                "Task: {}\nFrom: {}\nTo: {}\nState: {}\nFiles: {}\nUnresolved: {}\nNext Steps: {}\nNotes: {}",
                h.task,
                h.from_agent,
                h.to_agent.as_deref().unwrap_or("any"),
                h.state.as_str(),
                h.active_files.join(", "),
                h.unresolved_issues.join(", "),
                h.next_steps.join(", "),
                h.context_notes.as_deref().unwrap_or("None")
            );
            entries.push(TieredEntry {
                key: format!("handoff_{}", h.id),
                title: format!("Active Handoff: {}", h.title),
                abstract_text: Some(desc.clone()),
                summary_text: Some(desc),
                full_content: full,
            });
        }
    }

    // 4. Check facts and top memories from ICM SQLite store (.tokless/memory.db)
    let icm_db_path = crate::icm::resolve_icm_db_path();
    if icm_db_path.exists() {
        if let Ok(icm_store) = crate::icm::IcmStore::open(&icm_db_path) {
            if let Ok(facts) = icm_store.list_facts(None) {
                for f in facts.iter().take(5) {
                    let desc = format!("{}::{} = {}", f.entity, f.key, f.value);
                    entries.push(TieredEntry {
                        key: format!("fact_{}", f.id),
                        title: format!("Fact: {}::{}", f.entity, f.key),
                        abstract_text: Some(desc.clone()),
                        summary_text: Some(desc.clone()),
                        full_content: desc,
                    });
                }
            }
            if let Ok(hits) = icm_store.get_recent(5) {
                for hit in hits {
                    if hit.memory.importance == crate::icm::Importance::Critical || hit.memory.importance == crate::icm::Importance::High {
                        entries.push(TieredEntry {
                            key: format!("icm_{}", hit.memory.id),
                            title: format!("Memory ({}): {}", hit.memory.topic, hit.memory.importance.as_str()),
                            abstract_text: Some(hit.memory.summary.clone()),
                            summary_text: Some(hit.memory.summary.clone()),
                            full_content: hit.memory.raw_excerpt.unwrap_or(hit.memory.summary),
                        });
                    }
                }
            }
        }
    }

    // 5. Check wiki pages from aimemory
    let wiki_dir = crate::aimemory::default_wiki_dir();
    let wiki_store = crate::aimemory::WikiStore::new(&wiki_dir);
    if let Ok(wiki_entries) = wiki_store.to_tiered_entries() {
        for w in wiki_entries.into_iter().take(3) {
            entries.push(w);
        }
    }

    // 6. Fallback dummy entry if project has no memories yet
    if entries.is_empty() {
        entries.push(TieredEntry {
            key: "tokless_governor".to_string(),
            title: "Tokless Native Memory Governor".to_string(),
            abstract_text: Some("Zero token waste active. Progressive disclosure tier in effect.".to_string()),
            summary_text: Some("Tokless dynamic memory governor manages token budget across turns.".to_string()),
            full_content: "Tokless dynamic memory governor manages token budget across turns without LLM summarization overhead.".to_string(),
        });
    }

    entries
}

/// Builds the formatted dynamic memory brief for prompt injection
pub fn build_adaptive_memory_brief(current_tokens: usize, max_context: usize) -> (String, DisclosureTier) {
    let headroom = calculate_headroom(current_tokens, max_context);
    let (tier, budget) = select_tier_for_headroom(headroom);

    let entries = collect_memory_entries();
    let packed = TieredPacker::pack_tier(&entries, tier, budget);

    let mut out = String::new();
    out.push_str("<!-- tokless:adaptive-mem-governor tier=");
    out.push_str(match tier {
        DisclosureTier::L0 => "L0",
        DisclosureTier::L1 => "L1",
        DisclosureTier::L2 => "L2",
    });
    out.push_str(&format!(" headroom={:.1}% -->\n", headroom * 100.0));
    out.push_str(&packed.content);
    out.push('\n');

    (out, tier)
}

/// Injects or updates the dynamic memory context in Anthropic or OpenAI request bodies
pub fn inject_adaptive_memory(body: &mut Value) -> bool {
    let raw_str = body.to_string();
    let current_tokens = estimate_tokens(&raw_str);

    let Value::Object(map) = body else {
        return false;
    };

    let model_str = map.get("model").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let max_context = model_context_limit(&model_str);

    let (memory_brief, _) = build_adaptive_memory_brief(current_tokens, max_context);

    // Anthropic injection (into `system`)
    if map.contains_key("messages") && (map.contains_key("system") || model_str.contains("claude")) {
        if let Some(sys_val) = map.get_mut("system") {
            match sys_val {
                Value::String(s) => {
                    *s = format!("{}\n\n{}", memory_brief.trim(), s);
                    return true;
                }
                Value::Array(blocks) => {
                    blocks.insert(
                        0,
                        json!({
                            "type": "text",
                            "text": memory_brief.trim()
                        }),
                    );
                    return true;
                }
                _ => {}
            }
        } else {
            map.insert("system".to_string(), json!(memory_brief.trim()));
            return true;
        }
    }

    // OpenAI injection (into messages system/developer role)
    if let Some(Value::Array(messages)) = map.get_mut("messages") {
        if let Some(first_msg) = messages.first_mut() {
            let role = first_msg.get("role").and_then(|v| v.as_str()).unwrap_or("");
            if role == "system" || role == "developer" {
                if let Some(content) = first_msg.get_mut("content") {
                    if let Value::String(s) = content {
                        *s = format!("{}\n\n{}", memory_brief.trim(), s);
                        return true;
                    }
                }
            }
        }

        // Insert new system message at position 0
        messages.insert(
            0,
            json!({
                "role": "system",
                "content": memory_brief.trim()
            }),
        );
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headroom_and_tier_scaling() {
        // 95% free context -> L2 full detail
        let (tier0, budget0) = select_tier_for_headroom(0.95);
        assert_eq!(tier0, DisclosureTier::L2);
        assert_eq!(budget0, 1500);

        // 90% free context -> L1 full budget
        let (tier1, budget1) = select_tier_for_headroom(0.90);
        assert_eq!(tier1, DisclosureTier::L1);
        assert_eq!(budget1, 600);

        // 50% free context -> L1 compressed budget
        let (tier2, budget2) = select_tier_for_headroom(0.50);
        assert_eq!(tier2, DisclosureTier::L1);
        assert_eq!(budget2, 300);

        // 15% free context -> L0 titles only
        let (tier3, budget3) = select_tier_for_headroom(0.15);
        assert_eq!(tier3, DisclosureTier::L0);
        assert_eq!(budget3, 100);
    }

    #[test]
    fn test_inject_adaptive_memory_into_openai_request() {
        let mut req = json!({
            "model": "gpt-4o",
            "messages": [
                {"role": "user", "content": "Write a fast parser"}
            ]
        });

        let injected = inject_adaptive_memory(&mut req);
        assert!(injected);

        let msgs = req["messages"].as_array().unwrap();
        assert_eq!(msgs[0]["role"], "system");
        let content = msgs[0]["content"].as_str().unwrap();
        assert!(content.contains("tokless:adaptive-mem-governor"));
    }
}
