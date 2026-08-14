use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use baron_core::release::{
    supported_release_target, write_release_metadata, SUPPORTED_RELEASE_TARGETS,
};
use predicates::prelude::*;
use semver::Version;
use sha2::{Digest, Sha256};
use tempfile::tempdir;

const SOURCE_REVISION: &str = "0123456789abcdef0123456789abcdef01234567";

fn current_target() -> &'static str {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "x86_64-pc-windows-msvc"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "x86_64-unknown-linux-gnu"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "x86_64-apple-darwin"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else {
        panic!("unsupported Baron update transaction test target")
    }
}

fn snapshot(root: &Path, excluded: Option<&Path>) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(
        root: &Path,
        current: &Path,
        excluded: Option<&Path>,
        files: &mut BTreeMap<PathBuf, Vec<u8>>,
    ) {
        let mut entries = fs::read_dir(current)
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            if excluded.is_some_and(|skip| path.starts_with(skip)) {
                continue;
            }
            if path.is_dir() {
                collect(root, &path, excluded, files);
            } else if path.is_file() {
                files.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(&path).unwrap(),
                );
            }
        }
    }

    let mut files = BTreeMap::new();
    collect(root, root, excluded, &mut files);
    files
}

fn patched_upgrade_binary(source: &Path, destination: &Path, from: &str, to: &str) {
    assert_eq!(
        from.len(),
        to.len(),
        "test binary patch must preserve byte width"
    );
    let mut bytes = fs::read(source).unwrap();
    let needle = from.as_bytes();
    let replacement = to.as_bytes();
    let mut replacements = 0;
    for index in 0..=bytes.len().saturating_sub(needle.len()) {
        if bytes[index..index + needle.len()] == *needle {
            bytes[index..index + replacement.len()].copy_from_slice(replacement);
            replacements += 1;
        }
    }
    assert!(
        replacements > 0,
        "test binary must embed its package version"
    );
    fs::write(destination, bytes).unwrap();
    fs::set_permissions(destination, fs::metadata(source).unwrap().permissions()).unwrap();
}

#[cfg(unix)]
fn patched_unix_upgrade_binary(source: &Path, destination: &Path, from: &str, to: &str) {
    assert_eq!(
        from.len(),
        to.len(),
        "test binary patch must preserve byte width"
    );
    let mut bytes = fs::read(source).unwrap();
    let needle = from.as_bytes();
    let replacement = to.as_bytes();
    let mut replacements = 0;
    for index in 0..=bytes.len().saturating_sub(needle.len()) {
        if bytes[index..index + needle.len()] != *needle {
            continue;
        }
        // ELF binaries carry ABI version strings such as `GCC_4.2.0`. Rewrite
        // every non-ABI occurrence so Clap/help/version strings all agree, but
        // never touch a loader symbol-version token.
        let is_abi_version = (index >= 4 && &bytes[index - 4..index] == b"GCC_")
            || (index >= 6 && &bytes[index - 6..index] == b"GLIBC_");
        if is_abi_version {
            continue;
        }
        bytes[index..index + replacement.len()].copy_from_slice(replacement);
        replacements += 1;
    }
    assert!(
        replacements > 0,
        "Unix test binary must contain a safe package-version token"
    );
    fs::write(destination, bytes).unwrap();
    fs::set_permissions(destination, fs::metadata(source).unwrap().permissions()).unwrap();
}

fn unix_candidate_delegate_script(backing_binary: &Path, version: &str) -> String {
    let binary = backing_binary.to_string_lossy().replace('\'', "'\"'\"'");
    format!(
        "#!/bin/sh\nif [ \"${{1:-}}\" = \"--version\" ]; then\n  printf '%s\\n' 'baron {version}'\n  exit 0\nfi\nexec '{binary}' \"$@\"\n"
    )
}

fn write_upgrade_fixture(
    release: &Path,
    running_binary: &Path,
    version: &str,
    running_version: &str,
) {
    for target in SUPPORTED_RELEASE_TARGETS {
        fs::write(
            release.join(target.archive_name(version)),
            format!("archive:{}", target.triple),
        )
        .unwrap();
        let candidate = release.join(target.update_candidate_name(version));
        if target.triple == current_target() {
            #[cfg(target_os = "windows")]
            patched_upgrade_binary(running_binary, &candidate, running_version, version);
            #[cfg(not(target_os = "windows"))]
            {
                // The staged candidate obeys the production size limit while
                // the safe patcher preserves the candidate protocol behavior.
                let backing = release.join(format!("candidate-runtime-{}", target.triple));
                patched_unix_upgrade_binary(running_binary, &backing, running_version, version);
                fs::write(
                    &candidate,
                    unix_candidate_delegate_script(&backing, version),
                )
                .unwrap();
                // `Command::new` must exercise the same executable handoff that
                // a real Unix raw candidate uses. `fs::write` creates a shell
                // delegate as 0644 by default, which only fails on Unix CI.
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;

                    let mut permissions = fs::metadata(&candidate).unwrap().permissions();
                    permissions.set_mode(0o755);
                    fs::set_permissions(candidate, permissions).unwrap();
                }
            }
        } else {
            fs::write(candidate, format!("other-target:{}", target.triple)).unwrap();
        }
    }
    write_release_metadata(release, version, SOURCE_REVISION).unwrap();
}

fn transaction_state(repo: &Path) -> PathBuf {
    let transactions = repo.join(".baron/update/transactions");
    let entries = fs::read_dir(&transactions)
        .unwrap()
        .map(Result::unwrap)
        .filter(|entry| entry.path().is_dir())
        .collect::<Vec<_>>();
    assert_eq!(
        entries.len(),
        1,
        "one verified transaction should be staged"
    );
    entries[0].path().join("state.json")
}

fn write_sealed_state(path: &Path, state: &serde_json::Value) {
    let content = format!("{}\n", serde_json::to_string_pretty(state).unwrap());
    let checksum = format!("{:x}", Sha256::digest(content.as_bytes()));
    fs::write(path, content).unwrap();
    fs::write(
        path.parent().unwrap().join("state.sha256"),
        format!("{checksum}\n"),
    )
    .unwrap();
}

#[test]
fn verified_candidate_protocol_stages_then_aborts_without_touching_project_or_vault() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    let release = temp.path().join("release");
    fs::create_dir_all(repo.join(".codex/skills/custom")).unwrap();
    fs::create_dir_all(&release).unwrap();
    fs::write(
        repo.join(".codex/skills/custom/SKILL.md"),
        "custom user skill",
    )
    .unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "init",
            repo.to_str().unwrap(),
            "--codex",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();
    fs::write(
        vault.join("operator-memory.md"),
        "vault memory must survive",
    )
    .unwrap();

    let running_binary = Command::cargo_bin("baron")
        .unwrap()
        .get_program()
        .to_owned();
    let running_version = env!("CARGO_PKG_VERSION");
    let mut candidate_version = Version::parse(running_version).unwrap();
    candidate_version.patch += 1;
    let candidate_version = candidate_version.to_string();
    assert_eq!(candidate_version.len(), running_version.len());
    write_upgrade_fixture(
        &release,
        Path::new(&running_binary),
        &candidate_version,
        running_version,
    );
    let repo_before = snapshot(&repo, Some(&repo.join(".baron/update")));
    let vault_before = snapshot(&vault, None);

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "update",
            repo.to_str().unwrap(),
            "--verify-candidate",
            "--candidate-dir",
            release.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Runtime activation: not performed",
        ));

    let state_path = transaction_state(&repo);
    let state: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    assert_eq!(state["status"], "verified");
    let candidate_relative = state["candidate_relative_path"].as_str().unwrap();
    let staged_candidate = repo.join(".baron/update").join(candidate_relative);
    Command::new(&staged_candidate)
        .args([
            "update",
            repo.to_str().unwrap(),
            "--candidate-plan",
            "--transaction",
            state_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Candidate rendered only staged packets",
        ));

    let planned: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    assert_eq!(planned["status"], "planned");
    assert!(!planned["packets"].as_array().unwrap().is_empty());
    assert_eq!(
        snapshot(&repo, Some(&repo.join(".baron/update"))),
        repo_before
    );
    assert_eq!(snapshot(&vault, None), vault_before);

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "update",
            repo.to_str().unwrap(),
            "--abort",
            "--transaction",
            state_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: `aborted`"));

    assert!(!state_path.exists());
    assert_eq!(
        snapshot(&repo, Some(&repo.join(".baron/update"))),
        repo_before
    );
    assert_eq!(snapshot(&vault, None), vault_before);
    assert!(supported_release_target(current_target()).is_ok());
}

#[test]
fn public_update_keeps_a_pending_conflict_staged_without_project_or_vault_writes() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    let release = temp.path().join("release");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&release).unwrap();
    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "init",
            repo.to_str().unwrap(),
            "--codex",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();
    let running_binary = Command::cargo_bin("baron")
        .unwrap()
        .get_program()
        .to_owned();
    let running_version = env!("CARGO_PKG_VERSION");
    let mut candidate_version = Version::parse(running_version).unwrap();
    candidate_version.patch += 1;
    let candidate_version = candidate_version.to_string();
    write_upgrade_fixture(
        &release,
        Path::new(&running_binary),
        &candidate_version,
        running_version,
    );
    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "update",
            repo.to_str().unwrap(),
            "--verify-candidate",
            "--candidate-dir",
            release.to_str().unwrap(),
        ])
        .assert()
        .success();
    let state_path = transaction_state(&repo);
    let mut state: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    state["status"] = serde_json::Value::String("conflict".to_string());
    state["last_checkpoint"] = serde_json::Value::String("conflict_packets_staged".to_string());
    write_sealed_state(&state_path, &state);
    let repo_before = snapshot(&repo, Some(&repo.join(".baron/update")));
    let vault_before = snapshot(&vault, None);

    Command::cargo_bin("baron")
        .unwrap()
        .args(["update", repo.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Update Needs Review"))
        .stdout(predicate::str::contains(
            "Project managed assets: unchanged",
        ))
        .stdout(predicate::str::contains("Runtime: unchanged"));

    assert_eq!(
        snapshot(&repo, Some(&repo.join(".baron/update"))),
        repo_before
    );
    assert_eq!(snapshot(&vault, None), vault_before);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&state_path).unwrap())
            .unwrap()["status"],
        "conflict"
    );
}

#[test]
fn unix_candidate_delegate_script_escapes_paths_and_forwards_arguments() {
    assert_eq!(
        unix_candidate_delegate_script(Path::new("/tmp/Baron's candidate"), "4.2.1"),
        "#!/bin/sh\nif [ \"${1:-}\" = \"--version\" ]; then\n  printf '%s\\n' 'baron 4.2.1'\n  exit 0\nfi\nexec '/tmp/Baron'\"'\"'s candidate' \"$@\"\n"
    );
}
