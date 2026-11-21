// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/mod.rs
// created: 2026-10-31
// lastModified: 2026-10-31

#![deny(unsafe_code)]

pub mod agent_cli;
pub mod cascade;
pub mod circuit_breaker;
pub mod compat;
pub mod cost_quality_router;
pub mod gemini_wire;
pub mod governance;
pub mod inference_bridge;
pub mod keys;
pub mod loadbalancer;
pub mod local_engines;
pub mod normalizer;
pub mod providers;
pub mod proxy;
pub mod reconnect;
pub mod router;
pub mod routing;
pub mod semantic_cache;
pub mod server;
pub mod vertex_adc;

pub use agent_cli::*;
pub use cascade::*;
pub use circuit_breaker::*;
pub use compat::*;
pub use governance::*;
pub use keys::*;
pub use loadbalancer::*;
pub use normalizer::*;
pub use providers::*;
pub use proxy::*;
pub use reconnect::*;
pub use router::*;
pub use routing::*;
pub use semantic_cache::*;
pub use server::*;

#[cfg(test)]
mod tests;


