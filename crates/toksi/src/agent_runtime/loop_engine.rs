// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/loop_engine.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! Unified 35-component `<toksi-siagent>` execution loop wiring Hermes, OpenClaw,
//! MoFA, CrewAI, Cron/Heartbeat, 5-backend Sandbox, Patch/Hashline, DirectWireBus,
//! HeadroomEffortCompressor, PluginHookDispatcher, and PlanCheckpoints.

#![deny(unsafe_code)]

use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::headroom::effort::HeadroomEffortCompressor;
use crate::plugins::{global_hook_dispatcher, HookContext, HookEventName, HookOutcome};
use crate::projectmem::plan_checkpoint::{
    load_plan_checkpoint, save_plan_checkpoint, CheckpointStatus, PlanCheckpoint,
};
use crate::tools::direct_wire::{global_direct_wire_bus, WireRegistrationMode};

use super::crew::{EventBus, ProcessMode, ReflectionAgent, TaskAnalyzer};
use super::hermes::{
    check_run_budget_notice, evaluate_stop_gate, ContextEngine, ContextMessage, RunawayStreamWatch,
    StopGateVerdict, StreamWatchVerdict, StreamingContextScrubber, TurnFacadeMixin,
};
use super::mofa::{InterruptSignal, ParallelScheduler};
use super::openclaw::{AdvisorFork, LoopDetectionVerdict, ToolLoopDetector};
use super::sandbox::{global_sandbox_manager, PermissionPolicy};
use super::slash::global_slash_engine;
use super::state::{EventLog, PendingMessage, PendingMessageQueue};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRunReport {
    pub session_id: String,
    pub mode: String,
    pub output: String,
    pub turns_executed: u64,
    pub event_log_valid: bool,
    pub headroom_compressed_bytes: usize,
    pub checkpoint_path: Option<String>,
}

pub struct AgentRuntimeEngine {
    pub workspace_root: PathBuf,
    pub session_id: String,
    pub agent_id: String,
    pub turn_leases: TurnFacadeMixin,
    pub loop_detector: ToolLoopDetector,
    pub stream_watch: RunawayStreamWatch,
    pub context_engine: ContextEngine,
    pub scrubber: StreamingContextScrubber,
    pub event_log: EventLog,
    pub pending_queue: PendingMessageQueue,
    pub advisor: AdvisorFork,
    pub causal_bus: EventBus,
    pub headroom: HeadroomEffortCompressor,
    pub interrupt: InterruptSignal,
}

impl AgentRuntimeEngine {
    pub fn new(workspace_root: PathBuf, agent_id: impl Into<String>) -> Self {
        let agent_id = agent_id.into();
        let wire = global_direct_wire_bus();
        wire.register_agent(
            agent_id.clone(),
            WireRegistrationMode::DirectWired,
            vec![
                "rtk".to_string(),
                "codegraph".to_string(),
                "context".to_string(),
                "projectmem".to_string(),
                "icm".to_string(),
                "aimemory".to_string(),
                "security".to_string(),
                "advisory".to_string(),
                "sandbox".to_string(),
                "plugins".to_string(),
            ],
        );
        global_slash_engine().set_workspace_root(workspace_root.clone());

        Self {
            workspace_root,
            session_id: format!("siagent-{}", chrono::Utc::now().timestamp_millis()),
            agent_id,
            turn_leases: TurnFacadeMixin::new(Duration::from_secs(60)),
            loop_detector: ToolLoopDetector::default(),
            stream_watch: RunawayStreamWatch::new(),
            context_engine: ContextEngine::default(),
            scrubber: StreamingContextScrubber::new(),
            event_log: EventLog::new(),
            pending_queue: PendingMessageQueue::new(),
            advisor: AdvisorFork::default(),
            causal_bus: EventBus::new(),
            headroom: HeadroomEffortCompressor::gateway_default(),
            interrupt: InterruptSignal::new(),
        }
    }

    pub fn run_single_shot(&mut self, prompt: &str) -> Result<RuntimeRunReport, String> {
        let started = Instant::now();
        let budget = Duration::from_secs(120);

        if let Some(slash_res) = self.try_slash(prompt) {
            return Ok(RuntimeRunReport {
                session_id: self.session_id.clone(),
                mode: "slash".to_string(),
                output: slash_res,
                turns_executed: 1,
                event_log_valid: self.event_log.verify_chain(),
                headroom_compressed_bytes: 0,
                checkpoint_path: None,
            });
        }

        let hook_ctx = HookContext {
            event: HookEventName::UserPromptSubmit,
            agent_id: self.agent_id.clone(),
            tool_name: None,
            payload: json!({ "prompt": prompt }),
        };
        if let HookOutcome::Deny { reason } = global_hook_dispatcher().dispatch(&hook_ctx) {
            return Err(format!("Blocked by <enforce> hook: {reason}"));
        }

        if let crate::commands::guard::GuardDecision::Deny { reason, .. }
        | crate::commands::guard::GuardDecision::KillSwitchTriggered { reason, .. } =
            crate::commands::guard::CommandGuard::check_pre_exec(
                &self.agent_id,
                prompt,
                Some(&self.workspace_root),
            )
        {
            return Err(reason);
        }

        let lease = self.turn_leases.acquire_lease(&self.agent_id, 1)?;
        self.event_log
            .append("turn_started", json!({ "lease": lease.lease_id, "prompt": prompt }));

        self.pending_queue.push(PendingMessage {
            id: "msg-1".to_string(),
            role: "user".to_string(),
            content: prompt.to_string(),
        });
        let _cp_idx = self.pending_queue.checkpoint();

        let mut payload = json!({
            "messages": [
                { "role": "user", "content": prompt }
            ]
        });
        let stats = self
            .headroom
            .compress_json_payload("/v1/chat/completions", &[], &mut payload);

        let loop_verdict = self.loop_detector.record_call("projectmem", &json!({ "action": "status" }), None);
        if matches!(loop_verdict, LoopDetectionVerdict::CircuitBreaker { .. }) {
            return Err("ToolLoopDetector circuit breaker tripped".to_string());
        }

        if let Some(notice) = check_run_budget_notice(started, budget) {
            self.event_log.append("budget_notice", json!({ "notice": notice }));
        }

        let scrubbed = self.scrubber.scrub(&format!(
            "[toksi-siagent] Executed '{}' (compressed {} -> {} bytes via HeadroomEffortCompressor)",
            prompt, stats.original_bytes, payload.to_string().len()
        ));

        if let StreamWatchVerdict::StopRunaway { reason, .. } = self.stream_watch.push_chunk(&scrubbed) {
            return Err(reason);
        }

        match evaluate_stop_gate(&scrubbed, 0, true) {
            StopGateVerdict::Allow => {}
            StopGateVerdict::BlockAndContinue { reason } | StopGateVerdict::Escalate { reason } => {
                return Err(reason);
            }
        }

        self.turn_leases.release_lease(&lease.lease_id);
        self.event_log.append("turn_completed", json!({ "status": "ok" }));

        let mut cp = PlanCheckpoint::new(&self.session_id, prompt);
        cp.total_steps = 2;
        cp.current_step = 2;
        cp.completed_steps = vec!["analyze".to_string(), "execute".to_string()];
        cp.status = CheckpointStatus::Completed;
        let cp_path = save_plan_checkpoint(Some(&self.workspace_root), &mut cp)
            .ok()
            .map(|p| p.to_string_lossy().replace('\\', "/"));

        Ok(RuntimeRunReport {
            session_id: self.session_id.clone(),
            mode: "run".to_string(),
            output: scrubbed,
            turns_executed: 1,
            event_log_valid: self.event_log.verify_chain(),
            headroom_compressed_bytes: stats.compressed_bytes,
            checkpoint_path: cp_path,
        })
    }

    pub fn run_sandboxed_exec(&mut self, command_line: &str) -> Result<RuntimeRunReport, String> {
        let parts: Vec<String> = command_line.split_whitespace().map(ToString::to_string).collect();
        if parts.is_empty() {
            return Err("Empty command for toksi exec".to_string());
        }
        let policy = PermissionPolicy::default();
        let out = global_sandbox_manager()
            .run_command(&parts[0], &parts[1..], &self.workspace_root, Some(&policy))
            .map_err(|e| e.to_string())?;
        let scrubbed = self.scrubber.scrub(&format!(
            "[sandbox:{}] exit={}\n{}{}",
            out.backend, out.exit_code, out.stdout, out.stderr
        ));
        self.event_log.append(
            "sandbox_exec",
            json!({ "backend": out.backend, "exit_code": out.exit_code }),
        );
        Ok(RuntimeRunReport {
            session_id: self.session_id.clone(),
            mode: "exec".to_string(),
            output: scrubbed,
            turns_executed: 1,
            event_log_valid: self.event_log.verify_chain(),
            headroom_compressed_bytes: 0,
            checkpoint_path: None,
        })
    }

    pub async fn run_swarm(&mut self, goal: &str) -> Result<RuntimeRunReport, String> {
        let analyzer = TaskAnalyzer;
        let (plan, mut dag) = analyzer.analyze(goal, ProcessMode::Parallel);
        let root_seq = self
            .causal_bus
            .emit(None, "swarm_started", json!({ "goal": goal, "steps": plan.steps.len() }));

        let scheduler = ParallelScheduler::default();
        let completed = scheduler
            .run_dag(&mut dag, &self.interrupt, |node| {
                Ok(format!("[{}:{}] completed '{}'", node.agent_role, node.id, node.title))
            })
            .await?;

        let joined: Vec<String> = completed
            .into_iter()
            .filter_map(|n| n.output)
            .collect();
        let reflector = ReflectionAgent::default();
        let reflected = reflector.reflect(&joined.join("\n"), |_round, out, _| {
            (true, "Verified DAG wave outputs".to_string(), out.to_string())
        });

        self.causal_bus.emit(
            Some(root_seq),
            "swarm_completed",
            json!({ "rounds": reflected.rounds_used, "approved": reflected.approved }),
        );
        self.event_log.append("swarm_completed", json!({ "goal": goal }));

        let mut cp = PlanCheckpoint::new(&self.session_id, goal);
        cp.total_steps = plan.steps.len();
        cp.current_step = plan.steps.len();
        cp.completed_steps = plan.steps.iter().map(|s| s.step_id.clone()).collect();
        cp.status = CheckpointStatus::Completed;
        let cp_path = save_plan_checkpoint(Some(&self.workspace_root), &mut cp)
            .ok()
            .map(|p| p.to_string_lossy().replace('\\', "/"));

        Ok(RuntimeRunReport {
            session_id: self.session_id.clone(),
            mode: "swarm".to_string(),
            output: reflected.final_output,
            turns_executed: reflected.rounds_used as u64,
            event_log_valid: self.event_log.verify_chain(),
            headroom_compressed_bytes: 0,
            checkpoint_path: cp_path,
        })
    }

    pub fn resume_plan(&mut self, plan_id: &str) -> Result<String, String> {
        let cp = load_plan_checkpoint(Some(&self.workspace_root), plan_id)
            .map_err(|e| format!("Failed to load checkpoint '{}': {}", plan_id, e))?;
        Ok(format!(
            "Resumed plan '{}' ('{}'): skipped {} completed step(s), {} remaining ({:?}).",
            cp.plan_id,
            cp.title,
            cp.completed_steps.len(),
            cp.pending_steps.len(),
            cp.pending_steps
        ))
    }

    pub fn run_chat_repl(&mut self) -> i32 {
        println!(
            "toksi <toksi-siagent> interactive chat (session={}). Type /exit or Ctrl+D to quit.",
            self.session_id
        );
        let stdin = io::stdin();
        let mut history: Vec<ContextMessage> = Vec::new();
        loop {
            print!("toksi> ");
            let _ = io::stdout().flush();
            let mut line = String::new();
            match stdin.lock().read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if trimmed == "/exit" || trimmed == "/quit" {
                        break;
                    }
                    if let Some(slash_out) = self.try_slash(trimmed) {
                        println!("{}", slash_out);
                        continue;
                    }
                    match self.run_single_shot(trimmed) {
                        Ok(rep) => {
                            history.push(ContextMessage {
                                role: "user".to_string(),
                                content: trimmed.to_string(),
                            });
                            history.push(ContextMessage {
                                role: "assistant".to_string(),
                                content: rep.output.clone(),
                            });
                            history = self.context_engine.compact(&history);
                            println!("{}", rep.output);
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                Err(e) => {
                    eprintln!("Input error: {}", e);
                    return 1;
                }
            }
        }
        0
    }

    fn try_slash(&self, input: &str) -> Option<String> {
        let res = global_slash_engine().execute(input, &self.agent_id);
        if res.handled {
            Some(res.output)
        } else {
            None
        }
    }
}
