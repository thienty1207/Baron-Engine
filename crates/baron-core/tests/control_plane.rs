use std::fs;
use std::path::Path;

use baron_core::control_plane::validate_control_plane;
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
