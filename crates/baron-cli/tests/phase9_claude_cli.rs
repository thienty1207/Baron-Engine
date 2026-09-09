use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
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

fn init(repo: &Path, vault: &Path, adapter: &str) {
    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "init",
            repo.to_str().unwrap(),
            adapter,
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();
}

fn claude_projection_snapshot(repo: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    [
        "CLAUDE.md",
        ".claude/skills/baron-engine/SKILL.md",
        ".claude/agents/code-reviewer.md",
        ".claude/agents/security-auditor.md",
        ".claude/agents/test-engineer.md",
        ".claude/commands/baron-context.md",
        ".claude/commands/baron-status.md",
        ".claude/skills/INDEX.md",
        ".claude/agents/INDEX.md",
        ".claude/settings.json",
    ]
    .into_iter()
    .map(|relative| {
        let path = PathBuf::from(relative);
        (path.clone(), fs::read(repo.join(path)).unwrap())
    })
    .collect()
}

#[test]
fn target_claude_cli_install_is_a_thin_core_bridge() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("claude-bridge");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--claude");

    assert!(repo
        .join(".baron/core/skills/superpowers/SKILL.md")
        .is_file());
    assert!(repo.join(".baron/core/agents/code-reviewer.toml").is_file());
    assert!(repo.join("CLAUDE.md").is_file());
    assert!(repo.join(".claude/skills/baron-engine/SKILL.md").is_file());
    for name in ["code-reviewer", "security-auditor", "test-engineer"] {
        assert!(repo.join(format!(".claude/agents/{name}.md")).is_file());
    }
    assert!(repo.join(".claude/settings.json").is_file());
    assert!(!repo.join(".claude/skills/superpowers/SKILL.md").exists());
    assert!(!repo
        .join(".claude/skills/database-engineering/SKILL.md")
        .exists());
}

#[test]
fn target_claude_cli_init_is_idempotent_and_preserves_codex() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("coexist");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");
    let codex_before = snapshot(&repo.join(".codex"));

    init(&repo, &vault, "--claude");
    let first = claude_projection_snapshot(&repo);
    init(&repo, &vault, "--claude");
    let second = claude_projection_snapshot(&repo);

    assert_eq!(first, second);
    assert_eq!(codex_before, snapshot(&repo.join(".codex")));
    assert!(repo
        .join(".baron/core/skills/superpowers/SKILL.md")
        .is_file());
}

#[test]
fn target_claude_cli_preserves_user_files_settings_hooks_commands_and_agents() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("preserve");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join(".claude/skills/personal")).unwrap();
    fs::write(
        repo.join("CLAUDE.md"),
        b"# User contract\n\nKeep this text.\n",
    )
    .unwrap();
    fs::write(
        repo.join(".claude/skills/personal/SKILL.md"),
        b"# Personal skill\n",
    )
    .unwrap();
    fs::create_dir_all(repo.join(".claude/agents")).unwrap();
    fs::write(
        repo.join(".claude/agents/personal.md"),
        b"---\ndescription: personal\n---\n",
    )
    .unwrap();
    fs::create_dir_all(repo.join(".claude/commands")).unwrap();
    fs::write(
        repo.join(".claude/commands/personal.md"),
        b"# Personal command\n",
    )
    .unwrap();
    fs::write(
        repo.join(".claude/settings.json"),
        br#"{
  "autoMemoryEnabled": true,
  "permissions": {"allow": ["Read"]},
  "third_party": {"enabled": true},
  "hooks": {"Stop": [{"hooks": [{"type": "command", "command": "third-party-stop"}]}]}
}
"#,
    )
    .unwrap();
    fs::create_dir_all(repo.join(".codex/agents")).unwrap();
    fs::write(
        repo.join(".codex/agents/personal.toml"),
        b"name = \"personal\"\n",
    )
    .unwrap();

    init(&repo, &vault, "--claude");

    assert!(fs::read_to_string(repo.join("CLAUDE.md"))
        .unwrap()
        .contains("Keep this text."));
    assert_eq!(
        fs::read(repo.join(".claude/skills/personal/SKILL.md")).unwrap(),
        b"# Personal skill\n"
    );
    assert_eq!(
        fs::read(repo.join(".claude/agents/personal.md")).unwrap(),
        b"---\ndescription: personal\n---\n"
    );
    assert_eq!(
        fs::read(repo.join(".claude/commands/personal.md")).unwrap(),
        b"# Personal command\n"
    );
    assert_eq!(
        fs::read(repo.join(".codex/agents/personal.toml")).unwrap(),
        b"name = \"personal\"\n"
    );
    let settings: serde_json::Value =
        serde_json::from_slice(&fs::read(repo.join(".claude/settings.json")).unwrap()).unwrap();
    assert_eq!(settings["autoMemoryEnabled"], true);
    assert_eq!(settings["permissions"]["allow"][0], "Read");
    assert_eq!(settings["third_party"]["enabled"], true);
    assert!(settings["hooks"].to_string().contains("third-party-stop"));
}

#[test]
fn target_claude_cli_prepare_uses_structured_task_input_and_contract() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("prepare");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--claude");
    let contract = fs::read_to_string(repo.join("CLAUDE.md")).unwrap();
    for required in [
        "control-plane prepare --adapter claude --json",
        "structured",
        "PreparePacketV1",
        "selected",
        "verification",
        "auto memory",
        "non-authoritative",
    ] {
        assert!(contract.to_lowercase().contains(&required.to_lowercase()));
    }

    let payload = json!({
        "schema_version": 1,
        "repo": repo,
        "adapter": "claude",
        "task": "quotes \"unicode tiếng Việt\"; shell $() && ; newline\nsecond",
        "constraints": ["preserve user files"],
    });
    Command::cargo_bin("baron")
        .unwrap()
        .args(["control-plane", "prepare", "--adapter", "claude", "--json"])
        .current_dir(&repo)
        .write_stdin(serde_json::to_vec(&payload).unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("schema_version"));
}

#[test]
fn target_claude_cli_update_dry_run_keeps_native_projection_stable() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("update-preview");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--claude");
    let before = snapshot(&repo);

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["update", "--dry-run", "--installed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Safe Update Preview"))
        .stdout(predicate::str::contains("No project files were written."));

    assert_eq!(snapshot(&repo), before);
    assert!(!repo.join(".claude/skills/superpowers").exists());
}
