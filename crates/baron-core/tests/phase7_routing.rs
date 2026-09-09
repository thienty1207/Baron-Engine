use std::fs;
use std::path::Path;

use baron_core::config::{initialize_project_with_options, AdapterKind, ProjectPlatform};
use baron_core::control_plane::route_task;
use baron_core::risk::RiskLane;
use tempfile::tempdir;

// Portability: these routing fixtures are cross-platform and use only
// deterministic temporary files; no Unix or Windows filesystem mechanics.

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn install_contract(repo: &Path, platform: ProjectPlatform) {
    let vault = repo.join("Vault");
    initialize_project_with_options(repo, Some(AdapterKind::Codex), &vault, Some(platform))
        .unwrap();
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
            &format!(
                "name = \"{name}\"\ndescription = \"{description}\"\ndeveloper_instructions = \"Do not invoke other subagents.\"\n"
            ),
        );
    }
}

fn names(route: &baron_core::control_plane::RouteReport) -> Vec<String> {
    route
        .selected_skills
        .iter()
        .map(|item| item.name.clone())
        .collect()
}

#[test]
fn database_profile_routes_relational_work_without_a_diff() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("database");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Database);

    let route = route_task(
        &repo,
        "design relational schema with FK constraints",
        RiskLane::Medium,
    )
    .unwrap();

    assert!(
        names(&route).contains(&"database-engineering".to_string()),
        "database evidence was not selected: {route:?}"
    );
    assert!(route.explanation.to_lowercase().contains("database"));
}

#[test]
fn data_pipeline_does_not_load_database_skill_without_database_evidence() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("data");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Data);

    let route = route_task(
        &repo,
        "transform CSV ingestion pipeline with lineage and quality checks",
        RiskLane::Medium,
    )
    .unwrap();

    assert!(
        !names(&route).contains(&"database-engineering".to_string()),
        "unexpected database route: {route:?}"
    );
    assert!(route
        .verification
        .iter()
        .any(|item| item.name == "data-integrity"));
    assert!(route.explanation.to_lowercase().contains("data"));
}

#[test]
fn database_and_data_evidence_can_coexist_when_a_pipeline_changes_persistence() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("mixed-data-database");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Data);

    let route = route_task(
        &repo,
        "migrate a PostgreSQL-backed ETL pipeline with data quality checks",
        RiskLane::High,
    )
    .unwrap();

    assert!(names(&route).contains(&"database-engineering".to_string()));
    assert!(route
        .verification
        .iter()
        .any(|item| item.name == "schema-integrity"));
    assert!(route
        .verification
        .iter()
        .any(|item| item.name == "data-integrity"));
    let explanation = route.explanation.to_lowercase();
    assert!(
        explanation.contains("database and data"),
        "route: {route:?}"
    );
}

#[test]
fn incidental_sql_and_pure_http_work_do_not_load_database_skill() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("non-database");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Backend);

    for task in [
        "write a pure HTTP handler with no persistence concern",
        "review an SQL string mentioned incidentally",
    ] {
        let route = route_task(&repo, task, RiskLane::Medium).unwrap();
        assert!(
            !names(&route).contains(&"database-engineering".to_string()),
            "unexpected database route for {task}: {route:?}"
        );
    }
}

#[test]
fn mobile_application_work_is_distinct_from_apk_reverse_analysis() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("mobile");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Mobile);

    let app = route_task(
        &repo,
        "fix Android app offline navigation and lifecycle state",
        RiskLane::Medium,
    )
    .unwrap();
    assert!(names(&app).contains(&"mobile-application-engineering".to_string()));
    assert!(!names(&app).contains(&"apk-mobile-analysis".to_string()));
    assert!(app
        .verification
        .iter()
        .any(|item| item.name == "mobile-lifecycle"));
    assert!(app
        .verification
        .iter()
        .any(|item| item.name == "mobile-network-failure"));

    let reverse = route_task(
        &repo,
        "static APK manifest and Android binary review",
        RiskLane::High,
    )
    .unwrap();
    assert!(names(&reverse).contains(&"apk-mobile-analysis".to_string()));
    assert!(!names(&reverse).contains(&"mobile-application-engineering".to_string()));
}

#[test]
fn mobile_release_build_keeps_normal_application_routing() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("mobile-release");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Mobile);

    let release = route_task(&repo, "prepare Android APK release build", RiskLane::Low).unwrap();

    assert!(names(&release).contains(&"mobile-application-engineering".to_string()));
    assert!(!names(&release).contains(&"apk-mobile-analysis".to_string()));
}

#[test]
fn mobile_profile_can_select_app_flow_verification_before_a_diff() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("mobile-profile");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Mobile);

    let route = route_task(&repo, "review the app flow", RiskLane::Low).unwrap();

    assert!(names(&route).contains(&"mobile-application-engineering".to_string()));
    assert!(route
        .verification
        .iter()
        .any(|item| item.name == "mobile-lifecycle"));
}

#[test]
fn ordinary_api_work_does_not_trigger_security_by_the_word_api() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("api");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Backend);

    let route = route_task(
        &repo,
        "design a public REST API response contract",
        RiskLane::Medium,
    )
    .unwrap();

    assert!(names(&route).contains(&"api-and-interface-design".to_string()));
    assert!(!names(&route).contains(&"vibe-security-scan".to_string()));
    assert!(!route
        .mandatory_agents
        .contains(&"security-auditor".to_string()));
}

#[test]
fn ordinary_words_containing_api_do_not_load_api_skill() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("api-substring");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Backend);

    let route = route_task(&repo, "review capital allocation notes", RiskLane::Low).unwrap();

    assert!(!names(&route).contains(&"api-and-interface-design".to_string()));
}

#[test]
fn task_evidence_can_override_a_backend_profile() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("override");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Backend);

    let route = route_task(
        &repo,
        "implement iOS app permission and deep-link flow",
        RiskLane::Medium,
    )
    .unwrap();

    assert!(names(&route).contains(&"mobile-application-engineering".to_string()));
    assert!(!names(&route).contains(&"api-and-interface-design".to_string()));
}

#[test]
fn routing_is_bounded_and_explains_profile_and_verification_influence() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("bounded");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Database);

    let route = route_task(
        &repo,
        "destructive schema migration with query plan, deadlock, auth API and observability",
        RiskLane::High,
    )
    .unwrap();

    assert!(route.selected_skills.len() <= 5);
    assert!(route.mandatory_agents.len() <= 3);
    let details = format!("{} {:?}", route.explanation, route.skipped).to_lowercase();
    assert!(details.contains("profile") || details.contains("database"));
    assert!(details.contains("verification") || details.contains("gate"));
}

#[test]
fn selected_candidate_reasons_identify_their_own_domain_evidence() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("candidate-reasons");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Backend);

    let route = route_task(
        &repo,
        "review the database query API contract",
        RiskLane::Medium,
    )
    .unwrap();
    let database = route
        .selected_skills
        .iter()
        .find(|item| item.name == "database-engineering")
        .expect("database skill should be selected");
    let api = route
        .selected_skills
        .iter()
        .find(|item| item.name == "api-and-interface-design")
        .expect("API skill should be selected");

    assert!(database.reason.contains("database"));
    assert!(api.reason.contains("API") || api.reason.contains("interface"));
}

#[test]
fn equivalent_profile_task_state_routes_deterministically() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("deterministic");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Database);

    let first = route_task(&repo, "add a safe database index", RiskLane::Medium).unwrap();
    let second = route_task(&repo, "add a safe database index", RiskLane::Medium).unwrap();

    assert_eq!(first, second);
}

#[test]
fn database_profile_influences_verification_for_migration_work() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("verification");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Database);

    let route = route_task(
        &repo,
        "add an additive database migration and backfill",
        RiskLane::High,
    )
    .unwrap();
    let details = format!("{} {:?}", route.explanation, route.skipped).to_lowercase();
    assert!(
        details.contains("migration")
            || details.contains("rollback")
            || details.contains("verification")
    );
}

#[test]
fn configured_database_profile_can_select_persistence_work_before_a_diff() {
    let temp = tempdir().unwrap();
    let database_repo = temp.path().join("configured-database");
    let backend_repo = temp.path().join("configured-backend");
    fs::create_dir_all(&database_repo).unwrap();
    fs::create_dir_all(&backend_repo).unwrap();
    install_contract(&database_repo, ProjectPlatform::Database);
    install_contract(&backend_repo, ProjectPlatform::Backend);

    let database_route = route_task(
        &database_repo,
        "review data access boundary",
        RiskLane::Medium,
    )
    .unwrap();
    let backend_route = route_task(
        &backend_repo,
        "review data access boundary",
        RiskLane::Medium,
    )
    .unwrap();

    assert!(names(&database_route).contains(&"database-engineering".to_string()));
    assert!(!names(&backend_route).contains(&"database-engineering".to_string()));
    assert!(database_route
        .verification
        .iter()
        .any(|item| item.name == "schema-integrity"));
}

#[test]
fn routing_consumes_work_shape_state_capabilities_and_affected_area() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("route-inputs");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Backend);
    write(
        &repo.join("docs/baron/continuity/CURRENT.md"),
        "# Continuity\n\n- Current task: review the changed boundary\n- Changed files: db/migrations/001_add_users.sql\n",
    );
    write(
        &repo.join("docs/baron/plans/CURRENT.md"),
        "# Current Plan\n\n- Status: `in_progress`\n- Next action: review the changed boundary\n",
    );
    write(
        &repo.join(".baron/cache/capability-state.json"),
        r#"{"schema_version":1,"adapter":"codex","checked_at":"2026-09-07T00:00:00Z","observations":[],"required_gaps":[],"optional_gaps":[]}"#,
    );

    let route = route_task(&repo, "review the changed boundary", RiskLane::Medium).unwrap();

    assert!(names(&route).contains(&"database-engineering".to_string()));
    let explanation = route.explanation.to_lowercase();
    assert!(explanation.contains("work_shape="));
    assert!(explanation.contains("state=evidence"));
    assert!(explanation.contains("capabilities=available"));
    assert!(explanation.contains("affected=db/migrations/001_add_users.sql"));
}

#[test]
fn routing_metadata_is_consumed_for_a_canonical_skill_candidate() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("metadata");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Database);
    write(
        &repo.join(".codex/skills/database-engineering/SKILL.md"),
        "---\nname: database-engineering\ndescription: Metadata fixture.\nrouting_triggers: zebra-marker\nprofile_affinities: database\nrouting_dependencies: superpowers\nevidence_requirements: marker evidence\nverification_hints: marker-check\n---\n\n# Metadata fixture\n",
    );

    let route = route_task(&repo, "zebra-marker", RiskLane::Medium).unwrap();

    assert!(
        names(&route).contains(&"database-engineering".to_string()),
        "metadata was ignored: {route:?}"
    );
    assert!(route
        .selected_skills
        .iter()
        .any(|item| item.reason.contains("dependency-selected")));
    assert!(route
        .verification
        .iter()
        .any(|item| item.name == "marker-check"));
}

#[test]
fn canonical_core_metadata_takes_precedence_over_adapter_local_copies() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("core-metadata");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Backend);
    write(
        &repo.join(".codex/skills/database-engineering/SKILL.md"),
        "---\nname: database-engineering\ndescription: Adapter-local metadata.\nrouting_triggers: adapter-marker\nprofile_affinities: backend\n---\n\n# Adapter local\n",
    );
    write(
        &repo.join(".baron/core/skills/database-engineering/SKILL.md"),
        "---\nname: database-engineering\ndescription: Canonical Core metadata.\nrouting_triggers: canonical-marker\nprofile_affinities: backend\nverification_hints: canonical-check\n---\n\n# Core\n",
    );

    let route = route_task(&repo, "canonical-marker", RiskLane::Medium).unwrap();

    assert!(names(&route).contains(&"database-engineering".to_string()));
    assert!(route
        .verification
        .iter()
        .any(|item| item.name == "canonical-check"));
}

#[test]
fn routing_metadata_enforces_exclusions_and_conflicts() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("metadata-guards");
    fs::create_dir_all(&repo).unwrap();
    install_contract(&repo, ProjectPlatform::Backend);
    let canonical = repo.join(".baron/core/skills/database-engineering/SKILL.md");
    write(
        &canonical,
        "---\nname: database-engineering\ndescription: Guard fixture.\nrouting_triggers: shared-marker\nrouting_exclusions: blocked-marker\nrouting_conflicts: api-and-interface-design\n---\n\n# Guard fixture\n",
    );

    let excluded = route_task(&repo, "shared-marker blocked-marker", RiskLane::Medium).unwrap();
    assert!(!names(&excluded).contains(&"database-engineering".to_string()));
    assert!(excluded
        .skipped
        .iter()
        .any(|item| item.contains("database-engineering excluded")));

    let conflicted = route_task(&repo, "shared-marker API", RiskLane::Medium).unwrap();
    assert!(names(&conflicted).contains(&"api-and-interface-design".to_string()));
    assert!(!names(&conflicted).contains(&"database-engineering".to_string()));
    assert!(conflicted
        .skipped
        .iter()
        .any(|item| item.contains("database-engineering conflict-suppressed")));
}
