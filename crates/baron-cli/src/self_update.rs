use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use baron_core::release::{
    load_and_verify_release_metadata, parse_release_manifest, sha256_file,
    update_candidate_for_target, validate_release_manifest, ReleaseManifest, ReleaseUpdateArtifact,
};
use reqwest::blocking::{Client, Response};
use reqwest::{redirect::Policy, Url};
use semver::Version;
use serde::{Deserialize, Serialize};

const MAX_MANIFEST_BYTES: usize = 256 * 1024;
const MAX_CANDIDATE_BYTES: u64 = 128 * 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_REDIRECTS: usize = 3;

pub const DEFAULT_LATEST_MANIFEST_URL: &str =
    "https://github.com/thienty1207/Baron-Engine/releases/latest/download/release-manifest.json";
pub const DEFAULT_RELEASE_BASE_URL: &str =
    "https://github.com/thienty1207/Baron-Engine/releases/download/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateCandidate {
    pub version: String,
    pub source_revision: String,
    pub target: String,
    pub executable_name: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub staged_path: PathBuf,
}

pub trait CandidateSource {
    fn latest_manifest(&self) -> Result<ReleaseManifest>;
    fn fetch_candidate(
        &self,
        manifest: &ReleaseManifest,
        artifact: &ReleaseUpdateArtifact,
        destination: &Path,
    ) -> Result<()>;
}

pub trait CandidateBinaryInspector {
    fn reported_version(&self, candidate_path: &Path) -> Result<String>;
}

pub struct ProcessBinaryInspector;

impl CandidateBinaryInspector for ProcessBinaryInspector {
    fn reported_version(&self, candidate_path: &Path) -> Result<String> {
        let output = Command::new(candidate_path)
            .arg("--version")
            .output()
            .with_context(|| {
                format!(
                    "Could not run staged Baron candidate for version verification: {}",
                    candidate_path.display()
                )
            })?;
        if !output.status.success() {
            bail!(
                "Staged Baron candidate failed its --version check with status {}",
                output.status
            );
        }
        String::from_utf8(output.stdout)
            .context("Staged Baron candidate emitted a non-UTF-8 version response")
            .map(|version| version.trim().to_string())
    }
}

#[derive(Debug, Clone)]
pub struct DirectoryCandidateSource {
    root: PathBuf,
}

impl DirectoryCandidateSource {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl CandidateSource for DirectoryCandidateSource {
    fn latest_manifest(&self) -> Result<ReleaseManifest> {
        load_and_verify_release_metadata(&self.root)
    }

    fn fetch_candidate(
        &self,
        _manifest: &ReleaseManifest,
        artifact: &ReleaseUpdateArtifact,
        destination: &Path,
    ) -> Result<()> {
        validate_candidate_name(&artifact.name)?;
        let source = self.root.join(&artifact.name);
        let metadata = fs::metadata(&source)
            .with_context(|| format!("Raw update candidate is missing: {}", source.display()))?;
        if !metadata.is_file() {
            bail!("Raw update candidate is not a file: {}", source.display());
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&source, destination).with_context(|| {
            format!(
                "Could not stage raw update candidate from {}",
                source.display()
            )
        })?;
        sync_file(destination)
    }
}

#[derive(Debug, Clone)]
pub struct HttpsCandidateSource {
    latest_manifest_url: Url,
    release_base_url: Url,
    approved_hosts: BTreeSet<String>,
}

impl HttpsCandidateSource {
    pub fn github_release() -> Result<Self> {
        Self::new(DEFAULT_LATEST_MANIFEST_URL, DEFAULT_RELEASE_BASE_URL, [])
    }

    pub fn new(
        latest_manifest_url: &str,
        release_base_url: &str,
        trusted_mirror_hosts: impl IntoIterator<Item = String>,
    ) -> Result<Self> {
        let mut approved_hosts = BTreeSet::from([
            "github.com".to_string(),
            "objects.githubusercontent.com".to_string(),
            "github-releases.githubusercontent.com".to_string(),
            "release-assets.githubusercontent.com".to_string(),
        ]);
        approved_hosts.extend(
            trusted_mirror_hosts
                .into_iter()
                .map(|host| host.to_ascii_lowercase()),
        );
        let latest_manifest_url = approved_https_url(latest_manifest_url, &approved_hosts)?;
        let release_base_url = approved_https_url(release_base_url, &approved_hosts)?;
        Ok(Self {
            latest_manifest_url,
            release_base_url,
            approved_hosts,
        })
    }

    fn client() -> Result<Client> {
        Client::builder()
            .https_only(true)
            .redirect(Policy::none())
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .context("Could not create Baron HTTPS update client")
    }

    fn fetch_approved_bytes(&self, initial_url: Url, max_bytes: usize) -> Result<Vec<u8>> {
        let client = Self::client()?;
        let mut current = initial_url;
        for _ in 0..=MAX_REDIRECTS {
            ensure_approved_url(&current, &self.approved_hosts)?;
            let response = client.get(current.clone()).send().with_context(|| {
                format!(
                    "Could not download approved Baron release host {}",
                    host_label(&current)
                )
            })?;
            if response.status().is_redirection() {
                let location = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .context("Baron release redirect did not include a valid Location header")?;
                current = current
                    .join(location)
                    .context("Baron release redirect URL was invalid")?;
                ensure_approved_url(&current, &self.approved_hosts)?;
                continue;
            }
            if !response.status().is_success() {
                bail!(
                    "Baron release host {} returned HTTP {}",
                    host_label(&current),
                    response.status()
                );
            }
            return read_bounded_response(response, max_bytes);
        }
        bail!("Baron release download exceeded the redirect limit")
    }
}

impl CandidateSource for HttpsCandidateSource {
    fn latest_manifest(&self) -> Result<ReleaseManifest> {
        let bytes =
            self.fetch_approved_bytes(self.latest_manifest_url.clone(), MAX_MANIFEST_BYTES)?;
        let content =
            std::str::from_utf8(&bytes).context("Baron release manifest was not UTF-8")?;
        parse_release_manifest(content)
    }

    fn fetch_candidate(
        &self,
        manifest: &ReleaseManifest,
        artifact: &ReleaseUpdateArtifact,
        destination: &Path,
    ) -> Result<()> {
        validate_candidate_name(&artifact.name)?;
        let url = self
            .release_base_url
            .join(&format!("v{}/{}", manifest.version, artifact.name))
            .context("Could not construct Baron release candidate URL")?;
        let bytes = self.fetch_approved_bytes(url, MAX_CANDIDATE_BYTES as usize)?;
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(destination, bytes).with_context(|| {
            format!(
                "Could not stage raw Baron candidate: {}",
                destination.display()
            )
        })?;
        sync_file(destination)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuntimeHandoff {
    UnixAtomic {
        candidate_path: PathBuf,
        installed_binary: PathBuf,
        backup_path: PathBuf,
    },
    WindowsDelayed {
        finalizer_path: PathBuf,
        candidate_path: PathBuf,
        installed_binary: PathBuf,
        backup_path: PathBuf,
        expected_sha256: String,
    },
}

pub fn handoff_label(handoff: &RuntimeHandoff) -> &'static str {
    match handoff {
        RuntimeHandoff::UnixAtomic { .. } => "unix_atomic",
        RuntimeHandoff::WindowsDelayed { .. } => "windows_delayed",
    }
}

pub fn current_release_target() -> Result<&'static str> {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Ok("x86_64-pc-windows-msvc")
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Ok("x86_64-unknown-linux-gnu")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Ok("x86_64-apple-darwin")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Ok("aarch64-apple-darwin")
    } else {
        bail!("This Baron build does not support self-update on the current target")
    }
}

pub fn stage_verified_candidate(
    repo_root: &Path,
    source: &dyn CandidateSource,
    inspector: &dyn CandidateBinaryInspector,
    running_version: &str,
    target: &str,
) -> Result<UpdateCandidate> {
    let manifest = source.latest_manifest()?;
    validate_release_manifest(&manifest)?;
    ensure_upgrade(&manifest.version, running_version)?;
    let artifact = update_candidate_for_target(&manifest, target)?.clone();
    let stage_dir = create_stage_dir(repo_root, &manifest.version)?;
    let staged_path = stage_dir.join(&artifact.name);
    source.fetch_candidate(&manifest, &artifact, &staged_path)?;
    make_candidate_executable(&staged_path)?;
    verify_staged_candidate(&staged_path, &artifact, inspector, &manifest.version)?;
    Ok(UpdateCandidate {
        version: manifest.version,
        source_revision: manifest.source_revision,
        target: artifact.target.clone(),
        executable_name: artifact.binary.clone(),
        sha256: artifact.sha256.clone(),
        size_bytes: artifact.size_bytes,
        staged_path,
    })
}

pub fn prepare_runtime_handoff(
    repo_root: &Path,
    candidate: &UpdateCandidate,
    installed_binary: &Path,
) -> Result<RuntimeHandoff> {
    let update_root = safe_update_workspace(repo_root)?;
    let candidate_path = candidate.staged_path.canonicalize().with_context(|| {
        format!(
            "Verified candidate is no longer available: {}",
            candidate.staged_path.display()
        )
    })?;
    let update_root = update_root.canonicalize().with_context(|| {
        format!(
            "Baron update workspace is unavailable: {}",
            update_root.display()
        )
    })?;
    if !candidate_path.starts_with(&update_root) {
        bail!("Verified candidate escaped the Baron update workspace");
    }
    let backup_path =
        installed_binary.with_extension(format!("baron-backup-{}", candidate.version));
    #[cfg(target_os = "windows")]
    {
        let finalizer_path = candidate_path
            .parent()
            .expect("candidate has a staging parent")
            .join("windows-finalizer.json");
        let handoff = RuntimeHandoff::WindowsDelayed {
            finalizer_path: finalizer_path.clone(),
            candidate_path,
            installed_binary: installed_binary.to_path_buf(),
            backup_path,
            expected_sha256: candidate.sha256.clone(),
        };
        write_json_atomically(&finalizer_path, &handoff)?;
        return Ok(handoff);
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(RuntimeHandoff::UnixAtomic {
            candidate_path,
            installed_binary: installed_binary.to_path_buf(),
            backup_path,
        })
    }
}

#[cfg(not(target_os = "windows"))]
pub fn activate_unix_handoff(handoff: &RuntimeHandoff) -> Result<()> {
    let RuntimeHandoff::UnixAtomic {
        candidate_path,
        installed_binary,
        backup_path,
    } = handoff
    else {
        bail!("The provided runtime handoff is not an atomic Unix handoff");
    };
    if let Some(parent) = installed_binary.parent() {
        fs::create_dir_all(parent)?;
    }
    let had_installed = installed_binary.exists();
    if had_installed {
        fs::rename(installed_binary, backup_path).with_context(|| {
            format!(
                "Could not stage existing Baron binary backup: {}",
                installed_binary.display()
            )
        })?;
    }
    if let Err(error) = fs::rename(candidate_path, installed_binary) {
        if had_installed && backup_path.exists() {
            let _ = fs::rename(backup_path, installed_binary);
        }
        return Err(error)
            .with_context(|| "Could not atomically activate verified Baron candidate");
    }
    Ok(())
}

fn ensure_upgrade(candidate_version: &str, running_version: &str) -> Result<()> {
    let candidate = Version::parse(candidate_version)
        .with_context(|| format!("Candidate version is not valid semver: {candidate_version}"))?;
    let running = Version::parse(running_version)
        .with_context(|| format!("Running Baron version is not valid semver: {running_version}"))?;
    if candidate == running {
        bail!("Baron update candidate is already installed at version {candidate_version}");
    }
    if candidate < running {
        bail!(
            "Baron update candidate {candidate_version} is older than the running version {running_version}"
        );
    }
    Ok(())
}

fn verify_staged_candidate(
    staged_path: &Path,
    artifact: &ReleaseUpdateArtifact,
    inspector: &dyn CandidateBinaryInspector,
    version: &str,
) -> Result<()> {
    let metadata = fs::metadata(staged_path).with_context(|| {
        format!(
            "Staged Baron candidate is missing: {}",
            staged_path.display()
        )
    })?;
    if !metadata.is_file() {
        bail!("Staged Baron candidate is not a file");
    }
    if metadata.len() != artifact.size_bytes {
        bail!(
            "Staged Baron candidate size mismatch: expected {}, got {}",
            artifact.size_bytes,
            metadata.len()
        );
    }
    if metadata.len() > MAX_CANDIDATE_BYTES {
        bail!("Staged Baron candidate exceeds the bounded size limit");
    }
    let checksum = sha256_file(staged_path)?;
    if checksum != artifact.sha256 {
        bail!("Staged Baron candidate checksum mismatch");
    }
    let reported = inspector.reported_version(staged_path)?;
    let expected = format!("baron {version}");
    if reported != expected {
        bail!("Staged Baron candidate reported `{reported}`; expected `{expected}`");
    }
    Ok(())
}

fn create_stage_dir(repo_root: &Path, version: &str) -> Result<PathBuf> {
    let update_root = safe_update_workspace(repo_root)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("System clock is before the Unix epoch")?
        .as_millis();
    for sequence in 0..100 {
        let path = update_root.join(format!(
            "candidate-{version}-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => {
                if is_link_or_reparse_point(&path)? || !fs::metadata(&path)?.is_dir() {
                    bail!(
                        "Baron update stage became a symlink or junction while creating: {}",
                        path.display()
                    );
                }
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("Could not create Baron candidate stage: {}", path.display())
                })
            }
        }
    }
    bail!("Could not allocate a unique Baron candidate staging directory")
}

fn safe_update_workspace(repo_root: &Path) -> Result<PathBuf> {
    let repo_root = repo_root.canonicalize().with_context(|| {
        format!(
            "Could not resolve Baron project root: {}",
            repo_root.display()
        )
    })?;
    if !fs::metadata(&repo_root)?.is_dir() {
        bail!(
            "Baron project root is not a directory: {}",
            repo_root.display()
        );
    }
    let baron_dir = safe_child_directory(&repo_root, ".baron")?;
    safe_child_directory(&baron_dir, "update")
}

fn safe_child_directory(parent: &Path, name: &str) -> Result<PathBuf> {
    let path = parent.join(name);
    if path.exists() {
        if is_link_or_reparse_point(&path)? {
            bail!(
                "Baron update workspace cannot traverse a symlink or junction: {}",
                path.display()
            );
        }
        if !fs::metadata(&path)?.is_dir() {
            bail!(
                "Baron update workspace parent is not a directory: {}",
                path.display()
            );
        }
        return Ok(path);
    }
    match fs::create_dir(&path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "Could not create Baron update workspace directory: {}",
                    path.display()
                )
            })
        }
    }
    if is_link_or_reparse_point(&path)? || !fs::metadata(&path)?.is_dir() {
        bail!(
            "Baron update workspace became a symlink or junction while creating: {}",
            path.display()
        );
    }
    Ok(path)
}

fn is_link_or_reparse_point(path: &Path) -> Result<bool> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(true);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;

        return Ok(metadata.file_attributes() & 0x0400 != 0);
    }
    #[cfg(not(windows))]
    Ok(false)
}

fn read_bounded_response(response: Response, max_bytes: usize) -> Result<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        bail!("Baron release response exceeds the bounded size limit");
    }
    let mut bytes = Vec::new();
    response
        .take(max_bytes as u64 + 1)
        .read_to_end(&mut bytes)
        .context("Could not read Baron release response")?;
    if bytes.len() > max_bytes {
        bail!("Baron release response exceeds the bounded size limit");
    }
    Ok(bytes)
}

fn approved_https_url(raw: &str, approved_hosts: &BTreeSet<String>) -> Result<Url> {
    let url = Url::parse(raw).context("Baron update URL is invalid")?;
    ensure_approved_url(&url, approved_hosts)?;
    Ok(url)
}

fn ensure_approved_url(url: &Url, approved_hosts: &BTreeSet<String>) -> Result<()> {
    if url.scheme() != "https" {
        bail!("Baron update URL must use HTTPS");
    }
    if !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
        bail!("Baron update URL contains unsupported credentials or fragments");
    }
    let host = url
        .host_str()
        .map(str::to_ascii_lowercase)
        .context("Baron update URL has no host")?;
    if !approved_hosts.contains(&host) {
        bail!("Baron update host is not approved: {host}");
    }
    Ok(())
}

fn host_label(url: &Url) -> String {
    url.host_str().unwrap_or("unknown-host").to_string()
}

fn validate_candidate_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || Path::new(name).is_absolute()
    {
        bail!("Baron update candidate name is unsafe");
    }
    Ok(())
}

fn sync_file(path: &Path) -> Result<()> {
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .with_context(|| format!("Could not reopen staged candidate: {}", path.display()))?
        .sync_all()
        .with_context(|| format!("Could not sync staged candidate: {}", path.display()))
}

#[cfg(unix)]
fn make_candidate_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = fs::metadata(path)
        .with_context(|| format!("Could not inspect staged candidate: {}", path.display()))?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(permissions.mode() | 0o700);
    fs::set_permissions(path, permissions).with_context(|| {
        format!(
            "Could not mark staged candidate executable: {}",
            path.display()
        )
    })
}

#[cfg(not(unix))]
fn make_candidate_executable(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(target_os = "windows")]
fn write_json_atomically(path: &Path, handoff: &RuntimeHandoff) -> Result<()> {
    let content = format!("{}\n", serde_json::to_string_pretty(handoff)?);
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, content).with_context(|| {
        format!(
            "Could not write Windows handoff metadata: {}",
            temporary.display()
        )
    })?;
    sync_file(&temporary)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(&temporary, path).with_context(|| {
        format!(
            "Could not activate Windows handoff metadata: {}",
            path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use baron_core::release::{
        supported_release_target, write_release_metadata, SUPPORTED_RELEASE_TARGETS,
    };
    use tempfile::tempdir;

    const SOURCE_REVISION: &str = "0123456789abcdef0123456789abcdef01234567";

    struct StaticInspector(String);

    impl CandidateBinaryInspector for StaticInspector {
        fn reported_version(&self, _candidate_path: &Path) -> Result<String> {
            Ok(self.0.clone())
        }
    }

    struct FixtureSource {
        root: PathBuf,
        manifest: ReleaseManifest,
        corrupt_candidate: bool,
    }

    impl CandidateSource for FixtureSource {
        fn latest_manifest(&self) -> Result<ReleaseManifest> {
            Ok(self.manifest.clone())
        }

        fn fetch_candidate(
            &self,
            _manifest: &ReleaseManifest,
            artifact: &ReleaseUpdateArtifact,
            destination: &Path,
        ) -> Result<()> {
            if self.corrupt_candidate {
                fs::write(destination, b"corrupt")?;
            } else {
                fs::copy(self.root.join(&artifact.name), destination)?;
            }
            Ok(())
        }
    }

    fn complete_release(directory: &Path, version: &str) {
        for target in SUPPORTED_RELEASE_TARGETS {
            fs::write(
                directory.join(target.archive_name(version)),
                format!("archive:{}", target.triple),
            )
            .unwrap();
            fs::write(
                directory.join(target.update_candidate_name(version)),
                format!("candidate:{}", target.triple),
            )
            .unwrap();
        }
        write_release_metadata(directory, version, SOURCE_REVISION).unwrap();
    }

    #[test]
    fn malformed_manifest_identity_is_rejected_before_stage_creation() {
        let temp = tempdir().unwrap();
        let release = temp.path().join("release");
        let repo = temp.path().join("repo");
        fs::create_dir_all(&release).unwrap();
        fs::create_dir_all(&repo).unwrap();
        complete_release(&release, "3.4.0");
        let mut manifest = load_and_verify_release_metadata(&release).unwrap();
        manifest.source_revision = "not-a-commit".to_string();

        let error = stage_verified_candidate(
            &repo,
            &FixtureSource {
                root: release,
                manifest,
                corrupt_candidate: false,
            },
            &StaticInspector("baron 3.4.0".to_string()),
            "3.3.0",
            current_release_target().unwrap(),
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("40-character"));
        assert!(!repo.join(".baron/update").exists());
    }

    #[test]
    fn candidate_byte_tampering_is_rejected_inside_the_update_workspace() {
        let temp = tempdir().unwrap();
        let release = temp.path().join("release");
        let repo = temp.path().join("repo");
        fs::create_dir_all(&release).unwrap();
        fs::create_dir_all(&repo).unwrap();
        complete_release(&release, "3.4.0");
        let manifest = load_and_verify_release_metadata(&release).unwrap();

        let error = stage_verified_candidate(
            &repo,
            &FixtureSource {
                root: release,
                manifest,
                corrupt_candidate: true,
            },
            &StaticInspector("baron 3.4.0".to_string()),
            "3.3.0",
            current_release_target().unwrap(),
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("size mismatch") || error.contains("checksum mismatch"));
        assert!(repo.join(".baron/update").is_dir());
        assert!(!repo.join("AGENTS.md").exists());
    }

    #[test]
    fn directory_candidate_is_verified_and_staged_only_under_project_update_state() {
        let temp = tempdir().unwrap();
        let release = temp.path().join("release");
        let repo = temp.path().join("repo");
        fs::create_dir_all(&release).unwrap();
        fs::create_dir_all(&repo).unwrap();
        complete_release(&release, "3.4.0");

        let candidate = stage_verified_candidate(
            &repo,
            &DirectoryCandidateSource::new(&release),
            &StaticInspector("baron 3.4.0".to_string()),
            "3.3.0",
            current_release_target().unwrap(),
        )
        .unwrap();

        assert_eq!(candidate.version, "3.4.0");
        assert!(candidate.staged_path.is_file());
        let update_root = repo.join(".baron/update").canonicalize().unwrap();
        assert!(
            candidate.staged_path.starts_with(&update_root),
            "candidate path {} was expected below {}",
            candidate.staged_path.display(),
            update_root.display()
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            assert_ne!(
                fs::metadata(&candidate.staged_path)
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o111,
                0,
                "a verified Unix candidate must be executable before --version runs"
            );
        }
        assert!(!repo.join("AGENTS.md").exists());
    }

    #[test]
    fn same_version_candidate_is_rejected_before_stage_creation() {
        let temp = tempdir().unwrap();
        let release = temp.path().join("release");
        let repo = temp.path().join("repo");
        fs::create_dir_all(&release).unwrap();
        fs::create_dir_all(&repo).unwrap();
        complete_release(&release, "3.3.0");

        let error = stage_verified_candidate(
            &repo,
            &DirectoryCandidateSource::new(&release),
            &StaticInspector("baron 3.3.0".to_string()),
            "3.3.0",
            current_release_target().unwrap(),
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("already installed"));
        assert!(!repo.join(".baron/update").exists());
    }

    #[test]
    fn downgrade_candidate_is_rejected_before_stage_creation() {
        let temp = tempdir().unwrap();
        let release = temp.path().join("release");
        let repo = temp.path().join("repo");
        fs::create_dir_all(&release).unwrap();
        fs::create_dir_all(&repo).unwrap();
        complete_release(&release, "3.2.0");

        let error = stage_verified_candidate(
            &repo,
            &DirectoryCandidateSource::new(&release),
            &StaticInspector("baron 3.2.0".to_string()),
            "3.3.0",
            current_release_target().unwrap(),
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("older than the running version"));
        assert!(!repo.join(".baron/update").exists());
    }

    #[test]
    fn unsupported_candidate_target_is_rejected_before_stage_creation() {
        let temp = tempdir().unwrap();
        let release = temp.path().join("release");
        let repo = temp.path().join("repo");
        fs::create_dir_all(&release).unwrap();
        fs::create_dir_all(&repo).unwrap();
        complete_release(&release, "3.4.0");

        let error = stage_verified_candidate(
            &repo,
            &DirectoryCandidateSource::new(&release),
            &StaticInspector("baron 3.4.0".to_string()),
            "3.3.0",
            "unsupported-target",
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("unsupported Baron release target"));
        assert!(!repo.join(".baron/update").exists());
    }

    #[test]
    fn staged_candidate_requires_the_exact_reported_version() {
        let temp = tempdir().unwrap();
        let release = temp.path().join("release");
        let repo = temp.path().join("repo");
        fs::create_dir_all(&release).unwrap();
        fs::create_dir_all(&repo).unwrap();
        complete_release(&release, "3.4.0");

        let error = stage_verified_candidate(
            &repo,
            &DirectoryCandidateSource::new(&release),
            &StaticInspector("baron 3.3.0".to_string()),
            "3.3.0",
            current_release_target().unwrap(),
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("reported"));
        assert!(!repo.join("AGENTS.md").exists());
    }

    #[test]
    fn target_specific_handoff_is_prepared_without_activating_the_runtime() {
        let temp = tempdir().unwrap();
        let release = temp.path().join("release");
        let repo = temp.path().join("repo");
        fs::create_dir_all(&release).unwrap();
        fs::create_dir_all(&repo).unwrap();
        complete_release(&release, "3.4.0");
        let candidate = stage_verified_candidate(
            &repo,
            &DirectoryCandidateSource::new(&release),
            &StaticInspector("baron 3.4.0".to_string()),
            "3.3.0",
            current_release_target().unwrap(),
        )
        .unwrap();
        let installed = repo.join(
            supported_release_target(current_release_target().unwrap())
                .unwrap()
                .binary_name,
        );

        let handoff = prepare_runtime_handoff(&repo, &candidate, &installed).unwrap();

        assert!(!installed.exists());
        #[cfg(target_os = "windows")]
        assert!(matches!(handoff, RuntimeHandoff::WindowsDelayed { .. }));
        #[cfg(not(target_os = "windows"))]
        assert!(matches!(handoff, RuntimeHandoff::UnixAtomic { .. }));
    }

    #[test]
    fn https_source_rejects_non_https_and_untrusted_hosts_without_network_access() {
        let error = HttpsCandidateSource::new(
            "http://github.com/manifest.json",
            DEFAULT_RELEASE_BASE_URL,
            [],
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("HTTPS"));

        let error = HttpsCandidateSource::new(
            "https://example.invalid/manifest.json",
            DEFAULT_RELEASE_BASE_URL,
            [],
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("not approved"));
    }

    #[test]
    fn github_source_keeps_release_download_base_as_a_directory() {
        let source = HttpsCandidateSource::github_release().unwrap();

        assert_eq!(
            source.release_base_url.as_str(),
            "https://github.com/thienty1207/Baron-Engine/releases/download/"
        );
    }

    #[test]
    fn github_release_asset_redirect_host_is_approved_without_network_access() {
        let source = HttpsCandidateSource::new(
            "https://release-assets.githubusercontent.com/manifest.json",
            "https://release-assets.githubusercontent.com/download/",
            [],
        )
        .expect("GitHub release asset redirects must remain an approved release host");

        assert_eq!(
            source.latest_manifest_url.host_str(),
            Some("release-assets.githubusercontent.com")
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn update_workspace_junction_is_rejected_before_candidate_files_are_written() {
        use std::process::Command;

        let temp = tempdir().unwrap();
        let release = temp.path().join("release");
        let repo = temp.path().join("repo");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&release).unwrap();
        fs::create_dir_all(repo.join(".baron")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        complete_release(&release, "3.4.0");
        let script = format!(
            "New-Item -ItemType Junction -Path '{}' -Target '{}' | Out-Null",
            repo.join(".baron/update")
                .display()
                .to_string()
                .replace('\'', "''"),
            outside.display().to_string().replace('\'', "''")
        );
        assert!(Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()
            .unwrap()
            .success());

        let error = stage_verified_candidate(
            &repo,
            &DirectoryCandidateSource::new(&release),
            &StaticInspector("baron 3.4.0".to_string()),
            "3.3.0",
            current_release_target().unwrap(),
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("symlink or junction"));
        assert!(fs::read_dir(&outside).unwrap().next().is_none());
    }
}
