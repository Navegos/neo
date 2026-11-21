// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/security/cel_decision.rs
// created: 2026-11-21

//! Bounded fail-closed CEL policy decision evaluator across 5 presets:
//! `strict | balanced | low-noise | quiet | permissive`.

#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScanPreset {
    Strict,
    Balanced,
    LowNoise,
    Quiet,
    Permissive,
}

impl ScanPreset {
    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "strict" => Self::Strict,
            "low-noise" | "low_noise" => Self::LowNoise,
            "quiet" => Self::Quiet,
            "permissive" => Self::Permissive,
            _ => Self::Balanced,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Strict => "strict",
            Self::Balanced => "balanced",
            Self::LowNoise => "low-noise",
            Self::Quiet => "quiet",
            Self::Permissive => "permissive",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CelDecisionVerdict {
    Allow,
    Warn,
    DenyFailClosed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CelEvaluation {
    pub preset: ScanPreset,
    pub verdict: CelDecisionVerdict,
    pub reason: String,
}

/// Evaluates bounded fail-closed CEL policy over finding counts per severity.
pub fn evaluate_cel_policy(
    preset: ScanPreset,
    critical: usize,
    high: usize,
    medium: usize,
    low: usize,
    eval_error: Option<&str>,
) -> CelEvaluation {
    if let Some(err) = eval_error {
        return CelEvaluation {
            preset,
            verdict: CelDecisionVerdict::DenyFailClosed,
            reason: format!("CEL fail-closed on evaluation error: {err}"),
        };
    }

    match preset {
        ScanPreset::Strict => {
            if critical + high + medium > 0 {
                CelEvaluation {
                    preset,
                    verdict: CelDecisionVerdict::DenyFailClosed,
                    reason: format!("strict preset blocks critical={critical}, high={high}, medium={medium}"),
                }
            } else if low > 0 {
                CelEvaluation {
                    preset,
                    verdict: CelDecisionVerdict::Warn,
                    reason: format!("strict preset warns on low={low}"),
                }
            } else {
                CelEvaluation {
                    preset,
                    verdict: CelDecisionVerdict::Allow,
                    reason: "strict preset passed".to_string(),
                }
            }
        }
        ScanPreset::Balanced => {
            if critical > 0 || high > 0 {
                CelEvaluation {
                    preset,
                    verdict: CelDecisionVerdict::DenyFailClosed,
                    reason: format!("balanced preset blocks critical={critical}, high={high}"),
                }
            } else if medium > 0 {
                CelEvaluation {
                    preset,
                    verdict: CelDecisionVerdict::Warn,
                    reason: format!("balanced preset warns on medium={medium}"),
                }
            } else {
                CelEvaluation {
                    preset,
                    verdict: CelDecisionVerdict::Allow,
                    reason: "balanced preset passed".to_string(),
                }
            }
        }
        ScanPreset::LowNoise => {
            if critical > 0 {
                CelEvaluation {
                    preset,
                    verdict: CelDecisionVerdict::DenyFailClosed,
                    reason: format!("low-noise preset blocks critical={critical}"),
                }
            } else if high > 0 {
                CelEvaluation {
                    preset,
                    verdict: CelDecisionVerdict::Warn,
                    reason: format!("low-noise preset warns on high={high}"),
                }
            } else {
                CelEvaluation {
                    preset,
                    verdict: CelDecisionVerdict::Allow,
                    reason: "low-noise preset passed".to_string(),
                }
            }
        }
        ScanPreset::Quiet => {
            if critical > 0 {
                CelEvaluation {
                    preset,
                    verdict: CelDecisionVerdict::DenyFailClosed,
                    reason: format!("quiet preset blocks critical={critical}"),
                }
            } else {
                CelEvaluation {
                    preset,
                    verdict: CelDecisionVerdict::Allow,
                    reason: "quiet preset passed".to_string(),
                }
            }
        }
        ScanPreset::Permissive => {
            if critical > 0 {
                CelEvaluation {
                    preset,
                    verdict: CelDecisionVerdict::Warn,
                    reason: format!("permissive preset warns on critical={critical}"),
                }
            } else {
                CelEvaluation {
                    preset,
                    verdict: CelDecisionVerdict::Allow,
                    reason: "permissive preset passed".to_string(),
                }
            }
        }
    }
}
