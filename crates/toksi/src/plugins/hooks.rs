// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/plugins/hooks.rs
// created: 2026-11-20
// lastModified: 2026-11-20

//! Unified HookEventName matrix and non-blocking plugin hook dispatcher.

#![deny(unsafe_code)]

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookEventName {
    SessionStart,
    SessionEnd,
    UserPromptSubmit,
    PreToolUse,
    PostToolUse,
    PermissionRequest,
    PreCompact,
    PrePatch,
    PostPatch,
    SubagentStart,
    SubagentStop,
    Stop,
}

impl HookEventName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SessionStart => "session_start",
            Self::SessionEnd => "session_end",
            Self::UserPromptSubmit => "user_prompt_submit",
            Self::PreToolUse => "pre_tool_use",
            Self::PostToolUse => "post_tool_use",
            Self::PermissionRequest => "permission_request",
            Self::PreCompact => "pre_compact",
            Self::PrePatch => "pre_patch",
            Self::PostPatch => "post_patch",
            Self::SubagentStart => "subagent_start",
            Self::SubagentStop => "subagent_stop",
            Self::Stop => "stop",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().replace('-', "_").as_str() {
            "session_start" | "sessionstart" => Some(Self::SessionStart),
            "session_end" | "sessionend" => Some(Self::SessionEnd),
            "user_prompt_submit" | "userpromptsubmit" => Some(Self::UserPromptSubmit),
            "pre_tool_use" | "pretooluse" => Some(Self::PreToolUse),
            "post_tool_use" | "posttooluse" => Some(Self::PostToolUse),
            "permission_request" | "permissionrequest" => Some(Self::PermissionRequest),
            "pre_compact" | "precompact" => Some(Self::PreCompact),
            "pre_patch" | "prepatch" => Some(Self::PrePatch),
            "post_patch" | "postpatch" => Some(Self::PostPatch),
            "subagent_start" | "subagentstart" => Some(Self::SubagentStart),
            "subagent_stop" | "subagentstop" => Some(Self::SubagentStop),
            "stop" => Some(Self::Stop),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookOutcome {
    Allow,
    Deny { reason: String },
    Modify { payload: Value },
}

pub type HookCallback = Arc<dyn Fn(&HookContext) -> HookOutcome + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    pub event: HookEventName,
    pub agent_id: String,
    pub tool_name: Option<String>,
    pub payload: Value,
}

#[derive(Clone)]
struct RegisteredHook {
    plugin_id: String,
    priority: i32,
    callback: HookCallback,
}

#[derive(Default)]
pub struct PluginHookDispatcher {
    hooks: RwLock<HashMap<HookEventName, Vec<RegisteredHook>>>,
    fail_closed: RwLock<bool>,
}

impl PluginHookDispatcher {
    pub fn new(fail_closed: bool) -> Self {
        Self {
            hooks: RwLock::new(HashMap::new()),
            fail_closed: RwLock::new(fail_closed),
        }
    }

    pub fn set_fail_closed(&self, fail_closed: bool) {
        if let Ok(mut guard) = self.fail_closed.write() {
            *guard = fail_closed;
        }
    }

    pub fn register_hook(
        &self,
        plugin_id: impl Into<String>,
        event: HookEventName,
        priority: i32,
        callback: HookCallback,
    ) {
        let plugin_id = plugin_id.into();
        if let Ok(mut guard) = self.hooks.write() {
            let list = guard.entry(event).or_default();
            list.push(RegisteredHook {
                plugin_id,
                priority,
                callback,
            });
            list.sort_by_key(|a| std::cmp::Reverse(a.priority));
        }
    }

    pub fn unregister_plugin_hooks(&self, plugin_id: &str) {
        if let Ok(mut guard) = self.hooks.write() {
            for list in guard.values_mut() {
                list.retain(|h| h.plugin_id != plugin_id);
            }
        }
    }

    pub fn dispatch(&self, ctx: &HookContext) -> HookOutcome {
        let entries = match self.hooks.read() {
            Ok(guard) => guard.get(&ctx.event).cloned().unwrap_or_default(),
            Err(_) => {
                let fail_closed = self.fail_closed.read().map(|g| *g).unwrap_or(true);
                if fail_closed {
                    return HookOutcome::Deny {
                        reason: "Hook registry lock poisoned (fail-closed)".to_string(),
                    };
                }
                return HookOutcome::Allow;
            }
        };

        let mut current_ctx = ctx.clone();
        let mut modified = false;

        for entry in entries {
            match (entry.callback)(&current_ctx) {
                HookOutcome::Allow => {}
                HookOutcome::Deny { reason } => {
                    return HookOutcome::Deny {
                        reason: format!("[plugin:{}] {}", entry.plugin_id, reason),
                    };
                }
                HookOutcome::Modify { payload } => {
                    current_ctx.payload = payload;
                    modified = true;
                }
            }
        }

        if modified {
            HookOutcome::Modify {
                payload: current_ctx.payload,
            }
        } else {
            HookOutcome::Allow
        }
    }

    pub fn registered_count(&self) -> usize {
        self.hooks
            .read()
            .map(|g| g.values().map(Vec::len).sum())
            .unwrap_or(0)
    }
}

static GLOBAL_HOOK_DISPATCHER: OnceLock<Arc<PluginHookDispatcher>> = OnceLock::new();

pub fn global_hook_dispatcher() -> Arc<PluginHookDispatcher> {
    Arc::clone(GLOBAL_HOOK_DISPATCHER.get_or_init(|| Arc::new(PluginHookDispatcher::new(true))))
}
