// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/ipc/client.rs
// created: 2026-11-24

//! Cross-platform IPC client for querying/signaling worker subprocesses over Named Pipes or UDS.

#![deny(unsafe_code)]

use std::time::Duration;

use tokio::io::{AsyncRead, AsyncWrite};

use super::frame::{IpcFrame, IpcOpcode};
use super::transport::{ipc_endpoint_for_role, WorkerRole};

async fn exchange<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut S,
    frame: &IpcFrame,
) -> Result<IpcFrame, String> {
    frame.write_to(stream).await?;
    IpcFrame::read_from(stream).await
}

pub async fn send_frame_with_timeout(
    endpoint: &str,
    frame: &IpcFrame,
    timeout: Duration,
) -> Result<IpcFrame, String> {
    tokio::time::timeout(timeout, send_frame(endpoint, frame))
        .await
        .map_err(|_| format!("IPC request to '{endpoint}' timed out after {:?}", timeout))?
}

pub async fn send_frame(endpoint: &str, frame: &IpcFrame) -> Result<IpcFrame, String> {
    #[cfg(windows)]
    {
        use tokio::net::windows::named_pipe::ClientOptions;
        const ERROR_PIPE_BUSY: i32 = 231;

        for attempt in 0..20 {
            match ClientOptions::new().open(endpoint) {
                Ok(mut client) => return exchange(&mut client, frame).await,
                Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) && attempt < 19 => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Err(e) => {
                    return Err(format!("Failed opening named pipe '{endpoint}': {e}"));
                }
            }
        }
        Err(format!("Named pipe '{endpoint}' remained busy"))
    }

    #[cfg(not(windows))]
    {
        use tokio::net::UnixStream;
        let mut stream = UnixStream::connect(endpoint)
            .await
            .map_err(|e| format!("Failed connecting to UDS '{endpoint}': {e}"))?;
        exchange(&mut stream, frame).await
    }
}

pub fn send_frame_blocking(endpoint: &str, frame: &IpcFrame, timeout: Duration) -> Result<IpcFrame, String> {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        let ep = endpoint.to_string();
        let fr = frame.clone();
        std::thread::spawn(move || {
            let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() else {
                return Err("Failed creating helper tokio runtime".to_string());
            };
            rt.block_on(send_frame_with_timeout(&ep, &fr, timeout))
        })
        .join()
        .map_err(|_| format!("IPC thread panicked while contacting {handle:?}"))?
    } else {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed creating tokio runtime: {e}"))?;
        rt.block_on(send_frame_with_timeout(endpoint, frame, timeout))
    }
}

/// Broadcasts `KillSwitchTriggered` to all live worker endpoints; returns count acknowledged.
pub fn broadcast_kill_switch(reason: &str) -> usize {
    let mut acked = 0usize;
    let payload = serde_json::json!({ "reason": reason });
    let Ok(frame) = IpcFrame::json(IpcOpcode::KillSwitchTriggered, 0, &payload) else {
        return 0;
    };
    for role in WorkerRole::ALL {
        let ep = ipc_endpoint_for_role(role.as_str());
        if send_frame_blocking(&ep, &frame, Duration::from_millis(150)).is_ok() {
            acked += 1;
        }
    }
    acked
}

#[derive(Debug, Default, Clone)]
pub struct DirectWireIpcClient;

impl DirectWireIpcClient {
    pub fn ping(&self, role: WorkerRole) -> Result<serde_json::Value, String> {
        let ep = role.ipc_endpoint();
        let frame = IpcFrame::new(IpcOpcode::Ping, 1, Vec::new());
        let reply = send_frame_blocking(&ep, &frame, Duration::from_millis(500))?;
        reply.payload_json()
    }
}
