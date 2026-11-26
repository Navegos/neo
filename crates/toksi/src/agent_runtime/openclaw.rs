// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/openclaw.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! OpenClaw 6-detector ToolLoopDetector and periodic advisor fork.

#![deny(unsafe_code)]

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const HISTORY_SIZE: usize = 30;
pub const UNKNOWN_TOOL_REPEAT_THRESHOLD: usize = 10;
pub const CRITICAL_THRESHOLD: usize = 20;
pub const GLOBAL_CIRCUIT_BREAKER_THRESHOLD: usize = 30;
pub const WARNING_THRESHOLD: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopDetectorKind {
    GenericRepeat,
    ArgumentChurn,
    UnknownToolRepeat,
    KnownPollNoProgress,
    GlobalCircuitBreaker,
    PingPong,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopDetectionVerdict {
    Ok,
    Warning {
        detector: LoopDetectorKind,
        count: usize,
        message: String,
    },
    Critical {
        detector: LoopDetectorKind,
        count: usize,
        message: String,
    },
    CircuitBreaker {
        count: usize,
        message: String,
    },
}

#[derive(Debug, Clone)]
struct ToolCallRecord {
    tool_name: String,
    args_hash: String,
    result_hash: Option<String>,
    is_known_tool: bool,
}

/// 6-detector `ToolLoopDetector`:
/// 1. `generic_repeat`
/// 2. `argument_churn`
/// 3. `unknown_tool_repeat` (threshold 10)
/// 4. `known_poll_no_progress`
/// 5. `global_circuit_breaker` (threshold 30)
/// 6. `ping_pong`
#[derive(Debug)]
pub struct ToolLoopDetector {
    history: VecDeque<ToolCallRecord>,
    total_calls: usize,
    known_tools: HashSet<String>,
    poll_tools: HashSet<String>,
}

impl Default for ToolLoopDetector {
    fn default() -> Self {
        Self::new(&[
            "read_file",
            "write_file",
            "edit_file",
            "run_command",
            "apply_patch",
            "hashline_edit",
            "codegraph_explore",
            "projectmem",
            "ctxmode",
            "icm",
            "aimemory",
            "rtk",
            "sandbox",
            "security",
        ])
    }
}

impl ToolLoopDetector {
    pub fn new(known_tool_names: &[&str]) -> Self {
        let known_tools = known_tool_names
            .iter()
            .map(|s| s.to_ascii_lowercase())
            .collect();
        let poll_tools = ["poll", "status", "wait", "get_task", "tasks/get"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        Self {
            history: VecDeque::with_capacity(HISTORY_SIZE + 1),
            total_calls: 0,
            known_tools,
            poll_tools,
        }
    }

    pub fn normalize_sha256(tool_name: &str, args: &Value) -> String {
        let canonical = format!("{}|{}", tool_name.to_ascii_lowercase(), args);
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
    }

    pub fn record_call(
        &mut self,
        tool_name: &str,
        args: &Value,
        result: Option<&Value>,
    ) -> LoopDetectionVerdict {
        self.total_calls += 1;
        let norm_tool = tool_name.to_ascii_lowercase();
        let args_hash = Self::normalize_sha256(&norm_tool, args);
        let result_hash = result.map(|r| {
            let mut hasher = Sha256::new();
            hasher.update(r.to_string().as_bytes());
            hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>()
        });
        let is_known_tool = self.known_tools.contains(&norm_tool);

        if self.history.len() >= HISTORY_SIZE {
            self.history.pop_front();
        }
        self.history.push_back(ToolCallRecord {
            tool_name: norm_tool.clone(),
            args_hash: args_hash.clone(),
            result_hash: result_hash.clone(),
            is_known_tool,
        });

        // 5. Global circuit breaker (30)
        if self.total_calls >= GLOBAL_CIRCUIT_BREAKER_THRESHOLD && self.distinct_signatures() <= 4 {
            return LoopDetectionVerdict::CircuitBreaker {
                count: self.total_calls,
                message: format!(
                    "Global circuit breaker tripped after {} calls with low diversity",
                    self.total_calls
                ),
            };
        }

        // 3. Unknown tool repeat (10)
        if !is_known_tool {
            let unknown_count = self
                .history
                .iter()
                .filter(|r| !r.is_known_tool && r.tool_name == norm_tool)
                .count();
            if unknown_count >= CRITICAL_THRESHOLD {
                return LoopDetectionVerdict::Critical {
                    detector: LoopDetectorKind::UnknownToolRepeat,
                    count: unknown_count,
                    message: format!("Unknown tool '{}' repeated {} times", norm_tool, unknown_count),
                };
            }
            if unknown_count >= UNKNOWN_TOOL_REPEAT_THRESHOLD {
                return LoopDetectionVerdict::Warning {
                    detector: LoopDetectorKind::UnknownToolRepeat,
                    count: unknown_count,
                    message: format!("Unknown tool '{}' repeated {} times", norm_tool, unknown_count),
                };
            }
        }

        // 4. Known poll no progress
        if self.poll_tools.contains(&norm_tool)
            && let Some(ref rh) = result_hash {
                let same_poll_result = self
                    .history
                    .iter()
                    .filter(|r| r.tool_name == norm_tool && r.result_hash.as_ref() == Some(rh))
                    .count();
                if same_poll_result >= CRITICAL_THRESHOLD {
                    return LoopDetectionVerdict::Critical {
                        detector: LoopDetectorKind::KnownPollNoProgress,
                        count: same_poll_result,
                        message: format!("Polling '{}' made no progress for {} calls", norm_tool, same_poll_result),
                    };
                }
                if same_poll_result >= WARNING_THRESHOLD {
                    return LoopDetectionVerdict::Warning {
                        detector: LoopDetectorKind::KnownPollNoProgress,
                        count: same_poll_result,
                        message: format!("Polling '{}' returned identical state {} times", norm_tool, same_poll_result),
                    };
                }
            }

        // 1. Generic repeat (identical tool + args hash)
        let exact_repeats = self
            .history
            .iter()
            .filter(|r| r.args_hash == args_hash)
            .count();
        if exact_repeats >= CRITICAL_THRESHOLD {
            return LoopDetectionVerdict::Critical {
                detector: LoopDetectorKind::GenericRepeat,
                count: exact_repeats,
                message: format!("Tool '{}' with identical args repeated {} times", norm_tool, exact_repeats),
            };
        }
        if exact_repeats >= WARNING_THRESHOLD {
            return LoopDetectionVerdict::Warning {
                detector: LoopDetectorKind::GenericRepeat,
                count: exact_repeats,
                message: format!("Tool '{}' with identical args repeated {} times", norm_tool, exact_repeats),
            };
        }

        // 6. Ping-pong (A -> B -> A -> B)
        let ping_pong_len = self.detect_ping_pong();
        if ping_pong_len >= CRITICAL_THRESHOLD {
            return LoopDetectionVerdict::Critical {
                detector: LoopDetectorKind::PingPong,
                count: ping_pong_len,
                message: format!("Ping-pong oscillation detected over {} turns", ping_pong_len),
            };
        }
        if ping_pong_len >= WARNING_THRESHOLD {
            return LoopDetectionVerdict::Warning {
                detector: LoopDetectorKind::PingPong,
                count: ping_pong_len,
                message: format!("Ping-pong oscillation detected over {} turns", ping_pong_len),
            };
        }

        // 2. Argument churn (same tool, many different args without progress)
        let same_tool_distinct_args: HashSet<&str> = self
            .history
            .iter()
            .filter(|r| r.tool_name == norm_tool)
            .map(|r| r.args_hash.as_str())
            .collect();
        let same_tool_total = self
            .history
            .iter()
            .filter(|r| r.tool_name == norm_tool)
            .count();
        if same_tool_total >= CRITICAL_THRESHOLD && same_tool_distinct_args.len() >= CRITICAL_THRESHOLD / 2 {
            return LoopDetectionVerdict::Critical {
                detector: LoopDetectorKind::ArgumentChurn,
                count: same_tool_total,
                message: format!(
                    "Argument churn on '{}': {} calls across {} distinct arg hashes",
                    norm_tool,
                    same_tool_total,
                    same_tool_distinct_args.len()
                ),
            };
        }

        LoopDetectionVerdict::Ok
    }

    fn distinct_signatures(&self) -> usize {
        self.history
            .iter()
            .map(|r| r.args_hash.as_str())
            .collect::<HashSet<_>>()
            .len()
    }

    fn detect_ping_pong(&self) -> usize {
        let len = self.history.len();
        if len < 4 {
            return 0;
        }
        let a = &self.history[len - 1].tool_name;
        let b = &self.history[len - 2].tool_name;
        if a == b {
            return 0;
        }
        let mut count = 0usize;
        for (idx, rec) in self.history.iter().rev().enumerate() {
            let expected = if idx % 2 == 0 { a } else { b };
            if &rec.tool_name == expected {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    pub fn total_calls(&self) -> usize {
        self.total_calls
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisorAdvice {
    pub turn: u64,
    pub summary: String,
    pub recommended_action: String,
}

/// Periodic `advisor` fork that reviews turn trajectory every `interval_turns`.
#[derive(Debug)]
pub struct AdvisorFork {
    pub interval_turns: u64,
    pub history: HashMap<u64, AdvisorAdvice>,
}

impl Default for AdvisorFork {
    fn default() -> Self {
        Self {
            interval_turns: 5,
            history: HashMap::new(),
        }
    }
}

impl AdvisorFork {
    pub fn should_fork(&self, turn: u64, loop_verdict: &LoopDetectionVerdict) -> bool {
        if !matches!(loop_verdict, LoopDetectionVerdict::Ok) {
            return true;
        }
        turn > 0 && turn.is_multiple_of(self.interval_turns.max(1))
    }

    pub fn advise(&mut self, turn: u64, goal: &str, loop_verdict: &LoopDetectionVerdict) -> AdvisorAdvice {
        let recommended_action = match loop_verdict {
            LoopDetectionVerdict::Ok => "Continue current plan; verify incremental progress.".to_string(),
            LoopDetectionVerdict::Warning { message, .. } => {
                format!("Change strategy: {}", message)
            }
            LoopDetectionVerdict::Critical { message, .. }
            | LoopDetectionVerdict::CircuitBreaker { message, .. } => {
                format!("Halt repeated tool path and escalate/re-plan: {}", message)
            }
        };
        let advice = AdvisorAdvice {
            turn,
            summary: format!("Advisor review at turn {} for '{}'", turn, goal),
            recommended_action,
        };
        self.history.insert(turn, advice.clone());
        advice
    }
}
