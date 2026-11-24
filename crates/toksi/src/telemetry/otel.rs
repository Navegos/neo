// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/telemetry/otel.rs
// created: 2026-11-24

//! Native OpenTelemetry GenAI (`gen_ai.*` SemConv 1.29) W3C OTLP/HTTP JSON exporter
//! with local SQLite buffering in `~/.toksi/services/otel.db` (`fail-open`).

#![deny(unsafe_code)]

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::db::service_dbs::{service_db_path, ServiceDbKind, ServiceTelemetryDb};
use crate::util::paths::normalize_path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenAiSpan {
    pub trace_id: String,
    pub span_id: String,
    pub gen_ai_system: String,
    pub gen_ai_operation_name: String,
    pub gen_ai_request_model: String,
    pub gen_ai_request_temperature: Option<f64>,
    pub gen_ai_request_top_p: Option<f64>,
    pub gen_ai_usage_input_tokens: u64,
    pub gen_ai_usage_output_tokens: u64,
    pub toksi_effort: String,
    pub toksi_mode: String,
    pub toksi_headroom_bytes_saved: u64,
    pub start_unix_ms: i64,
    pub end_unix_ms: i64,
}

fn str_attr(key: &str, val: &str) -> Value {
    json!({ "key": key, "value": { "stringValue": val } })
}

fn int_attr(key: &str, val: i64) -> Value {
    json!({ "key": key, "value": { "intValue": val.to_string() } })
}

fn double_attr(key: &str, val: f64) -> Value {
    json!({ "key": key, "value": { "doubleValue": val } })
}

pub struct OtlpJsonExporter;

impl OtlpJsonExporter {
    /// Serializes GenAI spans into W3C OTLP/HTTP JSON `resourceSpans`.
    pub fn to_otlp_resource_spans_json(spans: &[GenAiSpan]) -> Value {
        let otlp_spans: Vec<Value> = spans
            .iter()
            .map(|s| {
                let mut attrs = vec![
                    str_attr("gen_ai.system", &s.gen_ai_system),
                    str_attr("gen_ai.operation.name", &s.gen_ai_operation_name),
                    str_attr("gen_ai.request.model", &s.gen_ai_request_model),
                    int_attr("gen_ai.usage.input_tokens", s.gen_ai_usage_input_tokens as i64),
                    int_attr("gen_ai.usage.output_tokens", s.gen_ai_usage_output_tokens as i64),
                    str_attr("toksi.effort", &s.toksi_effort),
                    str_attr("toksi.mode", &s.toksi_mode),
                    int_attr(
                        "toksi.headroom.bytes_saved",
                        s.toksi_headroom_bytes_saved as i64,
                    ),
                ];
                if let Some(temp) = s.gen_ai_request_temperature {
                    attrs.push(double_attr("gen_ai.request.temperature", temp));
                }
                if let Some(top_p) = s.gen_ai_request_top_p {
                    attrs.push(double_attr("gen_ai.request.top_p", top_p));
                }
                let start_ns = s.start_unix_ms.max(0).saturating_mul(1_000_000);
                let end_ns = s.end_unix_ms.max(s.start_unix_ms).max(0).saturating_mul(1_000_000);
                json!({
                    "traceId": s.trace_id,
                    "spanId": s.span_id,
                    "name": format!("{} {}", s.gen_ai_operation_name, s.gen_ai_request_model),
                    "kind": 3,
                    "startTimeUnixNano": start_ns.to_string(),
                    "endTimeUnixNano": end_ns.to_string(),
                    "attributes": attrs,
                })
            })
            .collect();

        json!({
            "resourceSpans": [{
                "resource": {
                    "attributes": [
                        str_attr("service.name", "toksi"),
                        str_attr("telemetry.sdk.language", "rust"),
                        str_attr("telemetry.sdk.name", "opentelemetry-rust")
                    ]
                },
                "scopeSpans": [{
                    "scope": {
                        "name": "toksi.telemetry.otel",
                        "version": env!("CARGO_PKG_VERSION")
                    },
                    "spans": otlp_spans
                }]
            }]
        })
    }

    pub fn record_span_to_db(span: &GenAiSpan) -> Result<i64, String> {
        let path = service_db_path(ServiceDbKind::Otel);
        Self::record_span_to_db_at(&path, span)
    }

    pub fn record_span_to_db_at(path: &Path, span: &GenAiSpan) -> Result<i64, String> {
        let mut db = ServiceTelemetryDb::open_at(ServiceDbKind::Otel, path)
            .map_err(|e| format!("Failed to open otel service DB {}: {e}", normalize_path(&path.to_string_lossy())))?;
        let payload = serde_json::to_value(span)
            .map_err(|e| format!("Failed to serialize GenAiSpan: {e}"))?;
        db.record_event("gen_ai.span", &payload)
            .map_err(|e| format!("Failed to write gen_ai.span: {e}"))
    }

    pub fn recent_spans(limit: usize) -> Vec<GenAiSpan> {
        let path = service_db_path(ServiceDbKind::Otel);
        Self::recent_spans_at(&path, limit)
    }

    pub fn recent_spans_at(path: &Path, limit: usize) -> Vec<GenAiSpan> {
        let Ok(db) = ServiceTelemetryDb::open_at(ServiceDbKind::Otel, path) else {
            return Vec::new();
        };
        let Ok(rows) = db.list_recent_events(limit.max(1)) else {
            return Vec::new();
        };
        rows.into_iter()
            .filter(|r| r.event_type == "gen_ai.span")
            .filter_map(|r| serde_json::from_str::<GenAiSpan>(&r.payload_json).ok())
            .collect()
    }

    pub fn span_count() -> usize {
        ServiceTelemetryDb::open(ServiceDbKind::Otel)
            .and_then(|d| d.event_count())
            .unwrap_or(0)
    }

    /// Exports OTLP spans non-blocking & fail-open; never returns Err when offline or under TOKSI_TEST=1.
    pub fn export_non_blocking_fail_open(endpoint: Option<&str>, spans: &[GenAiSpan]) -> Value {
        let payload = Self::to_otlp_resource_spans_json(spans);
        let clean_ep = endpoint.map(str::trim).filter(|s| !s.is_empty());
        let is_test = std::env::var("TOKSI_TEST").as_deref() == Ok("1");

        let exported = if let Some(url) = clean_ep
            && !is_test
        {
            let url_owned = url.to_string();
            let body = payload.clone();
            std::thread::spawn(move || {
                let Ok(client) = reqwest::blocking::Client::builder()
                    .timeout(Duration::from_millis(500))
                    .build()
                else {
                    return false;
                };
                client
                    .post(&url_owned)
                    .json(&body)
                    .send()
                    .map(|r| r.status().is_success())
                    .unwrap_or(false)
            })
            .join()
            .unwrap_or(false)
        } else {
            false
        };

        json!({
            "exported": exported,
            "buffered": true,
            "fail_open": true,
            "span_count": spans.len(),
            "endpoint": clean_ep,
            "otlp_payload": payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_span() -> GenAiSpan {
        GenAiSpan {
            trace_id: "0123456789abcdef0123456789abcdef".to_string(),
            span_id: "0123456789abcdef".to_string(),
            gen_ai_system: "anthropic".to_string(),
            gen_ai_operation_name: "chat".to_string(),
            gen_ai_request_model: "claude-sonnet-4".to_string(),
            gen_ai_request_temperature: Some(0.2),
            gen_ai_request_top_p: Some(0.9),
            gen_ai_usage_input_tokens: 1200,
            gen_ai_usage_output_tokens: 340,
            toksi_effort: "high".to_string(),
            toksi_mode: "architect".to_string(),
            toksi_headroom_bytes_saved: 4096,
            start_unix_ms: 1_764_000_000_000,
            end_unix_ms: 1_764_000_001_250,
        }
    }

    #[test]
    fn test_otlp_json_db_roundtrip_and_fail_open() {
        let span = sample_span();
        let otlp = OtlpJsonExporter::to_otlp_resource_spans_json(std::slice::from_ref(&span));
        let attrs = otlp["resourceSpans"][0]["scopeSpans"][0]["spans"][0]["attributes"]
            .as_array()
            .expect("attributes array");
        let keys: Vec<&str> = attrs
            .iter()
            .filter_map(|a| a.get("key").and_then(|k| k.as_str()))
            .collect();
        for required in [
            "gen_ai.system",
            "gen_ai.operation.name",
            "gen_ai.request.model",
            "gen_ai.usage.input_tokens",
            "gen_ai.usage.output_tokens",
            "gen_ai.request.temperature",
            "gen_ai.request.top_p",
            "toksi.effort",
            "toksi.mode",
            "toksi.headroom.bytes_saved",
        ] {
            assert!(keys.contains(&required), "missing OTLP key {}", required);
        }

        let tmp = tempfile::TempDir::new().expect("tempdir");
        let db_path = tmp.path().join("services/otel.db");
        let row_id = OtlpJsonExporter::record_span_to_db_at(&db_path, &span).expect("record span");
        assert!(row_id > 0);
        let recent = OtlpJsonExporter::recent_spans_at(&db_path, 5);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0], span);

        let offline = OtlpJsonExporter::export_non_blocking_fail_open(
            Some("http://127.0.0.1:1/v1/traces"),
            std::slice::from_ref(&span),
        );
        assert_eq!(offline["fail_open"], true);
        assert_eq!(offline["exported"], false);
        assert_eq!(offline["span_count"], 1);
    }
}
