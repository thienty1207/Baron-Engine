use std::fs;
use std::path::Path;

use baron_core::code_graph::{
    cached_code_graph_hits_for_task, code_graph_cache_root, compute_code_source_fingerprint,
    graphify_graph_path, load_code_graph_state, render_optional_code_map_context,
    write_code_graph_query_cache, write_code_graph_state, CodeGraphHit, GraphConfidence,
};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::firewall::recall;
use baron_core::memory::{build_memory_index, load_memory_records};
use baron_core::vault::vault_context_without_create;
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn seed_project_graph(repo: &Path, source_file: &str, symbol: &str, task: &str) {
    write(
        &repo.join(source_file),
        &format!("pub fn {symbol}() {{ /* project-local auth flow */ }}\n"),
    );
    let fingerprint = compute_code_source_fingerprint(repo).unwrap();
    let graph_path = graphify_graph_path(repo, &fingerprint).unwrap();
    write(&graph_path, r#"{"nodes":[],"edges":[]}"#);
    let state = write_code_graph_state(repo, "graphify-local", "0.9.25", &fingerprint, &graph_path)
        .unwrap();
    write_code_graph_query_cache(
        repo,
        &state,
        task,
        vec![CodeGraphHit {
            node_id: symbol.to_string(),
            label: symbol.to_string(),
            source_file: Some(source_file.to_string()),
            relation: Some("owns".to_string()),
            confidence: GraphConfidence::Extracted,
            explanation: format!("{symbol} is local to this repository"),
        }],
    )
    .unwrap();
}

#[test]
fn same_name_projects_keep_graphs_local_and_out_of_vault_memory() {
    let temp = tempdir().unwrap();
    let vault = temp.path().join("Vault");
    let first = temp.path().join("one").join("same-app");
    let second = temp.path().join("two").join("same-app");
    let task = "trace cross-module auth ownership";
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();

    initialize_project(&first, AdapterKind::Codex, &vault).unwrap();
    initialize_project(&second, AdapterKind::Codex, &vault).unwrap();
    let first_context = vault_context_without_create(&vault, &first).unwrap();
    let second_context = vault_context_without_create(&vault, &second).unwrap();
    write(
        &first_context.project_root.join("Facts.md"),
        "# Facts\n\n- First project owns the verified customer authentication boundary.\n",
    );
    write(
        &second_context.project_root.join("Facts.md"),
        "# Facts\n\n- Second project uses a separate authentication boundary.\n",
    );

    seed_project_graph(&first, "src/first_auth.rs", "first_auth_owner", task);
    seed_project_graph(&second, "src/second_auth.rs", "second_auth_owner", task);

    let first_state = load_code_graph_state(&first).unwrap().unwrap();
    let second_state = load_code_graph_state(&second).unwrap().unwrap();
    assert_eq!(first_context.project_slug, second_context.project_slug);
    assert_ne!(first_state.project_id, second_state.project_id);
    assert_ne!(first_state.repo_root, second_state.repo_root);

    let first_hits = cached_code_graph_hits_for_task(&first, &first_state, task)
        .unwrap()
        .unwrap();
    let second_hits = cached_code_graph_hits_for_task(&second, &second_state, task)
        .unwrap()
        .unwrap();
    assert!(first_hits
        .iter()
        .all(|hit| hit.source_file.as_deref() == Some("src/first_auth.rs")));
    assert!(second_hits
        .iter()
        .all(|hit| hit.source_file.as_deref() == Some("src/second_auth.rs")));

    let first_context_bundle = render_optional_code_map_context(&first, Some(task)).unwrap();
    assert!(first_context_bundle.contains("src/first_auth.rs"));
    assert!(!first_context_bundle.contains("src/second_auth.rs"));

    build_memory_index(&first_context).unwrap();
    let memory = load_memory_records(&first_context).unwrap();
    assert!(memory
        .iter()
        .all(|record| !record.path.contains("code-graph")));
    assert!(memory
        .iter()
        .all(|record| !record.excerpt.contains("first_auth_owner")));
    let recall = recall(&first_context, "authentication boundary", 10).unwrap();
    assert!(recall
        .results
        .iter()
        .all(|hit| !hit.record.excerpt.contains("first_auth_owner")));
    assert!(recall
        .results
        .iter()
        .all(|hit| !hit.record.excerpt.contains("second_auth_owner")));

    let first_cache = code_graph_cache_root(&first).unwrap();
    let second_cache = code_graph_cache_root(&second).unwrap();
    assert_ne!(first_cache, second_cache);
    assert!(!first_cache.starts_with(&vault));
    assert!(!second_cache.starts_with(&vault));
    fs::remove_dir_all(&first_cache).unwrap();
    assert!(!first_cache.exists());
    assert!(second_cache.is_dir());
    assert!(first_context.index_path.is_file());
    assert!(first_context.project_root.join("Facts.md").is_file());
    assert!(second_context.project_root.join("Facts.md").is_file());
}
