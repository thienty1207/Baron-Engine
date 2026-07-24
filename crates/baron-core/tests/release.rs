use std::fs;

use baron_core::release::{
    build_release_manifest, load_and_verify_release_metadata, render_sha256sums,
    verify_release_assets, verify_release_identity, write_release_metadata, ReleaseArtifactInput,
    SUPPORTED_RELEASE_TARGETS,
};
use tempfile::tempdir;

const SOURCE_REVISION: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn supported_targets_have_stable_native_archive_names() {
    let names = SUPPORTED_RELEASE_TARGETS
        .iter()
        .map(|target| target.archive_name("3.2.0"))
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            "baron-v3.2.0-x86_64-pc-windows-msvc.zip",
            "baron-v3.2.0-x86_64-unknown-linux-gnu.tar.gz",
            "baron-v3.2.0-x86_64-apple-darwin.tar.gz",
            "baron-v3.2.0-aarch64-apple-darwin.tar.gz",
        ]
    );
}

#[test]
fn manifest_and_checksum_output_are_deterministic_and_verifiable() {
    let temp = tempdir().unwrap();
    let windows = temp.path().join("baron-v3.2.0-x86_64-pc-windows-msvc.zip");
    let linux = temp
        .path()
        .join("baron-v3.2.0-x86_64-unknown-linux-gnu.tar.gz");
    fs::write(&windows, b"windows archive").unwrap();
    fs::write(&linux, b"linux archive").unwrap();

    let manifest = build_release_manifest(
        "3.2.0",
        SOURCE_REVISION,
        &[
            ReleaseArtifactInput::new("x86_64-unknown-linux-gnu", &linux),
            ReleaseArtifactInput::new("x86_64-pc-windows-msvc", &windows),
        ],
    )
    .unwrap();

    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.product, "Baron Engine");
    assert_eq!(manifest.version, "3.2.0");
    assert_eq!(manifest.source_revision, SOURCE_REVISION);
    assert_eq!(manifest.artifacts.len(), 2);
    assert_eq!(
        manifest.artifacts[0].name,
        "baron-v3.2.0-x86_64-pc-windows-msvc.zip"
    );
    assert_eq!(
        manifest.artifacts[1].name,
        "baron-v3.2.0-x86_64-unknown-linux-gnu.tar.gz"
    );

    let checksums = render_sha256sums(&manifest);
    assert!(checksums.ends_with('\n'));
    assert!(checksums.contains("  baron-v3.2.0-x86_64-pc-windows-msvc.zip\n"));
    assert!(checksums.contains("  baron-v3.2.0-x86_64-unknown-linux-gnu.tar.gz\n"));

    verify_release_assets(temp.path(), &manifest, &checksums).unwrap();
}

#[test]
fn verification_rejects_an_archive_modified_after_manifest_generation() {
    let temp = tempdir().unwrap();
    let archive = temp
        .path()
        .join("baron-v3.2.0-x86_64-unknown-linux-gnu.tar.gz");
    fs::write(&archive, b"original").unwrap();
    let manifest = build_release_manifest(
        "3.2.0",
        SOURCE_REVISION,
        &[ReleaseArtifactInput::new(
            "x86_64-unknown-linux-gnu",
            &archive,
        )],
    )
    .unwrap();
    let checksums = render_sha256sums(&manifest);

    fs::write(&archive, b"tampered").unwrap();

    let error = verify_release_assets(temp.path(), &manifest, &checksums)
        .unwrap_err()
        .to_string();
    assert!(error.contains("checksum mismatch"));
}

#[test]
fn release_metadata_writer_requires_and_verifies_the_complete_platform_set() {
    let temp = tempdir().unwrap();
    for target in SUPPORTED_RELEASE_TARGETS {
        fs::write(
            temp.path().join(target.archive_name("3.2.0")),
            target.triple.as_bytes(),
        )
        .unwrap();
    }

    write_release_metadata(temp.path(), "3.2.0", SOURCE_REVISION).unwrap();

    assert!(temp.path().join("SHA256SUMS").is_file());
    assert!(temp.path().join("release-manifest.json").is_file());
    let manifest = load_and_verify_release_metadata(temp.path()).unwrap();
    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.artifacts.len(), SUPPORTED_RELEASE_TARGETS.len());
    assert!(manifest.update_candidates.is_empty());
}

#[test]
fn schema_two_metadata_includes_one_raw_candidate_per_supported_target() {
    let temp = tempdir().unwrap();
    for target in SUPPORTED_RELEASE_TARGETS {
        fs::write(
            temp.path().join(target.archive_name("3.4.0")),
            format!("archive:{}", target.triple),
        )
        .unwrap();
        fs::write(
            temp.path().join(target.update_candidate_name("3.4.0")),
            format!("candidate:{}", target.triple),
        )
        .unwrap();
    }

    write_release_metadata(temp.path(), "3.4.0", SOURCE_REVISION).unwrap();

    let manifest = load_and_verify_release_metadata(temp.path()).unwrap();
    assert_eq!(manifest.schema_version, 2);
    assert_eq!(
        manifest.update_candidates.len(),
        SUPPORTED_RELEASE_TARGETS.len()
    );
    for target in SUPPORTED_RELEASE_TARGETS {
        assert!(manifest
            .update_candidates
            .iter()
            .any(|candidate| candidate.target == target.triple
                && candidate.name == target.update_candidate_name("3.4.0")
                && candidate.binary == target.binary_name));
    }
    let checksums = fs::read_to_string(temp.path().join("SHA256SUMS")).unwrap();
    assert!(checksums.contains("baron-v3.4.0-x86_64-pc-windows-msvc.exe"));
}

#[test]
fn metadata_rejects_a_partial_raw_candidate_set() {
    let temp = tempdir().unwrap();
    for target in SUPPORTED_RELEASE_TARGETS {
        fs::write(
            temp.path().join(target.archive_name("3.4.0")),
            target.triple.as_bytes(),
        )
        .unwrap();
    }
    fs::write(
        temp.path()
            .join(SUPPORTED_RELEASE_TARGETS[0].update_candidate_name("3.4.0")),
        b"only one raw candidate",
    )
    .unwrap();

    let error = write_release_metadata(temp.path(), "3.4.0", SOURCE_REVISION)
        .unwrap_err()
        .to_string();
    assert!(error.contains("partial raw update candidate set"));
}

#[test]
fn release_metadata_writer_rejects_a_missing_supported_target() {
    let temp = tempdir().unwrap();
    fs::write(
        temp.path().join("baron-v3.2.0-x86_64-pc-windows-msvc.zip"),
        b"windows",
    )
    .unwrap();

    let error = write_release_metadata(temp.path(), "3.2.0", SOURCE_REVISION)
        .unwrap_err()
        .to_string();
    assert!(error.contains("missing release artifact"));
}

#[test]
fn metadata_verification_rejects_tampered_target_identity() {
    let temp = tempdir().unwrap();
    for target in SUPPORTED_RELEASE_TARGETS {
        fs::write(
            temp.path().join(target.archive_name("3.2.0")),
            target.triple.as_bytes(),
        )
        .unwrap();
    }
    write_release_metadata(temp.path(), "3.2.0", SOURCE_REVISION).unwrap();

    let manifest_path = temp.path().join("release-manifest.json");
    let mut json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    json["artifacts"][0]["target"] = json["artifacts"][1]["target"].clone();
    fs::write(
        &manifest_path,
        format!("{}\n", serde_json::to_string_pretty(&json).unwrap()),
    )
    .unwrap();

    let error = load_and_verify_release_metadata(temp.path())
        .unwrap_err()
        .to_string();
    assert!(error.contains("release manifest target set is invalid"));
}

#[test]
fn source_revision_must_be_an_exact_git_commit_sha() {
    let error = build_release_manifest("3.2.0", "abc123", &[])
        .unwrap_err()
        .to_string();

    assert!(error.contains("40-character"));
}

#[test]
fn release_identity_must_match_the_approved_version_and_source() {
    let manifest = build_release_manifest("3.2.0", SOURCE_REVISION, &[]).unwrap();

    verify_release_identity(&manifest, "3.2.0", SOURCE_REVISION).unwrap();
    assert!(verify_release_identity(&manifest, "3.3.0", SOURCE_REVISION)
        .unwrap_err()
        .to_string()
        .contains("version mismatch"));
    assert!(verify_release_identity(
        &manifest,
        "3.2.0",
        "ffffffffffffffffffffffffffffffffffffffff"
    )
    .unwrap_err()
    .to_string()
    .contains("source revision mismatch"));
}
