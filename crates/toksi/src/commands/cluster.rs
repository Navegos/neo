// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/commands/cluster.rs
// created: 2026-11-24

//! CLI dispatcher for `toksi cluster <status|nodes|readiness|nvsentinel|recipe|kubeflow|spark|lan|kompose>`.

#![deny(unsafe_code)]

use std::path::Path;

use crate::agent_runtime::sandbox::parse_compose_yaml;
use crate::cluster::{
    evaluate_nvcre_readiness, kompose_translate, probe_k8s_tooling, render_aicr_recipe,
    ClusterBackendKind, ClusterNode, ClusterNodeHealth, ClusterTopology, LanPairMesh,
    LanPairPeer, NvSentinelStateMachine, SparkApplicationSpec, TrainJobSpec,
};

pub fn run_cluster_cli(args: &[String]) -> i32 {
    let sub = args.first().map(String::as_str).unwrap_or("status");
    match sub {
        "status" => {
            let tools = probe_k8s_tooling();
            let nodes = crate::db::shared_db()
                .ok()
                .and_then(|a| a.lock().ok().and_then(|d| d.list_cluster_nodes().ok()))
                .unwrap_or_default();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "tooling": tools,
                    "registered_nodes": nodes.len(),
                }))
                .unwrap_or_default()
            );
            0
        }
        "nodes" => {
            let list = crate::db::shared_db()
                .ok()
                .and_then(|a| {
                    a.lock()
                        .ok()
                        .and_then(|d| ClusterTopology::load_from_config_db(&d).ok())
                })
                .unwrap_or_default();
            println!("{}", serde_json::to_string_pretty(&list).unwrap_or_default());
            0
        }
        "readiness" => {
            let nodes = crate::db::shared_db()
                .ok()
                .and_then(|a| {
                    a.lock()
                        .ok()
                        .and_then(|d| ClusterTopology::load_from_config_db(&d).ok())
                })
                .map(|t| t.nodes)
                .unwrap_or_default();
            let report = evaluate_nvcre_readiness(&nodes);
            println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
            0
        }
        "nvsentinel" => {
            let xid = args.get(1).and_then(|s| s.parse::<u32>().ok());
            let temp = args.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(65);
            let mut node = ClusterNode {
                node_id: "probe-node".to_string(),
                backend: ClusterBackendKind::Local,
                endpoint: "local".to_string(),
                gpu_model: "local-gpu".to_string(),
                gpu_count: 1,
                vram_gb: 24,
                health: ClusterNodeHealth::Healthy,
                updated_at: chrono::Utc::now().timestamp(),
            };
            let (health, cordon) = NvSentinelStateMachine::ingest_health_event(&mut node, xid, temp);
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "health": health.as_str(),
                    "cordon_recommended": cordon,
                }))
                .unwrap_or_default()
            );
            0
        }
        "recipe" => {
            let name = args.get(1).map(String::as_str).unwrap_or("toksi-aicr");
            let gpus = args.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(8);
            let profile = args.get(3).map(String::as_str).unwrap_or("h100-sxm");
            println!(
                "{}",
                serde_json::to_string_pretty(&render_aicr_recipe(name, gpus, profile))
                    .unwrap_or_default()
            );
            0
        }
        "kubeflow" => {
            let name = args.get(1).map(String::as_str).unwrap_or("toksi-trainjob");
            let tj = TrainJobSpec {
                name: name.to_string(),
                image: "rust:1.88-slim-bookworm".to_string(),
                num_nodes: 1,
                gpus_per_node: 1,
                command: vec!["toksi".to_string(), "train".to_string()],
            };
            println!("{}", tj.render_yaml());
            0
        }
        "spark" => {
            let name = args.get(1).map(String::as_str).unwrap_or("toksi-cudf-spark");
            let sp = SparkApplicationSpec {
                name: name.to_string(),
                main_class: "org.toksi.RapidsJob".to_string(),
                executors: 2,
                gpu_enabled: true,
                cudf_rapids_enabled: true,
            };
            println!("{}", sp.render_manifest());
            0
        }
        "lan" => {
            let mut mesh = LanPairMesh::new();
            if let Ok(a) = crate::db::shared_db()
                && let Ok(db) = a.lock()
                && let Ok(recs) = db.list_cluster_nodes()
            {
                for r in recs {
                    let node = ClusterNode::from_db_record(&r);
                    if node.backend == ClusterBackendKind::LanPair {
                        mesh.register_peer(LanPairPeer {
                            node_id: node.node_id,
                            endpoint: node.endpoint,
                            vram_gb: node.vram_gb,
                            health: node.health,
                            active_models: Vec::new(),
                            latency_ms: 10,
                        });
                    }
                }
            }
            println!("{}", serde_json::to_string_pretty(&mesh).unwrap_or_default());
            0
        }
        "kompose" => {
            let path = args
                .get(1)
                .map(String::as_str)
                .unwrap_or("docker/compose.test.yaml");
            let Ok(raw) = std::fs::read_to_string(Path::new(path)) else {
                eprintln!("Compose file not found: {path}");
                return 1;
            };
            match parse_compose_yaml(&raw) {
                Ok(spec) => {
                    println!("{}", kompose_translate(&spec));
                    0
                }
                Err(e) => {
                    eprintln!("kompose parse error: {e}");
                    1
                }
            }
        }
        _ => {
            eprintln!(
                "Usage: toksi cluster <status|nodes|readiness|nvsentinel|recipe|kubeflow|spark|lan|kompose> [args...]"
            );
            2
        }
    }
}
