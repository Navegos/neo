// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/contextmode/compaction/mod.rs
// created: 2026-11-13

use std::collections::{HashMap, HashSet};
use super::journal::JournalRow;

pub const STATE_BUDGET: usize = 2048;
pub const POINTER_MIN: usize = 640;
pub const MAX_CANDIDATES: usize = 85;
pub const MAX_CANDIDATES_TOTAL: usize = 340;
pub const RECALL_BUDGET_BYTES: usize = 4096;
pub const RECALL_MIN_SCORE: f64 = 0.15;
pub const STATE_CLOSE_TAG: &str = "</context_mode_state>";

pub const DECAY_X1000: [u32; 128] = [
    1000, 881, 780, 696, 625, 565, 515, 473, 438, 408, 383, 361, 344, 329, 316, 306,
    297, 289, 283, 278, 273, 270, 267, 264, 262, 260, 258, 257, 256, 255, 254, 253,
    253, 252, 252, 252, 251, 251, 251, 251, 251, 251, 251, 250, 250, 250, 250, 250,
    250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250,
    250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250,
    250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250,
    250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250,
    250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250, 250,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Section {
    LastRequest,
    OpenErrors,
    ChangedFiles,
    Decisions,
    EarlierRequests,
    IndexedSources,
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub section: Section,
    pub seq: i64,
    pub turn: usize,
    pub text: String,
    pub bytes: usize,
    pub repeat_count: usize,
    pub query: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScoredCandidate {
    pub candidate: Candidate,
    pub score: f64,
}

#[derive(Debug, Clone, Default)]
pub struct CandidateSet {
    pub last_request: Option<Candidate>,
    pub open_errors: Vec<Candidate>,
    pub changed_files: Vec<Candidate>,
    pub decisions: Vec<Candidate>,
    pub earlier_requests: Vec<Candidate>,
    pub indexed_sources: Vec<Candidate>,
}

pub fn render_state(rows: &[JournalRow], current_turn: usize, prefix: &str) -> String {
    let mut newest_seq = 0i64;
    let mut session_id = String::new();
    for r in rows {
        if r.seq > newest_seq {
            newest_seq = r.seq;
            session_id = r.session_id.clone();
        }
    }

    if session_id.is_empty() {
        return String::new();
    }

    let set = build_candidates(rows, &session_id);
    let open_tag = format!("<context_mode_state session=\"{}\" upto=\"{}\">", session_id, newest_seq);
    let empty_more = format!("more: {}ctx_search(queries:[], scope:\"session\")", prefix);

    let budget = STATE_BUDGET.saturating_sub(open_tag.len() + 1 + empty_more.len() + 1 + STATE_CLOSE_TAG.len());
    let mut used = 0;

    let pinned_line = set.last_request.as_ref().map(|c| format_candidate_line(c, prefix));
    if let Some(ref pl) = pinned_line {
        used += pl.len() + 1;
    }

    let mut pool = Vec::new();
    pool.extend(set.open_errors);
    pool.extend(set.changed_files);
    pool.extend(set.decisions);
    pool.extend(set.earlier_requests);
    pool.extend(set.indexed_sources);

    let mut scored = score_candidates(pool, current_turn);
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal).then_with(|| b.candidate.seq.cmp(&a.candidate.seq)));

    let mut taken: Vec<ScoredCandidate> = Vec::new();
    let mut skipped: Vec<Candidate> = Vec::new();
    let mut section_counts: HashMap<Section, usize> = HashMap::new();

    for sc in scored {
        let line = format_candidate_line(&sc.candidate, prefix);
        let line_cost = line.len() + 1;
        let count = *section_counts.get(&sc.candidate.section).unwrap_or(&0);
        let decay = 0.7f64.powi(count as i32);
        let effective_score = sc.score * decay;

        if used + line_cost <= budget && effective_score > 0.0 {
            used += line_cost;
            section_counts.insert(sc.candidate.section, count + 1);
            taken.push(sc);
        } else {
            skipped.push(sc.candidate);
        }
    }

    // Output lines in fixed canonical order:
    // 1. openTag
    // 2. now: (last request)
    // 3. open_errors
    // 4. changed_files
    // 5. decisions
    // 6. earlier_requests
    // 7. indexed_sources
    // 8. more: ctx_search
    // 9. closeTag
    let mut lines = Vec::new();
    lines.push(open_tag);

    if let Some(pl) = pinned_line {
        lines.push(pl);
    }

    for target_sec in [
        Section::OpenErrors,
        Section::ChangedFiles,
        Section::Decisions,
        Section::EarlierRequests,
        Section::IndexedSources,
    ] {
        let mut sec_items: Vec<_> = taken.iter().filter(|sc| sc.candidate.section == target_sec).collect();
        sec_items.sort_by_key(|sc| sc.candidate.seq);
        for sc in sec_items {
            lines.push(format_candidate_line(&sc.candidate, prefix));
        }
    }

    // Build 'more: ...' line
    let mut queries: Vec<String> = Vec::new();
    let mut seen_queries = HashSet::new();
    for c in &skipped {
        if let Some(ref q) = c.query {
            if seen_queries.insert(q.clone()) {
                queries.push(q.clone());
            }
        }
    }

    let queries_json = serde_json::to_string(&queries).unwrap_or_else(|_| "[]".to_string());
    lines.push(format!("more: {}ctx_search(queries:{}, scope:\"session\")", prefix, queries_json));
    lines.push(STATE_CLOSE_TAG.to_string());

    lines.join("\n")
}

fn format_candidate_line(c: &Candidate, prefix: &str) -> String {
    if c.bytes > POINTER_MIN {
        format!("#{} ({} B) {}ctx_resume(seq:{})", c.seq, c.bytes, prefix, c.seq)
    } else {
        c.text.clone()
    }
}

pub fn build_candidates(rows: &[JournalRow], session_id: &str) -> CandidateSet {
    if session_id.is_empty() {
        return CandidateSet::default();
    }

    let mut own: Vec<&JournalRow> = rows.iter().filter(|r| r.session_id == session_id).collect();
    own.sort_by_key(|r| r.seq);

    // Prompts
    let prompts: Vec<&&JournalRow> = own.iter().filter(|r| r.kind == "prompt").collect();
    let last_prompt = prompts.last().copied();
    let last_request = last_prompt.map(|r| {
        let first = r.body.lines().next().unwrap_or("").trim();
        let text = format!("now: {}", first);
        Candidate {
            section: Section::LastRequest,
            seq: r.seq,
            turn: r.turn,
            bytes: text.len(),
            text,
            repeat_count: 1,
            query: None,
        }
    });

    let earlier_requests = if prompts.len() > 1 {
        prompts[..prompts.len() - 1].iter().map(|r| {
            let first = r.body.lines().next().unwrap_or("").trim();
            let text = format!("asked #{}: {}", r.seq, first);
            Candidate {
                section: Section::EarlierRequests,
                seq: r.seq,
                turn: r.turn,
                bytes: text.len(),
                text,
                repeat_count: 1,
                query: None,
            }
        }).collect()
    } else {
        Vec::new()
    };

    // Open Errors: latest failing cmd/toolerr per key, open unless a later row of that key succeeded
    let mut failures: HashMap<String, (&JournalRow, usize)> = HashMap::new();
    let mut last_ok: HashMap<String, i64> = HashMap::new();
    for r in &own {
        if r.kind != "cmd" && r.kind != "toolerr" {
            continue;
        }
        let key = match r.key.as_deref() {
            Some(k) if !k.is_empty() => k,
            _ => continue,
        };
        if r.ok == Some(1) {
            last_ok.insert(key.to_string(), r.seq);
        } else {
            let entry = failures.entry(key.to_string()).or_insert((r, 0));
            entry.0 = r;
            entry.1 += 1;
        }
    }

    let mut open_errors = Vec::new();
    for (k, (r, count)) in failures {
        let ok_seq = last_ok.get(&k).copied().unwrap_or(0);
        if ok_seq > r.seq {
            continue;
        }
        let tool_name = r.tool.as_deref().unwrap_or("");
        let text = format!("error #{} {}: {}", r.seq, tool_name, k);
        open_errors.push(Candidate {
            section: Section::OpenErrors,
            seq: r.seq,
            turn: r.turn,
            bytes: text.len(),
            text,
            repeat_count: count,
            query: Some(k),
        });
    }

    // Changed files: last edit wins per path, repeat_count = edits count
    let mut edits: HashMap<String, (&JournalRow, usize)> = HashMap::new();
    for r in &own {
        if r.kind != "edit" {
            continue;
        }
        if let Some(ref path) = r.path {
            let entry = edits.entry(path.clone()).or_insert((r, 0));
            entry.0 = r;
            entry.1 += 1;
        }
    }

    let mut changed_files = Vec::new();
    for (path, (r, count)) in edits {
        let text = format!("edited {} x{}", path, count);
        changed_files.push(Candidate {
            section: Section::ChangedFiles,
            seq: r.seq,
            turn: r.turn,
            bytes: text.len(),
            text,
            repeat_count: count,
            query: Some(path),
        });
    }

    // Decisions: one per decision key, latest row wins
    let mut decided: HashMap<String, (&JournalRow, usize)> = HashMap::new();
    for r in &own {
        if r.kind != "decision" || r.body.is_empty() {
            continue;
        }
        let key = r.key.clone().unwrap_or_else(|| r.body.clone());
        let entry = decided.entry(key).or_insert((r, 0));
        entry.0 = r;
        entry.1 += 1;
    }

    let mut decisions = Vec::new();
    for (key, (r, count)) in decided {
        let first = r.body.lines().next().unwrap_or("").trim();
        let text = format!("decided #{}: {}", r.seq, first);
        decisions.push(Candidate {
            section: Section::Decisions,
            seq: r.seq,
            turn: r.turn,
            bytes: text.len(),
            text,
            repeat_count: count,
            query: Some(key),
        });
    }

    // Indexed sources
    let mut indexed_sources = Vec::new();
    for r in &own {
        if r.kind == "index" {
            let k = r.key.clone().unwrap_or_default();
            let text = format!("indexed {}", k);
            indexed_sources.push(Candidate {
                section: Section::IndexedSources,
                seq: r.seq,
                turn: r.turn,
                bytes: text.len(),
                text,
                repeat_count: 1,
                query: Some(k),
            });
        }
    }

    CandidateSet {
        last_request,
        open_errors: cap_candidates(open_errors),
        changed_files: cap_candidates(changed_files),
        decisions: cap_candidates(decisions),
        earlier_requests: cap_candidates(earlier_requests),
        indexed_sources: cap_candidates(indexed_sources),
    }
}

fn cap_candidates(mut list: Vec<Candidate>) -> Vec<Candidate> {
    list.sort_by_key(|c| c.seq);
    if list.len() > MAX_CANDIDATES {
        list.split_off(list.len() - MAX_CANDIDATES)
    } else {
        list
    }
}

pub fn score_candidates(candidates: Vec<Candidate>, current_turn: usize) -> Vec<ScoredCandidate> {
    candidates.into_iter().map(|c| {
        let base_weight = match c.section {
            Section::OpenErrors => 900.0,
            Section::Decisions => 700.0,
            Section::LastRequest | Section::EarlierRequests => 500.0,
            Section::ChangedFiles => 400.0,
            Section::IndexedSources => 200.0,
        };

        let age = current_turn.saturating_sub(c.turn);
        let decay_idx = age.min(DECAY_X1000.len() - 1);
        let decay = (DECAY_X1000[decay_idx] as f64) / 1000.0;

        let bonus = 1.0 + (c.repeat_count as f64).log2().max(0.0);
        let score = (base_weight * decay * bonus) / c.bytes.max(1) as f64;

        ScoredCandidate {
            candidate: c,
            score,
        }
    }).collect()
}

pub fn pick_recall(rows: &[JournalRow], prompt: &str) -> String {
    if prompt.trim().is_empty() || rows.is_empty() {
        return String::new();
    }

    let prompt_words: HashSet<String> = prompt
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|w| w.len() >= 3)
        .collect();

    if prompt_words.is_empty() {
        return String::new();
    }

    let mut matched = Vec::new();
    for r in rows {
        if r.kind == "cost" || (r.kind == "prompt" && r.body == prompt) {
            continue;
        }

        let body_lower = r.body.to_lowercase();
        let hits = prompt_words.iter().filter(|&w| body_lower.contains(w)).count();
        if hits > 0 {
            let score = (hits as f64) / (prompt_words.len() as f64);
            if score >= RECALL_MIN_SCORE {
                matched.push((r, score));
            }
        }
    }

    if matched.is_empty() {
        return String::new();
    }

    matched.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then_with(|| b.0.seq.cmp(&a.0.seq)));

    let mut out = String::from("<context_mode_recall>\n");
    let mut total_bytes = out.len() + 25; // closing tag

    for (r, _score) in matched {
        let line = format!("#{} {} {}: {}\n", r.seq, r.kind, r.tool.as_deref().unwrap_or(""), r.body.lines().next().unwrap_or(""));
        if total_bytes + line.len() > RECALL_BUDGET_BYTES {
            break;
        }
        total_bytes += line.len();
        out.push_str(&line);
    }

    out.push_str("</context_mode_recall>");
    out
}
