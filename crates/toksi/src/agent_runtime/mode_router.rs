// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/mode_router.rs
// created: 2026-11-24

//! 22 Orchestrator Modes across 7 Cognitive Domains (`[toksi_siagent_router.*]`)
//! matching `toksi-siagent_Super_Intelligence_SI_Agent_Orchestrator_Mode_to_Architecture_Mapping.md`
//! and `toksi-siagent_Complete_LLM_Router_&_Model_Gateway_Configuration.toml`.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::config::load_toksi_toml_config;
use crate::db::{shared_db, OrchestratorModeRouteRecord, ToksiConfigDb};
use crate::headroom::effort::EffortLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveDomain {
    MetaCognitionGovernance,
    MemoryWorldModeling,
    DiscoveryEpistemics,
    PedagogyKnowledgeTransfer,
    StrategyArchitecture,
    AutonomousExecution,
    VerificationResilience,
}

impl CognitiveDomain {
    pub const fn key(self) -> &'static str {
        match self {
            Self::MetaCognitionGovernance => "1_meta_cognition_governance",
            Self::MemoryWorldModeling => "2_memory_world_modeling",
            Self::DiscoveryEpistemics => "3_discovery_epistemics",
            Self::PedagogyKnowledgeTransfer => "4_pedagogy_knowledge_transfer",
            Self::StrategyArchitecture => "5_strategy_architecture",
            Self::AutonomousExecution => "6_autonomous_execution",
            Self::VerificationResilience => "7_verification_resilience",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::MetaCognitionGovernance => "Meta-Cognition & Governance",
            Self::MemoryWorldModeling => "Memory & World Modeling",
            Self::DiscoveryEpistemics => "Discovery & Epistemics",
            Self::PedagogyKnowledgeTransfer => "Pedagogy & Knowledge Transfer",
            Self::StrategyArchitecture => "Strategy & Architecture",
            Self::AutonomousExecution => "Autonomous Execution",
            Self::VerificationResilience => "Verification & Resilience",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestratorMode {
    OrchestrateRoute,
    ReflectSelfCritique,
    AlignGuardrail,
    EvolveSelfImprove,
    RecallRetrieve,
    ConsolidateMemorize,
    SimulatePredict,
    ChatInterface,
    BrainstormDiverge,
    ResearchGround,
    SynthesizeAbstract,
    StudyDeconstruct,
    CoachSocratic,
    PlanDecompose,
    ToolsmithArchitect,
    WriteCodeBuild,
    RefactorOptimize,
    ActOperate,
    RunningTestsVerify,
    DebugHeal,
    AuditRedTeam,
    DocumentLog,
}

pub const ALL_ORCHESTRATOR_MODES: [OrchestratorMode; 22] = [
    OrchestratorMode::OrchestrateRoute,
    OrchestratorMode::ReflectSelfCritique,
    OrchestratorMode::AlignGuardrail,
    OrchestratorMode::EvolveSelfImprove,
    OrchestratorMode::RecallRetrieve,
    OrchestratorMode::ConsolidateMemorize,
    OrchestratorMode::SimulatePredict,
    OrchestratorMode::ChatInterface,
    OrchestratorMode::BrainstormDiverge,
    OrchestratorMode::ResearchGround,
    OrchestratorMode::SynthesizeAbstract,
    OrchestratorMode::StudyDeconstruct,
    OrchestratorMode::CoachSocratic,
    OrchestratorMode::PlanDecompose,
    OrchestratorMode::ToolsmithArchitect,
    OrchestratorMode::WriteCodeBuild,
    OrchestratorMode::RefactorOptimize,
    OrchestratorMode::ActOperate,
    OrchestratorMode::RunningTestsVerify,
    OrchestratorMode::DebugHeal,
    OrchestratorMode::AuditRedTeam,
    OrchestratorMode::DocumentLog,
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrchestratorModeSpec {
    pub mode_id: String,
    pub domain: CognitiveDomain,
    pub model_tier: String,
    pub primary_model: String,
    pub fallback_model: String,
    pub reasoning_effort: EffortLevel,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    pub architecture: Vec<String>,
    pub source: String,
}

impl OrchestratorMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OrchestrateRoute => "orchestrate_route",
            Self::ReflectSelfCritique => "reflect_self_critique",
            Self::AlignGuardrail => "align_guardrail",
            Self::EvolveSelfImprove => "evolve_self_improve",
            Self::RecallRetrieve => "recall_retrieve",
            Self::ConsolidateMemorize => "consolidate_memorize",
            Self::SimulatePredict => "simulate_predict",
            Self::ChatInterface => "chat_interface",
            Self::BrainstormDiverge => "brainstorm_diverge",
            Self::ResearchGround => "research_ground",
            Self::SynthesizeAbstract => "synthesize_abstract",
            Self::StudyDeconstruct => "study_deconstruct",
            Self::CoachSocratic => "coach_socratic",
            Self::PlanDecompose => "plan_decompose",
            Self::ToolsmithArchitect => "toolsmith_architect",
            Self::WriteCodeBuild => "write_code_build",
            Self::RefactorOptimize => "refactor_optimize",
            Self::ActOperate => "act_operate",
            Self::RunningTestsVerify => "running_tests_verify",
            Self::DebugHeal => "debug_heal",
            Self::AuditRedTeam => "audit_red_team",
            Self::DocumentLog => "document_log",
        }
    }

    pub const fn domain(self) -> CognitiveDomain {
        match self {
            Self::OrchestrateRoute
            | Self::ReflectSelfCritique
            | Self::AlignGuardrail
            | Self::EvolveSelfImprove => CognitiveDomain::MetaCognitionGovernance,
            Self::RecallRetrieve | Self::ConsolidateMemorize | Self::SimulatePredict => {
                CognitiveDomain::MemoryWorldModeling
            }
            Self::ChatInterface
            | Self::BrainstormDiverge
            | Self::ResearchGround
            | Self::SynthesizeAbstract => CognitiveDomain::DiscoveryEpistemics,
            Self::StudyDeconstruct | Self::CoachSocratic => {
                CognitiveDomain::PedagogyKnowledgeTransfer
            }
            Self::PlanDecompose | Self::ToolsmithArchitect => {
                CognitiveDomain::StrategyArchitecture
            }
            Self::WriteCodeBuild | Self::RefactorOptimize | Self::ActOperate => {
                CognitiveDomain::AutonomousExecution
            }
            Self::RunningTestsVerify
            | Self::DebugHeal
            | Self::AuditRedTeam
            | Self::DocumentLog => CognitiveDomain::VerificationResilience,
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        let key = s.trim().to_ascii_lowercase().replace('-', "_");
        match key.as_str() {
            "orchestrate_route" | "orchestrator" | "route" => Some(Self::OrchestrateRoute),
            "reflect_self_critique" | "reflect" | "critique" => Some(Self::ReflectSelfCritique),
            "align_guardrail" | "guardrail" | "align" => Some(Self::AlignGuardrail),
            "evolve_self_improve" | "evolve" => Some(Self::EvolveSelfImprove),
            "recall_retrieve" | "recall" | "retrieve" => Some(Self::RecallRetrieve),
            "consolidate_memorize" | "memorize" => Some(Self::ConsolidateMemorize),
            "simulate_predict" | "simulate" => Some(Self::SimulatePredict),
            "chat_interface" | "chat" | "ask" => Some(Self::ChatInterface),
            "brainstorm_diverge" | "brainstorm" => Some(Self::BrainstormDiverge),
            "research_ground" | "research" => Some(Self::ResearchGround),
            "synthesize_abstract" | "synthesize" => Some(Self::SynthesizeAbstract),
            "study_deconstruct" | "study" => Some(Self::StudyDeconstruct),
            "coach_socratic" | "coach" => Some(Self::CoachSocratic),
            "plan_decompose" | "plan" | "architect" => Some(Self::PlanDecompose),
            "toolsmith_architect" | "toolsmith" => Some(Self::ToolsmithArchitect),
            "write_code_build" | "code" | "build" => Some(Self::WriteCodeBuild),
            "refactor_optimize" | "refactor" => Some(Self::RefactorOptimize),
            "act_operate" | "act" | "operate" => Some(Self::ActOperate),
            "running_tests_verify" | "verify" | "test" => Some(Self::RunningTestsVerify),
            "debug_heal" | "debug" | "heal" => Some(Self::DebugHeal),
            "audit_red_team" | "audit" | "review" | "red_team" => Some(Self::AuditRedTeam),
            "document_log" | "document" => Some(Self::DocumentLog),
            _ => None,
        }
    }

    pub fn default_spec(self) -> OrchestratorModeSpec {
        let arch = |items: &[&str]| items.iter().map(|s| (*s).to_string()).collect();
        let (tier, prim, fall, eff, temp, tp, tk, a) = match self {
            Self::OrchestrateRoute => (
                "ultra-low-latency-classifier", "claude-haiku-5.5", "gpt-6-luna",
                EffortLevel::Low, Some(1.0), Some(0.85), Some(20),
                arch(&["Orchestrator", "LLM Router", "Model Gateway", "State Machine", "Event Bus", "Message Broker"]),
            ),
            Self::ReflectSelfCritique => (
                "deep-reasoning-critic", "gpt-6-astra", "gemini-3.1-pro",
                EffortLevel::High, Some(1.0), Some(0.95), Some(40),
                arch(&["Verification Engine", "Eval Harness", "Tracing Engine", "Telemetry Engine"]),
            ),
            Self::AlignGuardrail => (
                "deterministic-policy-verifier", "claude-haiku-5.5", "gemini-3.1-flash-lite",
                EffortLevel::Low, Some(1.0), Some(0.8), Some(15),
                arch(&["Guardrails", "Policy-as-Code Engine", "IAM Attestation Layer", "Circuit Breaker", "Kill Switch", "HITL Gateway"]),
            ),
            Self::EvolveSelfImprove => (
                "frontier-meta-architect", "claude-fable-5.2", "gpt-6-astra",
                EffortLevel::Max, Some(1.0), Some(0.95), Some(50),
                arch(&["Eval Harness", "Checkpointer", "Immutable Audit Log", "Tool Registry"]),
            ),
            Self::RecallRetrieve => (
                "retrieval-embedding-rerank", "text-embedding-3-large", "cohere-rerank-v3.5",
                EffortLevel::Low, None, Some(0.85), Some(20),
                arch(&["Context Store", "Episodic Memory", "Semantic Memory", "Vector Database", "Knowledge Graph", "Embedding Model", "Reranker", "Semantic Cache"]),
            ),
            Self::ConsolidateMemorize => (
                "structured-extraction-compression", "gemini-3.6-flash", "claude-sonnet-5.5",
                EffortLevel::Medium, Some(1.0), Some(0.85), Some(25),
                arch(&["Ingestion Pipeline", "Episodic Memory", "Semantic Memory", "Knowledge Graph", "Checkpointer"]),
            ),
            Self::SimulatePredict => (
                "causal-world-simulation", "gemini-4-neon", "gpt-6-astra",
                EffortLevel::High, Some(1.0), Some(0.95), Some(40),
                arch(&["World Model", "Hierarchical Planner", "Inference Engine"]),
            ),
            Self::ChatInterface => (
                "high-elo-conversational", "claude-sonnet-5.5", "gemini-3.6-flash",
                EffortLevel::Medium, Some(1.0), Some(0.92), Some(40),
                arch(&["HITL Gateway", "Context Store", "LLM Router", "Guardrails"]),
            ),
            Self::BrainstormDiverge => (
                "creative-lateral-synthesis", "claude-fable-5.2", "gemini-4-neon",
                EffortLevel::High, Some(1.0), Some(0.98), Some(64),
                arch(&["World Model", "Inference Engine", "Knowledge Graph"]),
            ),
            Self::ResearchGround => (
                "long-context-browser-synthesis", "gemini-4-neon", "gpt-6.1-sol",
                EffortLevel::High, Some(1.0), Some(0.9), Some(30),
                arch(&["MCP Server", "Tool Registry", "Ingestion Pipeline", "Reranker", "Vector Database"]),
            ),
            Self::SynthesizeAbstract => (
                "1m-context-distillation", "gemini-3.1-pro", "claude-opus-5.5",
                EffortLevel::High, Some(1.0), Some(0.9), Some(35),
                arch(&["Semantic Memory", "Knowledge Graph", "Inference Engine"]),
            ),
            Self::StudyDeconstruct => (
                "pedagogical-axiom-builder", "claude-sonnet-5.5", "gemini-3.1-pro",
                EffortLevel::Medium, Some(1.0), Some(0.9), Some(35),
                arch(&["Semantic Memory", "Knowledge Graph", "Hierarchical Planner"]),
            ),
            Self::CoachSocratic => (
                "adaptive-tutor", "claude-fable-5.2", "claude-sonnet-5.5",
                EffortLevel::Medium, Some(1.0), Some(0.92), Some(40),
                arch(&["Episodic Memory", "Context Store", "HITL Gateway"]),
            ),
            Self::PlanDecompose => (
                "hierarchical-dag-planner", "gpt-6-astra", "gemini-3.1-pro",
                EffortLevel::High, Some(1.0), Some(0.92), Some(35),
                arch(&["Hierarchical Planner", "World Model", "State Machine", "Checkpointer"]),
            ),
            Self::ToolsmithArchitect => (
                "agentic-tool-engineer", "gpt-6.2-astra", "claude-opus-5.5",
                EffortLevel::High, Some(1.0), Some(0.9), Some(30),
                arch(&["Tool Registry", "MCP Server", "Code Sandbox", "IAM Attestation Layer"]),
            ),
            Self::WriteCodeBuild => (
                "frontier-software-engineering", "gpt-6.2-astra", "claude-opus-5.5",
                EffortLevel::High, Some(1.0), Some(0.9), Some(30),
                arch(&["Code Sandbox", "Inference Engine", "Checkpointer"]),
            ),
            Self::RefactorOptimize => (
                "algorithmic-efficiency-coder", "claude-opus-5.5", "gpt-6.1-sol",
                EffortLevel::High, Some(1.0), Some(0.9), Some(30),
                arch(&["Code Sandbox", "Telemetry Engine", "Semantic Cache"]),
            ),
            Self::ActOperate => (
                "high-reliability-tool-caller", "gemini-4-neon", "gpt-6.1-terra",
                EffortLevel::Medium, Some(1.0), Some(0.85), Some(20),
                arch(&["Agent Runtime Daemon", "MCP Server", "Tool Registry", "IAM Attestation Layer", "Policy-as-Code Engine"]),
            ),
            Self::RunningTestsVerify => (
                "fast-deterministic-verifier", "deepseek-v4.1-flash", "gpt-6.1-terra",
                EffortLevel::Low, Some(0.0), Some(0.1), Some(1),
                arch(&["Verification Engine", "Code Sandbox", "Eval Harness"]),
            ),
            Self::DebugHeal => (
                "root-cause-trace-surgeon", "gpt-6.2-astra", "claude-opus-5.5",
                EffortLevel::High, Some(1.0), Some(0.9), Some(30),
                arch(&["Tracing Engine", "Telemetry Engine", "Code Sandbox", "Checkpointer"]),
            ),
            Self::AuditRedTeam => (
                "adversarial-security-auditor", "gpt-6-astra", "claude-fable-5.2",
                EffortLevel::Max, Some(1.0), Some(0.95), Some(45),
                arch(&["Verification Engine", "Policy-as-Code Engine", "Immutable Audit Log", "Circuit Breaker"]),
            ),
            Self::DocumentLog => (
                "fast-structured-writer", "gemini-3.6-flash", "claude-sonnet-5.5",
                EffortLevel::Low, Some(1.0), Some(0.85), Some(20),
                arch(&["Immutable Audit Log", "Tracing Engine", "Telemetry Engine", "Ingestion Pipeline"]),
            ),
        };
        OrchestratorModeSpec {
            mode_id: self.as_str().to_string(),
            domain: self.domain(),
            model_tier: tier.to_string(),
            primary_model: prim.to_string(),
            fallback_model: fall.to_string(),
            reasoning_effort: eff,
            temperature: temp,
            top_p: tp,
            top_k: tk,
            architecture: a,
            source: "default".to_string(),
        }
    }
}

/// Resolves mode route using precedence: DB (`orchestrator_mode_routes`) -> `config.toml` -> default.
/// Reuses an existing `ToksiConfigDb` handle when provided to avoid redundant SQLite opens.
pub fn resolve_mode_spec_with_db(
    mode: OrchestratorMode,
    db: Option<&ToksiConfigDb>,
) -> OrchestratorModeSpec {
    let mut spec = mode.default_spec();
    let cfg = load_toksi_toml_config();
    if let Some(domain_map) = cfg.toksi_siagent_router.domains.get(mode.domain().key())
        && let Some(over) = domain_map.get(mode.as_str())
    {
        if let Some(m) = over.primary_model.as_deref().or(over.embedding_model.as_deref()) {
            spec.primary_model = m.to_string();
        }
        if let Some(f) = over.fallback_model.as_deref().or(over.reranker_model.as_deref()) {
            spec.fallback_model = f.to_string();
        }
        if let Some(e) = over.reasoning_effort.as_deref().and_then(EffortLevel::from_str_loose) {
            spec.reasoning_effort = e;
        }
        if over.temperature.is_some() {
            spec.temperature = over.temperature;
        }
        if over.top_p.is_some() {
            spec.top_p = over.top_p;
        }
        if over.top_k.is_some() {
            spec.top_k = over.top_k;
        }
        spec.source = "config.toml".to_string();
    }

    if let Some(db_ref) = db
        && let Ok(Some(rec)) = db_ref.get_orchestrator_mode_route(mode.as_str())
    {
        spec = apply_db_override(spec, &rec);
    }
    spec
}

/// Resolves mode route using precedence: `shared_db()` -> `config.toml` -> default.
pub fn resolve_mode_spec(mode: OrchestratorMode) -> OrchestratorModeSpec {
    if let Ok(arc) = shared_db()
        && let Ok(guard) = arc.lock()
    {
        return resolve_mode_spec_with_db(mode, Some(&guard));
    }
    resolve_mode_spec_with_db(mode, None)
}

pub fn apply_db_override(
    mut spec: OrchestratorModeSpec,
    rec: &OrchestratorModeRouteRecord,
) -> OrchestratorModeSpec {
    if !rec.primary_model.trim().is_empty() {
        spec.primary_model = rec.primary_model.clone();
    }
    if !rec.fallback_model.trim().is_empty() {
        spec.fallback_model = rec.fallback_model.clone();
    }
    if let Some(e) = EffortLevel::from_str_loose(&rec.reasoning_effort) {
        spec.reasoning_effort = e;
    }
    if rec.temperature.is_some() {
        spec.temperature = rec.temperature;
    }
    if rec.top_p.is_some() {
        spec.top_p = rec.top_p;
    }
    if rec.top_k.is_some() {
        spec.top_k = rec.top_k.map(|v| v.max(0) as u32);
    }
    spec.source = "toksi.db".to_string();
    spec
}
