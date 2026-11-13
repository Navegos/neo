// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/icm/schema.rs
// created: 2026-10-28
// lastModified: 2026-11-13

#![deny(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IcmError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Rusqlite error: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("Security violation: {0}")]
    Security(String),
    #[error("Repository not found: {0}")]
    RepoNotFound(String),
}

pub type IcmResult<T> = Result<T, IcmError>;

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
    type Err = IcmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "critical" => Ok(Self::Critical),
            "high" => Ok(Self::High),
            "medium" => Ok(Self::Medium),
            "low" => Ok(Self::Low),
            _ => Err(IcmError::InvalidInput(format!("invalid importance level: {s}"))),
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
    pub subproject_id: Option<String>,
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
    pub subproject_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memoir {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
    pub consolidation_threshold: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: String,
    pub memoir_id: String,
    pub name: String,
    pub definition: String,
    pub labels: Vec<String>,
    pub confidence: f32,
    pub revision: u32,
    pub created_at: String,
    pub updated_at: String,
    pub source_memory_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptLink {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation: String,
    pub weight: f32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub id: String,
    pub topic: String,
    pub context: String,
    pub predicted: String,
    pub corrected: String,
    pub reason: Option<String>,
    pub source: String,
    pub created_at: String,
    pub applied_count: u32,
    pub subproject_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub agent: String,
    pub project: Option<String>,
    pub started_at: String,
    pub updated_at: String,
    pub metadata: String,
    pub subproject_id: Option<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeWindow {
    pub fn contains(&self, t: DateTime<Utc>) -> bool {
        t >= self.start && t < self.end
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FusedHit {
    pub id: String,
    pub score: f32,
    pub ranks: Vec<Option<u32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedHit {
    pub memory: Memory,
    pub score: f32,
    pub fts_rank: Option<u32>,
    pub time_rank: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicHealth {
    pub topic: String,
    pub count: usize,
    pub avg_weight: f32,
    pub critical_count: usize,
    pub stale_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcmStats {
    pub total_memories: usize,
    pub total_topics: usize,
    pub total_facts: usize,
    pub total_memoirs: usize,
    pub total_concepts: usize,
    pub total_feedback: usize,
    pub total_transcripts: usize,
    pub avg_weight: f32,
}
