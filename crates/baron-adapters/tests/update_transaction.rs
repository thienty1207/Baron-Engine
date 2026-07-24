use std::fs;
use std::path::PathBuf;

use baron_adapters::{
    managed_baseline_content, managed_target_path, record_managed_baseline, ManagedAssetPayload,
    ManagedMergeKind,
};
use tempfile::tempdir;

fn payload(path: &str, content: &str) -> ManagedAssetPayload {
    ManagedAssetPayload {
        adapter: "codex".to_string(),
        relative_path: PathBuf::from(path),
        merge_kind: ManagedMergeKind::FullText,
        content: content.to_string(),
    }
}

#[test]
fn transaction_helpers_keep_baseline_and_live_project_paths_separate() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let managed = payload("AGENTS.md", "known baseline");
    record_managed_baseline(repo, std::slice::from_ref(&managed), "3.3.0").unwrap();
    fs::write(repo.join("AGENTS.md"), "user-local content").unwrap();

    let baseline = baron_adapters::load_managed_baseline(repo).unwrap();
    let record = baseline.records.first().unwrap();
    assert_eq!(
        managed_baseline_content(repo, record).unwrap(),
        "known baseline"
    );
    assert_eq!(
        managed_target_path(repo, &record.relative_path).unwrap(),
        repo.join("AGENTS.md").canonicalize().unwrap()
    );
    assert_eq!(
        fs::read_to_string(repo.join("AGENTS.md")).unwrap(),
        "user-local content"
    );
    assert!(managed_target_path(repo, &PathBuf::from("../escape")).is_err());
}
