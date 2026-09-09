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
        "actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803",
        "windows-latest",
        "ubuntu-latest",
        "cargo test --workspace --all-targets",
        "cargo fmt --all -- --check",
        "cargo clippy --workspace --all-targets -- -D warnings",
    ] {
        assert!(workflow.contains(required), "CI is missing {required}");
    }
    assert!(
        !workflow.contains("macos-"),
        "CI release matrix intentionally targets Windows and Linux only"
    );
}

#[test]
fn release_workflow_proves_an_exact_candidate_before_immutable_promotion() {
    let workflow =
        fs::read_to_string(workspace_root().join(".github/workflows/release.yml")).unwrap();

    for required in [
        "workflow_dispatch:",
        "release_version:",
        "source_revision:",
        "actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803",
        "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
        "actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c",
        "windows-latest",
        "ubuntu-latest",
        "baron release verify",
        "SHA256SUMS",
        "release-manifest.json",
        "release-manifest.sig",
        "Generate canonical unsigned metadata",
        "Validate identity and sign metadata with protected seed",
        ".github/scripts/generate_release_manifest.py",
        "base64 --decode",
        "openssl pkey",
        "openssl pkeyutl -sign -rawin",
        "pinned.bin",
        "73a005a12cf79f1fa60612f0e359a13c83d2660806075b610b3d83f4f14c31b4",
        "installers/install.ps1",
        "installers/install.sh",
        "gh release create",
        "contents: write",
        "git ls-remote --exit-code --tags",
        "gh release view",
        "git tag -a",
        "BARON_RELEASE_SIGNING_KEY",
        "BARON_RELEASE_KEY_ID",
        "test -n \"$BARON_RELEASE_SIGNING_KEY\"",
    ] {
        assert!(
            workflow.contains(required),
            "release workflow is missing {required}"
        );
    }
    assert!(workflow.contains("push:"));
    assert!(workflow.contains("tags:\n      - \"v*\""));
    assert!(workflow.contains("workflow_dispatch:"));
    assert!(
        !workflow.contains("macos-"),
        "release matrix intentionally targets Windows and Linux only"
    );
    assert!(!workflow.contains("on:\n  tags:"));
    assert!(!workflow.contains("--clobber"));
    assert!(workflow.contains("contents: read"));
    assert_eq!(workflow.matches("contents: write").count(), 1);
    assert!(!workflow.contains("actions/checkout@v"));
    assert!(!workflow.contains("actions/upload-artifact@v"));
    assert!(!workflow.contains("actions/download-artifact@v"));
    assert!(workflow.contains("ref: ${{ needs.verify-candidate.outputs.source_sha }}"));
    assert!(
        workflow.contains("git tag -a \"v$version\" \"$source_sha\" -m \"Baron Engine v$version\"")
    );
    assert!(workflow.contains("--title \"Baron Engine v$version\""));

    let full_test = workflow
        .find("cargo test --workspace --all-targets")
        .unwrap();
    let create_tag = workflow.find("git tag -a").unwrap();
    let create_release = workflow.find("gh release create").unwrap();
    assert!(full_test < create_tag);
    assert!(create_tag < create_release);
    assert!(
        workflow.contains("protected GitHub Actions secret")
            || workflow.contains("protected repository secret")
    );
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
