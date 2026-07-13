use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

fn files(root: &Path) -> BTreeSet<PathBuf> {
    fn visit(root: &Path, current: &Path, output: &mut BTreeSet<PathBuf>) {
        for entry in fs::read_dir(current).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                visit(root, &entry.path(), output);
            } else {
                output.insert(entry.path().strip_prefix(root).unwrap().to_path_buf());
            }
        }
    }
    let mut output = BTreeSet::new();
    visit(root, root, &mut output);
    output
}

#[test]
fn hidden_authority_command_classifies_without_writing() {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("README.md"), "# Existing\n").unwrap();
    let before = files(temp.path());

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(temp.path())
        .args(["authority", "classify", "Review this repository only"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Authority: `read_only`"))
        .stdout(predicate::str::contains("Mutation allowed: `no`"));

    assert_eq!(before, files(temp.path()));
}

#[test]
fn authority_json_is_stable_for_agent_automation() {
    let output = Command::cargo_bin("baron")
        .unwrap()
        .args([
            "authority",
            "classify",
            "Review auth and apply valid fixes",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["authority"], "change");
    assert_eq!(value["mutationAllowed"], true);
    assert!(value["matchedChangeTerms"].as_array().unwrap().len() > 0);
}

#[test]
fn authority_command_stays_hidden_from_normal_help() {
    Command::cargo_bin("baron")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("authority").not());
}
