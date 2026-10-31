// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/env_injector/mod.rs
// created: 2026-10-31

pub mod heal;
pub mod teardown;
pub mod unix;
pub mod windows;

pub use heal::{heal_legacy_hijacks, heal_legacy_hijacks_in, HealingReport};
pub use teardown::{
    inject_proxy_env, register_teardown_handlers, teardown_proxy_env, TeardownGuard,
};
