// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/ipc/frame.rs
// created: 2026-11-24

//! Length-prefixed binary wire envelope (`TKSI`) with truncated BLAKE3 integrity tag.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::commands::test_runner::blake3_hash;

pub const IPC_MAGIC: [u8; 4] = *b"TKSI";
pub const IPC_VERSION: u8 = 1;
pub const IPC_HEADER_BYTES: usize = 18;
pub const IPC_TAG_BYTES: usize = 8;
pub const IPC_MIN_FRAME_BYTES: usize = IPC_HEADER_BYTES + IPC_TAG_BYTES;
pub const IPC_MAX_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpcOpcode {
    Ping = 1,
    Pong = 2,
    Heartbeat = 3,
    ToolDispatch = 4,
    ToolReply = 5,
    SiagentPrompt = 6,
    SiagentReply = 7,
    KillSwitchTriggered = 8,
    GracefulDrain = 9,
    StatusQuery = 10,
    StatusReply = 11,
}

impl IpcOpcode {
    pub fn from_u8(byte: u8) -> Result<Self, String> {
        match byte {
            1 => Ok(Self::Ping),
            2 => Ok(Self::Pong),
            3 => Ok(Self::Heartbeat),
            4 => Ok(Self::ToolDispatch),
            5 => Ok(Self::ToolReply),
            6 => Ok(Self::SiagentPrompt),
            7 => Ok(Self::SiagentReply),
            8 => Ok(Self::KillSwitchTriggered),
            9 => Ok(Self::GracefulDrain),
            10 => Ok(Self::StatusQuery),
            11 => Ok(Self::StatusReply),
            other => Err(format!("Unknown IPC opcode: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpcFrame {
    pub version: u8,
    pub opcode: IpcOpcode,
    pub correlation_id: u64,
    pub payload: Vec<u8>,
}

impl IpcFrame {
    pub fn new(opcode: IpcOpcode, correlation_id: u64, payload: Vec<u8>) -> Self {
        Self {
            version: IPC_VERSION,
            opcode,
            correlation_id,
            payload,
        }
    }

    pub fn json(opcode: IpcOpcode, correlation_id: u64, value: &serde_json::Value) -> Result<Self, String> {
        let bytes = serde_json::to_vec(value).map_err(|e| format!("IPC JSON encode error: {e}"))?;
        Ok(Self::new(opcode, correlation_id, bytes))
    }

    pub fn payload_json(&self) -> Result<serde_json::Value, String> {
        if self.payload.is_empty() {
            return Ok(serde_json::json!({}));
        }
        serde_json::from_slice(&self.payload).map_err(|e| format!("IPC JSON decode error: {e}"))
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        if self.payload.len() > IPC_MAX_PAYLOAD_BYTES {
            return Err(format!(
                "IPC payload exceeds max limit ({} > {})",
                self.payload.len(),
                IPC_MAX_PAYLOAD_BYTES
            ));
        }
        let payload_len = u32::try_from(self.payload.len())
            .map_err(|_| "IPC payload length overflow".to_string())?;
        let mut out = Vec::with_capacity(IPC_MIN_FRAME_BYTES + self.payload.len());
        out.extend_from_slice(&IPC_MAGIC);
        out.push(self.version);
        out.push(self.opcode as u8);
        out.extend_from_slice(&self.correlation_id.to_le_bytes());
        out.extend_from_slice(&payload_len.to_le_bytes());
        out.extend_from_slice(&self.payload);
        let digest = blake3_hash(&out);
        out.extend_from_slice(&digest[..IPC_TAG_BYTES]);
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < IPC_MIN_FRAME_BYTES {
            return Err(format!(
                "Truncated IPC frame ({} < {})",
                bytes.len(),
                IPC_MIN_FRAME_BYTES
            ));
        }
        if bytes[..4] != IPC_MAGIC {
            return Err("Invalid IPC frame magic".to_string());
        }
        let version = bytes[4];
        if version != IPC_VERSION {
            return Err(format!("Unsupported IPC version: {version}"));
        }
        let opcode = IpcOpcode::from_u8(bytes[5])?;
        let corr_bytes: [u8; 8] = bytes[6..14]
            .try_into()
            .map_err(|_| "Invalid IPC correlation slice".to_string())?;
        let correlation_id = u64::from_le_bytes(corr_bytes);
        let len_bytes: [u8; 4] = bytes[14..18]
            .try_into()
            .map_err(|_| "Invalid IPC length slice".to_string())?;
        let payload_len = u32::from_le_bytes(len_bytes) as usize;
        if payload_len > IPC_MAX_PAYLOAD_BYTES {
            return Err(format!(
                "IPC payload exceeds max limit ({payload_len} > {IPC_MAX_PAYLOAD_BYTES})"
            ));
        }
        let expected_len = IPC_MIN_FRAME_BYTES + payload_len;
        if bytes.len() != expected_len {
            return Err(format!(
                "IPC frame size mismatch (got {}, expected {})",
                bytes.len(),
                expected_len
            ));
        }
        let body_end = IPC_HEADER_BYTES + payload_len;
        let expected_tag = blake3_hash(&bytes[..body_end]);
        if bytes[body_end..] != expected_tag[..IPC_TAG_BYTES] {
            return Err("IPC frame checksum mismatch".to_string());
        }
        Ok(Self {
            version,
            opcode,
            correlation_id,
            payload: bytes[IPC_HEADER_BYTES..body_end].to_vec(),
        })
    }

    pub async fn write_to<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<(), String> {
        let encoded = self.encode()?;
        writer
            .write_all(&encoded)
            .await
            .map_err(|e| format!("IPC write failed: {e}"))?;
        writer
            .flush()
            .await
            .map_err(|e| format!("IPC flush failed: {e}"))?;
        Ok(())
    }

    pub async fn read_from<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self, String> {
        let mut header = [0u8; IPC_HEADER_BYTES];
        reader
            .read_exact(&mut header)
            .await
            .map_err(|e| format!("IPC read header failed: {e}"))?;
        if header[..4] != IPC_MAGIC {
            return Err("Invalid IPC stream magic".to_string());
        }
        let len_bytes: [u8; 4] = header[14..18]
            .try_into()
            .map_err(|_| "Invalid IPC stream length".to_string())?;
        let payload_len = u32::from_le_bytes(len_bytes) as usize;
        if payload_len > IPC_MAX_PAYLOAD_BYTES {
            return Err(format!(
                "IPC payload exceeds max limit ({payload_len} > {IPC_MAX_PAYLOAD_BYTES})"
            ));
        }
        let mut full = Vec::with_capacity(IPC_MIN_FRAME_BYTES + payload_len);
        full.extend_from_slice(&header);
        let mut rest = vec![0u8; payload_len + IPC_TAG_BYTES];
        reader
            .read_exact(&mut rest)
            .await
            .map_err(|e| format!("IPC read payload failed: {e}"))?;
        full.extend_from_slice(&rest);
        Self::decode(&full)
    }
}
