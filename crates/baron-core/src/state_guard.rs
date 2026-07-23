use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::config::{load_local_config, load_project_config, PROJECT_SCHEMA_VERSION};
use crate::vault::{load_capsule_metadata, vault_context_without_create, VaultContext};

const REPAIR_GUIDANCE: &str =
    "Run `baron update` to reconcile Baron-managed state before continuing.";
const CAPSULE_SCHEMA_VERSION: u32 = 2;

pub fn require_coherent_execution_state(
    repo_path: impl AsRef<Path>,
    vault_path: impl AsRef<Path>,
) -> Result<VaultContext> {
    let repo_root = repo_path.as_ref().canonicalize().with_context(|| {
        format!(
            "Could not resolve Baron project path: {}",
            repo_path.as_ref().display()
        )
    })?;
    let config = load_project_config(&repo_root)
        .with_context(|| format!("Baron project configuration is unreadable. {REPAIR_GUIDANCE}"))?;
    if config.schema_version != PROJECT_SCHEMA_VERSION {
        bail!(
            "Baron project schema {} is unsupported; expected {}. {}",
            config.schema_version,
            PROJECT_SCHEMA_VERSION,
            REPAIR_GUIDANCE
        );
    }
    if config.project_id.trim().is_empty() {
        bail!("Baron project identity is missing. {REPAIR_GUIDANCE}");
    }

    let vault_root = vault_path.as_ref().canonicalize().with_context(|| {
        format!(
            "Baron Vault is missing or unreadable at {}. {}",
            vault_path.as_ref().display(),
            REPAIR_GUIDANCE
        )
    })?;
    let configured_vault = load_local_config(&repo_root)
        .with_context(|| format!("Baron local Vault binding is unreadable. {REPAIR_GUIDANCE}"))?
        .vault_path
        .canonicalize()
        .with_context(|| format!("Baron local Vault binding is missing. {REPAIR_GUIDANCE}"))?;
    if configured_vault != vault_root {
        bail!(
            "Baron Vault binding mismatch: project is bound to {}, but execution requested {}. {}",
            configured_vault.display(),
            vault_root.display(),
            REPAIR_GUIDANCE
        );
    }

    let context = vault_context_without_create(&vault_root, &repo_root)?;
    if !context.project_root.is_dir() {
        bail!(
            "Baron project capsule is missing at {}. {}",
            context.project_root.display(),
            REPAIR_GUIDANCE
        );
    }
    let metadata = load_capsule_metadata(&context.project_root)
        .with_context(|| format!("Baron capsule metadata is unreadable. {REPAIR_GUIDANCE}"))?
        .with_context(|| format!("Baron capsule metadata is missing. {REPAIR_GUIDANCE}"))?;
    if metadata.schema_version != CAPSULE_SCHEMA_VERSION {
        bail!(
            "Baron capsule schema {} is unsupported; expected {}. {}",
            metadata.schema_version,
            CAPSULE_SCHEMA_VERSION,
            REPAIR_GUIDANCE
        );
    }
    if metadata.project_id != context.project_id || metadata.project_slug != context.project_slug {
        bail!(
            "Baron project identity mismatch between repo and Vault capsule. {}",
            REPAIR_GUIDANCE
        );
    }
    Ok(context)
}
