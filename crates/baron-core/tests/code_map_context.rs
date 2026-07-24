use std::fs;
use std::path::Path;

use baron_core::code_graph::{
    compute_code_source_fingerprint, graphify_graph_path, verify_graph_hit_source,
    write_code_graph_query_cache, write_code_graph_state, CodeGraphHit, GraphConfidence,
    SourceVerificationStatus,
};
use baron_core::context::{compile_context_for_task, compile_context_why, ContextTarget};
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn seed_graph(repo: &Path) -> baron_core::code_graph::CodeGraphState {
    write(
        &repo.join("src/checkout.rs"),
        "pub fn checkout() { authorize_customer(); }\nfn authorize_customer() {}\n",
    );
    let fingerprint = compute_code_source_fingerprint(repo).unwrap();
    let graph = graphify_graph_path(repo, &fingerprint).unwrap();
    write(&graph, r#"{"nodes":[],"edges":[]}"#);
    write_code_graph_state(repo, "graphify-local", "0.9.25", &fingerprint, &graph).unwrap()
}

#[test]
fn matching_task_context_loads_only_bounded_cached_code_map_hits() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("legacy-store");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let state = seed_graph(&repo);
    write_code_graph_query_cache(
        &repo,
        &state,
        "trace checkout call flow",
        vec![
            CodeGraphHit {
                node_id: "checkout".to_string(),
                label: "checkout".to_string(),
                source_file: Some("src/checkout.rs".to_string()),
                relation: Some("calls".to_string()),
                confidence: GraphConfidence::Extracted,
                explanation: "checkout calls authorization".to_string(),
            },
            CodeGraphHit {
                node_id: "authorize_customer".to_string(),
                label: "authorize_customer".to_string(),
                source_file: Some("src/checkout.rs".to_string()),
                relation: Some("depends_on".to_string()),
                confidence: GraphConfidence::Inferred,
                explanation: "authorization relationship needs source review".to_string(),
            },
        ],
    )
    .unwrap();

    let bundle = compile_context_for_task(
        &repo,
        &vault,
        ContextTarget::Codex,
        Some("trace checkout call flow"),
    )
    .unwrap();

    assert!(bundle.contains("## Optional Code Map"));
    assert!(bundle.contains("checkout calls authorization"), "{bundle}");
    assert!(bundle.contains("[inferred]"));
    assert!(bundle.contains("verify against source before proof"));
    assert!(bundle.chars().count() <= 20_000);

    let no_navigation = compile_context_for_task(
        &repo,
        &vault,
        ContextTarget::Codex,
        Some("update README copy"),
    )
    .unwrap();
    assert!(!no_navigation.contains("## Optional Code Map"));

    let why = compile_context_why(&repo, &vault, ContextTarget::Codex).unwrap();
    assert!(why.contains("Optional Code Map"));
    assert!(why.contains("does not run Graphify"));
}

#[test]
fn source_verification_requires_current_extracted_evidence_and_keeps_inference_advisory() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("legacy-store");
    fs::create_dir_all(&repo).unwrap();
    write(
        &repo.join("src/checkout.rs"),
        "pub fn checkout() { authorize_customer(); }\nfn authorize_customer() {}\n",
    );
    let extracted = CodeGraphHit {
        node_id: "checkout".to_string(),
        label: "checkout".to_string(),
        source_file: Some("src/checkout.rs".to_string()),
        relation: Some("calls".to_string()),
        confidence: GraphConfidence::Extracted,
        explanation: "source symbol".to_string(),
    };
    let inferred = CodeGraphHit {
        confidence: GraphConfidence::Inferred,
        ..extracted.clone()
    };
    let missing = CodeGraphHit {
        label: "removed_symbol".to_string(),
        node_id: "removed_symbol".to_string(),
        ..extracted.clone()
    };
    let escaping = CodeGraphHit {
        source_file: Some("../other-project/src/checkout.rs".to_string()),
        ..extracted.clone()
    };
    let deleted = CodeGraphHit {
        source_file: Some("src/deleted.rs".to_string()),
        ..extracted.clone()
    };
    let foreign = CodeGraphHit {
        source_file: Some(
            temp.path()
                .join("other-project/src/checkout.rs")
                .display()
                .to_string(),
        ),
        ..extracted.clone()
    };

    assert_eq!(
        verify_graph_hit_source(&repo, &extracted).unwrap().status,
        SourceVerificationStatus::Verified
    );
    assert_eq!(
        verify_graph_hit_source(&repo, &inferred).unwrap().status,
        SourceVerificationStatus::Advisory
    );
    assert_eq!(
        verify_graph_hit_source(&repo, &missing).unwrap().status,
        SourceVerificationStatus::MissingEvidence
    );
    assert_eq!(
        verify_graph_hit_source(&repo, &deleted).unwrap().status,
        SourceVerificationStatus::MissingSource
    );
    assert!(verify_graph_hit_source(&repo, &escaping).is_err());
    assert!(verify_graph_hit_source(&repo, &foreign).is_err());
}
