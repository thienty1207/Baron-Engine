use std::env;
use std::fs;

use baron_core::code_graph::{
    code_graph_cache_root, validate_code_graph_artifact, verify_graph_hit_source,
    CodeGraphProvider, QueryLimits, SourceVerificationStatus,
};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::graphify::{GraphifyProvider, SUPPORTED_GRAPHIFY_VERSION};
use tempfile::tempdir;

#[test]
fn real_graphify_0_9_25_contract_is_certified_when_opted_in() {
    if env::var("BARON_REAL_GRAPHIFY").as_deref() != Ok("1") {
        eprintln!("skipped: set BARON_REAL_GRAPHIFY=1 to run real Graphify certification");
        return;
    }
    assert!(
        env::var_os("GRAPHIFY_OUT").is_none(),
        "real certification must run without GRAPHIFY_OUT"
    );

    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(
        repo.join("src/lib.rs"),
        "pub fn entrypoint() { service(); }\nfn service() {}\n",
    )
    .unwrap();

    let provider = GraphifyProvider::new("graphify");
    let probe = provider.probe(&repo).unwrap();
    assert!(probe.present);
    assert_eq!(probe.version.as_deref(), Some(SUPPORTED_GRAPHIFY_VERSION));

    let cache = code_graph_cache_root(&repo).unwrap();
    let state = provider.refresh(&repo, &cache).unwrap();
    let graph_path = validate_code_graph_artifact(&repo, &state).unwrap();
    assert!(graph_path.starts_with(&cache));
    assert!(graph_path.ends_with(std::path::Path::new("graphify-out").join("graph.json")));

    let hits = provider
        .query(
            &repo,
            &cache,
            "entrypoint",
            QueryLimits {
                max_hits: 1,
                max_chars: 512,
            },
        )
        .unwrap();
    assert!(!hits.is_empty());
    assert_eq!(hits[0].source_file.as_deref(), Some("src/lib.rs"));
    let verification = verify_graph_hit_source(&repo, &hits[0]).unwrap();
    assert_eq!(verification.status, SourceVerificationStatus::Verified);
}
