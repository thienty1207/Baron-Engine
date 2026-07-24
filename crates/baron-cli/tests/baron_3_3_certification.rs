use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn all_platform_flags_generate_deep_profiles() {
    let temp = tempdir().unwrap();
    let vault = temp.path().join("Vault");
    for platform in [
        "frontend",
        "backend",
        "fullstack",
        "mobile",
        "desktop",
        "tool",
        "library",
        "data",
        "cloud",
    ] {
        let repo = temp.path().join(format!("{platform}-app"));
        fs::create_dir_all(&repo).unwrap();
        Command::cargo_bin("baron")
            .unwrap()
            .args([
                "init",
                repo.to_str().unwrap(),
                "--agent",
                &format!("--{platform}"),
                "--vault",
                vault.to_str().unwrap(),
            ])
            .assert()
            .success();
        assert!(repo
            .join(format!("docs/baron/platform/profiles/{platform}.md"))
            .exists());
    }
}

#[test]
fn three_adapters_expand_fullstack_to_mobile_without_losing_custom_or_legacy_files() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("legacy-product");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join("odd-layout")).unwrap();
    fs::write(repo.join("odd-layout/app.ts"), "legacy source").unwrap();

    for adapter in ["codex", "claude", "agent"] {
        let mut args = vec![
            "init",
            repo.to_str().unwrap(),
            "--vault",
            vault.to_str().unwrap(),
        ];
        args.push(match adapter {
            "codex" => "--codex",
            "claude" => "--claude",
            _ => "--agent",
        });
        if adapter == "codex" {
            args.push("--fullstack");
        }
        Command::cargo_bin("baron")
            .unwrap()
            .args(args)
            .assert()
            .success();
    }
    fs::create_dir_all(repo.join(".codex/skills/custom-domain")).unwrap();
    fs::write(repo.join(".codex/skills/custom-domain/SKILL.md"), "custom").unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args(["init", repo.to_str().unwrap(), "--mobile"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .args(["automation", "reconcile", repo.to_str().unwrap()])
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(repo.join("odd-layout/app.ts")).unwrap(),
        "legacy source"
    );
    assert!(repo.join(".codex/skills/custom-domain/SKILL.md").exists());
    assert!(repo.join("AGENTS.md").exists());
    assert!(repo.join("CLAUDE.md").exists());
    assert!(repo.join("AGENT.md").exists());
    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "context",
            repo.to_str().unwrap(),
            "--codex",
            "--task",
            "implement mobile client for backend API",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Platform Intelligence"))
        .stdout(predicate::str::contains("## Architecture Governor"))
        .stdout(predicate::str::contains("Loaded profile: `mobile`"));
}

#[test]
fn release_binary_reports_current_source_version() {
    Command::cargo_bin("baron")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("baron 3.6.0"));
}
