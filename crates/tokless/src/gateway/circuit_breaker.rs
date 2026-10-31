// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/gateway/circuit_breaker.rs

use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CircuitBreakerReason {
    RepeatingToolCallLoop { tool_name: String, repetitions: u32 },
    RepeatedErrorLoop { error_signature: String, repetitions: u32 },
    RunawayTokenLimitExceeded { session_tokens: u64, limit: u64 },
}

#[derive(Debug, Clone, Default)]
pub struct SessionBreakerState {
    pub last_tool_call: Option<(String, String)>, // (tool_name, arguments_json)
    pub tool_repeat_count: u32,
    pub last_error: Option<String>,
    pub error_repeat_count: u32,
    pub cumulative_tokens: u64,
}

pub struct CircuitBreaker {
    max_tool_repeats: u32,
    max_error_repeats: u32,
    max_session_tokens: u64,
    sessions: Mutex<HashMap<String, SessionBreakerState>>,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self {
            max_tool_repeats: 3,
            max_error_repeats: 4,
            max_session_tokens: 500_000,
            sessions: Mutex::new(HashMap::new()),
        }
    }
}

impl CircuitBreaker {
    pub fn new(max_tool_repeats: u32, max_error_repeats: u32, max_session_tokens: u64) -> Self {
        Self {
            max_tool_repeats,
            max_error_repeats,
            max_session_tokens,
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn inspect_tool_call(&self, session_id: &str, tool_name: &str, args: &Value) -> Result<(), CircuitBreakerReason> {
        let args_str = args.to_string();
        let mut map = self.sessions.lock().unwrap();
        let state = map.entry(session_id.to_string()).or_default();

        if let Some((ref last_name, ref last_args)) = state.last_tool_call {
            if last_name == tool_name && last_args == &args_str {
                state.tool_repeat_count += 1;
                if state.tool_repeat_count >= self.max_tool_repeats {
                    return Err(CircuitBreakerReason::RepeatingToolCallLoop {
                        tool_name: tool_name.to_string(),
                        repetitions: state.tool_repeat_count,
                    });
                }
                return Ok(());
            }
        }

        state.last_tool_call = Some((tool_name.to_string(), args_str));
        state.tool_repeat_count = 1;
        Ok(())
    }

    pub fn inspect_error(&self, session_id: &str, error_msg: &str) -> Result<(), CircuitBreakerReason> {
        let mut map = self.sessions.lock().unwrap();
        let state = map.entry(session_id.to_string()).or_default();

        if let Some(ref last_err) = state.last_error {
            if last_err == error_msg {
                state.error_repeat_count += 1;
                if state.error_repeat_count >= self.max_error_repeats {
                    return Err(CircuitBreakerReason::RepeatedErrorLoop {
                        error_signature: error_msg.to_string(),
                        repetitions: state.error_repeat_count,
                    });
                }
                return Ok(());
            }
        }

        state.last_error = Some(error_msg.to_string());
        state.error_repeat_count = 1;
        Ok(())
    }

    pub fn record_tokens(&self, session_id: &str, tokens: u64) -> Result<(), CircuitBreakerReason> {
        let mut map = self.sessions.lock().unwrap();
        let state = map.entry(session_id.to_string()).or_default();
        state.cumulative_tokens += tokens;

        if state.cumulative_tokens >= self.max_session_tokens {
            return Err(CircuitBreakerReason::RunawayTokenLimitExceeded {
                session_tokens: state.cumulative_tokens,
                limit: self.max_session_tokens,
            });
        }
        Ok(())
    }
}

