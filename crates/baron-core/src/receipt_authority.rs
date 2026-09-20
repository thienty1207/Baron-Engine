//! Machine-local Ed25519 authority for trusted execution receipts.
//!
//! The private seed is deliberately kept outside repositories and Vaults. A
//! receipt signed by this key is authoritative only on the machine that owns
//! the seed; losing or regenerating the seed makes older receipts diagnostic.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

use crate::config::machine_config_path;
use crate::safe_io::{ensure_directory_chain, read_bytes};

const SEED_BYTES: usize = 32;
const MAX_SEED_READ_ATTEMPTS: usize = 100;
const SEED_READ_RETRY: Duration = Duration::from_millis(10);
const SEED_RELATIVE_PATH: &str = "authority/execution-receipt-ed25519.seed";
const KEY_ID_PREFIX: &str = "ed25519-";

pub(crate) struct ReceiptAuthority {
    pub(crate) key_id: String,
    signing_key: SigningKey,
}

impl ReceiptAuthority {
    pub(crate) fn load_or_create() -> Result<Self> {
        let path = receipt_authority_seed_path()?;
        let parent = path
            .parent()
            .context("Receipt authority seed has no parent directory")?;
        ensure_directory_chain(parent)?;

        let mut create = OpenOptions::new();
        create.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut create, 0o600);
        match create.open(&path) {
            Ok(mut file) => {
                let mut seed = [0_u8; SEED_BYTES];
                if let Err(error) = getrandom::getrandom(&mut seed)
                    .map_err(|error| {
                        anyhow::anyhow!("Could not generate receipt authority seed: {error}")
                    })
                    .and_then(|_| {
                        file.write_all(&seed)
                            .context("Could not write receipt authority seed")?;
                        file.flush()
                            .context("Could not flush receipt authority seed")?;
                        file.sync_all()
                            .context("Could not sync receipt authority seed")?;
                        set_private_permissions(&file, &path)
                    })
                {
                    let _ = fs::remove_file(&path);
                    return Err(error);
                }
                Ok(Self::from_seed(seed))
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                Self::load_existing_with_race_retry(&path)
            }
            Err(error) => Err(error).with_context(|| {
                format!(
                    "Could not create receipt authority seed atomically: {}",
                    path.display()
                )
            }),
        }
    }

    pub(crate) fn load_existing() -> Result<Self> {
        let path = receipt_authority_seed_path()?;
        let seed = read_seed(&path)?;
        Ok(Self::from_seed(seed))
    }

    pub(crate) fn sign(&self, payload: &[u8]) -> String {
        hex_encode(self.signing_key.sign(payload).to_bytes())
    }

    pub(crate) fn verify(key_id: &str, signature: &str, payload: &[u8]) -> Result<()> {
        let authority = Self::load_existing()?;
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

    fn load_existing_with_race_retry(path: &Path) -> Result<Self> {
        let mut last_error = None;
        for _ in 0..MAX_SEED_READ_ATTEMPTS {
            match read_seed(path) {
                Ok(seed) => return Ok(Self::from_seed(seed)),
                Err(error) if is_retryable_seed_error(&error) => {
                    last_error = Some(error);
                    thread::sleep(SEED_READ_RETRY);
                }
                Err(error) => return Err(error),
            }
        }
        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!("receipt authority seed was not available after creation race")
        }))
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

pub(crate) fn receipt_authority_seed_path() -> Result<PathBuf> {
    let machine_config = machine_config_path()?;
    let machine_home = machine_config
        .parent()
        .context("Baron machine config has no parent directory")?;
    Ok(machine_home.join(SEED_RELATIVE_PATH))
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

fn is_retryable_seed_error(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.contains("must be exactly 32 bytes; found 0")
        || message.contains("must be exactly 32 bytes; found ")
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
