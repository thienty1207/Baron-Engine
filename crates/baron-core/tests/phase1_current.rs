use std::fs;
use std::path::Path;

use baron_core::config::{
    active_adapter, initialize_project, initialize_project_with_options, load_project_config,
    resolve_vault_path_for_repo, AdapterKind, ProjectPlatform,
};
use baron_core::context::{compile_context_for_task, ContextTarget};
use baron_core::continuity::{
    continuity_status, record_continuity_checkpoint, record_recovery, RecoveryInput,
    RecoveryOutcome,
};
use baron_core::firewall::{compact_memory_brief_for_task, recall, recall_v5};
use baron_core::intent::{record_intent, IntentBriefInput};
use baron_core::memory::{build_memory_index, load_memory_records};
use baron_core::platform::ensure_platform_intelligence;
use baron_core::proof::record_proof;
use baron_core::trace::{record_trace, score_trace, TraceOutcome};
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

// Portability: all Phase 1 core fixtures are cross-platform and avoid timing
// assumptions or platform-specific filesystem behavior.

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
fn current_multi_adapter_config_keeps_one_project_identity_and_shared_vault() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("multi-adapter");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    let codex = initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let claude = initialize_project(&repo, AdapterKind::Claude, &vault).unwrap();
    let loaded = load_project_config(&repo).unwrap();

    assert_eq!(codex.project_id, claude.project_id);
    assert_eq!(loaded.project_id, codex.project_id);
    assert_eq!(loaded.adapters, [AdapterKind::Codex, AdapterKind::Claude]);
    assert_eq!(active_adapter(&loaded), Some(AdapterKind::Claude));
    assert_eq!(resolve_vault_path_for_repo(None, &repo).unwrap(), vault);

    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let switched = load_project_config(&repo).unwrap();
    assert_eq!(active_adapter(&switched), Some(AdapterKind::Codex));
    assert_eq!(switched.project_id, codex.project_id);
    assert_eq!(resolve_vault_path_for_repo(None, &repo).unwrap(), vault);
}

#[test]
fn current_malformed_project_toml_is_reported_without_partial_config() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("malformed-config");
    fs::create_dir_all(repo.join(".baron")).unwrap();
    write(
        &repo.join(".baron/project.toml"),
        "schema_version = 4\nproject_slug = [unterminated\n",
    );

    let error = load_project_config(&repo).unwrap_err().to_string();

    assert!(error.contains("parse") || error.contains("Could not parse"));
}

#[test]
fn current_historical_adapter_value_deserializes_for_migration_input() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("historical-adapter");
    fs::create_dir_all(repo.join(".baron")).unwrap();
    let historical_value: String = ['r', 'e', 'a', 's', 'o', 'n', 'i', 'x'].iter().collect();
    write(
        &repo.join(".baron/project.toml"),
        &format!(
            "schema_version = 4\nproject_id = \"legacy-project\"\nproject_slug = \"historical-adapter\"\nadapters = [\"codex\", \"{historical_value}\"]\nactive_adapter = \"codex\"\n\n[automation]\ncontext = true\nplan = true\nharness = true\nproof = true\ntrace = true\n"
        ),
    );

    let config = load_project_config(&repo).unwrap();

    assert_eq!(config.adapters, vec![AdapterKind::Codex]);
    assert_eq!(config.legacy_adapters, vec![historical_value]);
    assert_eq!(config.active_adapter, Some(AdapterKind::Codex));
}

#[test]
fn current_platform_profiles_generate_fullstack_backend_data_and_mobile_assets() {
    for platform in [
        ProjectPlatform::Fullstack,
        ProjectPlatform::Backend,
        ProjectPlatform::Data,
        ProjectPlatform::Mobile,
    ] {
        let temp = tempdir().unwrap();
        let repo = temp
            .path()
            .join(format!("{}-project", platform_name(platform)));
        let vault = temp.path().join("Vault");
        fs::create_dir_all(&repo).unwrap();
        let config = initialize_project_with_options(
            &repo,
            Some(AdapterKind::Codex),
            &vault,
            Some(platform),
        )
        .unwrap();

        let report = ensure_platform_intelligence(&repo, &config).unwrap();

        assert!(report.project_profile.is_file());
        assert!(report
            .profile_files
            .iter()
            .any(|path| path.file_stem().and_then(|name| name.to_str())
                == Some(platform_name(platform))));
        let profile = fs::read_to_string(
            repo.join("docs/baron/platform/profiles")
                .join(format!("{}.md", platform_name(platform))),
        )
        .unwrap();
        assert!(profile
            .to_lowercase()
            .contains(&format!("{} engineering profile", platform_name(platform))));
    }
}

fn platform_name(platform: ProjectPlatform) -> &'static str {
    match platform {
        ProjectPlatform::Frontend => "frontend",
        ProjectPlatform::Backend => "backend",
        ProjectPlatform::Fullstack => "fullstack",
        ProjectPlatform::Mobile => "mobile",
        ProjectPlatform::Desktop => "desktop",
        ProjectPlatform::Tool => "tool",
        ProjectPlatform::Library => "library",
        ProjectPlatform::Data => "data",
        ProjectPlatform::Database => "database",
        ProjectPlatform::Cloud => "cloud",
        ProjectPlatform::Unknown => "unknown",
    }
}

#[test]
fn current_trust_fixture_exercises_verified_likely_candidates_and_project_firewall() {
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
        "Verified Current Evidence",
        "verified",
        "active",
        "trust-fixture verified current evidence is backed by a passing test.",
    );
    memory_fixture(
        &context.project_root.join("Notes/likely.md"),
        "Likely Current Evidence",
        "likely",
        "active",
        "trust-fixture likely current evidence needs ordinary review.",
    );
    memory_fixture(
        &context.project_root.join("Notes/candidate.md"),
        "Candidate Current Evidence",
        "candidate",
        "candidate",
        "trust-fixture candidate must remain a proposal.",
    );
    memory_fixture(
        &context.project_root.join("Notes/contested.md"),
        "Contested Current Evidence",
        "likely",
        "contested",
        "trust-fixture contested evidence has conflicting observations.",
    );
    memory_fixture(
        &context.project_root.join("Notes/superseded.md"),
        "Superseded Current Evidence",
        "likely",
        "superseded",
        "trust-fixture superseded evidence is retained for lineage.",
    );
    memory_fixture(
        &context.project_root.join("Notes/expired.md"),
        "Expired Current Evidence",
        "stale",
        "expired",
        "trust-fixture expired evidence is no longer current.",
    );
    memory_fixture(
        &other_context.project_root.join("Facts.md"),
        "Cross Project Evidence",
        "verified",
        "active",
        "trust-fixture cross project evidence stays isolated.",
    );
    write(
        &vault_root.join("Artifacts/Baron/APPROVED_GLOBAL.md"),
        "# Approved Global\n\n- trust-fixture approved global guidance is safe for all Baron repositories.\n",
    );
    write(
        &vault_root.join("Artifacts/Baron/GLOBAL_CANDIDATES.md"),
        "# Global Candidates\n\n- trust-fixture global candidate is not approved.\n",
    );

    build_memory_index(&context).unwrap();
    let records = load_memory_records(&context).unwrap();
    assert!(records
        .iter()
        .any(|record| record.excerpt.contains("verified current")));
    assert!(records
        .iter()
        .any(|record| record.excerpt.contains("global candidate")));

    let trusted = recall_v5(&context, "trust-fixture", 30).unwrap();
    assert!(trusted
        .results
        .iter()
        .any(|hit| hit.record.excerpt.contains("verified current")));
    assert!(trusted
        .results
        .iter()
        .any(|hit| hit.record.excerpt.contains("approved global")));
    for marker in [
        "candidate must",
        "contested evidence",
        "superseded evidence",
        "expired evidence",
    ] {
        assert!(
            !trusted
                .results
                .iter()
                .any(|hit| hit.record.excerpt.contains(marker)),
            "unexpected trusted hit: {marker}"
        );
    }
    assert!(!trusted
        .results
        .iter()
        .any(|hit| hit.record.excerpt.contains("cross project")));

    let explicit = recall(&context, "trust-fixture other-project", 30).unwrap();
    assert!(explicit
        .results
        .iter()
        .any(|hit| hit.record.excerpt.contains("cross project")));
}

#[test]
fn current_context_critical_retrieval_uses_the_trusted_recall_path() {
    let temp = tempdir().unwrap();
    let vault_root = temp.path().join("Vault");
    let repo = temp.path().join("candidate-path");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault_root, &repo).unwrap();
    memory_fixture(
        &context.project_root.join("Notes/candidate.md"),
        "Candidate Context Record",
        "candidate",
        "candidate",
        "candidate-only context-critical retrieval marker.",
    );
    build_memory_index(&context).unwrap();

    let current_brief =
        compact_memory_brief_for_task(&context, Some("context-critical retrieval")).unwrap();
    let trusted = recall_v5(&context, "context-critical retrieval", 10).unwrap();

    assert!(!current_brief.contains("candidate-only context-critical retrieval marker"));
    assert!(!trusted
        .results
        .iter()
        .any(|hit| hit.record.excerpt.contains("candidate-only")));
}

#[test]
fn current_interrupted_continuity_fixture_preserves_resume_evidence() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("interrupted-task");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let config = initialize_project(&repo, AdapterKind::Codex, &vault_root).unwrap();
    let context = ensure_vault(&vault_root, &repo).unwrap();
    record_intent(
        &repo,
        &context,
        IntentBriefInput {
            title: "resume interrupted auth task".to_string(),
            current_behavior: "The task stopped before the verification step.".to_string(),
            target_behavior: "Resume from the recorded checkpoint without restating scope."
                .to_string(),
            scope: "The auth integration test only.".to_string(),
            non_goals: vec!["No unrelated refactor.".to_string()],
            constraints: vec!["Keep the existing API contract.".to_string()],
            decisions: vec!["Use the current test fixture.".to_string()],
            required_proof: "The auth integration test passes.".to_string(),
            unknowns: vec!["Remote service availability.".to_string()],
            confirmed: true,
        },
    )
    .unwrap();
    write(
        &repo.join("docs/baron/plans/CURRENT.md"),
        "# Current Plan\n\n- Title: resume interrupted auth task\n- Status: `interrupted`\n- Next action: rerun the auth integration test\n",
    );
    write(
        &repo.join("docs/baron/harness/CURRENT.md"),
        "# Current Harness\n\n- Title: resume interrupted auth task\n- Risk: `high`\n",
    );
    let proof = record_proof(
        &repo,
        &context,
        "Auth proof was interrupted before final assertion.",
    )
    .unwrap();
    let trace = record_trace(
        &repo,
        &context,
        "Auth trace stopped at the remote service boundary.",
        TraceOutcome::Blocked,
    )
    .unwrap();
    score_trace(&repo, &context, Some(&trace.id)).unwrap();
    let recovery = record_recovery(
        &repo,
        &context,
        RecoveryInput {
            outcome: RecoveryOutcome::Interrupted,
            root_cause: "The session ended before verification completed.".to_string(),
            last_successful_step: "Intent confirmed and intake created.".to_string(),
            evidence: vec![
                format!("Proof packet recorded at {}.", proof.repo_path.display()),
                "Constraint: keep the existing API contract.".to_string(),
            ],
            affected_files: vec!["backend/auth.rs".to_string()],
            next_action: "Rerun the auth integration test and inspect the assertion.".to_string(),
            retry_conditions: vec!["The remote service health check passes.".to_string()],
        },
    )
    .unwrap();
    let checkpoint = record_continuity_checkpoint(
        &repo,
        &context,
        "Interrupted after the first verification attempt.",
        "codex",
    )
    .unwrap();

    assert_eq!(config.project_slug, "interrupted-task");
    assert!(checkpoint.repo_path.is_file());
    assert!(recovery.repo_path.is_file());
    let status = continuity_status(&repo, &context).unwrap();
    for marker in [
        "resume interrupted auth task",
        "existing API contract",
        "Intent confirmed and intake created",
        "interrupted",
        "backend/auth.rs",
        "Rerun the auth integration test",
        "Proof status",
        "Trace status",
    ] {
        assert!(
            status.contains(marker),
            "missing continuity marker: {marker}"
        );
    }
}

fn write_large_context_fixture(repo: &Path) {
    let domain_terms = (0..120)
        .map(|index| {
            format!(
                "| term-{index:03} | pressure fixture meaning {index:03} | verified | fixture.md |\n"
            )
        })
        .collect::<String>();
    write(
        &repo.join("docs/baron/plans/CURRENT.md"),
        &format!(
            "# Current Plan\n\n- Status: `in_progress`\n{}",
            "plan-pressure-marker\n".repeat(8_000)
        ),
    );
    write(
        &repo.join("docs/baron/harness/CURRENT.md"),
        &format!(
            "# Current Harness\n\n- Risk: `high`\n{}",
            "harness-pressure-marker\n".repeat(2_000)
        ),
    );
    write(
        &repo.join("docs/baron/harness/CURRENT_INTENT.md"),
        &format!(
            "# Current Intent\n\n- Title: pressure fixture\n{}",
            "intent-pressure-marker\n".repeat(5_000)
        ),
    );
    write(
        &repo.join("docs/baron/continuity/CURRENT.md"),
        &format!(
            "# Continuity\n\n- Next action: preserve Tier 0\n{}",
            "continuity-pressure-marker\n".repeat(4_000)
        ),
    );
    write(
        &repo.join("docs/baron/continuity/CURRENT_RECOVERY.md"),
        &format!(
            "# Recovery\n\n- Safe next action: resume safely\n{}",
            "recovery-pressure-marker\n".repeat(5_000)
        ),
    );
    write(
        &repo.join("docs/baron/architecture/CURRENT_ARCHITECTURE.md"),
        &format!(
            "# Architecture\n\n{}",
            "architecture-pressure-marker\n".repeat(4_000)
        ),
    );
    write(
        &repo.join("docs/baron/proofs/INDEX.md"),
        &format!(
            "# Proof Index\n\n{}",
            "proof-pressure-marker\n".repeat(2_000)
        ),
    );
    write(
        &repo.join("docs/baron/traces/INDEX.md"),
        &format!(
            "# Trace Index\n\n{}",
            "trace-pressure-marker\n".repeat(2_000)
        ),
    );
    write(
        &repo.join("docs/baron/harness/DOMAIN_LANGUAGE.md"),
        &format!(
            "# Product Domain Language\n\n## Terms\n\n| Term | Meaning | Status | Evidence |\n| --- | --- | --- | --- |\n{domain_terms}"
        ),
    );
}

#[test]
fn current_context_budget_fixture_preserves_priority_allocation() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("large-context");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    write_large_context_fixture(&repo);

    let bundle = compile_context_for_task(
        &repo,
        &vault,
        ContextTarget::Codex,
        Some("preserve Tier 0 state under pressure"),
    )
    .unwrap();

    assert!(bundle.chars().count() <= 20_000);
    assert!(bundle.contains("## Tier 0 — Protected Task State"));
    assert!(bundle.contains("## Context Budget"));
    assert!(!bundle.contains("[Context truncated by Baron"));
}
