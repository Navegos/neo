// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/util/grok.rs
// created: 2026-10-11
// lastModified: 2026-10-14

pub const GROK_OAUTH_PROXY_DEFAULT_PORT: u16 = 28787;

pub fn grok_oauth_proxy_port() -> u16 {
    if let Ok(v) = std::env::var("TOKLESS_GROK_PROXY_PORT") {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            if let Ok(p) = trimmed.parse::<u16>() {
                if p > 0 {
                    return p;
                }
            }
        }
    }
    GROK_OAUTH_PROXY_DEFAULT_PORT
}

pub fn grok_oauth_proxy_base_url() -> String {
    format!("http://127.0.0.1:{}/v1", grok_oauth_proxy_port())
}
