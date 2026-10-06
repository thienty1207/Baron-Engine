use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::safe_io::{
    acquire_project_lock, create_new_file, ensure_directory_chain, project_lock_path, read_bytes,
    read_text, read_text_required, replace_file, ProjectMutationLock,
};
use crate::vault::{
    ensure_vault_with_scaffold_observer, project_slug, vault_context_without_create, VaultContext,
    VaultScaffoldPhase,
};

const LEGACY_CONFIG: &str = "vault.config.json";
const LEGACY_MANIFEST: &str = ".agent-bootstrap-manifest.json";
const LEGACY_BLOCK_START: &str = "<!-- agent-bootstrap:start -->";
const LEGACY_BLOCK_END: &str = "<!-- agent-bootstrap:end -->";
const BARON_STATE: &str = ".baron/migration-state.json";

const BUNDLED_SKILLS: &[&str] = &[
    "superpowers",
    "frontend-design",
    "vibe-security-scan",
    "binary-reverse-analysis",
    "apk-mobile-analysis",
    "malware-triage",
    "database-engineering",
    "mobile-application-engineering",
];
const CORE_AGENTS: &[&str] = &[
    "code-reviewer.toml",
    "security-auditor.toml",
    "test-engineer.toml",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationAction {
    Import,
    Preserve,
    Quarantine,
    Remove,
    Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationAssetKind {
    LegacyConfig,
    LegacyManifest,
    LegacyRuntime,
    LegacyHook,
    ManagedInstruction,
    ManagedAsset,
    RepoPlan,
    ProductHarness,
    VaultMemory,
    CustomSkill,
    CustomAgent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationItem {
    pub relative_path: String,
    pub kind: MigrationAssetKind,
    pub action: MigrationAction,
    pub reason: String,
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationInventory {
    pub repo_root: PathBuf,
    pub source_vault: PathBuf,
    pub source_project_root: PathBuf,
    pub project_slug: String,
    pub items: Vec<MigrationItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationReceipt {
    pub migration_id: String,
    pub status: String,
    pub repo_root: PathBuf,
    pub source_vault: PathBuf,
    pub destination_vault: PathBuf,
    pub backup_root: PathBuf,
    pub imported_count: usize,
    pub quarantined_count: usize,
    pub removed_count: usize,
    pub preserved_count: usize,
    pub import_records: Vec<ImportRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackReport {
    pub migration_id: String,
    pub status: String,
    pub restored_count: usize,
}

/// A managed target and the content hash observed by the installer callback
/// before it returns. Migration rechecks this hash under the publication locks
/// so a concurrent post-install edit cannot become rollback authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationInstallOutput {
    pub relative_path: String,
    pub content_hash: Option<String>,
}

/// Exact managed targets published by a successful installer callback. These
/// captured hashes become rollback authority only after the callback reports
/// success and migration verifies the bytes have not changed; failed or raced
/// installer output stays outside the expected baseline for recovery.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MigrationInstallOutputs {
    pub repo_paths: Vec<MigrationInstallOutput>,
    pub vault_paths: Vec<MigrationInstallOutput>,
}

impl MigrationInstallOutputs {
    /// Capture expected hashes at the end of a successful installer callback.
    /// `execute_agent_bootstrap_migration_with_outputs` later verifies these
    /// hashes again under the repo/capsule/Vault lock ordering before making
    /// them rollback expectations.
    pub fn capture(
        repo_root: impl AsRef<Path>,
        vault_root: impl AsRef<Path>,
        repo_paths: Vec<String>,
        vault_paths: Vec<String>,
    ) -> Result<Self> {
        fn capture_scope(root: &Path, paths: Vec<String>) -> Result<Vec<MigrationInstallOutput>> {
            paths
                .into_iter()
                .map(|path| {
                    let relative_path = path.replace('\\', "/");
                    if !is_safe_relative_path(&relative_path) {
                        bail!("Migration installer returned an unsafe output path: {path}");
                    }
                    let target = validate_restore_target(root, &relative_path, "installer output")?;
                    Ok(MigrationInstallOutput {
                        relative_path,
                        content_hash: hash_path(&target)?,
                    })
                })
                .collect()
        }

        Ok(Self {
            repo_paths: capture_scope(repo_root.as_ref(), repo_paths)?,
            vault_paths: capture_scope(vault_root.as_ref(), vault_paths)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportRecord {
    pub source: String,
    pub destination: String,
    pub source_hash: String,
    pub destination_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyConfig {
    vault_root: PathBuf,
    project_slug: String,
    project_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyManifest {
    #[serde(default)]
    entries: BTreeMap<String, LegacyManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyManifestEntry {
    #[serde(rename = "syncedHash")]
    synced_hash: String,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupManifest {
    migration_id: String,
    repo_root: PathBuf,
    vault_root: PathBuf,
    #[serde(default)]
    post_handoff_captured: bool,
    #[serde(default)]
    capsule_relative: Option<String>,
    entries: Vec<BackupEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupEntry {
    scope: BackupScope,
    relative_path: String,
    existed: bool,
    was_directory: bool,
    original_hash: Option<String>,
    #[serde(default)]
    post_handoff_hash: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BackupScope {
    Repo,
    Vault,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MigrationState {
    migration_id: String,
    status: String,
    vault_root: PathBuf,
    backup_root: PathBuf,
    updated_at: String,
}

pub fn inventory_agent_bootstrap(
    repo_path: impl AsRef<Path>,
    _vault_override: Option<&Path>,
) -> Result<MigrationInventory> {
    let repo_root = canonical_directory(repo_path.as_ref())?;
    let config_path = repo_root.join(LEGACY_CONFIG);
    let config: LegacyConfig = read_json(&config_path).with_context(|| {
        format!(
            "Agent Bootstrap config not found or invalid at {}",
            config_path.display()
        )
    })?;
    if !is_safe_component(&config.project_slug) {
        bail!("unsafe legacy project slug: {}", config.project_slug);
    }
    let source_vault = config.vault_root.clone();
    let source_project_root = config
        .project_root
        .clone()
        .unwrap_or_else(|| source_vault.join("Projects").join(&config.project_slug));
    validate_source_project_root(&source_vault, &source_project_root)?;
    let manifest = read_legacy_manifest(&repo_root.join(LEGACY_MANIFEST))?;
    let mut items = Vec::new();

    push_file_item(
        &mut items,
        &repo_root,
        LEGACY_CONFIG,
        MigrationAssetKind::LegacyConfig,
        MigrationAction::Remove,
        "Baron stores routing in .baron project and local config",
    )?;
    push_file_item(
        &mut items,
        &repo_root,
        LEGACY_MANIFEST,
        MigrationAssetKind::LegacyManifest,
        MigrationAction::Remove,
        "legacy scaffold ownership metadata",
    )?;
    push_runtime_item(
        &mut items,
        &repo_root,
        "scripts/agent-memory.js",
        MigrationAssetKind::LegacyRuntime,
        "Baron replaces the generated Node runtime",
        &["agent-bootstrap", "vault.config.json"],
    )?;
    push_runtime_item(
        &mut items,
        &repo_root,
        ".githooks/post-commit",
        MigrationAssetKind::LegacyHook,
        "legacy hook invokes the generated Node runtime",
        &["scripts/agent-memory.js", "agent-bootstrap"],
    )?;
    if repo_root.join("AGENTS.md").exists() {
        items.push(MigrationItem {
            relative_path: "AGENTS.md".to_string(),
            kind: MigrationAssetKind::ManagedInstruction,
            action: MigrationAction::Replace,
            reason: "preserve user text and replace the Agent Bootstrap managed block".to_string(),
            content_hash: hash_path(&repo_root.join("AGENTS.md"))?,
        });
    }

    add_data_root(
        &mut items,
        &repo_root,
        "docs/superpowers/plans",
        MigrationAssetKind::RepoPlan,
        "convert legacy active plans into docs/baron/plans",
    )?;
    for relative in [
        "docs/product",
        "docs/stories",
        "docs/validation",
        "docs/decisions",
    ] {
        add_data_root(
            &mut items,
            &repo_root,
            relative,
            MigrationAssetKind::ProductHarness,
            "convert legacy Product Harness material into docs/baron/harness",
        )?;
    }
    if source_project_root.exists() {
        items.push(MigrationItem {
            relative_path: normalize(&source_project_root),
            kind: MigrationAssetKind::VaultMemory,
            action: MigrationAction::Import,
            reason: "import the legacy project capsule into Baron Vault memory".to_string(),
            content_hash: hash_path(&source_project_root)?,
        });
    }

    scan_custom_skills(&repo_root, &mut items)?;
    scan_custom_agents(&repo_root, &mut items)?;
    add_manifest_owned_assets(&repo_root, &manifest, &mut items)?;
    items.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    items.dedup_by(|left, right| left.relative_path == right.relative_path);

    Ok(MigrationInventory {
        repo_root,
        source_vault,
        source_project_root,
        project_slug: config.project_slug,
        items,
    })
}

pub fn render_migration_inventory(inventory: &MigrationInventory) -> String {
    let mut output = format!(
        "# Baron Agent Bootstrap Migration Dry Run\n\n- Repo: `{}`\n- Source Vault: `{}`\n- Legacy project: `{}`\n- Mode: read-only\n- No files were written.\n\n## Inventory\n\n",
        inventory.repo_root.display(),
        inventory.source_vault.display(),
        inventory.project_slug
    );
    for item in &inventory.items {
        output.push_str(&format!(
            "- `{:?}` `{:?}` `{}` - {}\n",
            item.action, item.kind, item.relative_path, item.reason
        ));
    }
    output
}

pub fn execute_agent_bootstrap_migration<F>(
    repo_path: impl AsRef<Path>,
    vault_override: Option<&Path>,
    install_baron: F,
) -> Result<MigrationReceipt>
where
    F: FnOnce(&Path, &Path) -> Result<()>,
{
    execute_agent_bootstrap_migration_with_outputs(repo_path, vault_override, move |repo, vault| {
        install_baron(repo, vault)?;
        Ok(MigrationInstallOutputs::default())
    })
}

pub fn execute_agent_bootstrap_migration_with_outputs<F>(
    repo_path: impl AsRef<Path>,
    vault_override: Option<&Path>,
    install_baron: F,
) -> Result<MigrationReceipt>
where
    F: FnOnce(&Path, &Path) -> Result<MigrationInstallOutputs>,
{
    // Inventory and backup are read-only/backup work and intentionally happen
    // without the project lock. Each actual migration publication below takes
    // the lock only around the bounded read/merge/write operation that it is
    // protecting. This keeps a large legacy tree from blocking all project
    // writers while preserving the SPEC-05 mutation invariant.
    let inventory = inventory_agent_bootstrap(repo_path, vault_override)?;
    let destination_vault = vault_override
        .map(Path::to_path_buf)
        .unwrap_or_else(|| inventory.source_vault.clone());
    let migration_id = migration_id(&inventory.repo_root);
    let backup_root = destination_vault
        .join("Artifacts/Baron/Migrations")
        .join(&migration_id);
    let manifest_path = backup_root.join("manifest.json");
    reserve_backup_root(&inventory.repo_root, &backup_root)?;

    let mut backup_manifest =
        create_backup_manifest(&inventory, &destination_vault, &backup_root, &migration_id)?;
    write_json(&manifest_path, &backup_manifest)?;

    let result: Result<MigrationReceipt> = (|| {
        let destination = ensure_migration_vault(&destination_vault, &mut backup_manifest)?;
        let mut import_records = Vec::new();
        import_legacy_vault(
            &inventory.source_project_root,
            &destination.project_root,
            &backup_root,
            &mut import_records,
            &inventory.repo_root,
            &mut backup_manifest,
        )?;
        import_repo_data(
            &inventory.repo_root,
            &backup_root,
            &mut import_records,
            &inventory.repo_root,
            &mut backup_manifest,
        )?;
        let quarantined_count = quarantine_invalid_assets(
            &inventory,
            &backup_root,
            &migration_id,
            &mut backup_manifest,
        )?;

        // Seal the hashes recorded at publication, never adopt a later read of
        // an imported path as migration-owned output. The installer is unlocked.
        capture_post_handoff_state(&inventory.repo_root, &mut backup_manifest)?;
        write_json(&backup_root.join("manifest.json"), &backup_manifest)?;
        let install_outputs = install_baron(&inventory.repo_root, &destination_vault)?;
        let _lock = acquire_project_lock(&inventory.repo_root)?;
        let _vault_lock = acquire_project_lock(&destination_vault)?;
        let _capsule_lock = lock_manifest_capsule(&backup_manifest)?;
        capture_installer_outputs(&mut backup_manifest, &install_outputs)?;
        // The failure path reloads the manifest from disk before rollback.
        // Persist the verified installer hashes now, while publication is
        // locked, so a later migration error can restore these exact outputs.
        write_json(&manifest_path, &backup_manifest)?;
        register_valid_custom_assets(&inventory, &mut backup_manifest, &manifest_path)?;
        let agents_path = inventory.repo_root.join("AGENTS.md");
        if let Some(entry) = backup_manifest
            .entries
            .iter()
            .find(|entry| entry.scope == BackupScope::Repo && entry.relative_path == "AGENTS.md")
        {
            let expected_hash = entry
                .post_handoff_hash
                .as_deref()
                .context("Migration has no captured AGENTS.md publication hash")?;
            remove_legacy_managed_block(&agents_path, expected_hash)?;
            record_post_handoff_target(&mut backup_manifest, BackupScope::Repo, "AGENTS.md")?;
            write_json(&manifest_path, &backup_manifest)?;
        } else if agents_path.exists() {
            bail!(
                "Migration AGENTS.md appeared after its backup inventory; preserving it for recovery: {}",
                agents_path.display()
            );
        }
        let removed_count =
            cleanup_legacy_runtime(&inventory, &mut backup_manifest, &manifest_path)?;
        verify_imports(&import_records)?;
        verify_native_state(&inventory.repo_root)?;

        let receipt = MigrationReceipt {
            migration_id: migration_id.clone(),
            status: "completed".to_string(),
            repo_root: inventory.repo_root.clone(),
            source_vault: inventory.source_vault.clone(),
            destination_vault: destination_vault.clone(),
            backup_root: backup_root.clone(),
            imported_count: import_records.len(),
            quarantined_count,
            removed_count,
            preserved_count: inventory
                .items
                .iter()
                .filter(|item| item.action == MigrationAction::Preserve)
                .count(),
            import_records,
        };
        write_json(&backup_root.join("receipt.json"), &receipt)?;
        write_state(
            &inventory.repo_root,
            MigrationState {
                migration_id: migration_id.clone(),
                status: "completed".to_string(),
                vault_root: destination_vault.clone(),
                backup_root: backup_root.clone(),
                updated_at: now(),
            },
        )?;
        record_post_handoff_target(&mut backup_manifest, BackupScope::Repo, BARON_STATE)?;
        write_json(&backup_root.join("manifest.json"), &backup_manifest)?;
        Ok(receipt)
    })();

    match result {
        Ok(receipt) => Ok(receipt),
        Err(error) => {
            // Only a successfully persisted handoff authorizes automatic
            // restoration. Partial imports keep their recovery copies.
            let manifest = read_json::<BackupManifest>(&backup_root.join("manifest.json"));
            let post_handoff_captured = manifest
                .as_ref()
                .is_ok_and(|manifest| manifest.post_handoff_captured);
            let rollback = match manifest {
                Ok(manifest) if manifest.post_handoff_captured => {
                    restore_from_manifest_if_unchanged(&manifest, &backup_root).map(|outcome| {
                        serde_json::json!({
                            "restored": outcome.restored,
                            "conflicts": outcome.conflicts
                        })
                    })
                }
                Ok(_) => Err(anyhow::anyhow!(
                    "automatic migration rollback has no persisted handoff baseline"
                )),
                Err(manifest_error) => Err(manifest_error),
            };
            let rollback_conflicts = rollback
                .as_ref()
                .ok()
                .and_then(|value| value.get("conflicts"))
                .and_then(serde_json::Value::as_u64)
                .unwrap_or_default();
            let status = if !post_handoff_captured {
                "needs_recovery"
            } else if rollback.is_err() {
                "rollback_failed"
            } else if rollback_conflicts > 0 {
                "rolled_back_with_conflicts"
            } else {
                "rolled_back"
            };
            let failure = serde_json::json!({
                "migrationId": migration_id,
                "status": status,
                "error": error.to_string(),
                "rollback": rollback.as_ref().ok(),
                "rollbackError": rollback.as_ref().err().map(|value| value.to_string()),
                "updatedAt": now()
            });
            let _ = write_json(&backup_root.join("failure.json"), &failure);
            if let Ok(_lock) = acquire_project_lock(&inventory.repo_root) {
                let _ = write_state(
                    &inventory.repo_root,
                    MigrationState {
                        migration_id,
                        status: status.to_string(),
                        vault_root: destination_vault,
                        backup_root,
                        updated_at: now(),
                    },
                );
            }
            Err(error)
        }
    }
}

pub fn migration_status(repo_path: impl AsRef<Path>) -> Result<String> {
    let repo_root = canonical_directory(repo_path.as_ref())?;
    let state_path = repo_root.join(BARON_STATE);
    let Some(content) = read_text(&state_path)? else {
        return Ok("# Baron Migration Status\n\n- Status: `never_run`\n".to_string());
    };
    let state: MigrationState = serde_json::from_str(&content)
        .with_context(|| format!("Could not parse {}", state_path.display()))?;
    Ok(format!(
        "# Baron Migration Status\n\n- Migration ID: `{}`\n- Status: `{}`\n- Vault: `{}`\n- Backup: `{}`\n- Updated: {}\n",
        state.migration_id,
        state.status,
        state.vault_root.display(),
        state.backup_root.display(),
        state.updated_at
    ))
}

pub fn rollback_migration(
    repo_path: impl AsRef<Path>,
    vault_path: impl AsRef<Path>,
    migration_id: &str,
) -> Result<RollbackReport> {
    if !is_safe_component(migration_id) {
        bail!("unsafe migration id: {migration_id}");
    }
    let repo_root = canonical_directory(repo_path.as_ref())?;
    let _lock = acquire_project_lock(&repo_root)?;
    let vault_root = vault_path.as_ref().canonicalize().with_context(|| {
        format!(
            "Could not resolve migration Vault: {}",
            vault_path.as_ref().display()
        )
    })?;
    let backup_root = vault_root
        .join("Artifacts/Baron/Migrations")
        .join(migration_id);
    let manifest: BackupManifest = read_json(&backup_root.join("manifest.json"))?;
    if manifest.migration_id != migration_id {
        bail!(
            "Migration manifest id `{}` does not match requested `{migration_id}`",
            manifest.migration_id
        );
    }
    if manifest.repo_root != repo_root {
        bail!(
            "Migration `{migration_id}` belongs to {}, not {}",
            manifest.repo_root.display(),
            repo_root.display()
        );
    }
    let manifest_vault = manifest.vault_root.canonicalize().with_context(|| {
        format!(
            "Could not resolve migration manifest Vault: {}",
            manifest.vault_root.display()
        )
    })?;
    if manifest_vault != vault_root {
        bail!(
            "Migration `{migration_id}` belongs to Vault {}, not {}",
            manifest_vault.display(),
            vault_root.display()
        );
    }
    if !manifest.post_handoff_captured {
        let message = "Explicit migration rollback has no complete persisted handoff baseline";
        record_rollback_conflict(&manifest, &backup_root, 0, message)?;
        bail!("{message}; migration recovery data was preserved");
    }
    let rollback = restore_from_manifest_if_unchanged(&manifest, &backup_root)?;
    if rollback.conflicts > 0 {
        let conflict_paths = rollback
            .conflict_paths
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>();
        let remaining = rollback.conflicts.saturating_sub(conflict_paths.len());
        let conflict_suffix = if remaining == 0 {
            conflict_paths.join(", ")
        } else {
            format!("{}, and {remaining} more", conflict_paths.join(", "))
        };
        let message = format!(
            "Explicit migration rollback found {} changed managed target(s): {conflict_suffix}",
            rollback.conflicts,
        );
        record_rollback_conflict(&manifest, &backup_root, rollback.conflicts, &message)?;
        bail!("{message}; migration recovery data was preserved");
    }
    let restored_count = rollback.restored;
    write_state(
        &repo_root,
        MigrationState {
            migration_id: migration_id.to_string(),
            status: "rolled_back".to_string(),
            vault_root: vault_path.as_ref().to_path_buf(),
            backup_root,
            updated_at: now(),
        },
    )?;
    Ok(RollbackReport {
        migration_id: migration_id.to_string(),
        status: "rolled_back".to_string(),
        restored_count,
    })
}

fn record_rollback_conflict(
    manifest: &BackupManifest,
    backup_root: &Path,
    conflicts: usize,
    message: &str,
) -> Result<()> {
    let _repo_lock = acquire_project_lock(&manifest.repo_root)?;
    let _vault_lock = acquire_project_lock(&manifest.vault_root)?;
    let _capsule_lock = lock_manifest_capsule(manifest)?;
    write_state(
        &manifest.repo_root,
        MigrationState {
            migration_id: manifest.migration_id.clone(),
            status: "needs_recovery".to_string(),
            vault_root: manifest.vault_root.clone(),
            backup_root: backup_root.to_path_buf(),
            updated_at: now(),
        },
    )?;
    write_json(
        &backup_root.join("failure.json"),
        &serde_json::json!({
            "migrationId": manifest.migration_id,
            "status": "needs_recovery",
            "error": message,
            "rollback": {"restored": 0, "conflicts": conflicts},
            "rollbackError": message,
            "updatedAt": now()
        }),
    )
}

fn create_backup_manifest(
    inventory: &MigrationInventory,
    destination_vault: &Path,
    backup_root: &Path,
    migration_id: &str,
) -> Result<BackupManifest> {
    ensure_directory_chain(backup_root)?;
    if inventory.source_project_root.exists() {
        copy_path(
            &inventory.source_project_root,
            &backup_root
                .join("source-vault")
                .join(&inventory.project_slug),
            false,
            None,
            None,
        )?;
    }
    let destination = {
        let _repo_lock = acquire_project_lock(&inventory.repo_root)?;
        let _vault_lock = acquire_project_lock(destination_vault)?;
        vault_context_without_create(destination_vault, &inventory.repo_root)?
    };
    let destination_project = &destination.project_root;
    let mut repo_paths = BTreeSet::new();
    for path in [
        LEGACY_CONFIG,
        LEGACY_MANIFEST,
        "AGENTS.md",
        "scripts/agent-memory.js",
        ".githooks/post-commit",
        ".baron/project.toml",
        ".baron/local.toml",
        ".baron/.gitignore",
        BARON_STATE,
        ".codex/INDEX.md",
        ".codex/skills/INDEX.md",
        ".codex/agents/INDEX.md",
    ] {
        repo_paths.insert(path.to_string());
    }
    add_repo_import_targets(&inventory.repo_root, &mut repo_paths)?;
    repo_paths.insert(format!(".baron/quarantine/{migration_id}"));
    for skill in BUNDLED_SKILLS {
        repo_paths.insert(format!(".codex/skills/{skill}"));
    }
    for agent in CORE_AGENTS {
        repo_paths.insert(format!(".codex/agents/{agent}"));
    }
    for item in &inventory.items {
        if item.action == MigrationAction::Quarantine {
            repo_paths.insert(item.relative_path.clone());
        }
    }

    let mut entries = Vec::new();
    for relative in repo_paths {
        entries.push(backup_entry(
            BackupScope::Repo,
            &inventory.repo_root,
            &relative,
            &backup_root.join("repo"),
        )?);
    }

    let mut vault_paths = BTreeSet::new();
    let destination_relative = destination
        .project_root
        .strip_prefix(destination_vault)
        .map(normalize)
        .unwrap_or_else(|_| format!("Projects/{}", project_slug(&inventory.repo_root)));
    for file in [
        "README.md",
        "Facts.md",
        "Decisions.md",
        "Tasks.md",
        ".baron-project.json",
    ] {
        vault_paths.insert(format!("{destination_relative}/{file}"));
    }
    if inventory.source_project_root.exists() {
        let mut source_files = Vec::new();
        collect_files(&inventory.source_project_root, &mut source_files)?;
        for source in source_files {
            let relative = source
                .strip_prefix(&inventory.source_project_root)
                .unwrap_or(&source);
            vault_paths.insert(format!("{destination_relative}/{}", normalize(relative)));
        }
    }
    for relative in [
        "AGENTS.md",
        "Init.md",
        "Artifacts/Baron/APPROVED_GLOBAL.md",
        "Artifacts/Baron/GLOBAL_CANDIDATES.md",
        "Artifacts/Baron/memory-engine-state.json",
        "Artifacts/Baron/memory-index.sqlite",
    ] {
        vault_paths.insert(relative.to_string());
    }
    for relative in vault_paths {
        // Back up each shared capsule file under checkout -> Vault -> capsule locks;
        // source traversal and the complete backup scan stay outside them.
        let _lock = acquire_project_lock(&inventory.repo_root)?;
        let _vault_lock = acquire_project_lock(destination_vault)?;
        let _capsule_lock = if destination_project.is_dir() {
            Some(acquire_project_lock(destination_project)?)
        } else {
            None
        };
        entries.push(backup_entry(
            BackupScope::Vault,
            destination_vault,
            &relative,
            &backup_root.join("vault"),
        )?);
    }

    Ok(BackupManifest {
        migration_id: migration_id.to_string(),
        repo_root: inventory.repo_root.clone(),
        vault_root: destination_vault.to_path_buf(),
        post_handoff_captured: false,
        capsule_relative: Some(destination_relative),
        entries,
    })
}

fn reserve_backup_root(repo_root: &Path, backup_root: &Path) -> Result<()> {
    let _lock = acquire_project_lock(repo_root)?;
    let parent = backup_root
        .parent()
        .context("Migration backup path has no parent directory")?;
    ensure_directory_chain(parent)?;
    match fs::symlink_metadata(backup_root) {
        Ok(_) => bail!("Migration backup already exists: {}", backup_root.display()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(backup_root).with_context(|| {
                format!(
                    "Could not reserve migration backup path: {}",
                    backup_root.display()
                )
            })?;
        }
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "Could not inspect migration backup path: {}",
                    backup_root.display()
                )
            })
        }
    }
    Ok(())
}

fn add_repo_import_targets(repo_root: &Path, paths: &mut BTreeSet<String>) -> Result<()> {
    for (source, destination) in [
        ("docs/superpowers/plans", "docs/baron/plans"),
        ("docs/product/traces", "docs/baron/traces"),
        ("docs/product/proofs", "docs/baron/proofs"),
        ("docs/product", "docs/baron/harness/product"),
        ("docs/stories", "docs/baron/harness/stories"),
        ("docs/validation", "docs/baron/harness/validation"),
        ("docs/decisions", "docs/baron/harness/decisions"),
    ] {
        let source_root = repo_root.join(source);
        if !source_root.exists() {
            continue;
        }
        let mut files = Vec::new();
        collect_files(&source_root, &mut files)?;
        for file in files {
            let relative = file.strip_prefix(&source_root).unwrap_or(&file);
            paths.insert(format!("{destination}/{}", normalize(relative)));
        }
    }
    Ok(())
}

fn backup_entry(
    scope: BackupScope,
    root: &Path,
    relative: &str,
    backup_scope_root: &Path,
) -> Result<BackupEntry> {
    let source = root.join(relative);
    let source_hash_before = hash_path(&source)?;
    let existed = source_hash_before.is_some();
    let was_directory = source.is_dir();
    if existed {
        copy_path(
            &source,
            &backup_scope_root.join(relative),
            false,
            None,
            None,
        )?;
    }
    let original_hash = if existed {
        hash_path(&backup_scope_root.join(relative))?
    } else {
        None
    };
    if existed && original_hash.is_none() {
        bail!(
            "Migration backup snapshot is missing for {}",
            source.display()
        );
    }
    if hash_path(&source)? != original_hash {
        bail!(
            "Migration source changed while its backup snapshot was captured: {}",
            source.display()
        );
    }
    Ok(BackupEntry {
        scope,
        relative_path: relative.to_string(),
        existed,
        was_directory,
        post_handoff_hash: original_hash.clone(),
        original_hash,
    })
}

fn post_handoff_target_matches(
    manifest: &BackupManifest,
    scope: BackupScope,
    relative_path: &str,
) -> Result<bool> {
    let root = match scope {
        BackupScope::Repo => &manifest.repo_root,
        BackupScope::Vault => &manifest.vault_root,
    };
    let entry = manifest
        .entries
        .iter()
        .find(|entry| entry.scope == scope && entry.relative_path == relative_path)
        .with_context(|| format!("Migration manifest does not track {relative_path}"))?;
    let target = root.join(relative_path);
    Ok(hash_path(&target)? == entry.post_handoff_hash)
}

fn ensure_post_handoff_target_unchanged(
    manifest: &BackupManifest,
    scope: BackupScope,
    relative_path: &str,
) -> Result<()> {
    if !post_handoff_target_matches(manifest, scope, relative_path)? {
        let root = match scope {
            BackupScope::Repo => &manifest.repo_root,
            BackupScope::Vault => &manifest.vault_root,
        };
        let target = root.join(relative_path);
        bail!(
            "Migration publication baseline changed: {}",
            target.display()
        );
    }
    Ok(())
}

fn record_post_handoff_target(
    manifest: &mut BackupManifest,
    scope: BackupScope,
    relative_path: &str,
) -> Result<()> {
    let root = match scope {
        BackupScope::Repo => &manifest.repo_root,
        BackupScope::Vault => &manifest.vault_root,
    };
    let hash = hash_path(&root.join(relative_path))?;
    let entry = manifest
        .entries
        .iter_mut()
        .find(|entry| entry.scope == scope && entry.relative_path == relative_path)
        .with_context(|| format!("Migration manifest does not track {relative_path}"))?;
    entry.post_handoff_hash = hash;
    Ok(())
}

fn capture_installer_outputs(
    manifest: &mut BackupManifest,
    outputs: &MigrationInstallOutputs,
) -> Result<()> {
    let mut captured: Vec<(usize, Option<String>)> = Vec::new();
    for (scope, paths) in [
        (BackupScope::Repo, &outputs.repo_paths),
        (BackupScope::Vault, &outputs.vault_paths),
    ] {
        let root = match scope {
            BackupScope::Repo => &manifest.repo_root,
            BackupScope::Vault => &manifest.vault_root,
        };
        for output in paths {
            let normalized = &output.relative_path;
            if !is_safe_relative_path(normalized) {
                bail!("Migration installer returned an unsafe output path: {normalized}");
            }
            let entry_index = manifest
                .entries
                .iter()
                .position(|entry| entry.scope == scope && entry.relative_path == *normalized)
                .with_context(|| {
                    format!("Migration backup does not track installer output {normalized}")
                })?;
            let target = validate_restore_target(root, normalized, "installer output")?;
            let current_hash = hash_path(&target)?;
            if current_hash != output.content_hash {
                bail!(
                    "Migration installer output changed before locked publication: {}",
                    target.display()
                );
            }
            captured.push((entry_index, output.content_hash.clone()));
        }
    }
    for (entry_index, content_hash) in captured {
        manifest.entries[entry_index].post_handoff_hash = content_hash;
    }
    Ok(())
}

fn import_legacy_vault(
    source_root: &Path,
    destination_root: &Path,
    backup_root: &Path,
    records: &mut Vec<ImportRecord>,
    mutation_root: &Path,
    manifest: &mut BackupManifest,
) -> Result<()> {
    if !source_root.exists() || source_root == destination_root {
        return Ok(());
    }
    copy_path(
        source_root,
        destination_root,
        true,
        Some(CopyPublication {
            backup_root,
            records,
            manifest,
            capsule_root: Some(destination_root),
        }),
        Some(mutation_root),
    )
}

fn import_repo_data(
    repo_root: &Path,
    backup_root: &Path,
    records: &mut Vec<ImportRecord>,
    mutation_root: &Path,
    manifest: &mut BackupManifest,
) -> Result<()> {
    for (source, destination) in [
        ("docs/superpowers/plans", "docs/baron/plans"),
        ("docs/product/traces", "docs/baron/traces"),
        ("docs/product/proofs", "docs/baron/proofs"),
        ("docs/product", "docs/baron/harness/product"),
        ("docs/stories", "docs/baron/harness/stories"),
        ("docs/validation", "docs/baron/harness/validation"),
        ("docs/decisions", "docs/baron/harness/decisions"),
    ] {
        let source = repo_root.join(source);
        if source.exists() {
            copy_path(
                &source,
                &repo_root.join(destination),
                true,
                Some(CopyPublication {
                    backup_root,
                    records,
                    manifest,
                    capsule_root: None,
                }),
                Some(mutation_root),
            )?;
        }
    }
    Ok(())
}

fn quarantine_invalid_assets(
    inventory: &MigrationInventory,
    backup_root: &Path,
    migration_id: &str,
    manifest: &mut BackupManifest,
) -> Result<usize> {
    let mut count = 0;
    for item in &inventory.items {
        if item.action != MigrationAction::Quarantine {
            continue;
        }
        let source = inventory.repo_root.join(&item.relative_path);
        // Acquire before checking existence: the check decides whether the
        // following copy/remove mutation is still valid.
        let _lock = acquire_project_lock(&inventory.repo_root)?;
        if !source.exists() {
            continue;
        }
        let repo_quarantine = inventory
            .repo_root
            .join(".baron/quarantine")
            .join(migration_id)
            .join(&item.relative_path);
        let vault_quarantine = backup_root.join("quarantine").join(&item.relative_path);
        let source_entry = prepare_publication(manifest, backup_root, &source)?;
        let quarantine_root = inventory
            .repo_root
            .join(".baron/quarantine")
            .join(migration_id);
        let quarantine_entry = prepare_publication(manifest, backup_root, &quarantine_root)?;
        // The copy-to-quarantine and source removal are one logical mutation
        // for this asset. Keep the lock bounded to the current asset rather
        // than the full inventory/quarantine scan.
        copy_path(&source, &repo_quarantine, false, None, None)?;
        copy_path(&source, &vault_quarantine, false, None, None)?;
        remove_path(&source)?;
        manifest.entries[source_entry].post_handoff_hash = None;
        manifest.entries[quarantine_entry].post_handoff_hash = hash_path(&quarantine_root)?;
        count += 1;
    }
    Ok(count)
}

fn register_valid_custom_assets(
    inventory: &MigrationInventory,
    manifest: &mut BackupManifest,
    manifest_path: &Path,
) -> Result<()> {
    let valid_skills = inventory
        .items
        .iter()
        .filter(|item| {
            item.kind == MigrationAssetKind::CustomSkill && item.action == MigrationAction::Import
        })
        .collect::<Vec<_>>();
    let valid_agents = inventory
        .items
        .iter()
        .filter(|item| {
            item.kind == MigrationAssetKind::CustomAgent && item.action == MigrationAction::Import
        })
        .collect::<Vec<_>>();
    if !valid_skills.is_empty() {
        ensure_post_handoff_target_unchanged(
            manifest,
            BackupScope::Repo,
            ".codex/skills/INDEX.md",
        )?;
    }
    append_custom_routes(
        &inventory.repo_root.join(".codex/skills/INDEX.md"),
        "Imported Custom Skills",
        valid_skills.iter().map(|item| {
            format!(
                "- `{}` - validated during migration",
                item.relative_path.trim_start_matches(".codex/skills/")
            )
        }),
    )?;
    if !valid_skills.is_empty() {
        record_post_handoff_target(manifest, BackupScope::Repo, ".codex/skills/INDEX.md")?;
        write_json(manifest_path, manifest)?;
    }
    if !valid_agents.is_empty() {
        ensure_post_handoff_target_unchanged(
            manifest,
            BackupScope::Repo,
            ".codex/agents/INDEX.md",
        )?;
    }
    append_custom_routes(
        &inventory.repo_root.join(".codex/agents/INDEX.md"),
        "Imported Custom Agents",
        valid_agents.iter().map(|item| {
            format!(
                "- `{}` - validated during migration",
                item.relative_path.trim_start_matches(".codex/agents/")
            )
        }),
    )?;
    if !valid_agents.is_empty() {
        record_post_handoff_target(manifest, BackupScope::Repo, ".codex/agents/INDEX.md")?;
        write_json(manifest_path, manifest)?;
    }
    Ok(())
}

fn append_custom_routes(
    path: &Path,
    heading: &str,
    routes: impl Iterator<Item = String>,
) -> Result<()> {
    let routes = routes.collect::<Vec<_>>();
    if routes.is_empty() {
        return Ok(());
    }
    let mut content = read_text(path)?.unwrap_or_default();
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&format!("\n## {heading}\n\n"));
    for route in routes {
        if !content.contains(&route) {
            content.push_str(&route);
            content.push('\n');
        }
    }
    atomic_write(path, content.as_bytes())
}

fn cleanup_legacy_runtime(
    inventory: &MigrationInventory,
    manifest: &mut BackupManifest,
    manifest_path: &Path,
) -> Result<usize> {
    let mut removed = 0;
    for item in &inventory.items {
        if item.action != MigrationAction::Remove {
            continue;
        }
        let path = inventory.repo_root.join(&item.relative_path);
        ensure_post_handoff_target_unchanged(manifest, BackupScope::Repo, &item.relative_path)?;
        if !path.exists() {
            continue;
        }
        let safe = match item.kind {
            MigrationAssetKind::LegacyConfig | MigrationAssetKind::LegacyManifest => true,
            MigrationAssetKind::LegacyRuntime => file_contains(&path, "agent-bootstrap")?,
            MigrationAssetKind::LegacyHook => {
                file_contains(&path, "scripts/agent-memory.js")?
                    || file_contains(&path, "agent-bootstrap")?
            }
            MigrationAssetKind::ManagedAsset => {
                item.content_hash.is_some() && hash_path(&path)? == item.content_hash
            }
            _ => false,
        };
        if safe {
            remove_path(&path)?;
            record_post_handoff_target(manifest, BackupScope::Repo, &item.relative_path)?;
            write_json(manifest_path, manifest)?;
            removed += 1;
        }
    }
    remove_empty_parent(&inventory.repo_root.join("scripts"), &inventory.repo_root)?;
    remove_empty_parent(&inventory.repo_root.join(".githooks"), &inventory.repo_root)?;
    Ok(removed)
}

fn verify_imports(records: &[ImportRecord]) -> Result<()> {
    for record in records {
        if record.source_hash == record.destination_hash {
            continue;
        }
        let source = read_text_required(&record.source)?;
        let destination = read_text_required(&record.destination)?;
        if source.trim().is_empty() || !destination.contains(source.trim()) {
            bail!(
                "Migration hash mismatch: {} -> {}",
                record.source,
                record.destination
            );
        }
    }
    Ok(())
}

fn verify_native_state(repo_root: &Path) -> Result<()> {
    if !repo_root.join(".baron/project.toml").is_file() {
        bail!("Baron verification failed: .baron/project.toml is missing");
    }
    if read_bytes(repo_root.join("scripts/agent-memory.js"))?.is_some() {
        bail!("Baron verification failed: legacy runtime still exists");
    }
    if read_bytes(repo_root.join(LEGACY_CONFIG))?.is_some() {
        bail!("Baron verification failed: legacy config still exists");
    }
    Ok(())
}

fn manifest_capsule_relative(manifest: &BackupManifest) -> Option<&str> {
    manifest.capsule_relative.as_deref().or_else(|| {
        // Older private manifests used the slug capsule. Keep them readable.
        manifest.entries.iter().find_map(|entry| {
            if entry.scope != BackupScope::Vault {
                return None;
            }
            let suffix = entry.relative_path.strip_prefix("Projects/")?;
            let slash = suffix.find('/').unwrap_or(suffix.len());
            Some(&entry.relative_path[.."Projects/".len() + slash])
        })
    })
}

fn lock_manifest_capsule(manifest: &BackupManifest) -> Result<Option<ProjectMutationLock>> {
    let Some(relative) = manifest_capsule_relative(manifest) else {
        return Ok(None);
    };
    let capsule = validate_restore_target(&manifest.vault_root, relative, "capsule lock")?;
    // Keep the capsule and its lock directory as stable scaffolding. Rollback
    // restores individual data paths, never removes this shared lock inode.
    ensure_directory_chain(&capsule)?;
    Ok(Some(acquire_project_lock(capsule)?))
}

fn ensure_migration_vault(
    vault_root: &Path,
    manifest: &mut BackupManifest,
) -> Result<VaultContext> {
    let capsule_relative = manifest
        .capsule_relative
        .as_deref()
        .context("Missing migration capsule")?;
    // The finite scaffold set is published by ensure_vault. Validate its old
    // bytes before setup and record its output while the same locks are held.
    // Creating the destination capsule first also keeps the legacy source in
    // place: migration copies data instead of renaming the source capsule.
    let setup_paths = [
        "README.md",
        "Facts.md",
        "Decisions.md",
        "Tasks.md",
        ".baron-project.json",
    ]
    .map(|file| format!("{capsule_relative}/{file}"));
    let globals = [
        "AGENTS.md",
        "Init.md",
        "Artifacts/Baron/APPROVED_GLOBAL.md",
        "Artifacts/Baron/GLOBAL_CANDIDATES.md",
        "Artifacts/Baron/memory-engine-state.json",
    ];
    let entries = manifest
        .entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            (entry.scope == BackupScope::Vault
                && (setup_paths.contains(&entry.relative_path)
                    || globals.contains(&entry.relative_path.as_str())))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    let before_relative = capsule_relative.to_string();
    ensure_vault_with_scaffold_observer(vault_root, &manifest.repo_root, |destination, phase| {
        match phase {
            VaultScaffoldPhase::Before => {
                let actual_relative = normalize(
                    destination
                        .project_root
                        .strip_prefix(&destination.vault_root)?,
                );
                if actual_relative != before_relative {
                    bail!(
                        "Migration capsule identity changed from `{before_relative}` to `{actual_relative}`"
                    );
                }
                for &index in &entries {
                    let entry = &manifest.entries[index];
                    if hash_path(&destination.vault_root.join(&entry.relative_path))?
                        != entry.original_hash
                    {
                        bail!("Migration setup baseline changed: {}", entry.relative_path);
                    }
                }
                Ok(())
            }
            VaultScaffoldPhase::After => {
                for &index in &entries {
                    let entry = &mut manifest.entries[index];
                    entry.post_handoff_hash =
                        hash_path(&destination.vault_root.join(&entry.relative_path))?;
                }
                Ok(())
            }
        }
    })
}

fn prepare_publication(
    manifest: &mut BackupManifest,
    backup_root: &Path,
    target: &Path,
) -> Result<usize> {
    let (scope, root, backup_scope) = if target.starts_with(&manifest.repo_root) {
        (BackupScope::Repo, &manifest.repo_root, "repo")
    } else if target.starts_with(&manifest.vault_root) {
        (BackupScope::Vault, &manifest.vault_root, "vault")
    } else {
        bail!(
            "Migration publication escapes its roots: {}",
            target.display()
        );
    };
    let relative = normalize(target.strip_prefix(root)?);
    let index = if let Some(index) = manifest
        .entries
        .iter()
        .position(|entry| entry.scope == scope && entry.relative_path == relative)
    {
        index
    } else {
        // Binary conflicts publish to LegacyImport, not the nominal target.
        // Back up and track the actual path before its first publication.
        manifest.entries.push(backup_entry(
            scope,
            root,
            &relative,
            &backup_root.join(backup_scope),
        )?);
        manifest.entries.len() - 1
    };
    let entry = &manifest.entries[index];
    if entry.existed {
        let backup = backup_root.join(backup_scope).join(&entry.relative_path);
        if hash_path(&backup)? != entry.original_hash {
            bail!(
                "Migration backup snapshot changed before publication: {}",
                backup.display()
            );
        }
    } else if entry.original_hash.is_some() {
        bail!(
            "Migration backup snapshot has an unexpected hash for absent target: {}",
            target.display()
        );
    }
    if hash_path(target)? != manifest.entries[index].post_handoff_hash {
        bail!(
            "Migration publication baseline changed: {}",
            target.display()
        );
    }
    Ok(index)
}

fn capture_post_handoff_state(repo_root: &Path, manifest: &mut BackupManifest) -> Result<()> {
    let _lock = acquire_project_lock(repo_root)?;
    let _vault_lock = acquire_project_lock(&manifest.vault_root)?;
    let _capsule_lock = lock_manifest_capsule(manifest)?;
    // Expected hashes were initialized from the backup and advanced only at
    // our own publications. A reread here would adopt a writer in the gap.
    manifest.post_handoff_captured = true;
    Ok(())
}

#[derive(Debug, Clone)]
struct ConditionalRollback {
    restored: usize,
    conflicts: usize,
    conflict_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct RollbackEntryProgress {
    schema_version: u32,
    migration_id: String,
    scope: BackupScope,
    relative_path: String,
    existed: bool,
    was_directory: bool,
    original_hash: Option<String>,
    post_handoff_hash: Option<String>,
    stage_name: String,
    displaced_name: String,
}

struct PreparedRollbackEntry<'a> {
    index: usize,
    entry: &'a BackupEntry,
    target: PathBuf,
    backup: Option<PathBuf>,
    progress: RollbackEntryProgress,
    progress_path: PathBuf,
    stage: PathBuf,
    displaced: PathBuf,
}

fn restore_from_manifest_if_unchanged(
    manifest: &BackupManifest,
    backup_root: &Path,
) -> Result<ConditionalRollback> {
    let _lock = acquire_project_lock(&manifest.repo_root)?;
    let _vault_lock = acquire_project_lock(&manifest.vault_root)?;
    let _capsule_lock = lock_manifest_capsule(manifest)?;
    let canonical_vault_root = manifest.vault_root.canonicalize().with_context(|| {
        format!(
            "Could not resolve migration Vault root: {}",
            manifest.vault_root.display()
        )
    })?;
    let backup_metadata = fs::symlink_metadata(backup_root)?;
    if backup_metadata.file_type().is_symlink()
        || is_reparse_point(&backup_metadata)
        || !backup_metadata.is_dir()
    {
        bail!(
            "Migration backup root must be a regular directory: {}",
            backup_root.display()
        );
    }
    // Explicit rollback may receive a canonical Vault path while the manifest
    // stores the original lexical path (notably Windows \?\ paths). Compare
    // resolved roots, then reconstruct the validated in-Vault path.
    let canonical_backup_root = backup_root.canonicalize().with_context(|| {
        format!(
            "Could not resolve migration backup root: {}",
            backup_root.display()
        )
    })?;
    let backup_relative = canonical_backup_root
        .strip_prefix(&canonical_vault_root)
        .context("Migration backup root escapes its Vault")?;
    let backup_relative = normalize(backup_relative);
    let backup_root = validate_restore_target(
        &canonical_vault_root,
        &backup_relative,
        "rollback backup root",
    )?;
    if backup_root != canonical_backup_root {
        bail!("Migration backup root resolves through an unsafe path");
    }
    let mut lock_paths = vec![
        project_lock_path(&manifest.repo_root)?,
        project_lock_path(&manifest.vault_root)?,
    ];
    if let Some(relative) = manifest_capsule_relative(manifest) {
        let capsule = validate_restore_target(&manifest.vault_root, relative, "capsule lock")?;
        lock_paths.push(project_lock_path(capsule)?);
    }

    let mut conflict_paths = Vec::new();
    let mut plan = Vec::with_capacity(manifest.entries.len());
    for (index, entry) in manifest.entries.iter().enumerate() {
        let root = match entry.scope {
            BackupScope::Repo => &manifest.repo_root,
            BackupScope::Vault => &manifest.vault_root,
        };
        let target = validate_restore_target(root, &entry.relative_path, "rollback target")?;
        if is_mutation_lock_path(&target) || lock_paths.iter().any(|lock| lock.starts_with(&target))
        {
            bail!(
                "Migration restore target contains a live mutation lock: {}",
                target.display()
            );
        }

        let backup = if entry.existed {
            let scope_root = match entry.scope {
                BackupScope::Repo => backup_root.join("repo"),
                BackupScope::Vault => backup_root.join("vault"),
            };
            let backup = validate_restore_target(&scope_root, &entry.relative_path, "backup copy")?;
            if hash_path(&backup)? != entry.original_hash {
                bail!(
                    "Migration recovery copy is missing or changed: {}",
                    backup.display()
                );
            }
            Some(backup)
        } else {
            None
        };

        let progress = rollback_entry_progress(manifest, index, entry);
        let progress_relative = format!(
            "{}/rollback-progress/entry-{index:04}.json",
            backup_relative
        );
        let progress_path = validate_restore_target(
            &canonical_vault_root,
            &progress_relative,
            "rollback progress",
        )?;
        let stage = target
            .parent()
            .context("Migration restore target has no parent")?
            .join(&progress.stage_name);
        let displaced = target
            .parent()
            .context("Migration restore target has no parent")?
            .join(&progress.displaced_name);
        let progress_record = read_rollback_progress(&progress_path)?;
        if progress_record
            .as_ref()
            .is_some_and(|stored| stored != &progress)
        {
            bail!(
                "Migration rollback progress does not match manifest entry: {}",
                entry.relative_path
            );
        }
        let flags = rollback_progress_flags(&backup_root, index, &progress)?;
        if progress_record.is_none() && flags.any() {
            bail!(
                "Migration rollback progress marker has no entry record: {}",
                entry.relative_path
            );
        }
        let current_hash = hash_path(&target)?;
        if !rollback_entry_state_is_safe(
            entry,
            &current_hash,
            progress_record.is_some(),
            &flags,
            &stage,
            &displaced,
            &progress,
        )? {
            conflict_paths.push(format!(
                "{}:{}",
                match entry.scope {
                    BackupScope::Repo => "repo",
                    BackupScope::Vault => "vault",
                },
                entry.relative_path
            ));
        }
        plan.push(PreparedRollbackEntry {
            index,
            entry,
            target,
            backup,
            progress,
            progress_path,
            stage,
            displaced,
        });
    }
    if !conflict_paths.is_empty() {
        // Do not partially restore a multi-path migration when any path was
        // changed after the external handoff or outside a journaled restart
        // state. Leaving every path untouched avoids replacing a concurrent
        // writer's bytes with the pre-migration backup.
        return Ok(ConditionalRollback {
            restored: 0,
            conflicts: conflict_paths.len(),
            conflict_paths,
        });
    }

    let mut restored = 0;
    for item in &plan {
        if hash_path(&item.target)? != item.entry.original_hash {
            restored += 1;
        }
    }
    for item in plan.into_iter().rev() {
        restore_one_entry(&backup_root, item)?;
    }
    Ok(ConditionalRollback {
        restored,
        conflicts: 0,
        conflict_paths: Vec::new(),
    })
}

#[cfg(test)]
fn restore_from_manifest(manifest: &BackupManifest, backup_root: &Path) -> Result<usize> {
    let outcome = restore_from_manifest_if_unchanged(manifest, backup_root)?;
    if outcome.conflicts > 0 {
        bail!(
            "Migration restore found {} changed managed target(s): {}",
            outcome.conflicts,
            outcome.conflict_paths.join(", ")
        );
    }
    Ok(outcome.restored)
}

fn rollback_entry_progress(
    manifest: &BackupManifest,
    index: usize,
    entry: &BackupEntry,
) -> RollbackEntryProgress {
    let key = format!(
        "{}\0{}\0{}\0{}",
        manifest.migration_id,
        index,
        match entry.scope {
            BackupScope::Repo => "repo",
            BackupScope::Vault => "vault",
        },
        entry.relative_path
    );
    let token = format!("{:x}", Sha256::digest(key.as_bytes()));
    RollbackEntryProgress {
        schema_version: 1,
        migration_id: manifest.migration_id.clone(),
        scope: entry.scope,
        relative_path: entry.relative_path.clone(),
        existed: entry.existed,
        was_directory: entry.was_directory,
        original_hash: entry.original_hash.clone(),
        post_handoff_hash: entry.post_handoff_hash.clone(),
        stage_name: format!(".baron-rb-{token}.stage"),
        displaced_name: format!(".baron-rb-{token}.old"),
    }
}

#[derive(Default)]
struct RollbackProgressFlags {
    stage_ready: bool,
    target_moved: bool,
    displaced_cleanup_started: bool,
}

impl RollbackProgressFlags {
    fn any(&self) -> bool {
        self.stage_ready || self.target_moved || self.displaced_cleanup_started
    }
}

fn rollback_flag_path(backup_root: &Path, index: usize, phase: &str) -> PathBuf {
    backup_root
        .join("rollback-progress")
        .join(format!("entry-{index:04}.{phase}"))
}

fn rollback_progress_payload(progress: &RollbackEntryProgress, phase: &str) -> Result<Vec<u8>> {
    let record = serde_json::to_vec(progress)?;
    let fingerprint = format!("{:x}", Sha256::digest(record));
    Ok(format!("{phase}:{fingerprint}\n").into_bytes())
}

fn read_rollback_progress(path: &Path) -> Result<Option<RollbackEntryProgress>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink()
                || is_reparse_point(&metadata)
                || !metadata.is_file()
            {
                bail!(
                    "Migration rollback progress is not a regular file: {}",
                    path.display()
                );
            }
            let bytes = fs::read(path)?;
            Ok(Some(serde_json::from_slice(&bytes).with_context(|| {
                format!(
                    "Could not parse migration rollback progress: {}",
                    path.display()
                )
            })?))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn read_rollback_flag(path: &Path, progress: &RollbackEntryProgress, phase: &str) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink()
                || is_reparse_point(&metadata)
                || !metadata.is_file()
            {
                bail!(
                    "Migration rollback marker is not a regular file: {}",
                    path.display()
                );
            }
            let actual = fs::read(path)?;
            if actual != rollback_progress_payload(progress, phase)? {
                bail!("Migration rollback marker changed: {}", path.display());
            }
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn rollback_progress_flags(
    backup_root: &Path,
    index: usize,
    progress: &RollbackEntryProgress,
) -> Result<RollbackProgressFlags> {
    Ok(RollbackProgressFlags {
        stage_ready: read_rollback_flag(
            &rollback_flag_path(backup_root, index, "stage-ready"),
            progress,
            "stage-ready",
        )?,
        target_moved: read_rollback_flag(
            &rollback_flag_path(backup_root, index, "target-moved"),
            progress,
            "target-moved",
        )?,
        displaced_cleanup_started: read_rollback_flag(
            &rollback_flag_path(backup_root, index, "cleanup-started"),
            progress,
            "cleanup-started",
        )?,
    })
}

fn ensure_rollback_flag(
    backup_root: &Path,
    index: usize,
    progress: &RollbackEntryProgress,
    phase: &str,
) -> Result<()> {
    let path = rollback_flag_path(backup_root, index, phase);
    let content = rollback_progress_payload(progress, phase)?;
    ensure_directory_chain(path.parent().context("Rollback marker has no parent")?)?;
    match create_new_file(&path, &content) {
        Ok(()) => Ok(()),
        Err(error) => {
            if read_rollback_flag(&path, progress, phase)? {
                Ok(())
            } else {
                Err(error)
            }
        }
    }
}

fn rollback_entry_state_is_safe(
    entry: &BackupEntry,
    current_hash: &Option<String>,
    has_progress: bool,
    flags: &RollbackProgressFlags,
    stage: &Path,
    displaced: &Path,
    progress: &RollbackEntryProgress,
) -> Result<bool> {
    if has_progress {
        // Inspect all journal-owned paths during preflight, before any entry
        // is changed, so links and unsupported filesystem objects fail closed.
        let _ = hash_path(stage)?;
        let _ = hash_path(displaced)?;
    }
    // Cleanup starts after target movement, but finalization removes markers in
    // order. A restart between deleting target-moved and cleanup-started is
    // safe only after the original target is restored and both sidecars are
    // gone; all other out-of-order states remain conflicts.
    if flags.displaced_cleanup_started && !flags.target_moved {
        return Ok(current_hash == &entry.original_hash
            && !path_exists(stage)?
            && !path_exists(displaced)?);
    }
    if current_hash == &entry.original_hash {
        if !has_progress {
            return Ok(!path_exists(stage)? && !path_exists(displaced)?);
        }
        if !entry.existed {
            if flags.stage_ready || path_exists(stage)? {
                return Ok(false);
            }
            if flags.displaced_cleanup_started {
                // The journal proves recursive cleanup began. A restart may
                // finish deleting its remaining displaced directory entries.
                return Ok(true);
            }
            if path_exists(displaced)? {
                return Ok(entry.post_handoff_hash.is_some()
                    && hash_path(displaced)? == entry.post_handoff_hash);
            }
            return Ok(!flags.target_moved);
        }
        if flags.target_moved && !flags.displaced_cleanup_started {
            return Ok(false);
        }
        if flags.stage_ready {
            match hash_path(stage)? {
                Some(hash) if entry.original_hash.as_deref() == Some(hash.as_str()) => {}
                None => {} // The stage was renamed into the target.
                _ => return Ok(false),
            }
        }
        if path_exists(displaced)? && !flags.displaced_cleanup_started {
            return Ok(false);
        }
        return Ok(true);
    }

    if current_hash == &entry.post_handoff_hash {
        if !has_progress {
            return Ok(!path_exists(stage)? && !path_exists(displaced)?);
        }
        if flags.target_moved || flags.displaced_cleanup_started || path_exists(displaced)? {
            return Ok(false);
        }
        if flags.stage_ready {
            return Ok(hash_path(stage)? == entry.original_hash);
        }
        return Ok(true);
    }

    if current_hash.is_none() && has_progress && entry.existed && flags.stage_ready {
        if hash_path(stage)? != entry.original_hash {
            return Ok(false);
        }
        if flags.displaced_cleanup_started {
            return Ok(true);
        }
        return Ok(hash_path(displaced)? == entry.post_handoff_hash
            && progress.post_handoff_hash.is_some());
    }

    Ok(false)
}

fn path_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn ensure_rollback_progress(backup_root: &Path, item: &PreparedRollbackEntry<'_>) -> Result<()> {
    if let Some(stored) = read_rollback_progress(&item.progress_path)? {
        if stored != item.progress {
            bail!(
                "Migration rollback progress does not match manifest entry: {}",
                item.entry.relative_path
            );
        }
        return Ok(());
    }
    let flags = rollback_progress_flags(backup_root, item.index, &item.progress)?;
    if flags.any() || path_exists(&item.stage)? || path_exists(&item.displaced)? {
        bail!(
            "Migration rollback sidecar exists without progress ownership: {}",
            item.entry.relative_path
        );
    }
    ensure_directory_chain(
        item.progress_path
            .parent()
            .context("Rollback progress record has no parent")?,
    )?;
    let bytes = serde_json::to_vec(&item.progress)?;
    match create_new_file(&item.progress_path, &bytes) {
        Ok(()) => Ok(()),
        Err(error) => match read_rollback_progress(&item.progress_path)? {
            Some(stored) if stored == item.progress => Ok(()),
            _ => Err(error),
        },
    }
}

fn restore_one_entry(backup_root: &Path, item: PreparedRollbackEntry<'_>) -> Result<()> {
    let entry = item.entry;
    let current_hash = hash_path(&item.target)?;
    let has_progress = read_rollback_progress(&item.progress_path)?.is_some();
    let flags = rollback_progress_flags(backup_root, item.index, &item.progress)?;
    if current_hash == entry.original_hash {
        if has_progress {
            let displaced_is_pending =
                !entry.existed && !flags.displaced_cleanup_started && path_exists(&item.displaced)?;
            if !displaced_is_pending {
                finalize_rollback_entry(backup_root, &item, &flags)?;
                return Ok(());
            }
        } else {
            return Ok(());
        }
    }
    if current_hash != entry.post_handoff_hash
        && !(current_hash.is_none()
            && has_progress
            && (entry.existed && flags.stage_ready
                || !entry.existed
                    && entry.post_handoff_hash.is_some()
                    && hash_path(&item.displaced)? == entry.post_handoff_hash))
    {
        bail!(
            "Migration rollback target changed after preflight: {}",
            item.target.display()
        );
    }
    ensure_rollback_progress(backup_root, &item)?;
    let mut flags = rollback_progress_flags(backup_root, item.index, &item.progress)?;

    if entry.existed {
        let stage_hash = hash_path(&item.stage)?;
        if flags.stage_ready {
            if stage_hash != entry.original_hash {
                bail!(
                    "Staged migration recovery copy changed: {}",
                    item.stage.display()
                );
            }
        } else {
            if path_exists(&item.stage)? {
                remove_path(&item.stage)?;
            }
            copy_path(
                item.backup
                    .as_ref()
                    .context("Migration backup path missing for an existing entry")?,
                &item.stage,
                false,
                None,
                None,
            )?;
            if hash_path(&item.stage)? != entry.original_hash {
                bail!(
                    "Staged migration recovery copy failed validation: {}",
                    item.stage.display()
                );
            }
            ensure_rollback_flag(backup_root, item.index, &item.progress, "stage-ready")?;
            flags.stage_ready = true;
        }
    } else if path_exists(&item.stage)? {
        bail!(
            "Unexpected migration rollback stage: {}",
            item.stage.display()
        );
    }

    let current_hash = hash_path(&item.target)?;
    if current_hash == entry.post_handoff_hash && current_hash.is_some() {
        if path_exists(&item.displaced)? {
            bail!(
                "Unexpected migration rollback displaced path: {}",
                item.displaced.display()
            );
        }
        fs::rename(&item.target, &item.displaced).with_context(|| {
            format!("Could not stage rollback target: {}", item.target.display())
        })?;
        ensure_rollback_flag(backup_root, item.index, &item.progress, "target-moved")?;
        flags.target_moved = true;
    } else if current_hash.is_none() && entry.post_handoff_hash.is_some() {
        if !flags.displaced_cleanup_started {
            if hash_path(&item.displaced)? != entry.post_handoff_hash {
                bail!(
                    "Migration rollback lost its displaced target: {}",
                    item.target.display()
                );
            }
            ensure_rollback_flag(backup_root, item.index, &item.progress, "target-moved")?;
        }
    } else if current_hash != entry.post_handoff_hash {
        bail!(
            "Migration rollback target changed after staging: {}",
            item.target.display()
        );
    }

    if path_exists(&item.displaced)? {
        if !flags.displaced_cleanup_started {
            if hash_path(&item.displaced)? != entry.post_handoff_hash {
                bail!(
                    "Displaced migration target changed: {}",
                    item.displaced.display()
                );
            }
            ensure_rollback_flag(backup_root, item.index, &item.progress, "cleanup-started")?;
        }
        remove_path(&item.displaced)?;
    } else if entry.post_handoff_hash.is_some()
        && current_hash.is_none()
        && !flags.displaced_cleanup_started
    {
        bail!(
            "Migration rollback displaced target is missing: {}",
            item.target.display()
        );
    }

    if entry.existed {
        ensure_directory_chain(
            item.target
                .parent()
                .context("Migration restore target has no parent")?,
        )?;
        fs::rename(&item.stage, &item.target).with_context(|| {
            format!(
                "Could not publish restored migration target: {}",
                item.target.display()
            )
        })?;
        if hash_path(&item.target)? != entry.original_hash {
            bail!(
                "Restored migration target failed validation: {}",
                item.target.display()
            );
        }
    } else if hash_path(&item.target)?.is_some() {
        bail!(
            "New migration target remains after rollback: {}",
            item.target.display()
        );
    }

    let flags = rollback_progress_flags(backup_root, item.index, &item.progress)?;
    finalize_rollback_entry(backup_root, &item, &flags)
}

fn finalize_rollback_entry(
    backup_root: &Path,
    item: &PreparedRollbackEntry<'_>,
    flags: &RollbackProgressFlags,
) -> Result<()> {
    if path_exists(&item.stage)? {
        remove_path(&item.stage)?;
    }
    if path_exists(&item.displaced)? {
        if !flags.displaced_cleanup_started {
            bail!(
                "Migration rollback has an unjournaled displaced path: {}",
                item.displaced.display()
            );
        }
        remove_path(&item.displaced)?;
    }
    for phase in ["stage-ready", "target-moved", "cleanup-started"] {
        let path = rollback_flag_path(backup_root, item.index, phase);
        if path_exists(&path)? {
            fs::remove_file(&path)?;
        }
    }
    if path_exists(&item.progress_path)? {
        fs::remove_file(&item.progress_path)?;
    }
    Ok(())
}

fn validate_restore_target(root: &Path, relative: &str, label: &str) -> Result<PathBuf> {
    if !is_safe_relative_path(relative) {
        bail!("Unsafe {label} path in migration manifest: {relative}");
    }
    let root_metadata = fs::symlink_metadata(root).with_context(|| {
        format!(
            "Could not inspect migration {label} root: {}",
            root.display()
        )
    })?;
    if root_metadata.file_type().is_symlink() || is_reparse_point(&root_metadata) {
        bail!(
            "Migration {label} root cannot be a symlink or reparse point: {}",
            root.display()
        );
    }
    if !root_metadata.is_dir() {
        bail!(
            "Migration {label} root is not a directory: {}",
            root.display()
        );
    }
    let root = root.canonicalize().with_context(|| {
        format!(
            "Could not resolve migration {label} root: {}",
            root.display()
        )
    })?;
    let target = root.join(relative);
    if !target.starts_with(&root) {
        bail!("Migration {label} escapes its root: {}", target.display());
    }
    validate_existing_parent_chain(&root, target.parent())?;
    Ok(target)
}

fn validate_existing_parent_chain(root: &Path, parent: Option<&Path>) -> Result<()> {
    let Some(parent) = parent else {
        return Ok(());
    };
    let relative = parent.strip_prefix(root).with_context(|| {
        format!(
            "Migration restore parent escapes root: {}",
            parent.display()
        )
    })?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            bail!("Migration restore parent contains an unsafe path component");
        };
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
                    bail!(
                        "Migration restore parent cannot traverse a symlink or reparse point: {}",
                        current.display()
                    );
                }
                if !metadata.is_dir() {
                    bail!(
                        "Migration restore parent is not a directory: {}",
                        current.display()
                    );
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn scan_custom_skills(repo_root: &Path, items: &mut Vec<MigrationItem>) -> Result<()> {
    let root = repo_root.join(".codex/skills");
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if BUNDLED_SKILLS.contains(&name.as_str()) {
            continue;
        }
        let path = entry.path();
        let validation = validate_skill(&path);
        items.push(MigrationItem {
            relative_path: format!(".codex/skills/{name}"),
            kind: MigrationAssetKind::CustomSkill,
            action: if validation.is_ok() {
                MigrationAction::Import
            } else {
                MigrationAction::Quarantine
            },
            reason: validation
                .err()
                .unwrap_or_else(|| "custom skill satisfies the Baron contract".to_string()),
            content_hash: hash_path(&path)?,
        });
    }
    Ok(())
}

fn scan_custom_agents(repo_root: &Path, items: &mut Vec<MigrationItem>) -> Result<()> {
    let root = repo_root.join(".codex/agents");
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".toml") || CORE_AGENTS.contains(&name.as_str()) {
            continue;
        }
        let path = entry.path();
        let validation = validate_agent(&path);
        items.push(MigrationItem {
            relative_path: format!(".codex/agents/{name}"),
            kind: MigrationAssetKind::CustomAgent,
            action: if validation.is_ok() {
                MigrationAction::Import
            } else {
                MigrationAction::Quarantine
            },
            reason: validation
                .err()
                .unwrap_or_else(|| "custom agent satisfies the Baron contract".to_string()),
            content_hash: hash_path(&path)?,
        });
    }
    Ok(())
}

fn validate_skill(path: &Path) -> std::result::Result<(), String> {
    let skill = path.join("SKILL.md");
    let content =
        read_text_required(&skill).map_err(|_| "missing readable SKILL.md".to_string())?;
    let lower = content.to_lowercase();
    if !content.starts_with("---") || !lower.contains("\nname:") {
        return Err("skill frontmatter must declare name".to_string());
    }
    if !lower.contains("description: use when") {
        return Err("skill description must use a precise `Use when` trigger".to_string());
    }
    if lower.contains("agent-bootstrap") || lower.contains("agent bootstrap") {
        return Err("skill depends on Agent Bootstrap runtime or wording".to_string());
    }
    if lower.contains("replace superpowers") || lower.contains("workflow core") {
        return Err("skill conflicts with Superpowers workflow ownership".to_string());
    }
    Ok(())
}

fn validate_agent(path: &Path) -> std::result::Result<(), String> {
    let content = read_text_required(path).map_err(|_| "agent TOML is not readable".to_string())?;
    let parsed: toml::Value =
        toml::from_str(&content).map_err(|error| format!("invalid agent TOML: {error}"))?;
    for key in ["name", "description", "developer_instructions"] {
        if parsed.get(key).and_then(toml::Value::as_str).is_none() {
            return Err(format!("agent must declare `{key}`"));
        }
    }
    let lower = content.to_lowercase();
    if lower.contains("agent-bootstrap") || lower.contains("agent bootstrap") {
        return Err("agent depends on Agent Bootstrap runtime or wording".to_string());
    }
    if !lower.contains("evidence") {
        return Err("agent instructions must require evidence-backed output".to_string());
    }
    if !(lower.contains("do not orchestrate") || lower.contains("no subagent")) {
        return Err("agent instructions must prohibit recursive orchestration".to_string());
    }
    Ok(())
}

fn add_manifest_owned_assets(
    repo_root: &Path,
    manifest: &LegacyManifest,
    items: &mut Vec<MigrationItem>,
) -> Result<()> {
    for (relative, entry) in &manifest.entries {
        if entry.status != "managed" {
            continue;
        }
        if !is_safe_relative_path(relative) {
            continue;
        }
        let path = repo_root.join(relative);
        if !path.exists() {
            continue;
        }
        let current_hash = hash_path(&path)?;
        if current_hash.as_deref() != Some(&entry.synced_hash) {
            continue;
        }
        if items.iter().any(|item| item.relative_path == *relative) {
            continue;
        }
        items.push(MigrationItem {
            relative_path: relative.clone(),
            kind: MigrationAssetKind::ManagedAsset,
            action: MigrationAction::Remove,
            reason: "unmodified Agent Bootstrap managed asset".to_string(),
            content_hash: current_hash,
        });
    }
    Ok(())
}

fn add_data_root(
    items: &mut Vec<MigrationItem>,
    repo_root: &Path,
    relative: &str,
    kind: MigrationAssetKind,
    reason: &str,
) -> Result<()> {
    let path = repo_root.join(relative);
    if path.exists() {
        items.push(MigrationItem {
            relative_path: relative.to_string(),
            kind,
            action: MigrationAction::Import,
            reason: reason.to_string(),
            content_hash: hash_path(&path)?,
        });
    }
    Ok(())
}

fn push_file_item(
    items: &mut Vec<MigrationItem>,
    repo_root: &Path,
    relative: &str,
    kind: MigrationAssetKind,
    action: MigrationAction,
    reason: &str,
) -> Result<()> {
    let path = repo_root.join(relative);
    if path.exists() {
        items.push(MigrationItem {
            relative_path: relative.to_string(),
            kind,
            action,
            reason: reason.to_string(),
            content_hash: hash_path(&path)?,
        });
    }
    Ok(())
}

fn push_runtime_item(
    items: &mut Vec<MigrationItem>,
    repo_root: &Path,
    relative: &str,
    kind: MigrationAssetKind,
    reason: &str,
    managed_signatures: &[&str],
) -> Result<()> {
    let path = repo_root.join(relative);
    if !path.exists() {
        return Ok(());
    }
    let content = read_text_required(&path)?;
    let managed = managed_signatures
        .iter()
        .any(|signature| content.contains(signature));
    items.push(MigrationItem {
        relative_path: relative.to_string(),
        kind,
        action: if managed {
            MigrationAction::Remove
        } else {
            MigrationAction::Quarantine
        },
        reason: if managed {
            reason.to_string()
        } else {
            format!("{reason}; file was customized, so Baron preserves it in quarantine")
        },
        content_hash: hash_path(&path)?,
    });
    Ok(())
}

struct CopyPublication<'a> {
    backup_root: &'a Path,
    records: &'a mut Vec<ImportRecord>,
    manifest: &'a mut BackupManifest,
    capsule_root: Option<&'a Path>,
}

fn copy_path(
    source: &Path,
    destination: &Path,
    merge: bool,
    mut publication: Option<CopyPublication<'_>>,
    mutation_root: Option<&Path>,
) -> Result<()> {
    // A legacy capsule may carry an old lock marker. Lock files are live
    // coordination scaffolding, never imported or restored project data.
    if is_mutation_lock_path(destination) {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(source)
        .with_context(|| format!("Could not inspect migration source: {}", source.display()))?;
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        bail!(
            "Migration source cannot be a symlink or reparse point: {}",
            source.display()
        );
    }
    if metadata.is_dir() {
        ensure_directory_chain(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_path(
                &entry.path(),
                &destination.join(entry.file_name()),
                merge,
                publication.as_mut().map(|publication| CopyPublication {
                    backup_root: publication.backup_root,
                    records: &mut *publication.records,
                    manifest: &mut *publication.manifest,
                    capsule_root: publication.capsule_root,
                }),
                mutation_root,
            )?;
        }
        return Ok(());
    }
    if !metadata.is_file() {
        bail!(
            "Migration source is not a regular file: {}",
            source.display()
        );
    }
    if let Some(parent) = destination.parent() {
        ensure_directory_chain(parent)?;
    }
    // Directory traversal and the ordinary destination-parent scaffold happen
    // outside the critical section. The lock starts before the shared
    // destination read (hash/merge decision) and covers publication plus the
    // import record.
    // Read the source once, before destination locks. Both publication and its
    // expected hash use these exact bytes, even if the source later changes.
    let source_bytes = read_bytes(source)?
        .ok_or_else(|| anyhow::anyhow!("Migration source disappeared: {}", source.display()))?;
    let source_hash = format!("{:x}", Sha256::digest(&source_bytes));
    let _lock = mutation_root.map(acquire_project_lock).transpose()?;
    let _capsule_lock = publication
        .as_ref()
        .and_then(|publication| publication.capsule_root)
        .map(acquire_project_lock)
        .transpose()?;
    let (final_destination, output) = if merge && destination.exists() {
        let existing_hash = hash_file(destination)?;
        if existing_hash == source_hash {
            (destination.to_path_buf(), None)
        } else if is_markdown(source) && is_markdown(destination) {
            let source_content = std::str::from_utf8(&source_bytes)?;
            let destination_content = read_text_required(destination)?;
            let output = if is_placeholder_markdown(&destination_content) {
                Some(source_bytes.clone())
            } else if !destination_content.contains(source_content.trim()) {
                Some(
                    format!(
                        "{}\n\n<!-- BARON:LEGACY-IMPORT -->\n\n{}\n",
                        destination_content.trim_end(),
                        source_content.trim()
                    )
                    .into_bytes(),
                )
            } else {
                None
            };
            (destination.to_path_buf(), output)
        } else {
            let conflict = destination
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("LegacyImport")
                .join(
                    destination
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .as_ref(),
                );
            ensure_directory_chain(conflict.parent().unwrap())?;
            (conflict, Some(source_bytes.clone()))
        }
    } else {
        (destination.to_path_buf(), Some(source_bytes.clone()))
    };
    let entry = publication
        .as_mut()
        .map(|publication| {
            prepare_publication(
                publication.manifest,
                publication.backup_root,
                &final_destination,
            )
        })
        .transpose()?;
    let destination_hash = if let Some(output) = &output {
        atomic_write(&final_destination, output)?;
        if output == &source_bytes {
            if let Ok(metadata) = fs::symlink_metadata(source) {
                let _ = fs::set_permissions(&final_destination, metadata.permissions());
            }
        }
        format!("{:x}", Sha256::digest(output))
    } else {
        hash_file(&final_destination)?
    };
    if let Some(publication) = publication.as_mut() {
        publication.manifest.entries[entry.context("Missing migration publication entry")?]
            .post_handoff_hash = Some(destination_hash.clone());
        publication.records.push(ImportRecord {
            source: normalize(source),
            destination: normalize(&final_destination),
            source_hash,
            destination_hash,
        });
    }
    Ok(())
}

fn is_markdown(path: &Path) -> bool {
    path.extension().and_then(|value| value.to_str()) == Some("md")
}

fn is_placeholder_markdown(content: &str) -> bool {
    let meaningful = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>();
    meaningful.is_empty()
        || meaningful.iter().all(|line| {
            line.contains("stores durable memory")
                || line.contains("Only durable lessons")
                || line.contains("Candidates are not loaded")
        })
}

fn remove_legacy_managed_block(path: &Path, expected_hash: &str) -> Result<()> {
    let Some(bytes) = read_bytes(path)? else {
        return Ok(());
    };
    let observed_hash = format!("{:x}", Sha256::digest(&bytes));
    if observed_hash != expected_hash {
        bail!(
            "Migration AGENTS.md changed after installer capture; preserving it for recovery: {}",
            path.display()
        );
    }
    let content = String::from_utf8(bytes)
        .with_context(|| format!("Migration AGENTS.md is not valid UTF-8: {}", path.display()))?;
    let Some(start) = content.find(LEGACY_BLOCK_START) else {
        return Ok(());
    };
    let Some(end_offset) = content[start..].find(LEGACY_BLOCK_END) else {
        return Ok(());
    };
    let end = start + end_offset + LEGACY_BLOCK_END.len();
    let mut next = format!("{}{}", &content[..start], &content[end..]);
    while next.contains("\n\n\n") {
        next = next.replace("\n\n\n", "\n\n");
    }
    atomic_write(path, next.trim().as_bytes())
}

fn remove_path(path: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        bail!(
            "Refusing to remove a symlink or reparse point: {}",
            path.display()
        );
    }
    if metadata.is_dir() {
        fs::remove_dir_all(path)?;
    } else if metadata.is_file() {
        fs::remove_file(path)?;
    } else {
        bail!(
            "Refusing to remove unsupported filesystem entry: {}",
            path.display()
        );
    }
    Ok(())
}

fn remove_empty_parent(path: &Path, stop: &Path) -> Result<()> {
    if path == stop {
        return Ok(());
    }
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        bail!(
            "Refusing to inspect a symlink or reparse point: {}",
            path.display()
        );
    }
    if metadata.is_dir() && fs::read_dir(path)?.next().is_none() {
        fs::remove_dir(path)?;
    }
    Ok(())
}

fn file_contains(path: &Path, needle: &str) -> Result<bool> {
    Ok(read_text(path)?.is_some_and(|content| content.contains(needle)))
}

fn read_legacy_manifest(path: &Path) -> Result<LegacyManifest> {
    match read_text(path)? {
        Some(content) => serde_json::from_str(&content)
            .with_context(|| format!("Could not parse legacy manifest: {}", path.display())),
        None => Ok(LegacyManifest {
            entries: BTreeMap::new(),
        }),
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let content =
        read_text_required(path).with_context(|| format!("Could not read {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("Could not parse {}", path.display()))
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let content = serde_json::to_vec_pretty(value)?;
    atomic_write(path, &content)
}

fn write_state(repo_root: &Path, state: MigrationState) -> Result<()> {
    write_json(&repo_root.join(BARON_STATE), &state)
}

fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    replace_file(path, content)
}

fn is_mutation_lock_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case(".baron-mutation.lock"))
        && path
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(".baron"))
}

fn hash_path(path: &Path) -> Result<Option<String>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        bail!("Cannot hash a symlink or reparse point: {}", path.display());
    }
    if metadata.is_file() {
        return Ok(Some(hash_file(path)?));
    }
    if !metadata.is_dir() {
        bail!(
            "Cannot hash unsupported filesystem entry: {}",
            path.display()
        );
    }
    let mut files = Vec::new();
    collect_files(path, &mut files)?;
    files.sort();
    let mut hasher = Sha256::new();
    for file in files {
        hasher.update(normalize(file.strip_prefix(path).unwrap_or(&file)).as_bytes());
        hasher.update(hash_file(&file)?.as_bytes());
    }
    Ok(Some(format!("{:x}", hasher.finalize())))
}

fn hash_file(path: &Path) -> Result<String> {
    let content = read_bytes(path)?
        .ok_or_else(|| anyhow::anyhow!("File disappeared while hashing: {}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(content)))
}

fn collect_files(root: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    // Volatile lock markers do not belong in backup hashes or import sets.
    if is_mutation_lock_path(root) {
        return Ok(());
    }
    let metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        bail!(
            "Cannot traverse a symlink or reparse point: {}",
            root.display()
        );
    }
    if metadata.is_file() {
        output.push(root.to_path_buf());
        return Ok(());
    }
    if !metadata.is_dir() {
        bail!(
            "Cannot traverse unsupported filesystem entry: {}",
            root.display()
        );
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        collect_files(&entry.path(), output)?;
    }
    Ok(())
}

fn canonical_directory(path: &Path) -> Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Could not resolve repo path: {}", path.display()))?;
    if !canonical.is_dir() {
        bail!("Repo path is not a directory: {}", canonical.display());
    }
    Ok(canonical)
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

fn validate_source_project_root(vault_root: &Path, project_root: &Path) -> Result<()> {
    if !project_root.exists() || !vault_root.exists() {
        return Ok(());
    }
    let vault = vault_root
        .canonicalize()
        .with_context(|| format!("Could not resolve source Vault: {}", vault_root.display()))?;
    let project = project_root.canonicalize().with_context(|| {
        format!(
            "Could not resolve legacy project capsule: {}",
            project_root.display()
        )
    })?;
    if !project.starts_with(&vault) {
        bail!(
            "legacy project capsule is outside the configured source Vault: {}",
            project.display()
        );
    }
    Ok(())
}

fn is_safe_component(value: &str) -> bool {
    !value.trim().is_empty()
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        && !value.contains('/')
        && !value.contains('\\')
}

fn is_safe_relative_path(value: &str) -> bool {
    let path = Path::new(value);
    !path.as_os_str().is_empty()
        && !value.contains('\\')
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn migration_id(repo_root: &Path) -> String {
    format!(
        "{}-{}",
        Local::now().format("%Y%m%dT%H%M%S%3f"),
        project_slug(repo_root)
    )
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}

fn normalize(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{capsule_key, project_id_for_path};
    use tempfile::tempdir;

    #[test]
    fn migration_refuses_publication_when_backup_differs_from_captured_snapshot() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        ensure_directory_chain(&repo).unwrap();
        ensure_directory_chain(&vault).unwrap();
        let repo = repo.canonicalize().unwrap();
        let vault = vault.canonicalize().unwrap();
        let target = repo.join("plan.md");
        atomic_write(&target, b"original source bytes\n").unwrap();
        let backup_root = vault.join("Artifacts/Baron/Migrations/snapshot-race");
        let backup_repo = backup_root.join("repo");
        ensure_directory_chain(&backup_repo).unwrap();
        let mut entry = backup_entry(BackupScope::Repo, &repo, "plan.md", &backup_repo).unwrap();

        // Model a torn copy from the race where the source changed during
        // backup I/O and returned to its pre-copy bytes before publication.
        atomic_write(&backup_repo.join("plan.md"), b"mixed backup snapshot\n").unwrap();
        entry.post_handoff_hash = entry.original_hash.clone();
        let mut manifest = BackupManifest {
            migration_id: "snapshot-race".into(),
            repo_root: repo.clone(),
            vault_root: vault,
            post_handoff_captured: false,
            capsule_relative: None,
            entries: vec![entry],
        };

        let error = prepare_publication(&mut manifest, &backup_root, &target).unwrap_err();
        assert!(error.to_string().contains("backup snapshot"), "{error}");
        assert_eq!(
            read_bytes(&target).unwrap().unwrap(),
            b"original source bytes\n"
        );
    }

    #[test]
    fn pre_handoff_repo_writer_is_not_adopted_as_rollback_baseline() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        ensure_directory_chain(&repo).unwrap();
        ensure_directory_chain(&vault).unwrap();
        let repo = repo.canonicalize().unwrap();
        let source = repo.join("docs/superpowers/plans/plan.md");
        let target = repo.join("docs/baron/plans/plan.md");
        atomic_write(&source, b"# Imported plan\n").unwrap();
        let inventory = MigrationInventory {
            repo_root: repo.clone(),
            source_vault: vault.clone(),
            source_project_root: vault.join("Projects/legacy"),
            project_slug: "legacy".into(),
            items: Vec::new(),
        };
        let backup = vault.join("Artifacts/Baron/Migrations/test");
        let mut manifest = create_backup_manifest(&inventory, &vault, &backup, "test").unwrap();
        let mut records = Vec::new();
        import_repo_data(&repo, &backup, &mut records, &repo, &mut manifest).unwrap();
        assert_eq!(read_bytes(&target).unwrap().unwrap(), b"# Imported plan\n");
        // Deterministically schedule the writer in the gap between import and
        // handoff capture. These are the real publication/rollback functions.
        {
            let _writer = acquire_project_lock(&repo).unwrap();
            atomic_write(&target, b"# Foreign plan must survive\n").unwrap();
        }
        capture_post_handoff_state(&repo, &mut manifest).unwrap();
        let _rollback = acquire_project_lock(&repo).unwrap();
        let outcome = restore_from_manifest_if_unchanged(&manifest, &backup).unwrap();
        assert!(
            outcome.conflicts > 0,
            "foreign bytes require a rollback conflict"
        );
        assert_eq!(
            read_bytes(&target).unwrap().unwrap(),
            b"# Foreign plan must survive\n"
        );
    }

    #[test]
    fn rollback_resume_accepts_entries_already_restored_before_restart() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        ensure_directory_chain(&repo).unwrap();
        ensure_directory_chain(&vault).unwrap();
        let repo = repo.canonicalize().unwrap();
        let vault = vault.canonicalize().unwrap();
        let backup = vault.join("Artifacts/Baron/Migrations/restart-test");
        ensure_directory_chain(&backup).unwrap();

        let first = repo.join("first.txt");
        let second = repo.join("second.txt");
        atomic_write(&first, b"first original").unwrap();
        atomic_write(&second, b"second original").unwrap();
        let mut first_entry =
            backup_entry(BackupScope::Repo, &repo, "first.txt", &backup.join("repo")).unwrap();
        let mut second_entry =
            backup_entry(BackupScope::Repo, &repo, "second.txt", &backup.join("repo")).unwrap();
        atomic_write(&first, b"first after handoff").unwrap();
        atomic_write(&second, b"second after handoff").unwrap();
        first_entry.post_handoff_hash = hash_path(&first).unwrap();
        second_entry.post_handoff_hash = hash_path(&second).unwrap();

        // Model a process stop after the first path was atomically restored,
        // but before the second path was reached.
        atomic_write(&first, b"first original").unwrap();
        let manifest = BackupManifest {
            migration_id: "restart-test".into(),
            repo_root: repo,
            vault_root: vault,
            post_handoff_captured: true,
            capsule_relative: None,
            entries: vec![first_entry, second_entry],
        };

        let outcome = restore_from_manifest_if_unchanged(&manifest, &backup).unwrap();

        assert_eq!(outcome.conflicts, 0);
        assert_eq!(read_bytes(first).unwrap().unwrap(), b"first original");
        assert_eq!(read_bytes(second).unwrap().unwrap(), b"second original");
    }

    #[test]
    fn rollback_resumes_after_target_moved_marker_is_removed_before_cleanup_marker() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        ensure_directory_chain(&repo).unwrap();
        ensure_directory_chain(&vault).unwrap();
        let repo = repo.canonicalize().unwrap();
        let vault = vault.canonicalize().unwrap();
        let backup = vault.join("Artifacts/Baron/Migrations/finalizer-marker-boundary");
        ensure_directory_chain(&backup).unwrap();

        let target = repo.join("managed.txt");
        atomic_write(&target, b"original bytes").unwrap();
        let mut entry = backup_entry(
            BackupScope::Repo,
            &repo,
            "managed.txt",
            &backup.join("repo"),
        )
        .unwrap();
        atomic_write(&target, b"migration output").unwrap();
        entry.post_handoff_hash = hash_path(&target).unwrap();
        let manifest = BackupManifest {
            migration_id: "finalizer-marker-boundary".into(),
            repo_root: repo.clone(),
            vault_root: vault,
            post_handoff_captured: true,
            capsule_relative: None,
            entries: vec![entry],
        };

        let progress = rollback_entry_progress(&manifest, 0, &manifest.entries[0]);
        let progress_path = backup.join("rollback-progress/entry-0000.json");
        ensure_directory_chain(progress_path.parent().unwrap()).unwrap();
        create_new_file(&progress_path, &serde_json::to_vec(&progress).unwrap()).unwrap();
        ensure_rollback_flag(&backup, 0, &progress, "cleanup-started").unwrap();
        atomic_write(&target, b"original bytes").unwrap();

        let outcome = restore_from_manifest_if_unchanged(&manifest, &backup).unwrap();

        assert_eq!(outcome.conflicts, 0);
        assert_eq!(read_bytes(&target).unwrap().unwrap(), b"original bytes");
        assert!(!rollback_flag_path(&backup, 0, "target-moved").exists());
        assert!(!rollback_flag_path(&backup, 0, "cleanup-started").exists());
        assert!(!progress_path.exists());
    }

    #[test]
    fn rollback_resumes_directory_replacement_at_each_rename_cleanup_boundary() {
        for cleanup_started in [false, true] {
            let temp = tempdir().unwrap();
            let repo = temp.path().join("repo");
            let vault = temp.path().join("vault");
            ensure_directory_chain(&repo).unwrap();
            ensure_directory_chain(&vault).unwrap();
            let repo = repo.canonicalize().unwrap();
            let vault = vault.canonicalize().unwrap();
            let backup = vault.join("Artifacts/Baron/Migrations/directory-restart");
            ensure_directory_chain(&backup).unwrap();

            let target = repo.join("assets/managed");
            atomic_write(&target.join("original.md"), b"original directory content").unwrap();
            let mut entry = backup_entry(
                BackupScope::Repo,
                &repo,
                "assets/managed",
                &backup.join("repo"),
            )
            .unwrap();
            remove_path(&target).unwrap();
            atomic_write(&target.join("post-handoff.md"), b"managed migration output").unwrap();
            entry.post_handoff_hash = hash_path(&target).unwrap();
            let manifest = BackupManifest {
                migration_id: "directory-restart".into(),
                repo_root: repo.clone(),
                vault_root: vault.clone(),
                post_handoff_captured: true,
                capsule_relative: None,
                entries: vec![entry.clone()],
            };

            let progress = rollback_entry_progress(&manifest, 0, &entry);
            let progress_path = backup.join("rollback-progress/entry-0000.json");
            ensure_directory_chain(progress_path.parent().unwrap()).unwrap();
            create_new_file(&progress_path, &serde_json::to_vec(&progress).unwrap()).unwrap();
            let stage = target.parent().unwrap().join(&progress.stage_name);
            let displaced = target.parent().unwrap().join(&progress.displaced_name);
            copy_path(
                &backup.join("repo/assets/managed"),
                &stage,
                false,
                None,
                None,
            )
            .unwrap();
            ensure_rollback_flag(&backup, 0, &progress, "stage-ready").unwrap();
            fs::rename(&target, &displaced).unwrap();
            if cleanup_started {
                ensure_rollback_flag(&backup, 0, &progress, "target-moved").unwrap();
                ensure_rollback_flag(&backup, 0, &progress, "cleanup-started").unwrap();
                fs::remove_file(displaced.join("post-handoff.md")).unwrap();
            }

            let outcome = restore_from_manifest_if_unchanged(&manifest, &backup).unwrap();

            assert_eq!(outcome.conflicts, 0);
            assert_eq!(hash_path(&target).unwrap(), entry.original_hash);
            assert_eq!(
                read_bytes(target.join("original.md")).unwrap().unwrap(),
                b"original directory content"
            );
            assert!(!stage.exists());
            assert!(!displaced.exists());
            assert!(!progress_path.exists());
        }
    }

    #[test]
    fn rollback_resumes_absent_directory_after_rename_before_target_moved_marker() {
        for marker_phase in 0..3 {
            let temp = tempdir().unwrap();
            let repo = temp.path().join("repo");
            let vault = temp.path().join("vault");
            ensure_directory_chain(&repo).unwrap();
            ensure_directory_chain(&vault).unwrap();
            let repo = repo.canonicalize().unwrap();
            let vault = vault.canonicalize().unwrap();
            let migration_id = format!("absent-directory-restart-{marker_phase}");
            let backup = vault.join("Artifacts/Baron/Migrations").join(&migration_id);
            ensure_directory_chain(&backup).unwrap();

            let target = repo.join("assets/generated");
            let mut entry = backup_entry(
                BackupScope::Repo,
                &repo,
                "assets/generated",
                &backup.join("repo"),
            )
            .unwrap();
            assert!(!entry.existed);
            assert_eq!(entry.original_hash, None);
            atomic_write(&target.join("managed.md"), b"migration output").unwrap();
            entry.post_handoff_hash = hash_path(&target).unwrap();
            let manifest = BackupManifest {
                migration_id,
                repo_root: repo.clone(),
                vault_root: vault,
                post_handoff_captured: true,
                capsule_relative: None,
                entries: vec![entry.clone()],
            };

            // Model process stops after target rename, after its durable marker,
            // and after cleanup begins. Absence remains the original state.
            let progress = rollback_entry_progress(&manifest, 0, &entry);
            let progress_path = backup.join("rollback-progress/entry-0000.json");
            ensure_directory_chain(progress_path.parent().unwrap()).unwrap();
            create_new_file(&progress_path, &serde_json::to_vec(&progress).unwrap()).unwrap();
            let displaced = target.parent().unwrap().join(&progress.displaced_name);
            fs::rename(&target, &displaced).unwrap();
            if marker_phase >= 1 {
                ensure_rollback_flag(&backup, 0, &progress, "target-moved").unwrap();
            }
            if marker_phase >= 2 {
                ensure_rollback_flag(&backup, 0, &progress, "cleanup-started").unwrap();
                fs::remove_file(displaced.join("managed.md")).unwrap();
            }

            let outcome = restore_from_manifest_if_unchanged(&manifest, &backup).unwrap();

            assert_eq!(outcome.conflicts, 0, "restart boundary {marker_phase}");
            assert!(!target.exists(), "rollback restores the original absence");
            assert!(!displaced.exists());
            assert!(!progress_path.exists());
        }
    }

    #[test]
    fn backup_manifest_targets_the_identity_bound_capsule() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        ensure_directory_chain(&repo).unwrap();
        ensure_directory_chain(&vault).unwrap();
        let repo = repo.canonicalize().unwrap();
        let capsule = capsule_key(&project_slug(&repo), &project_id_for_path(&repo).unwrap());
        let relative = format!("Projects/{capsule}/Facts.md");
        atomic_write(&vault.join(&relative), b"# Existing memory\n").unwrap();
        let inventory = MigrationInventory {
            repo_root: repo,
            source_vault: vault.clone(),
            source_project_root: vault.join("Projects/legacy"),
            project_slug: "legacy".into(),
            items: Vec::new(),
        };
        let backup = vault.join("Artifacts/Baron/Migrations/test");
        let manifest = create_backup_manifest(&inventory, &vault, &backup, "test").unwrap();
        assert!(manifest.entries.iter().any(|entry| {
            matches!(entry.scope, BackupScope::Vault)
                && entry.relative_path == relative
                && entry.existed
        }));
        assert_eq!(
            read_bytes(backup.join("vault").join(relative))
                .unwrap()
                .unwrap(),
            b"# Existing memory\n"
        );
    }

    #[test]
    fn pre_handoff_capsule_writer_requires_a_rollback_conflict() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        ensure_directory_chain(&repo).unwrap();
        ensure_directory_chain(&vault).unwrap();
        let repo = repo.canonicalize().unwrap();
        let source = vault.join("Projects/legacy");
        atomic_write(&source.join("Facts.md"), b"# Imported memory\n").unwrap();
        let inventory = MigrationInventory {
            repo_root: repo.clone(),
            source_vault: vault.clone(),
            source_project_root: source.clone(),
            project_slug: "legacy".into(),
            items: Vec::new(),
        };
        let backup = vault.join("Artifacts/Baron/Migrations/test");
        let mut manifest = create_backup_manifest(&inventory, &vault, &backup, "test").unwrap();
        let destination = ensure_migration_vault(&vault, &mut manifest).unwrap();
        let mut records = Vec::new();
        import_legacy_vault(
            &source,
            &destination.project_root,
            &backup,
            &mut records,
            &repo,
            &mut manifest,
        )
        .unwrap();
        let target = destination.project_root.join("Facts.md");
        // Another checkout shares the capsule, but does not share this repo's
        // lock. Complete its write before handoff capture, without sleeps.
        let other_repo = temp.path().join("other-checkout");
        ensure_directory_chain(&other_repo).unwrap();
        {
            let _checkout = acquire_project_lock(&other_repo).unwrap();
            let _capsule = acquire_project_lock(&destination.project_root).unwrap();
            atomic_write(&target, b"# Foreign memory must survive\n").unwrap();
        }
        capture_post_handoff_state(&repo, &mut manifest).unwrap();
        let _rollback = acquire_project_lock(&repo).unwrap();
        let outcome = restore_from_manifest_if_unchanged(&manifest, &backup).unwrap();
        assert!(
            outcome.conflicts > 0,
            "shared capsule changes must be detected"
        );
        assert_eq!(
            read_bytes(&target).unwrap().unwrap(),
            b"# Foreign memory must survive\n"
        );
    }

    #[test]
    fn import_cannot_publish_while_another_checkout_holds_the_capsule_lock() {
        use std::sync::mpsc;

        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let other_repo = temp.path().join("other-checkout");
        let source = temp.path().join("source");
        let capsule = temp.path().join("capsule");
        ensure_directory_chain(&repo).unwrap();
        ensure_directory_chain(&other_repo).unwrap();
        atomic_write(&source.join("Facts.md"), b"# Imported memory\n").unwrap();
        atomic_write(&capsule.join("Facts.md"), b"# Existing memory\n").unwrap();
        let backup = temp.path().join("backup");
        let mut manifest = BackupManifest {
            migration_id: "test".into(),
            repo_root: repo.clone(),
            vault_root: temp.path().to_path_buf(),
            post_handoff_captured: false,
            capsule_relative: Some("capsule".into()),
            entries: vec![backup_entry(
                BackupScope::Vault,
                temp.path(),
                "capsule/Facts.md",
                &backup.join("vault"),
            )
            .unwrap()],
        };
        let (ready_tx, ready_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let held_capsule = capsule.clone();
        let holder = std::thread::spawn(move || {
            let _checkout = acquire_project_lock(&other_repo).unwrap();
            let _capsule = acquire_project_lock(&held_capsule).unwrap();
            ready_tx.send(()).unwrap();
            release_rx.recv().unwrap();
        });
        ready_rx.recv().unwrap();
        let result = import_legacy_vault(
            &source,
            &capsule,
            &backup,
            &mut Vec::new(),
            &repo,
            &mut manifest,
        );
        release_tx.send(()).unwrap();
        holder.join().unwrap();
        assert!(
            result.is_err(),
            "a held capsule lock must prevent publication"
        );
        assert_eq!(
            read_bytes(capsule.join("Facts.md")).unwrap().unwrap(),
            b"# Existing memory\n"
        );
    }

    #[test]
    fn importing_a_legacy_lock_file_preserves_the_live_capsule_lock() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        let source = vault.join("Projects/legacy");
        let capsule = vault.join("Projects/capsule");
        ensure_directory_chain(&repo).unwrap();
        ensure_directory_chain(&capsule).unwrap();
        atomic_write(&source.join("Facts.md"), b"# Legacy memory\n").unwrap();
        atomic_write(
            &source.join(".baron/.baron-mutation.lock"),
            b"legacy lock marker\n",
        )
        .unwrap();
        let backup = vault.join("backup");
        let mut manifest = BackupManifest {
            migration_id: "test".into(),
            repo_root: repo.clone(),
            vault_root: vault,
            post_handoff_captured: false,
            capsule_relative: Some("Projects/capsule".into()),
            entries: Vec::new(),
        };
        let _checkout = acquire_project_lock(&repo).unwrap();
        let _capsule = acquire_project_lock(&capsule).unwrap();
        let lock_path = capsule.join(".baron/.baron-mutation.lock");
        let mut records = Vec::new();
        import_legacy_vault(
            &source,
            &capsule,
            &backup,
            &mut records,
            &repo,
            &mut manifest,
        )
        .unwrap();
        assert!(lock_path.is_file());
        assert_eq!(
            read_bytes(capsule.join("Facts.md")).unwrap().unwrap(),
            b"# Legacy memory\n"
        );
        assert!(records
            .iter()
            .all(|record| !record.destination.ends_with(".baron-mutation.lock")));
        let contender = std::thread::spawn(move || {
            crate::safe_io::acquire_project_lock_with_timeout(
                capsule,
                std::time::Duration::from_millis(100),
            )
            .is_err()
        });
        assert!(
            contender.join().unwrap(),
            "the original lock must still exclude writers"
        );
    }

    #[test]
    fn legacy_directory_restore_fails_closed_before_removing_a_live_capsule_lock() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        let capsule = vault.join("Projects/capsule");
        ensure_directory_chain(&repo).unwrap();
        atomic_write(&capsule.join("Facts.md"), b"# Original memory\n").unwrap();
        let backup = vault.join("backup");
        let entry = backup_entry(
            BackupScope::Vault,
            &vault,
            "Projects/capsule",
            &backup.join("vault"),
        )
        .unwrap();
        // This is the older private manifest shape: no capsule_relative field.
        let manifest: BackupManifest = serde_json::from_value(serde_json::json!({
            "migration_id": "test",
            "repo_root": repo,
            "vault_root": vault,
            "post_handoff_captured": true,
            "entries": [entry]
        }))
        .unwrap();
        atomic_write(&capsule.join("Facts.md"), b"# Current memory\n").unwrap();
        let _checkout = acquire_project_lock(&repo).unwrap();
        let _capsule = acquire_project_lock(&capsule).unwrap();
        let lock_path = capsule.join(".baron/.baron-mutation.lock");
        let error = restore_from_manifest(&manifest, &backup).unwrap_err();
        assert!(error.to_string().contains("mutation lock"));
        assert!(lock_path.is_file());
        assert_eq!(
            read_bytes(capsule.join("Facts.md")).unwrap().unwrap(),
            b"# Current memory\n"
        );
    }
}
