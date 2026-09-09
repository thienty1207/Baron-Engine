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

fn codex_projection_snapshot(repo: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    [
        "AGENTS.md",
        ".agents/skills/baron-engine/SKILL.md",
        ".agents/skills/baron-engine/agents/openai.yaml",
        ".codex/INDEX.md",
        ".codex/agents/INDEX.md",
        ".codex/agents/code-reviewer.toml",
        ".codex/agents/security-auditor.toml",
        ".codex/agents/test-engineer.toml",
        ".codex/hooks.json",
        ".baron/core/skills/superpowers/SKILL.md",
    ]
    .into_iter()
    .map(|relative| {
        let path = PathBuf::from(relative);
        (path.clone(), fs::read(repo.join(path)).unwrap())
    })
    .collect()
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

#[test]
fn target_codex_cli_install_is_a_thin_core_bridge() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("codex-bridge");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");

    assert!(repo
        .join(".baron/core/skills/superpowers/SKILL.md")
        .is_file());
    assert!(repo.join(".agents/skills/baron-engine/SKILL.md").is_file());
    assert!(repo
        .join(".agents/skills/baron-engine/agents/openai.yaml")
        .is_file());
    assert!(repo.join(".codex/agents/code-reviewer.toml").is_file());
    assert!(repo.join(".codex/agents/security-auditor.toml").is_file());
    assert!(repo.join(".codex/agents/test-engineer.toml").is_file());
    assert!(!repo.join(".codex/skills/superpowers/SKILL.md").exists());
    assert!(!repo.join(".codex/skills").is_dir());
}

#[test]
fn target_codex_cli_init_is_idempotent_and_preserves_claude() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("coexist");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--claude");
    let claude_before = snapshot(&repo.join(".claude"));
    init(&repo, &vault, "--codex");
    let first = codex_projection_snapshot(&repo);
    init(&repo, &vault, "--codex");
    let second = codex_projection_snapshot(&repo);

    assert_eq!(first, second);
    assert_eq!(claude_before, snapshot(&repo.join(".claude")));
}

#[test]
fn target_codex_cli_preserves_user_files_and_hook_entries() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("preserve");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join(".codex")).unwrap();
    fs::write(
        repo.join("AGENTS.md"),
        "# User contract\n\nDo not delete this text.\n",
    )
    .unwrap();
    fs::write(
        repo.join(".codex/hooks.json"),
        r#"{"hooks":{"SessionStart":[{"command":"user-hook"}]},"userSetting":42}"#,
    )
    .unwrap();
    fs::create_dir_all(repo.join(".agents/skills/project-local")).unwrap();
    fs::write(
        repo.join(".agents/skills/project-local/SKILL.md"),
        "# Project-local Codex skill\n",
    )
    .unwrap();
    fs::create_dir_all(repo.join(".codex/agents")).unwrap();
    fs::write(
        repo.join(".codex/agents/personal.toml"),
        "name = \"personal\"\n",
    )
    .unwrap();
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(repo.join("src/AGENTS.md"), "# Nested instructions\n").unwrap();

    init(&repo, &vault, "--codex");

    let agents = fs::read_to_string(repo.join("AGENTS.md")).unwrap();
    assert!(agents.contains("# User contract"));
    assert!(agents.contains("Do not delete this text."));
    assert_eq!(
        fs::read_to_string(repo.join(".agents/skills/project-local/SKILL.md")).unwrap(),
        "# Project-local Codex skill\n"
    );
    assert_eq!(
        fs::read_to_string(repo.join(".codex/agents/personal.toml")).unwrap(),
        "name = \"personal\"\n"
    );
    assert_eq!(
        fs::read_to_string(repo.join("src/AGENTS.md")).unwrap(),
        "# Nested instructions\n"
    );
    let hooks = fs::read_to_string(repo.join(".codex/hooks.json")).unwrap();
    assert!(hooks.contains("user-hook"));
    assert!(hooks.contains("\"userSetting\": 42"));
}

#[test]
fn target_codex_cli_prepare_guidance_uses_structured_task_input() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("prepare");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");
    let agents = fs::read_to_string(repo.join("AGENTS.md")).unwrap();
    assert!(agents.contains("control-plane prepare --adapter codex --json"));
    assert!(agents.contains("structured"));
    assert!(agents.contains("shell"));
    assert!(agents.contains("selected"));
    assert!(agents.contains("verification"));

    let payload = json!({
        "schema_version": 1,
        "repo": repo,
        "adapter": "codex",
        "task": "quotes \"unicode tiếng Việt\"; shell $() && ; newline\nsecond",
        "constraints": ["preserve user files"],
    });
    Command::cargo_bin("baron")
        .unwrap()
        .args(["control-plane", "prepare", "--adapter", "codex", "--json"])
        .current_dir(&repo)
        .write_stdin(serde_json::to_vec(&payload).unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("schema_version"));
}

#[test]
fn target_codex_cli_update_dry_run_keeps_native_projection_stable() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("update-preview");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");
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
    assert!(!repo.join(".codex/skills").exists());
}
