// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/ipc/mod.rs
// created: 2026-11-24

//! Direct-Wire multi-process IPC transport (`iggy`-inspired binary frames over
//! Windows Named Pipes & POSIX Unix Domain Sockets) plus in-process bus helpers.

#![deny(unsafe_code)]

pub mod client;
pub mod frame;
pub mod network_manager;
pub mod server;
pub mod transport;

pub use client::{broadcast_kill_switch, send_frame, send_frame_with_timeout, DirectWireIpcClient};
pub use frame::{IpcFrame, IpcOpcode, IPC_HEADER_BYTES, IPC_MAGIC, IPC_MAX_PAYLOAD_BYTES, IPC_VERSION};
pub use network_manager::{
    bind_single_owner_listeners, forward_payload_over_ipc, run_network_manager,
    spawn_network_manager_listeners, NetworkManagerHandle, NetworkServiceRouteTable,
    PortRouteBinding,
};
pub use server::{serve_worker_ipc, IpcRequestHandler};
pub use transport::{
    current_ipc_namespace, ipc_endpoint_for_role, supervisor_pids_dir, sanitize_ipc_segment,
    WorkerRole,
};
