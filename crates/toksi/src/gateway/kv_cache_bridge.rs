// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/gateway/kv_cache_bridge.rs
// created: 2026-11-21

//! KV-cache transfer & shared-memory bridge descriptors (`nixl`, `radixshmem`, `3FS`).

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KvTransportKind {
    NixlRdma,
    RadixShmem,
    ThreeFs,
    TcpFallback,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KvCacheTransferPlan {
    pub transport: KvTransportKind,
    pub source_node: String,
    pub target_node: String,
    pub block_count: usize,
    pub bytes_estimated: u64,
}

pub fn plan_kv_transfer(
    transport: KvTransportKind,
    source_node: &str,
    target_node: &str,
    block_count: usize,
    bytes_per_block: u64,
) -> KvCacheTransferPlan {
    KvCacheTransferPlan {
        transport,
        source_node: source_node.to_string(),
        target_node: target_node.to_string(),
        block_count,
        bytes_estimated: (block_count as u64).saturating_mul(bytes_per_block),
    }
}
