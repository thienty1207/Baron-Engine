use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
pub struct ReleaseManifest {
    pub schema_version: u32,
    pub product: String,
    pub version: String,
    pub source_revision: String,
    pub artifacts: Vec<ReleaseArtifact>,
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
    if source_revision.trim().is_empty() {
        bail!("source revision cannot be empty");
    }

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
        schema_version: 1,
        product: "Baron Engine".to_string(),
        version: version.to_string(),
        source_revision: source_revision.to_string(),
        artifacts,
    })
}

pub fn render_sha256sums(manifest: &ReleaseManifest) -> String {
    manifest
        .artifacts
        .iter()
        .map(|artifact| format!("{}  {}\n", artifact.sha256, artifact.name))
        .collect()
}

pub fn verify_release_assets(
    artifacts_dir: &Path,
    manifest: &ReleaseManifest,
    checksums: &str,
) -> Result<()> {
    let checksum_entries = parse_sha256sums(checksums)?;
    if checksum_entries.len() != manifest.artifacts.len() {
        bail!(
            "checksum entry count mismatch: expected {}, got {}",
            manifest.artifacts.len(),
            checksum_entries.len()
        );
    }

    for artifact in &manifest.artifacts {
        let expected_line_checksum = checksum_entries
            .iter()
            .find(|(_, name)| name == &artifact.name)
            .map(|(checksum, _)| checksum)
            .with_context(|| format!("checksum entry missing for {}", artifact.name))?;
        if expected_line_checksum != &artifact.sha256 {
            bail!("manifest/checksum mismatch for {}", artifact.name);
        }

        let path = artifacts_dir.join(&artifact.name);
        let metadata = fs::metadata(&path)
            .with_context(|| format!("release artifact missing: {}", artifact.name))?;
        if metadata.len() != artifact.size_bytes {
            bail!("release artifact size mismatch for {}", artifact.name);
        }
        let actual = sha256_file(&path)?;
        if actual != artifact.sha256 {
            bail!("checksum mismatch for {}", artifact.name);
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

    let manifest = build_release_manifest(version, source_revision, &inputs)?;
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
    let manifest: ReleaseManifest = serde_json::from_str(
        &fs::read_to_string(&manifest_path)
            .with_context(|| format!("cannot read {}", manifest_path.display()))?,
    )
    .context("invalid release-manifest.json")?;
    if manifest.schema_version != 1 {
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

    let checksums_path = artifacts_dir.join("SHA256SUMS");
    let checksums = fs::read_to_string(&checksums_path)
        .with_context(|| format!("cannot read {}", checksums_path.display()))?;
    verify_release_assets(artifacts_dir, &manifest, &checksums)?;
    Ok(manifest)
}

fn validate_complete_manifest(manifest: &ReleaseManifest) -> Result<()> {
    if manifest.source_revision.trim().is_empty() {
        bail!("release manifest source revision cannot be empty");
    }
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
