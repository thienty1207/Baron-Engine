use std::fs;

use baron_core::domain_language::{
    ensure_domain_language, read_domain_language, render_domain_language_context,
};
use baron_core::memory::{build_memory_index, load_memory_records};
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

fn write(path: &std::path::Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn language_with_terms() -> String {
    "# Product Domain Language\n\n\
## Rules\n\n\
- Add terms only from user, repository, product, or verified runtime evidence.\n\
- Mark disputed or unclear meanings as `ambiguous`.\n\
- Do not promote a term to verified without an evidence path.\n\n\
## Terms\n\n\
| Term | Meaning | Status | Evidence |\n\
| --- | --- | --- | --- |\n\
| workspace | A tenant-scoped working area. | verified | docs/architecture/tenancy.md |\n\
| shelf | Whether archived content remains visible is unresolved. | ambiguous | user request 2026-07-24 |\n"
        .to_string()
}

#[test]
fn ensure_creates_project_scoped_domain_language_without_inventing_terms() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("catalog");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let vault = ensure_vault(&vault_root, &repo).unwrap();

    let status = ensure_domain_language(&repo, &vault).unwrap();

    let repo_path = repo.join("docs/baron/harness/DOMAIN_LANGUAGE.md");
    let vault_path = vault.project_root.join("ProductHarness/DOMAIN_LANGUAGE.md");
    assert_eq!(status.path, repo_path);
    assert_eq!(status.term_count, 0);
    assert_eq!(status.ambiguous_count, 0);
    assert!(repo_path.is_file());
    assert!(vault_path.is_file());
    let content = fs::read_to_string(&repo_path).unwrap();
    assert!(content.contains("# Product Domain Language"));
    assert!(content.contains("| Term | Meaning | Status | Evidence |"));
    assert!(!content.contains("workspace"));
}

#[test]
fn read_only_status_does_not_create_domain_language_files() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("catalog");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let vault = ensure_vault(&vault_root, &repo).unwrap();

    let status = read_domain_language(&repo, &vault).unwrap();

    assert_eq!(status.term_count, 0);
    assert!(!status.mirror_in_sync);
    assert!(!repo.join("docs/baron/harness/DOMAIN_LANGUAGE.md").exists());
    assert!(!vault
        .project_root
        .join("ProductHarness/DOMAIN_LANGUAGE.md")
        .exists());
}

#[test]
fn repeated_ensure_preserves_user_terms_and_tracks_ambiguous_ones() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("catalog");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let vault = ensure_vault(&vault_root, &repo).unwrap();
    ensure_domain_language(&repo, &vault).unwrap();

    let content = language_with_terms();
    let repo_path = repo.join("docs/baron/harness/DOMAIN_LANGUAGE.md");
    let vault_path = vault.project_root.join("ProductHarness/DOMAIN_LANGUAGE.md");
    write(&repo_path, &content);
    write(&vault_path, &content);

    let status = ensure_domain_language(&repo, &vault).unwrap();

    assert_eq!(fs::read_to_string(&repo_path).unwrap(), content);
    assert_eq!(fs::read_to_string(&vault_path).unwrap(), content);
    assert_eq!(status.term_count, 2);
    assert_eq!(status.ambiguous_count, 1);
}

#[test]
fn bounded_context_keeps_status_and_evidence_without_promoting_ambiguity() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("catalog");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let vault = ensure_vault(&vault_root, &repo).unwrap();
    ensure_domain_language(&repo, &vault).unwrap();
    write(
        &repo.join("docs/baron/harness/DOMAIN_LANGUAGE.md"),
        &language_with_terms(),
    );

    let rendered = render_domain_language_context(&repo, 700).unwrap();
    assert!(rendered.contains("## Product Domain Language"));
    assert!(rendered.contains("workspace"));
    assert!(rendered.contains("verified"));
    assert!(rendered.contains("docs/architecture/tenancy.md"));
    assert!(rendered.contains("shelf"));
    assert!(rendered.contains("ambiguous"));
    assert!(rendered.chars().count() <= 700);

    let limited = render_domain_language_context(&repo, 80).unwrap();
    assert!(limited.chars().count() <= 80);
}

#[test]
fn projects_sharing_a_vault_never_share_domain_language() {
    let temp = tempdir().unwrap();
    let repo_one = temp.path().join("catalog");
    let repo_two = temp.path().join("billing");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo_one).unwrap();
    fs::create_dir_all(&repo_two).unwrap();
    let vault_one = ensure_vault(&vault_root, &repo_one).unwrap();
    let vault_two = ensure_vault(&vault_root, &repo_two).unwrap();
    ensure_domain_language(&repo_one, &vault_one).unwrap();
    ensure_domain_language(&repo_two, &vault_two).unwrap();

    write(
        &repo_one.join("docs/baron/harness/DOMAIN_LANGUAGE.md"),
        &language_with_terms(),
    );

    assert_ne!(vault_one.project_root, vault_two.project_root);
    let second = render_domain_language_context(&repo_two, 700).unwrap();
    assert!(!second.contains("tenant-scoped working area"));
}

#[test]
fn domain_language_uses_its_status_aware_renderer_not_generic_memory_excerpts() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("catalog");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let vault = ensure_vault(&vault_root, &repo).unwrap();
    ensure_domain_language(&repo, &vault).unwrap();
    let content = format!(
        "{}\n# Historical Notes\n\nThis body must not become a generic memory excerpt.\n",
        language_with_terms()
    );
    write(
        &repo.join("docs/baron/harness/DOMAIN_LANGUAGE.md"),
        &content,
    );
    write(
        &vault.project_root.join("ProductHarness/DOMAIN_LANGUAGE.md"),
        &content,
    );

    build_memory_index(&vault).unwrap();
    let records = load_memory_records(&vault).unwrap();

    assert!(records
        .iter()
        .all(|record| !record.path.ends_with("ProductHarness/DOMAIN_LANGUAGE.md")));
}

#[test]
fn divergent_repo_and_vault_terms_are_preserved_but_not_trusted_as_synced() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("catalog");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let vault = ensure_vault(&vault_root, &repo).unwrap();
    ensure_domain_language(&repo, &vault).unwrap();

    let repo_content = language_with_terms();
    let vault_content = "# Product Domain Language\n\n## Terms\n\n| Term | Meaning | Status | Evidence |\n| --- | --- | --- | --- |\n";
    let repo_path = repo.join("docs/baron/harness/DOMAIN_LANGUAGE.md");
    let vault_path = vault.project_root.join("ProductHarness/DOMAIN_LANGUAGE.md");
    write(&repo_path, &repo_content);
    write(&vault_path, vault_content);

    let status = ensure_domain_language(&repo, &vault).unwrap();

    assert!(!status.mirror_in_sync);
    assert_eq!(fs::read_to_string(&repo_path).unwrap(), repo_content);
    assert_eq!(fs::read_to_string(&vault_path).unwrap(), vault_content);
}
