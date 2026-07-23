use std::fs;

use assert_cmd::Command;
use baron_core::release::SUPPORTED_RELEASE_TARGETS;
use predicates::prelude::*;
use tempfile::tempdir;

const SOURCE_REVISION: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn hidden_release_commands_generate_and_verify_metadata() {
    let temp = tempdir().unwrap();
    for target in SUPPORTED_RELEASE_TARGETS {
        fs::write(
            temp.path().join(target.archive_name("3.2.0")),
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
            "3.2.0",
            "--source-revision",
            SOURCE_REVISION,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Release metadata generated"))
        .stdout(predicate::str::contains("Artifacts: 4"));

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "release",
            "verify",
            temp.path().to_str().unwrap(),
            "--expected-version",
            "3.2.0",
            "--expected-source-revision",
            SOURCE_REVISION,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Release assets verified"))
        .stdout(predicate::str::contains("Version: `3.2.0`"));
}

#[test]
fn release_verify_rejects_an_unapproved_source_identity() {
    let temp = tempdir().unwrap();
    for target in SUPPORTED_RELEASE_TARGETS {
        fs::write(
            temp.path().join(target.archive_name("3.2.0")),
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
            "3.2.0",
            "--source-revision",
            SOURCE_REVISION,
        ])
        .assert()
        .success();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "release",
            "verify",
            temp.path().to_str().unwrap(),
            "--expected-version",
            "3.2.0",
            "--expected-source-revision",
            "ffffffffffffffffffffffffffffffffffffffff",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("source revision mismatch"));
}
