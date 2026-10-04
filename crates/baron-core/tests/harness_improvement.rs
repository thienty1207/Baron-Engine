use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use baron_core::automation::{record_lifecycle_event, AutomationEvent, HookAdapter};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::harness::record_friction;
use baron_core::harness_improvement::{
    audit_harness, propose_improvements, record_improvement_outcome, record_intervention,
    verify_open_stories,
};
use baron_core::plan::start_or_resume_plan;
use baron_core::safe_io::acquire_project_lock;
use baron_core::vault::{ensure_vault, VaultContext};
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn audit_scores_context_reads_and_reports_harness_gaps() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    start_or_resume_plan(&repo, &context, "auth login").unwrap();
    record_friction(&repo, &context, "proof command was unclear").unwrap();

    let audit = audit_harness(&repo, &context).unwrap();

    assert!(audit.context_read_score < 100);
    assert!(audit
        .diagnostics
        .iter()
        .any(|item| item.contains("context was not observed")));
    assert!(audit
        .diagnostics
        .iter()
        .any(|item| item.contains("proof is missing")));
    assert_eq!(audit.open_friction_count, 1);

    record_lifecycle_event(
        &context,
        HookAdapter::Codex,
        AutomationEvent::ContextCompiled,
    )
    .unwrap();
    let improved = audit_harness(&repo, &context).unwrap();
    assert!(improved.context_read_score > audit.context_read_score);
}

#[test]
fn intervention_records_are_mirrored_to_vault() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let record =
        record_intervention(&repo, &context, "reviewer corrected missing security proof").unwrap();

    assert!(record.repo_path.exists());
    assert!(record.vault_path.exists());
    assert!(fs::read_to_string(record.repo_path)
        .unwrap()
        .contains("missing security proof"));
    assert!(fs::read_to_string(record.vault_path)
        .unwrap()
        .contains("missing security proof"));
}

#[test]
fn drift_audit_reports_contradictory_status_files_without_rewriting_them() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    write(
        &repo.join("docs/BARON_STATUS.md"),
        "Phase 11 - completed\nPhase 12 - planned\n",
    );
    write(
        &repo.join("docs/BARON_STATUS.json"),
        r#"{"currentPhaseStatus":"in_progress"}"#,
    );

    let audit = audit_harness(&repo, &context).unwrap();

    assert!(audit
        .diagnostics
        .iter()
        .any(|item| item.contains("documentation drift")));
    assert!(fs::read_to_string(repo.join("docs/BARON_STATUS.md"))
        .unwrap()
        .contains("Phase 11 - completed"));
}

#[test]
fn verify_open_stories_reports_pending_and_insufficient_proof_gaps() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    fs::create_dir_all(repo.join("docs/baron/harness")).unwrap();
    write(
        &repo.join("docs/baron/harness/TEST_MATRIX.md"),
        "# Baron Validation Matrix\n\n\
| Story | Risk | Status | Evidence |\n\
| --- | --- | --- | --- |\n\
| auth login | high | pending | pending |\n\
| docs copy | low | verified | cargo test passed |\n\
| billing webhook | high | insufficient | missing replay smoke |\n",
    );

    let report = verify_open_stories(&repo, 10).unwrap();

    assert_eq!(report.checked_count, 3);
    assert_eq!(report.proof_gaps.len(), 2);
    assert!(report
        .proof_gaps
        .iter()
        .any(|gap| gap.contains("auth login")));
    assert!(report
        .proof_gaps
        .iter()
        .any(|gap| gap.contains("billing webhook")));
}

#[test]
fn repeated_friction_creates_human_approval_proposal_and_tracks_outcome() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    record_friction(&repo, &context, "proof command was unclear").unwrap();
    record_friction(&repo, &context, "proof evidence was unclear").unwrap();
    record_friction(&repo, &context, "trace proof was unclear").unwrap();

    let proposal = propose_improvements(&repo, &context).unwrap();

    assert!(proposal.proposal_count >= 1);
    let content = fs::read_to_string(&proposal.repo_path).unwrap();
    assert!(content.contains("human approval required"));
    assert!(content.contains("proof"));
    assert!(!fs::read_to_string(repo.join("AGENTS.md"))
        .unwrap_or_default()
        .contains("proof command was unclear"));

    record_improvement_outcome(
        &repo,
        &context,
        &proposal.proposal_ids[0],
        "After adding clearer proof guidance, repeated proof friction dropped.",
    )
    .unwrap();
    let updated = fs::read_to_string(&proposal.repo_path).unwrap();
    assert!(updated.contains("Actual outcome"));
    assert!(updated.contains("friction dropped"));
    let vault_updated = fs::read_to_string(&proposal.vault_path).unwrap();
    assert!(vault_updated.contains("friction dropped"));
}

fn shared_checkouts(root: &Path) -> (PathBuf, PathBuf, VaultContext, VaultContext) {
    let repo_a = root.join("checkout-a/demo");
    let repo_b = root.join("checkout-b/demo");
    let vault = root.join("Vault");
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

fn proposal_section<'a>(content: &'a str, id: &str) -> &'a str {
    content
        .split_once(&format!("## {id}\n"))
        .unwrap_or_else(|| panic!("missing proposal {id}: {content}"))
        .1
        .split("\n## ")
        .next()
        .unwrap()
}

#[test]
fn stale_checkouts_merge_proposals_and_preserve_local_markdown() {
    let temp = tempdir().unwrap();
    let (repo_a, repo_b, context_a, context_b) = shared_checkouts(temp.path());
    write(
        &repo_a.join("docs/baron/harness/IMPROVEMENTS.md"),
        "# Baron Harness Improvement Proposals\n\nUser notes from A.\n\n## local-a\n\nKeep local record A.\n",
    );
    write(
        &repo_b.join("docs/baron/harness/IMPROVEMENTS.md"),
        "# Baron Harness Improvement Proposals\n\nUser notes from B.\n\n## local-b\n\nKeep local record B.\n",
    );
    for note in ["proof command unclear", "verification guidance unclear"] {
        record_friction(&repo_a, &context_a, note).unwrap();
    }
    for note in ["context startup unclear", "routing startup unclear"] {
        record_friction(&repo_b, &context_b, note).unwrap();
    }

    let proposal_a = propose_improvements(&repo_a, &context_a).unwrap();
    assert_eq!(proposal_a.proposal_ids, ["proposal-proof-guidance"]);
    let proposal_b = propose_improvements(&repo_b, &context_b).unwrap();
    assert_eq!(proposal_b.proposal_ids, ["proposal-context-routing"]);
    // A still has its pre-B projection. Refreshing it must not erase B.
    propose_improvements(&repo_a, &context_a).unwrap();

    let shared = fs::read_to_string(&proposal_a.vault_path).unwrap();
    for heading in [
        "## proposal-proof-guidance\n",
        "## proposal-context-routing\n",
        "## local-a\n",
        "## local-b\n",
    ] {
        assert_eq!(
            shared.matches(heading).count(),
            1,
            "lost/duplicated {heading}"
        );
    }
    for note in [
        "User notes from A.",
        "User notes from B.",
        "Keep local record A.",
        "Keep local record B.",
    ] {
        assert!(shared.contains(note), "lost local text: {note}");
    }
    assert_eq!(fs::read_to_string(proposal_a.repo_path).unwrap(), shared);
}

#[test]
fn stale_checkouts_retain_each_proposals_outcomes_in_its_own_section() {
    let temp = tempdir().unwrap();
    let (repo_a, repo_b, context_a, context_b) = shared_checkouts(temp.path());
    for note in ["proof command unclear", "verification guidance unclear"] {
        record_friction(&repo_a, &context_a, note).unwrap();
    }
    for note in ["context startup unclear", "routing startup unclear"] {
        record_friction(&repo_b, &context_b, note).unwrap();
    }
    let proposal_a = propose_improvements(&repo_a, &context_a).unwrap();
    propose_improvements(&repo_b, &context_b).unwrap();

    record_improvement_outcome(
        &repo_a,
        &context_a,
        "proposal-proof-guidance",
        "A proof guidance improved",
    )
    .unwrap();
    record_improvement_outcome(
        &repo_b,
        &context_b,
        "proposal-context-routing",
        "B context routing improved",
    )
    .unwrap();
    // B's outcome is newer than A's projection, and the target is not last.
    record_improvement_outcome(
        &repo_a,
        &context_a,
        "proposal-proof-guidance",
        "A follow-up confirmed",
    )
    .unwrap();

    let shared = fs::read_to_string(&proposal_a.vault_path).unwrap();
    let section_a = proposal_section(&shared, "proposal-proof-guidance");
    let section_b = proposal_section(&shared, "proposal-context-routing");
    assert!(section_a.contains("A proof guidance improved"));
    assert!(section_a.contains("A follow-up confirmed"));
    assert!(!section_a.contains("B context routing improved"));
    assert!(section_b.contains("B context routing improved"));
    assert!(!section_b.contains("A proof guidance improved"));
    assert!(!section_b.contains("A follow-up confirmed"));
    assert_eq!(fs::read_to_string(proposal_a.repo_path).unwrap(), shared);
}

#[test]
fn same_proposal_merges_local_edits_and_outcomes_without_replaying_them() {
    let temp = tempdir().unwrap();
    let (repo_a, repo_b, context_a, context_b) = shared_checkouts(temp.path());
    for repo in [&repo_a, &repo_b] {
        write(
            &repo.join("docs/baron/harness/FRICTION.md"),
            "- [ ] proof unclear\n- [ ] verification unclear\n",
        );
    }
    let proposal = propose_improvements(&repo_a, &context_a).unwrap();
    propose_improvements(&repo_b, &context_b).unwrap();
    let local_b = repo_b.join("docs/baron/harness/IMPROVEMENTS.md");
    let edited = fs::read_to_string(&local_b).unwrap().replace(
        "Clarify proof requirements based on 2 repeated friction signals.",
        "User-edited proof guidance from B.\n- Custom reviewer note: B-only note.\n  Preserve its continuation.",
    );
    write(&local_b, &edited);

    record_improvement_outcome(&repo_a, &context_a, "proposal-proof-guidance", "A outcome")
        .unwrap();
    record_improvement_outcome(&repo_b, &context_b, "proposal-proof-guidance", "B outcome")
        .unwrap();
    propose_improvements(&repo_a, &context_a).unwrap();
    propose_improvements(&repo_b, &context_b).unwrap();

    let shared = fs::read_to_string(proposal.vault_path).unwrap();
    assert_eq!(shared.matches("## proposal-proof-guidance\n").count(), 1);
    assert_eq!(shared.matches("A outcome").count(), 1);
    assert_eq!(shared.matches("B outcome").count(), 1);
    assert!(shared.contains("User-edited proof guidance from B."));
    assert!(shared.contains("- Custom reviewer note: B-only note.\n  Preserve its continuation."));
}

#[test]
fn improvement_mutations_fail_without_writes_when_shared_vault_is_locked() {
    let temp = tempdir().unwrap();
    let (repo_a, repo_b, context_a, context_b) = shared_checkouts(temp.path());
    let path_a = repo_a.join("docs/baron/harness/IMPROVEMENTS.md");
    let path_b = repo_b.join("docs/baron/harness/IMPROVEMENTS.md");
    let shared_path = context_a
        .project_root
        .join("ProductHarness/IMPROVEMENTS.md");
    for path in [&path_a, &path_b, &shared_path] {
        write(path, "# User improvements\n\nPreserve these bytes.\n");
    }
    write(
        &repo_a.join("docs/baron/harness/FRICTION.md"),
        "- [ ] proof unclear\n- [ ] verification unclear\n",
    );
    let before = [&path_a, &path_b, &shared_path].map(|path| fs::read(path).unwrap());
    let vault_lock = acquire_project_lock(&context_a.project_root).unwrap();
    let results = thread::scope(|scope| {
        let proposals = scope.spawn(|| propose_improvements(&repo_a, &context_a).map(|_| ()));
        let outcomes = scope.spawn(|| {
            record_improvement_outcome(&repo_b, &context_b, "local-b", "must not be published")
        });
        [proposals.join().unwrap(), outcomes.join().unwrap()]
    });
    drop(vault_lock);

    for (operation, result) in ["proposals", "outcomes"].into_iter().zip(results) {
        let error = result.expect_err(&format!("{operation} bypassed the shared Vault lock"));
        assert!(error.to_string().contains("Timed out waiting"), "{error:#}");
    }
    for (path, bytes) in [&path_a, &path_b, &shared_path].into_iter().zip(before) {
        assert_eq!(fs::read(path).unwrap(), bytes, "changed {}", path.display());
    }
}

#[test]
fn unreadable_improvement_documents_fail_closed_before_any_publication() {
    for invalid_shared in [true, false] {
        for record_outcome in [true, false] {
            let temp = tempdir().unwrap();
            let (repo_a, _, context_a, _) = shared_checkouts(temp.path());
            let local = repo_a.join("docs/baron/harness/IMPROVEMENTS.md");
            let shared = context_a
                .project_root
                .join("ProductHarness/IMPROVEMENTS.md");
            write(&local, "# Local user improvements\n");
            write(&shared, "# Shared user improvements\n");
            fs::write(if invalid_shared { &shared } else { &local }, [0xff]).unwrap();
            let before = [&local, &shared].map(|path| fs::read(path).unwrap());

            let result = if record_outcome {
                record_improvement_outcome(&repo_a, &context_a, "local", "do not publish")
            } else {
                propose_improvements(&repo_a, &context_a).map(|_| ())
            };

            assert!(
                result.is_err(),
                "an unreadable document was silently replaced"
            );
            for (path, bytes) in [&local, &shared].into_iter().zip(before) {
                assert_eq!(fs::read(path).unwrap(), bytes, "changed {}", path.display());
            }
        }
    }
}
