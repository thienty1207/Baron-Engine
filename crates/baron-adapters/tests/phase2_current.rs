//! Phase 2 safety expectations. These tests are active because they describe
//! the behavior the shared Safe I/O layer must provide to current consumers.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};

use baron_adapters::{
    install_adapter, reconcile_installed_managed_assets, record_managed_baseline, AgentAdapter,
    ManagedAssetPayload, ManagedMergeKind,
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
fn malformed_codex_hooks_are_preserved_and_reported_before_install_writes() {
    let temp = tempdir().unwrap();
    let path = temp.path().join(".codex/hooks.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let before = b"{ not json\n".to_vec();
    fs::write(&path, &before).unwrap();

    let result = install_adapter(temp.path(), AgentAdapter::Codex);

    assert!(result.is_err());
    assert_eq!(fs::read(path).unwrap(), before);
    assert!(!temp.path().join("AGENTS.md").exists());
}

#[test]
fn malformed_claude_settings_are_preserved_and_reported_before_install_writes() {
    let temp = tempdir().unwrap();
    let path = temp.path().join(".claude/settings.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let before = b"{ not json\n".to_vec();
    fs::write(&path, &before).unwrap();

    let result = install_adapter(temp.path(), AgentAdapter::Claude);

    assert!(result.is_err());
    assert_eq!(fs::read(path).unwrap(), before);
    assert!(!temp.path().join("CLAUDE.md").exists());
}

#[test]
fn invalid_utf8_managed_text_is_preserved_and_reported() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("AGENTS.md");
    let before = vec![0xff, 0xfe, 0xfd, 0x00];
    fs::write(&path, &before).unwrap();

    let result = install_adapter(temp.path(), AgentAdapter::Codex);

    assert!(result.is_err());
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn managed_directory_is_rejected_without_replacement() {
    let temp = tempdir().unwrap();
    let target = temp.path().join("AGENTS.md");
    fs::create_dir_all(&target).unwrap();
    let managed = payload("AGENTS.md", "managed content\n");
    record_managed_baseline(temp.path(), std::slice::from_ref(&managed), "4.2.2").unwrap();

    let result = reconcile_installed_managed_assets(temp.path(), &[managed], "4.2.2");

    assert!(result.is_err());
    assert!(target.is_dir());
}

#[test]
fn legacy_temp_collision_is_reported_without_destroying_the_sentinel() {
    let temp = tempdir().unwrap();
    let collision = temp
        .path()
        .join(".baron/managed-state/base/codex/AGENTS.baron-tmp");
    fs::create_dir_all(collision.parent().unwrap()).unwrap();
    fs::write(&collision, "sentinel temp content\n").unwrap();
    let managed = payload("AGENTS.md", "managed content\n");

    let result = record_managed_baseline(temp.path(), &[managed], "4.2.2");

    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(collision).unwrap(),
        "sentinel temp content\n"
    );
}

#[test]
fn path_boundary_rejects_parent_and_absolute_targets() {
    let temp = tempdir().unwrap();
    let parent = payload("../escape.md", "unsafe\n");
    let absolute = payload(
        Path::new("outside.md")
            .canonicalize()
            .unwrap_or_else(|_| temp.path().join("outside.md"))
            .to_string_lossy()
            .as_ref(),
        "unsafe\n",
    );

    assert!(record_managed_baseline(temp.path(), &[parent], "4.2.2").is_err());
    assert!(record_managed_baseline(temp.path(), &[absolute], "4.2.2").is_err());
}

#[test]
fn concurrent_baseline_publication_leaves_one_complete_manifest() {
    let temp = tempdir().unwrap();
    let repo = temp.path().to_path_buf();
    let barrier = Arc::new(Barrier::new(3));
    let first_repo = repo.clone();
    let first_barrier = Arc::clone(&barrier);
    let second_repo = repo.clone();
    let second_barrier = Arc::clone(&barrier);
    let first = std::thread::spawn(move || {
        first_barrier.wait();
        record_managed_baseline(
            &first_repo,
            &[payload("AGENTS.md", "writer one\n")],
            "4.2.2",
        )
    });
    let second = std::thread::spawn(move || {
        second_barrier.wait();
        record_managed_baseline(
            &second_repo,
            &[payload("AGENTS.md", "writer two\n")],
            "4.2.2",
        )
    });
    barrier.wait();

    assert!(first.join().unwrap().is_ok());
    assert!(second.join().unwrap().is_ok());
    let baseline = baron_adapters::load_managed_baseline(&repo).unwrap();
    assert_eq!(baseline.records.len(), 1);
    let baseline_content =
        fs::read_to_string(repo.join(".baron/managed-state/base/codex/AGENTS.md")).unwrap();
    assert!(baseline_content == "writer one\n" || baseline_content == "writer two\n");
    assert_eq!(
        baron_adapters::managed_baseline_content(&repo, &baseline.records[0]).unwrap(),
        baseline_content
    );
}

#[test]
fn concurrent_reconcile_publication_keeps_target_and_baseline_consistent() {
    let temp = tempdir().unwrap();
    let repo = temp.path().to_path_buf();
    let base = payload("AGENTS.md", "baseline\n");
    record_managed_baseline(&repo, std::slice::from_ref(&base), "4.2.2").unwrap();
    fs::write(repo.join("AGENTS.md"), "baseline\n").unwrap();
    let barrier = Arc::new(Barrier::new(3));
    let first_repo = repo.clone();
    let first_barrier = Arc::clone(&barrier);
    let second_repo = repo.clone();
    let second_barrier = Arc::clone(&barrier);
    let first = std::thread::spawn(move || {
        first_barrier.wait();
        reconcile_installed_managed_assets(
            &first_repo,
            &[payload("AGENTS.md", "upstream one\n")],
            "4.2.3",
        )
    });
    let second = std::thread::spawn(move || {
        second_barrier.wait();
        reconcile_installed_managed_assets(
            &second_repo,
            &[payload("AGENTS.md", "upstream two\n")],
            "4.2.4",
        )
    });
    barrier.wait();

    assert!(first.join().unwrap().is_ok());
    assert!(second.join().unwrap().is_ok());
    let target = fs::read_to_string(repo.join("AGENTS.md")).unwrap();
    assert!(target == "upstream one\n" || target == "upstream two\n");
    let baseline = baron_adapters::managed_baseline_content(
        &repo,
        &baron_adapters::load_managed_baseline(&repo)
            .unwrap()
            .records[0],
    )
    .unwrap();
    assert_eq!(baseline, target);
}
