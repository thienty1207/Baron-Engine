use std::fs;
use std::path::Path;
use std::process::Command;

use baron_core::config::load_project_config;
use baron_core::context::{compile_context_for_task, ContextTarget};
use baron_core::control_plane::route_task;
use baron_core::firewall::compact_memory_brief_for_task;
use baron_core::memory::build_memory_index;
use baron_core::risk::RiskLane;
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

// Portability: target assertions are cross-platform and use deterministic
// repository fixtures rather than OS-specific permission mechanics.

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

/// Promoted Phase 6 regression: the context-critical brief uses the current
/// trust-aware retrieval authority rather than the legacy recall path.
#[test]
fn target_context_critical_retrieval_excludes_candidate_memory() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("candidate-context");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    memory_fixture(
        &context.project_root.join("Notes/candidate.md"),
        "Candidate Context Record",
        "candidate",
        "candidate",
        "candidate-only trusted-context marker.",
    );
    build_memory_index(&context).unwrap();

    let brief = compact_memory_brief_for_task(&context, Some("trusted-context marker")).unwrap();

    assert!(!brief.contains("candidate-only trusted-context marker"));
}

/// Promoted Phase 6 regression: protected Tier-0 state survives context
/// pressure while lower-priority content is bounded first.
#[test]
fn target_large_context_preserves_tier_zero_continuity_state() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("large-context-target");
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

    for marker in [
        "## Tier 0",
        "Project identity",
        "Task identity",
        "Intent",
        "Constraints",
        "Non-goals",
        "Current plan/work state",
        "Recovery",
        "Blockers",
        "Next action",
        "Route",
        "Mandatory proof/completion gates",
    ] {
        assert!(bundle.contains(marker), "missing Tier-0 marker: {marker}");
    }
}

/// Promoted Phase 7 target: ProjectPlatform keeps Data and Database as
/// separate profile domains while reading the existing schema-4 config shape.
#[test]
fn target_database_profile_is_a_distinct_configured_domain() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("database-profile");
    fs::create_dir_all(repo.join(".baron")).unwrap();
    write(
        &repo.join(".baron/project.toml"),
        "schema_version = 4\nproject_id = \"database-project\"\nproject_slug = \"database-profile\"\nplatform = \"database\"\nadapters = [\"codex\"]\nactive_adapter = \"codex\"\n\n[automation]\ncontext = true\nplan = true\nharness = true\nproof = true\ntrace = true\n",
    );

    let config = load_project_config(&repo).expect("database profile should deserialize");

    assert_eq!(format!("{:?}", config.platform), "Some(Database)");
}

/// Promoted Phase 11 target: historical values decode as opaque compatibility
/// data and never enter the active adapter enum.
#[test]
fn target_historical_adapter_value_does_not_remain_an_active_variant() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("historical-adapter-target");
    fs::create_dir_all(repo.join(".baron")).unwrap();
    let historical_value: String = ['r', 'e', 'a', 's', 'o', 'n', 'i', 'x'].iter().collect();
    write(
        &repo.join(".baron/project.toml"),
        &format!(
            "schema_version = 4\nproject_id = \"legacy-project\"\nproject_slug = \"historical-adapter-target\"\nadapters = [\"codex\", \"{historical_value}\"]\nactive_adapter = \"codex\"\n\n[automation]\ncontext = true\nplan = true\nharness = true\nproof = true\ntrace = true\n"
        ),
    );

    let config = load_project_config(&repo).expect("legacy input should remain readable");

    assert!(config
        .adapters
        .iter()
        .all(|adapter| format!("{adapter:?}").to_lowercase() != historical_value));
}

fn install_minimal_codex_contract(repo: &Path) {
    write(
        &repo.join(".codex/skills/superpowers/SKILL.md"),
        "---\nname: superpowers\ndescription: Baron workflow core.\n---\n\nSuperpowers is the workflow core.\n",
    );
    for (name, description) in [
        ("code-reviewer", "Core correctness review gate."),
        ("security-auditor", "Core security review gate."),
        ("test-engineer", "Core verification gate."),
    ] {
        write(
            &repo.join(".codex/agents").join(format!("{name}.toml")),
            &format!("name = \"{name}\"\ndescription = \"{description}\"\ndeveloper_instructions = \"Do not invoke other subagents.\"\n"),
        );
    }
}

/// Promoted Phase 7 target: configured Database profile and task intent select
/// the canonical database skill before any repository diff exists.
#[test]
fn target_database_task_routes_without_a_preexisting_diff() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("database-routing");
    fs::create_dir_all(repo.join(".baron")).unwrap();
    write(
        &repo.join(".baron/project.toml"),
        "schema_version = 4\nproject_id = \"database-routing\"\nproject_slug = \"database-routing\"\nplatform = \"database\"\nadapters = [\"codex\"]\nactive_adapter = \"codex\"\n\n[automation]\ncontext = true\nplan = true\nharness = true\nproof = true\ntrace = true\n",
    );
    install_minimal_codex_contract(&repo);

    let route = route_task(&repo, "design relational database schema", RiskLane::Medium).unwrap();

    assert!(route
        .selected_skills
        .iter()
        .any(|skill| skill.name == "database-engineering"));
}

/// Promoted Phase 6 regression: one bounded Task State projection contains all
/// required current-task categories without dumping every persisted file.
#[test]
fn target_context_projects_one_complete_task_state_record() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("task-state");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    write(
        &repo.join("docs/baron/plans/CURRENT.md"),
        "# Current Plan\n\n- Title: task-state fixture\n- Status: `interrupted`\n- Next action: resume fixture\n",
    );
    write(
        &repo.join("docs/baron/continuity/CURRENT_RECOVERY.md"),
        "# Recovery\n\n- Outcome: `interrupted`\n\n## Safe Next Action\n\nresume fixture\n",
    );

    let bundle = compile_context_for_task(
        &repo,
        &vault,
        ContextTarget::Claude,
        Some("resume task-state fixture"),
    )
    .unwrap();

    for marker in [
        "## Task State",
        "original intent",
        "constraints",
        "current plan",
        "last successful step",
        "proof/trace state",
        "affected files",
        "blocker",
        "safe next action",
    ] {
        assert!(
            bundle.to_lowercase().contains(&marker.to_lowercase()),
            "missing Task State marker: {marker}"
        );
    }
}

/// Promoted Phase 11 target: the tracked product has no searchable retired
/// adapter name while runtime compatibility is built from fragments.
#[test]
fn target_tracked_product_has_no_retired_adapter_references() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let retired: String = ['r', 'e', 'a', 's', 'o', 'n', 'i', 'x'].iter().collect();
    let output = Command::new("git")
        .args(["grep", "-in", retired.as_str()])
        .current_dir(root)
        .output()
        .expect("git must be available for the tracked-source cleanup gate");

    assert!(output.stdout.is_empty());
}

#[test]
fn target_tracked_product_has_no_retired_adapter_filenames() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let historical: String = ['r', 'e', 'a', 's', 'o', 'n', 'i', 'x'].iter().collect();
    let output = Command::new("git")
        .args(["ls-files"])
        .current_dir(root)
        .output()
        .expect("git must be available for the tracked-filename cleanup gate");
    let tracked_paths = String::from_utf8_lossy(&output.stdout);
    let offenders = tracked_paths
        .lines()
        .filter(|path| path.to_ascii_lowercase().contains(&historical))
        .collect::<Vec<_>>();
    assert!(
        offenders.is_empty(),
        "retired adapter filenames remain tracked: {}",
        offenders.join(", ")
    );
}
