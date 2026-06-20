use std::fs;

use assert_cmd::Command;
use baron_core::release::SUPPORTED_RELEASE_TARGETS;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn hidden_release_commands_generate_and_verify_metadata() {
    let temp = tempdir().unwrap();
    for target in SUPPORTED_RELEASE_TARGETS {
        fs::write(
            temp.path().join(target.archive_name("3.1.2")),
            target.triple.as_bytes(),
        )
        .unwrap();
    }

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "release",
            "metadata",
            temp.path().to_str().unwrap(),
            "--release-version",
            "3.1.2",
            "--source-revision",
            "abc123",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Release metadata generated"))
        .stdout(predicate::str::contains("Artifacts: 4"));

    Command::cargo_bin("baron")
        .unwrap()
        .args(["release", "verify", temp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Release assets verified"))
        .stdout(predicate::str::contains("Version: `3.1.2`"));
}
