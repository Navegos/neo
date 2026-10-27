// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/tokless/src/projectmem/heatmap.rs
// created: 2026-10-27

use std::collections::{HashMap, HashSet};
use std::path::Path;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};

use super::schema::{location_to_file, Event};
use super::store::read_events;

pub const HOURS_PER_FAILED_APPROACH: f64 = 0.5;
pub const HOURS_PER_FIX_WITH_CONTEXT: f64 = 0.25;
pub const HOURS_PER_DECISION: f64 = 0.15;
pub const HOURS_PER_CHURN_FLAG: f64 = 0.75;
pub const TOKENS_PER_FAILED_APPROACH: i64 = 3000;
pub const TOKENS_PER_CONTEXT_REBUILD: i64 = 2000;
pub const TOKENS_PER_DECISION: i64 = 500;
pub const USD_PER_MILLION_TOKENS: f64 = 10.0;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoreComponents {
    pub failed_approaches: i64,
    pub successful_attempts: i64,
    pub decisions_documented: i64,
    pub fixes_with_context: i64,
    pub notes_count: i64,
    pub files_with_gotchas: i64,
    pub high_churn_files: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoreCapture {
    pub auto_captured: i64,
    pub manual: i64,
    pub total: i64,
    pub auto_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoreValue {
    pub debugging_hours_saved: f64,
    pub tokens_saved: i64,
    pub usd_saved: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoreBreakdown {
    pub failed_knowledge: f64,
    pub decisions: f64,
    pub fixes: f64,
    pub file_coverage: f64,
    pub general_coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoreResult {
    pub score: i32,
    pub grade: String,
    pub components: ScoreComponents,
    pub capture: ScoreCapture,
    pub value: ScoreValue,
    pub score_breakdown: ScoreBreakdown,
    #[serde(default, rename = "tokens_saved")]
    pub tokens_saved: i64,
    #[serde(default, rename = "usd_saved")]
    pub usd_saved: f64,
    #[serde(default, rename = "debugging_hours_saved")]
    pub debugging_hours_saved: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileFailureStats {
    pub file: String,
    pub failed_attempts: usize,
    pub successful_attempts: usize,
    pub open_issues: usize,
    pub resolved_issues: usize,
    pub decisions: usize,
    pub notes: usize,
    pub total_events: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeatmapResult {
    pub score: ScoreResult,
    pub top_hotspots: Vec<FileFailureStats>,
    pub directory_clusters: HashMap<String, usize>,
}

pub fn calculate_score(events: &[Event], since_days: Option<i64>) -> ScoreResult {
    let cutoff = since_days.map(|days| Utc::now() - ChronoDuration::days(days));

    let filtered: Vec<&Event> = events
        .iter()
        .filter(|e| {
            if let Some(c) = cutoff {
                if let Ok(dt) = DateTime::parse_from_rfc3339(&e.timestamp) {
                    return dt.with_timezone(&Utc) >= c;
                }
            }
            true
        })
        .collect();

    let mut failed_approaches = 0i64;
    let mut successful_attempts = 0i64;
    let mut fixes_with_context = 0i64;
    let mut decisions_documented = 0i64;
    let mut notes_count = 0i64;
    let mut auto_captured_count = 0i64;
    let mut manual_count = 0i64;

    let mut files_with_failures: HashSet<String> = HashSet::new();
    let mut files_with_gotchas: HashSet<String> = HashSet::new();
    let mut file_event_counts: HashMap<String, i64> = HashMap::new();

    for e in &filtered {
        if e.auto_captured {
            auto_captured_count += 1;
        } else {
            manual_count += 1;
        }

        let mut event_files = e.files.clone();
        if let Some(f) = location_to_file(e.location.as_deref()) {
            event_files.push(f);
        }

        for f in &event_files {
            *file_event_counts.entry(f.clone()).or_insert(0) += 1;
            files_with_gotchas.insert(f.clone());
        }

        match e.event_type.as_str() {
            "attempt" => match e.outcome.as_deref() {
                Some("failed") => {
                    failed_approaches += 1;
                    for f in &event_files {
                        files_with_failures.insert(f.clone());
                    }
                }
                Some("worked") => {
                    successful_attempts += 1;
                }
                _ => {}
            },
            "fix" => {
                fixes_with_context += 1;
            }
            "decision" => {
                decisions_documented += 1;
            }
            "note" => {
                notes_count += 1;
            }
            _ => {}
        }
    }

    let high_churn_files = file_event_counts.values().filter(|&&c| c >= 4).count() as i64;

    let mut debugging_hours_saved = (failed_approaches as f64 * HOURS_PER_FAILED_APPROACH)
        + (fixes_with_context as f64 * HOURS_PER_FIX_WITH_CONTEXT)
        + (decisions_documented as f64 * HOURS_PER_DECISION);
    debugging_hours_saved += high_churn_files as f64 * HOURS_PER_CHURN_FLAG;
    debugging_hours_saved = (debugging_hours_saved * 10.0).round() / 10.0;

    let tokens_saved = (failed_approaches * TOKENS_PER_FAILED_APPROACH)
        + (fixes_with_context * TOKENS_PER_CONTEXT_REBUILD)
        + (decisions_documented * TOKENS_PER_DECISION)
        + (auto_captured_count * TOKENS_PER_CONTEXT_REBUILD);

    let usd_saved = ((tokens_saved as f64 / 1_000_000.0) * USD_PER_MILLION_TOKENS * 100.0).round() / 100.0;

    let failed_score = ((failed_approaches * 2) as f64).min(20.0);
    let decision_score = ((decisions_documented as f64 * 1.5)).min(20.0);
    let fix_score = ((fixes_with_context * 2) as f64).min(20.0);
    let gotcha_score = ((files_with_gotchas.len() as f64 * 0.5)).min(20.0);
    let coverage_score = ((notes_count as f64 * 0.5)
        + (auto_captured_count as f64 * 0.3)
        + (successful_attempts as f64 * 1.0))
        .min(20.0);

    let total_score = (failed_score + decision_score + fix_score + gotcha_score + coverage_score)
        .round()
        .min(100.0) as i32;

    let grade = if total_score >= 90 {
        "A+".to_string()
    } else if total_score >= 80 {
        "A".to_string()
    } else if total_score >= 60 {
        "B".to_string()
    } else if total_score >= 40 {
        "C".to_string()
    } else if total_score >= 20 {
        "D".to_string()
    } else {
        "F".to_string()
    };

    let total_evs = filtered.len() as i64;
    let auto_rate = if total_evs > 0 {
        ((auto_captured_count as f64 / total_evs as f64) * 1000.0).round() / 10.0
    } else {
        0.0
    };

    ScoreResult {
        score: total_score,
        grade,
        components: ScoreComponents {
            failed_approaches,
            successful_attempts,
            decisions_documented,
            fixes_with_context,
            notes_count,
            files_with_gotchas: files_with_gotchas.len() as i64,
            high_churn_files,
        },
        capture: ScoreCapture {
            auto_captured: auto_captured_count,
            manual: manual_count,
            total: total_evs,
            auto_rate,
        },
        value: ScoreValue {
            debugging_hours_saved,
            tokens_saved,
            usd_saved,
        },
        score_breakdown: ScoreBreakdown {
            failed_knowledge: (failed_score * 10.0).round() / 10.0,
            decisions: (decision_score * 10.0).round() / 10.0,
            fixes: (fix_score * 10.0).round() / 10.0,
            file_coverage: (gotcha_score * 10.0).round() / 10.0,
            general_coverage: (coverage_score * 10.0).round() / 10.0,
        },
        tokens_saved,
        usd_saved,
        debugging_hours_saved,
    }
}

pub fn generate_heatmap(events: &[Event]) -> HeatmapResult {
    let score = calculate_score(events, None);

    let mut file_map: HashMap<String, FileFailureStats> = HashMap::new();
    let mut dir_clusters: HashMap<String, usize> = HashMap::new();

    let open_issue_ids: HashSet<String> = {
        let closed: HashSet<String> = events
            .iter()
            .filter(|e| e.event_type == "fix" && e.issue_id.is_some())
            .filter_map(|e| e.issue_id.clone())
            .collect();
        events
            .iter()
            .filter(|e| e.event_type == "issue" && e.issue_id.is_some())
            .filter_map(|e| e.issue_id.clone())
            .filter(|iid| !closed.contains(iid))
            .collect()
    };

    for e in events {
        let mut target_files = e.files.clone();
        if let Some(f) = location_to_file(e.location.as_deref()) {
            if !target_files.contains(&f) {
                target_files.push(f);
            }
        }

        for f in target_files {
            let stats = file_map.entry(f.clone()).or_insert_with(|| FileFailureStats {
                file: f.clone(),
                ..Default::default()
            });

            stats.total_events += 1;
            match e.event_type.as_str() {
                "attempt" => match e.outcome.as_deref() {
                    Some("failed") => stats.failed_attempts += 1,
                    Some("worked") => stats.successful_attempts += 1,
                    _ => {}
                },
                "issue" => {
                    if let Some(ref iid) = e.issue_id {
                        if open_issue_ids.contains(iid) {
                            stats.open_issues += 1;
                        } else {
                            stats.resolved_issues += 1;
                        }
                    }
                }
                "decision" => stats.decisions += 1,
                "note" => stats.notes += 1,
                _ => {}
            }

            let p = Path::new(&f);
            if let Some(parent) = p.parent() {
                let dir_str = parent.to_string_lossy().replace('\\', "/");
                if !dir_str.is_empty() && dir_str != "." {
                    *dir_clusters.entry(dir_str).or_insert(0) += 1;
                }
            }
        }
    }

    let mut hotspots: Vec<FileFailureStats> = file_map.into_values().collect();
    hotspots.sort_by(|a, b| {
        b.failed_attempts
            .cmp(&a.failed_attempts)
            .then_with(|| b.open_issues.cmp(&a.open_issues))
            .then_with(|| b.total_events.cmp(&a.total_events))
    });

    HeatmapResult {
        score,
        top_hotspots: hotspots,
        directory_clusters: dir_clusters,
    }
}

pub fn format_score_text(res: &ScoreResult) -> String {
    let mut out = String::new();
    out.push_str("========================================\n");
    out.push_str("  PROJECTMEM TOKEN ROI DASHBOARD\n");
    out.push_str("========================================\n");
    out.push_str(&format!("  Score:                 {}/100 ({})\n", res.score, res.grade));
    out.push_str(&format!("  Total Tokens Saved:    {}\n", res.tokens_saved));
    out.push_str(&format!("  Estimated USD Saved:   ${:.2}\n", res.usd_saved));
    out.push_str(&format!("  Debug Hours Saved:     ~{:.1}h\n", res.debugging_hours_saved));
    out.push_str("----------------------------------------\n");
    out.push_str(&format!("  Failed approaches:   {}\n", res.components.failed_approaches));
    out.push_str(&format!("  Decisions documented:{}\n", res.components.decisions_documented));
    out.push_str(&format!("  Fixes with context:  {}\n", res.components.fixes_with_context));
    out.push_str(&format!("  Files with gotchas:  {}\n", res.components.files_with_gotchas));
    out.push_str(&format!("  High churn files:    {}\n", res.components.high_churn_files));
    out.push_str("========================================\n");
    out
}

pub fn compute_stats_for_root(root: Option<&Path>) -> Result<ScoreResult, String> {
    let events = read_events(root)?;
    Ok(calculate_score(&events, None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_score_and_heatmap() {
        let mut e1 = Event::new("issue", "Bug 1");
        e1.issue_id = Some("0001".into());
        e1.location = Some("src/auth.rs:10".into());

        let mut e2 = Event::new("attempt", "Failed approach");
        e2.issue_id = Some("0001".into());
        e2.outcome = Some("failed".into());
        e2.location = Some("src/auth.rs:10".into());

        let mut e3 = Event::new("fix", "Working fix");
        e3.issue_id = Some("0001".into());
        e3.location = Some("src/auth.rs:10".into());

        let mut e4 = Event::new("decision", "Architecture decision");
        e4.location = Some("src/auth.rs:10".into());

        let events = vec![e1, e2, e3, e4];
        let score = calculate_score(&events, None);
        assert_eq!(score.components.failed_approaches, 1);
        assert_eq!(score.components.fixes_with_context, 1);
        assert_eq!(score.components.decisions_documented, 1);
        assert!(score.tokens_saved > 0);
        assert!(score.debugging_hours_saved > 0.0);

        let heatmap = generate_heatmap(&events);
        assert_eq!(heatmap.top_hotspots.len(), 1);
        assert_eq!(heatmap.top_hotspots[0].file, "src/auth.rs");
        assert_eq!(heatmap.top_hotspots[0].failed_attempts, 1);
    }
}

