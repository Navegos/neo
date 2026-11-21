// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/telemetry/guardrails.rs
// created: 2026-11-21

//! `agent-control` telemetry guardrail evaluation event recorder.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardrailOutcome {
    Pass,
    Warn,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailEvalEvent {
    pub control_id: String,
    pub agent_id: String,
    pub outcome: GuardrailOutcome,
    pub reason: String,
    pub timestamp_unix: i64,
}

#[derive(Debug, Default)]
pub struct AgentControlTelemetry {
    events: Vec<GuardrailEvalEvent>,
}

impl AgentControlTelemetry {
    pub fn record(&mut self, event: GuardrailEvalEvent) {
        self.events.push(event);
    }

    pub fn counts(&self) -> (usize, usize, usize) {
        let mut pass = 0;
        let mut warn = 0;
        let mut block = 0;
        for ev in &self.events {
            match ev.outcome {
                GuardrailOutcome::Pass => pass += 1,
                GuardrailOutcome::Warn => warn += 1,
                GuardrailOutcome::Block => block += 1,
            }
        }
        (pass, warn, block)
    }

    pub fn events(&self) -> &[GuardrailEvalEvent] {
        &self.events
    }
}
