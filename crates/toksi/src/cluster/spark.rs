// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/cluster/spark.rs
// created: 2026-11-24

//! Pure-Rust Apache `spark` & `cudf-spark` (RAPIDS GPU) SparkApplication contract.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparkApplicationSpec {
    pub name: String,
    pub main_class: String,
    pub executors: u32,
    pub gpu_enabled: bool,
    pub cudf_rapids_enabled: bool,
}

impl SparkApplicationSpec {
    pub fn render_manifest(&self) -> String {
        let mut rapids = "";
        if self.cudf_rapids_enabled {
            rapids = "    spark.plugins: com.nvidia.spark.SQLPlugin\n    spark.rapids.sql.enabled: \"true\"\n";
        }
        format!(
            "apiVersion: sparkoperator.k8s.io/v1beta2\nkind: SparkApplication\nmetadata:\n  name: {}\nspec:\n  mainClass: {}\n  executor:\n    instances: {}\n    gpu:\n      enabled: {}\n  sparkConf:\n{}",
            self.name,
            self.main_class,
            self.executors.max(1),
            self.gpu_enabled,
            rapids
        )
    }
}
