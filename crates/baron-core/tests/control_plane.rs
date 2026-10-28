use std::fs;
use std::path::Path;

use baron_core::control_plane::{
    gate_evidence_status, record_gate_evidence, route_task, validate_control_plane,
};
use baron_core::risk::RiskLane;
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn install_minimal_codex_contract(repo: &Path) {
    write(
        &repo.join(".codex/skills/INDEX.md"),
        "# Baron Skill Routing\n\n\
| Skill | Role | Trigger |\n\
| --- | --- | --- |\n\
| Superpowers | workflow core | planning, TDD, debugging, review, verification |\n\
| `frontend-design` | optional domain | UI, layout, responsive, accessibility, browser-facing flows |\n\
| `security-audit` | optional domain | application source changes, security-sensitive boundaries, focused audit |\n",
    );
    write(
        &repo.join(".codex/agents/INDEX.md"),
        "# Baron Agent Routing\n\n\
| Agent | Gate |\n\
| --- | --- |\n\
| `code-reviewer` | correctness, regressions, maintainability, architecture |\n\
| `security-auditor` | exploitable security and sensitive-memory risks |\n\
| `test-engineer` | verification evidence and missing coverage |\n",
    );
    write(
        &repo.join(".codex/skills/superpowers/SKILL.md"),
        "---\nname: superpowers\ndescription: Baron workflow core for planning, TDD, debugging, review, and verification.\n---\n\n# Superpowers\n\nSuperpowers is Baron's only workflow core. Output must include proof and trace evidence.\n",
    );
    write(
        &repo.join(".codex/skills/frontend-design/SKILL.md"),
        "---\nname: frontend-design\ndescription: Use when building frontend UI, layout, responsive behavior, accessibility, or browser-facing flows.\n---\n\n# Frontend Design\n\nOptional domain skill. Superpowers remains the workflow authority. Output must include files, accessibility risks, and verification.\n",
    );
    write(
        &repo.join(".codex/skills/security-audit/SKILL.md"),
        "---\nname: security-audit\ndescription: Use for bounded security review of meaningful application source changes and security-sensitive boundaries.\n---\n\n# Baron Security Audit\n\nThe security-audit methodology is distinct from the independent security-auditor gate. Superpowers remains the workflow authority. Never claim an audit ran when only this route was selected.\n",
    );
    for (name, description) in [
        (
            "api-and-interface-design",
            "Use when designing or changing APIs, public interfaces, SDK contracts, request/response shapes, versioning, compatibility, or boundary behavior.",
        ),
        (
            "observability-and-instrumentation",
            "Use when adding logs, metrics, tracing, alerts, dashboards, SLOs, audit events, or production diagnostics.",
        ),
        (
            "performance-optimization",
            "Use when optimizing runtime speed, bundle size, database/query performance, caching, latency, throughput, or resource use.",
        ),
        (
            "deprecation-and-migration",
            "Use when migrating APIs, frameworks, data models, feature flags, config, compatibility layers, or legacy behavior.",
        ),
    ] {
        write(
            &repo.join(".codex/skills").join(name).join("SKILL.md"),
            &format!(
                "---\nname: {name}\ndescription: {description}\n---\n\n# {name}\n\nOptional Baron domain skill. Superpowers remains the workflow authority. Output must include evidence, risks, and verification.\n"
            ),
        );
    }
    for (name, description, instructions) in [
        (
            "code-reviewer",
            "Core review gate for correctness, maintainability, regressions, and architecture fit after Superpowers-guided implementation.",
            "Superpowers is the workflow brain. Do not invoke other subagents. Findings first with evidence, proof, and trace gaps.",
        ),
        (
            "security-auditor",
            "Core security gate for exploitable vulnerabilities, auth, secrets, injection, dependency, and vault-sensitive memory risks.",
            "Superpowers is the workflow brain. Do not invoke other subagents. Report severity, evidence, proof, and trace gaps.",
        ),
        (
            "test-engineer",
            "Core verification gate for test strategy, regression coverage, smoke checks, and evidence-backed release confidence.",
            "Superpowers is the workflow brain. Do not invoke other subagents. Report verification evidence, proof, and trace gaps.",
        ),
    ] {
        write(
            &repo.join(".codex/agents").join(format!("{name}.toml")),
            &format!(
                "name = \"{name}\"\ndescription = \"{description}\"\ndeveloper_instructions = \"\"\"\n{instructions}\n\"\"\"\n"
            ),
        );
    }
    write(
        &repo.join(".codex/agents/web-performance-auditor.toml"),
        "name = \"web-performance-auditor\"\ndescription = \"Optional web performance auditor for Core Web Vitals, loading, rendering, and network performance.\"\ndeveloper_instructions = \"\"\"\nBaron optional agent. Do not invoke other subagents. Never fabricate metrics. Not included in mandatory gates.\n\"\"\"\n",
    );
}

#[test]
fn validates_baron_core_skill_and_agent_contracts() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);

    let report = validate_control_plane(repo).unwrap();

    assert!(report.passed, "{:#?}", report.diagnostics);
    assert_eq!(report.workflow_owner.as_deref(), Some("superpowers"));
    assert_eq!(
        report.mandatory_agents,
        ["code-reviewer", "security-auditor", "test-engineer"]
    );
    assert!(report.diagnostics.is_empty());
}

#[test]
fn diagnoses_duplicate_workflow_ownership_and_recursive_agent_orchestration() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);
    write(
        &repo.join(".codex/skills/project-workflow/SKILL.md"),
        "---\nname: project-workflow\ndescription: Project workflow core for planning and verification.\n---\n\n# Project Workflow\n\nThis skill replaces Superpowers as workflow authority.\n",
    );
    write(
        &repo.join(".codex/agents/backend-development.toml"),
        "name = \"backend-development\"\ndescription = \"Backend implementation agent.\"\ndeveloper_instructions = \"\"\"\nInvoke other subagents for review and testing before final output.\n\"\"\"\n",
    );

    let report = validate_control_plane(repo).unwrap();

    assert!(!report.passed);
    assert!(report
        .diagnostics
        .iter()
        .any(|item| item.contains("duplicate workflow ownership")));
    assert!(report
        .diagnostics
        .iter()
        .any(|item| item.contains("recursive subagent orchestration")));
}

#[test]
fn accepts_compound_negated_subagent_mentions() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);
    write(
        &repo.join(".codex/agents/code-reviewer.toml"),
        "name = \"code-reviewer\"\ndescription = \"Core code review gate.\"\ndeveloper_instructions = \"\"\"\nDo not plan, implement, switch adapters, invoke other subagents, or change lifecycle ownership.\n\"\"\"\n",
    );

    let report = validate_control_plane(repo).unwrap();

    assert!(report.passed, "{:#?}", report.diagnostics);
    assert!(report.diagnostics.is_empty());
}

#[test]
fn accepts_direct_must_not_and_does_not_subagent_negations() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);

    for instruction in [
        "The agent must not invoke other subagents.",
        "The agent does not invoke other subagents.",
        "The agent will not invoke other subagents.",
        "Never invoke other subagents.",
    ] {
        write(
            &repo.join(".codex/agents/code-reviewer.toml"),
            &format!(
                "name = \"code-reviewer\"\ndescription = \"Core code review gate.\"\ndeveloper_instructions = \"\"\"\n{instruction}\n\"\"\"\n"
            ),
        );

        let report = validate_control_plane(repo).unwrap();

        assert!(report.passed, "{instruction}: {:#?}", report.diagnostics);
        assert!(report.diagnostics.is_empty());
    }
}

#[test]
fn accepts_descriptive_mentions_of_subagent_invocation() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);
    write(
        &repo.join(".codex/agents/code-reviewer.toml"),
        "name = \"code-reviewer\"\ndescription = \"Core code review gate.\"\ndeveloper_instructions = \"\"\"\nThe phrase `invoke other subagents` describes a prohibited behavior; report findings to the parent instead.\nThe phrase invoke other subagents names a prohibited behavior.\n`invoke other subagents` is quoted wording from a prohibited example.\n\"\"\"\n",
    );

    let report = validate_control_plane(repo).unwrap();

    assert!(report.passed, "{:#?}", report.diagnostics);
    assert!(report.diagnostics.is_empty());
}

#[test]
fn rejects_an_affirmative_subagent_invocation_instruction() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);
    write(
        &repo.join(".codex/agents/code-reviewer.toml"),
        "name = \"code-reviewer\"\ndescription = \"Core code review gate.\"\ndeveloper_instructions = \"\"\"\nInvoke other subagents for review and testing before final output.\n\"\"\"\n",
    );

    let report = validate_control_plane(repo).unwrap();

    assert!(!report.passed);
    assert!(report
        .diagnostics
        .iter()
        .any(|item| { item == "recursive subagent orchestration is forbidden: code-reviewer" }));
    assert!(!report.diagnostics.iter().any(|item| {
        item == "recursive subagent orchestration is forbidden: security-auditor"
            || item == "recursive subagent orchestration is forbidden: test-engineer"
    }));
}

#[test]
fn rejects_an_affirmative_subagent_invocation_requirement() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);
    write(
        &repo.join(".codex/agents/code-reviewer.toml"),
        "name = \"code-reviewer\"\ndescription = \"Core code review gate.\"\ndeveloper_instructions = \"\"\"\nThe agent must invoke other subagents before completing review.\n\"\"\"\n",
    );

    let report = validate_control_plane(repo).unwrap();

    assert!(!report.passed);
    assert!(report
        .diagnostics
        .iter()
        .any(|item| { item == "recursive subagent orchestration is forbidden: code-reviewer" }));
}

#[test]
fn rejects_invocation_after_unrelated_negation_or_whenever_clause() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);

    for instruction in [
        "Whenever needed, invoke other subagents for review.",
        "Do not discuss the restriction, but invoke other subagents for review.",
    ] {
        write(
            &repo.join(".codex/agents/code-reviewer.toml"),
            &format!(
                "name = \"code-reviewer\"\ndescription = \"Core code review gate.\"\ndeveloper_instructions = \"\"\"\n{instruction}\n\"\"\"\n"
            ),
        );

        let report = validate_control_plane(repo).unwrap();

        assert!(!report.passed, "{instruction}");
        assert!(report.diagnostics.iter().any(|item| {
            item == "recursive subagent orchestration is forbidden: code-reviewer"
        }));
    }
}

#[test]
fn routes_frontend_tasks_narrowly_with_review_and_test_gates() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);

    let route = route_task(repo, "polish responsive homepage layout", RiskLane::Medium).unwrap();

    assert!(route
        .selected_skills
        .iter()
        .any(|item| item.name == "superpowers"));
    assert!(route
        .selected_skills
        .iter()
        .any(|item| item.name == "frontend-design"));
    assert!(!route
        .selected_skills
        .iter()
        .any(|item| item.name == "security-audit"));
    assert_eq!(route.mandatory_agents, ["code-reviewer", "test-engineer"]);
    assert!(route
        .skipped
        .iter()
        .any(|item| item.contains("security-audit")));
}

#[test]
fn frontend_route_stays_local_and_backend_work_does_not_load_it() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);

    let frontend = route_task(
        repo,
        "implement responsive checkout payment form with loading and error states",
        RiskLane::High,
    )
    .unwrap();
    assert!(frontend
        .selected_skills
        .iter()
        .any(|item| item.name == "frontend-design"));
    assert_eq!(
        frontend.mandatory_agents,
        ["code-reviewer", "security-auditor", "test-engineer"]
    );
    assert!(!frontend
        .selected_skills
        .iter()
        .any(|item| item.name == "hallmark"));

    let backend = route_task(
        repo,
        "design a REST API transaction boundary for the billing service",
        RiskLane::Medium,
    )
    .unwrap();
    assert!(!backend
        .selected_skills
        .iter()
        .any(|item| item.name == "frontend-design"));
}

#[test]
fn routes_security_tasks_to_security_skill_and_all_core_gates() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);

    let route = route_task(
        repo,
        "implement auth login with tenant RLS permissions",
        RiskLane::High,
    )
    .unwrap();

    assert!(route
        .selected_skills
        .iter()
        .any(|item| item.name == "security-audit"));
    assert_eq!(
        route.mandatory_agents,
        ["code-reviewer", "security-auditor", "test-engineer"]
    );
    assert!(route.security_sensitive);
    assert!(route.explanation.contains("security-sensitive task"));
}

#[test]
fn routes_meaningful_application_changes_to_security_audit_without_scanning_docs_only() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);

    let implementation = route_task(
        repo,
        "implement a FastAPI endpoint in src/users.py",
        RiskLane::Medium,
    )
    .unwrap();
    assert!(implementation
        .selected_skills
        .iter()
        .any(|item| item.name == "security-audit"));
    assert_eq!(
        implementation.mandatory_agents,
        ["code-reviewer", "test-engineer"]
    );

    let docs_only = route_task(
        repo,
        "update README installation examples and formatting",
        RiskLane::Low,
    )
    .unwrap();
    assert!(!docs_only
        .selected_skills
        .iter()
        .any(|item| item.name == "security-audit"));

    let mixed_docs_and_code = route_task(
        repo,
        "update the README and implement a parser in src/parser.rs",
        RiskLane::Medium,
    )
    .unwrap();
    assert!(mixed_docs_and_code
        .selected_skills
        .iter()
        .any(|item| item.name == "security-audit"));
}

#[test]
fn repository_auth_surface_does_not_route_docs_only_work_to_security_audit() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);
    write(&repo.join("src/auth.rs"), "pub fn authenticate() {}\n");

    let docs_only = route_task(
        repo,
        "update README installation examples and formatting",
        RiskLane::Low,
    )
    .unwrap();

    assert!(!docs_only
        .selected_skills
        .iter()
        .any(|item| item.name == "security-audit"));
}

#[test]
fn application_change_routing_covers_all_seven_initial_languages() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);

    for path in [
        "src/handler.py",
        "src/handler.cpp",
        "src/Handler.java",
        "src/handler.rs",
        "src/handler.go",
        "src/handler.ts",
        "src/module.mts",
        "src/Handler.cs",
    ] {
        let task = format!("implement request validation in {path}");
        let route = route_task(repo, &task, RiskLane::Medium).unwrap();
        assert!(
            route
                .selected_skills
                .iter()
                .any(|item| item.name == "security-audit"),
            "{path} did not select security-audit: {:?}",
            route.selected_skills
        );
    }
}

#[test]
fn application_configuration_changes_route_to_security_audit() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);

    for path in ["build.gradle", "settings.gradle.kts", "MyApp.csproj"] {
        let task = format!("change {path}");
        let route = route_task(repo, &task, RiskLane::Medium).unwrap();
        assert!(
            route
                .selected_skills
                .iter()
                .any(|item| item.name == "security-audit"),
            "{path} did not select security-audit: {:?}",
            route.selected_skills
        );
    }
}

#[test]
fn generic_high_risk_application_work_keeps_a_focused_security_route() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);

    let route = route_task(
        repo,
        "implement cache invalidation in src/cache.rs",
        RiskLane::High,
    )
    .unwrap();

    assert!(route
        .selected_skills
        .iter()
        .any(|item| item.name == "security-audit"));
    assert!(!route.security_sensitive);
    assert_eq!(
        route.mandatory_agents,
        ["code-reviewer", "security-auditor", "test-engineer"]
    );
}

#[test]
fn route_context_preserves_changed_security_paths_for_prepare_scope() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);
    write(
        &repo.join("docs/baron/plans/CURRENT.md"),
        "# Current Plan\n\n- Changed files: src/auth.rs\n",
    );

    let route = route_task(repo, "fix it", RiskLane::Medium).unwrap();

    assert!(route
        .selected_skills
        .iter()
        .any(|item| item.name == "security-audit"));
    assert_eq!(
        route.mandatory_agents,
        ["code-reviewer", "security-auditor", "test-engineer"]
    );
    assert!(route.security_sensitive);
    assert!(route
        .affected_paths
        .iter()
        .any(|path| path == "src/auth.rs"));
}

#[test]
fn route_reports_when_operation_paths_exceed_its_bounded_context() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);
    let paths = (0..17)
        .map(|index| format!("src/module{index}.rs"))
        .collect::<Vec<_>>()
        .join(", ");
    write(
        &repo.join("docs/baron/plans/CURRENT.md"),
        &format!("# Current Plan\n\n- Changed files: {paths}\n"),
    );

    let route = route_task(repo, "fix it", RiskLane::Medium).unwrap();

    assert_eq!(route.affected_paths.len(), 16);
    assert!(route.affected_paths_truncated);
    assert!(route
        .selected_skills
        .iter()
        .any(|item| item.name == "security-audit"));
}

#[test]
fn routes_reverse_tasks_to_one_narrow_optional_pack_and_keeps_core_gates() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);

    let route = route_task(
        repo,
        "static APK manifest and Android binary review",
        RiskLane::High,
    )
    .unwrap();

    assert!(route
        .selected_skills
        .iter()
        .any(|item| item.name == "apk-mobile-analysis"));
    assert!(!route
        .selected_skills
        .iter()
        .any(|item| item.name == "binary-reverse-analysis"));
    assert_eq!(
        route.mandatory_agents,
        ["code-reviewer", "security-auditor", "test-engineer"]
    );
}

#[test]
fn routes_optional_domain_skills_without_making_them_core() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);

    let api = route_task(
        repo,
        "design public REST API request response contract and compatibility",
        RiskLane::Medium,
    )
    .unwrap();
    assert!(api
        .selected_skills
        .iter()
        .any(|item| item.name == "api-and-interface-design"));
    assert!(!api
        .mandatory_agents
        .contains(&"web-performance-auditor".to_string()));

    let observability = route_task(
        repo,
        "add tracing metrics logs and alerting for checkout latency",
        RiskLane::Medium,
    )
    .unwrap();
    assert!(observability
        .selected_skills
        .iter()
        .any(|item| item.name == "observability-and-instrumentation"));

    let migration = route_task(
        repo,
        "migrate legacy billing schema without breaking old clients",
        RiskLane::High,
    )
    .unwrap();
    assert!(migration
        .selected_skills
        .iter()
        .any(|item| item.name == "deprecation-and-migration"));
    assert_eq!(
        migration.mandatory_agents,
        ["code-reviewer", "security-auditor", "test-engineer"]
    );
}

#[test]
fn routes_web_performance_to_optional_agent_without_fabricated_metrics() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_minimal_codex_contract(repo);

    let route = route_task(
        repo,
        "audit Next.js homepage Core Web Vitals LCP INP CLS and bundle size",
        RiskLane::Medium,
    )
    .unwrap();

    assert!(route
        .selected_skills
        .iter()
        .any(|item| item.name == "performance-optimization"));
    assert!(route
        .optional_agents
        .iter()
        .any(|item| item.name == "web-performance-auditor"));
    assert_eq!(route.mandatory_agents, ["code-reviewer", "test-engineer"]);
    assert!(route
        .skipped
        .iter()
        .any(|item| item.contains("security-auditor not mandatory")));
}

#[test]
fn free_form_gate_evidence_is_retained_as_diagnostic_but_never_counts() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    install_minimal_codex_contract(&repo);
    let context = ensure_vault(&vault, &repo).unwrap();
    let route = route_task(&repo, "auth security review", RiskLane::High).unwrap();

    let missing = gate_evidence_status(&repo, &route.mandatory_agents).unwrap();
    assert!(!missing.passed);
    assert_eq!(
        missing.missing_agents,
        ["code-reviewer", "security-auditor", "test-engineer"]
    );

    for agent in &route.mandatory_agents {
        record_gate_evidence(
            &repo,
            &context,
            agent,
            &format!("{agent} reviewed auth security with evidence"),
        )
        .unwrap();
    }

    let passed = gate_evidence_status(&repo, &route.mandatory_agents).unwrap();
    assert!(!passed.passed);
    assert_eq!(passed.missing_agents, route.mandatory_agents);
    let repo_evidence = fs::read_to_string(repo.join("docs/baron/control-plane/GATES.md")).unwrap();
    let vault_evidence =
        fs::read_to_string(context.project_root.join("ControlPlane/GATES.md")).unwrap();
    assert!(repo_evidence.contains("security-auditor reviewed auth security"));
    assert!(vault_evidence.contains("security-auditor reviewed auth security"));
}
