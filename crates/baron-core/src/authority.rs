use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestAuthority {
    ReadOnly,
    Change,
    Ambiguous,
}

impl RequestAuthority {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::Change => "change",
            Self::Ambiguous => "ambiguous",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityDecision {
    pub authority: RequestAuthority,
    pub mutation_allowed: bool,
    pub matched_change_terms: Vec<String>,
    pub matched_read_only_terms: Vec<String>,
    pub reason: String,
    pub next_action: String,
}

impl AuthorityDecision {
    pub fn mutation_allowed(&self) -> bool {
        self.mutation_allowed
    }
}

const CHANGE_TERMS: &[&str] = &[
    "implement",
    "implemented",
    "implementing",
    "build",
    "create",
    "add",
    "update",
    "modify",
    "edit",
    "fix",
    "fixes",
    "repair",
    "refactor",
    "remove",
    "delete",
    "migrate",
    "migration",
    "apply",
    "write",
    "change",
    "upgrade",
    "install",
    "triển khai",
    "xây",
    "tạo",
    "thêm",
    "cập nhật",
    "chỉnh sửa",
    "sửa",
    "xóa",
    "xoá",
    "loại bỏ",
    "nâng cấp",
    "áp dụng",
    "viết",
    "cài",
];

const READ_ONLY_TERMS: &[&str] = &[
    "answer",
    "explain",
    "review",
    "diagnose",
    "analyze",
    "inspect",
    "report",
    "status",
    "compare",
    "summarize",
    "plan",
    "question",
    "tell",
    "show",
    "check",
    "evaluate",
    "audit",
    "search",
    "find",
    "giải thích",
    "đánh giá",
    "kiểm tra",
    "chẩn đoán",
    "báo cáo",
    "trạng thái",
    "so sánh",
    "tóm tắt",
    "lên plan",
    "kế hoạch",
    "tìm",
    "đọc",
    "hỏi",
    "trả lời",
];

const NEGATED_CHANGE_PHRASES: &[&str] = &[
    "do not implement",
    "don't implement",
    "without implementing",
    "without changing",
    "no code changes",
    "no changes",
    "chưa implement",
    "không implement",
    "chưa triển khai",
    "không triển khai",
    "chưa sửa",
    "không sửa",
    "chỉ plan",
    "plan thôi",
];

pub fn classify_request(request: &str) -> AuthorityDecision {
    let normalized = normalize(request);
    let mut change_input = normalized.clone();
    for phrase in NEGATED_CHANGE_PHRASES {
        change_input = change_input.replace(phrase, " ");
    }

    let matched_change_terms = matched_terms(&change_input, CHANGE_TERMS);
    let matched_read_only_terms = matched_terms(&normalized, READ_ONLY_TERMS);

    let (authority, reason, next_action) = if !matched_change_terms.is_empty() {
        (
            RequestAuthority::Change,
            "The requested outcome explicitly changes repository or Baron state.".to_string(),
            "Run the normal Baron change workflow before durable edits.".to_string(),
        )
    } else if !matched_read_only_terms.is_empty() {
        (
            RequestAuthority::ReadOnly,
            "The requested outcome is inspection or advice only.".to_string(),
            "Inspect only the evidence needed to answer; do not mutate durable Baron state."
                .to_string(),
        )
    } else {
        (
            RequestAuthority::Ambiguous,
            "The requested outcome does not grant clear mutation authority.".to_string(),
            "Remain read-only and ask for explicit change authority before durable writes."
                .to_string(),
        )
    };

    AuthorityDecision {
        authority,
        mutation_allowed: authority == RequestAuthority::Change,
        matched_change_terms,
        matched_read_only_terms,
        reason,
        next_action,
    }
}

fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn matched_terms(input: &str, terms: &[&str]) -> Vec<String> {
    let tokens = input
        .split(|character: char| !character.is_alphanumeric() && character != '_')
        .filter(|token| !token.is_empty())
        .collect::<BTreeSet<_>>();
    terms
        .iter()
        .filter(|term| {
            if term.contains(' ') {
                input.contains(**term)
            } else {
                tokens.contains(**term)
            }
        })
        .map(|term| (*term).to_string())
        .collect()
}
