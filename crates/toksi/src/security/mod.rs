// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/mod.rs
// created: 2026-11-21

//! Security, `<enforce>` Governance, Policy-as-Code (`opa`, `cedar`), SPIFFE IAM (`spire`),
//! Guardrails / LLM-Guard / SafeAgents / Garak / Anonymizer, `defenseclaw`, `skill-scanner`,
//! Zero-Plaintext Blind Signature Engine (`BLAKE3` + `sparse::DFA`), and 41-Language SAST.

#![deny(unsafe_code)]

pub mod advisory_client;
pub mod analyzer;
pub mod audit;
pub mod blind_compiler;
pub mod bytecode_audit;
pub mod cel_decision;
pub mod defenseclaw;
pub mod epss;
pub mod external_analyzers;
pub mod garak_redteam;
pub mod guardrails_validators;
pub mod hookguard;
pub mod iam_attestation;
pub mod llm_guard;
pub mod osv;
pub mod pipeline_taint;
pub mod policy_cedar;
pub mod policy_opa;
pub mod polyglot;
pub mod safeagents;
pub mod sage_rules;
pub mod sarif;
pub mod scanner;
pub mod schema;
pub mod secure_export;
pub mod semgrep_engine;
pub mod skill_scanner;
pub mod yara_mcp;

pub use advisory_client::*;
pub use analyzer::*;
pub use audit::*;
pub use blind_compiler::*;
pub use bytecode_audit::*;
pub use cel_decision::*;
pub use defenseclaw::*;
pub use epss::*;
pub use external_analyzers::*;
pub use garak_redteam::*;
pub use guardrails_validators::*;
pub use hookguard::*;
pub use iam_attestation::*;
pub use llm_guard::*;
pub use osv::*;
pub use pipeline_taint::*;
pub use policy_cedar::*;
pub use policy_opa::*;
pub use polyglot::*;
pub use safeagents::*;
pub use sage_rules::*;
pub use sarif::*;
pub use scanner::*;
pub use schema::*;
pub use secure_export::*;
pub use semgrep_engine::*;
pub use skill_scanner::*;
pub use yara_mcp::*;
