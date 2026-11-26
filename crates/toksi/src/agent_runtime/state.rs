// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/agent_runtime/state.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! OpenClaw-rs state primitives: BLAKE2b-256 EventLog, LWW CRDT merge, and PendingMessageQueue rollback.

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Pure-Rust BLAKE2b-256 implementation (RFC 7693) for tamper-evident EventLog chaining.
pub fn blake2b_256(input: &[u8]) -> [u8; 32] {
    const IV: [u64; 8] = [
        0x6a09e667f3bcc908,
        0xbb67ae8584caa73b,
        0x3c6ef372fe94f82b,
        0xa54ff53a5f1d36f1,
        0x510e527fade682d1,
        0x9b05688c2b3e6c1f,
        0x1f83d9abfb41bd6b,
        0x5be0cd19137e2179,
    ];
    const SIGMA: [[usize; 16]; 12] = [
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
        [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
        [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
        [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
        [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
        [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
        [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
        [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
        [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    ];

    fn g(v: &mut [u64; 16], a: usize, b: usize, c: usize, d: usize, x: u64, y: u64) {
        v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
        v[d] = (v[d] ^ v[a]).rotate_right(32);
        v[c] = v[c].wrapping_add(v[d]);
        v[b] = (v[b] ^ v[c]).rotate_right(24);
        v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
        v[d] = (v[d] ^ v[a]).rotate_right(16);
        v[c] = v[c].wrapping_add(v[d]);
        v[b] = (v[b] ^ v[c]).rotate_right(63);
    }

    fn compress(h: &mut [u64; 8], block: &[u8; 128], t: u128, last: bool) {
        let mut m = [0u64; 16];
        for (i, chunk) in block.as_chunks::<8>().0.iter().enumerate() {
            m[i] = u64::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
            ]);
        }
        let mut v = [0u64; 16];
        v[..8].copy_from_slice(h);
        v[8..].copy_from_slice(&IV);
        v[12] ^= t as u64;
        v[13] ^= (t >> 64) as u64;
        if last {
            v[14] = !v[14];
        }
        for s in &SIGMA {
            g(&mut v, 0, 4, 8, 12, m[s[0]], m[s[1]]);
            g(&mut v, 1, 5, 9, 13, m[s[2]], m[s[3]]);
            g(&mut v, 2, 6, 10, 14, m[s[4]], m[s[5]]);
            g(&mut v, 3, 7, 11, 15, m[s[6]], m[s[7]]);
            g(&mut v, 0, 5, 10, 15, m[s[8]], m[s[9]]);
            g(&mut v, 1, 6, 11, 12, m[s[10]], m[s[11]]);
            g(&mut v, 2, 7, 8, 13, m[s[12]], m[s[13]]);
            g(&mut v, 3, 4, 9, 14, m[s[14]], m[s[15]]);
        }
        for i in 0..8 {
            h[i] ^= v[i] ^ v[i + 8];
        }
    }

    let mut h = IV;
    h[0] ^= 0x01010020; // digest_len = 32, key_len = 0, fanout = 1, depth = 1

    let mut offset = 0usize;
    if input.len() > 128 {
        while input.len() - offset > 128 {
            let mut block = [0u8; 128];
            block.copy_from_slice(&input[offset..offset + 128]);
            offset += 128;
            compress(&mut h, &block, offset as u128, false);
        }
    }

    let mut last_block = [0u8; 128];
    let rem = input.len() - offset;
    last_block[..rem].copy_from_slice(&input[offset..]);
    compress(&mut h, &last_block, input.len() as u128, true);

    let mut out = [0u8; 32];
    for (i, word) in h[..4].iter().enumerate() {
        out[i * 8..(i + 1) * 8].copy_from_slice(&word.to_le_bytes());
    }
    out
}

pub fn blake2b_256_hex(input: &[u8]) -> String {
    blake2b_256(input)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogEntry {
    pub seq: u64,
    pub kind: String,
    pub payload: Value,
    pub timestamp_ms: i64,
    pub prev_hash: String,
    pub hash: String,
}

/// Append-only BLAKE2b-256 hash-chained event log.
#[derive(Debug, Default, Clone)]
pub struct EventLog {
    entries: Vec<EventLogEntry>,
}

impl EventLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, kind: impl Into<String>, payload: Value) -> EventLogEntry {
        let seq = self.entries.len() as u64 + 1;
        let kind = kind.into();
        let timestamp_ms = now_ms();
        let prev_hash = self
            .entries
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(|| "0".repeat(64));
        let body = format!(
            "{}|{}|{}|{}|{}",
            seq, kind, timestamp_ms, prev_hash, payload
        );
        let hash = blake2b_256_hex(body.as_bytes());
        let entry = EventLogEntry {
            seq,
            kind,
            payload,
            timestamp_ms,
            prev_hash,
            hash,
        };
        self.entries.push(entry.clone());
        entry
    }

    pub fn verify_chain(&self) -> bool {
        let mut expected_prev = "0".repeat(64);
        for e in &self.entries {
            if e.prev_hash != expected_prev {
                return false;
            }
            let body = format!(
                "{}|{}|{}|{}|{}",
                e.seq, e.kind, e.timestamp_ms, e.prev_hash, e.payload
            );
            let computed = blake2b_256_hex(body.as_bytes());
            if computed != e.hash {
                return false;
            }
            expected_prev = e.hash.clone();
        }
        true
    }

    pub fn entries(&self) -> &[EventLogEntry] {
        &self.entries
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrdtEntry {
    pub value: Value,
    pub lamport_ts: u64,
    pub replica_id: String,
}

/// Last-Write-Wins (LWW) CRDT state map for distributed swarm synchronization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrdtState {
    pub replica_id: String,
    pub clock: u64,
    pub entries: HashMap<String, CrdtEntry>,
}

impl CrdtState {
    pub fn new(replica_id: impl Into<String>) -> Self {
        Self {
            replica_id: replica_id.into(),
            clock: 0,
            entries: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.clock += 1;
        self.entries.insert(
            key.into(),
            CrdtEntry {
                value,
                lamport_ts: self.clock,
                replica_id: self.replica_id.clone(),
            },
        );
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.entries.get(key).map(|e| &e.value)
    }

    pub fn merge(&mut self, other: &Self) {
        self.clock = self.clock.max(other.clock);
        for (k, remote) in &other.entries {
            match self.entries.get(k) {
                Some(local)
                    if local.lamport_ts > remote.lamport_ts
                        || (local.lamport_ts == remote.lamport_ts
                            && local.replica_id >= remote.replica_id) => {}
                _ => {
                    self.entries.insert(k.clone(), remote.clone());
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingMessage {
    pub id: String,
    pub role: String,
    pub content: String,
}

/// Rollback-capable pending message queue.
#[derive(Debug, Default, Clone)]
pub struct PendingMessageQueue {
    queue: Vec<PendingMessage>,
    checkpoints: Vec<Vec<PendingMessage>>,
}

impl PendingMessageQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, msg: PendingMessage) {
        self.queue.push(msg);
    }

    pub fn pop(&mut self) -> Option<PendingMessage> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.queue.remove(0))
        }
    }

    pub fn checkpoint(&mut self) -> usize {
        self.checkpoints.push(self.queue.clone());
        self.checkpoints.len() - 1
    }

    pub fn rollback(&mut self, checkpoint_idx: usize) -> bool {
        if let Some(snap) = self.checkpoints.get(checkpoint_idx).cloned() {
            self.queue = snap;
            self.checkpoints.truncate(checkpoint_idx + 1);
            return true;
        }
        false
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
