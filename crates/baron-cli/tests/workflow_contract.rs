use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn ci_covers_all_supported_native_platforms_and_quality_gates() {
    let workflow = fs::read_to_string(workspace_root().join(".github/workflows/ci.yml")).unwrap();

    for required in [
        "actions/checkout@v6",
        "windows-latest",
        "ubuntu-latest",
        "macos-15-intel",
        "macos-15",
        "cargo test --workspace --all-targets",
        "cargo fmt --all -- --check",
        "cargo clippy --workspace --all-targets -- -D warnings",
    ] {
        assert!(workflow.contains(required), "CI is missing {required}");
    }
}

#[test]
fn release_workflow_proves_an_exact_candidate_before_immutable_promotion() {
    let workflow =
        fs::read_to_string(workspace_root().join(".github/workflows/release.yml")).unwrap();

    for required in [
        "workflow_dispatch:",
        "release_version:",
        "source_revision:",
        "actions/checkout@v6",
        "actions/upload-artifact@v7",
        "actions/download-artifact@v8",
        "windows-latest",
        "ubuntu-latest",
        "macos-15-intel",
        "macos-15",
        "baron release metadata",
        "baron release verify",
        "SHA256SUMS",
        "release-manifest.json",
        "installers/install.ps1",
        "installers/install.sh",
        "gh release create",
        "contents: write",
        "git ls-remote --exit-code --tags",
        "gh release view",
        "git tag -a",
    ] {
        assert!(
            workflow.contains(required),
            "release workflow is missing {required}"
        );
    }
    assert!(workflow.contains("push:"));
    assert!(workflow.contains("tags:\n      - \"v*\""));
    assert!(workflow.contains("workflow_dispatch:"));
    assert!(!workflow.contains("on:\n  tags:"));
    assert!(!workflow.contains("--clobber"));
    assert!(workflow.contains("contents: read"));
    assert_eq!(workflow.matches("contents: write").count(), 1);
    assert!(workflow.contains("ref: ${{ needs.verify-candidate.outputs.source_sha }}"));

    let full_test = workflow
        .find("cargo test --workspace --all-targets")
        .unwrap();
    let create_tag = workflow.find("git tag -a").unwrap();
    let create_release = workflow.find("gh release create").unwrap();
    assert!(full_test < create_tag);
    assert!(create_tag < create_release);
}

#[test]
fn release_workflow_stages_and_publishes_raw_self_update_candidates() {
    let workflow =
        fs::read_to_string(workspace_root().join(".github/workflows/release.yml")).unwrap();

    for required in [
        "binary_suffix:",
        "Stage raw self-update candidate",
        "baron-v${{ needs.verify-candidate.outputs.version }}-${{ matrix.target }}${{ matrix.binary_suffix }}",
        "Raw native executable is missing",
    ] {
        assert!(
            workflow.contains(required),
            "release workflow is missing raw self-update candidate contract: {required}"
        );
    }
}
