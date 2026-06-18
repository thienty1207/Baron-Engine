use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn list_files(root: &Path) -> BTreeSet<PathBuf> {
    fn visit(root: &Path, current: &Path, files: &mut BTreeSet<PathBuf>) {
        for entry in fs::read_dir(current).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(path.strip_prefix(root).unwrap().to_path_buf());
            }
        }
    }

    let mut files = BTreeSet::new();
    visit(root, root, &mut files);
    files
}

fn fixture_repo() -> tempfile::TempDir {
    let temp = tempdir().unwrap();
    let root = temp.path();
    write(
        &root.join("package.json"),
        r#"{"scripts":{"build":"vite build","test":"vitest"},"dependencies":{"@vitejs/plugin-react":"latest"}}"#,
    );
    write(&root.join("src/main.tsx"), "console.log('app');\n");
    write(
        &root.join("src/auth/login.ts"),
        "export function login() {}\n",
    );
    write(&root.join("README.md"), "# Fixture\n");
    temp
}

#[test]
fn cli_reports_the_release_version() {
    Command::cargo_bin("baron")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("baron 3.0.0"));
}

#[test]
fn top_level_help_stays_focused_on_user_commands() {
    Command::cargo_bin("baron")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("setup"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("memory").not())
        .stdout(predicate::str::contains("automation").not())
        .stdout(predicate::str::contains("autopilot").not())
        .stdout(predicate::str::contains("runtime").not())
        .stdout(predicate::str::contains("control-plane").not())
        .stdout(predicate::str::contains("harness").not());

    Command::cargo_bin("baron")
        .unwrap()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--codex"))
        .stdout(predicate::str::contains("--claude"))
        .stdout(predicate::str::contains("--agent"))
        .stdout(predicate::str::contains("--fullstack"))
        .stdout(predicate::str::contains("--tool"));
}

#[test]
fn hidden_automation_commands_remain_available_for_agents() {
    Command::cargo_bin("baron")
        .unwrap()
        .args(["memory", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("import-sessions"));

    Command::cargo_bin("baron")
        .unwrap()
        .args(["automation", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("reconcile"))
        .stdout(predicate::str::contains("hook"));

    Command::cargo_bin("baron")
        .unwrap()
        .args(["control-plane", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("route"))
        .stdout(predicate::str::contains("record-gate"));

    Command::cargo_bin("baron")
        .unwrap()
        .args(["harness", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("audit"))
        .stdout(predicate::str::contains("verify-all"))
        .stdout(predicate::str::contains("propose"));

    Command::cargo_bin("baron")
        .unwrap()
        .args(["asset", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("audit"))
        .stdout(predicate::str::contains("quarantine"))
        .stdout(predicate::str::contains("propose-skill"));

    Command::cargo_bin("baron")
        .unwrap()
        .args(["session-replay", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("index"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("replay"));

    Command::cargo_bin("baron")
        .unwrap()
        .args(["autopilot", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("review"))
        .stdout(predicate::str::contains("approve"))
        .stdout(predicate::str::contains("reject"));

    Command::cargo_bin("baron")
        .unwrap()
        .args(["runtime", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("check"));
}

#[test]
fn survey_prints_markdown_project_atlas() {
    let temp = fixture_repo();

    Command::cargo_bin("baron")
        .unwrap()
        .args(["survey", temp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Project Atlas"))
        .stdout(predicate::str::contains("## Overview"))
        .stdout(predicate::str::contains("## Stack Hints"))
        .stdout(predicate::str::contains("## Shadow Safety"));
}

#[test]
fn survey_json_output_is_machine_readable() {
    let temp = fixture_repo();

    let output = Command::cargo_bin("baron")
        .unwrap()
        .args(["survey", temp.path().to_str().unwrap(), "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["project_type"], "frontend");
    assert!(json["stack_hints"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["label"] == "Node package"));
}

#[test]
fn shadow_init_previews_codex_without_writing_files() {
    let temp = fixture_repo();
    let before = list_files(temp.path());

    Command::cargo_bin("baron")
        .unwrap()
        .args(["init", temp.path().to_str().unwrap(), "--codex", "--shadow"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Shadow Init Preview"))
        .stdout(predicate::str::contains("AGENTS.md"))
        .stdout(predicate::str::contains(".codex/skills"))
        .stdout(predicate::str::contains("No files were written"));

    assert_eq!(before, list_files(temp.path()));
}

#[test]
fn shadow_init_previews_claude_without_writing_files() {
    let temp = fixture_repo();
    let before = list_files(temp.path());

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "init",
            temp.path().to_str().unwrap(),
            "--claude",
            "--shadow",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("CLAUDE.md"))
        .stdout(predicate::str::contains(".claude/commands"))
        .stdout(predicate::str::contains("No files were written"));

    assert_eq!(before, list_files(temp.path()));
}

#[test]
fn shadow_init_previews_generic_agent_without_writing_files() {
    let temp = fixture_repo();
    let before = list_files(temp.path());

    Command::cargo_bin("baron")
        .unwrap()
        .args(["init", temp.path().to_str().unwrap(), "--agent", "--shadow"])
        .assert()
        .success()
        .stdout(predicate::str::contains("AGENT.md"))
        .stdout(predicate::str::contains("baron-context.md"))
        .stdout(predicate::str::contains("baron-context.json"))
        .stdout(predicate::str::contains("No files were written"));

    assert_eq!(before, list_files(temp.path()));
}
