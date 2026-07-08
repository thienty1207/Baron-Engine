use std::fs;

use baron_core::review_gate::{close_finding, record_finding, review_status, ReviewFindingInput};
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
