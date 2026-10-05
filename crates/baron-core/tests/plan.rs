use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use baron_core::automation::{
    handle_hook, reconcile, reconcile_for_operation, AutomationEvent, HookAdapter,
};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::control_plane::record_gate_evidence_with_receipt_bound;
use baron_core::execution_receipt::{
    execute_command_for_identity, ExecutionRequest, ReceiptContext,
};
use baron_core::harness::{current_harness_title_for_operation, start_or_resume_intake};
use baron_core::intent::{record_intent, IntentBriefInput};
use baron_core::operation::{
    task_id_for_task, AuthoritativeLifecycleIdentity, LifecycleIdentity, OperationContext,
    SupportedAdapter,
};
use baron_core::plan::{
    active_plan_authority, active_plan_operation_binding, complete_plan,
    complete_plan_for_identity, interrupt_plan, interrupt_plan_for_identity, plan_status,
    start_or_resume_plan, start_or_resume_plan_for_identity, start_or_resume_plan_for_operation,
    update_plan, update_plan_for_identity,
};
use baron_core::proof::{
    record_proof, record_proof_for_operation, record_proof_from_receipt_bound,
};
use baron_core::trace::{
    record_trace, record_trace_for_operation, score_trace, TraceOperationBinding, TraceOutcome,
};
use baron_core::vault::{ensure_vault, vault_context_without_create, VaultContext};
use tempfile::{tempdir, TempDir};

fn setup_git(repo: &std::path::Path) {
    Command::new("git").arg("init").arg(repo).output().unwrap();
    Command::new("git")
        .args(["config", "user.email", "baron@example.test"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Baron Test"])
        .current_dir(repo)
        .output()
        .unwrap();
}

fn confirm_intent(repo: &std::path::Path, vault: &baron_core::vault::VaultContext, title: &str) {
    record_intent(
        repo,
        vault,
        IntentBriefInput {
            title: title.to_string(),
            current_behavior: "Current behavior recorded from project evidence.".to_string(),
            target_behavior: "Target behavior confirmed for this story.".to_string(),
            scope: "Only work required by this story.".to_string(),
            non_goals: vec!["No unrelated cleanup.".to_string()],
            constraints: vec!["Preserve existing contracts.".to_string()],
            decisions: vec!["Use the current architecture.".to_string()],
            required_proof: "Focused verification passes.".to_string(),
            unknowns: Vec::new(),
            confirmed: true,
        },
    )
    .unwrap();
}

fn passing_execution(
    repo: &std::path::Path,
    identity: &AuthoritativeLifecycleIdentity,
    gate_kind: &str,
) -> baron_core::execution_receipt::ExecutionReceipt {
    #[cfg(windows)]
    let (executable, arguments) = ("cmd", vec!["/C".to_string(), "exit 0".to_string()]);
    #[cfg(not(windows))]
    let (executable, arguments) = ("sh", vec!["-c".to_string(), "exit 0".to_string()]);
    execute_command_for_identity(
        ExecutionRequest {
            capability: "security-authorization".to_string(),
            provider: "test-runner".to_string(),
            executable: executable.to_string(),
            arguments,
            working_directory: repo.to_path_buf(),
            timeout: Duration::from_secs(5),
        },
        identity,
        gate_kind,
    )
    .unwrap()
}

fn passing_gate_execution(
    repo: &std::path::Path,
    agent: &str,
    identity: &AuthoritativeLifecycleIdentity,
) -> (
    baron_core::execution_receipt::ExecutionReceipt,
    ReceiptContext,
) {
    #[cfg(windows)]
    let (executable, arguments) = ("cmd", vec!["/C".to_string(), "exit 0".to_string()]);
    #[cfg(not(windows))]
    let (executable, arguments) = ("sh", vec!["-c".to_string(), "exit 0".to_string()]);
    let binding = ReceiptContext::new(
        identity.task_id(),
        identity.operation_id(),
        identity.adapter().as_str(),
        identity.session_id(),
        identity.request_id(),
        format!("quality:{agent}"),
    );
    let receipt = execute_command_for_identity(
        ExecutionRequest {
            capability: "security-authorization".to_string(),
            provider: "test-runner".to_string(),
            executable: executable.to_string(),
            arguments,
            working_directory: repo.to_path_buf(),
            timeout: Duration::from_secs(5),
        },
        identity,
        &binding.gate_kind,
    )
    .unwrap();
    (receipt, binding)
}

#[test]
fn start_creates_dated_plan_current_index_and_vault_mirror() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let plan = start_or_resume_plan(&repo, &context, "frontend HomePage").unwrap();

    assert!(plan.repo_path.exists());
    assert!(plan.vault_path.exists());
    assert!(!plan.resumed);
    assert!(fs::read_to_string(repo.join("docs/baron/plans/CURRENT.md"))
        .unwrap()
        .contains("frontend HomePage"));
    assert!(repo.join("docs/baron/plans/INDEX.md").exists());
}

#[test]
fn repeated_start_resumes_matching_plan() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let first = start_or_resume_plan(&repo, &context, "frontend HomePage").unwrap();
    let second = start_or_resume_plan(&repo, &context, "frontend HomePage").unwrap();

    assert_eq!(first.repo_path, second.repo_path);
    assert!(second.resumed);
}

#[test]
fn distinct_active_operations_resume_their_exact_plan_paths_after_current_changes() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let first_title = "backend operation alpha";
    let first_identity = LifecycleIdentity::resolve(
        &context.project_id,
        first_title,
        SupportedAdapter::Codex,
        Some("session-alpha"),
        Some("request-alpha"),
    )
    .unwrap();
    let second_title = "frontend operation beta";
    let second_identity = LifecycleIdentity::resolve(
        &context.project_id,
        second_title,
        SupportedAdapter::Claude,
        Some("session-beta"),
        Some("request-beta"),
    )
    .unwrap();

    let first =
        start_or_resume_plan_for_identity(&repo, &context, first_title, &first_identity).unwrap();
    let second =
        start_or_resume_plan_for_identity(&repo, &context, second_title, &second_identity).unwrap();
    let resumed_first =
        start_or_resume_plan_for_identity(&repo, &context, first_title, &first_identity).unwrap();
    let resumed_second =
        start_or_resume_plan_for_identity(&repo, &context, second_title, &second_identity).unwrap();

    assert_eq!(resumed_first.repo_path, first.repo_path);
    assert_eq!(resumed_second.repo_path, second.repo_path);
    assert!(resumed_first.resumed);
    assert!(resumed_second.resumed);

    let plan_files = fs::read_dir(first.repo_path.parent().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "md")
        })
        .count();
    assert_eq!(
        plan_files, 2,
        "distinct operations must not create A2/B2 plans"
    );

    let plan_index = fs::read_to_string(repo.join("docs/baron/plans/INDEX.md")).unwrap();
    for plan in [&first, &second] {
        let relative = plan
            .repo_path
            .strip_prefix(&repo)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        assert_eq!(plan_index.matches(&relative).count(), 1);
    }
    let active_index = fs::read_to_string(repo.join("docs/baron/plans/ACTIVE.md")).unwrap();
    for (identity, plan) in [(&first_identity, &first), (&second_identity, &second)] {
        let relative = plan
            .repo_path
            .strip_prefix(&repo)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        assert_eq!(active_index.matches(identity.operation_id()).count(), 1);
        assert_eq!(active_index.matches(&relative).count(), 1);
    }

    let current = fs::read_to_string(repo.join("docs/baron/plans/CURRENT.md")).unwrap();
    assert!(current.contains(
        &resumed_second
            .repo_path
            .strip_prefix(&repo)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/")
    ));
}

#[test]
fn shared_vault_active_plan_index_preserves_operations_from_other_checkouts() {
    let temp = tempdir().unwrap();
    let repo_a = temp.path().join("checkout-a");
    let repo_b = temp.path().join("checkout-b");
    let vault = temp.path().join("shared-vault");
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
    assert_eq!(
        context_a.project_root, context_b.project_root,
        "same persisted project identity must resolve to one Vault capsule across checkout names"
    );
    let read_only_context_b = vault_context_without_create(&vault, &repo_b).unwrap();
    assert_eq!(context_a.project_root, read_only_context_b.project_root);

    let identity_a = LifecycleIdentity::resolve(
        &context_a.project_id,
        "checkout A active plan",
        SupportedAdapter::Codex,
        Some("session-a"),
        Some("request-a"),
    )
    .unwrap();
    let identity_b = LifecycleIdentity::resolve(
        &context_b.project_id,
        "checkout B active plan",
        SupportedAdapter::Claude,
        Some("session-b"),
        Some("request-b"),
    )
    .unwrap();

    start_or_resume_plan_for_identity(&repo_a, &context_a, "checkout A active plan", &identity_a)
        .unwrap();
    start_or_resume_plan_for_identity(&repo_b, &context_b, "checkout B active plan", &identity_b)
        .unwrap();

    let vault_active_path = context_a.project_root.join("Plans/ACTIVE.md");
    let vault_active = fs::read_to_string(&vault_active_path).unwrap();
    assert!(vault_active.contains(identity_a.operation_id()));
    assert!(vault_active.contains(identity_b.operation_id()));

    update_plan_for_identity(&repo_b, &context_b, "B updated", &identity_b).unwrap();

    let vault_active = fs::read_to_string(vault_active_path).unwrap();
    assert!(vault_active.contains(identity_a.operation_id()));
    assert!(vault_active.contains(identity_b.operation_id()));
    assert!(vault_active.contains("\"status\":\"in_progress\""));

    let repo_a_active = fs::read_to_string(repo_a.join("docs/baron/plans/ACTIVE.md")).unwrap();
    let repo_b_active = fs::read_to_string(repo_b.join("docs/baron/plans/ACTIVE.md")).unwrap();
    assert!(repo_a_active.contains(identity_a.operation_id()));
    assert!(!repo_a_active.contains(identity_b.operation_id()));
    assert!(repo_b_active.contains(identity_b.operation_id()));
    assert!(!repo_b_active.contains(identity_a.operation_id()));
}

#[test]
fn interrupted_plan_status_transition_recovers_after_projection_write_failure() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let title = "fix README recover interrupted lifecycle";
    let identity = LifecycleIdentity::resolve(
        &context.project_id,
        title,
        SupportedAdapter::Codex,
        Some("recovery-session"),
        Some("recovery-request"),
    )
    .unwrap();
    let plan = start_or_resume_plan_for_identity(&repo, &context, title, &identity).unwrap();

    let current_path = repo.join("docs/baron/plans/CURRENT.md");
    fs::remove_file(&current_path).unwrap();
    fs::create_dir(&current_path).unwrap();
    let interrupted = interrupt_plan_for_identity(
        &repo,
        &context,
        "recover after mirror publication failure",
        &identity,
    )
    .unwrap_err();
    assert!(
        interrupted.to_string().contains("Could not write"),
        "{interrupted:#}"
    );
    let interrupted_plan = fs::read(&plan.repo_path).unwrap();
    assert!(String::from_utf8_lossy(&interrupted_plan).contains("status: interrupted"));
    assert!(repo.join(".baron/plan-transition.json").exists());
    assert!(fs::read_to_string(&plan.vault_path)
        .unwrap()
        .contains("status: interrupted"));

    fs::remove_dir(&current_path).unwrap();
    let completion =
        complete_plan_for_identity(&repo, &context, "restart recovery verification", &identity)
            .unwrap_err();

    assert!(
        completion.to_string().contains("Plan completion blocked"),
        "transition recovery should restore canonical ACTIVE authority before completion checks: {completion:#}"
    );
    let active = fs::read_to_string(repo.join("docs/baron/plans/ACTIVE.md")).unwrap();
    let active_vault = fs::read_to_string(context.project_root.join("Plans/ACTIVE.md")).unwrap();
    assert!(active.contains(identity.operation_id()));
    assert!(active.contains("\"status\":\"interrupted\""));
    assert_eq!(active, active_vault);
    assert!(!repo.join(".baron/plan-transition.json").exists());
}

#[test]
fn identified_update_must_not_follow_the_current_projection_red() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let first_title = "fix README alpha typo";
    let second_title = "fix README beta typo";
    let first_identity = LifecycleIdentity::resolve(
        &context.project_id,
        first_title,
        SupportedAdapter::Codex,
        Some("session-alpha"),
        Some("request-alpha"),
    )
    .unwrap();
    let second_identity = LifecycleIdentity::resolve(
        &context.project_id,
        second_title,
        SupportedAdapter::Claude,
        Some("session-beta"),
        Some("request-beta"),
    )
    .unwrap();
    let first =
        start_or_resume_plan_for_identity(&repo, &context, first_title, &first_identity).unwrap();
    let second =
        start_or_resume_plan_for_identity(&repo, &context, second_title, &second_identity).unwrap();
    let first_before = fs::read(&first.repo_path).unwrap();
    let second_before = fs::read(&second.repo_path).unwrap();

    update_plan_for_identity(&repo, &context, "alpha-only progress", &first_identity).unwrap();

    let first_after = fs::read(&first.repo_path).unwrap();
    let second_after = fs::read(&second.repo_path).unwrap();
    assert_ne!(first_before, first_after, "A must receive its own update");
    assert_eq!(second_before, second_after, "B must remain byte-identical");
}

#[test]
fn legacy_mutation_must_fail_closed_when_identified_operations_are_ambiguous_red() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    for (title, adapter, session, request) in [
        (
            "fix README alpha typo",
            SupportedAdapter::Codex,
            "session-alpha",
            "request-alpha",
        ),
        (
            "fix README beta typo",
            SupportedAdapter::Claude,
            "session-beta",
            "request-beta",
        ),
    ] {
        let identity = LifecycleIdentity::resolve(
            &context.project_id,
            title,
            adapter,
            Some(session),
            Some(request),
        )
        .unwrap();
        start_or_resume_plan_for_identity(&repo, &context, title, &identity).unwrap();
    }

    let error = update_plan(&repo, &context, "ambiguous legacy update").unwrap_err();
    assert!(error.to_string().contains("ambiguous"));
}

#[test]
fn operation_scoped_plan_mutations_keep_a_and_b_isolated_end_to_end() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let first_title = "fix README alpha typo";
    let second_title = "fix README beta typo";
    let first_identity = LifecycleIdentity::resolve(
        &context.project_id,
        first_title,
        SupportedAdapter::Codex,
        Some("session-alpha"),
        Some("request-alpha"),
    )
    .unwrap();
    let second_identity = LifecycleIdentity::resolve(
        &context.project_id,
        second_title,
        SupportedAdapter::Claude,
        Some("session-beta"),
        Some("request-beta"),
    )
    .unwrap();
    let first =
        start_or_resume_plan_for_identity(&repo, &context, first_title, &first_identity).unwrap();
    let second =
        start_or_resume_plan_for_identity(&repo, &context, second_title, &second_identity).unwrap();
    let resumed_first =
        start_or_resume_plan_for_identity(&repo, &context, first_title, &first_identity).unwrap();
    let resumed_second =
        start_or_resume_plan_for_identity(&repo, &context, second_title, &second_identity).unwrap();
    assert_eq!(first.repo_path, resumed_first.repo_path);
    assert_eq!(second.repo_path, resumed_second.repo_path);

    let second_before = fs::read(&second.repo_path).unwrap();
    update_plan_for_identity(&repo, &context, "alpha-only progress", &first_identity).unwrap();
    assert!(fs::read_to_string(&first.repo_path)
        .unwrap()
        .contains("alpha-only progress"));
    assert_eq!(fs::read(&second.repo_path).unwrap(), second_before);
    assert_eq!(
        fs::read_to_string(&first.repo_path).unwrap(),
        fs::read_to_string(&first.vault_path).unwrap()
    );
    assert_eq!(
        fs::read_to_string(&second.repo_path).unwrap(),
        fs::read_to_string(&second.vault_path).unwrap()
    );

    interrupt_plan_for_identity(
        &repo,
        &context,
        "alpha paused for another session",
        &first_identity,
    )
    .unwrap();
    assert!(fs::read_to_string(&first.repo_path)
        .unwrap()
        .contains("status: interrupted"));
    assert!(fs::read_to_string(&second.repo_path)
        .unwrap()
        .contains("status: in_progress"));

    update_plan_for_identity(&repo, &context, "beta-only progress", &second_identity).unwrap();
    assert!(fs::read_to_string(&second.repo_path)
        .unwrap()
        .contains("beta-only progress"));
    assert!(fs::read_to_string(&first.repo_path)
        .unwrap()
        .contains("status: interrupted"));

    let second_operation = OperationContext::from_identity(&second_identity);
    let proof = record_proof_for_operation(
        &repo,
        &context,
        &second_operation,
        "Beta README verification passed",
    )
    .unwrap();
    let trace_binding =
        TraceOperationBinding::from_operation(&second_operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "Beta README task completed",
        TraceOutcome::Completed,
        &trace_binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );
    complete_plan_for_identity(
        &repo,
        &context,
        "Beta README verification passed",
        &second_identity,
    )
    .unwrap();

    let first_after = fs::read_to_string(&first.repo_path).unwrap();
    let second_after = fs::read_to_string(&second.repo_path).unwrap();
    assert!(first_after.contains("status: interrupted"));
    assert!(second_after.contains("status: completed"));
    assert!(second_after.contains("Beta README verification passed"));
    assert_eq!(
        second_after,
        fs::read_to_string(&second.vault_path).unwrap()
    );
    let active = fs::read_to_string(repo.join("docs/baron/plans/ACTIVE.md")).unwrap();
    let active_vault = fs::read_to_string(context.project_root.join("Plans/ACTIVE.md")).unwrap();
    assert_eq!(active, active_vault);
    assert!(active.contains(&first_identity.operation_id().to_string()));
    assert!(active.contains(&second_identity.operation_id().to_string()));
    assert!(active.contains("\"status\":\"interrupted\""));
    assert!(active.contains("\"status\":\"completed\""));
    let current = fs::read_to_string(repo.join("docs/baron/plans/CURRENT.md")).unwrap();
    assert!(current.contains(second_title));
    assert!(current.contains("Status: `completed`"));
    assert_eq!(
        fs::read_dir(first.repo_path.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().is_some_and(|value| value == "md"))
            .count(),
        2,
        "A/B mutation must not create A2/B2 artifacts"
    );
}

#[test]
fn identified_mutation_requires_active_index_not_only_legacy_frontmatter() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("missing-index");
    fs::create_dir_all(&repo).unwrap();
    let vault = ensure_vault(temp.path().join("Vault"), &repo).unwrap();
    let identity = LifecycleIdentity::resolve(
        &vault.project_id,
        "fix README typo",
        SupportedAdapter::Codex,
        Some("indexed-session"),
        Some("indexed-request"),
    )
    .unwrap();
    let plan =
        start_or_resume_plan_for_identity(&repo, &vault, "fix README typo", &identity).unwrap();
    fs::remove_file(repo.join("docs/baron/plans/ACTIVE.md")).unwrap();
    let before = fs::read(&plan.repo_path).unwrap();
    let current = fs::read(repo.join("docs/baron/plans/CURRENT.md")).unwrap();
    let result = update_plan_for_identity(&repo, &vault, "must not publish", &identity);
    assert!(
        result.is_err(),
        "unindexed frontmatter authorized a mutation"
    );
    assert!(result.unwrap_err().to_string().contains("ACTIVE"));
    assert!(interrupt_plan_for_identity(&repo, &vault, "must not interrupt", &identity).is_err());
    assert!(update_plan(&repo, &vault, "legacy must not publish").is_err());
    assert_eq!(fs::read(&plan.repo_path).unwrap(), before);
    assert_eq!(
        fs::read(repo.join("docs/baron/plans/CURRENT.md")).unwrap(),
        current
    );
    // An explicit start/resume can safely migrate an old indexed-less plan
    // under the lock, after which mutation is authorized again.
    let restored =
        start_or_resume_plan_for_identity(&repo, &vault, "fix README typo", &identity).unwrap();
    assert_eq!(restored.repo_path, plan.repo_path);
    update_plan_for_identity(
        &repo,
        &vault,
        "safe after authority registration",
        &identity,
    )
    .unwrap();
}

#[test]
fn completed_operation_identity_cannot_reopen_a_plan() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let title = "fix README completion alpha";
    let identity = LifecycleIdentity::resolve(
        &context.project_id,
        title,
        SupportedAdapter::Codex,
        Some("completion-session"),
        Some("completion-turn"),
    )
    .unwrap();
    let plan = start_or_resume_plan_for_identity(&repo, &context, title, &identity).unwrap();
    let operation = OperationContext::from_identity(&identity);
    let proof = record_proof_for_operation(
        &repo,
        &context,
        &operation,
        "README completion verification passed",
    )
    .unwrap();
    let trace_binding = TraceOperationBinding::from_operation(&operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "README completion task finished",
        TraceOutcome::Completed,
        &trace_binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );
    complete_plan_for_identity(
        &repo,
        &context,
        "README completion verification passed",
        &identity,
    )
    .unwrap();

    let active_index = fs::read_to_string(repo.join("docs/baron/plans/ACTIVE.md")).unwrap();
    let completed_plan = fs::read_to_string(&plan.repo_path).unwrap();
    let plan_count = fs::read_dir(plan.repo_path.parent().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "md")
        })
        .count();

    let error = start_or_resume_plan_for_identity(&repo, &context, title, &identity).unwrap_err();

    assert!(error.to_string().contains("completed"), "{error:#}");
    assert_eq!(
        fs::read_to_string(repo.join("docs/baron/plans/ACTIVE.md")).unwrap(),
        active_index,
        "replaying a completed lifecycle identity must not rewrite ACTIVE authority"
    );
    assert_eq!(fs::read_to_string(&plan.repo_path).unwrap(), completed_plan);
    assert_eq!(
        fs::read_dir(plan.repo_path.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "md"))
            .count(),
        plan_count,
        "a completed lifecycle identity must not create a replacement plan"
    );
}

#[test]
fn sole_active_operation_ignores_stale_current_for_binding_and_authority() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let active_title = "fix README alpha typo";
    let completed_title = "fix README beta typo";
    let active_identity = LifecycleIdentity::resolve(
        &context.project_id,
        active_title,
        SupportedAdapter::Codex,
        Some("active-session"),
        Some("active-turn"),
    )
    .unwrap();
    let completed_identity = LifecycleIdentity::resolve(
        &context.project_id,
        completed_title,
        SupportedAdapter::Claude,
        Some("completed-session"),
        Some("completed-turn"),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &context, active_title, &active_identity).unwrap();
    start_or_resume_plan_for_identity(&repo, &context, completed_title, &completed_identity)
        .unwrap();
    let completed_operation = OperationContext::from_identity(&completed_identity);
    let proof = record_proof_for_operation(
        &repo,
        &context,
        &completed_operation,
        "Beta README verification passed",
    )
    .unwrap();
    let trace_binding =
        TraceOperationBinding::from_operation(&completed_operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "Beta README task finished",
        TraceOutcome::Completed,
        &trace_binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );
    complete_plan_for_identity(
        &repo,
        &context,
        "Beta README verification passed",
        &completed_identity,
    )
    .unwrap();
    assert!(fs::read_to_string(repo.join("docs/baron/plans/CURRENT.md"))
        .unwrap()
        .contains("Status: `completed`"));

    let current_path = repo.join("docs/baron/plans/CURRENT.md");
    let current_projection = fs::read_to_string(&current_path).unwrap();
    let current_plan_line = current_projection
        .lines()
        .find(|line| line.starts_with("- Plan: `"))
        .unwrap();
    let stale_projection = current_projection.replace(
        current_plan_line,
        "- Plan: `docs/baron/plans/missing-current-projection.md`",
    );
    fs::write(&current_path, stale_projection).unwrap();

    let authority = active_plan_authority(&repo).unwrap().unwrap();
    assert_eq!(
        authority.binding.unwrap().operation_id,
        active_identity.operation_id(),
        "CURRENT is presentation-only and must not veto Core authority for the sole active operation"
    );

    let active_operation = OperationContext::from_identity(&active_identity);
    record_proof_for_operation(
        &repo,
        &context,
        &active_operation,
        "Alpha README verification passed",
    )
    .unwrap();
    let trace = record_trace(
        &repo,
        &context,
        "Alpha README lifecycle finished",
        TraceOutcome::Completed,
    )
    .unwrap();
    assert_eq!(
        trace.binding.as_ref().unwrap().operation_id,
        active_identity.operation_id(),
        "legacy trace ingress must bind to the sole active operation, not completed CURRENT"
    );
    let score = score_trace(&repo, &context, None).unwrap();
    assert_eq!(
        score.binding.as_ref().unwrap().operation_id,
        active_identity.operation_id(),
        "auto trace scoring must use the sole active operation, not completed CURRENT"
    );
    let completion_status = baron_core::plan::active_plan_completion_evidence_status(&repo)
        .unwrap()
        .unwrap();
    assert!(
        !completion_status
            .issues
            .iter()
            .any(|issue| issue.contains("CURRENT")),
        "completion/reconciliation status must not depend on a stale CURRENT projection: {:?}",
        completion_status.issues
    );

    let binding = active_plan_operation_binding(&repo).unwrap().unwrap();

    assert_eq!(
        binding.operation_id,
        active_identity.operation_id(),
        "CURRENT is a presentation pointer and must not veto the sole validated active operation"
    );

    update_plan(
        &repo,
        &context,
        "Alpha resumed after a stale CURRENT projection",
    )
    .unwrap();
}

#[test]
fn operation_trace_uses_a_plan_when_current_points_to_b() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let first_title = "fix README alpha typo";
    let second_title = "backend login security";
    let first_identity = LifecycleIdentity::resolve(
        &context.project_id,
        first_title,
        SupportedAdapter::Codex,
        Some("session-alpha"),
        Some("request-alpha"),
    )
    .unwrap();
    let second_identity = LifecycleIdentity::resolve(
        &context.project_id,
        second_title,
        SupportedAdapter::Claude,
        Some("session-beta"),
        Some("request-beta"),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &context, first_title, &first_identity).unwrap();
    start_or_resume_plan_for_identity(&repo, &context, second_title, &second_identity).unwrap();
    let first_operation = OperationContext::from_identity(&first_identity);
    let proof = record_proof_for_operation(
        &repo,
        &context,
        &first_operation,
        "Alpha README verification passed",
    )
    .unwrap();
    let binding = TraceOperationBinding::from_operation(&first_operation, &proof.id).unwrap();

    let trace = record_trace_for_operation(
        &repo,
        &context,
        "Alpha README task completed",
        TraceOutcome::Completed,
        &binding,
    )
    .unwrap();
    let content = fs::read_to_string(trace.repo_path).unwrap();
    assert!(content.contains("- Risk: `low`"));
    assert!(content.contains(&format!("- Current plan: `{first_title}`")));
    assert!(!content.contains(second_title));
    assert!(content.contains(&format!(
        "- Operation ID: `{}`",
        first_identity.operation_id()
    )));
}

#[test]
fn active_index_integrity_mismatch_fails_closed_before_operation_mutation() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let title = "fix README alpha typo";
    let identity = LifecycleIdentity::resolve(
        &context.project_id,
        title,
        SupportedAdapter::Codex,
        Some("session-alpha"),
        Some("request-alpha"),
    )
    .unwrap();
    let plan = start_or_resume_plan_for_identity(&repo, &context, title, &identity).unwrap();
    let active_path = repo.join("docs/baron/plans/ACTIVE.md");
    let active = fs::read_to_string(&active_path).unwrap();
    let entry = active
        .lines()
        .find(|line| line.contains(identity.operation_id()))
        .unwrap();
    fs::write(&active_path, format!("{active}{entry}\n")).unwrap();
    let before = fs::read(&plan.repo_path).unwrap();
    let error = update_plan_for_identity(&repo, &context, "must not write", &identity).unwrap_err();
    assert!(error.to_string().contains("duplicate operation"));
    assert_eq!(fs::read(&plan.repo_path).unwrap(), before);
}

#[test]
fn identified_plan_persists_exact_identity_and_rejects_hijack() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = LifecycleIdentity::resolve(
        &context.project_id,
        "frontend dashboard",
        SupportedAdapter::Codex,
        Some("session-dashboard"),
        Some("request-dashboard-1"),
    )
    .unwrap();

    let first = start_or_resume_plan_for_identity(&repo, &context, "frontend dashboard", &identity)
        .unwrap();
    let repo_content = fs::read_to_string(&first.repo_path).unwrap();
    let current_content = fs::read_to_string(repo.join("docs/baron/plans/CURRENT.md")).unwrap();
    let vault_content = fs::read_to_string(&first.vault_path).unwrap();
    for content in [&repo_content, &vault_content] {
        assert!(content.contains(&format!("task_id: {}", identity.task_id())));
        assert!(content.contains(&format!("operation_id: {}", identity.operation_id())));
        assert!(content.contains("adapter: codex"));
        assert!(content.contains("session_id: session-dashboard"));
        assert!(content.contains("request_id: request-dashboard-1"));
    }
    assert!(current_content.contains(&format!("Task ID: `{}`", identity.task_id())));
    assert!(current_content.contains(&format!("Operation ID: `{}`", identity.operation_id())));
    assert!(current_content.contains("Adapter: `codex`"));
    assert!(current_content.contains("Session ID: `session-dashboard`"));
    assert!(current_content.contains("Request ID: `request-dashboard-1`"));

    let resumed =
        start_or_resume_plan_for_identity(&repo, &context, "frontend dashboard", &identity)
            .unwrap();
    assert!(resumed.resumed);

    let hijacker = LifecycleIdentity::resolve(
        &context.project_id,
        "frontend dashboard",
        SupportedAdapter::Codex,
        Some("session-dashboard"),
        Some("request-dashboard-2"),
    )
    .unwrap();
    let error = start_or_resume_plan_for_identity(&repo, &context, "frontend dashboard", &hijacker)
        .unwrap_err();
    assert!(error.to_string().contains("different operation identity"));
}

#[test]
fn incomplete_operation_cannot_write_a_plan() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let error = start_or_resume_plan_for_operation(
        &repo,
        &context,
        "frontend dashboard",
        &OperationContext::new(SupportedAdapter::Codex),
    )
    .unwrap_err();

    assert!(error.to_string().contains("session_id"));
    assert!(!repo.join("docs/baron/plans/CURRENT.md").exists());
    assert!(fs::read_dir(context.project_root.join("Plans"))
        .unwrap()
        .next()
        .is_none());
}

#[test]
fn forged_operation_context_cannot_write_a_plan() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let task_id =
        baron_core::operation::task_id_for_task(&context.project_id, "frontend dashboard").unwrap();
    let operation = OperationContext::new(SupportedAdapter::Codex)
        .with_task_id(task_id)
        .with_operation_id("operation-FAKE")
        .with_session_id("session-a")
        .with_request_id("request-a");

    let error =
        start_or_resume_plan_for_operation(&repo, &context, "frontend dashboard", &operation)
            .unwrap_err();

    assert!(error.to_string().contains("does not match"));
    assert!(!repo.join("docs/baron/plans/CURRENT.md").exists());
    assert!(fs::read_dir(context.project_root.join("Plans"))
        .unwrap()
        .next()
        .is_none());
}

#[test]
fn task_id_mismatch_is_rejected_before_plan_writes() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = LifecycleIdentity::resolve(
        &context.project_id,
        "different canonical task",
        SupportedAdapter::Claude,
        Some("session-a"),
        Some("request-a"),
    )
    .unwrap();

    let error =
        start_or_resume_plan_for_identity(&repo, &context, "requested plan task", &identity)
            .unwrap_err();

    assert!(error.to_string().contains("task_id"));
    assert!(!repo.join("docs/baron/plans/CURRENT.md").exists());
    assert!(fs::read_dir(context.project_root.join("Plans"))
        .unwrap()
        .next()
        .is_none());
}

#[test]
fn wrong_project_identity_is_rejected_before_plan_writes() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = LifecycleIdentity::resolve(
        "other-project",
        "frontend dashboard",
        SupportedAdapter::Codex,
        Some("session-a"),
        Some("request-a"),
    )
    .unwrap();

    let error = start_or_resume_plan_for_identity(&repo, &context, "frontend dashboard", &identity)
        .unwrap_err();

    assert!(error.to_string().contains("does not match Vault project"));
    assert!(!repo.join("docs/baron/plans/CURRENT.md").exists());
    assert!(fs::read_dir(context.project_root.join("Plans"))
        .unwrap()
        .next()
        .is_none());
}

#[test]
fn identified_start_cannot_authorize_a_legacy_unbound_plan() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let legacy = start_or_resume_plan(&repo, &context, "frontend dashboard").unwrap();
    let identity = LifecycleIdentity::resolve(
        &context.project_id,
        "frontend dashboard",
        SupportedAdapter::Claude,
        Some("session-a"),
        Some("request-a"),
    )
    .unwrap();

    let error = start_or_resume_plan_for_identity(&repo, &context, "frontend dashboard", &identity)
        .unwrap_err();

    assert!(error.to_string().contains("legacy unbound plan"));
    assert_eq!(legacy.repo_path, active_plan_path(&repo));
    let current = fs::read_to_string(repo.join("docs/baron/plans/CURRENT.md")).unwrap();
    assert!(!current.contains("Operation ID:"));
}

fn active_plan_path(repo: &std::path::Path) -> std::path::PathBuf {
    let current = fs::read_to_string(repo.join("docs/baron/plans/CURRENT.md")).unwrap();
    let path = current
        .lines()
        .find_map(|line| line.strip_prefix("- Plan: `"))
        .and_then(|value| value.strip_suffix('`'))
        .unwrap();
    repo.join(path)
}

fn rewrite_current_plan_pointer(repo: &Path, relative_path: &str) {
    let current_path = repo.join("docs/baron/plans/CURRENT.md");
    let current = fs::read_to_string(&current_path).unwrap();
    let rewritten = current
        .lines()
        .map(|line| {
            if line.starts_with("- Plan: `") {
                format!("- Plan: `{relative_path}`")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(current_path, format!("{rewritten}\n")).unwrap();
}

fn repo_relative_path(repo: &Path, path: &Path) -> String {
    path.strip_prefix(repo)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}

fn snapshot_tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, current: &Path, snapshot: &mut BTreeMap<PathBuf, Vec<u8>>) {
        let entries = fs::read_dir(current).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            if metadata.is_dir() {
                visit(root, &path, snapshot);
            } else if metadata.is_file() {
                snapshot.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }

    let mut snapshot = BTreeMap::new();
    if root.is_dir() {
        visit(root, root, &mut snapshot);
    }
    snapshot
}

fn assert_outside_plan_pointer_is_ignored_for_identified_operation(relative_path: &str) {
    let (_temp, repo, context, identity, plan_path) = identified_plan_fixture(
        "backend login security",
        SupportedAdapter::Codex,
        "active-session",
        "active-request",
    );
    let target = repo.join(relative_path);
    let linked = fs::read(&plan_path).unwrap();
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, &linked).unwrap();
    let target_before = fs::read(&target).unwrap();
    let linked_before = fs::read(&plan_path).unwrap();
    rewrite_current_plan_pointer(&repo, relative_path);

    let authority = active_plan_authority(&repo).unwrap().unwrap();
    assert_eq!(
        authority.binding.unwrap().operation_id,
        identity.operation_id(),
        "CURRENT must not replace the exact ACTIVE/frontmatter operation"
    );
    let reconciliation = reconcile(&repo).unwrap();
    assert!(!reconciliation.passed);
    assert!(reconciliation
        .gaps
        .iter()
        .all(|gap| !gap.contains("CURRENT")));
    assert_stop_blocks(&repo, &context, "backend login security", &identity);
    update_plan(&repo, &context, "canonical operation continued").unwrap();

    assert_eq!(fs::read(target).unwrap(), target_before);
    assert_ne!(fs::read(&plan_path).unwrap(), linked_before);
}

#[test]
fn outside_current_plan_pointers_cannot_replace_active_authority_or_mutate_targets() {
    assert_outside_plan_pointer_is_ignored_for_identified_operation("README.md");
    assert_outside_plan_pointer_is_ignored_for_identified_operation("src/fake.md");
    assert_outside_plan_pointer_is_ignored_for_identified_operation("docs/fake-plan.md");
    assert_outside_plan_pointer_is_ignored_for_identified_operation(
        "docs/baron/plans-evil/fake.md",
    );
}

#[test]
fn current_pointer_without_type_marker_cannot_veto_active_authority() {
    let (_temp, repo, _context, identity, plan_path) = identified_plan_fixture(
        "backend login security",
        SupportedAdapter::Codex,
        "active-session",
        "active-request",
    );
    let target = repo.join("docs/baron/plans/2099/fake.md");
    let content = fs::read_to_string(&plan_path)
        .unwrap()
        .lines()
        .filter(|line| *line != "type: baron-plan")
        .collect::<Vec<_>>()
        .join("\n");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, format!("{content}\n")).unwrap();
    rewrite_current_plan_pointer(&repo, &repo_relative_path(&repo, &target));

    assert_eq!(
        active_plan_authority(&repo)
            .unwrap()
            .unwrap()
            .binding
            .unwrap()
            .operation_id,
        identity.operation_id()
    );
    let reconciliation = reconcile(&repo).unwrap();
    assert!(!reconciliation.passed);
    assert!(reconciliation
        .gaps
        .iter()
        .all(|gap| !gap.contains("CURRENT")));
}

#[test]
fn current_pointer_body_only_metadata_cannot_veto_active_authority() {
    let (_temp, repo, _context, identity, plan_path) = identified_plan_fixture(
        "backend login security",
        SupportedAdapter::Codex,
        "active-session",
        "active-request",
    );
    let target = repo.join("docs/baron/plans/2099/body-only.md");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    let linked = fs::read_to_string(&plan_path).unwrap();
    let frontmatter = linked
        .strip_prefix("---\n")
        .and_then(|value| value.split_once("\n---\n"))
        .map(|(fields, _)| fields)
        .unwrap();
    fs::write(&target, format!("# Fake\n\n{frontmatter}\n")).unwrap();
    rewrite_current_plan_pointer(&repo, &repo_relative_path(&repo, &target));

    assert_eq!(
        active_plan_authority(&repo)
            .unwrap()
            .unwrap()
            .binding
            .unwrap()
            .operation_id,
        identity.operation_id()
    );
    let reconciliation = reconcile(&repo).unwrap();
    assert!(!reconciliation.passed);
    assert!(reconciliation
        .gaps
        .iter()
        .all(|gap| !gap.contains("CURRENT")));
}

#[test]
fn duplicate_risk_in_managed_plan_frontmatter_fails_closed() {
    let (_temp, repo, _context, _identity, plan_path) = identified_plan_fixture(
        "backend login security",
        SupportedAdapter::Codex,
        "active-session",
        "active-request",
    );
    let target = repo.join("docs/baron/plans/2099/duplicate-risk.md");
    let linked = fs::read_to_string(&plan_path).unwrap();
    let content = linked.replacen("\n---\n\n# ", "\nrisk: low\n---\n\n# ", 1);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, content).unwrap();
    rewrite_current_plan_pointer(&repo, &repo_relative_path(&repo, &target));

    assert!(active_plan_authority(&repo).is_err());
    assert!(!reconcile(&repo).unwrap().passed);
}

#[test]
fn duplicate_operation_id_in_managed_plan_frontmatter_fails_closed() {
    let (_temp, repo, _context, _identity, plan_path) = identified_plan_fixture(
        "backend login security",
        SupportedAdapter::Codex,
        "active-session",
        "active-request",
    );
    let target = repo.join("docs/baron/plans/2099/duplicate-operation.md");
    let linked = fs::read_to_string(&plan_path).unwrap();
    let content = linked.replacen(
        "\n---\n\n# ",
        "\noperation_id: forged-operation\n---\n\n# ",
        1,
    );
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, content).unwrap();
    rewrite_current_plan_pointer(&repo, &repo_relative_path(&repo, &target));

    assert!(active_plan_authority(&repo).is_err());
    assert!(!reconcile(&repo).unwrap().passed);
}

#[cfg(unix)]
#[test]
fn current_symlink_pointer_cannot_veto_active_authority() {
    use std::os::unix::fs::symlink;

    let (_temp, repo, _context, identity, plan_path) = identified_plan_fixture(
        "backend login security",
        SupportedAdapter::Codex,
        "active-session",
        "active-request",
    );
    let target = repo.join("docs/baron/plans/2099/evil.md");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    symlink(repo.join("README.md"), &target).unwrap();
    rewrite_current_plan_pointer(&repo, &repo_relative_path(&repo, &target));

    assert_eq!(
        active_plan_authority(&repo)
            .unwrap()
            .unwrap()
            .binding
            .unwrap()
            .operation_id,
        identity.operation_id()
    );
    let reconciliation = reconcile(&repo).unwrap();
    assert!(!reconciliation.passed);
    assert!(reconciliation
        .gaps
        .iter()
        .all(|gap| !gap.contains("CURRENT")));
    assert!(complete_plan(&repo, &_context, "verification attempted").is_err());
    assert!(fs::read(&plan_path).is_ok());
}

#[cfg(windows)]
#[test]
fn current_reparse_or_symlink_pointer_cannot_veto_active_authority() {
    use std::os::windows::fs::symlink_file;

    let (_temp, repo, _context, identity, plan_path) = identified_plan_fixture(
        "backend login security",
        SupportedAdapter::Codex,
        "active-session",
        "active-request",
    );
    let target = repo.join("docs/baron/plans/2099/evil.md");
    let outside = repo.join("outside");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(repo.join("README.md"), fs::read(&plan_path).unwrap()).unwrap();
    let linked_file = symlink_file(repo.join("README.md"), &target).is_ok();
    let linked = if linked_file {
        true
    } else {
        let script = format!(
            "New-Item -ItemType Junction -Path '{}' -Target '{}' | Out-Null",
            target.display().to_string().replace('\'', "''"),
            outside.display().to_string().replace('\'', "''")
        );
        Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()
            .map(|status| status.success() && target.is_dir())
            .unwrap_or(false)
    };
    if !linked {
        return;
    }
    rewrite_current_plan_pointer(&repo, &repo_relative_path(&repo, &target));

    assert_eq!(
        active_plan_authority(&repo)
            .unwrap()
            .unwrap()
            .binding
            .unwrap()
            .operation_id,
        identity.operation_id()
    );
    let reconciliation = reconcile(&repo).unwrap();
    assert!(!reconciliation.passed);
    assert!(reconciliation
        .gaps
        .iter()
        .all(|gap| !gap.contains("CURRENT")));
    assert!(complete_plan(&repo, &_context, "verification attempted").is_err());
    assert!(fs::read(&plan_path).is_ok());
}

fn identified_plan_fixture(
    title: &str,
    adapter: SupportedAdapter,
    session_id: &str,
    request_id: &str,
) -> (
    TempDir,
    PathBuf,
    VaultContext,
    AuthoritativeLifecycleIdentity,
    PathBuf,
) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        title,
        adapter,
        Some(session_id),
        Some(request_id),
    )
    .unwrap();
    let plan = start_or_resume_plan_for_identity(&repo, &context, title, &identity).unwrap();
    (temp, repo, context, identity, plan.repo_path)
}

fn assert_stop_blocks(
    repo: &Path,
    context: &VaultContext,
    title: &str,
    identity: &AuthoritativeLifecycleIdentity,
) {
    let adapter = match identity.adapter() {
        SupportedAdapter::Codex => HookAdapter::Codex,
        SupportedAdapter::Claude => HookAdapter::Claude,
    };
    let stop = handle_hook(
        repo,
        context,
        adapter,
        AutomationEvent::Stop,
        &format!(
            r#"{{"task":"{title}","session_id":"{}","request_id":"{}","stop_hook_active":false}}"#,
            identity.session_id(),
            identity.request_id()
        ),
    )
    .unwrap();
    assert!(
        stop.contains(r#""decision":"block""#),
        "Stop hook unexpectedly allowed the mismatched plan: {stop}"
    );
}

fn assert_current_metadata_mismatch_is_presentation_only<F>(mutate: F)
where
    F: FnOnce(String, &AuthoritativeLifecycleIdentity) -> String,
{
    let (_temp, repo, context, identity, plan_path) = identified_plan_fixture(
        "backend login security",
        SupportedAdapter::Codex,
        "active-session",
        "active-request",
    );
    let current_path = repo.join("docs/baron/plans/CURRENT.md");
    let current = fs::read_to_string(&current_path).unwrap();
    fs::write(&current_path, mutate(current, &identity)).unwrap();

    let authority = active_plan_authority(&repo).unwrap().unwrap();
    assert_eq!(
        authority.binding.unwrap().operation_id,
        identity.operation_id(),
        "CURRENT metadata cannot veto or substitute the exact operation authority"
    );
    let error = complete_plan(&repo, &context, "verification attempted").unwrap_err();
    assert!(!error.to_string().contains("authority mismatch"));
    let reconciliation = reconcile(&repo).unwrap();
    assert!(!reconciliation.passed);
    assert!(reconciliation
        .gaps
        .iter()
        .all(|gap| !gap.contains("CURRENT")));
    assert_stop_blocks(&repo, &context, "backend login security", &identity);
    assert!(fs::read_to_string(plan_path)
        .unwrap()
        .contains("status: in_progress"));
}

fn rewrite_current_status(repo: &Path, status: Option<&str>) {
    let path = repo.join("docs/baron/plans/CURRENT.md");
    let current = fs::read_to_string(&path).unwrap();
    let lines = current
        .lines()
        .filter(|line| !line.starts_with("- Status: "))
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut rewritten = lines;
    if let Some(status) = status {
        let index = rewritten
            .iter()
            .position(|line| line.starts_with("- Plan: "))
            .unwrap()
            + 1;
        rewritten.insert(index, format!("- Status: `{status}`"));
    }
    fs::write(path, format!("{}\n", rewritten.join("\n"))).unwrap();
}

fn rewrite_linked_status(path: &Path, status: &str) {
    let content = fs::read_to_string(path).unwrap();
    let rewritten = content
        .lines()
        .map(|line| {
            if line.starts_with("status: ") {
                format!("status: {status}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(path, format!("{rewritten}\n")).unwrap();
}

fn assert_linked_canonical_tamper_blocks<F, G>(mutate_current: F, mutate_linked: G)
where
    F: FnOnce(String, &AuthoritativeLifecycleIdentity) -> String,
    G: FnOnce(String, &AuthoritativeLifecycleIdentity) -> String,
{
    let (_temp, repo, context, identity, plan_path) = identified_plan_fixture(
        "backend login security",
        SupportedAdapter::Codex,
        "active-session",
        "active-request",
    );
    let operation = OperationContext::from_identity(&identity);
    let receipt = passing_execution(&repo, &identity, "proof");
    let proof_binding = ReceiptContext::new(
        identity.task_id(),
        identity.operation_id(),
        identity.adapter().as_str(),
        identity.session_id(),
        identity.request_id(),
        "proof",
    );
    let proof =
        record_proof_from_receipt_bound(&repo, &context, &receipt.receipt_id, &proof_binding)
            .unwrap();
    let binding = TraceOperationBinding::from_operation(&operation, &proof.id).unwrap();
    let current_path = repo.join("docs/baron/plans/CURRENT.md");
    let current = fs::read_to_string(&current_path).unwrap();
    let linked = fs::read_to_string(&plan_path).unwrap();
    fs::write(&current_path, mutate_current(current, &identity)).unwrap();
    fs::write(&plan_path, mutate_linked(linked, &identity)).unwrap();

    let error = record_trace_for_operation(
        &repo,
        &context,
        "security trace after metadata tamper",
        TraceOutcome::Completed,
        &binding,
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("canonical")
            || error.to_string().contains("authority")
            || error.to_string().contains("incomplete"),
        "unexpected tamper error: {error}"
    );
    assert!(!reconcile(&repo).unwrap().passed);
    assert_stop_blocks(&repo, &context, "backend login security", &identity);
    assert!(fs::read_to_string(plan_path)
        .unwrap()
        .contains("status: in_progress"));
}

#[test]
fn current_completed_cannot_hide_linked_in_progress_plan() {
    let (_temp, repo, context, identity, plan_path) = identified_plan_fixture(
        "backend login security",
        SupportedAdapter::Codex,
        "active-session",
        "active-request",
    );
    rewrite_current_status(&repo, Some("completed"));

    let report = reconcile(&repo).unwrap();
    assert!(report.active_plan);
    assert!(!report.passed);
    assert_stop_blocks(&repo, &context, "backend login security", &identity);
    assert!(plan_status(&repo)
        .unwrap()
        .contains("Completion integrity: `failed`"));
    assert!(fs::read_to_string(plan_path)
        .unwrap()
        .contains("status: in_progress"));
}

#[test]
fn current_active_cannot_hide_linked_completed_plan() {
    let (_temp, repo, context, identity, plan_path) = identified_plan_fixture(
        "backend login security",
        SupportedAdapter::Codex,
        "active-session",
        "active-request",
    );
    rewrite_linked_status(&plan_path, "completed");

    let report = reconcile(&repo).unwrap();
    assert!(report.active_plan);
    assert!(!report.passed);
    assert_stop_blocks(&repo, &context, "backend login security", &identity);
    let status = plan_status(&repo).unwrap();
    assert!(status.contains("Completion integrity: `failed`"));
    assert!(status.contains("status"));
    assert!(fs::read_to_string(plan_path)
        .unwrap()
        .contains("status: completed"));
}

#[test]
fn malformed_current_status_cannot_hide_linked_active_plan() {
    for status in [Some("unknown"), None] {
        let (_temp, repo, context, identity, plan_path) = identified_plan_fixture(
            "backend login security",
            SupportedAdapter::Codex,
            "active-session",
            "active-request",
        );
        rewrite_current_status(&repo, status);

        let report = reconcile(&repo).unwrap();
        assert!(report.active_plan);
        assert!(!report.passed);
        assert_stop_blocks(&repo, &context, "backend login security", &identity);
        assert!(plan_status(&repo)
            .unwrap()
            .contains("Completion integrity: `failed`"));
        assert!(fs::read_to_string(plan_path)
            .unwrap()
            .contains("status: in_progress"));
    }
}

#[test]
fn linked_risk_must_match_the_canonical_classifier() {
    assert_linked_canonical_tamper_blocks(
        |current, _| current.replace("- Risk: `high`", "- Risk: `low`"),
        |linked, _| linked.replace("risk: high", "risk: low"),
    );
}

#[test]
fn linked_task_id_must_match_the_canonical_task() {
    assert_linked_canonical_tamper_blocks(
        |current, _| current.replace("- Task ID: `", "- Task ID: `forged-"),
        |linked, _| linked.replace("task_id: ", "task_id: forged-"),
    );
}

#[test]
fn linked_operation_id_must_match_the_canonical_tuple() {
    assert_linked_canonical_tamper_blocks(
        |current, _| current.replace("- Operation ID: `", "- Operation ID: `operation-forged"),
        |linked, _| linked.replace("operation_id: ", "operation_id: operation-forged"),
    );
}

#[test]
fn linked_adapter_must_participate_in_canonical_operation_validation() {
    assert_linked_canonical_tamper_blocks(
        |current, _| current.replace("- Adapter: `codex`", "- Adapter: `claude`"),
        |linked, _| linked.replace("adapter: codex", "adapter: claude"),
    );
}

#[test]
fn linked_session_id_must_participate_in_canonical_operation_validation() {
    assert_linked_canonical_tamper_blocks(
        |current, _| current.replace("active-session", "forged-session"),
        |linked, _| linked.replace("session_id: active-session", "session_id: forged-session"),
    );
}

#[test]
fn linked_request_id_must_participate_in_canonical_operation_validation() {
    assert_linked_canonical_tamper_blocks(
        |current, _| current.replace("active-request", "forged-request"),
        |linked, _| linked.replace("request_id: active-request", "request_id: forged-request"),
    );
}

#[test]
fn partial_linked_identity_fails_closed() {
    assert_linked_canonical_tamper_blocks(
        |current, _| {
            current
                .lines()
                .filter(|line| !line.starts_with("- Request ID: "))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        },
        |linked, _| {
            linked
                .lines()
                .filter(|line| !line.starts_with("request_id: "))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        },
    );
}

#[test]
fn linked_plan_cannot_be_rebound_to_low_risk_operation_b() {
    let (_temp, repo, context, active, active_path) = identified_plan_fixture(
        "backend login security",
        SupportedAdapter::Codex,
        "active-session",
        "active-request",
    );
    let current_path = repo.join("docs/baron/plans/CURRENT.md");
    let current_a = fs::read_to_string(&current_path).unwrap();
    let linked_a = fs::read_to_string(&active_path).unwrap();
    let unrelated = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "fix README typo",
        SupportedAdapter::Claude,
        Some("other-session"),
        Some("other-request"),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &context, "fix README typo", &unrelated).unwrap();
    let unrelated_operation = OperationContext::from_identity(&unrelated);
    let proof = record_proof_for_operation(
        &repo,
        &context,
        &unrelated_operation,
        "README verification passed",
    )
    .unwrap();
    let trace_binding =
        TraceOperationBinding::from_operation(&unrelated_operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "README typo corrected",
        TraceOutcome::Completed,
        &trace_binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );

    let current_tampered = current_a
        .replace("- Risk: `high`", "- Risk: `low`")
        .replace(
            &format!("- Task ID: `{}`", active.task_id()),
            &format!("- Task ID: `{}`", unrelated.task_id()),
        )
        .replace(
            &format!("- Operation ID: `{}`", active.operation_id()),
            &format!("- Operation ID: `{}`", unrelated.operation_id()),
        )
        .replace("- Adapter: `codex`", "- Adapter: `claude`")
        .replace(
            &format!("- Session ID: `{}`", active.session_id()),
            &format!("- Session ID: `{}`", unrelated.session_id()),
        )
        .replace(
            &format!("- Request ID: `{}`", active.request_id()),
            &format!("- Request ID: `{}`", unrelated.request_id()),
        );
    let linked_tampered = linked_a
        .replace("risk: high", "risk: low")
        .replace(
            &format!("task_id: {}", active.task_id()),
            &format!("task_id: {}", unrelated.task_id()),
        )
        .replace(
            &format!("operation_id: {}", active.operation_id()),
            &format!("operation_id: {}", unrelated.operation_id()),
        )
        .replace("adapter: codex", "adapter: claude")
        .replace(
            &format!("session_id: {}", active.session_id()),
            &format!("session_id: {}", unrelated.session_id()),
        )
        .replace(
            &format!("request_id: {}", active.request_id()),
            &format!("request_id: {}", unrelated.request_id()),
        );
    fs::write(&current_path, current_tampered).unwrap();
    fs::write(&active_path, linked_tampered).unwrap();

    assert!(!reconcile(&repo).unwrap().passed);
    assert_stop_blocks(&repo, &context, "fix README typo", &unrelated);
    assert!(complete_plan(&repo, &context, "verification attempted").is_err());
    assert!(fs::read_to_string(active_path)
        .unwrap()
        .contains("status: in_progress"));
}

#[test]
fn legacy_linked_plan_task_id_must_match_the_canonical_task() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let plan = start_or_resume_plan(&repo, &context, "fix README typo").unwrap();
    let current_path = repo.join("docs/baron/plans/CURRENT.md");
    let current = fs::read_to_string(&current_path).unwrap();
    let linked = fs::read_to_string(&plan.repo_path).unwrap();
    fs::write(
        &current_path,
        current.replace(
            &format!(
                "- Task ID: `{}`",
                task_id_for_task(&context.project_id, "fix README typo").unwrap()
            ),
            "- Task ID: `task-forged-legacy`",
        ),
    )
    .unwrap();
    fs::write(
        &plan.repo_path,
        linked.replace(
            &format!(
                "task_id: {}",
                task_id_for_task(&context.project_id, "fix README typo").unwrap()
            ),
            "task_id: task-forged-legacy",
        ),
    )
    .unwrap();

    let error = active_plan_authority(&repo).unwrap_err();
    assert!(error
        .to_string()
        .contains("linked plan task_id does not match canonical task"));
    assert!(!reconcile(&repo).unwrap().passed);
    assert!(plan_status(&repo)
        .unwrap()
        .contains("Completion integrity: `failed`"));
}

#[test]
fn historical_legacy_task_id_is_preserved_during_lifecycle_projection() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    let title = "fix README typo";
    let legacy_task_id = "task-fix-readme-typo";
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let plan = start_or_resume_plan(&repo, &context, title).unwrap();
    let canonical_task_id = task_id_for_task(&context.project_id, title).unwrap();
    let current_path = repo.join("docs/baron/plans/CURRENT.md");
    let current = fs::read_to_string(&current_path).unwrap();
    let linked = fs::read_to_string(&plan.repo_path).unwrap();
    fs::write(
        &current_path,
        current.replace(
            &format!("- Task ID: `{canonical_task_id}`"),
            &format!("- Task ID: `{legacy_task_id}`"),
        ),
    )
    .unwrap();
    fs::write(
        &plan.repo_path,
        linked.replace(
            &format!("task_id: {canonical_task_id}"),
            &format!("task_id: {legacy_task_id}"),
        ),
    )
    .unwrap();

    start_or_resume_plan(&repo, &context, title).unwrap();
    update_plan(&repo, &context, "legacy plan remains readable").unwrap();
    interrupt_plan(&repo, &context, "legacy lifecycle projection checked").unwrap();

    assert!(fs::read_to_string(&current_path)
        .unwrap()
        .contains(&format!("- Task ID: `{legacy_task_id}`")));
    assert!(fs::read_to_string(&plan.repo_path)
        .unwrap()
        .contains(&format!("task_id: {legacy_task_id}")));
}

#[test]
fn current_risk_mismatch_is_presentation_only() {
    assert_current_metadata_mismatch_is_presentation_only(|current, _| {
        current.replace("- Risk: `high`", "- Risk: `low`")
    });
}

#[test]
fn current_task_id_mismatch_is_presentation_only() {
    assert_current_metadata_mismatch_is_presentation_only(|current, identity| {
        current.replace(
            &format!("- Task ID: `{}`", identity.task_id()),
            "- Task ID: `tampered-task`",
        )
    });
}

#[test]
fn current_operation_id_mismatch_is_presentation_only() {
    assert_current_metadata_mismatch_is_presentation_only(|current, identity| {
        current.replace(
            &format!("- Operation ID: `{}`", identity.operation_id()),
            "- Operation ID: `tampered-operation`",
        )
    });
}

#[test]
fn current_adapter_mismatch_is_presentation_only() {
    assert_current_metadata_mismatch_is_presentation_only(|current, _| {
        current.replace("- Adapter: `codex`", "- Adapter: `claude`")
    });
}

#[test]
fn current_session_id_mismatch_is_presentation_only() {
    assert_current_metadata_mismatch_is_presentation_only(|current, identity| {
        current.replace(
            &format!("- Session ID: `{}`", identity.session_id()),
            "- Session ID: `tampered-session`",
        )
    });
}

#[test]
fn current_request_id_mismatch_is_presentation_only() {
    assert_current_metadata_mismatch_is_presentation_only(|current, identity| {
        current.replace(
            &format!("- Request ID: `{}`", identity.request_id()),
            "- Request ID: `tampered-request`",
        )
    });
}

#[test]
fn current_title_mismatch_is_presentation_only() {
    assert_current_metadata_mismatch_is_presentation_only(|current, _| {
        current.replace("- Title: backend login security", "- Title: unrelated task")
    });
}

#[test]
fn current_risk_and_operation_swap_cannot_authorize_low_risk_evidence() {
    let (_temp, repo, context, active, plan_path) = identified_plan_fixture(
        "backend login security",
        SupportedAdapter::Codex,
        "active-session",
        "active-request",
    );
    let unrelated = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "fix README typo",
        SupportedAdapter::Claude,
        Some("other-session"),
        Some("other-request"),
    )
    .unwrap();
    let current_path = repo.join("docs/baron/plans/CURRENT.md");
    let current_a = fs::read_to_string(&current_path).unwrap();
    start_or_resume_plan_for_identity(&repo, &context, "fix README typo", &unrelated).unwrap();

    let unrelated_operation = OperationContext::from_identity(&unrelated);
    let proof = record_proof_for_operation(
        &repo,
        &context,
        &unrelated_operation,
        "README verification passed",
    )
    .unwrap();
    let trace_binding =
        TraceOperationBinding::from_operation(&unrelated_operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "README typo corrected",
        TraceOutcome::Completed,
        &trace_binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );

    let tampered = current_a
        .replace("- Risk: `high`", "- Risk: `low`")
        .replace(
            &format!("- Task ID: `{}`", active.task_id()),
            &format!("- Task ID: `{}`", unrelated.task_id()),
        )
        .replace(
            &format!("- Operation ID: `{}`", active.operation_id()),
            &format!("- Operation ID: `{}`", unrelated.operation_id()),
        )
        .replace("- Adapter: `codex`", "- Adapter: `claude`")
        .replace(
            &format!("- Session ID: `{}`", active.session_id()),
            &format!("- Session ID: `{}`", unrelated.session_id()),
        )
        .replace(
            &format!("- Request ID: `{}`", active.request_id()),
            &format!("- Request ID: `{}`", unrelated.request_id()),
        );
    fs::write(&current_path, tampered).unwrap();

    let reconciliation = reconcile(&repo).unwrap();
    assert!(!reconciliation.passed);
    assert!(reconciliation.gaps.iter().any(|gap| {
        gap.contains("risk") || gap.contains("binding") || gap.contains("ambiguous")
    }));
    // The forged display cannot downgrade A; valid B evidence is usable only
    // through B's exact identity and never completes A.
    assert_stop_blocks(&repo, &context, "backend login security", &active);
    assert!(
        reconcile_for_operation(&repo, &context, &unrelated)
            .unwrap()
            .passed
    );
    assert!(complete_plan(&repo, &context, "verification attempted").is_err());
    assert!(fs::read_to_string(plan_path)
        .unwrap()
        .contains("status: in_progress"));
}

#[test]
fn current_pointer_cannot_combine_metadata_with_another_valid_plan() {
    let (_temp, repo, context, active, active_path) = identified_plan_fixture(
        "fix README typo",
        SupportedAdapter::Codex,
        "active-session",
        "active-request",
    );
    let active_operation = OperationContext::from_identity(&active);
    let proof = record_proof_for_operation(
        &repo,
        &context,
        &active_operation,
        "README verification passed",
    )
    .unwrap();
    let trace_binding =
        TraceOperationBinding::from_operation(&active_operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "README typo corrected",
        TraceOutcome::Completed,
        &trace_binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );

    let current_path = repo.join("docs/baron/plans/CURRENT.md");
    let current_a = fs::read_to_string(&current_path).unwrap();
    let other = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "copy README typo",
        SupportedAdapter::Claude,
        Some("other-session"),
        Some("other-request"),
    )
    .unwrap();
    let other_plan =
        start_or_resume_plan_for_identity(&repo, &context, "copy README typo", &other).unwrap();
    let other_relative = other_plan
        .repo_path
        .strip_prefix(&repo)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    let current_plan_line = current_a
        .lines()
        .find(|line| line.starts_with("- Plan: `"))
        .unwrap();
    let switched = current_a.replace(current_plan_line, &format!("- Plan: `{other_relative}`"));
    fs::write(&current_path, switched).unwrap();

    assert!(!reconcile(&repo).unwrap().passed);
    // Exact identified authority no longer treats a presentation mismatch as
    // its plan selector. It must still validate A via ACTIVE/frontmatter.
    assert!(
        reconcile_for_operation(&repo, &context, &active)
            .unwrap()
            .passed
    );
    assert!(complete_plan(&repo, &context, "verification attempted").is_err());
    assert!(fs::read_to_string(other_plan.repo_path)
        .unwrap()
        .contains("status: in_progress"));
    assert!(fs::read_to_string(active_path)
        .unwrap()
        .contains("status: in_progress"));
}

#[test]
fn legacy_reconciliation_cannot_hide_multiple_active_operations_without_current() {
    let (_temp, repo, context, _identity, _plan_path) = identified_plan_fixture(
        "fix README alpha typo",
        SupportedAdapter::Codex,
        "session-a",
        "request-a",
    );
    let b = LifecycleIdentity::resolve(
        &context.project_id,
        "backend login beta",
        SupportedAdapter::Claude,
        Some("session-b"),
        Some("request-b"),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &context, "backend login beta", &b).unwrap();
    fs::remove_file(repo.join("docs/baron/plans/CURRENT.md")).unwrap();
    let before = snapshot_tree(&repo.join("docs/baron"));
    let report = reconcile(&repo).unwrap();
    assert!(!report.passed);
    assert!(report.gaps.iter().any(|issue| issue.contains("ambiguous")));
    assert_eq!(snapshot_tree(&repo.join("docs/baron")), before);
}

#[test]
fn identified_and_legacy_plan_metadata_states_cannot_cross_authorize() {
    let (_temp, repo, context, plan_path) = {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("demo");
        let vault = temp.path().join("Vault");
        fs::create_dir_all(&repo).unwrap();
        let context = ensure_vault(&vault, &repo).unwrap();
        let plan = start_or_resume_plan(&repo, &context, "fix README typo").unwrap();
        let identity = AuthoritativeLifecycleIdentity::resolve(
            &context.project_id,
            "fix README typo",
            SupportedAdapter::Claude,
            Some("identified-session"),
            Some("identified-request"),
        )
        .unwrap();
        let operation = OperationContext::from_identity(&identity);
        let proof =
            record_proof_for_operation(&repo, &context, &operation, "README verification passed")
                .unwrap();
        let binding = TraceOperationBinding::from_operation(&operation, &proof.id).unwrap();
        let error = record_trace_for_operation(
            &repo,
            &context,
            "README typo corrected",
            TraceOutcome::Completed,
            &binding,
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("validated active plan for this operation"));
        let current_path = repo.join("docs/baron/plans/CURRENT.md");
        let current = fs::read_to_string(&current_path).unwrap();
        let identified = current.replace(
            "- Verification: not_run",
            &format!(
                "- Operation ID: `{}`\n- Adapter: `claude`\n- Session ID: `identified-session`\n- Request ID: `identified-request`\n- Verification: not_run",
                identity.operation_id()
            ),
        );
        fs::write(&current_path, identified).unwrap();
        assert!(complete_plan(&repo, &context, "verification attempted").is_err());
        let reconciliation = reconcile(&repo).unwrap();
        assert!(!reconciliation.passed);
        assert!(reconciliation
            .gaps
            .iter()
            .any(|gap| gap.contains("binding state")));
        assert_stop_blocks(&repo, &context, "fix README typo", &identity);
        assert!(fs::read_to_string(&plan.repo_path)
            .unwrap()
            .contains("status: in_progress"));
        (temp, repo, context, plan.repo_path)
    };
    let _ = (_temp, repo, context, plan_path);

    let (_temp, repo, context, _identity, plan_path) = identified_plan_fixture(
        "fix README typo",
        SupportedAdapter::Codex,
        "identified-session",
        "identified-request",
    );
    let current_path = repo.join("docs/baron/plans/CURRENT.md");
    let current = fs::read_to_string(&current_path).unwrap();
    let legacy = current
        .lines()
        .filter(|line| {
            !line.starts_with("- Operation ID: ")
                && !line.starts_with("- Adapter: ")
                && !line.starts_with("- Session ID: ")
                && !line.starts_with("- Request ID: ")
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&current_path, format!("{legacy}\n")).unwrap();
    assert!(complete_plan(&repo, &context, "verification attempted").is_err());
    let reconciliation = reconcile(&repo).unwrap();
    assert!(!reconciliation.passed);
    assert!(reconciliation
        .gaps
        .iter()
        .all(|gap| !gap.contains("binding state")));
    assert_stop_blocks(&repo, &context, "fix README typo", &_identity);
    assert!(fs::read_to_string(plan_path)
        .unwrap()
        .contains("status: in_progress"));
}

#[test]
fn update_and_interrupt_preserve_last_known_state() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    start_or_resume_plan(&repo, &context, "frontend HomePage").unwrap();

    update_plan(&repo, &context, "Implemented hero; responsive remains").unwrap();
    interrupt_plan(
        &repo,
        &context,
        "Stopped before mobile verification; next run responsive tests",
    )
    .unwrap();

    let status = plan_status(&repo).unwrap();
    assert!(status.contains("Status: `interrupted`"));
    assert!(status.contains("mobile verification"));
    let plan = fs::read_to_string(
        fs::read_dir(repo.join("docs/baron/plans"))
            .unwrap()
            .filter_map(Result::ok)
            .find(|entry| entry.path().is_dir())
            .unwrap()
            .path()
            .read_dir()
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path(),
    )
    .unwrap();
    assert!(plan.contains("Implemented hero"));
    let repo_index = fs::read_to_string(repo.join("docs/baron/plans/INDEX.md")).unwrap();
    let vault_index = fs::read_to_string(context.project_root.join("Plans/INDEX.md")).unwrap();
    for index in [repo_index, vault_index] {
        assert!(index.contains("frontend HomePage"));
        assert!(index.contains("status: `interrupted`"));
        assert!(!index.contains("frontend HomePage") || !index.contains("status: `in_progress`"));
    }
}

#[test]
fn completion_is_blocked_without_proof_and_passing_trace() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    start_or_resume_plan(&repo, &context, "backend login security").unwrap();

    let error = complete_plan(&repo, &context, "all done").unwrap_err();

    assert!(error.to_string().contains("proof"));
    assert!(plan_status(&repo).unwrap().contains("in_progress"));
}

#[test]
fn hand_edited_completed_state_is_reported_as_failed_integrity() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let plan = start_or_resume_plan(&repo, &context, "backend login security").unwrap();
    let current_path = repo.join("docs/baron/plans/CURRENT.md");
    let current = fs::read_to_string(&current_path)
        .unwrap()
        .replace("- Status: `in_progress`", "- Status: `completed`")
        .replace(
            "- Verification: not_run",
            "- Verification: claimed manually",
        );
    fs::write(&current_path, current).unwrap();
    let body = fs::read_to_string(&plan.repo_path)
        .unwrap()
        .replace("status: in_progress", "status: completed")
        .replace("verification: not_run", "verification: claimed manually");
    fs::write(&plan.repo_path, body).unwrap();

    let status = plan_status(&repo).unwrap();

    assert!(status.contains("Completion integrity: `failed`"));
    assert!(status.contains("proof is missing"));
    assert!(status.contains("passing trace is missing"));
}

#[test]
fn high_risk_plan_completes_after_valid_proof_and_detailed_trace() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join("src")).unwrap();
    setup_git(&repo);
    fs::write(repo.join("README.md"), "# Demo\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(&repo)
        .output()
        .unwrap();
    fs::write(repo.join("src/auth.rs"), "pub fn login() {}\n").unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "backend login security",
        SupportedAdapter::Codex,
        Some("session-proof"),
        Some("request-proof"),
    )
    .unwrap();
    let operation = OperationContext::from_identity(&identity);
    let plan =
        start_or_resume_plan_for_operation(&repo, &context, "backend login security", &operation)
            .unwrap();
    confirm_intent(&repo, &context, "backend login security");
    start_or_resume_intake(&repo, &context, "backend login security").unwrap();
    assert_eq!(
        current_harness_title_for_operation(&repo, &operation)
            .unwrap()
            .as_deref(),
        Some("backend login security")
    );
    let proof_binding = ReceiptContext::new(
        identity.task_id(),
        identity.operation_id(),
        identity.adapter().as_str(),
        identity.session_id(),
        identity.request_id(),
        "proof",
    );
    let receipt = passing_execution(&repo, &identity, "proof");
    let proof =
        record_proof_from_receipt_bound(&repo, &context, &receipt.receipt_id, &proof_binding)
            .unwrap();
    for agent in ["code-reviewer", "security-auditor", "test-engineer"] {
        let (gate_receipt, binding) = passing_gate_execution(&repo, agent, &identity);
        record_gate_evidence_with_receipt_bound(
            &repo,
            &context,
            agent,
            &format!("{agent} reviewed auth security with evidence"),
            &gate_receipt.receipt_id,
            &binding,
        )
        .unwrap();
    }
    let trace_binding = TraceOperationBinding::from_operation(&operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "Implemented backend login security",
        TraceOutcome::Completed,
        &trace_binding,
    )
    .unwrap();
    let trace_content = fs::read_to_string(&trace.repo_path).unwrap();
    assert!(
        trace_content.contains("- Current story: `backend login security`"),
        "{trace_content}"
    );
    let score = score_trace(&repo, &context, Some(&trace.id)).unwrap();
    assert!(score.passed, "score={score:?}\ntrace={trace_content}");

    complete_plan(
        &repo,
        &context,
        "cargo test auth passed with authorization review",
    )
    .unwrap();

    let status = plan_status(&repo).unwrap();
    assert!(status.contains("Status: `completed`"));
    assert!(status.contains("authorization review"));
    let plan_content = fs::read_to_string(&plan.repo_path).unwrap();
    assert!(plan_content.contains("verification: cargo test auth passed with authorization review"));
    assert!(!plan_content.contains("verification: not_run"));
    let repo_index = fs::read_to_string(repo.join("docs/baron/plans/INDEX.md")).unwrap();
    let vault_index = fs::read_to_string(context.project_root.join("Plans/INDEX.md")).unwrap();
    for index in [repo_index, vault_index] {
        assert!(index.contains("backend login security"));
        assert!(index.contains("status: `completed`"));
    }

    let current_path = repo.join("docs/baron/plans/CURRENT.md");
    let tampered_current = fs::read_to_string(&current_path)
        .unwrap()
        .replace("- Risk: `high`", "- Risk: `low`");
    fs::write(&current_path, tampered_current).unwrap();
    let tampered_linked = fs::read_to_string(&plan.repo_path)
        .unwrap()
        .replace("risk: high", "risk: low");
    fs::write(&plan.repo_path, tampered_linked).unwrap();
    let tampered_status = plan_status(&repo).unwrap();
    assert!(tampered_status.contains("Completion integrity: `failed`"));
    assert!(tampered_status.contains("linked plan risk does not match canonical classifier"));
}

#[test]
fn completion_ignores_newer_unrelated_proof_and_trace_artifacts() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let first = LifecycleIdentity::resolve(
        &context.project_id,
        "fix README typo",
        SupportedAdapter::Codex,
        Some("first-session"),
        Some("first-request"),
    )
    .unwrap();
    let first_operation = OperationContext::from_identity(&first);
    start_or_resume_plan_for_operation(&repo, &context, "fix README typo", &first_operation)
        .unwrap();
    let first_proof = record_proof_for_operation(
        &repo,
        &context,
        &first_operation,
        "README verification passed",
    )
    .unwrap();
    let first_trace_binding =
        TraceOperationBinding::from_operation(&first_operation, &first_proof.id).unwrap();
    let first_trace = record_trace_for_operation(
        &repo,
        &context,
        "README typo corrected",
        TraceOutcome::Completed,
        &first_trace_binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&first_trace.id))
            .unwrap()
            .passed
    );

    thread::sleep(Duration::from_millis(5));
    let unrelated = LifecycleIdentity::resolve(
        &context.project_id,
        "unrelated docs task",
        SupportedAdapter::Claude,
        Some("unrelated-session"),
        Some("unrelated-request"),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &context, "unrelated docs task", &unrelated).unwrap();
    let unrelated_operation = OperationContext::from_identity(&unrelated);
    let unrelated_proof = record_proof_for_operation(
        &repo,
        &context,
        &unrelated_operation,
        "Unrelated verification passed",
    )
    .unwrap();
    let unrelated_trace_binding =
        TraceOperationBinding::from_operation(&unrelated_operation, &unrelated_proof.id).unwrap();
    let unrelated_trace = record_trace_for_operation(
        &repo,
        &context,
        "Unrelated docs task completed",
        TraceOutcome::Completed,
        &unrelated_trace_binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&unrelated_trace.id))
            .unwrap()
            .passed
    );

    complete_plan_for_identity(&repo, &context, "README verification passed", &first).unwrap();
    assert!(plan_status(&repo).unwrap().contains("Status: `completed`"));
}

#[test]
fn legacy_unbound_proof_and_trace_are_diagnostic_only_for_identified_completion() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = LifecycleIdentity::resolve(
        &context.project_id,
        "fix README typo",
        SupportedAdapter::Codex,
        Some("identified-session"),
        Some("identified-request"),
    )
    .unwrap();
    record_proof(&repo, &context, "README verification passed").unwrap();
    let trace = record_trace(
        &repo,
        &context,
        "README typo corrected",
        TraceOutcome::Completed,
    )
    .unwrap();
    assert!(trace.binding.is_none());
    let diagnostic_score = score_trace(&repo, &context, Some(&trace.id)).unwrap();
    assert!(!diagnostic_score.passed);
    assert!(diagnostic_score
        .missing_fields
        .contains(&"current plan".to_string()));

    start_or_resume_plan_for_identity(&repo, &context, "fix README typo", &identity).unwrap();

    let error = complete_plan(&repo, &context, "README verification passed").unwrap_err();
    assert!(error.to_string().contains("proof is missing"));
    assert!(plan_status(&repo)
        .unwrap()
        .contains("Status: `in_progress`"));
}

#[test]
fn cross_operation_quality_gates_cannot_complete_the_active_operation() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join("src")).unwrap();
    setup_git(&repo);
    fs::write(repo.join("README.md"), "# Demo\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(&repo)
        .output()
        .unwrap();
    fs::write(repo.join("src/auth.rs"), "pub fn login() {}\n").unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let active = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "backend login security",
        SupportedAdapter::Codex,
        Some("active-session"),
        Some("active-request"),
    )
    .unwrap();
    let active_operation = OperationContext::from_identity(&active);
    start_or_resume_plan_for_operation(
        &repo,
        &context,
        "backend login security",
        &active_operation,
    )
    .unwrap();
    confirm_intent(&repo, &context, "backend login security");
    start_or_resume_intake(&repo, &context, "backend login security").unwrap();
    let proof_binding = ReceiptContext::new(
        active.task_id(),
        active.operation_id(),
        active.adapter().as_str(),
        active.session_id(),
        active.request_id(),
        "proof",
    );
    let proof_receipt = passing_execution(&repo, &active, "proof");
    let proof =
        record_proof_from_receipt_bound(&repo, &context, &proof_receipt.receipt_id, &proof_binding)
            .unwrap();
    let trace_binding =
        TraceOperationBinding::from_operation(&active_operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "Implemented backend login security",
        TraceOutcome::Completed,
        &trace_binding,
    )
    .unwrap();
    assert!(
        !score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );

    let unrelated = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "fix README typo",
        SupportedAdapter::Codex,
        Some("other-session"),
        Some("other-request"),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &context, "fix README typo", &unrelated).unwrap();
    for agent in ["code-reviewer", "security-auditor", "test-engineer"] {
        let (gate_receipt, binding) = passing_gate_execution(&repo, agent, &unrelated);
        record_gate_evidence_with_receipt_bound(
            &repo,
            &context,
            agent,
            &format!("{agent} reviewed the unrelated operation"),
            &gate_receipt.receipt_id,
            &binding,
        )
        .unwrap();
    }

    let unrelated_proof_binding = ReceiptContext::for_identity(&unrelated, "proof").unwrap();
    let unrelated_proof_receipt = passing_execution(&repo, &unrelated, "proof");
    let unrelated_proof = record_proof_from_receipt_bound(
        &repo,
        &context,
        &unrelated_proof_receipt.receipt_id,
        &unrelated_proof_binding,
    )
    .unwrap();
    let unrelated_trace_binding = TraceOperationBinding::from_operation(
        &OperationContext::from_identity(&unrelated),
        &unrelated_proof.id,
    )
    .unwrap();
    let unrelated_trace = record_trace_for_operation(
        &repo,
        &context,
        "Unrelated backend login security verification passed",
        TraceOutcome::Completed,
        &unrelated_trace_binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&unrelated_trace.id))
            .unwrap()
            .passed
    );

    let reconciliation = reconcile_for_operation(&repo, &context, &active).unwrap();
    assert!(!reconciliation.passed);
    assert!(reconciliation
        .gaps
        .iter()
        .any(|gap| gap.contains("quality-gate") || gap.contains("trace")));

    let error = complete_plan_for_identity(
        &repo,
        &context,
        "cargo test auth passed with authorization review",
        &active,
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("trusted quality-gate receipts are missing"));
}
