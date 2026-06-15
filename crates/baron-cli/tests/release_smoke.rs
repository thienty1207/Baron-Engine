use std::fs;
use std::path::Path;

use assert_cmd::Command;
use baron_core::vault::project_slug;
use predicates::prelude::*;
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
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
fn fresh_and_old_projects_preserve_user_owned_content() {
    let temp = tempdir().unwrap();
    let vault = temp.path().join("vault");
    let fresh = temp.path().join("fresh-project");
    let old = temp.path().join("ten-year-old-project");
    fs::create_dir_all(&fresh).unwrap();
    fs::create_dir_all(&old).unwrap();
    write(
        &fresh.join("Cargo.toml"),
        "[package]\nname='fresh'\nversion='0.1.0'\n",
    );
    write(
        &old.join("AGENTS.md"),
        "# Existing Team Rules\n\nKeep this.\n",
    );
    write(
        &old.join(".codex/skills/custom-domain/SKILL.md"),
        "---\nname: custom-domain\ndescription: Use for custom work.\n---\n",
    );
    write(
        &old.join("package.json"),
        "{\"scripts\":{\"test\":\"node --test\"}}\n",
    );

    init(&fresh, &vault, "--codex");
    init(&old, &vault, "--codex");
    Command::cargo_bin("baron")
        .unwrap()
        .args(["update", old.to_str().unwrap()])
        .assert()
        .success();

    assert!(fresh.join("AGENTS.md").is_file());
    assert!(fresh.join(".codex/skills/superpowers/SKILL.md").is_file());
    assert!(fs::read_to_string(old.join("AGENTS.md"))
        .unwrap()
        .contains("Keep this."));
    assert!(old.join(".codex/skills/custom-domain/SKILL.md").is_file());
}

#[test]
fn large_repository_survey_and_context_remain_bounded() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("large-repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    write(
        &repo.join("package.json"),
        "{\"scripts\":{\"build\":\"vite build\",\"test\":\"vitest\"}}\n",
    );
    for index in 0..2_000 {
        write(
            &repo.join(format!("src/modules/module-{index}.ts")),
            &format!("export const value{index} = {index};\n"),
        );
    }
    for index in 0..500 {
        write(
            &repo.join(format!("node_modules/ignored-{index}/index.js")),
            "ignored\n",
        );
    }

    let survey = Command::cargo_bin("baron")
        .unwrap()
        .args(["survey", repo.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(survey.len() < 100_000, "survey output was not bounded");

    init(&repo, &vault, "--codex");
    let context = Command::cargo_bin("baron")
        .unwrap()
        .args([
            "context",
            repo.to_str().unwrap(),
            "--codex",
            "--task",
            "review frontend build",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(context.len() < 120_000, "context output was not bounded");
}

#[test]
fn shared_vault_keeps_weak_cross_project_memory_out() {
    let temp = tempdir().unwrap();
    let vault = temp.path().join("vault");
    let alpha = temp.path().join("alpha-app");
    let beta = temp.path().join("beta-app");
    fs::create_dir_all(&alpha).unwrap();
    fs::create_dir_all(&beta).unwrap();
    write(&alpha.join("README.md"), "# Alpha\n");
    write(&beta.join("README.md"), "# Beta\n");
    init(&alpha, &vault, "--agent");
    init(&beta, &vault, "--agent");

    let alpha_slug = project_slug(&alpha);
    let beta_slug = project_slug(&beta);
    write(
        &vault.join(format!("Projects/{alpha_slug}/Facts.md")),
        "# Facts\n\nAlpha authentication uses signed sessions.\n",
    );
    write(
        &vault.join(format!("Projects/{beta_slug}/Facts.md")),
        "# Facts\n\nBeta authentication secret must never leak into Alpha.\n",
    );
    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "memory",
            "index",
            alpha.to_str().unwrap(),
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "recall",
            "authentication",
            alpha.to_str().unwrap(),
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Alpha authentication"))
        .stdout(predicate::str::contains("Beta authentication secret").not());
}

#[test]
fn three_adapters_and_capability_degradation_work_together() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("multi-agent");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    write(&repo.join("README.md"), "# Multi Agent\n");

    init(&repo, &vault, "--codex");
    init(&repo, &vault, "--claude");
    init(&repo, &vault, "--agent");
    assert!(repo.join("AGENTS.md").is_file());
    assert!(repo.join("CLAUDE.md").is_file());
    assert!(repo.join("AGENT.md").is_file());

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "capability",
            "register",
            "optional deploy verification",
            repo.to_str().unwrap(),
            "--name",
            "missing-deploy-cli",
            "--kind",
            "cli",
            "--command",
            "definitely-not-installed-baron-smoke",
            "--adapter",
            "codex",
            "--description",
            "Optional deployment verification provider.",
        ])
        .assert()
        .success();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["capability", "check", "--adapter", "codex"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Presence: `missing`"))
        .stdout(predicate::str::contains("Optional gaps:"));
}
