// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/hermes.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! Hermes agent runtime primitives: durable turn leases, 80% run budget notice,
//! runaway stream watchdog, post-answer stop gate, context compaction,
//! streaming secret scrubber, MoA chat completions, and background review runs.

#![deny(unsafe_code)]

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const MIN_FRAGMENT_LENGTH: usize = 400;
pub const REPEAT_WINDOW: usize = 60;
pub const MIN_REPEAT_COUNT: usize = 5;
pub const STOP_PATH_MIN_CHARS: usize = 16_000;
pub const RUN_BUDGET_WRAPUP_RATIO: f64 = 0.80;
pub const RUN_BUDGET_WRAPUP_NOTICE: &str =
    "[RUN_BUDGET_WRAPUP_NOTICE] 80% of wall-clock run budget elapsed. Finalize current step, persist checkpoint, and summarize remaining work.";
pub const CONTEXT_COMPACTION_MARKER: &str = "[CONTEXT COMPACTION — REFERENCE ONLY]";

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnLease {
    pub lease_id: String,
    pub owner: String,
    pub turn_index: u64,
    pub acquired_at_ms: i64,
    pub expires_at_ms: i64,
}

/// Durable turn lease manager (`TurnFacadeMixin`).
#[derive(Debug, Default)]
pub struct TurnFacadeMixin {
    active_lease: Option<TurnLease>,
    default_ttl_ms: i64,
}

impl TurnFacadeMixin {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            active_lease: None,
            default_ttl_ms: default_ttl.as_millis().max(1_000) as i64,
        }
    }

    pub fn acquire_lease(&mut self, owner: &str, turn_index: u64) -> Result<TurnLease, String> {
        let now = now_ms();
        if let Some(ref existing) = self.active_lease {
            if existing.expires_at_ms > now && existing.owner != owner {
                return Err(format!(
                    "Turn lease '{}' still held by '{}' until {}",
                    existing.lease_id, existing.owner, existing.expires_at_ms
                ));
            }
        }
        let lease = TurnLease {
            lease_id: format!("lease-{}-{}", turn_index, now),
            owner: owner.to_string(),
            turn_index,
            acquired_at_ms: now,
            expires_at_ms: now + self.default_ttl_ms,
        };
        self.active_lease = Some(lease.clone());
        Ok(lease)
    }

    pub fn renew_lease(&mut self, lease_id: &str) -> Result<TurnLease, String> {
        let now = now_ms();
        let Some(ref mut lease) = self.active_lease else {
            return Err("No active turn lease to renew".to_string());
        };
        if lease.lease_id != lease_id {
            return Err(format!("Lease mismatch: active='{}', requested='{}'", lease.lease_id, lease_id));
        }
        lease.expires_at_ms = now + self.default_ttl_ms;
        Ok(lease.clone())
    }

    pub fn release_lease(&mut self, lease_id: &str) -> bool {
        if self.active_lease.as_ref().is_some_and(|l| l.lease_id == lease_id) {
            self.active_lease = None;
            return true;
        }
        false
    }

    pub fn active(&self) -> Option<&TurnLease> {
        self.active_lease.as_ref()
    }
}

/// Returns `RUN_BUDGET_WRAPUP_NOTICE` once elapsed time reaches 80% of `budget`.
pub fn check_run_budget_notice(started_at: Instant, budget: Duration) -> Option<&'static str> {
    if budget.is_zero() {
        return None;
    }
    let ratio = started_at.elapsed().as_secs_f64() / budget.as_secs_f64();
    if ratio >= RUN_BUDGET_WRAPUP_RATIO {
        Some(RUN_BUDGET_WRAPUP_NOTICE)
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamWatchVerdict {
    Continue,
    StopRunaway { reason: String, total_chars: usize, repeat_count: usize },
}

/// Detects runaway repetition in streaming LLM output.
#[derive(Debug)]
pub struct RunawayStreamWatch {
    buffer: String,
    window: VecDeque<String>,
    counts: HashMap<String, usize>,
    total_chars: usize,
}

impl Default for RunawayStreamWatch {
    fn default() -> Self {
        Self::new()
    }
}

impl RunawayStreamWatch {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            window: VecDeque::with_capacity(REPEAT_WINDOW + 1),
            counts: HashMap::new(),
            total_chars: 0,
        }
    }

    pub fn push_chunk(&mut self, chunk: &str) -> StreamWatchVerdict {
        self.total_chars = self.total_chars.saturating_add(chunk.len());
        self.buffer.push_str(chunk);

        while self.buffer.len() >= MIN_FRAGMENT_LENGTH {
            let split_idx = floor_char_boundary(&self.buffer, MIN_FRAGMENT_LENGTH);
            if split_idx == 0 {
                break;
            }
            let fragment = self.buffer[..split_idx].to_string();
            self.buffer.drain(..split_idx);

            let mut hasher = Sha256::new();
            hasher.update(fragment.as_bytes());
            let hash: String = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect();

            if self.window.len() >= REPEAT_WINDOW {
                if let Some(old) = self.window.pop_front() {
                    if let Some(c) = self.counts.get_mut(&old) {
                        *c = c.saturating_sub(1);
                        if *c == 0 {
                            self.counts.remove(&old);
                        }
                    }
                }
            }

            let count = self.counts.entry(hash.clone()).or_insert(0);
            *count += 1;
            let current_count = *count;
            self.window.push_back(hash);

            if current_count >= MIN_REPEAT_COUNT && self.total_chars >= STOP_PATH_MIN_CHARS {
                return StreamWatchVerdict::StopRunaway {
                    reason: format!(
                        "Runaway repetition detected: fragment repeated {} times after {} chars",
                        current_count, self.total_chars
                    ),
                    total_chars: self.total_chars,
                    repeat_count: current_count,
                };
            }
        }

        StreamWatchVerdict::Continue
    }

    pub fn total_chars(&self) -> usize {
        self.total_chars
    }
}

fn floor_char_boundary(s: &str, max_bytes: usize) -> usize {
    if max_bytes >= s.len() {
        return s.len();
    }
    let mut idx = max_bytes;
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopGateVerdict {
    Allow,
    BlockAndContinue { reason: String },
    Escalate { reason: String },
}

/// Post-answer barrier ensuring pending verifications or unresolved tool promises are addressed.
pub fn evaluate_stop_gate(
    answer: &str,
    pending_tools: usize,
    verification_passed: bool,
) -> StopGateVerdict {
    if pending_tools > 0 {
        return StopGateVerdict::BlockAndContinue {
            reason: format!("{} pending tool call(s) still open", pending_tools),
        };
    }
    if !verification_passed {
        return StopGateVerdict::BlockAndContinue {
            reason: "Required verification check has not passed".to_string(),
        };
    }
    if answer.trim().is_empty() {
        return StopGateVerdict::Escalate {
            reason: "Empty final answer produced".to_string(),
        };
    }
    StopGateVerdict::Allow
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessage {
    pub role: String,
    pub content: String,
}

/// Context compaction engine tagging compacted history with `[CONTEXT COMPACTION — REFERENCE ONLY]`.
#[derive(Debug, Clone)]
pub struct ContextEngine {
    pub max_messages: usize,
    pub keep_recent: usize,
}

impl Default for ContextEngine {
    fn default() -> Self {
        Self {
            max_messages: 24,
            keep_recent: 8,
        }
    }
}

impl ContextEngine {
    pub fn compact(&self, messages: &[ContextMessage]) -> Vec<ContextMessage> {
        if messages.len() <= self.max_messages {
            return messages.to_vec();
        }
        let split_at = messages.len().saturating_sub(self.keep_recent);
        let older = &messages[..split_at];
        let recent = &messages[split_at..];

        let mut summary_lines = Vec::with_capacity(older.len() + 1);
        summary_lines.push(CONTEXT_COMPACTION_MARKER.to_string());
        for m in older {
            let preview: String = m.content.chars().take(180).collect();
            summary_lines.push(format!("- [{}] {}", m.role, preview.replace('\n', " ")));
        }

        let mut out = Vec::with_capacity(recent.len() + 1);
        out.push(ContextMessage {
            role: "system".to_string(),
            content: summary_lines.join("\n"),
        });
        out.extend_from_slice(recent);
        out
    }
}

/// Redacts sensitive tokens/keys in streaming text chunks.
pub struct StreamingContextScrubber {
    patterns: Vec<(Regex, &'static str)>,
}

impl Default for StreamingContextScrubber {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingContextScrubber {
    pub fn new() -> Self {
        let specs = [
            (r"sk-[A-Za-z0-9_-]{16,}", "[REDACTED_OPENAI_KEY]"),
            (r"sk-ant-[A-Za-z0-9_-]{16,}", "[REDACTED_ANTHROPIC_KEY]"),
            (r"AIza[0-9A-Za-z_-]{20,}", "[REDACTED_GCP_KEY]"),
            (r"ghp_[A-Za-z0-9]{20,}", "[REDACTED_GITHUB_TOKEN]"),
            (r"Bearer\s+[A-Za-z0-9._-]{16,}", "Bearer [REDACTED_TOKEN]"),
        ];
        let patterns = specs
            .iter()
            .filter_map(|(pat, repl)| Regex::new(pat).ok().map(|r| (r, *repl)))
            .collect();
        Self { patterns }
    }

    pub fn scrub(&self, text: &str) -> String {
        let mut out = text.to_string();
        for (re, repl) in &self.patterns {
            out = re.replace_all(&out, *repl).into_owned();
        }
        out
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoACandidate {
    pub model_id: String,
    pub response: String,
    pub score: f64,
}

/// Mixture-of-Agents (MoA) completion aggregator.
#[derive(Debug, Default)]
pub struct MoAChatCompletions {
    pub candidates: Vec<MoACandidate>,
}

impl MoAChatCompletions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_candidate(&mut self, model_id: impl Into<String>, response: impl Into<String>, score: f64) {
        self.candidates.push(MoACandidate {
            model_id: model_id.into(),
            response: response.into(),
            score,
        });
    }

    pub fn synthesize(&self) -> Option<String> {
        let best = self
            .candidates
            .iter()
            .max_by(|a, b| a.score.total_cmp(&b.score))?;
        if self.candidates.len() == 1 {
            return Some(best.response.clone());
        }
        Some(format!(
            "[MoA Synthesis | primary={} | candidates={}]\n{}",
            best.model_id,
            self.candidates.len(),
            best.response
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _BackgroundReviewRun {
    pub run_id: String,
    pub target: String,
    pub status: String,
    pub findings: Vec<String>,
    pub started_at_ms: i64,
    pub completed_at_ms: Option<i64>,
}

impl _BackgroundReviewRun {
    pub fn start(run_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            run_id: run_id.into(),
            target: target.into(),
            status: "running".to_string(),
            findings: Vec::new(),
            started_at_ms: now_ms(),
            completed_at_ms: None,
        }
    }

    pub fn finish(&mut self, findings: Vec<String>) {
        self.findings = findings;
        self.status = "completed".to_string();
        self.completed_at_ms = Some(now_ms());
    }
}
