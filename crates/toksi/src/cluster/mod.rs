// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/cluster/mod.rs
// created: 2026-11-24

//! Distributed Cluster Stack unifying Section 2.15 Kubernetes, Kubeflow,
//! NVIDIA AI Factory (NVCRE, `NVSentinel`, `aicr`), Spark (`cudf-spark`), and
//! `Personal-AI-Router` (`PAIR`) LAN GPU mesh failover.

#![deny(unsafe_code)]

pub mod kubeflow;
pub mod kubernetes;
pub mod lan_pair;
pub mod otel_pipeline;
pub mod readiness;
pub mod spark;
pub mod topology;

pub use kubeflow::{
    KatibHpoTrialSpec, ModelRegistryEntry, NemoRunExperimentSpec, PipelineRunSpec, TrainJobSpec,
};
pub use kubernetes::{
    kompose_translate, probe_k8s_tooling, render_aicr_recipe, AicrClusterRecipe,
    K8sToolingStatus, KnavigatorScalePlan,
};
pub use lan_pair::{LanPairMesh, LanPairPeer};
pub use otel_pipeline::{render_otel_collector_yaml, render_otel_operator_crd};
pub use readiness::{
    evaluate_nvcre_readiness, NvSentinelStateMachine, ReadinessGate, ReadinessReport,
};
pub use spark::SparkApplicationSpec;
pub use topology::{ClusterBackendKind, ClusterNode, ClusterNodeHealth, ClusterTopology};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_runtime::sandbox::parse_compose_yaml;

    #[test]
    fn test_cluster_nvcre_nvsentinel_and_lan_pair() {
        let mut n1 = ClusterNode {
            node_id: "gpu-node-1".to_string(),
            backend: ClusterBackendKind::LanPair,
            endpoint: "http://10.0.0.11:8080".to_string(),
            gpu_model: "RTX-5090".to_string(),
            gpu_count: 1,
            vram_gb: 32,
            health: ClusterNodeHealth::Healthy,
            updated_at: 1000,
        };
        let rep_ok = evaluate_nvcre_readiness(std::slice::from_ref(&n1));
        assert!(rep_ok.ready);

        let (health, cordon) = NvSentinelStateMachine::ingest_health_event(&mut n1, Some(79), 85);
        assert_eq!(health, ClusterNodeHealth::Quarantined);
        assert!(cordon);
        assert!(!evaluate_nvcre_readiness(std::slice::from_ref(&n1)).ready);

        let mut mesh = LanPairMesh::new();
        mesh.register_peer(LanPairPeer {
            node_id: "dead".to_string(),
            endpoint: "http://10.0.0.10:8080".to_string(),
            vram_gb: 64,
            health: ClusterNodeHealth::Quarantined,
            active_models: vec!["qwen3".to_string()],
            latency_ms: 1,
        });
        mesh.register_peer(LanPairPeer {
            node_id: "alive".to_string(),
            endpoint: "http://10.0.0.12:8080".to_string(),
            vram_gb: 32,
            health: ClusterNodeHealth::Healthy,
            active_models: vec!["qwen3".to_string()],
            latency_ms: 5,
        });
        let best = mesh.select_best_peer(24, "qwen3").expect("alive peer selected");
        assert_eq!(best.node_id, "alive");
    }

    #[test]
    fn test_kompose_kubeflow_spark_and_aicr() {
        let compose = parse_compose_yaml(
            "services:\n  api:\n    image: toksi-test:local\n    security_opt:\n      - no-new-privileges:true\n",
        )
        .expect("parse compose");
        let k8s_yaml = kompose_translate(&compose);
        assert!(k8s_yaml.contains("kind: Deployment"));
        assert!(k8s_yaml.contains("allowPrivilegeEscalation: false"));

        let tj = TrainJobSpec {
            name: "sft-run".to_string(),
            image: "nvcr.io/nvidia/pytorch:25.01".to_string(),
            num_nodes: 2,
            gpus_per_node: 8,
            command: vec!["torchrun".to_string()],
        };
        assert!(tj.render_yaml().contains("kind: TrainJob"));

        let spark = SparkApplicationSpec {
            name: "etl-rapids".to_string(),
            main_class: "org.toksi.Etl".to_string(),
            executors: 4,
            gpu_enabled: true,
            cudf_rapids_enabled: true,
        };
        assert!(spark.render_manifest().contains("spark.rapids.sql.enabled"));

        let recipe = render_aicr_recipe("prod-h100", 8, "h100-sxm");
        assert!(recipe.rdma_enabled);
    }
}
