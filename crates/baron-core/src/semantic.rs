//! Deterministic local semantic ranking for Baron 4.1.
//!
//! The default engine must work offline and without a model account. This
//! module therefore combines BM25-style lexical scoring, bilingual concept
//! expansion, character n-grams, and a small hashed vector space. It is not a
//! claim that a hash vector replaces a trained embedding model; it is a stable,
//! inspectable accelerator with a graceful fallback when optional models are
//! unavailable.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticDocument {
    pub id: String,
    pub title: String,
    pub body: String,
    pub path: String,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticHit {
    pub id: String,
    pub lexical_score: f64,
    pub vector_score: f64,
    pub ngram_score: f64,
    pub rrf_score: f64,
    pub score: f64,
    pub lexical_rank: usize,
    pub vector_rank: usize,
    pub notes: Vec<String>,
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
    pub evidence_channels: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SemanticPolicy42 {
    pub minimum_confidence: f64,
    pub minimum_vector_similarity: f64,
    pub minimum_ngram_similarity: f64,
    pub allow_vector_only: bool,
}

impl Default for SemanticPolicy42 {
    fn default() -> Self {
        Self {
            minimum_confidence: 0.48,
            minimum_vector_similarity: 0.50,
            minimum_ngram_similarity: 0.08,
            allow_vector_only: false,
        }
    }
}

const VECTOR_DIMENSIONS: usize = 64;
const RRF_K: f64 = 60.0;

/// Rank documents with a deterministic BM25/vector/RRF fusion.
pub fn rank_documents(
    query: &str,
    documents: &[SemanticDocument],
    limit: usize,
) -> Vec<SemanticHit> {
    if documents.is_empty() || limit == 0 {
        return Vec::new();
    }
    let query = normalize(query);
    let query_tokens = expanded_tokens(&query);
    if query_tokens.is_empty() {
        return Vec::new();
    }
    // Query-side n-grams are invariant for every document. Computing them once
    // avoids an O(documents * query_length) allocation multiplier on large Wiki
    // and CodeGraph indexes.
    let query_ngrams = ngrams(&query);
    let tokenized = documents
        .iter()
        .map(|document| {
            tokenize(&normalize(&format!(
                "{} {} {} {}",
                document.title,
                document.body,
                document.path,
                document.tags.join(" ")
            )))
        })
        .collect::<Vec<_>>();
    let average_length = tokenized
        .iter()
        .map(|tokens| tokens.len() as f64)
        .sum::<f64>()
        / documents.len() as f64;
    let mut document_frequency = BTreeMap::<String, usize>::new();
    for tokens in &tokenized {
        let unique = tokens.iter().cloned().collect::<BTreeSet<_>>();
        for token in unique {
            *document_frequency.entry(token).or_default() += 1;
        }
    }
    let query_vector = vectorize(&query_tokens.iter().cloned().collect::<Vec<_>>());
    let mut lexical = documents
        .iter()
        .zip(tokenized.iter())
        .enumerate()
        .map(|(index, (document, tokens))| {
            let body = normalize(&document.body);
            let title = normalize(&document.title);
            let lexical_score = bm25(
                &query_tokens,
                tokens,
                documents.len(),
                &document_frequency,
                average_length,
            ) + title_bonus(&query_tokens, &title);
            let vector_score = cosine(&query_vector, &vectorize(tokens));
            let ngram_score = ngram_similarity_with_query(&query_ngrams, &body);
            (index, lexical_score, vector_score, ngram_score)
        })
        .collect::<Vec<_>>();
    let mut lexical_order = lexical.clone();
    lexical_order.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.0.cmp(&right.0))
    });
    let mut vector_order = lexical.clone();
    vector_order.sort_by(|left, right| {
        right
            .2
            .partial_cmp(&left.2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.0.cmp(&right.0))
    });
    let lexical_ranks = lexical_order
        .iter()
        .enumerate()
        .map(|(rank, item)| (item.0, rank + 1))
        .collect::<BTreeMap<_, _>>();
    let vector_ranks = vector_order
        .iter()
        .enumerate()
        .map(|(rank, item)| (item.0, rank + 1))
        .collect::<BTreeMap<_, _>>();
    let mut hits = lexical
        .drain(..)
        .map(|(index, lexical_score, vector_score, ngram_score)| {
            let lexical_rank = lexical_ranks.get(&index).copied().unwrap_or(usize::MAX);
            let vector_rank = vector_ranks.get(&index).copied().unwrap_or(usize::MAX);
            let rrf_score =
                1.0 / (RRF_K + lexical_rank as f64) + 1.0 / (RRF_K + vector_rank as f64);
            let score = lexical_score
                + (vector_score.max(0.0) * 12.0)
                + (ngram_score * 8.0)
                + (rrf_score * 100.0);
            let mut notes = Vec::new();
            if lexical_score > 0.0 {
                notes.push(format!("bm25:{lexical_score:.3}"));
            }
            if vector_score > 0.0 {
                notes.push(format!("vector:{vector_score:.3}"));
            }
            if ngram_score > 0.0 {
                notes.push(format!("ngram:{ngram_score:.3}"));
            }
            notes.push(format!("rrf:{rrf_score:.5}"));
            SemanticHit {
                id: documents[index].id.clone(),
                lexical_score,
                vector_score,
                ngram_score,
                rrf_score,
                score,
                lexical_rank,
                vector_rank,
                notes,
                confidence: 0.0,
                evidence_channels: Vec::new(),
            }
        })
        .collect::<Vec<_>>();
    hits.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.id.cmp(&right.id))
    });
    hits.truncate(limit);
    hits
}

/// Baron 4.2 retrieval policy. Unlike the legacy 4.1 helper, this path never
/// returns a document merely because it received an RRF rank. At least one
/// meaningful evidence channel and a calibrated confidence threshold are
/// required. Callers still perform project/trust filtering before this point.
pub fn rank_documents_v42(
    query: &str,
    documents: &[SemanticDocument],
    limit: usize,
) -> Vec<SemanticHit> {
    rank_documents_v42_with_policy(query, documents, limit, SemanticPolicy42::default())
}

pub fn rank_documents_v42_with_policy(
    query: &str,
    documents: &[SemanticDocument],
    limit: usize,
    policy: SemanticPolicy42,
) -> Vec<SemanticHit> {
    if documents.is_empty() || limit == 0 {
        return Vec::new();
    }
    let normalized_query = normalize(query);
    let query_tokens = expanded_tokens(&normalized_query);
    let mut hits = rank_documents(query, documents, documents.len());
    for hit in &mut hits {
        let mut channels = Vec::new();
        if hit.lexical_score > 0.0 {
            channels.push("lexical".to_string());
        }
        if hit.ngram_score >= policy.minimum_ngram_similarity {
            channels.push("ngram".to_string());
        }
        if hit.vector_score >= policy.minimum_vector_similarity {
            channels.push("dense".to_string());
        }
        let document = documents.iter().find(|document| document.id == hit.id);
        if document.is_some_and(|document| {
            document.tags.iter().any(|tag| tag == "exact")
                || (!normalized_query.is_empty()
                    && normalize(&format!(
                        "{} {} {}",
                        document.title, document.body, document.path
                    ))
                    .contains(&normalized_query))
        }) {
            channels.push("exact".to_string());
        }
        if query_tokens.iter().any(|token| {
            (token.contains('_') || token.chars().any(|character| character.is_ascii_digit()))
                && document.is_some_and(|document| {
                    normalize(&format!(
                        "{} {} {}",
                        document.title, document.body, document.path
                    ))
                    .contains(token)
                })
        }) {
            channels.push("identifier".to_string());
        }
        let coverage = document
            .map(|document| query_term_coverage(&normalized_query, document))
            .unwrap_or_default();
        hit.confidence = calibrated_confidence(hit, &channels, coverage);
        hit.evidence_channels = channels;
        hit.notes
            .push(format!("v42-confidence:{:.3}", hit.confidence));
        if hit.evidence_channels.is_empty() {
            hit.notes
                .push("v42-abstain:no-relevant-evidence".to_string());
        } else if hit.confidence < policy.minimum_confidence {
            hit.notes
                .push("v42-abstain:confidence-below-threshold".to_string());
        }
    }
    hits.retain(|hit| {
        let vector_only = hit.evidence_channels.len() == 1
            && hit
                .evidence_channels
                .iter()
                .any(|channel| channel == "dense");
        !hit.evidence_channels.is_empty()
            && hit.confidence >= policy.minimum_confidence
            && (!vector_only || policy.allow_vector_only)
    });
    // A bounded diversity pass prevents repeated excerpts from crowding out
    // independent evidence. The stable id tie-break keeps clean/warm runs equal.
    let mut seen_paths = BTreeSet::new();
    hits.retain(|hit| {
        let path = documents
            .iter()
            .find(|document| document.id == hit.id)
            .map(|document| format!("{}::{}", document.path, document.title))
            .unwrap_or_default();
        if path.is_empty() {
            return true;
        }
        seen_paths.insert(path)
            || hit
                .evidence_channels
                .iter()
                .any(|channel| channel == "exact")
    });
    hits.sort_by(|left, right| {
        right
            .confidence
            .partial_cmp(&left.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                right
                    .score
                    .partial_cmp(&left.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| left.id.cmp(&right.id))
    });
    hits.truncate(limit);
    hits
}

fn calibrated_confidence(hit: &SemanticHit, channels: &[String], coverage: f64) -> f64 {
    if channels.is_empty() {
        return 0.0;
    }
    let lexical = (hit.lexical_score / 3.0).clamp(0.0, 1.0);
    let dense = (((hit.vector_score + 1.0) / 2.0) * 0.85).clamp(0.0, 1.0);
    let ngram = hit.ngram_score.clamp(0.0, 1.0);
    let exact_bonus = if channels.iter().any(|channel| channel == "exact") {
        0.20
    } else {
        0.0
    };
    (lexical * 0.32
        + dense * 0.22
        + ngram * 0.16
        + coverage * 0.22
        + exact_bonus
        + channels.len() as f64 * 0.04)
        .clamp(0.0, 1.0)
}

fn query_term_coverage(query: &str, document: &SemanticDocument) -> f64 {
    let query_tokens = tokenize(query).into_iter().collect::<BTreeSet<_>>();
    if query_tokens.is_empty() {
        return 0.0;
    }
    let document_tokens = tokenize(&normalize(&format!(
        "{} {} {} {}",
        document.title,
        document.body,
        document.path,
        document.tags.join(" ")
    )))
    .into_iter()
    .collect::<BTreeSet<_>>();
    query_tokens.intersection(&document_tokens).count() as f64 / query_tokens.len() as f64
}

pub fn expand_query(query: &str) -> String {
    let normalized = normalize(query);
    let mut terms = expanded_tokens(&normalized).into_iter().collect::<Vec<_>>();
    terms.sort();
    terms.dedup();
    terms.join(" ")
}

pub fn normalize_query(value: &str) -> String {
    normalize(value)
}

fn bm25(
    query_tokens: &BTreeSet<String>,
    document_tokens: &[String],
    document_count: usize,
    document_frequency: &BTreeMap<String, usize>,
    average_length: f64,
) -> f64 {
    let mut frequencies = BTreeMap::<&str, usize>::new();
    for token in document_tokens {
        *frequencies.entry(token.as_str()).or_default() += 1;
    }
    let length = document_tokens.len().max(1) as f64;
    query_tokens
        .iter()
        .map(|term| {
            let frequency = frequencies.get(term.as_str()).copied().unwrap_or_default() as f64;
            if frequency == 0.0 {
                return 0.0;
            }
            let df = document_frequency.get(term).copied().unwrap_or(0) as f64;
            let idf = (((document_count as f64 - df + 0.5) / (df + 0.5)) + 1.0).ln();
            let k1 = 1.2;
            let b = 0.75;
            idf * (frequency * (k1 + 1.0))
                / (frequency + k1 * (1.0 - b + b * length / average_length.max(1.0)))
        })
        .sum()
}

fn title_bonus(query_tokens: &BTreeSet<String>, title: &str) -> f64 {
    let title_tokens = tokenize(title).into_iter().collect::<BTreeSet<_>>();
    query_tokens.intersection(&title_tokens).count() as f64 * 0.8
}

fn vectorize(tokens: &[String]) -> [f64; VECTOR_DIMENSIONS] {
    let mut vector = [0.0; VECTOR_DIMENSIONS];
    for token in tokens {
        let hash = fnv1a(token.as_bytes());
        let index = (hash as usize) % VECTOR_DIMENSIONS;
        let sign = if (hash >> 7) & 1 == 0 { 1.0 } else { -1.0 };
        vector[index] += sign;
        let second = ((hash >> 17) as usize) % VECTOR_DIMENSIONS;
        vector[second] += sign * 0.35;
    }
    vector
}

fn cosine(left: &[f64; VECTOR_DIMENSIONS], right: &[f64; VECTOR_DIMENSIONS]) -> f64 {
    let dot = left.iter().zip(right).map(|(a, b)| a * b).sum::<f64>();
    let left_norm = left.iter().map(|value| value * value).sum::<f64>().sqrt();
    let right_norm = right.iter().map(|value| value * value).sum::<f64>().sqrt();
    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm * right_norm)
    }
}

fn ngram_similarity_with_query(left: &BTreeSet<u64>, right: &str) -> f64 {
    let right = ngrams(right);
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let intersection = left.intersection(&right).count() as f64;
    intersection / left.len().min(right.len()) as f64
}

fn ngrams(value: &str) -> BTreeSet<u64> {
    let chars = value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<Vec<_>>();
    chars
        .windows(3)
        .map(|window| {
            let mut bytes = [0_u8; 12];
            let mut offset = 0;
            for character in window {
                let encoded = character.encode_utf8(&mut bytes[offset..]);
                offset += encoded.len();
            }
            fnv1a(&bytes[..offset])
        })
        .collect()
}

fn tokenize(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_alphanumeric())
        .map(str::trim)
        .filter(|token| token.chars().count() >= 2 && !STOP_WORDS.contains(token))
        .map(ToString::to_string)
        .collect()
}

fn expanded_tokens(value: &str) -> BTreeSet<String> {
    let mut tokens = tokenize(value).into_iter().collect::<BTreeSet<_>>();
    for (concept, aliases) in CONCEPT_ALIASES {
        if aliases.iter().any(|alias| value.contains(alias)) {
            tokens.insert((*concept).to_string());
            tokens.extend(aliases.iter().map(|alias| (*alias).to_string()));
        }
    }
    tokens
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| character.to_lowercase())
        .map(fold_vietnamese)
        .collect::<String>()
        .replace(['_', '/', '\\', '-'], " ")
}

fn fold_vietnamese(character: char) -> char {
    match character {
        'à' | 'á' | 'ạ' | 'ả' | 'ã' | 'â' | 'ầ' | 'ấ' | 'ậ' | 'ẩ' | 'ẫ' | 'ă' | 'ằ' | 'ắ' | 'ặ'
        | 'ẳ' | 'ẵ' => 'a',
        'è' | 'é' | 'ẹ' | 'ẻ' | 'ẽ' | 'ê' | 'ề' | 'ế' | 'ệ' | 'ể' | 'ễ' => 'e',
        'ì' | 'í' | 'ị' | 'ỉ' | 'ĩ' => 'i',
        'ò' | 'ó' | 'ọ' | 'ỏ' | 'õ' | 'ô' | 'ồ' | 'ố' | 'ộ' | 'ổ' | 'ỗ' | 'ơ' | 'ờ' | 'ớ' | 'ợ'
        | 'ở' | 'ỡ' => 'o',
        'ù' | 'ú' | 'ụ' | 'ủ' | 'ũ' | 'ư' | 'ừ' | 'ứ' | 'ự' | 'ử' | 'ữ' => 'u',
        'ỳ' | 'ý' | 'ỵ' | 'ỷ' | 'ỹ' => 'y',
        'đ' => 'd',
        _ => character,
    }
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

const STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "in", "is", "it", "of", "on",
    "or", "the", "to", "with", "va", "la", "cua", "cho", "trong", "mot", "nhung", "duoc", "can",
    "de",
];

const CONCEPT_ALIASES: &[(&str, &[&str])] = &[
    (
        "memory",
        &[
            "memory", "memories", "ghi nho", "nho", "vault", "context", "ky uc",
        ],
    ),
    (
        "semantic_retrieval",
        &[
            "semantic",
            "ngữ nghĩa",
            "ngu nghia",
            "meaning",
            "recall",
            "retrieval",
            "search",
            "tìm kiếm",
            "tim kiem",
        ],
    ),
    (
        "session_learning",
        &[
            "session",
            "learning",
            "học từ phiên",
            "hoc tu phien",
            "conversation",
            "lesson",
            "bài học",
            "bai hoc",
        ],
    ),
    (
        "temporal_memory",
        &[
            "temporal",
            "time",
            "thời gian",
            "thoi gian",
            "stale",
            "superseded",
            "history",
            "lịch sử",
            "lich su",
        ],
    ),
    (
        "code_graph",
        &[
            "codegraph",
            "code graph",
            "impact",
            "caller",
            "callee",
            "dependency",
            "phụ thuộc",
            "phu thuoc",
        ],
    ),
    (
        "handoff",
        &[
            "handoff",
            "resume",
            "checkpoint",
            "tiep tuc",
            "ban giao",
            "continuity",
        ],
    ),
    (
        "proof",
        &[
            "proof",
            "evidence",
            "verify",
            "verification",
            "bang chung",
            "xac minh",
        ],
    ),
    (
        "wiki",
        &[
            "wiki",
            "documentation",
            "docs",
            "tai lieu",
            "architecture",
            "guide",
        ],
    ),
    (
        "security",
        &[
            "security",
            "secure",
            "bao mat",
            "vulnerability",
            "fail closed",
            "attack",
        ],
    ),
    (
        "cost",
        &["cost", "token", "budget", "chi phi", "latency", "cheap"],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn document(id: &str, title: &str, body: &str) -> SemanticDocument {
        SemanticDocument {
            id: id.to_string(),
            title: title.to_string(),
            body: body.to_string(),
            path: format!("docs/{id}.md"),
            project_id: None,
            tags: Vec::new(),
        }
    }

    #[test]
    fn ranking_bridges_vietnamese_and_english_concepts() {
        let documents = vec![
            document(
                "memory",
                "Semantic retrieval",
                "Hybrid recall finds meaning and related facts.",
            ),
            document(
                "unrelated",
                "Release",
                "Installer checksum and archive metadata.",
            ),
        ];
        let hits = rank_documents("tìm kiếm ngữ nghĩa", &documents, 2);
        assert_eq!(hits.first().map(|hit| hit.id.as_str()), Some("memory"));
        assert!(hits[0].notes.iter().any(|note| note.starts_with("rrf:")));
        let memory_hits = rank_documents("ghi nho vault", &documents, 2);
        assert_eq!(
            memory_hits.first().map(|hit| hit.id.as_str()),
            Some("memory")
        );
    }

    #[test]
    fn ranking_is_deterministic_and_bounded() {
        let documents = vec![
            document("a", "Temporal memory", "history stale superseded"),
            document("b", "CodeGraph", "callers and impact"),
        ];
        let first = rank_documents("temporal memory", &documents, 1);
        let second = rank_documents("temporal memory", &documents, 1);
        assert_eq!(first, second);
        assert_eq!(first.len(), 1);
    }

    #[test]
    fn v42_retrieval_rejects_positive_rank_without_evidence() {
        let documents = vec![
            document("memory", "Project memory", "proof and next action"),
            document("release", "Release archive", "checksum installer metadata"),
        ];
        let no_match = rank_documents_v42("quantum database that is not present", &documents, 8);
        assert!(no_match.is_empty());
        let match_hits = rank_documents_v42("proof next action", &documents, 8);
        assert_eq!(
            match_hits.first().map(|hit| hit.id.as_str()),
            Some("memory")
        );
        assert!(match_hits[0].confidence >= SemanticPolicy42::default().minimum_confidence);
        assert!(!match_hits[0].evidence_channels.is_empty());
    }

    #[test]
    fn v42_rerank_is_repeatable_and_explains_abstention() {
        let documents = vec![
            document("a", "Memory", "proof current"),
            document("b", "Wiki", "architecture"),
        ];
        let first = rank_documents_v42("missing fact", &documents, 8);
        let second = rank_documents_v42("missing fact", &documents, 8);
        assert_eq!(first, second);
        let permissive = rank_documents_v42_with_policy(
            "missing fact",
            &documents,
            8,
            SemanticPolicy42 {
                minimum_confidence: 0.99,
                ..SemanticPolicy42::default()
            },
        );
        assert!(permissive.is_empty());
    }
}
