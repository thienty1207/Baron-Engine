use std::fs;

use baron_core::automation::{record_lifecycle_event, AutomationEvent, HookAdapter};
use baron_core::autopilot::{
    approve_candidate, autopilot_status, reject_candidate, review_after_task,
};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::continuity::record_continuity_checkpoint;
use baron_core::continuity::{record_recovery, RecoveryInput, RecoveryOutcome};
use baron_core::intent::{record_intent, IntentBriefInput};
use baron_core::plan::start_or_resume_plan;
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

#[test]
fn post_task_review_writes_candidate_learning_not_trusted_facts() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    start_or_resume_plan(&repo, &context, "auth login flow").unwrap();
    record_continuity_checkpoint(
        &repo,
        &context,
        "auth login still needs security proof",
        "codex",
    )
    .unwrap();
    record_intent(
        &repo,
        &context,
        IntentBriefInput {
            title: "auth login flow".to_string(),
            current_behavior: "Login flow needs evidence-backed review.".to_string(),
            target_behavior: "Confirmed tenant-safe login behavior.".to_string(),
            scope: "Auth login only.".to_string(),
            non_goals: vec!["No unrelated account work.".to_string()],
            constraints: vec!["Preserve existing sessions.".to_string()],
            decisions: vec!["Use current auth architecture.".to_string()],
            required_proof: "Auth and tenant tests pass.".to_string(),
            unknowns: Vec::new(),
            confirmed: true,
        },
    )
    .unwrap();
    record_recovery(
        &repo,
        &context,
        RecoveryInput {
            outcome: RecoveryOutcome::Interrupted,
            root_cause: "Network connection was lost.".to_string(),
            last_successful_step: "Intent and plan were recorded.".to_string(),
            evidence: vec!["No verification ran after the interruption.".to_string()],
            affected_files: vec!["src/auth.rs".to_string()],
            next_action: "Resume the auth test suite.".to_string(),
            retry_conditions: vec!["Network is stable.".to_string()],
        },
    )
    .unwrap();
    record_lifecycle_event(
        &context,
        HookAdapter::Codex,
        AutomationEvent::ContextCompiled,
    )
    .unwrap();

    let review = review_after_task(
        &repo,
        &context,
        "Learned that auth login needs tenant-safe proof before completion.",
    )
    .unwrap();

    assert!(review.candidate_count >= 1);
    assert!(review.approval_required);
    assert!(review
        .observed_automation
        .contains(&"ContextCompiled".to_string()));
    assert!(review
        .resume_sources
        .contains(&"docs/baron/harness/CURRENT_INTENT.md".to_string()));
    assert!(review
        .resume_sources
        .contains(&"docs/baron/continuity/CURRENT_RECOVERY.md".to_string()));
    let repo_candidates = fs::read_to_string(&review.repo_path).unwrap();
    let vault_candidates = fs::read_to_string(&review.vault_path).unwrap();
    for content in [&repo_candidates, &vault_candidates] {
        assert!(content.contains("Status: `candidate`"));
        assert!(content.contains("Trusted fact: `no`"));
        assert!(content.contains("Approval required: `yes`"));
        assert!(content.contains("auth login"));
    }
    assert!(!context
        .project_root
        .join("Facts.md")
        .read_to_string_lossy()
        .contains("tenant-safe"));
}

#[test]
fn approval_and_rejection_update_candidates_without_overwriting_runtime_assets() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join("assets/core/skills/superpowers")).unwrap();
    fs::write(
        repo.join("assets/core/skills/superpowers/SKILL.md"),
        "core skill",
    )
    .unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let first = review_after_task(
        &repo,
        &context,
        "Skill routing could use clearer proof wording.",
    )
    .unwrap();
    let first_id = first.candidate_ids[0].clone();
    approve_candidate(&repo, &context, &first_id).unwrap();

    let second = review_after_task(
        &repo,
        &context,
        "Ignore this weak uncertain memory candidate.",
    )
    .unwrap();
    let second_id = second.candidate_ids[0].clone();
    reject_candidate(&repo, &context, &second_id).unwrap();

    let content = fs::read_to_string(repo.join("docs/baron/autopilot/CANDIDATES.md")).unwrap();
    assert!(content.contains(&format!("## {first_id}")));
    assert!(content.contains("Status: `approved`"));
    assert!(content.contains(&format!("## {second_id}")));
    assert!(content.contains("Status: `rejected`"));
    assert_eq!(
        fs::read_to_string(repo.join("assets/core/skills/superpowers/SKILL.md")).unwrap(),
        "core skill"
    );
}

#[test]
fn status_surfaces_resume_and_observed_automation_without_guessing() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Claude, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    start_or_resume_plan(&repo, &context, "interrupted checkout flow").unwrap();
    record_lifecycle_event(&context, HookAdapter::Claude, AutomationEvent::PlanStarted).unwrap();
    record_continuity_checkpoint(&repo, &context, "network loss before tests", "claude").unwrap();

    let status = autopilot_status(&repo, &context).unwrap();

    assert!(status.contains("# Baron Autopilot Status"));
    assert!(status.contains("PlanStarted"));
    assert!(status.contains("network loss before tests"));
    assert!(status.contains("Do not infer completion"));
}

trait ReadToStringLossy {
    fn read_to_string_lossy(&self) -> String;
}

impl ReadToStringLossy for std::path::PathBuf {
    fn read_to_string_lossy(&self) -> String {
        fs::read_to_string(self).unwrap_or_default()
    }
}
