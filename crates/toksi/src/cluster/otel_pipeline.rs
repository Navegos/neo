// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/cluster/otel_pipeline.rs
// created: 2026-11-24

//! `opentelemetry-collector` YAML pipeline & `opentelemetry-operator` CRD
//! (`OpenTelemetryCollector`) manifests for cluster LLMOps observability.

#![deny(unsafe_code)]

/// Renders `opentelemetry-collector` YAML pipeline configuration.
pub fn render_otel_collector_yaml(otlp_port: u16, prometheus_port: u16) -> String {
    let grpc_port = otlp_port.saturating_sub(1).max(4317);
    format!(
        "receivers:\n  otlp:\n    protocols:\n      grpc:\n        endpoint: 0.0.0.0:{grpc_port}\n      http:\n        endpoint: 0.0.0.0:{otlp_port}\n\nprocessors:\n  memory_limiter:\n    check_interval: 1s\n    limit_mib: 512\n  batch:\n    timeout: 5s\n    send_batch_size: 512\n\nexporters:\n  prometheus:\n    endpoint: 0.0.0.0:{prometheus_port}\n  debug:\n    verbosity: basic\n\nservice:\n  pipelines:\n    traces:\n      receivers: [otlp]\n      processors: [memory_limiter, batch]\n      exporters: [debug]\n    metrics:\n      receivers: [otlp]\n      processors: [memory_limiter, batch]\n      exporters: [prometheus]\n"
    )
}

/// Renders Kubernetes `opentelemetry-operator` `OpenTelemetryCollector` CRD manifest.
pub fn render_otel_operator_crd(name: &str, namespace: &str, mode: &str) -> String {
    let clean_name = if name.trim().is_empty() {
        "toksi-otel-collector"
    } else {
        name.trim()
    };
    let clean_ns = if namespace.trim().is_empty() {
        "toksi"
    } else {
        namespace.trim()
    };
    let clean_mode = match mode.trim().to_ascii_lowercase().as_str() {
        "daemonset" => "daemonset",
        "sidecar" => "sidecar",
        _ => "deployment",
    };
    let collector_cfg = render_otel_collector_yaml(4318, 9464)
        .lines()
        .map(|l| {
            if l.is_empty() {
                String::new()
            } else {
                format!("      {l}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "apiVersion: opentelemetry.io/v1beta1\nkind: OpenTelemetryCollector\nmetadata:\n  name: {clean_name}\n  namespace: {clean_ns}\nspec:\n  mode: {clean_mode}\n  config:\n    |\n{collector_cfg}\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otel_collector_yaml_and_operator_crd() {
        let cfg = render_otel_collector_yaml(4318, 9464);
        assert!(cfg.contains("receivers:"));
        assert!(cfg.contains("endpoint: 0.0.0.0:4318"));
        assert!(cfg.contains("endpoint: 0.0.0.0:9464"));
        assert!(cfg.contains("memory_limiter:"));
        assert!(cfg.contains("pipelines:"));

        let crd = render_otel_operator_crd("toksi-otel", "toksi-sys", "daemonset");
        assert!(crd.contains("apiVersion: opentelemetry.io/v1beta1"));
        assert!(crd.contains("kind: OpenTelemetryCollector"));
        assert!(crd.contains("mode: daemonset"));
        assert!(crd.contains("namespace: toksi-sys"));
    }
}
