use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const RELEASE_MANIFEST_SCHEMA_V1: u32 = 1;
const RELEASE_MANIFEST_SCHEMA_V2: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveKind {
    Zip,
    TarGz,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseTarget {
    pub triple: &'static str,
    pub archive_kind: ArchiveKind,
    pub binary_name: &'static str,
}

impl ReleaseTarget {
    pub fn archive_name(&self, version: &str) -> String {
        let extension = match self.archive_kind {
            ArchiveKind::Zip => "zip",
            ArchiveKind::TarGz => "tar.gz",
        };
        format!("baron-v{version}-{}.{extension}", self.triple)
    }

    pub fn update_candidate_name(&self, version: &str) -> String {
        let extension = if self.binary_name.ends_with(".exe") {
            ".exe"
        } else {
            ""
        };
        format!("baron-v{version}-{}{extension}", self.triple)
    }
}

pub const SUPPORTED_RELEASE_TARGETS: [ReleaseTarget; 4] = [
    ReleaseTarget {
        triple: "x86_64-pc-windows-msvc",
        archive_kind: ArchiveKind::Zip,
        binary_name: "baron.exe",
    },
    ReleaseTarget {
        triple: "x86_64-unknown-linux-gnu",
        archive_kind: ArchiveKind::TarGz,
        binary_name: "baron",
    },
    ReleaseTarget {
        triple: "x86_64-apple-darwin",
        archive_kind: ArchiveKind::TarGz,
        binary_name: "baron",
    },
    ReleaseTarget {
        triple: "aarch64-apple-darwin",
        archive_kind: ArchiveKind::TarGz,
        binary_name: "baron",
    },
];

#[derive(Debug, Clone)]
pub struct ReleaseArtifactInput {
    pub target: String,
    pub path: PathBuf,
}

impl ReleaseArtifactInput {
    pub fn new(target: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            target: target.into(),
            path: path.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseArtifact {
    pub name: String,
    pub target: String,
    pub binary: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseUpdateArtifact {
    pub name: String,
    pub target: String,
    pub binary: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseManifest {
    pub schema_version: u32,
    pub product: String,
    pub version: String,
    pub source_revision: String,
    pub artifacts: Vec<ReleaseArtifact>,
    #[serde(default)]
    pub update_candidates: Vec<ReleaseUpdateArtifact>,
}

pub fn supported_release_target(target: &str) -> Result<ReleaseTarget> {
    SUPPORTED_RELEASE_TARGETS
        .iter()
        .copied()
        .find(|candidate| candidate.triple == target)
        .with_context(|| format!("unsupported Baron release target: {target}"))
}

pub fn build_release_manifest(
    version: &str,
    source_revision: &str,
    inputs: &[ReleaseArtifactInput],
) -> Result<ReleaseManifest> {
    validate_version(version)?;
    validate_source_revision(source_revision)?;

    let mut artifacts = Vec::with_capacity(inputs.len());
    for input in inputs {
        let target = supported_release_target(&input.target)?;
        let expected_name = target.archive_name(version);
        let actual_name = input
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .with_context(|| {
                format!("release artifact has no valid file name: {:?}", input.path)
            })?;
        if actual_name != expected_name {
            bail!(
                "release artifact name mismatch for {}: expected {}, got {}",
                input.target,
                expected_name,
                actual_name
            );
        }
        let metadata = fs::metadata(&input.path)
            .with_context(|| format!("cannot read release artifact: {:?}", input.path))?;
        if !metadata.is_file() {
            bail!("release artifact is not a file: {:?}", input.path);
        }
        artifacts.push(ReleaseArtifact {
            name: expected_name,
            target: target.triple.to_string(),
            binary: target.binary_name.to_string(),
            sha256: sha256_file(&input.path)?,
            size_bytes: metadata.len(),
        });
    }

    artifacts.sort_by_key(|artifact| {
        SUPPORTED_RELEASE_TARGETS
            .iter()
            .position(|target| target.triple == artifact.target)
            .unwrap_or(usize::MAX)
    });

    Ok(ReleaseManifest {
        schema_version: RELEASE_MANIFEST_SCHEMA_V1,
        product: "Baron Engine".to_string(),
        version: version.to_string(),
        source_revision: source_revision.to_string(),
        artifacts,
        update_candidates: Vec::new(),
    })
}

fn build_update_candidates(
    version: &str,
    inputs: &[ReleaseArtifactInput],
) -> Result<Vec<ReleaseUpdateArtifact>> {
    if inputs.len() != SUPPORTED_RELEASE_TARGETS.len() {
        bail!("raw update candidate target set is invalid");
    }
    let mut candidates = Vec::with_capacity(inputs.len());
    let mut seen_targets = BTreeSet::new();
    let mut seen_names = BTreeSet::new();
    for input in inputs {
        let target = supported_release_target(&input.target)
            .map_err(|_| anyhow::anyhow!("raw update candidate target set is invalid"))?;
        let expected_name = target.update_candidate_name(version);
        let actual_name = input
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .with_context(|| {
                format!(
                    "raw update candidate has no valid file name: {:?}",
                    input.path
                )
            })?;
        if actual_name != expected_name
            || !seen_targets.insert(target.triple)
            || !seen_names.insert(actual_name)
        {
            bail!("raw update candidate target set is invalid");
        }
        let metadata = fs::metadata(&input.path)
            .with_context(|| format!("cannot read raw update candidate: {:?}", input.path))?;
        if !metadata.is_file() {
            bail!("raw update candidate is not a file: {:?}", input.path);
        }
        candidates.push(ReleaseUpdateArtifact {
            name: expected_name,
            target: target.triple.to_string(),
            binary: target.binary_name.to_string(),
            sha256: sha256_file(&input.path)?,
            size_bytes: metadata.len(),
        });
    }
    candidates.sort_by_key(|candidate| {
        SUPPORTED_RELEASE_TARGETS
            .iter()
            .position(|target| target.triple == candidate.target)
            .unwrap_or(usize::MAX)
    });
    Ok(candidates)
}

pub fn render_sha256sums(manifest: &ReleaseManifest) -> String {
    manifest
        .artifacts
        .iter()
        .map(|artifact| (&artifact.sha256, &artifact.name))
        .chain(
            manifest
                .update_candidates
                .iter()
                .map(|candidate| (&candidate.sha256, &candidate.name)),
        )
        .map(|(sha256, name)| format!("{sha256}  {name}\n"))
        .collect()
}

pub fn verify_release_assets(
    artifacts_dir: &Path,
    manifest: &ReleaseManifest,
    checksums: &str,
) -> Result<()> {
    let checksum_entries = parse_sha256sums(checksums)?;
    let expected = manifest
        .artifacts
        .iter()
        .map(|artifact| (&artifact.name, &artifact.sha256, artifact.size_bytes))
        .chain(
            manifest
                .update_candidates
                .iter()
                .map(|candidate| (&candidate.name, &candidate.sha256, candidate.size_bytes)),
        )
        .collect::<Vec<_>>();
    if checksum_entries.len() != expected.len() {
        bail!(
            "checksum entry count mismatch: expected {}, got {}",
            expected.len(),
            checksum_entries.len()
        );
    }

    for (name, expected_sha256, expected_size) in expected {
        let expected_line_checksum = checksum_entries
            .iter()
            .find(|(_, checksum_name)| checksum_name == name)
            .map(|(checksum, _)| checksum)
            .with_context(|| format!("checksum entry missing for {name}"))?;
        if expected_line_checksum != expected_sha256 {
            bail!("manifest/checksum mismatch for {name}");
        }

        let path = artifacts_dir.join(name);
        let metadata =
            fs::metadata(&path).with_context(|| format!("release artifact missing: {name}"))?;
        if metadata.len() != expected_size {
            bail!("release artifact size mismatch for {name}");
        }
        let actual = sha256_file(&path)?;
        if actual != *expected_sha256 {
            bail!("checksum mismatch for {name}");
        }
    }
    Ok(())
}

pub fn write_release_metadata(
    artifacts_dir: &Path,
    version: &str,
    source_revision: &str,
) -> Result<ReleaseManifest> {
    let mut inputs = Vec::with_capacity(SUPPORTED_RELEASE_TARGETS.len());
    for target in SUPPORTED_RELEASE_TARGETS {
        let path = artifacts_dir.join(target.archive_name(version));
        if !path.is_file() {
            bail!("missing release artifact: {}", path.display());
        }
        inputs.push(ReleaseArtifactInput::new(target.triple, path));
    }

    let mut manifest = build_release_manifest(version, source_revision, &inputs)?;
    let candidate_inputs = SUPPORTED_RELEASE_TARGETS
        .iter()
        .map(|target| {
            ReleaseArtifactInput::new(
                target.triple,
                artifacts_dir.join(target.update_candidate_name(version)),
            )
        })
        .collect::<Vec<_>>();
    let candidate_files = candidate_inputs
        .iter()
        .filter(|input| input.path.exists())
        .count();
    if candidate_files != 0 && candidate_files != SUPPORTED_RELEASE_TARGETS.len() {
        bail!("partial raw update candidate set is not allowed");
    }
    if candidate_files == SUPPORTED_RELEASE_TARGETS.len() {
        manifest.schema_version = RELEASE_MANIFEST_SCHEMA_V2;
        manifest.update_candidates = build_update_candidates(version, &candidate_inputs)?;
    }
    let mut manifest_json = serde_json::to_string_pretty(&manifest)?;
    manifest_json.push('\n');
    fs::write(artifacts_dir.join("release-manifest.json"), manifest_json)
        .context("cannot write release-manifest.json")?;
    fs::write(
        artifacts_dir.join("SHA256SUMS"),
        render_sha256sums(&manifest),
    )
    .context("cannot write SHA256SUMS")?;
    Ok(manifest)
}

pub fn load_and_verify_release_metadata(artifacts_dir: &Path) -> Result<ReleaseManifest> {
    let manifest_path = artifacts_dir.join("release-manifest.json");
    let manifest = parse_release_manifest(
        &fs::read_to_string(&manifest_path)
            .with_context(|| format!("cannot read {}", manifest_path.display()))?,
    )?;
    let checksums_path = artifacts_dir.join("SHA256SUMS");
    let checksums = fs::read_to_string(&checksums_path)
        .with_context(|| format!("cannot read {}", checksums_path.display()))?;
    verify_release_assets(artifacts_dir, &manifest, &checksums)?;
    Ok(manifest)
}

pub fn parse_release_manifest(content: &str) -> Result<ReleaseManifest> {
    let manifest: ReleaseManifest =
        serde_json::from_str(content).context("invalid release-manifest.json")?;
    validate_release_manifest(&manifest)?;
    Ok(manifest)
}

pub fn validate_release_manifest(manifest: &ReleaseManifest) -> Result<()> {
    if !matches!(
        manifest.schema_version,
        RELEASE_MANIFEST_SCHEMA_V1 | RELEASE_MANIFEST_SCHEMA_V2
    ) {
        bail!(
            "unsupported release manifest schema: {}",
            manifest.schema_version
        );
    }
    if manifest.product != "Baron Engine" {
        bail!("release manifest product is not Baron Engine");
    }
    validate_version(&manifest.version)?;
    validate_complete_manifest(&manifest)?;
    Ok(())
}

pub fn update_candidate_for_target<'a>(
    manifest: &'a ReleaseManifest,
    target: &str,
) -> Result<&'a ReleaseUpdateArtifact> {
    if manifest.schema_version != RELEASE_MANIFEST_SCHEMA_V2 {
        bail!("release manifest does not contain raw self-update candidates")
    }
    supported_release_target(target)?;
    manifest
        .update_candidates
        .iter()
        .find(|candidate| candidate.target == target)
        .with_context(|| format!("release manifest is missing a raw update candidate for {target}"))
}

pub fn verify_release_identity(
    manifest: &ReleaseManifest,
    expected_version: &str,
    expected_source_revision: &str,
) -> Result<()> {
    validate_version(expected_version)?;
    validate_source_revision(expected_source_revision)?;
    if manifest.version != expected_version {
        bail!(
            "release version mismatch: expected {}, got {}",
            expected_version,
            manifest.version
        );
    }
    if !manifest
        .source_revision
        .eq_ignore_ascii_case(expected_source_revision)
    {
        bail!(
            "release source revision mismatch: expected {}, got {}",
            expected_source_revision,
            manifest.source_revision
        );
    }
    Ok(())
}

fn validate_complete_manifest(manifest: &ReleaseManifest) -> Result<()> {
    validate_source_revision(&manifest.source_revision)?;
    if manifest.artifacts.len() != SUPPORTED_RELEASE_TARGETS.len() {
        bail!("release manifest target set is invalid");
    }

    let mut seen_targets = BTreeSet::new();
    let mut seen_names = BTreeSet::new();
    for artifact in &manifest.artifacts {
        let target = supported_release_target(&artifact.target)
            .map_err(|_| anyhow::anyhow!("release manifest target set is invalid"))?;
        if !seen_targets.insert(artifact.target.as_str())
            || !seen_names.insert(artifact.name.as_str())
            || artifact.name != target.archive_name(&manifest.version)
            || artifact.binary != target.binary_name
        {
            bail!("release manifest target set is invalid");
        }
    }
    if SUPPORTED_RELEASE_TARGETS
        .iter()
        .any(|target| !seen_targets.contains(target.triple))
    {
        bail!("release manifest target set is invalid");
    }
    match manifest.schema_version {
        RELEASE_MANIFEST_SCHEMA_V1 if manifest.update_candidates.is_empty() => Ok(()),
        RELEASE_MANIFEST_SCHEMA_V1 => {
            bail!("schema 1 release manifest cannot include raw update candidates")
        }
        RELEASE_MANIFEST_SCHEMA_V2 => validate_update_candidate_set(manifest),
        _ => bail!(
            "unsupported release manifest schema: {}",
            manifest.schema_version
        ),
    }
}

fn validate_update_candidate_set(manifest: &ReleaseManifest) -> Result<()> {
    if manifest.update_candidates.len() != SUPPORTED_RELEASE_TARGETS.len() {
        bail!("release manifest update candidate target set is invalid");
    }
    let mut seen_targets = BTreeSet::new();
    let mut seen_names = BTreeSet::new();
    for candidate in &manifest.update_candidates {
        let target = supported_release_target(&candidate.target).map_err(|_| {
            anyhow::anyhow!("release manifest update candidate target set is invalid")
        })?;
        if !seen_targets.insert(candidate.target.as_str())
            || !seen_names.insert(candidate.name.as_str())
            || candidate.name != target.update_candidate_name(&manifest.version)
            || candidate.binary != target.binary_name
            || candidate.sha256.len() != 64
            || !candidate
                .sha256
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            bail!("release manifest update candidate target set is invalid");
        }
    }
    if SUPPORTED_RELEASE_TARGETS
        .iter()
        .any(|target| !seen_targets.contains(target.triple))
    {
        bail!("release manifest update candidate target set is invalid");
    }
    Ok(())
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file =
        fs::File::open(path).with_context(|| format!("cannot open file for checksum: {path:?}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("cannot read file for checksum: {path:?}"))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn validate_version(version: &str) -> Result<()> {
    let parts = version.split('.').collect::<Vec<_>>();
    if parts.len() != 3
        || parts
            .iter()
            .any(|part| part.is_empty() || part.parse::<u64>().is_err())
    {
        bail!("release version must use numeric major.minor.patch form");
    }
    Ok(())
}

fn validate_source_revision(source_revision: &str) -> Result<()> {
    if source_revision.len() != 40
        || !source_revision
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        bail!("source revision must be a 40-character hexadecimal Git commit SHA");
    }
    Ok(())
}

fn parse_sha256sums(content: &str) -> Result<Vec<(String, String)>> {
    let mut entries = Vec::new();
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        let (checksum, name) = line
            .split_once("  ")
            .with_context(|| format!("invalid SHA256SUMS line: {line}"))?;
        if checksum.len() != 64
            || !checksum
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            bail!("invalid SHA-256 value for {name}");
        }
        if name.contains('/') || name.contains('\\') || name.contains("..") {
            bail!("unsafe artifact name in SHA256SUMS: {name}");
        }
        entries.push((checksum.to_ascii_lowercase(), name.to_string()));
    }
    Ok(entries)
}
