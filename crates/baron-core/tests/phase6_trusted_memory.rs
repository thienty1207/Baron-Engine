use std::fs;
use std::path::Path;

use baron_core::context::{compile_context_for_task, ContextTarget};
use baron_core::firewall::{
    compact_memory_brief_for_task, record_is_currently_trusted, trusted_recall,
    trusted_recall_current, TrustedRecallPolicy,
};
use baron_core::intelligence41::{
    load_temporal_ledger, refresh_temporal_ledger, temporal_ledger_path,
};
use baron_core::memory::{build_memory_index, load_memory_records};
use baron_core::task_state::{compile_task_state, render_task_state};
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn memory_fixture(path: &Path, title: &str, confidence: &str, status: &str, excerpt: &str) {
    write(
        path,
        &format!("---\nconfidence: {confidence}\nstatus: {status}\n---\n#{title}\n\n- {excerpt}\n"),
    );
}

#[test]
fn phase6_named_trusted_recall_enforces_current_memory_matrix() {
    let temp = tempdir().unwrap();
    let vault_root = temp.path().join("Vault");
    let repo = temp.path().join("current-project");
    let other = temp.path().join("other-project");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&other).unwrap();
    let context = ensure_vault(&vault_root, &repo).unwrap();
    let other_context = ensure_vault(&vault_root, &other).unwrap();

    memory_fixture(
        &context.project_root.join("Facts.md"),
        "Phase 6 Verified Evidence",
        "verified",
        "active",
        "phase6 matrix verified current evidence is safe task truth.",
    );
    memory_fixture(
        &context.project_root.join("Notes/likely.md"),
        "Phase 6 Likely Evidence",
        "likely",
        "active",
        "phase6 matrix likely current evidence remains eligible.",
    );
    memory_fixture(
        &context.project_root.join("Notes/candidate.md"),
        "Phase 6 Candidate Evidence",
        "candidate",
        "candidate",
        "phase6 matrix candidate evidence must stay out of task truth.",
    );
    memory_fixture(
        &context.project_root.join("Notes/contested.md"),
        "Phase 6 Contested Evidence",
        "likely",
        "contested",
        "phase6 matrix contested evidence has unresolved conflict.",
    );
    memory_fixture(
        &context.project_root.join("Notes/superseded.md"),
        "Phase 6 Superseded Evidence",
        "likely",
        "superseded",
        "phase6 matrix superseded evidence is historical only.",
    );
    memory_fixture(
        &context.project_root.join("Notes/expired.md"),
        "Phase 6 Expired Evidence",
        "stale",
        "expired",
        "phase6 matrix expired evidence is no longer current.",
    );
    memory_fixture(
        &other_context.project_root.join("Facts.md"),
        "Phase 6 Cross Project Evidence",
        "verified",
        "active",
        "phase6 matrix cross project evidence stays isolated.",
    );
    write(
        &vault_root.join("Artifacts/Baron/APPROVED_GLOBAL.md"),
        "# Approved Global\n\n- phase6 matrix approved global guidance is allowed when relevant.\n",
    );
    write(
        &vault_root.join("Artifacts/Baron/GLOBAL_CANDIDATES.md"),
        "# Global Candidates\n\n- phase6 matrix global candidate is not approved.\n",
    );
    build_memory_index(&context).unwrap();

    let policy_result =
        trusted_recall(&context, "phase6 matrix", 40, TrustedRecallPolicy::Current).unwrap();
    let wrapper_result = trusted_recall_current(&context, "phase6 matrix", 40).unwrap();
    assert_eq!(policy_result, wrapper_result);
    assert!(policy_result
        .results
        .iter()
        .any(|hit| hit.record.excerpt.contains("verified current")));
    assert!(policy_result
        .results
        .iter()
        .any(|hit| hit.record.excerpt.contains("approved global")));
    for marker in [
        "candidate evidence",
        "contested evidence",
        "superseded evidence",
        "expired evidence",
        "cross project evidence",
        "global candidate",
    ] {
        assert!(
            !policy_result
                .results
                .iter()
                .any(|hit| hit.record.excerpt.contains(marker)),
            "untrusted marker entered current retrieval: {marker}"
        );
    }

    let brief = compact_memory_brief_for_task(&context, Some("phase6 matrix")).unwrap();
    assert!(brief.contains("verified current"));
    assert!(brief.contains("approved global"));
    assert!(!brief.contains("candidate evidence"));
    assert!(!brief.contains("global candidate"));
    assert!(load_memory_records(&context)
        .unwrap()
        .iter()
        .filter(|record| record.excerpt.contains("candidate evidence"))
        .all(|record| !record_is_currently_trusted(record)));
}

#[test]
fn phase6_task_state_projects_authorities_without_persisting_a_new_record() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("task-state");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let vault = ensure_vault(&vault_root, &repo).unwrap();
    write(
        &repo.join("docs/baron/harness/CURRENT_INTENT.md"),
        "# Intent\n\n- Title: resume authentication migration\n\n## Target Behavior\n\n- restore the authenticated flow\n\n## Constraints\n\n- preserve the API contract\n\n## Non-Goals\n\n- no unrelated cleanup\n",
    );
    write(
        &repo.join("docs/baron/plans/CURRENT.md"),
        "# Plan\n\n- Title: unrelated frontend dashboard polish\n- Status: `interrupted`\n- Next action: inspect the dashboard fixture\n",
    );
    write(
        &repo.join("docs/baron/continuity/CURRENT.md"),
        "# Continuity\n\n- Current task: resume authentication migration\n- Next action: resume from the checkpoint\n- Changed files: src/auth.rs\n",
    );
    write(
        &repo.join("docs/baron/continuity/CURRENT_RECOVERY.md"),
        "# Recovery\n\n- Outcome: `interrupted`\n\n## Last Successful Step\n\nintent was confirmed\n\n## Affected Files\n\n- src/auth.rs\n\n## Safe Next Action\n\nresume from the checkpoint\n",
    );

    let first = compile_task_state(&repo, &vault, Some("resume authentication migration")).unwrap();
    let second =
        compile_task_state(&repo, &vault, Some("resume authentication migration")).unwrap();
    assert_eq!(first, second);
    assert!(first.resumed);
    assert!(!first.conflicts.is_empty());
    assert!(first
        .affected_files
        .iter()
        .any(|path| path == "src/auth.rs"));
    let rendered = render_task_state(&first, 6_000);
    for marker in [
        "## Task State",
        "Original intent",
        "Constraints",
        "Current plan/work state",
        "Last successful step",
        "Proof/trace state",
        "Affected files",
        "Blockers",
        "Safe next action",
    ] {
        assert!(
            rendered.contains(marker),
            "missing task-state marker: {marker}"
        );
    }
    assert!(!repo.join(".baron/task-state.json").exists());
    assert!(!vault.project_root.join("TaskState.md").exists());
}

#[test]
fn phase6_temporal_contested_entries_are_excluded_without_dropping_approved_global() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("temporal-project");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault_root, &repo).unwrap();
    memory_fixture(
        &context.project_root.join("Facts.md"),
        "Phase 6 Temporal Current",
        "verified",
        "active",
        "phase6 temporal current project evidence",
    );
    write(
        &vault_root.join("Artifacts/Baron/APPROVED_GLOBAL.md"),
        "# Approved Global\n\n- phase6 temporal approved global evidence\n",
    );
    build_memory_index(&context).unwrap();
    refresh_temporal_ledger(&context).unwrap();
    let mut ledger = load_temporal_ledger(&context).unwrap();
    assert!(!ledger.entries.is_empty());
    ledger.entries[0].contested = true;
    write(
        &temporal_ledger_path(&context),
        &serde_json::to_string_pretty(&ledger).unwrap(),
    );

    let result = trusted_recall_current(&context, "phase6 temporal evidence", 20).unwrap();
    assert!(!result
        .results
        .iter()
        .any(|hit| hit.record.excerpt.contains("current project evidence")));
    assert!(result
        .results
        .iter()
        .any(|hit| hit.record.excerpt.contains("approved global evidence")));
}

#[test]
fn phase6_context_priority_output_is_bounded_and_reproducible() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("pressure");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    write(
        &repo.join("docs/baron/plans/CURRENT.md"),
        &format!(
            "# Current Plan\n\n- Title: pressure fixture\n- Status: `in_progress`\n- Next action: preserve Tier 0\n{}",
            "- detail: pressure\n".repeat(2_000)
        ),
    );
    write(
        &repo.join("docs/baron/continuity/CURRENT_RECOVERY.md"),
        "# Recovery\n\n- Outcome: `interrupted`\n\n## Safe Next Action\n\nresume pressure fixture\n",
    );

    let first = compile_context_for_task(
        &repo,
        &vault,
        ContextTarget::Codex,
        Some("preserve Tier 0 state under pressure"),
    )
    .unwrap();
    let second = compile_context_for_task(
        &repo,
        &vault,
        ContextTarget::Codex,
        Some("preserve Tier 0 state under pressure"),
    )
    .unwrap();
    assert!(first.chars().count() <= 20_000);
    assert!(first.contains("## Tier 0 — Protected Task State"));
    assert!(first.contains("## Task State"));
    assert!(first.contains("Project identity"));
    assert!(first.contains("Next action"));
    assert!(first.contains("Mandatory proof/completion gates"));
    assert_eq!(first, second);
}
