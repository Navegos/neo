// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/mod.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! 35-Component `<toksi-siagent>` Native Coding CLI, Swarm Engine, 5-Backend Sandbox,
//! and 16 In-Chat Slash Commands.

#![deny(unsafe_code)]

pub mod acp;
pub mod checkpoint;
pub mod codemode;
pub mod crew;
pub mod durable_journal;
pub mod fleet;
pub mod graph_checkpoint;
pub mod hermes;
pub mod loop_engine;
pub mod modes;
pub mod mofa;
pub mod openclaw;
pub mod patch;
pub mod sandbox;
pub mod scheduler;
pub mod slash;
pub mod state;
pub mod workflow_state;

pub use loop_engine::{AgentRuntimeEngine, RuntimeRunReport};
pub use slash::{global_slash_engine, SlashCommandEngine, SlashDispatchResult, CORE_SLASH_COMMANDS};
