use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::vault::{ensure_vault, project_slug};

const LEGACY_CONFIG: &str = "vault.config.json";
const LEGACY_MANIFEST: &str = ".agent-bootstrap-manifest.json";
const LEGACY_BLOCK_START: &str = "<!-- agent-bootstrap:start -->";
const LEGACY_BLOCK_END: &str = "<!-- agent-bootstrap:end -->";
const BARON_STATE: &str = ".baron/migration-state.json";

const BUNDLED_SKILLS: &[&str] = &["superpowers", "frontend-design", "vibe-security-scan"];
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
    entries: Vec<BackupEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupEntry {
    scope: BackupScope,
    relative_path: String,
    existed: bool,
    was_directory: bool,
    original_hash: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
    vault_override: Option<&Path>,
) -> Result<MigrationInventory> {
    let repo_root = canonical_directory(repo_path.as_ref())?;
    let config_path = repo_root.join(LEGACY_CONFIG);
    let config: LegacyConfig = read_json(&config_path).with_context(|| {
        format!(
            "Agent Bootstrap config not found or invalid at {}",
            config_path.display()
        )
    })?;
    let source_vault = vault_override
        .map(Path::to_path_buf)
        .unwrap_or_else(|| config.vault_root.clone());
    let source_project_root = config
        .project_root
        .clone()
        .unwrap_or_else(|| source_vault.join("Projects").join(&config.project_slug));
    let manifest = read_legacy_manifest(&repo_root.join(LEGACY_MANIFEST));
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
    let inventory = inventory_agent_bootstrap(repo_path, vault_override)?;
    let destination_vault = vault_override
        .map(Path::to_path_buf)
        .unwrap_or_else(|| inventory.source_vault.clone());
    let migration_id = migration_id(&inventory.repo_root);
    let backup_root = destination_vault
        .join("Artifacts/Baron/Migrations")
        .join(&migration_id);
    if backup_root.exists() {
        bail!("Migration backup already exists: {}", backup_root.display());
    }

    let backup_manifest =
        create_backup_manifest(&inventory, &destination_vault, &backup_root, &migration_id)?;
    write_json(&backup_root.join("manifest.json"), &backup_manifest)?;

    let result: Result<MigrationReceipt> = (|| {
        let destination = ensure_vault(&destination_vault, &inventory.repo_root)?;
        let mut import_records = Vec::new();
        import_legacy_vault(
            &inventory.source_project_root,
            &destination.project_root,
            &backup_root,
            &mut import_records,
        )?;
        import_repo_data(&inventory.repo_root, &backup_root, &mut import_records)?;
        let quarantined_count = quarantine_invalid_assets(&inventory, &backup_root, &migration_id)?;

        install_baron(&inventory.repo_root, &destination_vault)?;
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
            let rollback = restore_from_manifest(&backup_manifest, &backup_root);
            let failure = serde_json::json!({
                "migrationId": migration_id,
                "status": "rolled_back",
                "error": error.to_string(),
                "rollbackError": rollback.err().map(|value| value.to_string()),
                "updatedAt": now()
            });
            let _ = write_json(&backup_root.join("failure.json"), &failure);
            let _ = write_state(
                &inventory.repo_root,
                MigrationState {
                    migration_id,
                    status: "rolled_back".to_string(),
                    vault_root: destination_vault,
                    backup_root,
                    updated_at: now(),
                },
            );
            Err(error)
        }
    }
}

pub fn migration_status(repo_path: impl AsRef<Path>) -> Result<String> {
    let repo_root = canonical_directory(repo_path.as_ref())?;
    let state_path = repo_root.join(BARON_STATE);
    if !state_path.exists() {
        return Ok("# Baron Migration Status\n\n- Status: `never_run`\n".to_string());
    }
    let state: MigrationState = read_json(&state_path)?;
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
    let repo_root = canonical_directory(repo_path.as_ref())?;
    let backup_root = vault_path
        .as_ref()
        .join("Artifacts/Baron/Migrations")
        .join(migration_id);
    let manifest: BackupManifest = read_json(&backup_root.join("manifest.json"))?;
    if manifest.repo_root != repo_root {
        bail!(
            "Migration `{migration_id}` belongs to {}, not {}",
            manifest.repo_root.display(),
            repo_root.display()
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
    fs::create_dir_all(backup_root)?;
    let destination_slug = project_slug(&inventory.repo_root);
    let destination_project = destination_vault.join("Projects").join(destination_slug);
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
        "docs/baron/plans",
        "docs/baron/harness",
        "docs/baron/proofs",
        "docs/baron/traces",
    ] {
        repo_paths.insert(path.to_string());
    }
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
    if let Ok(relative) = inventory
        .source_project_root
        .strip_prefix(&inventory.source_vault)
    {
        vault_paths.insert(normalize(relative));
    }
    if let Ok(relative) = destination_project.strip_prefix(destination_vault) {
        vault_paths.insert(normalize(relative));
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
        entries,
    })
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
        copy_path(&source, &backup_scope_root.join(relative), false, None)?;
    }
    Ok(BackupEntry {
        scope,
        relative_path: relative.to_string(),
        existed,
        was_directory,
        original_hash,
    })
}

fn import_legacy_vault(
    source_root: &Path,
    destination_root: &Path,
    backup_root: &Path,
    records: &mut Vec<ImportRecord>,
) -> Result<()> {
    if !source_root.exists() || source_root == destination_root {
        return Ok(());
    }
    copy_path(
        source_root,
        destination_root,
        true,
        Some((backup_root, records)),
    )
}

fn import_repo_data(
    repo_root: &Path,
    backup_root: &Path,
    records: &mut Vec<ImportRecord>,
) -> Result<()> {
    for (source, destination) in [
        ("docs/superpowers/plans", "docs/baron/plans"),
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
                Some((backup_root, records)),
            )?;
        }
    }
    Ok(())
}

fn quarantine_invalid_assets(
    inventory: &MigrationInventory,
    backup_root: &Path,
    migration_id: &str,
) -> Result<usize> {
    let mut count = 0;
    for item in &inventory.items {
        if item.action != MigrationAction::Quarantine {
            continue;
        }
        let source = inventory.repo_root.join(&item.relative_path);
        if !source.exists() {
            continue;
        }
        let repo_quarantine = inventory
            .repo_root
            .join(".baron/quarantine")
            .join(migration_id)
            .join(&item.relative_path);
        let vault_quarantine = backup_root.join("quarantine").join(&item.relative_path);
        copy_path(&source, &repo_quarantine, false, None)?;
        copy_path(&source, &vault_quarantine, false, None)?;
        remove_path(&source)?;
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
    let mut content = fs::read_to_string(path).unwrap_or_default();
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
        let source = fs::read_to_string(&record.source).unwrap_or_default();
        let destination = fs::read_to_string(&record.destination).unwrap_or_default();
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
    if repo_root.join("scripts/agent-memory.js").exists() {
        bail!("Baron verification failed: legacy runtime still exists");
    }
    if repo_root.join(LEGACY_CONFIG).exists() {
        bail!("Baron verification failed: legacy config still exists");
    }
    Ok(())
}

fn restore_from_manifest(manifest: &BackupManifest, backup_root: &Path) -> Result<usize> {
    let mut restored = 0;
    for entry in manifest.entries.iter().rev() {
        let (root, backup_scope) = match entry.scope {
            BackupScope::Repo => (&manifest.repo_root, backup_root.join("repo")),
            BackupScope::Vault => (&manifest.vault_root, backup_root.join("vault")),
        };
        let target = root.join(&entry.relative_path);
        let backup = backup_scope.join(&entry.relative_path);
        if entry.existed {
            remove_path(&target)?;
            copy_path(&backup, &target, false, None)?;
            restored += 1;
        } else if target.exists() {
            remove_path(&target)?;
            restored += 1;
        }
    }
    Ok(restored)
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
        fs::read_to_string(&skill).map_err(|_| "missing readable SKILL.md".to_string())?;
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
    let content = fs::read_to_string(path).map_err(|_| "agent TOML is not readable".to_string())?;
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
    let content = fs::read_to_string(&path).unwrap_or_default();
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

fn copy_path(
    source: &Path,
    destination: &Path,
    merge: bool,
    mut records: Option<(&Path, &mut Vec<ImportRecord>)>,
) -> Result<()> {
    if source.is_dir() {
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_path(
                &entry.path(),
                &destination.join(entry.file_name()),
                merge,
                records
                    .as_mut()
                    .map(|(backup, records)| (*backup, &mut **records)),
            )?;
        }
        return Ok(());
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let source_hash = hash_file(source)?;
    let final_destination = if merge && destination.exists() {
        let existing_hash = hash_file(destination)?;
        if existing_hash == source_hash {
            destination.to_path_buf()
        } else if is_markdown(source) && is_markdown(destination) {
            let source_content = fs::read_to_string(source)?;
            let destination_content = fs::read_to_string(destination)?;
            if is_placeholder_markdown(&destination_content) {
                fs::copy(source, destination)?;
            } else if !destination_content.contains(source_content.trim()) {
                let merged = format!(
                    "{}\n\n<!-- BARON:LEGACY-IMPORT -->\n\n{}\n",
                    destination_content.trim_end(),
                    source_content.trim()
                );
                atomic_write(destination, merged.as_bytes())?;
            }
            destination.to_path_buf()
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
            fs::create_dir_all(conflict.parent().unwrap())?;
            fs::copy(source, &conflict)?;
            conflict
        }
    } else {
        fs::copy(source, destination)
            .with_context(|| format!("Could not copy {}", source.display()))?;
        destination.to_path_buf()
    };
    if let Some((_, records)) = records.as_mut() {
        records.push(ImportRecord {
            source: normalize(source),
            destination: normalize(&final_destination),
            source_hash,
            destination_hash: hash_file(&final_destination)?,
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
    if !path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(path)?;
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
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn remove_empty_parent(path: &Path, stop: &Path) -> Result<()> {
    if path != stop && path.is_dir() && fs::read_dir(path)?.next().is_none() {
        fs::remove_dir(path)?;
    }
    Ok(())
}

fn file_contains(path: &Path, needle: &str) -> Result<bool> {
    Ok(fs::read_to_string(path)
        .unwrap_or_default()
        .contains(needle))
}

fn read_legacy_manifest(path: &Path) -> LegacyManifest {
    read_json(path).unwrap_or(LegacyManifest {
        entries: BTreeMap::new(),
    })
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Could not read {}", path.display()))?;
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
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("baron-migration-tmp");
    fs::write(&temp, content)?;
    if path.exists() {
        remove_path(path)?;
    }
    fs::rename(&temp, path)?;
    Ok(())
}

fn hash_path(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    if path.is_file() {
        return Ok(Some(hash_file(path)?));
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
    let content = fs::read(path)?;
    Ok(format!("{:x}", Sha256::digest(content)))
}

fn collect_files(root: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    if root.is_file() {
        output.push(root.to_path_buf());
        return Ok(());
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
