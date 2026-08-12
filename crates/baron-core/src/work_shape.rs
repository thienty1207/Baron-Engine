use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::authority::{classify_request, RequestAuthority};
use crate::risk::{classify_risk, RiskLane};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DurabilityNeed {
    None,
    Ephemeral,
    Durable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JudgmentNeed {
    None,
    UserConfirmation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleDepth {
    ReadOnly,
    Focused,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkShape {
    ReadOnly,
    FocusedEphemeral,
    Durable,
    RequiresConfirmation,
}

impl WorkShape {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::FocusedEphemeral => "focused_ephemeral",
            Self::Durable => "durable",
            Self::RequiresConfirmation => "requires_confirmation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkShapeDecision {
    pub authority: RequestAuthority,
    pub risk: RiskLane,
    pub durability: DurabilityNeed,
    pub judgment: JudgmentNeed,
    pub lifecycle: LifecycleDepth,
    pub proof_required: bool,
    pub work_shape: WorkShape,
    pub reasons: Vec<String>,
    pub next_action: String,
}

/// Decide lifecycle depth without writing plans, Harness records, proof, or
/// trace state. This is deliberately conservative around ambiguity and risk.
pub fn decide_work_shape(_repo_root: impl AsRef<Path>, task: &str) -> Result<WorkShapeDecision> {
    let task = task.trim();
    let authority = classify_request(task).authority;
    let risk = classify_risk(task);
    let lower = task.to_lowercase();
    let durable_signal = contains_any(
        &lower,
        &[
            "multi-session",
            "multi session",
            "coordinate",
            "coordinat",
            "handoff",
            "recovery",
            "resume",
            "interrupt",
            "migration",
            "migrate",
            "rollout",
            "release",
            "deploy",
            "architecture",
            "refactor",
            "across projects",
            "nhiều phiên",
            "phối hợp",
            "khôi phục",
            "triển khai",
            "kiến trúc",
        ],
    );
    let judgment_signal = contains_any(
        &lower,
        &[
            "ambiguous",
            "unclear",
            "choose",
            "decide",
            "policy",
            "which approach",
            "should we",
            "quyết định",
            "chưa rõ",
            "chọn",
            "chính sách",
        ],
    );

    let (durability, judgment, lifecycle, proof_required, work_shape, next_action) = match authority
    {
        RequestAuthority::ReadOnly => (
            DurabilityNeed::None,
            JudgmentNeed::None,
            LifecycleDepth::ReadOnly,
            false,
            WorkShape::ReadOnly,
            "Inspect and answer without writing Baron lifecycle state.".to_string(),
        ),
        RequestAuthority::Ambiguous => (
            DurabilityNeed::None,
            JudgmentNeed::UserConfirmation,
            LifecycleDepth::ReadOnly,
            false,
            WorkShape::RequiresConfirmation,
            "Ask one high-value question before mutation or durable state.".to_string(),
        ),
        RequestAuthority::Change if risk == RiskLane::High || durable_signal => (
            DurabilityNeed::Durable,
            if judgment_signal {
                JudgmentNeed::UserConfirmation
            } else {
                JudgmentNeed::None
            },
            LifecycleDepth::Full,
            true,
            if judgment_signal {
                WorkShape::RequiresConfirmation
            } else {
                WorkShape::Durable
            },
            "Use confirmed intent, durable plan/recovery, mandatory gates, proof, and trace."
                .to_string(),
        ),
        RequestAuthority::Change => (
            DurabilityNeed::Ephemeral,
            if judgment_signal {
                JudgmentNeed::UserConfirmation
            } else {
                JudgmentNeed::None
            },
            if judgment_signal {
                LifecycleDepth::ReadOnly
            } else {
                LifecycleDepth::Focused
            },
            true,
            if judgment_signal {
                WorkShape::RequiresConfirmation
            } else {
                WorkShape::FocusedEphemeral
            },
            if judgment_signal {
                "Resolve the material choice before applying a focused change.".to_string()
            } else {
                "Use a focused change path and record only task-specific proof.".to_string()
            },
        ),
    };

    let mut reasons = Vec::new();
    reasons.push(format!("authority classified as `{}`", authority.as_str()));
    reasons.push(format!("risk classified as `{}`", risk.as_str()));
    if durable_signal {
        reasons.push("durable or recovery-oriented work signal detected".to_string());
    }
    if judgment_signal {
        reasons.push("material product or policy choice remains open".to_string());
    }
    if reasons.len() == 2 {
        reasons.push(
            "no multi-session, coordination, or unresolved-choice signal detected".to_string(),
        );
    }

    Ok(WorkShapeDecision {
        authority,
        risk,
        durability,
        judgment,
        lifecycle,
        proof_required,
        work_shape,
        reasons,
        next_action,
    })
}

fn contains_any(value: &str, terms: &[&str]) -> bool {
    terms.iter().any(|term| value.contains(term))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_only_request_has_no_lifecycle_writes() {
        let decision =
            decide_work_shape(".", "review the current README and explain the status").unwrap();
        assert_eq!(decision.work_shape, WorkShape::ReadOnly);
        assert_eq!(decision.durability, DurabilityNeed::None);
        assert!(!decision.proof_required);
    }

    #[test]
    fn bounded_change_uses_focused_ephemeral_path() {
        let decision = decide_work_shape(".", "fix a typo in the README").unwrap();
        assert_eq!(decision.work_shape, WorkShape::FocusedEphemeral);
        assert_eq!(decision.lifecycle, LifecycleDepth::Focused);
        assert!(decision.proof_required);
    }

    #[test]
    fn risky_short_change_keeps_full_safety_path() {
        let decision = decide_work_shape(".", "fix the login permission check").unwrap();
        assert_eq!(decision.work_shape, WorkShape::Durable);
        assert_eq!(decision.lifecycle, LifecycleDepth::Full);
        assert!(decision.proof_required);
    }

    #[test]
    fn vietnamese_ambiguity_requires_confirmation() {
        let decision = decide_work_shape(".", "chọn chính sách phân quyền phù hợp").unwrap();
        assert_eq!(decision.work_shape, WorkShape::RequiresConfirmation);
        assert_eq!(decision.judgment, JudgmentNeed::UserConfirmation);
    }
}
