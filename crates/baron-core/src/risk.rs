use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLane {
    Low,
    Medium,
    High,
}

pub fn classify_risk(task: &str) -> RiskLane {
    let lower = task.to_lowercase();
    let high_terms = [
        "auth",
        "login",
        "password",
        "token",
        "permission",
        "authorization",
        "tenant",
        "rls",
        "payment",
        "billing",
        "subscription",
        "migration",
        "security",
        "secret",
        "upload",
        "external provider",
        "destructive",
        "data loss",
    ];
    if high_terms.iter().any(|term| lower.contains(term)) {
        return RiskLane::High;
    }
    let low_terms = ["docs", "readme", "copy", "typo", "spelling"];
    if low_terms.iter().any(|term| lower.contains(term)) {
        RiskLane::Low
    } else {
        RiskLane::Medium
    }
}

impl RiskLane {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub fn required_trace_tier(self) -> &'static str {
        match self {
            Self::Low => "minimal",
            Self::Medium => "standard",
            Self::High => "detailed",
        }
    }
}
