use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Barrier;
use std::thread;

use baron_core::config::{initialize_project, AdapterKind};
use baron_core::harness_experiment::{finalize_experiment, record_fresh_rerun, start_experiment};
use baron_core::safe_io::acquire_project_lock;
use baron_core::vault::{ensure_vault, VaultContext};

fn shared_checkouts(root: &Path) -> (PathBuf, PathBuf, VaultContext, VaultContext) {
    let repo_a = root.join("checkout-a");
    let repo_b = root.join("checkout-b");
    let vault = root.join("shared-vault");
    fs::create_dir_all(&repo_a).unwrap();
    initialize_project(&repo_a, AdapterKind::Codex, &vault).unwrap();
    let context_a = ensure_vault(&vault, &repo_a).unwrap();
    fs::create_dir_all(repo_b.join(".baron")).unwrap();
    fs::copy(
        repo_a.join(".baron/project.toml"),
        repo_b.join(".baron/project.toml"),
    )
    .unwrap();
    let context_b = ensure_vault(&vault, &repo_b).unwrap();
    assert_eq!(context_a.project_id, context_b.project_id);
    assert_eq!(context_a.project_root, context_b.project_root);
    (repo_a, repo_b, context_a, context_b)
}

fn snapshot_tree(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn visit(root: &Path, current: &Path, output: &mut Vec<(String, Vec<u8>)>) {
        if !current.exists() {
            return;
        }
        for entry in fs::read_dir(current).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                visit(root, &path, output);
            } else {
                output.push((
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                    fs::read(&path).unwrap(),
                ));
            }
        }
    }
    let mut files = Vec::new();
    visit(root, root, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

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

#[test]
fn experiment_shared_writers_fail_without_writes_when_capsule_is_locked() {
    let temp = tempfile::tempdir().unwrap();
    let (repo, _, context, _) = shared_checkouts(temp.path());
    let repo_experiments = repo.join("docs/baron/harness/experiments");
    let vault_experiments = context.project_root.join("ProductHarness/Experiments");
    let before_repo = snapshot_tree(&repo_experiments);
    let before_vault = snapshot_tree(&vault_experiments);

    let vault_lock = acquire_project_lock(&context.project_root).unwrap();
    let start = thread::scope(|scope| {
        scope
            .spawn(|| {
                start_experiment(
                    &repo,
                    &context,
                    "baseline",
                    "hypothesis",
                    "intervention",
                    true,
                )
            })
            .join()
            .unwrap()
    })
    .expect_err("experiment creation must wait for the shared capsule lock");
    assert!(start.to_string().contains("Timed out waiting"), "{start:#}");
    assert_eq!(snapshot_tree(&repo_experiments), before_repo);
    assert_eq!(snapshot_tree(&vault_experiments), before_vault);
    drop(vault_lock);

    let record = start_experiment(
        &repo,
        &context,
        "baseline",
        "hypothesis",
        "intervention",
        true,
    )
    .unwrap();
    let before_repo = fs::read(&record.repo_path).unwrap();
    let before_vault = fs::read(&record.vault_path).unwrap();
    let vault_lock = acquire_project_lock(&context.project_root).unwrap();
    let error = thread::scope(|scope| {
        scope
            .spawn(|| {
                record_fresh_rerun(
                    &repo, &context, &record.id, true, true, true, true, "improved",
                )
            })
            .join()
            .unwrap()
    })
    .expect_err("rerun publication must wait for the shared capsule lock");
    assert!(error.to_string().contains("Timed out waiting"), "{error:#}");
    assert_eq!(fs::read(&record.repo_path).unwrap(), before_repo);
    assert_eq!(fs::read(&record.vault_path).unwrap(), before_vault);
    drop(vault_lock);
}

#[test]
fn experiment_finalize_fails_without_writes_when_capsule_is_locked() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
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
    record_fresh_rerun(
        &repo, &context, &record.id, true, true, true, true, "improved",
    )
    .unwrap();
    let before_repo = fs::read(&record.repo_path).unwrap();
    let before_vault = fs::read(&record.vault_path).unwrap();

    let vault_lock = acquire_project_lock(&context.project_root).unwrap();
    let error = thread::scope(|scope| {
        scope
            .spawn(|| finalize_experiment(&repo, &context, &record.id, "keep"))
            .join()
            .unwrap()
    })
    .expect_err("finalize publication must wait for the shared capsule lock");
    assert!(error.to_string().contains("Timed out waiting"), "{error:#}");
    assert_eq!(fs::read(&record.repo_path).unwrap(), before_repo);
    assert_eq!(fs::read(&record.vault_path).unwrap(), before_vault);
    drop(vault_lock);
}

#[test]
fn experiment_mutation_preserves_divergent_local_user_content() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
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
    let mut local = fs::read(&record.repo_path).unwrap();
    local.extend_from_slice(b"\n## User Notes\n\nKeep this local note.\n");
    fs::write(&record.repo_path, &local).unwrap();
    let shared_before = fs::read(&record.vault_path).unwrap();

    let error = record_fresh_rerun(
        &repo, &context, &record.id, true, true, true, true, "improved",
    )
    .expect_err("divergent user content must not be replaced by the Vault projection");

    assert!(error.to_string().contains("diverge"), "{error:#}");
    assert_eq!(fs::read(&record.repo_path).unwrap(), local);
    assert_eq!(fs::read(&record.vault_path).unwrap(), shared_before);
}

#[test]
fn shared_checkouts_serialize_experiment_reruns_and_finalization() {
    let temp = tempfile::tempdir().unwrap();
    let (repo_a, repo_b, context_a, context_b) = shared_checkouts(temp.path());
    let record = start_experiment(
        &repo_a,
        &context_a,
        "baseline",
        "hypothesis",
        "intervention",
        true,
    )
    .unwrap();
    let relative = record.repo_path.strip_prefix(&repo_a).unwrap();
    let repo_b_path = repo_b.join(relative);
    fs::create_dir_all(repo_b_path.parent().unwrap()).unwrap();
    fs::copy(&record.repo_path, &repo_b_path).unwrap();
    let before_b = fs::read(&repo_b_path).unwrap();
    let barrier = Barrier::new(3);

    let (rerun_a, rerun_b) = thread::scope(|scope| {
        let a = scope.spawn(|| {
            barrier.wait();
            record_fresh_rerun(
                &repo_a,
                &context_a,
                &record.id,
                true,
                true,
                true,
                true,
                "outcome from A",
            )
        });
        let b = scope.spawn(|| {
            barrier.wait();
            record_fresh_rerun(
                &repo_b,
                &context_b,
                &record.id,
                false,
                false,
                false,
                false,
                "outcome from B",
            )
        });
        barrier.wait();
        (a.join().unwrap(), b.join().unwrap())
    });
    assert_ne!(
        rerun_a.is_ok(),
        rerun_b.is_ok(),
        "only one concurrent fresh rerun may be accepted"
    );
    let shared = fs::read_to_string(&record.vault_path).unwrap();
    assert!(shared.contains("- Status: `rerun_recorded`"));
    assert_ne!(
        shared.contains("outcome from A"),
        shared.contains("outcome from B"),
        "the losing checkout must not overwrite the accepted shared outcome"
    );
    if rerun_a.is_ok() {
        assert_eq!(fs::read(&repo_b_path).unwrap(), before_b);
    } else {
        assert_eq!(fs::read(&record.repo_path).unwrap(), before_b);
    }

    fs::write(&repo_b_path, fs::read(&record.vault_path).unwrap()).unwrap();
    let finalize_barrier = Barrier::new(3);
    let (finalize_a, finalize_b) = thread::scope(|scope| {
        let a = scope.spawn(|| {
            finalize_barrier.wait();
            finalize_experiment(&repo_a, &context_a, &record.id, "keep")
        });
        let b = scope.spawn(|| {
            finalize_barrier.wait();
            finalize_experiment(&repo_b, &context_b, &record.id, "remove")
        });
        finalize_barrier.wait();
        (a.join().unwrap(), b.join().unwrap())
    });
    assert_ne!(
        finalize_a.is_ok(),
        finalize_b.is_ok(),
        "only one concurrent final decision may be accepted"
    );
    let shared = fs::read_to_string(&record.vault_path).unwrap();
    assert_ne!(
        shared.contains("completed_keep"),
        shared.contains("completed_remove"),
        "the losing checkout must not overwrite the accepted final decision"
    );
}
