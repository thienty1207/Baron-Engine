use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use baron_core::automation::{handle_hook, reconcile, AutomationEvent, HookAdapter};
use baron_core::control_plane::record_gate_evidence_with_receipt_bound;
use baron_core::execution_receipt::{
    execute_command_for_identity, ExecutionRequest, ReceiptContext,
};
use baron_core::harness::start_or_resume_intake;
use baron_core::intent::{record_intent, IntentBriefInput};
use baron_core::operation::{
    AuthoritativeLifecycleIdentity, LifecycleIdentity, OperationContext, SupportedAdapter,
};
use baron_core::plan::{
    complete_plan, interrupt_plan, plan_status, start_or_resume_plan,
    start_or_resume_plan_for_identity, start_or_resume_plan_for_operation, update_plan,
};
use baron_core::proof::{
    record_proof, record_proof_for_operation, record_proof_from_receipt_bound,
};
use baron_core::trace::{
    record_trace, record_trace_for_operation, score_trace, TraceOperationBinding, TraceOutcome,
};
use baron_core::vault::{ensure_vault, VaultContext};
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

fn assert_current_metadata_mismatch_blocks<F>(mutate: F)
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

    let error = complete_plan(&repo, &context, "verification attempted").unwrap_err();
    assert!(error.to_string().contains("authority mismatch"));
    let reconciliation = reconcile(&repo).unwrap();
    assert!(!reconciliation.passed);
    assert!(reconciliation
        .gaps
        .iter()
        .any(|gap| gap.contains("CURRENT")));
    assert_stop_blocks(&repo, &context, "backend login security", &identity);
    assert!(fs::read_to_string(plan_path)
        .unwrap()
        .contains("status: in_progress"));
}

#[test]
fn current_risk_mismatch_fails_closed() {
    assert_current_metadata_mismatch_blocks(|current, _| {
        current.replace("- Risk: `high`", "- Risk: `low`")
    });
}

#[test]
fn current_task_id_mismatch_fails_closed() {
    assert_current_metadata_mismatch_blocks(|current, identity| {
        current.replace(
            &format!("- Task ID: `{}`", identity.task_id()),
            "- Task ID: `tampered-task`",
        )
    });
}

#[test]
fn current_operation_id_mismatch_fails_closed() {
    assert_current_metadata_mismatch_blocks(|current, identity| {
        current.replace(
            &format!("- Operation ID: `{}`", identity.operation_id()),
            "- Operation ID: `tampered-operation`",
        )
    });
}

#[test]
fn current_adapter_mismatch_fails_closed() {
    assert_current_metadata_mismatch_blocks(|current, _| {
        current.replace("- Adapter: `codex`", "- Adapter: `claude`")
    });
}

#[test]
fn current_session_id_mismatch_fails_closed() {
    assert_current_metadata_mismatch_blocks(|current, identity| {
        current.replace(
            &format!("- Session ID: `{}`", identity.session_id()),
            "- Session ID: `tampered-session`",
        )
    });
}

#[test]
fn current_request_id_mismatch_fails_closed() {
    assert_current_metadata_mismatch_blocks(|current, identity| {
        current.replace(
            &format!("- Request ID: `{}`", identity.request_id()),
            "- Request ID: `tampered-request`",
        )
    });
}

#[test]
fn current_title_mismatch_fails_closed() {
    assert_current_metadata_mismatch_blocks(|current, _| {
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
    assert!(reconciliation
        .gaps
        .iter()
        .any(|gap| gap.contains("risk") || gap.contains("binding")));
    assert_stop_blocks(&repo, &context, "fix README typo", &unrelated);
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
    assert_stop_blocks(&repo, &context, "fix README typo", &active);
    assert!(complete_plan(&repo, &context, "verification attempted").is_err());
    assert!(fs::read_to_string(other_plan.repo_path)
        .unwrap()
        .contains("status: in_progress"));
    assert!(fs::read_to_string(active_path)
        .unwrap()
        .contains("status: in_progress"));
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
        let trace = record_trace_for_operation(
            &repo,
            &context,
            "README typo corrected",
            TraceOutcome::Completed,
            &binding,
        )
        .unwrap();
        assert!(
            score_trace(&repo, &context, Some(&trace.id))
                .unwrap()
                .passed
        );
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
        .any(|gap| gap.contains("binding state")));
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
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );

    complete_plan(
        &repo,
        &context,
        "cargo test auth passed with authorization review",
    )
    .unwrap();

    let status = plan_status(&repo).unwrap();
    assert!(status.contains("Status: `completed`"));
    assert!(status.contains("authorization review"));
    let plan_content = fs::read_to_string(plan.repo_path).unwrap();
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
    let tampered_status = plan_status(&repo).unwrap();
    assert!(tampered_status.contains("Completion integrity: `failed`"));
    assert!(tampered_status.contains("CURRENT risk does not match linked plan risk"));
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
        "unrelated documentation task",
        SupportedAdapter::Claude,
        Some("unrelated-session"),
        Some("unrelated-request"),
    )
    .unwrap();
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
        "Unrelated documentation task completed",
        TraceOutcome::Completed,
        &unrelated_trace_binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&unrelated_trace.id))
            .unwrap()
            .passed
    );

    complete_plan(&repo, &context, "README verification passed").unwrap();
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
    start_or_resume_plan_for_identity(&repo, &context, "fix README typo", &identity).unwrap();
    record_proof(&repo, &context, "README verification passed").unwrap();
    let trace = record_trace(
        &repo,
        &context,
        "README typo corrected",
        TraceOutcome::Completed,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );

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
        "backend login security",
        SupportedAdapter::Codex,
        Some("other-session"),
        Some("other-request"),
    )
    .unwrap();
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

    let reconciliation = reconcile(&repo).unwrap();
    assert!(!reconciliation.passed);
    assert!(reconciliation
        .gaps
        .iter()
        .any(|gap| gap.contains("quality-gate") || gap.contains("trace")));

    let error = complete_plan(
        &repo,
        &context,
        "cargo test auth passed with authorization review",
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("trusted quality-gate receipts are missing"));
}
