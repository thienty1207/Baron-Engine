use std::fs;

use baron_core::release::{
    build_release_manifest, load_and_verify_release_metadata, render_sha256sums,
    verify_release_assets, write_release_metadata, ReleaseArtifactInput, SUPPORTED_RELEASE_TARGETS,
};
use tempfile::tempdir;

#[test]
fn supported_targets_have_stable_native_archive_names() {
    let names = SUPPORTED_RELEASE_TARGETS
        .iter()
        .map(|target| target.archive_name("1.0.0"))
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            "baron-v1.0.0-x86_64-pc-windows-msvc.zip",
            "baron-v1.0.0-x86_64-unknown-linux-gnu.tar.gz",
            "baron-v1.0.0-x86_64-apple-darwin.tar.gz",
            "baron-v1.0.0-aarch64-apple-darwin.tar.gz",
        ]
    );
}

#[test]
fn manifest_and_checksum_output_are_deterministic_and_verifiable() {
    let temp = tempdir().unwrap();
    let windows = temp.path().join("baron-v1.0.0-x86_64-pc-windows-msvc.zip");
    let linux = temp
        .path()
        .join("baron-v1.0.0-x86_64-unknown-linux-gnu.tar.gz");
    fs::write(&windows, b"windows archive").unwrap();
    fs::write(&linux, b"linux archive").unwrap();

    let manifest = build_release_manifest(
        "1.0.0",
        "abc123",
        &[
            ReleaseArtifactInput::new("x86_64-unknown-linux-gnu", &linux),
            ReleaseArtifactInput::new("x86_64-pc-windows-msvc", &windows),
        ],
    )
    .unwrap();

    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.product, "Baron Engine");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.source_revision, "abc123");
    assert_eq!(manifest.artifacts.len(), 2);
    assert_eq!(
        manifest.artifacts[0].name,
        "baron-v1.0.0-x86_64-pc-windows-msvc.zip"
    );
    assert_eq!(
        manifest.artifacts[1].name,
        "baron-v1.0.0-x86_64-unknown-linux-gnu.tar.gz"
    );

    let checksums = render_sha256sums(&manifest);
    assert!(checksums.ends_with('\n'));
    assert!(checksums.contains("  baron-v1.0.0-x86_64-pc-windows-msvc.zip\n"));
    assert!(checksums.contains("  baron-v1.0.0-x86_64-unknown-linux-gnu.tar.gz\n"));

    verify_release_assets(temp.path(), &manifest, &checksums).unwrap();
}

#[test]
fn verification_rejects_an_archive_modified_after_manifest_generation() {
    let temp = tempdir().unwrap();
    let archive = temp
        .path()
        .join("baron-v1.0.0-x86_64-unknown-linux-gnu.tar.gz");
    fs::write(&archive, b"original").unwrap();
    let manifest = build_release_manifest(
        "1.0.0",
        "abc123",
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
            temp.path().join(target.archive_name("1.0.0")),
            target.triple.as_bytes(),
        )
        .unwrap();
    }

    write_release_metadata(temp.path(), "1.0.0", "abc123").unwrap();

    assert!(temp.path().join("SHA256SUMS").is_file());
    assert!(temp.path().join("release-manifest.json").is_file());
    let manifest = load_and_verify_release_metadata(temp.path()).unwrap();
    assert_eq!(manifest.artifacts.len(), SUPPORTED_RELEASE_TARGETS.len());
}

#[test]
fn release_metadata_writer_rejects_a_missing_supported_target() {
    let temp = tempdir().unwrap();
    fs::write(
        temp.path().join("baron-v1.0.0-x86_64-pc-windows-msvc.zip"),
        b"windows",
    )
    .unwrap();

    let error = write_release_metadata(temp.path(), "1.0.0", "abc123")
        .unwrap_err()
        .to_string();
    assert!(error.contains("missing release artifact"));
}

#[test]
fn metadata_verification_rejects_tampered_target_identity() {
    let temp = tempdir().unwrap();
    for target in SUPPORTED_RELEASE_TARGETS {
        fs::write(
            temp.path().join(target.archive_name("1.0.0")),
            target.triple.as_bytes(),
        )
        .unwrap();
    }
    write_release_metadata(temp.path(), "1.0.0", "abc123").unwrap();

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
