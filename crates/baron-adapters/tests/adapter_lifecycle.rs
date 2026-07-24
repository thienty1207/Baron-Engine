use std::fs;
use std::path::{Path, PathBuf};

use baron_adapters::{install_adapter, AgentAdapter};
use sha2::{Digest, Sha256};
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn collect_text_files(root: &Path, files: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }

    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            collect_text_files(&path, files);
        } else {
            files.push(path);
        }
    }
}

#[test]
fn assets_core_is_the_only_bundled_runtime_source() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let assets = workspace_root.join("assets/core");

    for required in [
        "skills/superpowers/SKILL.md",
        "agents/code-reviewer.toml",
        "agents/security-auditor.toml",
        "agents/test-engineer.toml",
    ] {
        assert!(
            assets.join(required).is_file(),
            "missing runtime asset {required}"
        );
    }

    assert!(
        !workspace_root.join("blueprints/core").exists(),
        "stale duplicate runtime source exists: blueprints/core"
    );

    let mut runtime_files = Vec::new();
    for relative in [
        "crates/baron-adapters/src",
        "crates/baron-core/src",
        "crates/baron-cli/src",
        "installers",
        ".github",
    ] {
        collect_text_files(&workspace_root.join(relative), &mut runtime_files);
    }
    for manifest in ["Cargo.toml", "Cargo.lock"] {
        runtime_files.push(workspace_root.join(manifest));
    }

    let offenders = runtime_files
        .into_iter()
        .filter_map(|path| {
            fs::read_to_string(&path).ok().and_then(|content| {
                (content.contains("blueprints/core") || content.contains("blueprints\\\\core"))
                    .then(|| {
                        path.strip_prefix(&workspace_root)
                            .unwrap()
                            .display()
                            .to_string()
                    })
            })
        })
        .collect::<Vec<_>>();
    assert!(
        offenders.is_empty(),
        "runtime code must not read stale blueprints: {}",
        offenders.join(", ")
    );

    let temp = tempdir().unwrap();
    let repo = temp.path();
    for adapter in [
        AgentAdapter::Codex,
        AgentAdapter::Claude,
        AgentAdapter::Generic,
    ] {
        install_adapter(repo, adapter).unwrap();
    }
    for root in [
        repo.join(".codex"),
        repo.join(".claude"),
        repo.join(".baron/core"),
    ] {
        assert!(root.join("skills/superpowers/SKILL.md").is_file());
        assert!(
            root.join("agents/code-reviewer.toml").is_file()
                || root.join("agents/code-reviewer.md").is_file()
        );
        assert!(
            root.join("agents/security-auditor.toml").is_file()
                || root.join("agents/security-auditor.md").is_file()
        );
        assert!(
            root.join("agents/test-engineer.toml").is_file()
                || root.join("agents/test-engineer.md").is_file()
        );
    }
}

fn superpowers_upstream_tree_digest(root: &Path) -> (usize, String) {
    let excluded = [
        "LICENSE.txt",
        "NOTICE.md",
        "README.md",
        "README.upstream.md",
        "SKILL.md",
        "UPSTREAM.json",
    ];
    let mut stack = vec![root.to_path_buf()];
    let mut entries = Vec::new();
    while let Some(directory) = stack.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                stack.push(entry.path());
                continue;
            }
            let relative = entry
                .path()
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if excluded.contains(&relative.as_str()) {
                continue;
            }
            let content = fs::read(entry.path()).unwrap();
            let normalized = canonicalize_line_endings(&content);
            let file_hash = Sha256::digest(normalized)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            entries.push(format!("{relative}\0{file_hash}"));
        }
    }
    entries.sort();
    let digest = Sha256::digest(entries.join("\n").as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    (entries.len(), digest)
}

fn canonicalize_line_endings(content: &[u8]) -> Vec<u8> {
    let mut normalized = Vec::with_capacity(content.len());
    let mut index = 0;
    while index < content.len() {
        if content[index] == b'\r' && content.get(index + 1) == Some(&b'\n') {
            normalized.push(b'\n');
            index += 2;
        } else {
            normalized.push(content[index]);
            index += 1;
        }
    }
    normalized
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
fn superpowers_core_is_pinned_to_v6_2_and_installs_the_complete_workflow() {
    let temp = tempdir().unwrap();
    let repo = temp.path();

    install_adapter(repo, AgentAdapter::Codex).unwrap();
    install_adapter(repo, AgentAdapter::Claude).unwrap();
    install_adapter(repo, AgentAdapter::Generic).unwrap();
    for root in [
        repo.join(".codex/skills/superpowers"),
        repo.join(".claude/skills/superpowers"),
        repo.join(".baron/core/skills/superpowers"),
    ] {
        let provenance: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join("UPSTREAM.json")).unwrap()).unwrap();
        assert_eq!(provenance["repository"], "obra/superpowers");
        assert_eq!(provenance["version"], "6.2.0");
        assert_eq!(
            provenance["commit"],
            "3dcbd5c4b48e02263fbf4a3c01e3fe4f81d584d9"
        );
        assert_eq!(
            provenance["upstreamTreeSha256"],
            "e76b5a985b27d626b7338028e292ecd944664ef5d85981c6fd05a8ae71b9291a"
        );
        assert_eq!(provenance["vendoredTreeHashNormalization"], "LF");
        assert_eq!(
            provenance["baronPatches"][0]["path"],
            "brainstorming/scripts/server.cjs"
        );
        let (file_count, tree_digest) = superpowers_upstream_tree_digest(&root);
        assert_eq!(
            file_count,
            provenance["vendoredFileCount"].as_u64().unwrap() as usize
        );
        assert_eq!(
            tree_digest,
            provenance["vendoredTreeSha256"].as_str().unwrap(),
            "vendored Superpowers tree changed at {}",
            root.display()
        );
        let baron_contract = fs::read_to_string(root.join("SKILL.md")).unwrap();
        assert!(baron_contract.contains("plan-scoped SDD workspace"));
        assert!(baron_contract.contains("same basename"));
        assert!(baron_contract.contains("Rounds 1-3 resume the original implementer"));
        assert!(baron_contract.contains("Rounds 4-5 use a fresh implementer"));
        assert!(baron_contract.contains("After round 5"));
        assert!(baron_contract.contains("Text presence alone is not test proof"));

        for path in [
            "LICENSE.txt",
            "NOTICE.md",
            "subagent-driven-development/task-reviewer-prompt.md",
            "subagent-driven-development/re-review-prompt.md",
            "subagent-driven-development/scripts/sdd-workspace",
            "subagent-driven-development/scripts/task-brief",
            "subagent-driven-development/scripts/review-package",
            "test-driven-development/writing-good-tests.md",
            "using-superpowers/references/antigravity-tools.md",
            "using-superpowers/references/gemini-tools.md",
            "using-superpowers/references/pi-tools.md",
        ] {
            assert!(
                root.join(path).is_file(),
                "missing Superpowers 6.2 asset {path}"
            );
        }

        for obsolete in [
            "subagent-driven-development/code-quality-reviewer-prompt.md",
            "subagent-driven-development/spec-reviewer-prompt.md",
            "test-driven-development/testing-anti-patterns.md",
            "using-superpowers/references/copilot-tools.md",
        ] {
            assert!(
                !root.join(obsolete).exists(),
                "obsolete Superpowers asset survived refresh: {obsolete}"
            );
        }

        let visual_server =
            fs::read_to_string(root.join("brainstorming/scripts/server.cjs")).unwrap();
        assert!(visual_server.contains("Baron Superpowers v"));
        assert!(visual_server.contains("UPSTREAM.json"));
        assert!(!visual_server.contains("https://"));
        assert!(!visual_server.contains("TELEMETRY"));
    }

    let root = repo.join(".codex/skills/superpowers");
    let sdd = fs::read_to_string(root.join("subagent-driven-development/SKILL.md")).unwrap();
    assert!(sdd.contains(".superpowers/sdd/<plan-basename>/"));
    assert!(sdd.contains("resume the original implementer"));
    assert!(sdd.contains("Five rounds maximum per task"));
    assert!(sdd.contains("scoped re-review"));

    let good_tests =
        fs::read_to_string(root.join("test-driven-development/writing-good-tests.md")).unwrap();
    assert!(good_tests.contains("Behavior, not text"));
    assert!(good_tests.contains("production change"));
    assert!(good_tests.contains("The Mutation Check"));
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
fn adapter_update_rejects_malformed_managed_markers_without_overwriting_user_content() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_adapter(repo, AgentAdapter::Codex).unwrap();
    let malformed = "# User Header\n\n<!-- BARON:MANAGED:START -->\npartial Baron content\n";
    fs::write(repo.join("AGENTS.md"), malformed).unwrap();

    let error = install_adapter(repo, AgentAdapter::Codex).unwrap_err();

    assert!(error.to_string().contains("malformed"));
    assert_eq!(
        fs::read_to_string(repo.join("AGENTS.md")).unwrap(),
        malformed
    );
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
fn baron_owned_runtime_assets_are_self_contained_and_deep() {
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
        let content =
            fs::read_to_string(repo.join(".codex/skills").join(skill).join("SKILL.md")).unwrap();
        let lower = content.to_lowercase();
        assert!(
            !lower.contains("http://")
                && !lower.contains("https://")
                && !lower.contains("github.com")
                && !lower.contains("public source"),
            "{skill} runtime guidance must not depend on live external links"
        );
        for required in [
            "## Baron Contract",
            "## Use When",
            "## Output Contract",
            "## Verification",
        ] {
            assert!(content.contains(required), "{skill} missing {required}");
        }
        for required in ["Superpowers", "proof", "trace", "unknown", "evidence"] {
            assert!(
                lower.contains(&required.to_lowercase()),
                "{skill} missing {required}"
            );
        }
        assert!(
            content.lines().count() >= 80,
            "{skill} is too thin to be a Baron-owned runtime skill"
        );
    }

    let skill_expectations = [
        (
            "api-and-interface-design",
            [
                "versioning",
                "compatibility",
                "pagination",
                "idempotency",
                "error schema",
                "auth boundary",
            ],
        ),
        (
            "observability-and-instrumentation",
            ["correlation id", "metrics", "traces", "slo", "alert", "pii"],
        ),
        (
            "deprecation-and-migration",
            [
                "rollback",
                "feature flag",
                "compatibility window",
                "dual-read",
                "dual-write",
                "data migration",
            ],
        ),
        (
            "vibe-security-scan",
            [
                "trust boundary",
                "data-flow",
                "idor",
                "ssrf",
                "command injection",
                "no weaponized",
            ],
        ),
    ];
    for (skill, terms) in skill_expectations {
        let content = fs::read_to_string(repo.join(".codex/skills").join(skill).join("SKILL.md"))
            .unwrap()
            .to_lowercase();
        for term in terms {
            assert!(content.contains(term), "{skill} missing {term}");
        }
    }
}

#[test]
fn bundled_agents_are_self_contained_and_deep_quality_gates() {
    let temp = tempdir().unwrap();
    let repo = temp.path();

    install_adapter(repo, AgentAdapter::Codex).unwrap();

    for agent in [
        "code-reviewer",
        "security-auditor",
        "test-engineer",
        "web-performance-auditor",
    ] {
        let content =
            fs::read_to_string(repo.join(".codex/agents").join(format!("{agent}.toml"))).unwrap();
        let lower = content.to_lowercase();
        assert!(
            !lower.contains("http://") && !lower.contains("https://") && !lower.contains("github"),
            "{agent} runtime instructions must not depend on live external links"
        );
        for required in [
            "core contract",
            "scope",
            "anti-hallucination",
            "output contract",
            "evidence",
            "proof",
            "trace",
            "do not invoke other subagents",
        ] {
            assert!(lower.contains(required), "{agent} missing {required}");
        }
        assert!(
            content.lines().count() >= 45,
            "{agent} instructions are too thin for Baron 3.0"
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
        assert!(
            content.contains(&format!("baron runtime check --adapter {adapter}")),
            "{path} must trigger automatic runtime backend policy checks"
        );
        assert!(
            content.contains("baron autopilot status"),
            "{path} must inspect autopilot learning and resume state"
        );
        assert!(content.contains(&format!("baron context --{adapter}")));
        assert!(
            content.contains("presence is not execution evidence"),
            "{path} must prevent false tool-backed completion claims"
        );
        assert!(content.contains("baron proof record"));
        assert!(content.contains("baron autopilot review"));
    }
    let claude_context =
        fs::read_to_string(repo.join(".claude/commands/baron-context.md")).unwrap();
    assert!(claude_context.contains("baron capability check"));
    assert!(claude_context.contains("baron runtime check"));
    assert!(claude_context.contains("baron autopilot status"));
    let generic_context = fs::read_to_string(repo.join("baron-context.json")).unwrap();
    assert!(generic_context
        .contains("\"capabilityCheckCommand\": \"baron capability check --adapter agent\""));
    assert!(generic_context
        .contains("\"runtimeCheckCommand\": \"baron runtime check --adapter agent\""));
    assert!(generic_context.contains("\"autopilotStatusCommand\": \"baron autopilot status\""));
}

#[test]
fn every_adapter_enforces_intent_clarity_and_actionable_recovery() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    fs::create_dir_all(&repo).unwrap();

    install_adapter(&repo, AgentAdapter::Codex).unwrap();
    install_adapter(&repo, AgentAdapter::Claude).unwrap();
    install_adapter(&repo, AgentAdapter::Generic).unwrap();

    for path in ["AGENTS.md", "CLAUDE.md", "AGENT.md"] {
        let content = fs::read_to_string(repo.join(path)).unwrap();
        assert!(
            content.contains(
                "read repo, Vault, current plan, Harness, continuity, and decisions before asking"
            ),
            "{path} must require evidence-first intent discovery"
        );
        assert!(
            content.contains("ask exactly one missing high-value question at a time"),
            "{path} must keep clarification bounded"
        );
        assert!(content.contains("baron harness intent"));
        assert!(
            content.contains("do not pass `--confirmed` until the user explicitly confirms"),
            "{path} must not fabricate confirmation"
        );
        assert!(content.contains("baron harness intent-status"));
        assert!(content.contains("baron continuity recover"));
        assert!(
            content.contains("preserve the failed attempt"),
            "{path} must preserve recovery evidence"
        );
    }
}

#[test]
fn every_adapter_automates_platform_architecture_and_review_closure() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    fs::create_dir_all(&repo).unwrap();
    for adapter in [
        AgentAdapter::Codex,
        AgentAdapter::Claude,
        AgentAdapter::Generic,
    ] {
        install_adapter(&repo, adapter).unwrap();
    }

    for path in ["AGENTS.md", "CLAUDE.md", "AGENT.md"] {
        let content = fs::read_to_string(repo.join(path)).unwrap();
        assert!(
            content.contains("docs/baron/platform/PROJECT_PROFILE.md"),
            "{path}"
        );
        assert!(
            content.contains("docs/baron/architecture/CURRENT_ARCHITECTURE.md"),
            "{path}"
        );
        assert!(content.contains("baron init --<platform>"), "{path}");
        assert!(content.contains("baron review finding"), "{path}");
        assert!(content.contains("baron review close"), "{path}");
        assert!(content.contains("fix evidence and verification"), "{path}");
    }

    let frontend = fs::read_to_string(repo.join(".codex/skills/frontend-design/SKILL.md")).unwrap();
    assert!(frontend.contains("Baron Design Quality Gate"));
    assert!(frontend.contains("overflow and clipping"));
    assert!(frontend.contains("narrow and a wide viewport"));
}

#[test]
fn every_adapter_classifies_request_authority_before_durable_writes() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    fs::create_dir_all(&repo).unwrap();
    for adapter in [
        AgentAdapter::Codex,
        AgentAdapter::Claude,
        AgentAdapter::Generic,
    ] {
        install_adapter(&repo, adapter).unwrap();
    }

    for path in ["AGENTS.md", "CLAUDE.md", "AGENT.md"] {
        let content = fs::read_to_string(repo.join(path)).unwrap();
        assert!(
            content.contains("baron authority classify"),
            "{path} must classify request authority before automation"
        );
        assert!(content.contains("read_only"), "{path}");
        assert!(content.contains("ambiguous"), "{path}");
        assert!(
            content.contains("do not create or update plan, Harness, proof, trace, review, friction, or learning state"),
            "{path} must keep inspection requests mutation-free"
        );
        assert!(
            content.contains("review and apply fixes"),
            "{path} must classify by requested outcome, not one keyword"
        );
        assert!(
            content.contains("run `baron automation reconcile`"),
            "{path}"
        );
        assert!(
            content.contains("Never run public `baron update`"),
            "{path}"
        );
        assert!(
            content.contains("never repair Baron metadata by hand"),
            "{path}"
        );
    }
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
