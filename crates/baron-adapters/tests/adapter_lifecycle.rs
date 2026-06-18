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
    for skill in [
        "api-and-interface-design",
        "observability-and-instrumentation",
        "performance-optimization",
        "deprecation-and-migration",
    ] {
        assert!(repo
            .join(".codex/skills")
            .join(skill)
            .join("SKILL.md")
            .exists());
    }
    assert!(repo.join(".codex/agents/code-reviewer.toml").exists());
    assert!(repo.join(".codex/agents/security-auditor.toml").exists());
    assert!(repo.join(".codex/agents/test-engineer.toml").exists());
    assert!(repo
        .join(".codex/agents/web-performance-auditor.toml")
        .exists());
    let hooks: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(repo.join(".codex/hooks.json")).unwrap()).unwrap();
    assert!(hooks["hooks"]["SessionStart"]
        .to_string()
        .contains("baron automation hook session-start"));
    assert!(hooks["hooks"]["Stop"]
        .to_string()
        .contains("baron automation hook stop"));
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
    let hooks: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(repo.join(".claude/settings.json")).unwrap())
            .unwrap();
    assert!(hooks["hooks"]["SessionStart"]
        .to_string()
        .contains("baron automation hook session-start"));
    assert!(hooks["hooks"]["Stop"]
        .to_string()
        .contains("baron automation hook stop"));
}

#[test]
fn repeated_adapter_updates_preserve_custom_native_hooks() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    write(
        &repo.join(".codex/hooks.json"),
        r#"{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "custom-start-command"
          }
        ]
      }
    ]
  }
}
"#,
    );

    install_adapter(repo, AgentAdapter::Codex).unwrap();
    install_adapter(repo, AgentAdapter::Codex).unwrap();

    let content = fs::read_to_string(repo.join(".codex/hooks.json")).unwrap();
    assert!(content.contains("custom-start-command"));
    let hooks: serde_json::Value = serde_json::from_str(&content).unwrap();
    let baron_groups = hooks["hooks"]["SessionStart"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| {
            entry
                .to_string()
                .contains("baron automation hook session-start")
        })
        .count();
    assert_eq!(baron_groups, 1);
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
fn update_preserves_custom_skill_and_agent_routing_entries() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    write(
        &repo.join(".codex/skills/INDEX.md"),
        "# Existing Routing\n\n## Custom Skills\n\n- `rust-api`: use for Axum backend work.\n",
    );
    write(
        &repo.join(".codex/agents/INDEX.md"),
        "# Existing Agent Routing\n\n## Custom Agents\n\n- `backend-development`: owns Rust API implementation.\n",
    );

    install_adapter(repo, AgentAdapter::Codex).unwrap();
    install_adapter(repo, AgentAdapter::Codex).unwrap();

    let skills = fs::read_to_string(repo.join(".codex/skills/INDEX.md")).unwrap();
    let agents = fs::read_to_string(repo.join(".codex/agents/INDEX.md")).unwrap();
    assert!(skills.contains("- `rust-api`: use for Axum backend work."));
    assert!(agents.contains("- `backend-development`: owns Rust API implementation."));
    assert_eq!(skills.matches("BARON:ROUTING:START").count(), 1);
    assert_eq!(agents.matches("BARON:ROUTING:START").count(), 1);
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
    assert!(skills.contains("api-and-interface-design"));
    assert!(skills.contains("observability-and-instrumentation"));
    assert!(skills.contains("performance-optimization"));
    assert!(skills.contains("deprecation-and-migration"));
    assert!(skills.contains("Do not recursively load"));
    let agents = fs::read_to_string(repo.join(".codex/agents/INDEX.md")).unwrap();
    assert!(agents.contains("code-reviewer"));
    assert!(agents.contains("security-auditor"));
    assert!(agents.contains("test-engineer"));
    assert!(agents.contains("web-performance-auditor"));
    assert!(agents.contains("optional web performance"));
    assert!(agents.contains("not included in mandatory gates"));
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
        assert!(lower.contains("findings"));
        assert!(lower.contains("verification"));
        assert!(!lower.contains("agent-bootstrap"));
        assert!(!lower.contains("agent bootstrap"));
    }
}

#[test]
fn optional_web_performance_agent_is_not_a_core_quality_gate() {
    let temp = tempdir().unwrap();
    let repo = temp.path();

    install_adapter(repo, AgentAdapter::Codex).unwrap();

    let content =
        fs::read_to_string(repo.join(".codex/agents/web-performance-auditor.toml")).unwrap();
    let lower = content.to_lowercase();
    assert!(content.contains("Baron"));
    assert!(lower.contains("optional"));
    assert!(lower.contains("core web vitals"));
    assert!(lower.contains("never fabricate metrics"));
    assert!(lower.contains("not included in mandatory gates"));
    assert!(lower.contains("do not invoke other subagents"));
}

#[test]
fn performance_optimization_skill_is_operationally_detailed() {
    let temp = tempdir().unwrap();
    let repo = temp.path();

    install_adapter(repo, AgentAdapter::Codex).unwrap();

    let content =
        fs::read_to_string(repo.join(".codex/skills/performance-optimization/SKILL.md")).unwrap();
    let lower = content.to_lowercase();

    for required in [
        "measure",
        "identify",
        "fix",
        "verify",
        "guard",
        "lcp",
        "inp",
        "cls",
        "n+1",
        "pagination",
        "bundle",
        "cache",
        "performance budget",
        "before and after",
        "never fabricate metrics",
        "Baron",
    ] {
        assert!(
            lower.contains(&required.to_lowercase()),
            "performance skill missing {required}"
        );
    }
}

#[test]
fn bundled_domain_skills_do_not_depend_on_agent_bootstrap_runtime() {
    let temp = tempdir().unwrap();
    let repo = temp.path();

    install_adapter(repo, AgentAdapter::Codex).unwrap();

    for skill in [
        "frontend-design",
        "vibe-security-scan",
        "api-and-interface-design",
        "observability-and-instrumentation",
        "performance-optimization",
        "deprecation-and-migration",
    ] {
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

#[test]
fn every_adapter_automatically_refreshes_capabilities_without_claiming_execution() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    fs::create_dir_all(&repo).unwrap();

    install_adapter(&repo, AgentAdapter::Codex).unwrap();
    install_adapter(&repo, AgentAdapter::Claude).unwrap();
    install_adapter(&repo, AgentAdapter::Generic).unwrap();

    for (path, adapter) in [
        ("AGENTS.md", "codex"),
        ("CLAUDE.md", "claude"),
        ("AGENT.md", "agent"),
    ] {
        let content = fs::read_to_string(repo.join(path)).unwrap();
        assert!(
            content.contains(&format!("baron capability check --adapter {adapter}")),
            "{path} must trigger automatic capability refresh"
        );
        assert!(content.contains(&format!("baron context --{adapter}")));
        assert!(
            content.contains("presence is not execution evidence"),
            "{path} must prevent false tool-backed completion claims"
        );
        assert!(content.contains("baron proof record"));
    }
    let claude_context =
        fs::read_to_string(repo.join(".claude/commands/baron-context.md")).unwrap();
    assert!(claude_context.contains("baron capability check"));
    let generic_context = fs::read_to_string(repo.join("baron-context.json")).unwrap();
    assert!(generic_context
        .contains("\"capabilityCheckCommand\": \"baron capability check --adapter agent\""));
}

#[test]
fn generated_indexes_define_strict_contract_fields_and_control_plane_startup() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    fs::create_dir_all(&repo).unwrap();

    install_adapter(&repo, AgentAdapter::Codex).unwrap();
    install_adapter(&repo, AgentAdapter::Claude).unwrap();
    install_adapter(&repo, AgentAdapter::Generic).unwrap();

    for path in ["AGENTS.md", "CLAUDE.md", "AGENT.md"] {
        let content = fs::read_to_string(repo.join(path)).unwrap();
        assert!(
            content.contains("baron control-plane route"),
            "{path} must require explainable routing"
        );
        assert!(
            content.contains("baron control-plane record-gate"),
            "{path} must require quality gate evidence"
        );
    }

    for path in [
        ".codex/skills/INDEX.md",
        ".claude/skills/INDEX.md",
        ".baron/core/skills/INDEX.md",
        ".codex/agents/INDEX.md",
        ".claude/agents/INDEX.md",
        ".baron/core/agents/INDEX.md",
    ] {
        let content = fs::read_to_string(repo.join(path)).unwrap();
        for required in ["Ownership", "Trigger", "Exclusion", "Evidence", "Conflicts"] {
            assert!(content.contains(required), "{path} missing {required}");
        }
        assert!(
            content.contains("baron control-plane"),
            "{path} must point agents to control-plane validation"
        );
    }
}
