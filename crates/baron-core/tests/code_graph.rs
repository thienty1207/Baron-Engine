use std::fs;
use std::path::Path;
use std::time::Duration;

use baron_core::code_graph::{
    code_graph_cache_root, compute_code_source_fingerprint, graph_state_freshness,
    graphify_graph_path, load_code_graph_state, normalize_code_graph_hits, render_code_graph_hits,
    validate_code_graph_state, write_code_graph_state, CodeGraphHit, GraphConfidence,
    GraphFreshness, QueryLimits,
};
use baron_core::config::{initialize_project, AdapterKind};
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn hit(
    node_id: &str,
    source_file: Option<&str>,
    confidence: GraphConfidence,
    explanation: &str,
) -> CodeGraphHit {
    CodeGraphHit {
        node_id: node_id.to_string(),
        label: node_id.to_string(),
        source_file: source_file.map(str::to_string),
        relation: Some("imports".to_string()),
        confidence,
        explanation: explanation.to_string(),
    }
}

#[test]
fn graph_hits_keep_confidence_explicit_and_enforce_safe_bounded_output() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    write(&repo.join("src/lib.rs"), "pub fn entry() {}\n");

    let normalized = normalize_code_graph_hits(
        &repo,
        vec![
            hit(
                "entry",
                Some("src/lib.rs"),
                GraphConfidence::Extracted,
                "direct import from the application entry point",
            ),
            hit(
                "entry",
                Some("src/lib.rs"),
                GraphConfidence::Extracted,
                "duplicate provider row",
            ),
            hit(
                "related",
                Some("src/lib.rs"),
                GraphConfidence::Inferred,
                "likely related through a provider inference",
            ),
        ],
        QueryLimits {
            max_hits: 2,
            max_chars: 160,
        },
    )
    .unwrap();

    assert_eq!(normalized.len(), 2);
    assert_eq!(normalized[0].confidence, GraphConfidence::Extracted);
    assert_eq!(
        serde_json::to_value(&normalized[1]).unwrap()["confidence"],
        "inferred"
    );
    let rendered = render_code_graph_hits(
        &normalized,
        QueryLimits {
            max_hits: 2,
            max_chars: 120,
        },
    );
    assert!(rendered.chars().count() <= 120);
    assert!(rendered.contains("extracted"));

    let unsafe_hit = normalize_code_graph_hits(
        &repo,
        vec![hit(
            "escape",
            Some("../outside.rs"),
            GraphConfidence::Extracted,
            "unsafe source path",
        )],
        QueryLimits::default(),
    );
    assert!(unsafe_hit.is_err());
}

#[test]
fn graph_cache_is_project_scoped_identity_bound_and_disposable() {
    let temp = tempdir().unwrap();
    let vault = temp.path().join("Vault");
    let first = temp.path().join("one/demo");
    let second = temp.path().join("two/demo");
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();
    initialize_project(&first, AdapterKind::Codex, &vault).unwrap();
    initialize_project(&second, AdapterKind::Codex, &vault).unwrap();
    write(&first.join("src/lib.rs"), "pub fn first() {}\n");
    write(&second.join("src/lib.rs"), "pub fn second() {}\n");

    let first_cache = code_graph_cache_root(&first).unwrap();
    let second_cache = code_graph_cache_root(&second).unwrap();
    assert_ne!(first_cache, second_cache);
    assert!(first_cache.starts_with(first.canonicalize().unwrap()));
    assert!(!first_cache.starts_with(&vault));

    let fingerprint = compute_code_source_fingerprint(&first).unwrap();
    let graph_path = graphify_graph_path(&first, &fingerprint).unwrap();
    write(&graph_path, "{\"nodes\": []}\n");
    let state = write_code_graph_state(&first, "test-provider", "1.0.0", &fingerprint, &graph_path)
        .unwrap();
    assert_eq!(
        graph_state_freshness(&first, &state).unwrap(),
        GraphFreshness::Fresh
    );
    assert_eq!(
        load_code_graph_state(&first).unwrap().unwrap().project_id,
        state.project_id
    );
    let mut foreign_state = state.clone();
    foreign_state.project_id = "another-project".to_string();
    assert!(validate_code_graph_state(&first, &foreign_state).is_err());

    let foreign = graph_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("../../outside/graph.json");
    assert!(baron_core::code_graph::validate_code_graph_cache_path(&first, &foreign).is_err());

    let facts = vault.join("Projects");
    fs::create_dir_all(&facts).unwrap();
    write(&facts.join("keep.md"), "Vault memory remains durable.\n");
    fs::remove_dir_all(&first_cache).unwrap();
    assert!(facts.join("keep.md").is_file());
    assert!(!first_cache.exists());
}

#[cfg(unix)]
#[test]
fn graph_cache_symlink_is_rejected_before_any_state_write() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let outside = temp.path().join("outside");
    fs::create_dir_all(repo.join(".baron/cache")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    symlink(&outside, repo.join(".baron/cache/code-graph")).unwrap();

    let error = code_graph_cache_root(&repo).unwrap_err().to_string();

    assert!(error.contains("symlink or junction"));
    assert!(!outside.join("state.json").exists());
}

#[cfg(windows)]
#[test]
fn graph_cache_junction_is_rejected_before_any_state_write() {
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "baron-code-graph-junction-{}-{nonce}",
        std::process::id()
    ));
    let repo = root.join("repo");
    let outside = root.join("outside");
    fs::create_dir_all(repo.join(".baron/cache")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let link = repo.join(".baron/cache/code-graph");
    let script = format!(
        "New-Item -ItemType Junction -Path '{}' -Target '{}' | Out-Null",
        link.display().to_string().replace('\'', "''"),
        outside.display().to_string().replace('\'', "''")
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "junction command failed: {script}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let error = code_graph_cache_root(&repo).unwrap_err().to_string();

    assert!(error.contains("symlink or junction"));
    assert!(!outside.join("state.json").exists());
    fs::remove_dir(&link).unwrap();
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn source_fingerprint_tracks_real_source_changes_without_heavy_folder_noise() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();
    write(&repo.join("src/api.rs"), "pub fn login() {}\n");
    write(&repo.join("node_modules/package/index.js"), "ignored one\n");
    write(&repo.join("target/debug/old.bin"), "ignored two\n");
    write(
        &repo.join("docs/baron/harness/CURRENT.md"),
        "generated state\n",
    );
    write(&repo.join(".codex/skills/INDEX.md"), "generated routing\n");

    let initial = compute_code_source_fingerprint(&repo).unwrap();
    write(
        &repo.join("node_modules/package/index.js"),
        "ignored folder changed\n",
    );
    assert_eq!(initial, compute_code_source_fingerprint(&repo).unwrap());
    write(
        &repo.join("docs/baron/harness/CURRENT.md"),
        "new generated state\n",
    );
    write(
        &repo.join(".codex/skills/INDEX.md"),
        "new generated routing\n",
    );
    assert_eq!(initial, compute_code_source_fingerprint(&repo).unwrap());

    write(
        &repo.join("src/api.rs"),
        "pub fn login_with_password() {}\n",
    );
    let edited = compute_code_source_fingerprint(&repo).unwrap();
    assert_ne!(initial, edited);

    std::thread::sleep(Duration::from_millis(20));
    write(
        &repo.join("src/api.rs"),
        "pub fn token_with_password() {}\n",
    );
    let same_size_edit = compute_code_source_fingerprint(&repo).unwrap();
    assert_ne!(edited, same_size_edit);

    write(&repo.join("src/session.rs"), "pub fn session() {}\n");
    let added = compute_code_source_fingerprint(&repo).unwrap();
    assert_ne!(same_size_edit, added);

    fs::rename(repo.join("src/session.rs"), repo.join("src/token.rs")).unwrap();
    let renamed = compute_code_source_fingerprint(&repo).unwrap();
    assert_ne!(added, renamed);

    fs::remove_file(repo.join("src/token.rs")).unwrap();
    assert_ne!(renamed, compute_code_source_fingerprint(&repo).unwrap());
}
