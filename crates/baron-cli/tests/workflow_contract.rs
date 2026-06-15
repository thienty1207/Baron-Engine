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
fn release_workflow_builds_native_archives_and_publishes_verified_metadata() {
    let workflow =
        fs::read_to_string(workspace_root().join(".github/workflows/release.yml")).unwrap();

    for required in [
        "tags:",
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
    ] {
        assert!(
            workflow.contains(required),
            "release workflow is missing {required}"
        );
    }
}
