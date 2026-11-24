// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/cluster/kubernetes.rs
// created: 2026-11-24

//! Kubernetes, Minikube, Helm, Kops, Kompose, `aicr`, `knavigator`, `autoscaler`,
//! and `node-problem-detector` contracts (`PATH`-checked with zero bundled binaries).

#![deny(unsafe_code)]

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::agent_runtime::sandbox::ComposeSpec;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct K8sToolingStatus {
    pub kubectl: bool,
    pub helm: bool,
    pub minikube: bool,
    pub kops: bool,
}

static K8S_TOOLING_CACHE: OnceLock<Mutex<Option<(Instant, K8sToolingStatus)>>> = OnceLock::new();

/// Probes host PATH for external Kubernetes CLI tooling gracefully (cached 30s to keep slash/UI calls <1ms).
pub fn probe_k8s_tooling() -> K8sToolingStatus {
    let cache = K8S_TOOLING_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = cache.lock()
        && let Some((ts, ref cached)) = *guard
        && ts.elapsed() < Duration::from_secs(30)
    {
        return cached.clone();
    }
    let fresh = K8sToolingStatus {
        kubectl: which::which("kubectl").is_ok(),
        helm: which::which("helm").is_ok(),
        minikube: which::which("minikube").is_ok(),
        kops: which::which("kops").is_ok(),
    };
    if let Ok(mut guard) = cache.lock() {
        *guard = Some((Instant::now(), fresh.clone()));
    }
    fresh
}

/// Pure-Rust `kompose`-style Compose-to-Kubernetes Deployment YAML translator.
pub fn kompose_translate(spec: &ComposeSpec) -> String {
    let mut out = String::new();
    for svc in &spec.services {
        let img = if svc.image.trim().is_empty() {
            "toksi-test:local"
        } else {
            svc.image.trim()
        };
        out.push_str(&format!(
            "---\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: {}\nspec:\n  replicas: 1\n  selector:\n    matchLabels:\n      app: {}\n  template:\n    metadata:\n      labels:\n        app: {}\n    spec:\n      containers:\n        - name: {}\n          image: {}\n",
            svc.name, svc.name, svc.name, svc.name, img
        ));
        if !svc.security_opt.is_empty() {
            out.push_str("          securityContext:\n            allowPrivilegeEscalation: false\n");
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AicrClusterRecipe {
    pub name: String,
    pub profile: String,
    pub gpu_count: u32,
    pub nvidia_driver_version: String,
    pub rdma_enabled: bool,
}

pub fn render_aicr_recipe(name: &str, gpu_count: u32, profile: &str) -> AicrClusterRecipe {
    AicrClusterRecipe {
        name: name.to_string(),
        profile: if profile.trim().is_empty() {
            "h100-sxm".to_string()
        } else {
            profile.to_string()
        },
        gpu_count: gpu_count.max(1),
        nvidia_driver_version: "570.86".to_string(),
        rdma_enabled: gpu_count >= 2,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnavigatorScalePlan {
    pub min_nodes: u32,
    pub max_nodes: u32,
    pub target_gpu_util_pct: u32,
    pub node_problem_detector_enabled: bool,
}

impl Default for KnavigatorScalePlan {
    fn default() -> Self {
        Self {
            min_nodes: 1,
            max_nodes: 16,
            target_gpu_util_pct: 75,
            node_problem_detector_enabled: true,
        }
    }
}
