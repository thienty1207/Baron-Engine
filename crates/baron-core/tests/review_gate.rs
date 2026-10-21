use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use baron_core::config::{initialize_project, AdapterKind};
use baron_core::review_gate::{close_finding, record_finding, review_status, ReviewFindingInput};
use baron_core::safe_io::acquire_project_lock;
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

fn finding() -> ReviewFindingInput {
    ReviewFindingInput {
        severity: "important".to_string(),
        summary: "Mobile navigation overlaps the footer".to_string(),
        evidence: vec!["Screenshot at 390px shows overlap".to_string()],
        affected_files: vec!["src/HomePage.tsx".to_string()],
    }
}

#[test]
fn finding_is_mirrored_and_cannot_close_without_fix_and_verification() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("app");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let record = record_finding(&repo, &context, finding()).unwrap();
    assert!(record.repo_path.exists());
    assert!(record.vault_path.exists());

    let missing_fix =
        close_finding(&repo, &context, &record.id, "", "responsive test passed").unwrap_err();
    assert!(missing_fix.to_string().contains("fix evidence"));
    let missing_verification =
        close_finding(&repo, &context, &record.id, "CSS grid corrected", "").unwrap_err();
    assert!(missing_verification.to_string().contains("verification"));
}

#[test]
fn closure_preserves_original_finding_and_records_evidence() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("app");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let record = record_finding(&repo, &context, finding()).unwrap();

    close_finding(
        &repo,
        &context,
        &record.id,
        "Changed footer layout constraints in src/HomePage.tsx",
        "responsive DOM tests and 390px/1440px screenshots passed",
    )
    .unwrap();

    let content = fs::read_to_string(&record.repo_path).unwrap();
    assert!(content.contains("Mobile navigation overlaps"));
    assert!(content.contains("Status: `closed`"));
    assert!(content.contains("Changed footer layout constraints"));
    assert!(content.contains("390px/1440px"));
    assert!(review_status(&repo).unwrap().contains("Open findings: 0"));
}

#[test]
fn record_finding_waits_for_the_shared_vault_capsule_lock_across_checkouts() {
    let temp = tempdir().unwrap();
    let (first_repo, second_repo, first_vault, second_vault) = shared_vault_checkouts(temp.path());
    assert_ne!(first_vault.repo_root, second_vault.repo_root);
    assert_eq!(first_vault.project_root, second_vault.project_root);

    let mutation_repo = second_repo.clone();
    let mutation_vault = second_vault.clone();
    let record = run_while_capsule_locked(&first_vault.project_root, move || {
        record_finding(&mutation_repo, &mutation_vault, finding())
    })
    .unwrap();

    assert!(record.repo_path.is_file());
    assert!(record.vault_path.is_file());
    assert!(!first_repo.join("docs/baron/reviews/INDEX.md").exists());
    assert!(second_repo.join("docs/baron/reviews/INDEX.md").is_file());
}

#[test]
fn close_finding_waits_for_the_shared_vault_capsule_lock_across_checkouts() {
    let temp = tempdir().unwrap();
    let (first_repo, second_repo, first_vault, second_vault) = shared_vault_checkouts(temp.path());
    assert_ne!(first_vault.repo_root, second_vault.repo_root);
    assert_eq!(first_vault.project_root, second_vault.project_root);

    let record = record_finding(&first_repo, &first_vault, finding()).unwrap();
    let second_finding = second_repo
        .join("docs/baron/reviews/findings")
        .join(format!("{}.md", record.id));
    fs::create_dir_all(second_finding.parent().unwrap()).unwrap();
    fs::copy(&record.repo_path, &second_finding).unwrap();

    let mutation_repo = second_repo.clone();
    let mutation_vault = second_vault.clone();
    let mutation_id = record.id.clone();
    run_while_capsule_locked(&first_vault.project_root, move || {
        close_finding(
            &mutation_repo,
            &mutation_vault,
            &mutation_id,
            "Updated navigation layout",
            "responsive checks passed",
        )
    })
    .unwrap();

    let repo_content = fs::read_to_string(&second_finding).unwrap();
    let vault_content = fs::read_to_string(&record.vault_path).unwrap();
    assert!(repo_content.contains("Status: `closed`"));
    assert!(repo_content.contains("Updated navigation layout"));
    assert!(vault_content.contains("Status: `closed`"));
    assert!(vault_content.contains("responsive checks passed"));
}

#[test]
fn stale_checkout_cannot_overwrite_shared_vault_closure_evidence() {
    let temp = tempdir().unwrap();
    let (first_repo, second_repo, first_vault, second_vault) = shared_vault_checkouts(temp.path());
    let record = record_finding(&first_repo, &first_vault, finding()).unwrap();
    let second_finding = second_repo
        .join("docs/baron/reviews/findings")
        .join(format!("{}.md", record.id));
    fs::create_dir_all(second_finding.parent().unwrap()).unwrap();
    fs::copy(&record.repo_path, &second_finding).unwrap();

    close_finding(
        &first_repo,
        &first_vault,
        &record.id,
        "First checkout fix evidence",
        "First checkout verification",
    )
    .unwrap();
    close_finding(
        &second_repo,
        &second_vault,
        &record.id,
        "Stale second checkout evidence",
        "Stale second checkout verification",
    )
    .unwrap();

    let shared = fs::read_to_string(&record.vault_path).unwrap();
    let second_local = fs::read_to_string(&second_finding).unwrap();
    assert!(shared.contains("First checkout fix evidence"));
    assert!(shared.contains("First checkout verification"));
    assert!(!shared.contains("Stale second checkout"));
    assert!(second_local.contains("First checkout fix evidence"));
    assert!(second_local.contains("First checkout verification"));
}

#[test]
fn record_finding_lock_timeout_leaves_repo_and_vault_unchanged() {
    let temp = tempdir().unwrap();
    let (_, second_repo, first_vault, second_vault) = shared_vault_checkouts(temp.path());
    let capsule_root = first_vault.project_root.clone();
    let (ready_tx, ready_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let owner = std::thread::spawn(move || {
        let _lock = acquire_project_lock(capsule_root).unwrap();
        ready_tx.send(()).unwrap();
        release_rx.recv().unwrap();
    });
    ready_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("capsule lock owner did not start");

    let result = record_finding(&second_repo, &second_vault, finding());
    release_tx.send(()).unwrap();
    owner.join().unwrap();

    let error = result.unwrap_err().to_string();
    assert!(error.contains("Timed out waiting for Baron mutation lock"));
    assert!(!second_repo.join("docs/baron/reviews").exists());
    assert!(!second_vault.project_root.join("Reviews").exists());
}

fn shared_vault_checkouts(
    root: &Path,
) -> (
    PathBuf,
    PathBuf,
    baron_core::vault::VaultContext,
    baron_core::vault::VaultContext,
) {
    let first_repo = root.join("first/checkout");
    let second_repo = root.join("second/checkout");
    let vault_root = root.join("Vault");
    fs::create_dir_all(&first_repo).unwrap();
    fs::create_dir_all(&second_repo).unwrap();

    initialize_project(&first_repo, AdapterKind::Codex, &vault_root).unwrap();
    fs::create_dir_all(second_repo.join(".baron")).unwrap();
    fs::copy(
        first_repo.join(".baron/project.toml"),
        second_repo.join(".baron/project.toml"),
    )
    .unwrap();

    let first_vault = ensure_vault(&vault_root, &first_repo).unwrap();
    let second_vault = ensure_vault(&vault_root, &second_repo).unwrap();
    (first_repo, second_repo, first_vault, second_vault)
}

fn run_while_capsule_locked<T: Send + 'static>(
    capsule_root: &Path,
    mutation: impl FnOnce() -> T + Send + 'static,
) -> T {
    let capsule_lock = acquire_project_lock(capsule_root).unwrap();
    let (started_tx, started_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        started_tx.send(()).unwrap();
        result_tx.send(mutation()).unwrap();
    });
    started_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("review-gate mutation did not start");
    let completed_while_locked = result_rx.recv_timeout(Duration::from_millis(200)).is_ok();
    drop(capsule_lock);

    if completed_while_locked {
        worker.join().unwrap();
        panic!("review-gate mutation completed while the shared capsule lock was held");
    }
    let result = result_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("review-gate mutation did not resume after the capsule lock was released");
    worker.join().unwrap();
    result
}
