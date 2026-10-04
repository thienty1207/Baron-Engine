use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::safe_io::{
    acquire_project_lock, ensure_directory_chain, project_lock_path, read_bytes, read_text,
    read_text_required, replace_file, ProjectMutationLock,
};
use crate::vault::{ensure_vault, project_slug, vault_context_without_create, VaultContext};

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
    reserve_backup_root(&inventory.repo_root, &backup_root)?;

    let mut backup_manifest =
        create_backup_manifest(&inventory, &destination_vault, &backup_root, &migration_id)?;
    write_json(&backup_root.join("manifest.json"), &backup_manifest)?;

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
        install_baron(&inventory.repo_root, &destination_vault)?;
        let _lock = acquire_project_lock(&inventory.repo_root)?;
        let _capsule_lock = lock_manifest_capsule(&backup_manifest)?;
        register_valid_custom_assets(&inventory)?;
        remove_legacy_managed_block(&inventory.repo_root.join("AGENTS.md"))?;
        let removed_count = cleanup_legacy_runtime(&inventory)?;
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
    let restored_count = restore_from_manifest(&manifest, &backup_root)?;
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
    let destination = vault_context_without_create(destination_vault, &inventory.repo_root)?;
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
    let destination_relative = destination_project
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
        // Back up each shared capsule file under checkout -> capsule locks;
        // source traversal and the complete backup scan stay outside them.
        let _lock = acquire_project_lock(&inventory.repo_root)?;
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
    let existed = source.exists();
    let was_directory = source.is_dir();
    let original_hash = hash_path(&source)?;
    if existed {
        copy_path(
            &source,
            &backup_scope_root.join(relative),
            false,
            None,
            None,
        )?;
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

fn register_valid_custom_assets(inventory: &MigrationInventory) -> Result<()> {
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
    append_custom_routes(
        &inventory.repo_root.join(".codex/agents/INDEX.md"),
        "Imported Custom Agents",
        valid_agents.iter().map(|item| {
            format!(
                "- `{}` - validated during migration",
                item.relative_path.trim_start_matches(".codex/agents/")
            )
        }),
    )
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

fn cleanup_legacy_runtime(inventory: &MigrationInventory) -> Result<usize> {
    let mut removed = 0;
    for item in &inventory.items {
        if item.action != MigrationAction::Remove {
            continue;
        }
        let path = inventory.repo_root.join(&item.relative_path);
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
    let _checkout = acquire_project_lock(&manifest.repo_root)?;
    let _capsule = lock_manifest_capsule(manifest)?;
    let _vault = acquire_project_lock(vault_root)?;
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
    for &index in &entries {
        let entry = &manifest.entries[index];
        if hash_path(&vault_root.join(&entry.relative_path))? != entry.original_hash {
            bail!("Migration setup baseline changed: {}", entry.relative_path);
        }
    }
    let destination = ensure_vault(vault_root, &manifest.repo_root)?;
    for index in entries {
        let entry = &mut manifest.entries[index];
        entry.post_handoff_hash = hash_path(&vault_root.join(&entry.relative_path))?;
    }
    Ok(destination)
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
    let _capsule_lock = lock_manifest_capsule(manifest)?;
    // Expected hashes were initialized from the backup and advanced only at
    // our own publications. A reread here would adopt a writer in the gap.
    manifest.post_handoff_captured = true;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct ConditionalRollback {
    restored: usize,
    conflicts: usize,
}

fn restore_from_manifest_if_unchanged(
    manifest: &BackupManifest,
    backup_root: &Path,
) -> Result<ConditionalRollback> {
    let _lock = acquire_project_lock(&manifest.repo_root)?;
    let _capsule_lock = lock_manifest_capsule(manifest)?;
    let _vault_lock = acquire_project_lock(&manifest.vault_root)?;
    let mut conflicts = 0;
    for entry in &manifest.entries {
        let root = match entry.scope {
            BackupScope::Repo => &manifest.repo_root,
            BackupScope::Vault => &manifest.vault_root,
        };
        let target = validate_restore_target(root, &entry.relative_path, "rollback target")?;
        if hash_path(&target)? != entry.post_handoff_hash {
            conflicts += 1;
        }
    }
    if conflicts > 0 {
        // Do not partially restore a multi-path migration when any path was
        // changed after the external handoff. Leaving every path untouched is
        // fail-closed and avoids replacing a concurrent writer's bytes with
        // the pre-migration backup.
        return Ok(ConditionalRollback {
            restored: 0,
            conflicts,
        });
    }
    Ok(ConditionalRollback {
        restored: restore_from_manifest(manifest, backup_root)?,
        conflicts: 0,
    })
}

fn restore_from_manifest(manifest: &BackupManifest, backup_root: &Path) -> Result<usize> {
    let _lock = acquire_project_lock(&manifest.repo_root)?;
    let _capsule_lock = lock_manifest_capsule(manifest)?;
    let _vault_lock = acquire_project_lock(&manifest.vault_root)?;
    let mut lock_paths = vec![
        project_lock_path(&manifest.repo_root)?,
        project_lock_path(&manifest.vault_root)?,
    ];
    if let Some(relative) = manifest_capsule_relative(manifest) {
        let capsule = validate_restore_target(&manifest.vault_root, relative, "capsule lock")?;
        lock_paths.push(project_lock_path(capsule)?);
    }
    let mut plan = Vec::with_capacity(manifest.entries.len());
    for entry in &manifest.entries {
        let (root, backup_scope) = match entry.scope {
            BackupScope::Repo => (&manifest.repo_root, backup_root.join("repo")),
            BackupScope::Vault => (&manifest.vault_root, backup_root.join("vault")),
        };
        let target = validate_restore_target(root, &entry.relative_path, "restore target")?;
        // Older manifests can contain an entire capsule directory. Never
        // remove/replace the inode backing a held lock, even during explicit
        // rollback; validate the complete restore set before touching data.
        if is_mutation_lock_path(&target) || lock_paths.iter().any(|lock| lock.starts_with(&target))
        {
            bail!(
                "Migration restore target contains a live mutation lock: {}",
                target.display()
            );
        }
        let backup = if entry.existed {
            Some(validate_restore_target(
                &backup_scope,
                &entry.relative_path,
                "backup copy",
            )?)
        } else {
            None
        };
        if let Some(backup) = &backup {
            if hash_path(backup)? != entry.original_hash {
                bail!(
                    "Migration recovery copy is missing or changed: {}",
                    backup.display()
                );
            }
        }
        plan.push((entry, target, backup));
    }

    let mut restored = 0;
    for (entry, target, backup) in plan.into_iter().rev() {
        if entry.existed {
            remove_path(&target)?;
            copy_path(
                backup
                    .as_ref()
                    .context("Migration backup path missing for an existing entry")?,
                &target,
                false,
                None,
                None,
            )?;
            restored += 1;
        } else {
            match fs::symlink_metadata(&target) {
                Ok(_) => {
                    remove_path(&target)?;
                    restored += 1;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
    }
    Ok(restored)
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

fn remove_legacy_managed_block(path: &Path) -> Result<()> {
    let Some(content) = read_text(path)? else {
        return Ok(());
    };
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
