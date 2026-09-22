use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use baron_core::plan::start_or_resume_plan;
use baron_core::proof::record_proof;
use baron_core::trace::{record_trace, score_trace, TraceOutcome};
use baron_core::vault::{ensure_vault, vault_context_without_create};
use tempfile::tempdir;

#[test]
fn new_proof_trace_and_plan_instances_have_collision_resistant_names() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let first_plan = start_or_resume_plan(&repo, &context, "same day title").unwrap();
    let resumed_plan = start_or_resume_plan(&repo, &context, "same day title").unwrap();
    let proof = record_proof(&repo, &context, "cargo test passed").unwrap();
    let trace = record_trace(
        &repo,
        &context,
        "concurrent state persisted",
        TraceOutcome::Completed,
    )
    .unwrap();

    assert_eq!(first_plan.repo_path, resumed_plan.repo_path);
    assert!(first_plan
        .repo_path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .contains("-same-day-title-"));
    for id in [&proof.id, &trace.id] {
        let parts = id.split('-').collect::<Vec<_>>();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 20);
        assert_eq!(parts[2].len(), 32);
    }
    assert!(proof.repo_path.is_file());
    assert!(proof.vault_path.is_file());
    assert!(trace.repo_path.is_file());
    assert!(trace.vault_path.is_file());
}

#[test]
fn multiprocess_proof_and_trace_publications_are_lossless() {
    const WORKER_COUNT: usize = 4;
    let temp = tempdir().unwrap();

    let proof_repo = temp.path().join("proof-repo");
    let proof_vault = temp.path().join("proof-vault");
    fs::create_dir_all(&proof_repo).unwrap();
    ensure_vault(&proof_vault, &proof_repo).unwrap();
    let proof_ids = run_workers("proof", &proof_repo, &proof_vault, WORKER_COUNT);
    assert_eq!(proof_ids.len(), WORKER_COUNT);
    assert_eq!(
        artifact_files(&proof_repo.join("docs/baron/proofs")).len(),
        WORKER_COUNT
    );
    let proof_index = fs::read_to_string(proof_repo.join("docs/baron/proofs/INDEX.md")).unwrap();
    assert!(proof_ids.iter().all(|id| proof_index.contains(id)));

    let trace_repo = temp.path().join("trace-repo");
    let trace_vault = temp.path().join("trace-vault");
    fs::create_dir_all(&trace_repo).unwrap();
    ensure_vault(&trace_vault, &trace_repo).unwrap();
    let trace_ids = run_workers("trace", &trace_repo, &trace_vault, WORKER_COUNT);
    assert_eq!(trace_ids.len(), WORKER_COUNT);
    assert_eq!(
        artifact_files(&trace_repo.join("docs/baron/traces")).len(),
        WORKER_COUNT
    );
    let trace_index = fs::read_to_string(trace_repo.join("docs/baron/traces/INDEX.md")).unwrap();
    assert!(trace_ids.iter().all(|id| trace_index.contains(id)));
    let score_trace_id = trace_ids.iter().next().unwrap();
    let scored_ids = run_score_workers(&trace_repo, &trace_vault, score_trace_id, WORKER_COUNT);
    assert_eq!(scored_ids, BTreeSet::from([score_trace_id.clone()]));
    let scored_content = fs::read_to_string(
        trace_repo
            .join("docs/baron/traces")
            .join(find_trace_file(&trace_repo, score_trace_id)),
    )
    .unwrap();
    assert_eq!(scored_content.matches("BARON:TRACE-SCORE:START").count(), 1);
}

fn run_workers(mode: &str, repo: &Path, vault: &Path, count: usize) -> BTreeSet<String> {
    let ready = repo.join("ready");
    let release = repo.join("release");
    fs::create_dir_all(&ready).unwrap();
    let binary = env::current_exe().unwrap();
    let mut children = Vec::new();
    for index in 0..count {
        children.push(
            Command::new(&binary)
                .args(["--exact", "concurrency_worker", "--nocapture"])
                .env("BARON_CONCURRENCY_WORKER", "1")
                .env("BARON_CONCURRENCY_MODE", mode)
                .env("BARON_CONCURRENCY_REPO", repo)
                .env("BARON_CONCURRENCY_VAULT", vault)
                .env("BARON_CONCURRENCY_INDEX", index.to_string())
                .env("BARON_CONCURRENCY_READY", &ready)
                .env("BARON_CONCURRENCY_RELEASE", &release)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
    }

    let deadline = Instant::now() + Duration::from_secs(15);
    while fs::read_dir(&ready).unwrap().count() < count {
        assert!(Instant::now() < deadline, "workers did not reach barrier");
        thread::sleep(Duration::from_millis(10));
    }
    fs::write(&release, b"release").unwrap();

    let mut ids = BTreeSet::new();
    for child in children {
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "worker failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let marker = if mode == "proof" {
            "PROOF_ID="
        } else {
            "TRACE_ID="
        };
        let id = String::from_utf8_lossy(&output.stdout)
            .lines()
            .find_map(|line| line.strip_prefix(marker))
            .map(str::to_string)
            .expect("worker did not emit an artifact ID");
        ids.insert(id);
    }
    ids
}

fn run_score_workers(repo: &Path, vault: &Path, trace_id: &str, count: usize) -> BTreeSet<String> {
    let ready = repo.join("score-ready");
    let release = repo.join("score-release");
    fs::create_dir_all(&ready).unwrap();
    let binary = env::current_exe().unwrap();
    let mut children = Vec::new();
    for index in 0..count {
        children.push(
            Command::new(&binary)
                .args(["--exact", "concurrency_worker", "--nocapture"])
                .env("BARON_CONCURRENCY_WORKER", "1")
                .env("BARON_CONCURRENCY_MODE", "score")
                .env("BARON_CONCURRENCY_REPO", repo)
                .env("BARON_CONCURRENCY_VAULT", vault)
                .env("BARON_CONCURRENCY_INDEX", index.to_string())
                .env("BARON_CONCURRENCY_TRACE_ID", trace_id)
                .env("BARON_CONCURRENCY_READY", &ready)
                .env("BARON_CONCURRENCY_RELEASE", &release)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
    }
    let deadline = Instant::now() + Duration::from_secs(15);
    while fs::read_dir(&ready).unwrap().count() < count {
        assert!(
            Instant::now() < deadline,
            "score workers did not reach barrier"
        );
        thread::sleep(Duration::from_millis(10));
    }
    fs::write(&release, b"release").unwrap();

    let mut ids = BTreeSet::new();
    for child in children {
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "score worker failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let id = String::from_utf8_lossy(&output.stdout)
            .lines()
            .find_map(|line| line.strip_prefix("SCORE_ID="))
            .map(str::to_string)
            .expect("score worker did not emit a trace ID");
        ids.insert(id);
    }
    ids
}

fn find_trace_file(repo: &Path, trace_id: &str) -> PathBuf {
    artifact_files(&repo.join("docs/baron/traces"))
        .into_iter()
        .find(|path| path.file_stem().and_then(|value| value.to_str()) == Some(trace_id))
        .unwrap()
        .strip_prefix(repo.join("docs/baron/traces"))
        .unwrap()
        .to_path_buf()
}

fn artifact_files(root: &Path) -> Vec<PathBuf> {
    if !root.exists() {
        return Vec::new();
    }
    fs::read_dir(root)
        .unwrap()
        .flat_map(|entry| {
            let path = entry.unwrap().path();
            if path.is_dir() {
                artifact_files(&path)
            } else if path.extension().and_then(|value| value.to_str()) == Some("md")
                && path.file_name().and_then(|value| value.to_str()) != Some("INDEX.md")
            {
                vec![path]
            } else {
                Vec::new()
            }
        })
        .collect()
}

#[test]
fn concurrency_worker() {
    if env::var_os("BARON_CONCURRENCY_WORKER").is_none() {
        return;
    }
    let mode = env::var("BARON_CONCURRENCY_MODE").unwrap();
    let repo = PathBuf::from(env::var_os("BARON_CONCURRENCY_REPO").unwrap());
    let vault = PathBuf::from(env::var_os("BARON_CONCURRENCY_VAULT").unwrap());
    let index = env::var("BARON_CONCURRENCY_INDEX").unwrap();
    let ready = PathBuf::from(env::var_os("BARON_CONCURRENCY_READY").unwrap());
    let release = PathBuf::from(env::var_os("BARON_CONCURRENCY_RELEASE").unwrap());
    fs::write(ready.join(format!("ready-{index}")), b"ready").unwrap();
    while !release.exists() {
        thread::sleep(Duration::from_millis(10));
    }
    let context = vault_context_without_create(&vault, &repo).unwrap();
    match mode.as_str() {
        "proof" => {
            let proof = record_proof(
                &repo,
                &context,
                &format!("worker {index} cargo test passed"),
            )
            .unwrap();
            println!("PROOF_ID={}", proof.id);
        }
        "trace" => {
            let trace = record_trace(
                &repo,
                &context,
                &format!("worker {index} state persisted"),
                TraceOutcome::Completed,
            )
            .unwrap();
            println!("TRACE_ID={}", trace.id);
        }
        "score" => {
            let trace_id = env::var("BARON_CONCURRENCY_TRACE_ID").unwrap();
            let score = score_trace(&repo, &context, Some(&trace_id)).unwrap();
            println!("SCORE_ID={}", score.trace_id);
        }
        other => panic!("unknown concurrency worker mode: {other}"),
    }
}
