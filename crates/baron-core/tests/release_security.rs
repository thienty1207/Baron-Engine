use std::fs;

use baron_core::release::{
    build_release_manifest, canonical_release_manifest_bytes, compiled_trusted_release_keys,
    decode_release_signing_seed, load_and_verify_release_metadata,
    load_and_verify_release_metadata_with_keys, sign_release_manifest,
    verify_signed_release_manifest, write_release_metadata_with_signing_key, ReleaseArtifactInput,
    ReleaseSignatureV1, TrustedReleaseKey, SUPPORTED_RELEASE_TARGETS,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use tempfile::tempdir;

const SOURCE_REVISION: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn release_signing_secret_uses_base64_raw_seed_encoding() {
    let encoded = STANDARD.encode([7_u8; 32]);
    assert_eq!(decode_release_signing_seed(&encoded).unwrap(), [7_u8; 32]);
    assert!(decode_release_signing_seed(&STANDARD.encode([7_u8; 31])).is_err());
    assert!(decode_release_signing_seed(&"07".repeat(32)).is_err());
}

fn seed() -> [u8; 32] {
    // Ephemeral test material only. No production release private key is stored here.
    [7_u8; 32]
}

fn keys(seed: &[u8; 32]) -> Vec<TrustedReleaseKey> {
    let signing_key = SigningKey::from_bytes(seed);
    vec![TrustedReleaseKey::new(
        "baron-debug-test-key",
        signing_key.verifying_key().to_bytes(),
    )]
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn complete_release(path: &std::path::Path, version: &str) {
    for target in SUPPORTED_RELEASE_TARGETS {
        fs::write(
            path.join(target.archive_name(version)),
            target.triple.as_bytes(),
        )
        .unwrap();
    }
}

fn signed_fixture(path: &std::path::Path, version: &str) -> Vec<TrustedReleaseKey> {
    complete_release(path, version);
    let signing_seed = seed();
    write_release_metadata_with_signing_key(
        path,
        version,
        SOURCE_REVISION,
        "baron-debug-test-key",
        &signing_seed,
    )
    .unwrap();
    keys(&signing_seed)
}

#[test]
fn compiled_production_release_key_matches_the_pinned_identity() {
    let key = compiled_trusted_release_keys()
        .into_iter()
        .find(|key| key.key_id == "baron-release-2026")
        .expect("production release key must be compiled into Baron");
    assert_eq!(
        STANDARD.encode(key.public_key),
        "SBgrmtK5emhgD5e6UW664fXR3qogCjVPHtrxeVk2TcA="
    );
    let fingerprint = Sha256::digest(key.public_key);
    assert_eq!(
        format!("{fingerprint:x}"),
        "73a005a12cf79f1fa60612f0e359a13c83d2660806075b610b3d83f4f14c31b4"
    );
}

#[test]
fn authentic_manifest_verifies_before_artifact_selection_and_hash_check() {
    let temp = tempdir().unwrap();
    let trusted = signed_fixture(temp.path(), "4.2.3");
    let manifest = load_and_verify_release_metadata_with_keys(temp.path(), &trusted).unwrap();
    assert_eq!(manifest.version, "4.2.3");
    assert_eq!(manifest.release_identity, "github:thienty1207/Baron-Engine");
    assert_eq!(manifest.minimum_compatible_version, "4.2.3");
    assert_eq!(manifest.artifacts.len(), SUPPORTED_RELEASE_TARGETS.len());
}

#[test]
fn trusted_signature_cannot_authorize_a_different_release_identity() {
    let temp = tempdir().unwrap();
    let trusted = signed_fixture(temp.path(), "4.2.3");
    let mut manifest: baron_core::release::ReleaseManifest =
        serde_json::from_slice(&fs::read(temp.path().join("release-manifest.json")).unwrap())
            .unwrap();
    manifest.release_identity = "github:attacker/Baron-Engine".to_string();
    let raw = serde_json::to_vec(&manifest).unwrap();
    let signing_key = SigningKey::from_bytes(&seed());
    let mut signed = b"baron-release-manifest-v1\0".to_vec();
    signed.extend_from_slice(&raw);
    let signature = signing_key.sign(&signed);
    let signature_json = serde_json::to_string(&ReleaseSignatureV1 {
        schema_version: 1,
        key_id: "baron-debug-test-key".to_string(),
        signature: hex(&signature.to_bytes()),
    })
    .unwrap();
    let error = verify_signed_release_manifest(
        std::str::from_utf8(&raw).unwrap(),
        &signature_json,
        &trusted,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("release manifest identity mismatch"));
}

#[test]
fn canonical_manifest_bytes_are_deterministic_and_layout_changes_fail_authentication() {
    let temp = tempdir().unwrap();
    let trusted = signed_fixture(temp.path(), "4.2.3");
    let manifest_path = temp.path().join("release-manifest.json");
    let signature = fs::read_to_string(temp.path().join("release-manifest.sig")).unwrap();
    let original = fs::read_to_string(&manifest_path).unwrap();
    let payload: baron_core::release::ReleaseManifest = serde_json::from_str(&original).unwrap();
    assert!(!canonical_release_manifest_bytes(&payload)
        .unwrap()
        .is_empty());

    let compact = serde_json::to_string(&payload).unwrap();
    let pretty_crlf = serde_json::to_string_pretty(&payload)
        .unwrap()
        .replace('\n', "\r\n");
    assert!(verify_signed_release_manifest(&compact, &signature, &trusted).is_ok());
    assert!(verify_signed_release_manifest(&pretty_crlf, &signature, &trusted).is_err());

    let mut reordered = serde_json::Map::new();
    if let serde_json::Value::Object(manifest) = serde_json::from_str(&original).unwrap() {
        for (name, value) in manifest.iter().rev() {
            reordered.insert(name.clone(), value.clone());
        }
    }
    assert!(verify_signed_release_manifest(
        &serde_json::to_string(&serde_json::Value::Object(reordered)).unwrap(),
        &signature,
        &trusted,
    )
    .is_err());

    let duplicate = format!(
        "{}{}",
        original.trim_end_matches('}'),
        ",\"version\":\"4.2.3\"}"
    );
    assert!(verify_signed_release_manifest(&duplicate, &signature, &trusted).is_err());

    let mut unknown = serde_json::from_str::<serde_json::Value>(&original).unwrap();
    unknown["unexpected_security_field"] = serde_json::json!(true);
    assert!(verify_signed_release_manifest(
        &serde_json::to_string(&unknown).unwrap(),
        &signature,
        &trusted,
    )
    .is_err());
}

fn assert_tamper_rejected(mutator: impl FnOnce(&mut serde_json::Value)) {
    let temp = tempdir().unwrap();
    let trusted = signed_fixture(temp.path(), "4.2.3");
    let path = temp.path().join("release-manifest.json");
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    mutator(&mut value);
    fs::write(&path, serde_json::to_string(&value).unwrap()).unwrap();
    let error = load_and_verify_release_metadata_with_keys(temp.path(), &trusted)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("signature verification failed")
            || error.contains("signature is not")
            || error.contains("invalid release-manifest")
            || error.contains("canonical"),
        "unexpected authentication error: {error}"
    );
}

#[test]
fn manifest_version_digest_identity_and_size_tampering_fail_closed() {
    assert_tamper_rejected(|value| value["version"] = serde_json::json!("9.9.9"));
    assert_tamper_rejected(|value| {
        value["artifacts"][0]["sha256"] = serde_json::json!("0".repeat(64))
    });
    assert_tamper_rejected(|value| {
        value["release_identity"] = serde_json::json!("https://attacker.invalid")
    });
    assert_tamper_rejected(|value| {
        value["artifacts"][0]["size_bytes"] = serde_json::json!(999999_u64)
    });
}

#[test]
fn signature_tampering_missing_signature_and_attacker_key_fail_without_mutation() {
    let temp = tempdir().unwrap();
    let trusted = signed_fixture(temp.path(), "4.2.3");
    let manifest_path = temp.path().join("release-manifest.json");
    let signature_path = temp.path().join("release-manifest.sig");
    let checksums_path = temp.path().join("SHA256SUMS");
    let before_manifest = fs::read(&manifest_path).unwrap();
    let before_signature = fs::read(&signature_path).unwrap();
    let before_checksums = fs::read(&checksums_path).unwrap();

    let manifest: baron_core::release::ReleaseManifest =
        serde_json::from_slice(&before_manifest).unwrap();
    let attacker = SigningKey::from_bytes(&[9_u8; 32]);
    let signature = attacker.sign(&canonical_release_manifest_bytes(&manifest).unwrap());
    fs::write(
        &signature_path,
        serde_json::to_vec(&ReleaseSignatureV1 {
            schema_version: 1,
            key_id: "attacker-key".to_string(),
            signature: hex(&signature.to_bytes()),
        })
        .unwrap(),
    )
    .unwrap();
    let error = load_and_verify_release_metadata(temp.path())
        .unwrap_err()
        .to_string();
    assert!(error.contains("unknown release signing key ID"));

    fs::write(&signature_path, b"{}").unwrap();
    assert!(load_and_verify_release_metadata(temp.path()).is_err());
    fs::remove_file(&signature_path).unwrap();
    assert!(load_and_verify_release_metadata(temp.path()).is_err());
    fs::write(&manifest_path, &before_manifest).unwrap();
    fs::write(&signature_path, &before_signature).unwrap();
    assert_eq!(fs::read(&manifest_path).unwrap(), before_manifest);
    assert_eq!(fs::read(&signature_path).unwrap(), before_signature);
    assert_eq!(fs::read(&checksums_path).unwrap(), before_checksums);
    assert_eq!(trusted.len(), 1);
}

#[test]
fn signed_manifest_builder_rejects_invalid_payload_before_signing() {
    let temp = tempdir().unwrap();
    let archive = temp
        .path()
        .join("baron-v4.2.3-x86_64-unknown-linux-gnu.tar.gz");
    fs::write(&archive, b"archive").unwrap();
    let manifest = build_release_manifest(
        "4.2.3",
        SOURCE_REVISION,
        &[ReleaseArtifactInput::new(
            "x86_64-unknown-linux-gnu",
            archive,
        )],
    )
    .unwrap();
    assert!(sign_release_manifest(&manifest, "baron-debug-test-key", &seed()).is_err());
}
