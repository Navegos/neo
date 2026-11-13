// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/circuit_breaker.rs
// created: 2026-10-31
// lastModified: 2026-11-13

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
        let mut map = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
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
        let mut map = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
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
        let mut map = self.sessions.lock().unwrap_or_else(|e| e.into_inner());
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

/// Provider-level circuit state for gateway upstream health tracking.
#[derive(Debug, Clone)]
pub struct ProviderCircuitState {
    pub consecutive_failures: u32,
    pub opened_at: Option<std::time::Instant>,
    pub cooldown: std::time::Duration,
}

impl Default for ProviderCircuitState {
    fn default() -> Self {
        Self {
            consecutive_failures: 0,
            opened_at: None,
            cooldown: std::time::Duration::from_secs(30),
        }
    }
}

/// Registry of per-provider circuit breakers consulted by the 7-D router and cascade dispatcher.
#[derive(Debug)]
pub struct CircuitBreakerRegistry {
    failure_threshold: u32,
    default_cooldown: std::time::Duration,
    states: std::sync::RwLock<HashMap<crate::gateway::providers::ProviderId, ProviderCircuitState>>,
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new(3, std::time::Duration::from_secs(30))
    }
}

impl CircuitBreakerRegistry {
    pub fn new(failure_threshold: u32, default_cooldown: std::time::Duration) -> Self {
        Self {
            failure_threshold: failure_threshold.max(1),
            default_cooldown,
            states: std::sync::RwLock::new(HashMap::new()),
        }
    }

    pub fn is_open(&self, provider: crate::gateway::providers::ProviderId) -> bool {
        let mut map = self.states.write().unwrap_or_else(|e| e.into_inner());
        let Some(state) = map.get_mut(&provider) else {
            return false;
        };
        if let Some(opened_at) = state.opened_at {
            if opened_at.elapsed() >= state.cooldown {
                // Cooldown expired: allow half-open retry
                state.opened_at = None;
                state.consecutive_failures = 0;
                return false;
            }
            return true;
        }
        false
    }

    pub fn record_success(&self, provider: crate::gateway::providers::ProviderId) {
        let mut map = self.states.write().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = map.get_mut(&provider) {
            state.consecutive_failures = 0;
            state.opened_at = None;
        }
    }

    pub fn record_failure(
        &self,
        provider: crate::gateway::providers::ProviderId,
        cooldown_override: Option<std::time::Duration>,
    ) {
        let mut map = self.states.write().unwrap_or_else(|e| e.into_inner());
        let state = map.entry(provider).or_insert_with(|| ProviderCircuitState {
            consecutive_failures: 0,
            opened_at: None,
            cooldown: self.default_cooldown,
        });
        state.consecutive_failures += 1;
        if let Some(cd) = cooldown_override {
            state.cooldown = cd;
        }
        if state.consecutive_failures >= self.failure_threshold {
            state.opened_at = Some(std::time::Instant::now());
        }
    }
}

