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
        for entry in fs::read_dir(current).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                visit(root, &path, files);
            } else {
                if path.ends_with(".baron-mutation.lock") {
                    continue;
                }
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
fn target_codex_payloads_include_native_bridge_metadata_and_core_wrappers() {
    let payloads = managed_payloads_for_adapter(AgentAdapter::Codex).unwrap();
    let paths = payloads
        .iter()
        .map(|payload| {
            (
                payload.relative_path.to_string_lossy().replace('\\', "/"),
                payload,
            )
        })
        .collect::<BTreeMap<_, _>>();

    assert!(payloads.iter().all(|payload| payload.adapter == "codex"));

    for path in [
        "AGENTS.md",
        ".agents/skills/baron-engine/SKILL.md",
        ".agents/skills/baron-engine/agents/openai.yaml",
        ".codex/INDEX.md",
        ".codex/agents/INDEX.md",
        ".codex/agents/code-reviewer.toml",
        ".codex/agents/security-auditor.toml",
        ".codex/agents/test-engineer.toml",
        ".codex/hooks.json",
    ] {
        assert!(paths.contains_key(path), "missing Codex payload {path}");
    }

    assert!(!paths
        .keys()
        .any(|path| path.starts_with(".codex/skills/") && path.ends_with("/SKILL.md")));
    assert_eq!(
        paths[".agents/skills/baron-engine/agents/openai.yaml"].merge_kind,
        ManagedMergeKind::FullText
    );
    for path in [
        ".agents/skills/baron-engine/SKILL.md",
        ".codex/INDEX.md",
        ".codex/agents/code-reviewer.toml",
        ".codex/agents/security-auditor.toml",
        ".codex/agents/test-engineer.toml",
    ] {
        assert_eq!(paths[path].merge_kind, ManagedMergeKind::FullText);
    }
    assert_eq!(
        paths[".codex/hooks.json"].merge_kind,
        ManagedMergeKind::JsonOwnedEntries
    );
    assert!(paths[".agents/skills/baron-engine/agents/openai.yaml"]
        .content
        .contains("allow_implicit_invocation: false"));
    assert!(paths[".agents/skills/baron-engine/agents/openai.yaml"]
        .content
        .contains("codex"));
    assert!(paths[".agents/skills/baron-engine/SKILL.md"]
        .content
        .contains("control-plane prepare"));
    assert!(!paths[".agents/skills/baron-engine/SKILL.md"]
        .content
        .contains("# Baron Skill Routing"));

    for name in ["code-reviewer", "security-auditor", "test-engineer"] {
        let path = format!(".codex/agents/{name}.toml");
        let content = &paths[&path].content;
        assert!(content.contains(&format!(".baron/core/agents/{name}.toml")));
        assert!(content.contains("parent"));
        assert!(
            content.lines().count() < 30,
            "wrapper {name} copied semantic policy"
        );
    }
}

#[test]
fn target_fresh_codex_install_has_one_thin_native_surface() {
    let temp = tempdir().unwrap();
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();

    for path in [
        "AGENTS.md",
        ".agents/skills/baron-engine/SKILL.md",
        ".agents/skills/baron-engine/agents/openai.yaml",
        ".codex/INDEX.md",
        ".codex/agents/code-reviewer.toml",
        ".codex/agents/security-auditor.toml",
        ".codex/agents/test-engineer.toml",
        ".codex/hooks.json",
    ] {
        assert!(temp.path().join(path).is_file(), "missing {path}");
    }
    assert!(temp
        .path()
        .join(".baron/core/skills/superpowers/SKILL.md")
        .is_file());
    assert!(!temp
        .path()
        .join(".codex/skills/superpowers/SKILL.md")
        .exists());
    assert!(!temp.path().join(".codex/skills").is_dir());

    let baseline = load_managed_baseline(temp.path()).unwrap();
    for path in [
        ".agents/skills/baron-engine/SKILL.md",
        ".agents/skills/baron-engine/agents/openai.yaml",
        ".codex/agents/code-reviewer.toml",
        ".codex/agents/security-auditor.toml",
        ".codex/agents/test-engineer.toml",
    ] {
        let record = baseline
            .records
            .iter()
            .find(|record| record.relative_path == Path::new(path))
            .unwrap_or_else(|| panic!("missing baseline record {path}"));
        assert_eq!(
            record.owner,
            ManagedOwner::Adapter(SupportedManagedAdapter::Codex)
        );
    }
}

#[test]
fn target_codex_agents_contract_is_compact_and_automatic() {
    let temp = tempdir().unwrap();
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();
    let content = fs::read_to_string(temp.path().join("AGENTS.md")).unwrap();
    let start = content.find("<!-- BARON:MANAGED:START -->").unwrap();
    let end = content.find("<!-- BARON:MANAGED:END -->").unwrap();
    let managed = &content[start..end];

    assert!(managed.len() < 7_000, "managed Codex contract is too large");
    let managed_lower = managed.to_lowercase();
    for required in [
        "control-plane prepare --adapter codex --json",
        "resume",
        "selected",
        ".baron/core/skills/<skill>",
        "verification",
        "checkpoint",
        "user's current explicit instructions take precedence",
        "do not ask the user to run hidden baron commands",
        "do not recursively load the full `.baron/core/**` tree",
        "baron capability check --adapter codex",
        "preparepacketv1",
        "baron proof record",
        "baron autopilot review",
        "baron authority classify",
    ] {
        assert!(
            managed_lower.contains(required),
            "AGENTS contract missing {required}"
        );
    }
    assert!(!managed.contains("read all .baron/core/skills"));
}

#[test]
fn target_codex_projection_preserves_colliding_user_bridge_and_agents() {
    let temp = tempdir().unwrap();
    let bridge = temp.path().join(".agents/skills/baron-engine/SKILL.md");
    let agent = temp.path().join(".codex/agents/code-reviewer.toml");
    fs::create_dir_all(bridge.parent().unwrap()).unwrap();
    fs::create_dir_all(agent.parent().unwrap()).unwrap();
    fs::write(&bridge, "# User bridge\n").unwrap();
    fs::write(&agent, "name = \"my-reviewer\"\n").unwrap();

    let report = install_adapter(temp.path(), AgentAdapter::Codex).unwrap();

    assert_eq!(fs::read_to_string(&bridge).unwrap(), "# User bridge\n");
    assert_eq!(
        fs::read_to_string(&agent).unwrap(),
        "name = \"my-reviewer\"\n"
    );
    assert!(report
        .conflicts
        .iter()
        .any(|path| path == ".agents/skills/baron-engine/SKILL.md"));
    assert!(report
        .conflicts
        .iter()
        .any(|path| path == ".codex/agents/code-reviewer.toml"));
}

#[test]
fn target_codex_install_is_idempotent_and_does_not_mutate_claude() {
    let temp = tempdir().unwrap();
    install_adapter(temp.path(), AgentAdapter::Claude).unwrap();
    let claude_before = snapshot(&temp.path().join(".claude"));
    let core_before = snapshot(&temp.path().join(".baron/core"));
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();
    let first = snapshot(temp.path());
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();
    let second = snapshot(temp.path());

    assert_eq!(first, second);
    assert_eq!(claude_before, snapshot(&temp.path().join(".claude")));
    assert_eq!(core_before, snapshot(&temp.path().join(".baron/core")));
    assert!(temp
        .path()
        .join(".baron/core/agents/code-reviewer.toml")
        .is_file());
}

#[test]
fn target_codex_hooks_and_user_agents_text_are_preserved() {
    let temp = tempdir().unwrap();
    fs::write(
        temp.path().join("AGENTS.md"),
        "# Project instructions\n\nKeep this exact text.\n",
    )
    .unwrap();
    let unrelated_skill = temp.path().join(".agents/skills/project-local/SKILL.md");
    fs::create_dir_all(unrelated_skill.parent().unwrap()).unwrap();
    fs::write(&unrelated_skill, "# Project-local Codex skill\n").unwrap();
    let nested_agents = temp.path().join("src/AGENTS.md");
    fs::create_dir_all(nested_agents.parent().unwrap()).unwrap();
    fs::write(&nested_agents, "# Nested instructions\n").unwrap();
    let personal_agent = temp.path().join(".codex/agents/personal.toml");
    fs::create_dir_all(personal_agent.parent().unwrap()).unwrap();
    fs::write(&personal_agent, "name = \"personal\"\n").unwrap();
    let hooks = temp.path().join(".codex/hooks.json");
    fs::create_dir_all(hooks.parent().unwrap()).unwrap();
    fs::write(
        &hooks,
        r#"{"hooks":{"SessionStart":[{"command":"third-party-hook"}]},"custom":true}"#,
    )
    .unwrap();

    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();

    let agents = fs::read_to_string(temp.path().join("AGENTS.md")).unwrap();
    assert!(agents.contains("# Project instructions"));
    assert!(agents.contains("Keep this exact text."));
    assert_eq!(
        fs::read_to_string(unrelated_skill).unwrap(),
        "# Project-local Codex skill\n"
    );
    assert_eq!(
        fs::read_to_string(nested_agents).unwrap(),
        "# Nested instructions\n"
    );
    assert_eq!(
        fs::read_to_string(personal_agent).unwrap(),
        "name = \"personal\"\n"
    );
    let hooks = fs::read_to_string(hooks).unwrap();
    assert!(hooks.contains("third-party-hook"));
    assert!(hooks.contains("\"custom\": true"));
}
