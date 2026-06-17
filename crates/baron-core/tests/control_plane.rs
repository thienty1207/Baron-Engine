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
| `vibe-security-scan` | optional domain | auth, API, secrets, RLS, uploads, payment, dependencies, permissions |\n",
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
        &repo.join(".codex/skills/vibe-security-scan/SKILL.md"),
        "---\nname: vibe-security-scan\ndescription: Use when reviewing auth, API, secrets, RLS, uploads, payment, dependency, CORS, JWT, rate limit, or access control security risks.\n---\n\n# Vibe Security Scan\n\nOptional defensive security domain skill. Superpowers remains the workflow authority. Output must include severity, evidence, fix, and verification.\n",
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
        .any(|item| item.name == "vibe-security-scan"));
    assert_eq!(route.mandatory_agents, ["code-reviewer", "test-engineer"]);
    assert!(route
        .skipped
        .iter()
        .any(|item| item.contains("vibe-security-scan")));
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
        .any(|item| item.name == "vibe-security-scan"));
    assert_eq!(
        route.mandatory_agents,
        ["code-reviewer", "security-auditor", "test-engineer"]
    );
    assert!(route.explanation.contains("security-sensitive task"));
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
fn mandatory_gate_evidence_must_be_recorded_before_it_counts() {
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
    assert!(passed.passed);
    assert!(passed.missing_agents.is_empty());
    let repo_evidence = fs::read_to_string(repo.join("docs/baron/control-plane/GATES.md")).unwrap();
    let vault_evidence =
        fs::read_to_string(context.project_root.join("ControlPlane/GATES.md")).unwrap();
    assert!(repo_evidence.contains("security-auditor reviewed auth security"));
    assert!(vault_evidence.contains("security-auditor reviewed auth security"));
}
