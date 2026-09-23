use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use baron_core::capability::{
    load_registry, record_runtime_execution, register_provider, CapabilityExecutionEvidence,
    CapabilityProvider, ProviderKind, Requirement,
};
use baron_core::config::{load_project_config, AdapterKind};
use baron_core::continuity::{record_recovery, RecoveryInput, RecoveryOutcome};
use baron_core::control_plane::record_gate_evidence;
use baron_core::harness::{record_friction, start_or_resume_intake};
use baron_core::harness_improvement::record_intervention;
use baron_core::intent::{record_intent, IntentBriefInput};
use baron_core::operation::{LifecycleIdentity, OperationContext, SupportedAdapter};
use baron_core::plan::{complete_plan, start_or_resume_plan_for_identity};
use baron_core::proof::{record_proof, record_proof_for_operation};
use baron_core::safe_io::acquire_project_lock;
use baron_core::trace::{
    record_trace, record_trace_for_operation, score_trace, TraceOperationBinding, TraceOutcome,
};
use baron_core::vault::{ensure_vault, vault_context_without_create};
use tempfile::tempdir;

#[test]
fn new_proof_trace_and_plan_instances_have_collision_resistant_names() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let first_identity = LifecycleIdentity::resolve(
        &context.project_id,
        "same day docs title",
        SupportedAdapter::Codex,
        Some("plan-session-1"),
        Some("plan-request-1"),
    )
    .unwrap();
    let first_operation = OperationContext::from_identity(&first_identity);
    let first_plan =
        start_or_resume_plan_for_identity(&repo, &context, "same day docs title", &first_identity)
            .unwrap();
    let resumed_plan =
        start_or_resume_plan_for_identity(&repo, &context, "same day docs title", &first_identity)
            .unwrap();
    let proof =
        record_proof_for_operation(&repo, &context, &first_operation, "cargo test passed").unwrap();
    let binding = TraceOperationBinding::from_operation(&first_operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "concurrent state persisted",
        TraceOutcome::Completed,
        &binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );
    complete_plan(&repo, &context, "concurrency test complete").unwrap();
    let completed_first_plan_bytes = fs::read(&first_plan.repo_path).unwrap();
    let second_identity = LifecycleIdentity::resolve(
        &context.project_id,
        "same day docs title",
        SupportedAdapter::Codex,
        Some("plan-session-2"),
        Some("plan-request-2"),
    )
    .unwrap();
    let second_plan =
        start_or_resume_plan_for_identity(&repo, &context, "same day docs title", &second_identity)
            .unwrap();
    let plan_index = fs::read_to_string(repo.join("docs/baron/plans/INDEX.md")).unwrap();
    let first_relative = first_plan
        .repo_path
        .strip_prefix(&repo)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    let second_relative = second_plan
        .repo_path
        .strip_prefix(&repo)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    assert!(plan_index.contains(&first_relative));
    assert!(plan_index.contains(&second_relative));

    assert_eq!(first_plan.repo_path, resumed_plan.repo_path);
    assert_ne!(first_plan.repo_path, second_plan.repo_path);
    assert_eq!(
        fs::read(&first_plan.repo_path).unwrap(),
        completed_first_plan_bytes
    );
    assert!(first_plan
        .repo_path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .contains("-same-day-docs-title-"));
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
    let seed_trace = record_trace(
        &trace_repo,
        &vault_context_without_create(&trace_vault, &trace_repo).unwrap(),
        "seed trace for concurrent scoring",
        TraceOutcome::Completed,
    )
    .unwrap();
    let (trace_ids, scored_ids) =
        run_trace_score_workers(&trace_repo, &trace_vault, &seed_trace.id, WORKER_COUNT);
    assert_eq!(trace_ids.len(), WORKER_COUNT);
    assert_eq!(
        artifact_files(&trace_repo.join("docs/baron/traces")).len(),
        WORKER_COUNT + 1
    );
    let trace_index = fs::read_to_string(trace_repo.join("docs/baron/traces/INDEX.md")).unwrap();
    assert!(trace_ids.iter().all(|id| trace_index.contains(id)));
    assert_eq!(scored_ids, BTreeSet::from([seed_trace.id.clone()]));
    let scored_content = fs::read_to_string(
        trace_repo
            .join("docs/baron/traces")
            .join(find_trace_file(&trace_repo, &seed_trace.id)),
    )
    .unwrap();
    assert_eq!(scored_content.matches("BARON:TRACE-SCORE:START").count(), 1);
}

#[test]
fn multiprocess_plan_instances_preserve_history() {
    const WORKER_COUNT: usize = 4;
    let temp = tempdir().unwrap();
    let repo = temp.path().join("plan-repo");
    let vault = temp.path().join("plan-vault");
    fs::create_dir_all(&repo).unwrap();
    ensure_vault(&vault, &repo).unwrap();

    let plan_paths = run_workers("plan", &repo, &vault, WORKER_COUNT);
    assert_eq!(plan_paths.len(), WORKER_COUNT);
    assert_eq!(
        artifact_files(&repo.join("docs/baron/plans")).len(),
        WORKER_COUNT + 1
    );
    let plan_index = fs::read_to_string(repo.join("docs/baron/plans/INDEX.md")).unwrap();
    assert!(plan_paths.iter().all(|path| plan_index.contains(path)));
}

#[test]
fn multiprocess_first_config_initialization_preserves_one_identity() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("config-repo");
    let vault = temp.path().join("config-vault");
    fs::create_dir_all(&repo).unwrap();

    let identities = run_workers("config-init", &repo, &vault, 2);
    let config = load_project_config(&repo).unwrap();

    assert_eq!(identities.len(), 1);
    assert!(!config.project_id.is_empty());
    assert_eq!(config.adapters.len(), 2);
    assert!(config.adapters.contains(&AdapterKind::Codex));
    assert!(config.adapters.contains(&AdapterKind::Claude));
    assert!(matches!(
        config.active_adapter,
        Some(AdapterKind::Codex | AdapterKind::Claude)
    ));
    assert!(toml::from_str::<toml::Value>(
        &fs::read_to_string(repo.join(".baron/project.toml")).unwrap()
    )
    .is_ok());
}

#[test]
fn multiprocess_config_setters_preserve_all_supported_values() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("config-set-repo");
    let vault = temp.path().join("config-set-vault");
    fs::create_dir_all(&repo).unwrap();
    baron_core::config::initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();

    let result_ids = run_workers("config-set", &repo, &vault, 2);
    let config = load_project_config(&repo).unwrap();

    assert_eq!(result_ids.len(), 1);
    assert_eq!(config.adapters.len(), 2);
    assert!(config.adapters.contains(&AdapterKind::Codex));
    assert!(config.adapters.contains(&AdapterKind::Claude));
    assert!(config.platform.is_some());
    assert!(config.platform_extensions.len() <= 1);
}

#[test]
fn multiprocess_harness_append_and_matrix_publications_are_lossless() {
    const WORKER_COUNT: usize = 4;
    let temp = tempdir().unwrap();
    let repo = temp.path().join("harness-repo");
    let vault = temp.path().join("harness-vault");
    fs::create_dir_all(&repo).unwrap();
    ensure_vault(&vault, &repo).unwrap();

    assert_eq!(
        run_workers("friction", &repo, &vault, WORKER_COUNT).len(),
        WORKER_COUNT
    );
    assert_eq!(
        run_workers("intervention", &repo, &vault, WORKER_COUNT).len(),
        WORKER_COUNT
    );
    assert_eq!(
        run_workers("matrix", &repo, &vault, WORKER_COUNT).len(),
        WORKER_COUNT
    );

    let friction = fs::read_to_string(repo.join("docs/baron/harness/FRICTION.md")).unwrap();
    assert_eq!(friction.matches("worker friction").count(), WORKER_COUNT);
    let interventions =
        fs::read_to_string(repo.join("docs/baron/harness/INTERVENTIONS.md")).unwrap();
    assert_eq!(
        interventions.matches("worker intervention").count(),
        WORKER_COUNT
    );
    let matrix = fs::read_to_string(repo.join("docs/baron/harness/TEST_MATRIX.md")).unwrap();
    for index in 0..WORKER_COUNT {
        assert!(matrix.contains(&format!("fix README typo {index}")));
    }
}

#[test]
fn multiprocess_gate_evidence_publications_are_lossless() {
    const WORKER_COUNT: usize = 4;
    let temp = tempdir().unwrap();
    let repo = temp.path().join("gate-repo");
    let vault = temp.path().join("gate-vault");
    fs::create_dir_all(&repo).unwrap();
    let vault_context = ensure_vault(&vault, &repo).unwrap();

    assert_eq!(
        run_workers("gate", &repo, &vault, WORKER_COUNT).len(),
        WORKER_COUNT
    );
    let repo_gates = fs::read_to_string(repo.join("docs/baron/control-plane/GATES.md")).unwrap();
    let vault_gates =
        fs::read_to_string(vault_context.project_root.join("ControlPlane/GATES.md")).unwrap();
    assert_eq!(repo_gates.matches("worker gate").count(), WORKER_COUNT);
    assert_eq!(vault_gates.matches("worker gate").count(), WORKER_COUNT);
}

#[test]
fn multiprocess_capability_registry_and_runtime_evidence_are_lossless() {
    const WORKER_COUNT: usize = 4;
    let temp = tempdir().unwrap();
    let repo = temp.path().join("capability-repo");
    let vault = temp.path().join("capability-vault");
    fs::create_dir_all(&repo).unwrap();
    ensure_vault(&vault, &repo).unwrap();

    assert_eq!(
        run_workers("registry", &repo, &vault, WORKER_COUNT).len(),
        WORKER_COUNT
    );
    let registry = load_registry(&repo).unwrap();
    assert_eq!(registry.providers.len(), WORKER_COUNT);

    assert_eq!(
        run_workers("runtime", &repo, &vault, WORKER_COUNT).len(),
        WORKER_COUNT
    );
    let runtime = fs::read_to_string(repo.join(".baron/cache/runtime-execution.jsonl")).unwrap();
    let records = runtime
        .lines()
        .map(serde_json::from_str::<serde_json::Value>)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(records.len(), WORKER_COUNT);
    assert!(records.iter().all(|record| record["summary"]
        .as_str()
        .is_some_and(|summary| summary.contains("worker runtime"))));
}

#[test]
fn multiprocess_intent_and_recovery_history_are_lossless() {
    const WORKER_COUNT: usize = 4;
    let temp = tempdir().unwrap();
    let repo = temp.path().join("continuity-repo");
    let vault = temp.path().join("continuity-vault");
    fs::create_dir_all(&repo).unwrap();
    ensure_vault(&vault, &repo).unwrap();

    assert_eq!(
        run_workers("intent", &repo, &vault, WORKER_COUNT).len(),
        WORKER_COUNT
    );
    assert_eq!(
        run_workers("recovery", &repo, &vault, WORKER_COUNT).len(),
        WORKER_COUNT
    );
    assert_eq!(
        artifact_files(&repo.join("docs/baron/harness/intents")).len(),
        WORKER_COUNT
    );
    assert_eq!(
        artifact_files(&repo.join("docs/baron/continuity/recovery")).len(),
        WORKER_COUNT
    );
    let intent_index = fs::read_to_string(repo.join("docs/baron/harness/INTENTS.md")).unwrap();
    let recovery_index =
        fs::read_to_string(repo.join("docs/baron/continuity/RECOVERY_INDEX.md")).unwrap();
    assert_eq!(intent_index.matches("worker intent").count(), WORKER_COUNT);
    assert_eq!(
        recovery_index
            .lines()
            .filter(|line| line.contains("outcome: `interrupted`"))
            .count(),
        WORKER_COUNT
    );
}

#[test]
fn proof_mutation_lock_timeout_fails_without_writing_state() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("timeout-repo");
    let vault = temp.path().join("timeout-vault");
    fs::create_dir_all(&repo).unwrap();
    ensure_vault(&vault, &repo).unwrap();
    let owner = acquire_project_lock(&repo).unwrap();

    let output = run_timeout_worker("timeout", &repo, &vault);
    drop(owner);

    assert!(String::from_utf8_lossy(&output.stdout).contains("TIMEOUT_OK"));
    assert!(!repo.join("docs/baron/proofs").exists());
    assert!(!repo.join("docs/baron/harness").exists());
}

#[test]
fn plan_and_trace_lock_timeouts_fail_without_writing_state() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("timeout-repo");
    let vault = temp.path().join("timeout-vault");
    fs::create_dir_all(&repo).unwrap();
    ensure_vault(&vault, &repo).unwrap();
    let owner = acquire_project_lock(&repo).unwrap();

    let plan_output = run_timeout_worker("plan-timeout", &repo, &vault);
    let trace_output = run_timeout_worker("trace-timeout", &repo, &vault);
    drop(owner);

    assert!(String::from_utf8_lossy(&plan_output.stdout).contains("PLAN_TIMEOUT_OK"));
    assert!(String::from_utf8_lossy(&trace_output.stdout).contains("TRACE_TIMEOUT_OK"));
    assert!(!repo.join("docs/baron/plans").exists());
    assert!(!repo.join("docs/baron/traces").exists());
}

fn run_timeout_worker(mode: &str, repo: &Path, vault: &Path) -> std::process::Output {
    let output = Command::new(env::current_exe().unwrap())
        .args(["--exact", "concurrency_worker", "--nocapture"])
        .env("BARON_CONCURRENCY_WORKER", "1")
        .env("BARON_CONCURRENCY_MODE", mode)
        .env("BARON_CONCURRENCY_REPO", repo)
        .env("BARON_CONCURRENCY_VAULT", vault)
        .env("BARON_CONCURRENCY_INDEX", "timeout")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{mode} worker failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn run_workers(mode: &str, repo: &Path, vault: &Path, count: usize) -> BTreeSet<String> {
    let ready = repo.join(format!("{mode}-ready"));
    let release = repo.join(format!("{mode}-release"));
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
        let marker = match mode {
            "proof" => "PROOF_ID=",
            "plan" => "PLAN_ID=",
            "config-init" => "CONFIG_ID=",
            "config-set" => "CONFIG_SET_ID=",
            "friction" => "FRICTION_ID=",
            "intervention" => "INTERVENTION_ID=",
            "matrix" => "MATRIX_ID=",
            "gate" => "GATE_ID=",
            "registry" => "REGISTRY_ID=",
            "runtime" => "RUNTIME_ID=",
            "intent" => "INTENT_ID=",
            "recovery" => "RECOVERY_ID=",
            "timeout" => "TIMEOUT_OK",
            _ => "TRACE_ID=",
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

fn run_trace_score_workers(
    repo: &Path,
    vault: &Path,
    trace_id: &str,
    count: usize,
) -> (BTreeSet<String>, BTreeSet<String>) {
    let ready = repo.join("trace-score-ready");
    let release = repo.join("trace-score-release");
    fs::create_dir_all(&ready).unwrap();
    let binary = env::current_exe().unwrap();
    let mut children = Vec::new();
    for (mode, index) in (0..count)
        .map(|index| ("trace", index))
        .chain((0..count).map(|index| ("score", index)))
    {
        let worker_index = if mode == "score" {
            count + index
        } else {
            index
        };
        let mut command = Command::new(&binary);
        command
            .args(["--exact", "concurrency_worker", "--nocapture"])
            .env("BARON_CONCURRENCY_WORKER", "1")
            .env("BARON_CONCURRENCY_MODE", mode)
            .env("BARON_CONCURRENCY_REPO", repo)
            .env("BARON_CONCURRENCY_VAULT", vault)
            .env("BARON_CONCURRENCY_INDEX", worker_index.to_string())
            .env("BARON_CONCURRENCY_TRACE_ID", trace_id)
            .env("BARON_CONCURRENCY_READY", &ready)
            .env("BARON_CONCURRENCY_RELEASE", &release)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        children.push((mode, command.spawn().unwrap()));
    }
    let expected = count * 2;
    let deadline = Instant::now() + Duration::from_secs(15);
    while fs::read_dir(&ready).unwrap().count() < expected {
        assert!(
            Instant::now() < deadline,
            "trace/score workers did not reach barrier"
        );
        thread::sleep(Duration::from_millis(10));
    }
    fs::write(&release, b"release").unwrap();

    let mut trace_ids = BTreeSet::new();
    let mut score_ids = BTreeSet::new();
    for (mode, child) in children {
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{mode} worker failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let output = String::from_utf8_lossy(&output.stdout);
        match mode {
            "trace" => {
                let id = output
                    .lines()
                    .find_map(|line| line.strip_prefix("TRACE_ID="))
                    .map(str::to_string)
                    .expect("trace worker did not emit an artifact ID");
                trace_ids.insert(id);
            }
            "score" => {
                let id = output
                    .lines()
                    .find_map(|line| line.strip_prefix("SCORE_ID="))
                    .map(str::to_string)
                    .expect("score worker did not emit a trace ID");
                score_ids.insert(id);
            }
            _ => unreachable!(),
        }
    }
    (trace_ids, score_ids)
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
    if !matches!(mode.as_str(), "timeout" | "plan-timeout" | "trace-timeout") {
        let ready = PathBuf::from(env::var_os("BARON_CONCURRENCY_READY").unwrap());
        let release = PathBuf::from(env::var_os("BARON_CONCURRENCY_RELEASE").unwrap());
        fs::write(ready.join(format!("ready-{index}")), b"ready").unwrap();
        while !release.exists() {
            thread::sleep(Duration::from_millis(10));
        }
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
        "config-init" => {
            let adapter = if index.parse::<usize>().unwrap() == 0 {
                AdapterKind::Codex
            } else {
                AdapterKind::Claude
            };
            let config = baron_core::config::initialize_project(&repo, adapter, &vault).unwrap();
            println!("CONFIG_ID={}", config.project_id);
        }
        "config-set" => {
            let index = index.parse::<usize>().unwrap();
            let platform = if index == 0 {
                baron_core::config::ProjectPlatform::Fullstack
            } else {
                baron_core::config::ProjectPlatform::Tool
            };
            let adapter = if index == 0 {
                AdapterKind::Claude
            } else {
                AdapterKind::Codex
            };
            baron_core::config::set_project_platform(&repo, platform).unwrap();
            let config = baron_core::config::set_active_adapter(&repo, adapter).unwrap();
            println!("CONFIG_SET_ID={}", config.project_id);
        }
        "friction" => {
            record_friction(&repo, &context, &format!("worker friction {index}")).unwrap();
            println!("FRICTION_ID={index}");
        }
        "intervention" => {
            record_intervention(&repo, &context, &format!("worker intervention {index}")).unwrap();
            println!("INTERVENTION_ID={index}");
        }
        "matrix" => {
            start_or_resume_intake(&repo, &context, &format!("fix README typo {index}")).unwrap();
            println!("MATRIX_ID={index}");
        }
        "gate" => {
            record_gate_evidence(
                &repo,
                &context,
                "test-engineer",
                &format!("worker gate {index}"),
            )
            .unwrap();
            println!("GATE_ID={index}");
        }
        "registry" => {
            register_provider(
                &repo,
                CapabilityProvider {
                    name: format!("worker-provider-{index}"),
                    capability: format!("worker-capability-{index}"),
                    kind: ProviderKind::Cli,
                    requirement: Requirement::Optional,
                    command: Some("git".to_string()),
                    scan_target: None,
                    adapters: Vec::new(),
                    description: "Concurrent registry test provider".to_string(),
                },
            )
            .unwrap();
            println!("REGISTRY_ID={index}");
        }
        "runtime" => {
            record_runtime_execution(
                &repo,
                &[CapabilityExecutionEvidence {
                    capability: "worker-capability".to_string(),
                    provider: "worker-provider".to_string(),
                    summary: format!("worker runtime {index}"),
                    receipt_id: None,
                    task_id: None,
                    operation_id: None,
                    gate_kind: None,
                    session_id: None,
                    request_id: None,
                }],
            )
            .unwrap();
            println!("RUNTIME_ID={index}");
        }
        "intent" => {
            let title = format!("worker intent {index}");
            let intent = record_intent(
                &repo,
                &context,
                IntentBriefInput {
                    title,
                    current_behavior: "Current behavior is recorded.".to_string(),
                    target_behavior: "Target behavior is explicit.".to_string(),
                    scope: "Worker scope only.".to_string(),
                    non_goals: vec!["No unrelated cleanup.".to_string()],
                    constraints: vec!["Preserve the contract.".to_string()],
                    decisions: vec!["Use the existing Core path.".to_string()],
                    required_proof: "Focused test proof.".to_string(),
                    unknowns: Vec::new(),
                    confirmed: true,
                },
            )
            .unwrap();
            println!("INTENT_ID={}", intent.id);
        }
        "recovery" => {
            let recovery = record_recovery(
                &repo,
                &context,
                RecoveryInput {
                    outcome: RecoveryOutcome::Interrupted,
                    root_cause: format!("worker recovery {index}"),
                    last_successful_step: "Worker reached the barrier.".to_string(),
                    evidence: vec!["Concurrent test evidence.".to_string()],
                    affected_files: vec![format!("worker-{index}.rs")],
                    next_action: "Retry the worker safely.".to_string(),
                    retry_conditions: vec!["The project lock is available.".to_string()],
                },
            )
            .unwrap();
            println!("RECOVERY_ID={}", recovery.id);
        }
        "timeout" => {
            let error = record_proof(&repo, &context, "timeout worker proof").unwrap_err();
            assert!(error
                .to_string()
                .contains("Timed out waiting for Baron mutation lock"));
            println!("TIMEOUT_OK");
        }
        "plan-timeout" => {
            let title = "timeout plan worker";
            let session = "timeout-plan-session";
            let request = "timeout-plan-request";
            let identity = LifecycleIdentity::resolve(
                &context.project_id,
                title,
                SupportedAdapter::Codex,
                Some(session),
                Some(request),
            )
            .unwrap();
            let error =
                start_or_resume_plan_for_identity(&repo, &context, title, &identity).unwrap_err();
            assert!(error
                .to_string()
                .contains("Timed out waiting for Baron mutation lock"));
            println!("PLAN_TIMEOUT_OK");
        }
        "trace-timeout" => {
            let error = record_trace(
                &repo,
                &context,
                "timeout worker trace",
                TraceOutcome::Completed,
            )
            .unwrap_err();
            assert!(error
                .to_string()
                .contains("Timed out waiting for Baron mutation lock"));
            println!("TRACE_TIMEOUT_OK");
        }
        "plan" => {
            let title = format!("concurrent plan instance {index}");
            let session = format!("plan-session-{index}");
            let request = format!("plan-request-{index}");
            let identity = LifecycleIdentity::resolve(
                &context.project_id,
                &title,
                SupportedAdapter::Codex,
                Some(&session),
                Some(&request),
            )
            .unwrap();
            let plan =
                start_or_resume_plan_for_identity(&repo, &context, &title, &identity).unwrap();
            let relative = plan
                .repo_path
                .strip_prefix(&repo)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            println!("PLAN_ID={relative}");
        }
        other => panic!("unknown concurrency worker mode: {other}"),
    }
}
