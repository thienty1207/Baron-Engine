use std::fs;
use std::path::Path;

use baron_core::firewall::{compact_memory_brief, recall};
use baron_core::memory::{build_memory_index, load_memory_records, MemoryKind, MemoryScope};
use baron_core::vault::{ensure_vault, project_slug};
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

#[test]
fn vault_scaffold_creates_root_and_project_capsule_without_overwriting() {
    let temp = tempdir().unwrap();
    let vault = temp.path().join("Vault");
    let repo = temp.path().join("My Legacy App");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&vault).unwrap();
    write(&vault.join("Init.md"), "# My Existing Vault\n");

    let context = ensure_vault(&vault, &repo).unwrap();

    assert_eq!(context.project_slug, "my-legacy-app");
    assert_eq!(read(&vault.join("Init.md")), "# My Existing Vault\n");
    assert!(vault.join("AGENTS.md").exists());
    assert!(vault.join("Projects/my-legacy-app/README.md").exists());
    assert!(vault.join("Projects/my-legacy-app/Facts.md").exists());
    assert!(vault.join("Projects/my-legacy-app/Decisions.md").exists());
    assert!(vault.join("Projects/my-legacy-app/Tasks.md").exists());
    assert!(vault.join("Projects/my-legacy-app/Plans").is_dir());
    assert!(vault.join("Projects/my-legacy-app/ProductHarness").is_dir());
    assert!(vault.join("Projects/my-legacy-app/Sessions").is_dir());
    assert!(vault.join("Projects/my-legacy-app/Artifacts").is_dir());
    assert!(vault.join("Artifacts/Baron/APPROVED_GLOBAL.md").exists());
    assert!(vault.join("Artifacts/Baron/GLOBAL_CANDIDATES.md").exists());
}

#[test]
fn project_slug_is_stable_and_filesystem_safe() {
    assert_eq!(project_slug(Path::new("D:/Work/TomoTy")), "tomoty");
    assert_eq!(
        project_slug(Path::new("D:/Work/My Backend API!!")),
        "my-backend-api"
    );
}

#[test]
fn memory_index_rebuilds_from_markdown_sources() {
    let temp = tempdir().unwrap();
    let vault = temp.path().join("Vault");
    let tomoty = temp.path().join("tomoty");
    let legacy = temp.path().join("legacy-crm");
    fs::create_dir_all(&tomoty).unwrap();
    fs::create_dir_all(&legacy).unwrap();

    let tomoty_context = ensure_vault(&vault, &tomoty).unwrap();
    let legacy_context = ensure_vault(&vault, &legacy).unwrap();
    write(
        &tomoty_context.project_root.join("Facts.md"),
        "# Facts\n\n- Auth login uses Rust Axum and verified tests.\n",
    );
    write(
        &legacy_context.project_root.join("Decisions.md"),
        "# Decisions\n\n- Auth login uses old PHP sessions.\n",
    );
    write(
        &vault.join("Artifacts/Baron/APPROVED_GLOBAL.md"),
        "# Approved Global\n\n- Always redact secrets before writing memory.\n",
    );
    write(
        &vault.join("Artifacts/Baron/GLOBAL_CANDIDATES.md"),
        "# Global Candidates\n\n- Maybe all apps should use one login UI.\n",
    );

    let report = build_memory_index(&tomoty_context).unwrap();
    let records = load_memory_records(&tomoty_context).unwrap();

    assert!(tomoty_context.index_path.exists());
    assert!(report.total_records >= 4);
    assert!(records
        .iter()
        .any(|record| record.project_slug.as_deref() == Some("tomoty")
            && record.excerpt.contains("Rust Axum")));
    assert!(records.iter().any(
        |record| record.project_slug.as_deref() == Some("legacy-crm")
            && record.excerpt.contains("old PHP")
    ));
    assert!(records
        .iter()
        .any(|record| record.scope == MemoryScope::GlobalVerified
            && record.excerpt.contains("redact secrets")));
    assert!(records
        .iter()
        .any(|record| record.scope == MemoryScope::GlobalCandidate
            && record.excerpt.contains("one login UI")));
}

#[test]
fn firewall_prioritizes_current_project_and_blocks_weak_cross_project_matches() {
    let temp = tempdir().unwrap();
    let vault = temp.path().join("Vault");
    let tomoty = temp.path().join("tomoty");
    let legacy = temp.path().join("legacy-crm");
    fs::create_dir_all(&tomoty).unwrap();
    fs::create_dir_all(&legacy).unwrap();

    let tomoty_context = ensure_vault(&vault, &tomoty).unwrap();
    let legacy_context = ensure_vault(&vault, &legacy).unwrap();
    write(
        &tomoty_context.project_root.join("Facts.md"),
        "# Facts\n\n- Auth login uses Rust Axum and verified tests.\n",
    );
    write(
        &legacy_context.project_root.join("Facts.md"),
        "# Facts\n\n- Auth login uses old PHP sessions for legacy-crm.\n",
    );
    write(
        &vault.join("Artifacts/Baron/APPROVED_GLOBAL.md"),
        "# Approved Global\n\n- Login work must include rate limit proof.\n",
    );
    write(
        &vault.join("Artifacts/Baron/GLOBAL_CANDIDATES.md"),
        "# Global Candidates\n\n- Login button color might be blue.\n",
    );
    build_memory_index(&tomoty_context).unwrap();

    let weak = recall(&tomoty_context, "auth login", 10).unwrap();
    assert_eq!(
        weak.results[0].record.project_slug.as_deref(),
        Some("tomoty")
    );
    assert!(weak.results.iter().any(|hit| {
        hit.record.scope == MemoryScope::GlobalVerified && hit.record.excerpt.contains("rate limit")
    }));
    assert!(!weak.results.iter().any(|hit| {
        hit.record.project_slug.as_deref() == Some("legacy-crm")
            && hit.record.excerpt.contains("old PHP")
    }));
    assert!(!weak
        .results
        .iter()
        .any(|hit| hit.record.scope == MemoryScope::GlobalCandidate));
    assert!(weak.blocked_cross_project > 0);

    let explicit = recall(&tomoty_context, "legacy-crm auth login", 10).unwrap();
    assert!(explicit.results.iter().any(|hit| {
        hit.record.project_slug.as_deref() == Some("legacy-crm")
            && hit.record.excerpt.contains("old PHP")
    }));
}

#[test]
fn compact_brief_is_bounded_and_labels_unknowns() {
    let temp = tempdir().unwrap();
    let vault = temp.path().join("Vault");
    let repo = temp.path().join("tomoty");
    fs::create_dir_all(&repo).unwrap();

    let context = ensure_vault(&vault, &repo).unwrap();
    write(
        &context.project_root.join("Facts.md"),
        "# Facts\n\n- Verified project fact with proof.\n",
    );
    build_memory_index(&context).unwrap();

    let brief = compact_memory_brief(&context).unwrap();

    assert!(brief.contains("# Memory Firewall Brief"));
    assert!(brief.contains("Verified project fact"));
    assert!(brief.contains("Unknowns"));
    assert!(brief.contains("No missing memory facts detected"));
}

#[test]
fn scanner_indexes_proof_and_trace_markdown() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    write(
        &context.project_root.join("Proofs/auth-proof.md"),
        "# Proof\n\n- Auth test passed and verified.\n",
    );
    write(
        &context.project_root.join("Traces/auth-trace.md"),
        "# Trace\n\n- Detailed auth trace passed.\n",
    );

    build_memory_index(&context).unwrap();
    let records = load_memory_records(&context).unwrap();

    assert!(records
        .iter()
        .any(|record| record.kind == MemoryKind::Proof));
    assert!(records
        .iter()
        .any(|record| record.kind == MemoryKind::Trace));
}
