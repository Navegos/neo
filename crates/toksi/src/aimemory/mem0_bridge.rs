// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/aimemory/mem0_bridge.rs
// created: 2026-11-09
// lastModified: 2026-11-09

use std::collections::HashMap;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::util::paths::normalize_path;

#[derive(Error, Debug)]
pub enum Mem0Error {
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Lock poisoned")]
    LockPoisoned,
    #[error("Fact not found: '{0}'")]
    FactNotFound(String),
}

/// Category of extracted long-term entity fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactCategory {
    UserPreference,
    ProjectDecision,
    TechnicalConstraint,
    WorkflowGuideline,
    EnvironmentConfig,
}

impl FactCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserPreference => "user_preference",
            Self::ProjectDecision => "project_decision",
            Self::TechnicalConstraint => "technical_constraint",
            Self::WorkflowGuideline => "workflow_guideline",
            Self::EnvironmentConfig => "environment_config",
        }
    }
}

/// A persistent extracted entity-fact record conforming to Mem0 schemas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityFact {
    pub id: String,
    pub entity: String,       // e.g. "user", "project", "database", "rust_compiler"
    pub fact: String,         // e.g. "User prefers Rust edition 2024 with zero unwrap policy"
    pub category: FactCategory,
    pub confidence: f32,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub source_ref: Option<String>,
}

/// Memory operation action emitted during conversational ingestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryAction {
    Add(EntityFact),
    Update { id: String, new_fact: String },
    Delete { id: String },
    Noop,
}

/// In-memory Mem0 bridge engine providing personalized entity-fact extraction and querying.
pub struct Mem0BridgeEngine {
    facts: RwLock<HashMap<String, EntityFact>>,
}

impl Default for Mem0BridgeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Mem0BridgeEngine {
    pub fn new() -> Self {
        Self {
            facts: RwLock::new(HashMap::new()),
        }
    }

    /// Ingests a new fact or updates an existing fact if an exact entity/concept match exists.
    pub fn add_fact(
        &self,
        entity: &str,
        fact_text: &str,
        category: FactCategory,
        confidence: f32,
        source_ref: Option<&str>,
    ) -> Result<EntityFact, Mem0Error> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let id = format!("fact_{:x}", md5_hash(&format!("{}:{}", entity, fact_text)));

        let fact = EntityFact {
            id: id.clone(),
            entity: entity.to_string(),
            fact: fact_text.to_string(),
            category,
            confidence: confidence.clamp(0.0, 1.0),
            created_at_ms: now,
            updated_at_ms: now,
            source_ref: source_ref.map(|s| normalize_path(s)),
        };

        let mut lock = self.facts.write().map_err(|_| Mem0Error::LockPoisoned)?;
        lock.insert(id, fact.clone());
        Ok(fact)
    }

    /// Searches stored facts by keyword matching against entity and fact text.
    pub fn search(&self, query: &str, category: Option<FactCategory>) -> Result<Vec<EntityFact>, Mem0Error> {
        let q_lower = query.to_ascii_lowercase();
        let lock = self.facts.read().map_err(|_| Mem0Error::LockPoisoned)?;

        let mut results: Vec<EntityFact> = lock
            .values()
            .filter(|f| {
                if let Some(cat) = category {
                    if f.category != cat {
                        return false;
                    }
                }
                if q_lower.is_empty() {
                    return true;
                }
                f.entity.to_ascii_lowercase().contains(&q_lower)
                    || f.fact.to_ascii_lowercase().contains(&q_lower)
            })
            .cloned()
            .collect();

        results.sort_by(|a, b| b.updated_at_ms.cmp(&a.updated_at_ms));
        Ok(results)
    }

    /// Deletes a fact by id.
    pub fn delete_fact(&self, id: &str) -> Result<bool, Mem0Error> {
        let mut lock = self.facts.write().map_err(|_| Mem0Error::LockPoisoned)?;
        Ok(lock.remove(id).is_some())
    }

    /// Extracts fact candidates from text interaction lines heuristics.
    pub fn extract_facts_from_turn(&self, text: &str, user_or_agent: &str) -> Vec<MemoryAction> {
        let mut actions = Vec::new();
        let lower = text.to_ascii_lowercase();

        // Heuristics for decisions and preferences
        if lower.contains("prefer ") || lower.contains("always use ") || lower.contains("must use ") {
            let fact_str = text.trim().to_string();
            let fact = EntityFact {
                id: format!("cand_{:x}", md5_hash(&fact_str)),
                entity: user_or_agent.to_string(),
                fact: fact_str,
                category: FactCategory::UserPreference,
                confidence: 0.85,
                created_at_ms: 0,
                updated_at_ms: 0,
                source_ref: None,
            };
            actions.push(MemoryAction::Add(fact));
        } else if lower.contains("decision:") || lower.contains("decided to ") || lower.contains("architecture mandate") {
            let fact_str = text.trim().to_string();
            let fact = EntityFact {
                id: format!("cand_{:x}", md5_hash(&fact_str)),
                entity: "architecture".to_string(),
                fact: fact_str,
                category: FactCategory::ProjectDecision,
                confidence: 0.90,
                created_at_ms: 0,
                updated_at_ms: 0,
                source_ref: None,
            };
            actions.push(MemoryAction::Add(fact));
        }

        actions
    }
}

fn md5_hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mem0_fact_lifecycle() {
        let bridge = Mem0BridgeEngine::new();
        let fact = bridge.add_fact(
            "developer",
            "Prefers dark theme and strict forward-slash paths",
            FactCategory::UserPreference,
            0.95,
            Some("config/settings.json"),
        ).expect("add fact");

        assert_eq!(fact.category, FactCategory::UserPreference);
        assert_eq!(fact.confidence, 0.95);

        let search_res = bridge.search("forward-slash", None).expect("search");
        assert_eq!(search_res.len(), 1);
        assert_eq!(search_res[0].entity, "developer");

        let deleted = bridge.delete_fact(&fact.id).expect("delete");
        assert!(deleted);
        assert_eq!(bridge.search("forward-slash", None).expect("search empty").len(), 0);
    }

    #[test]
    fn test_extract_facts_from_turn() {
        let bridge = Mem0BridgeEngine::new();
        let actions = bridge.extract_facts_from_turn("We must always use Safe Rust Edition 2024", "user");
        assert_eq!(actions.len(), 1);
        if let MemoryAction::Add(fact) = &actions[0] {
            assert_eq!(fact.category, FactCategory::UserPreference);
        } else {
            panic!("Expected MemoryAction::Add");
        }
    }
}
