// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/keyring/envelope.rs
// created: 2026-11-04
// lastModified: 2026-11-04

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OAuthEnvelope {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
    pub token_type: Option<String>,
    pub raw_json: Option<Value>,
}

impl OAuthEnvelope {
    pub fn is_expired(&self, buffer_secs: i64) -> bool {
        let Some(exp) = self.expires_at else {
            return false;
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        now + buffer_secs >= exp
    }
}

pub fn decode_base64(input: &str) -> Option<Vec<u8>> {
    let clean = input.trim();
    if clean.is_empty() {
        return None;
    }

    let mut buf = Vec::with_capacity((clean.len() * 3) / 4);
    let mut accum: u32 = 0;
    let mut bits: u32 = 0;

    for &b in clean.as_bytes() {
        if b.is_ascii_whitespace() || b == b'=' {
            continue;
        }
        let val = match b {
            b'A'..=b'Z' => (b - b'A') as u32,
            b'a'..=b'z' => (b - b'a' + 26) as u32,
            b'0'..=b'9' => (b - b'0' + 52) as u32,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => return None,
        };
        accum = (accum << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            buf.push(((accum >> bits) & 0xff) as u8);
        }
    }
    Some(buf)
}

pub fn encode_base64(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    let mut i = 0;
    while i < input.len() {
        let b0 = input[i] as u32;
        let b1 = if i + 1 < input.len() { input[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < input.len() { input[i + 2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3f) as usize] as char);

        if i + 1 < input.len() {
            out.push(TABLE[((triple >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }

        if i + 2 < input.len() {
            out.push(TABLE[(triple & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        i += 3;
    }
    out
}

pub fn parse_credential_envelope(raw: &str) -> Option<OAuthEnvelope> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 1. Try base64 decoding
    if let Some(decoded) = decode_base64(trimmed) {
        if let Ok(utf8_str) = std::str::from_utf8(&decoded) {
            if let Ok(val) = serde_json::from_str::<Value>(utf8_str) {
                if let Some(env) = extract_envelope_from_json(val) {
                    return Some(env);
                }
            }
        }
    }

    // 2. Try direct JSON parsing
    if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
        if let Some(env) = extract_envelope_from_json(val) {
            return Some(env);
        }
    }

    // 3. Fallback: treat as raw token string
    if trimmed.len() >= 8 {
        Some(OAuthEnvelope {
            access_token: trimmed.to_string(),
            refresh_token: None,
            expires_at: None,
            token_type: Some("Bearer".to_string()),
            raw_json: None,
        })
    } else {
        None
    }
}

fn extract_envelope_from_json(val: Value) -> Option<OAuthEnvelope> {
    let access_token = val
        .get("access_token")
        .or_else(|| val.get("token"))
        .or_else(|| val.get("oauth_token"))
        .or_else(|| val.get("sessionKey"))
        .or_else(|| val.get("primaryApiKey"))
        .or_else(|| val.get("apiKey"))
        .or_else(|| val.get("key"))
        .and_then(Value::as_str)?
        .to_string();

    let refresh_token = val
        .get("refresh_token")
        .and_then(Value::as_str)
        .map(|s| s.to_string());

    let token_type = val
        .get("token_type")
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .or_else(|| Some("Bearer".to_string()));

    let mut expires_at: Option<i64> = None;
    if let Some(exp_num) = val.get("expires_at").or_else(|| val.get("expiry")).or_else(|| val.get("expiration")).and_then(Value::as_i64) {
        expires_at = Some(exp_num);
    } else if let Some(exp_str) = val.get("expiry").or_else(|| val.get("expiration")).and_then(Value::as_str) {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(exp_str) {
            expires_at = Some(dt.timestamp());
        }
    } else if let Some(in_secs) = val.get("expires_in").and_then(Value::as_i64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        expires_at = Some(now + in_secs);
    }

    Some(OAuthEnvelope {
        access_token,
        refresh_token,
        expires_at,
        token_type,
        raw_json: Some(val),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let data = b"hello world from antigravity secure keyring envelope";
        let encoded = encode_base64(data);
        let decoded = decode_base64(&encoded).expect("decode");
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_envelope_base64_json() {
        let json = r#"{"access_token":"ya29.test123","expires_in":3600,"token_type":"Bearer"}"#;
        let b64 = encode_base64(json.as_bytes());
        let env = parse_credential_envelope(&b64).expect("parse envelope");
        assert_eq!(env.access_token, "ya29.test123");
        assert!(!env.is_expired(0));
    }

    #[test]
    fn test_envelope_direct_json() {
        let json = r#"{"token":"ya29.direct_test","expiry":"2099-01-01T00:00:00Z"}"#;
        let env = parse_credential_envelope(json).expect("parse envelope");
        assert_eq!(env.access_token, "ya29.direct_test");
        assert!(!env.is_expired(0));
    }

    #[test]
    fn test_envelope_plain_token() {
        let env = parse_credential_envelope("sk-ant-api03-12345678").expect("plain token");
        assert_eq!(env.access_token, "sk-ant-api03-12345678");
    }
}

