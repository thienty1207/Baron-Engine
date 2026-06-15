use std::fs;
use std::path::Path;

use baron_adapters::{install_adapter, AgentAdapter};
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn codex_adapter_installs_core_and_optional_assets() {
    let temp = tempdir().unwrap();
    let repo = temp.path();

    install_adapter(repo, AgentAdapter::Codex).unwrap();

    let agents = fs::read_to_string(repo.join("AGENTS.md")).unwrap();
    assert!(agents.contains("BARON:MANAGED:START"));
    assert!(agents.contains("baron context"));
    assert!(agents.contains("baron trace score"));
    assert!(repo.join(".codex/skills/superpowers/SKILL.md").exists());
    assert!(repo.join(".codex/skills/frontend-design/SKILL.md").exists());
    assert!(repo
        .join(".codex/skills/vibe-security-scan/SKILL.md")
        .exists());
    assert!(repo.join(".codex/agents/code-reviewer.toml").exists());
    assert!(repo.join(".codex/agents/security-auditor.toml").exists());
    assert!(repo.join(".codex/agents/test-engineer.toml").exists());
}

#[test]
fn claude_adapter_installs_same_core_in_claude_shapes() {
    let temp = tempdir().unwrap();
    let repo = temp.path();

    install_adapter(repo, AgentAdapter::Claude).unwrap();

    assert!(fs::read_to_string(repo.join("CLAUDE.md"))
        .unwrap()
        .contains("BARON:MANAGED:START"));
    assert!(repo.join(".claude/commands/baron-context.md").exists());
    assert!(repo.join(".claude/commands/baron-status.md").exists());
    assert!(repo.join(".claude/skills/superpowers/SKILL.md").exists());
    assert!(repo.join(".claude/agents/code-reviewer.md").exists());
    assert!(repo.join(".claude/agents/security-auditor.md").exists());
    assert!(repo.join(".claude/agents/test-engineer.md").exists());
}

#[test]
fn generic_adapter_installs_portable_contract_and_core_assets() {
    let temp = tempdir().unwrap();
    let repo = temp.path();

    install_adapter(repo, AgentAdapter::Generic).unwrap();

    assert!(repo.join("AGENT.md").exists());
    assert!(repo.join("baron-context.md").exists());
    assert!(repo.join("baron-context.json").exists());
    assert!(repo
        .join(".baron/core/skills/superpowers/SKILL.md")
        .exists());
    assert!(repo.join(".baron/core/agents/code-reviewer.toml").exists());
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(repo.join("baron-context.json")).unwrap())
            .unwrap();
    assert_eq!(json["engine"], "baron");
}

#[test]
fn update_preserves_user_text_outside_managed_block() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    write(
        &repo.join("AGENTS.md"),
        "# User Rules\n\nNever delete this.\n",
    );

    install_adapter(repo, AgentAdapter::Codex).unwrap();
    install_adapter(repo, AgentAdapter::Codex).unwrap();

    let content = fs::read_to_string(repo.join("AGENTS.md")).unwrap();
    assert!(content.contains("# User Rules"));
    assert!(content.contains("Never delete this."));
    assert_eq!(content.matches("BARON:MANAGED:START").count(), 1);
}

#[test]
fn update_preserves_custom_skills_and_agents() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    write(
        &repo.join(".codex/skills/rust-api/SKILL.md"),
        "# Custom Rust API\n",
    );
    write(
        &repo.join(".codex/agents/backend-development.toml"),
        "name = \"backend-development\"\n",
    );

    install_adapter(repo, AgentAdapter::Codex).unwrap();
    install_adapter(repo, AgentAdapter::Codex).unwrap();

    assert!(repo.join(".codex/skills/rust-api/SKILL.md").exists());
    assert!(repo.join(".codex/agents/backend-development.toml").exists());
}

#[test]
fn skills_and_agents_indexes_route_narrowly() {
    let temp = tempdir().unwrap();
    let repo = temp.path();

    install_adapter(repo, AgentAdapter::Codex).unwrap();

    let skills = fs::read_to_string(repo.join(".codex/skills/INDEX.md")).unwrap();
    assert!(skills.contains("Superpowers"));
    assert!(skills.contains("frontend-design"));
    assert!(skills.contains("vibe-security-scan"));
    assert!(skills.contains("Do not recursively load"));
    let agents = fs::read_to_string(repo.join(".codex/agents/INDEX.md")).unwrap();
    assert!(agents.contains("code-reviewer"));
    assert!(agents.contains("security-auditor"));
    assert!(agents.contains("test-engineer"));
}

#[test]
fn core_agents_are_baron_native_and_enforce_quality_gates() {
    let temp = tempdir().unwrap();
    let repo = temp.path();

    install_adapter(repo, AgentAdapter::Codex).unwrap();

    for file in [
        "code-reviewer.toml",
        "security-auditor.toml",
        "test-engineer.toml",
    ] {
        let content = fs::read_to_string(repo.join(".codex/agents").join(file)).unwrap();
        let lower = content.to_lowercase();
        assert!(content.contains("Baron"));
        assert!(content.contains("Superpowers"));
        assert!(lower.contains("vault"));
        assert!(lower.contains("evidence"));
        assert!(lower.contains("proof"));
        assert!(lower.contains("trace"));
        assert!(lower.contains("do not invoke other subagents"));
        assert!(!lower.contains("agent-bootstrap"));
        assert!(!lower.contains("agent bootstrap"));
    }
}

#[test]
fn bundled_domain_skills_do_not_depend_on_agent_bootstrap_runtime() {
    let temp = tempdir().unwrap();
    let repo = temp.path();

    install_adapter(repo, AgentAdapter::Codex).unwrap();

    for skill in ["frontend-design", "vibe-security-scan"] {
        let root = repo.join(".codex/skills").join(skill);
        let mut stack = vec![root];
        while let Some(path) = stack.pop() {
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    stack.push(entry.path());
                    continue;
                }
                let content = fs::read_to_string(entry.path()).unwrap();
                let lower = content.to_lowercase();
                assert!(!lower.contains("agent-bootstrap"));
                assert!(!lower.contains("agent bootstrap"));
            }
        }
    }
}
