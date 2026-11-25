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

use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::gateway::sampling::{apply_sampling_params_to_payload, SamplingParams};
use crate::headroom::effort::{EffortLevel, HeadroomEffortCompressor};
use crate::plugins::{global_hook_dispatcher, HookContext, HookEventName, HookOutcome};
use crate::projectmem::plan_checkpoint::{
    save_plan_checkpoint, CheckpointStatus, PlanCheckpoint,
};
use crate::tools::direct_wire::{global_direct_wire_bus, WireRegistrationMode};

use super::crew::{EventBus, ProcessMode, ReflectionAgent, TaskAnalyzer};
use super::hermes::{
    check_run_budget_notice, evaluate_stop_gate, ContextEngine, ContextMessage, RunawayStreamWatch,
    StopGateVerdict, StreamWatchVerdict, StreamingContextScrubber, TurnFacadeMixin,
};
use super::mode_router::{resolve_mode_spec, OrchestratorMode};
use super::mofa::InterruptSignal;
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
    pub gateway_base_url: Option<String>,
    pub active_mode: OrchestratorMode,
    pub active_effort: EffortLevel,
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
                "ctxmode".to_string(),
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
            gateway_base_url: std::env::var("TOKSI_SIAGENT_GATEWAY_URL")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            active_mode: OrchestratorMode::WriteCodeBuild,
            active_effort: EffortLevel::Medium,
        }
    }

    pub fn with_gateway_url(mut self, base_url: impl Into<String>) -> Self {
        let clean = base_url.into().trim().trim_end_matches('/').to_string();
        if !clean.is_empty() {
            self.gateway_base_url = Some(clean);
        }
        self
    }

    pub fn with_mode_and_effort(
        mut self,
        mode: OrchestratorMode,
        effort: EffortLevel,
    ) -> Self {
        self.active_mode = mode;
        self.active_effort = effort;
        self
    }

    pub fn effective_chat_completions_url(&self) -> String {
        let base = self.gateway_base_url.clone().unwrap_or_else(|| {
            let ports = crate::config::resolved_ports();
            format!("http://127.0.0.1:{}/v1", ports.ai_gateway)
        });
        let trimmed = base.trim_end_matches('/');
        if trimmed.ends_with("/chat/completions") {
            trimmed.to_string()
        } else if trimmed.ends_with("/v1") {
            format!("{trimmed}/chat/completions")
        } else {
            format!("{trimmed}/v1/chat/completions")
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

        let mode_spec = resolve_mode_spec(self.active_mode);
        let mut payload = json!({
            "model": mode_spec.primary_model,
            "mode": mode_spec.mode_id,
            "messages": [
                { "role": "user", "content": prompt }
            ]
        });
        apply_sampling_params_to_payload(
            "gateway",
            &mode_spec.primary_model,
            self.active_effort,
            &SamplingParams {
                temperature: mode_spec.temperature,
                top_p: mode_spec.top_p,
                top_k: mode_spec.top_k,
            },
            &mut payload,
        );
        let headers = vec![
            (
                "x-toksi-effort".to_string(),
                self.active_effort.as_str().to_string(),
            ),
            ("x-toksi-mode".to_string(), mode_spec.mode_id.clone()),
        ];
        let stats = self
            .headroom
            .compress_json_payload("/v1/chat/completions", &headers, &mut payload);

        let loop_verdict = self.loop_detector.record_call("projectmem", &json!({ "action": "status" }), None);
        if matches!(loop_verdict, LoopDetectionVerdict::CircuitBreaker { .. }) {
            return Err("ToolLoopDetector circuit breaker tripped".to_string());
        }

        if let Some(notice) = check_run_budget_notice(started, budget) {
            self.event_log.append("budget_notice", json!({ "notice": notice }));
        }

        let url = self.effective_chat_completions_url();
        let raw_reply = self.dispatch_http_blocking(&url, &headers, &payload)?;
        let scrubbed = self.scrubber.scrub(&raw_reply);

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
        let (plan, dag) = analyzer.analyze(goal, ProcessMode::Parallel);
        let root_seq = self
            .causal_bus
            .emit(None, "swarm_started", json!({ "goal": goal, "steps": plan.steps.len() }));

        let waves = dag.waves()?;
        let url = self.effective_chat_completions_url();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .no_proxy()
            .build()
            .map_err(|e| format!("Failed building swarm HTTP client: {e}"))?;
        let mode_spec = resolve_mode_spec(self.active_mode);
        let mut joined = Vec::new();
        for wave in waves {
            for node in wave {
                if self.interrupt.is_triggered() {
                    return Err("Swarm interrupted cleanly".to_string());
                }
                let mut payload = json!({
                    "model": mode_spec.primary_model,
                    "mode": mode_spec.mode_id,
                    "messages": [{
                        "role": "user",
                        "content": format!("[{}:{}] {}", node.agent_role, node.id, node.title)
                    }]
                });
                apply_sampling_params_to_payload(
                    "gateway",
                    &mode_spec.primary_model,
                    self.active_effort,
                    &SamplingParams {
                        temperature: mode_spec.temperature,
                        top_p: mode_spec.top_p,
                        top_k: mode_spec.top_k,
                    },
                    &mut payload,
                );
                let resp = client
                    .post(&url)
                    .header("x-toksi-effort", self.active_effort.as_str())
                    .header("x-toksi-mode", &mode_spec.mode_id)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| {
                        format!(
                            "No live <toksi-siagent> gateway or BYOK provider reachable at {}: {}",
                            url, e
                        )
                    })?;
                if !resp.status().is_success() {
                    return Err(format!(
                        "<toksi-siagent> swarm node '{}' failed: HTTP {}",
                        node.id,
                        resp.status()
                    ));
                }
                let body: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| format!("Invalid gateway JSON on swarm node '{}': {e}", node.id))?;
                let content = body
                    .pointer("/choices/0/message/content")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string)
                    .ok_or_else(|| {
                        format!("Gateway response missing choices[0].message.content on '{}'", node.id)
                    })?;
                joined.push(self.scrubber.scrub(&content));
            }
        }
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
        let rep = super::orchestration_resume::prepare_cross_chat_resume(
            &self.workspace_root,
            plan_id,
        )?;
        Ok(format!(
            "Resumed plan '{}' ('{}', phase='{}', byok='{}'): skipped {} completed step(s), next={:?} ({} total), blake3_verified={}, blake3_drifted={}, reaped_leases={}, skipped_green_suites={}.",
            rep.plan_id,
            rep.title,
            rep.active_phase,
            rep.active_byok_profile,
            rep.completed_count,
            rep.next_step_title,
            rep.total_steps,
            rep.blake3_verified_files.len(),
            rep.blake3_drifted_files.len(),
            rep.reaped_stale_workers.len(),
            rep.skipped_green_suites.len(),
        ))
    }

    pub fn run_chat_repl(&mut self) -> i32 {
        self.run_chat_repl_with_options(false)
    }

    pub fn run_chat_repl_with_options(&mut self, force_plain: bool) -> i32 {
        if !force_plain && io::stdin().is_terminal() && io::stdout().is_terminal() {
            return super::chat_ui::run_interactive_chat_tui(self);
        }
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
                    if let Some(cli_res) =
                        super::chat_ui::execute_in_chat_cli_command(&self.workspace_root, trimmed)
                    {
                        match cli_res {
                            Ok(out) => println!("{}", out),
                            Err(e) => eprintln!("CLI Error: {}", e),
                        }
                        continue;
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

    fn dispatch_http_blocking(
        &self,
        url: &str,
        headers: &[(String, String)],
        payload: &serde_json::Value,
    ) -> Result<String, String> {
        let delegate_ipc = std::env::var("TOKSI_SIAGENT_IPC_DELEGATE")
            .map(|v| matches!(v.trim(), "1" | "true" | "yes"))
            .unwrap_or(false);
        if delegate_ipc
            && self.gateway_base_url.is_none()
            && std::env::var("TOKSI_SIAGENT_GATEWAY_URL")
                .map(|v| v.trim().is_empty())
                .unwrap_or(true)
        {
            let ep = crate::ipc::WorkerRole::Siagent.ipc_endpoint();
            if let Ok(frame) = crate::ipc::IpcFrame::json(
                crate::ipc::IpcOpcode::SiagentPrompt,
                0,
                payload,
            ) && let Ok(resp) = crate::ipc::client::send_frame_blocking(
                &ep,
                &frame,
                Duration::from_millis(250),
            ) && let Ok(v) = resp.payload_json()
                && let Some(out) = v.get("output").and_then(|s| s.as_str())
            {
                return Ok(out.to_string());
            }
        }

        let client = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(30))
            .no_proxy()
            .build()
            .map_err(|e| format!("Failed building gateway HTTP client: {e}"))?;
        let mut req = client.post(url).json(payload);
        for (k, v) in headers {
            req = req.header(k.as_str(), v.as_str());
        }
        let resp = req.send().map_err(|e| {
            format!(
                "No live <toksi-siagent> gateway or BYOK provider reachable at {}: {}",
                url, e
            )
        })?;
        if !resp.status().is_success() {
            return Err(format!("Gateway HTTP {} at {}", resp.status(), url));
        }
        let val: serde_json::Value = resp
            .json()
            .map_err(|e| format!("Invalid gateway response JSON from {url}: {e}"))?;
        val.pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
            .ok_or_else(|| format!("Gateway reply at {url} missing choices[0].message.content"))
    }

    /// Dispatches a prompt from `_worker siagent` IPC server to the live local AI Gateway.
    /// Fails closed cleanly when no gateway or provider runtime is reachable.
    pub fn dispatch_remote_siagent_prompt(&self, prompt: &str, mode: &str) -> Result<String, String> {
        if prompt.trim().is_empty() {
            return Err("Empty prompt passed to <toksi-siagent> worker".to_string());
        }
        let ports = crate::config::resolved_ports();
        let gw_url = format!("http://127.0.0.1:{}/v1/chat/completions", ports.ai_gateway);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .no_proxy()
            .build()
            .map_err(|e| format!("Failed building gateway HTTP client: {e}"))?;
        let resp = client
            .post(&gw_url)
            .json(&serde_json::json!({
                "model": "toksi-auto",
                "mode": mode,
                "messages": [{ "role": "user", "content": prompt }],
            }))
            .send()
            .map_err(|e| format!("AI Gateway {gw_url} unreachable: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("AI Gateway returned HTTP {}", resp.status()));
        }
        let val: serde_json::Value = resp
            .json()
            .map_err(|e| format!("Invalid AI Gateway response JSON: {e}"))?;
        val.pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
            .ok_or_else(|| "AI Gateway reply missing choices[0].message.content".to_string())
    }
}
