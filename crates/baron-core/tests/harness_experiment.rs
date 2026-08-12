use baron_core::harness_experiment::{finalize_experiment, record_fresh_rerun, start_experiment};
use baron_core::vault::ensure_vault;

#[test]
fn experiment_requires_fresh_rerun_before_keep() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    std::fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let record = start_experiment(
        &repo,
        &context,
        "baseline",
        "hypothesis",
        "intervention",
        true,
    )
    .unwrap();
    assert!(finalize_experiment(&repo, &context, &record.id, "keep").is_err());
    record_fresh_rerun(
        &repo, &context, &record.id, true, true, true, true, "improved",
    )
    .unwrap();
    finalize_experiment(&repo, &context, &record.id, "keep").unwrap();
    assert!(std::fs::read_to_string(record.repo_path)
        .unwrap()
        .contains("completed_keep"));
}
