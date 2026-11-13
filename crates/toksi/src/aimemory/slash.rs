// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/slash.rs
// created: 2026-11-13

#![deny(unsafe_code)]

use std::path::Path;
use crate::aimemory::handoff::HandoffStore;
use crate::aimemory::message::{MessageState, MessageStore};
use crate::aimemory::tiered::TieredPacker;
use crate::aimemory::wiki::WikiStore;

pub fn handle_slash_command(input: &str, root: Option<&Path>) -> String {
    let trimmed = input.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    if parts.is_empty() || (parts[0] != "/ai-memory" && parts[0] != "/aimemory") {
        return "Unknown command. Use `/ai-memory help` for available options.".to_string();
    }

    let sub = parts.get(1).copied().unwrap_or("help");

    let wiki = WikiStore::open_default();
    let handoff = HandoffStore::open_default();
    let message = MessageStore::open_default();

    match sub {
        "help" => {
            "**ai-memory Slash Commands:**\n\
             - `/ai-memory status` - Show wiki pages, active handoffs, and pending messages\n\
             - `/ai-memory query <term>` - Search wiki pages and active handoffs\n\
             - `/ai-memory list` - List all wiki concept pages\n\
             - `/ai-memory handoffs` - List active multi-agent handoffs\n\
             - `/ai-memory messages` - List pending cross-project messages\n\
             - `/ai-memory brief [budget]` - Generate progressive disclosure briefing (L0/L1/L2)\n\
             - `/ai-memory purge` - Purge wiki pages matching .gitignore\n\
             - `/ai-memory help` - Show this help message"
                .to_string()
        }
        "status" => {
            let page_count = wiki.list_pages().map(|p| p.len()).unwrap_or(0);
            let handoff_count = handoff.list_active().map(|h| h.len()).unwrap_or(0);
            let msg_pending_count = message
                .list(None, Some(MessageState::Pending))
                .map(|m| m.len())
                .unwrap_or(0);

            format!(
                "**ai-memory Status & Storage:**\n\
                 - Wiki Root: `{}`\n\
                 - Handoffs Root: `{}`\n\
                 - Messages Root: `{}`\n\
                 - OKF Wiki Pages: {}\n\
                 - Active Handoffs: {}\n\
                 - Pending Messages: {}",
                wiki.root_dir().display(),
                handoff.base_dir().display(),
                message.base_dir().display(),
                page_count,
                handoff_count,
                msg_pending_count
            )
        }
        "query" => {
            if parts.len() < 3 {
                return "Usage: `/ai-memory query <term>`".to_string();
            }
            let query = parts[2..].join(" ").to_lowercase();
            let mut out = format!("**Query Results for '{}':**\n", query);

            if let Ok(pages) = wiki.list_pages() {
                for p in pages {
                    if p.slug.to_lowercase().contains(&query) || p.title.to_lowercase().contains(&query) {
                        out.push_str(&format!("- Page: `{}` - {}\n", p.slug, p.title));
                    }
                }
            }

            if let Ok(handoffs) = handoff.list_active() {
                for h in handoffs {
                    if h.title.to_lowercase().contains(&query) || h.task.to_lowercase().contains(&query) {
                        out.push_str(&format!("- Handoff: `{}` - {} (from {})\n", h.id, h.title, h.from_agent));
                    }
                }
            }

            out
        }
        "list" => {
            match wiki.list_pages() {
                Ok(pages) if pages.is_empty() => "No wiki pages recorded yet.".to_string(),
                Ok(pages) => {
                    let mut out = format!("**OKF Wiki Pages ({}):**\n", pages.len());
                    for p in pages.iter().take(30) {
                        out.push_str(&format!("- `{}`: {}\n", p.slug, p.title));
                    }
                    if pages.len() > 30 {
                        out.push_str(&format!("*... and {} more*", pages.len() - 30));
                    }
                    out
                }
                Err(e) => format!("Error listing wiki pages: {e}"),
            }
        }
        "handoffs" => {
            match handoff.list_active() {
                Ok(list) if list.is_empty() => "No active handoffs.".to_string(),
                Ok(list) => {
                    let mut out = format!("**Active Handoffs ({}):**\n", list.len());
                    for h in list {
                        out.push_str(&format!(
                            "- `{}`: {} (state: {}, from: {}, to: {})\n",
                            h.id,
                            h.title,
                            h.state.as_str(),
                            h.from_agent,
                            h.to_agent.as_deref().unwrap_or("any")
                        ));
                    }
                    out
                }
                Err(e) => format!("Error listing handoffs: {e}"),
            }
        }
        "messages" => {
            match message.list(None, Some(MessageState::Pending)) {
                Ok(list) if list.is_empty() => "No pending messages in queue.".to_string(),
                Ok(list) => {
                    let mut out = format!("**Pending Messages ({}):**\n", list.len());
                    for m in list {
                        out.push_str(&format!(
                            "- `{}`: to `{}` (nature: {}) - {}\n",
                            m.id,
                            m.recipient_project,
                            m.nature.as_str(),
                            m.body
                        ));
                    }
                    out
                }
                Err(e) => format!("Error listing messages: {e}"),
            }
        }
        "brief" => {
            let budget = parts.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(600);
            match wiki.to_tiered_entries() {
                Ok(entries) => {
                    let packed = TieredPacker::pack(&entries, budget);
                    format!(
                        "**Progressive Brief ({:?}, ~{} tokens, {} entries):**\n\n{}",
                        packed.tier, packed.estimated_tokens, packed.entries_included, packed.content
                    )
                }
                Err(e) => format!("Error generating brief: {e}"),
            }
        }
        "purge" => {
            let project_root = root
                .map(|p| p.to_path_buf())
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| std::path::PathBuf::from("."));

            match wiki.purge_ignored_pages(&project_root) {
                Ok(count) => format!("Purged {count} wiki pages matching .gitignore/build artifacts."),
                Err(e) => format!("Error purging ignored pages: {e}"),
            }
        }
        _ => "Unknown subcommand. Use `/ai-memory help` for available options.".to_string(),
    }
}
