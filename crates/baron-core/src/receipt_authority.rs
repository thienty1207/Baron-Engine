//! Machine-local Ed25519 authority for trusted execution receipts.
//!
//! The private seed is deliberately kept outside repositories and Vaults. A
//! receipt signed by this key is authoritative across process boundaries on
//! the machine that owns the seed; losing or regenerating the seed makes
//! older receipts diagnostic.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

use crate::config::machine_config_path;
use crate::safe_io::{ensure_directory_chain, read_bytes};

const SEED_BYTES: usize = 32;
const SEED_RELATIVE_PATH: &str = "authority/execution-receipt-ed25519.seed";
const KEY_ID_PREFIX: &str = "ed25519-";
const MAX_STAGING_ATTEMPTS: usize = 128;
const LEGACY_STAGING_EXTENSION: &str = "baron-tmp";
static STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) struct ReceiptAuthority {
    pub(crate) key_id: String,
    signing_key: SigningKey,
}

impl ReceiptAuthority {
    pub(crate) fn load_or_create_for_project(
        repo_root: &Path,
        vault_root: Option<&Path>,
    ) -> Result<Self> {
        let path = receipt_authority_seed_path_for_project(repo_root, vault_root)?;
        let parent = path
            .parent()
            .context("Receipt authority seed has no parent directory")?;
        ensure_directory_chain(parent)?;

        match fs::symlink_metadata(&path) {
            Ok(_) => return Ok(Self::from_seed(read_seed(&path)?)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "Could not inspect receipt authority seed: {}",
                        path.display()
                    )
                })
            }
        }

        let mut seed = [0_u8; SEED_BYTES];
        getrandom::getrandom(&mut seed).map_err(|error| {
            anyhow::anyhow!("Could not generate receipt authority seed: {error}")
        })?;
        let seed = create_seed_without_replacement(&path, &seed)?;
        Ok(Self::from_seed(seed))
    }

    pub(crate) fn load_existing_for_project(
        repo_root: &Path,
        vault_root: Option<&Path>,
    ) -> Result<Self> {
        let path = receipt_authority_seed_path_for_project(repo_root, vault_root)?;
        let seed = read_seed(&path)?;
        Ok(Self::from_seed(seed))
    }

    pub(crate) fn sign(&self, payload: &[u8]) -> String {
        hex_encode(self.signing_key.sign(payload).to_bytes())
    }

    pub(crate) fn verify_for_project(
        repo_root: &Path,
        vault_root: Option<&Path>,
        key_id: &str,
        signature: &str,
        payload: &[u8],
    ) -> Result<()> {
        let authority = Self::load_existing_for_project(repo_root, vault_root)?;
        if authority.key_id != key_id {
            bail!("receipt authority key ID `{key_id}` does not match this machine authority");
        }
        let signature_bytes = decode_fixed_hex::<64>(signature)
            .context("receipt authority signature is not a 64-byte hexadecimal signature")?;
        authority
            .signing_key
            .verifying_key()
            .verify(payload, &Signature::from_bytes(&signature_bytes))
            .context("receipt authority signature verification failed")
    }

    pub(crate) fn key_id(&self) -> &str {
        &self.key_id
    }

    fn from_seed(seed: [u8; SEED_BYTES]) -> Self {
        let signing_key = SigningKey::from_bytes(&seed);
        let key_id = authority_key_id(&signing_key.verifying_key());
        Self {
            key_id,
            signing_key,
        }
    }
}

fn receipt_authority_seed_path_for_project(
    repo_root: &Path,
    vault_root: Option<&Path>,
) -> Result<PathBuf> {
    let repo_root = canonical_boundary_path(repo_root)?;
    let vault_root = vault_root.map(canonical_boundary_path).transpose()?;
    let machine_config = machine_config_path()?;
    let machine_home = machine_config
        .parent()
        .context("Baron machine config has no parent directory")?;
    let machine_home = boundary_path(machine_home)?;
    let seed_path = machine_home.join(SEED_RELATIVE_PATH);
    reject_unsafe_existing_components(&seed_path)?;
    let seed_boundary = canonical_boundary_path(&seed_path)?;
    if is_within_boundary(&seed_boundary, &repo_root) {
        bail!(
            "Receipt authority seed must be outside the repository boundary: {}",
            seed_path.display()
        );
    }
    if vault_root
        .as_ref()
        .is_some_and(|vault| is_within_boundary(&seed_boundary, vault))
    {
        bail!(
            "Receipt authority seed must be outside the Vault boundary: {}",
            seed_path.display()
        );
    }
    Ok(seed_path)
}

fn read_seed(path: &Path) -> Result<[u8; SEED_BYTES]> {
    let bytes = read_bytes(path)?.with_context(|| {
        format!(
            "Receipt authority seed is missing: {}; initialize it through Baron",
            path.display()
        )
    })?;
    if bytes.len() != SEED_BYTES {
        bail!(
            "Receipt authority seed must be exactly {SEED_BYTES} bytes; found {}",
            bytes.len()
        );
    }
    let mut seed = [0_u8; SEED_BYTES];
    seed.copy_from_slice(&bytes);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::symlink_metadata(path)?.permissions().mode();
        if mode & 0o077 != 0 {
            bail!("Receipt authority seed permissions must be owner-only");
        }
    }
    Ok(seed)
}

fn set_private_permissions(file: &std::fs::File, path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = file
            .metadata()
            .with_context(|| {
                format!(
                    "Could not inspect receipt authority seed: {}",
                    path.display()
                )
            })?
            .permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(path, permissions).with_context(|| {
            format!(
                "Could not restrict receipt authority seed: {}",
                path.display()
            )
        })?;
    }
    #[cfg(not(unix))]
    {
        let _ = (file, path);
    }
    Ok(())
}

fn create_seed_without_replacement(
    path: &Path,
    seed: &[u8; SEED_BYTES],
) -> Result<[u8; SEED_BYTES]> {
    let legacy = path.with_extension(LEGACY_STAGING_EXTENSION);
    match fs::symlink_metadata(&legacy) {
        Ok(_) => {
            bail!(
                "Receipt authority staging collision at {}; refusing to touch the existing file",
                legacy.display()
            )
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "Could not inspect receipt authority staging path: {}",
                    legacy.display()
                )
            })
        }
    }

    let parent = path
        .parent()
        .context("Receipt authority seed has no parent directory")?;
    for _ in 0..MAX_STAGING_ATTEMPTS {
        let staging = next_staging_path(path);
        let mut create = OpenOptions::new();
        create.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut create, 0o600);
        let mut file = match create.open(&staging) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "Could not create receipt authority staging file: {}",
                        staging.display()
                    )
                })
            }
        };

        let result = (|| {
            file.write_all(seed)
                .context("Could not write receipt authority staging seed")?;
            file.flush()
                .context("Could not flush receipt authority staging seed")?;
            file.sync_all()
                .context("Could not sync receipt authority staging seed")?;
            set_private_permissions(&file, &staging)?;
            file.sync_all()
                .context("Could not sync receipt authority staging permissions")?;
            match fs::hard_link(&staging, path) {
                Ok(()) => {
                    sync_parent_directory(parent)?;
                    Ok(true)
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
                Err(error) => Err(error).with_context(|| {
                    format!(
                        "Could not activate receipt authority seed without replacement: {}",
                        path.display()
                    )
                }),
            }
        })();

        let activated = match result {
            Ok(activated) => activated,
            Err(error) => {
                drop(file);
                let _ = fs::remove_file(&staging);
                return Err(error);
            }
        };
        drop(file);
        let _ = fs::remove_file(&staging);
        if activated {
            return Ok(*seed);
        }
        return read_seed(path);
    }

    bail!(
        "Could not allocate a unique receipt authority staging file in {}",
        parent.display()
    )
}

fn next_staging_path(path: &Path) -> PathBuf {
    let sequence = STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let process = std::process::id();
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("execution-receipt-ed25519.seed");
    path.with_file_name(format!(".{name}.{process}.{timestamp}.{sequence}.stage"))
}

fn sync_parent_directory(parent: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        fs::File::open(parent)
            .with_context(|| {
                format!(
                    "Could not open receipt authority parent for sync: {}",
                    parent.display()
                )
            })?
            .sync_all()
            .with_context(|| {
                format!(
                    "Could not sync receipt authority parent: {}",
                    parent.display()
                )
            })?;
    }
    #[cfg(not(unix))]
    let _ = parent;
    Ok(())
}

fn boundary_path(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            std::path::Component::RootDir => normalized.push(std::path::MAIN_SEPARATOR_STR),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    bail!("Path escapes its absolute boundary: {}", path.display());
                }
            }
            std::path::Component::Normal(part) => normalized.push(part),
        }
    }
    Ok(normalized)
}

fn canonical_boundary_path(path: &Path) -> Result<PathBuf> {
    let normalized = boundary_path(path)?;
    let mut existing = normalized.clone();
    let mut missing = Vec::new();
    loop {
        match fs::symlink_metadata(&existing) {
            Ok(_) => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let component = existing.file_name().with_context(|| {
                    format!(
                        "Could not resolve an existing ancestor for authority boundary: {}",
                        path.display()
                    )
                })?;
                missing.push(component.to_os_string());
                if !existing.pop() {
                    bail!(
                        "Could not resolve an existing ancestor for authority boundary: {}",
                        path.display()
                    );
                }
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "Could not inspect authority boundary path: {}",
                        path.display()
                    )
                })
            }
        }
    }

    let mut canonical = existing.canonicalize().with_context(|| {
        format!(
            "Could not canonicalize authority boundary ancestor: {}",
            existing.display()
        )
    })?;
    for component in missing.iter().rev() {
        canonical.push(component);
    }
    boundary_path(&canonical)
}

fn is_within_boundary(candidate: &Path, root: &Path) -> bool {
    let candidate = boundary_key(candidate);
    let root = boundary_key(root);
    candidate == root
        || if root.ends_with('/') {
            candidate.starts_with(&root)
        } else {
            candidate.starts_with(&(root + "/"))
        }
}

fn boundary_key(path: &Path) -> String {
    let mut value = path.to_string_lossy().replace('\\', "/");
    #[cfg(windows)]
    {
        value = value.to_ascii_lowercase();
        if let Some(stripped) = value.strip_prefix("//?/unc/") {
            value = format!("//{stripped}");
        } else if let Some(stripped) = value.strip_prefix("//?/") {
            value = stripped.to_string();
        }
    }
    while value.ends_with('/') && value.len() > 1 {
        value.pop();
    }
    value
}

fn reject_unsafe_existing_components(path: &Path) -> Result<()> {
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Prefix(prefix) => current.push(prefix.as_os_str()),
            std::path::Component::RootDir => current.push(std::path::MAIN_SEPARATOR_STR),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => bail!(
                "Receipt authority path contains a parent traversal: {}",
                path.display()
            ),
            std::path::Component::Normal(part) => {
                current.push(part);
                match fs::symlink_metadata(&current) {
                    Ok(metadata)
                        if metadata.file_type().is_symlink() || is_reparse_point(&metadata) =>
                    {
                        bail!(
                            "Receipt authority path cannot traverse a symlink or reparse point: {}",
                            current.display()
                        )
                    }
                    Ok(_) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                    Err(error) => {
                        return Err(error).with_context(|| {
                            format!(
                                "Could not inspect receipt authority path component: {}",
                                current.display()
                            )
                        })
                    }
                }
            }
        }
    }
    Ok(())
}

fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x0400 != 0
    }
    #[cfg(not(windows))]
    {
        let _ = metadata;
        false
    }
}

fn authority_key_id(key: &VerifyingKey) -> String {
    format!(
        "{KEY_ID_PREFIX}{}",
        hex_encode(Sha256::digest(key.to_bytes()))
    )
}

pub(crate) fn hex_encode(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn decode_fixed_hex<const N: usize>(value: &str) -> Result<[u8; N]> {
    let value = value.trim();
    if value.len() != N * 2 {
        bail!("expected {} hexadecimal characters", N * 2);
    }
    let mut output = [0_u8; N];
    for (index, slot) in output.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .with_context(|| "receipt authority signature contains non-hexadecimal data")?;
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{boundary_path, is_within_boundary};
    use std::path::Path;

    #[test]
    fn repository_descendant_is_inside_the_authority_boundary() {
        let repo = boundary_path(Path::new(r"C:\workspace\repo")).unwrap();
        let seed = boundary_path(Path::new(
            r"C:\workspace\repo\.baron\machine-home\authority\execution-receipt-ed25519.seed",
        ))
        .unwrap();
        assert!(is_within_boundary(&seed, &repo));

        let extended_repo = boundary_path(Path::new(r"\\?\C:\workspace\repo")).unwrap();
        assert!(is_within_boundary(&seed, &extended_repo));
    }
}
