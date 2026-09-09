use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use baron_adapters::{
    install_adapter, load_managed_baseline, managed_payloads_for_adapter, AgentAdapter,
    ManagedMergeKind, ManagedOwner, SupportedManagedAdapter,
};
use tempfile::tempdir;

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, current: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        if !current.exists() {
            return;
        }
        let mut entries = fs::read_dir(current)
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else if path.is_file() && !path.ends_with(".baron-mutation.lock") {
                files.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }

    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn target_claude_payloads_include_native_bridge_and_core_wrappers() {
    let payloads = managed_payloads_for_adapter(AgentAdapter::Claude).unwrap();
    let paths = payloads
        .iter()
        .map(|payload| {
            (
                payload.relative_path.to_string_lossy().replace('\\', "/"),
                payload,
            )
        })
        .collect::<BTreeMap<_, _>>();

    assert!(payloads.iter().all(|payload| payload.adapter == "claude"));
    for path in [
        "CLAUDE.md",
        ".claude/skills/baron-engine/SKILL.md",
        ".claude/agents/code-reviewer.md",
        ".claude/agents/security-auditor.md",
        ".claude/agents/test-engineer.md",
        ".claude/settings.json",
    ] {
        assert!(paths.contains_key(path), "missing Claude payload {path}");
    }
    assert!(!paths.keys().any(|path| path.starts_with(".claude/skills/")
        && path.ends_with("/SKILL.md")
        && path != ".claude/skills/baron-engine/SKILL.md"));
    assert_eq!(
        paths[".claude/skills/baron-engine/SKILL.md"].merge_kind,
        ManagedMergeKind::FullText
    );
    assert_eq!(
        paths[".claude/settings.json"].merge_kind,
        ManagedMergeKind::JsonOwnedEntries
    );

    let bridge = &paths[".claude/skills/baron-engine/SKILL.md"].content;
    assert!(bridge.contains("disable-model-invocation: true"));
    assert!(bridge.contains("control-plane prepare --adapter claude --json"));
    assert!(bridge.contains(".baron/core/skills/<name>"));
    assert!(!bridge.contains("# Baron Skill Routing"));

    for name in ["code-reviewer", "security-auditor", "test-engineer"] {
        let path = format!(".claude/agents/{name}.md");
        let wrapper = &paths[&path].content;
        assert!(wrapper.contains(&format!(".baron/core/agents/{name}.toml")));
        assert!(wrapper.contains("parent"));
        assert!(wrapper.lines().count() < 30, "wrapper {name} is not thin");
        assert_eq!(paths[&path].merge_kind, ManagedMergeKind::FullText);
    }
}

#[test]
fn target_fresh_claude_install_has_one_thin_native_surface() {
    let temp = tempdir().unwrap();
    install_adapter(temp.path(), AgentAdapter::Claude).unwrap();

    for path in [
        "CLAUDE.md",
        ".claude/skills/baron-engine/SKILL.md",
        ".claude/agents/code-reviewer.md",
        ".claude/agents/security-auditor.md",
        ".claude/agents/test-engineer.md",
        ".claude/settings.json",
    ] {
        assert!(temp.path().join(path).is_file(), "missing {path}");
    }
    assert!(temp
        .path()
        .join(".baron/core/skills/superpowers/SKILL.md")
        .is_file());
    assert!(temp
        .path()
        .join(".baron/core/agents/code-reviewer.toml")
        .is_file());
    assert!(!temp
        .path()
        .join(".claude/skills/superpowers/SKILL.md")
        .exists());
    assert!(!temp
        .path()
        .join(".claude/skills/database-engineering/SKILL.md")
        .exists());

    let baseline = load_managed_baseline(temp.path()).unwrap();
    for path in [
        ".claude/skills/baron-engine/SKILL.md",
        ".claude/agents/code-reviewer.md",
        ".claude/agents/security-auditor.md",
        ".claude/agents/test-engineer.md",
    ] {
        let record = baseline
            .records
            .iter()
            .find(|record| record.relative_path == Path::new(path))
            .unwrap_or_else(|| panic!("missing baseline record {path}"));
        assert_eq!(
            record.owner,
            ManagedOwner::Adapter(SupportedManagedAdapter::Claude)
        );
    }
}

#[test]
fn target_claude_contract_is_compact_automatic_and_memory_safe() {
    let temp = tempdir().unwrap();
    install_adapter(temp.path(), AgentAdapter::Claude).unwrap();
    let content = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
    let start = content.find("<!-- BARON:MANAGED:START -->").unwrap();
    let end = content.find("<!-- BARON:MANAGED:END -->").unwrap();
    let managed = &content[start..end];
    assert!(
        managed.len() < 7_000,
        "managed Claude contract is too large"
    );
    let lower = managed.to_lowercase();
    for required in [
        "control-plane prepare --adapter claude --json",
        "preparerequestv1",
        "preparepacketv1",
        "the user's current explicit instructions take precedence",
        "resume",
        "recovery",
        "selected",
        ".baron/core/skills/<skill>",
        "verification",
        "checkpoint",
        "do not ask the user to run hidden baron commands",
        "do not recursively load the full `.baron/core/**` tree",
        "baron capability check --adapter claude",
        "baron control-plane route",
        "baron proof record",
        "baron autopilot review",
        "auto memory",
        "non-authoritative",
        "baron trusted project state",
    ] {
        assert!(
            lower.contains(required),
            "CLAUDE contract missing {required}"
        );
    }
    assert!(!lower.contains("read all .baron/core/skills"));
}

#[test]
fn target_claude_wrappers_are_parent_owned_core_projections() {
    let temp = tempdir().unwrap();
    install_adapter(temp.path(), AgentAdapter::Claude).unwrap();

    for name in ["code-reviewer", "security-auditor", "test-engineer"] {
        let wrapper =
            fs::read_to_string(temp.path().join(format!(".claude/agents/{name}.md"))).unwrap();
        assert!(wrapper.starts_with("---\n"));
        assert!(wrapper.contains("description:"));
        assert!(wrapper.contains(&format!(".baron/core/agents/{name}.toml")));
        assert!(wrapper.contains("parent session"));
        assert!(wrapper.contains("must not"));
        assert!(!wrapper.contains("full semantic policy"));
        assert!(wrapper.lines().count() < 30);
    }
}

#[test]
fn target_claude_projection_preserves_colliding_bridge_agent_and_command() {
    let temp = tempdir().unwrap();
    let bridge = temp.path().join(".claude/skills/baron-engine/SKILL.md");
    let agent = temp.path().join(".claude/agents/code-reviewer.md");
    let command = temp.path().join(".claude/commands/baron-context.md");
    fs::create_dir_all(bridge.parent().unwrap()).unwrap();
    fs::create_dir_all(agent.parent().unwrap()).unwrap();
    fs::create_dir_all(command.parent().unwrap()).unwrap();
    fs::write(&bridge, b"# User bridge\n").unwrap();
    fs::write(&agent, b"---\ndescription: user reviewer\n---\n").unwrap();
    fs::write(&command, b"# User command\n").unwrap();

    let report = install_adapter(temp.path(), AgentAdapter::Claude).unwrap();

    assert_eq!(fs::read(&bridge).unwrap(), b"# User bridge\n");
    assert_eq!(
        fs::read(&agent).unwrap(),
        b"---\ndescription: user reviewer\n---\n"
    );
    assert_eq!(fs::read(&command).unwrap(), b"# User command\n");
    for path in [
        ".claude/skills/baron-engine/SKILL.md",
        ".claude/agents/code-reviewer.md",
        ".claude/commands/baron-context.md",
    ] {
        assert!(
            report.conflicts.iter().any(|value| value == path),
            "missing conflict {path}"
        );
    }
}

#[test]
fn target_claude_install_is_idempotent_and_does_not_mutate_codex() {
    let temp = tempdir().unwrap();
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();
    let codex_before = snapshot(&temp.path().join(".codex"));
    let core_before = snapshot(&temp.path().join(".baron/core"));

    install_adapter(temp.path(), AgentAdapter::Claude).unwrap();
    let first = snapshot(temp.path());
    install_adapter(temp.path(), AgentAdapter::Claude).unwrap();
    let second = snapshot(temp.path());

    assert_eq!(first, second);
    assert_eq!(codex_before, snapshot(&temp.path().join(".codex")));
    assert_eq!(core_before, snapshot(&temp.path().join(".baron/core")));
}

#[test]
fn target_claude_preserves_user_text_skills_agents_commands_settings_and_hooks() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    fs::write(
        repo.join("CLAUDE.md"),
        b"# User instructions\n\nKeep this text.\n",
    )
    .unwrap();
    fs::create_dir_all(repo.join(".claude/skills/personal")).unwrap();
    fs::write(
        repo.join(".claude/skills/personal/SKILL.md"),
        b"# Personal skill\n",
    )
    .unwrap();
    fs::create_dir_all(repo.join(".claude/agents")).unwrap();
    fs::write(
        repo.join(".claude/agents/personal.md"),
        b"---\ndescription: personal agent\n---\n",
    )
    .unwrap();
    fs::create_dir_all(repo.join(".claude/commands")).unwrap();
    fs::write(
        repo.join(".claude/commands/personal.md"),
        b"# Personal command\n",
    )
    .unwrap();
    fs::create_dir_all(repo.join(".claude")).unwrap();
    fs::write(
        repo.join(".claude/settings.json"),
        br#"{
  "autoMemoryEnabled": true,
  "permissions": {"allow": ["Read"]},
  "third_party": {"enabled": true},
  "hooks": {"Stop": [{"hooks": [{"type": "command", "command": "personal-stop"}]}]}
}
"#,
    )
    .unwrap();

    install_adapter(repo, AgentAdapter::Claude).unwrap();

    assert!(fs::read_to_string(repo.join("CLAUDE.md"))
        .unwrap()
        .contains("Keep this text."));
    assert_eq!(
        fs::read(repo.join(".claude/skills/personal/SKILL.md")).unwrap(),
        b"# Personal skill\n"
    );
    assert_eq!(
        fs::read(repo.join(".claude/agents/personal.md")).unwrap(),
        b"---\ndescription: personal agent\n---\n"
    );
    assert_eq!(
        fs::read(repo.join(".claude/commands/personal.md")).unwrap(),
        b"# Personal command\n"
    );
    let settings: serde_json::Value =
        serde_json::from_slice(&fs::read(repo.join(".claude/settings.json")).unwrap()).unwrap();
    assert_eq!(settings["autoMemoryEnabled"], true);
    assert_eq!(settings["permissions"]["allow"][0], "Read");
    assert_eq!(settings["third_party"]["enabled"], true);
    assert!(settings["hooks"]["Stop"]
        .to_string()
        .contains("personal-stop"));
}

#[test]
fn target_claude_hooks_are_accelerators_without_a_second_prepare_router() {
    let temp = tempdir().unwrap();
    install_adapter(temp.path(), AgentAdapter::Claude).unwrap();
    let settings: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(temp.path().join(".claude/settings.json")).unwrap(),
    )
    .unwrap();
    let hooks = settings["hooks"].to_string();
    assert!(hooks.contains("baron automation hook"));
    assert!(!hooks.contains("control-plane prepare"));
    assert!(hooks.contains("SessionStart"));
    assert!(hooks.contains("Stop"));
}
