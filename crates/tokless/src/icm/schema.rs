// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/icm/schema.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Importance {
    Critical,
    High,
    Medium,
    Low,
}

impl Importance {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

impl std::str::FromStr for Importance {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "critical" => Ok(Self::Critical),
            "high" => Ok(Self::High),
            "medium" => Ok(Self::Medium),
            "low" => Ok(Self::Low),
            _ => Err(format!("invalid importance: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_accessed: String,
    pub access_count: u32,
    pub weight: f32,

    pub topic: String,
    pub summary: String,
    pub raw_excerpt: Option<String>,
    pub keywords: Vec<String>,

    pub importance: Importance,
    pub source_type: String,
    pub source_data: Option<String>,

    pub related_ids: Vec<String>,
    pub summary_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub count: usize,
    pub avg_weight: f32,
    pub last_accessed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub id: String,
    pub entity: String,
    pub key: String,
    pub value: String,
    pub source: String,
    pub created_at: String,
    pub superseded_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub agent: String,
    pub project: Option<String>,
    pub started_at: String,
    pub updated_at: String,
    pub metadata: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub tool_name: Option<String>,
    pub tokens: Option<i64>,
    pub ts: String,
    pub metadata: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub memory: Memory,
    pub score: f32,
    pub rank: usize,
}

