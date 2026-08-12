use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use baron_core::release::{
    supported_release_target, write_release_metadata, SUPPORTED_RELEASE_TARGETS,
};
use predicates::prelude::*;
use tempfile::tempdir;

const SOURCE_REVISION: &str = "0123456789abcdef0123456789abcdef01234567";

fn release_version() -> String {
    let mut version = semver::Version::parse(env!("CARGO_PKG_VERSION"))
        .expect("test package version must be valid semver");
    version.patch += 1;
    version.to_string()
}

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
        panic!("unsupported Baron self-update test target")
    }
}

fn snapshot_outside_update_workspace(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(root: &Path, current: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        let mut entries = fs::read_dir(current)
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap();
            if relative.starts_with(Path::new(".baron/update")) {
                continue;
            }
            if path.is_dir() {
                collect(root, &path, files);
            } else if path.is_file() {
                files.insert(relative.to_path_buf(), fs::read(&path).unwrap());
            }
        }
    }

    let mut files = BTreeMap::new();
    collect(root, root, &mut files);
    files
}

fn write_release_fixture(release: &Path, running_binary: &Path, release_version: &str) {
    for target in SUPPORTED_RELEASE_TARGETS {
        fs::write(
            release.join(target.archive_name(release_version)),
            format!("archive:{}", target.triple),
        )
        .unwrap();
        let candidate = release.join(target.update_candidate_name(release_version));
        if target.triple == current_target() {
            #[cfg(target_os = "windows")]
            fs::copy(running_binary, candidate).unwrap();
            #[cfg(not(target_os = "windows"))]
            {
                let binary = running_binary.to_string_lossy().replace('\'', "'\"'\"'");
                fs::write(candidate, format!("#!/bin/sh\nexec '{binary}' \"$@\"\n")).unwrap();
            }
        } else {
            fs::write(candidate, format!("other-target:{}", target.triple)).unwrap();
        }
    }
    write_release_metadata(release, release_version, SOURCE_REVISION).unwrap();
}

#[test]
fn candidate_verification_stages_only_under_update_workspace_when_binary_identity_fails() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    let release = temp.path().join("release");
    let release_version = release_version();
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
    write_release_fixture(&release, Path::new(&running_binary), &release_version);
    let before = snapshot_outside_update_workspace(&repo);

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
        .failure()
        .stderr(predicate::str::contains("Staged Baron candidate reported"));

    assert_eq!(snapshot_outside_update_workspace(&repo), before);
    assert!(repo.join(".baron/update").is_dir());
    let target = supported_release_target(current_target()).unwrap();
    assert!(repo
        .join(".baron/update")
        .read_dir()
        .unwrap()
        .filter_map(Result::ok)
        .any(|entry| entry
            .path()
            .join(target.update_candidate_name(&release_version))
            .is_file()));
}
