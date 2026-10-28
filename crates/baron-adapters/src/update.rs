use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use baron_core::safe_io::{
    acquire_project_lock, read_bytes, read_text, read_text_required, replace_file, replace_text,
};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

use crate::managed::delimited_block_bounds;

const MANAGED_STATE_SCHEMA_VERSION: u32 = 2;
const LEGACY_MANAGED_STATE_SCHEMA_VERSION: u32 = 1;
const MANAGED_STATE_DIR: &str = ".baron/managed-state";
const MANAGED_BASE_DIR: &str = "base";
const MANAGED_MANIFEST: &str = "manifest.json";
const MANAGED_START: &str = "<!-- BARON:MANAGED:START -->";
const MANAGED_END: &str = "<!-- BARON:MANAGED:END -->";
const ROUTING_START: &str = "<!-- BARON:ROUTING:START -->";
const ROUTING_END: &str = "<!-- BARON:ROUTING:END -->";
const RETIRED_VIBE_CORE_PREFIX: &str = ".baron/core/skills/vibe-security-scan";
const RETIRED_VIBE_ARCHIVE_ROOT: &str = ".baron/managed-state/retired/vibe-security-scan";
const RETIRED_VIBE_RECEIPT: &str = ".baron/managed-state/retired-vibe-security.json";
const PRESERVED_PATH_PREVIEW_LIMIT: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedMergeKind {
    MarkerBlock,
    RoutingBlock,
    JsonOwnedEntries,
    FullText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedManagedAdapter {
    Codex,
    Claude,
}

impl SupportedManagedAdapter {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagedOwner {
    Core,
    Adapter(SupportedManagedAdapter),
    UnsupportedLegacy(String),
}

impl Default for ManagedOwner {
    fn default() -> Self {
        Self::UnsupportedLegacy(String::new())
    }
}

impl ManagedOwner {
    pub fn for_payload(adapter: &str, relative_path: &Path) -> Self {
        if is_core_path(relative_path) {
            return Self::Core;
        }
        match adapter {
            "codex" => Self::Adapter(SupportedManagedAdapter::Codex),
            "claude" => Self::Adapter(SupportedManagedAdapter::Claude),
            other => Self::UnsupportedLegacy(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Core => "core",
            Self::Adapter(adapter) => adapter.as_str(),
            Self::UnsupportedLegacy(name) => name,
        }
    }

    fn baseline_key(&self) -> String {
        match self {
            Self::Core => "core".to_string(),
            Self::Adapter(adapter) => adapter.as_str().to_string(),
            Self::UnsupportedLegacy(name) => {
                let normalized = name
                    .chars()
                    .map(|character| {
                        if character.is_ascii_alphanumeric() || character == '-' || character == '_'
                        {
                            character
                        } else {
                            '_'
                        }
                    })
                    .collect::<String>();
                format!("legacy-{normalized}")
            }
        }
    }

    fn adapter_alias(&self) -> String {
        self.as_str().to_string()
    }
}

impl Serialize for ManagedOwner {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ManagedOwner {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OwnerVisitor;

        impl<'de> Visitor<'de> for OwnerVisitor {
            type Value = ManagedOwner;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a managed owner string")
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value == "core" {
                    Ok(ManagedOwner::Core)
                } else if value == "codex" {
                    Ok(ManagedOwner::Adapter(SupportedManagedAdapter::Codex))
                } else if value == "claude" {
                    Ok(ManagedOwner::Adapter(SupportedManagedAdapter::Claude))
                } else if value.trim().is_empty() {
                    Err(E::custom("managed owner cannot be empty"))
                } else {
                    Ok(ManagedOwner::UnsupportedLegacy(value.to_string()))
                }
            }
        }

        deserializer.deserialize_string(OwnerVisitor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ManagedProvenance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_owner: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub ownership_migrated: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedAssetRecord {
    #[serde(default)]
    pub owner: ManagedOwner,
    #[serde(default, skip_serializing)]
    pub adapter: String,
    pub relative_path: PathBuf,
    pub base_sha256: String,
    pub installed_version: String,
    pub merge_kind: ManagedMergeKind,
    #[serde(default, skip_serializing_if = "is_default_provenance")]
    pub provenance: ManagedProvenance,
    #[serde(skip)]
    legacy_baseline_adapter: Option<String>,
}

fn is_default_provenance(value: &ManagedProvenance) -> bool {
    value == &ManagedProvenance::default()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedAssetPayload {
    pub adapter: String,
    pub relative_path: PathBuf,
    pub merge_kind: ManagedMergeKind,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedBaseline {
    pub schema_version: u32,
    pub installed_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum_writer_schema: Option<u32>,
    pub records: Vec<ManagedAssetRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateDisposition {
    TakeUpstream,
    KeepLocal,
    Identical,
    AutoMerge,
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedUpdateAction {
    pub adapter: String,
    pub relative_path: PathBuf,
    pub merge_kind: ManagedMergeKind,
    pub disposition: UpdateDisposition,
    pub resolved_content: Option<String>,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedUpdatePlan {
    pub actions: Vec<ManagedUpdateAction>,
    pub conflicts: Vec<PathBuf>,
    pub preserved_paths: Vec<String>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalReconcileReport {
    pub applied_paths: Vec<PathBuf>,
    pub conflicts: Vec<PathBuf>,
    pub preserved_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct VibeRetirementReceipt {
    schema_version: u32,
    legacy_skill_id: String,
    replacement_skill_id: String,
    #[serde(default)]
    retired_assets: Vec<VibeRetiredAsset>,
    #[serde(default)]
    pending_assets: Vec<VibeRetiredAsset>,
    #[serde(default)]
    review_required_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct VibeRetiredAsset {
    original_path: String,
    archived_path: String,
    sha256: String,
}

struct VibeRetirementStage {
    original_path: PathBuf,
    archived_path: PathBuf,
    expected_bytes: Vec<u8>,
    archive_existed: bool,
}

/// Evidence returned by the Phase 3 ownership migration. A migration report
/// is deliberately small: the manifest and byte-preserving filesystem changes
/// remain the durable evidence, while these paths make the caller's receipt
/// explainable without reopening the old adapter implementation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OwnershipMigrationReport {
    pub migrated_paths: Vec<PathBuf>,
    pub preserved_paths: Vec<String>,
    pub conflicts: Vec<PathBuf>,
    pub published_schema: Option<u32>,
}

impl ManagedUpdatePlan {
    pub fn action_for(&self, relative_path: &str) -> Option<&ManagedUpdateAction> {
        self.actions
            .iter()
            .find(|action| action.relative_path == Path::new(relative_path))
    }

    pub fn action_for_adapter(
        &self,
        adapter: &str,
        relative_path: &str,
    ) -> Option<&ManagedUpdateAction> {
        self.actions.iter().find(|action| {
            action.adapter == adapter && action.relative_path == Path::new(relative_path)
        })
    }
}

pub fn managed_state_dir(repo_root: impl AsRef<Path>) -> PathBuf {
    repo_root.as_ref().join(MANAGED_STATE_DIR)
}

/// Upgrade legacy managed-state ownership before an adapter writes anything.
///
/// Schema 1 identified every record by an adapter string. That was sufficient
/// for the old per-adapter trees, but it could not express that `.baron/core`
/// is the canonical owner. This migration keeps the old manifest readable,
/// inventories every effective live path, transfers verified unchanged legacy
/// skills into Core, and publishes one schema-2 manifest only after all
/// conflicts have been ruled out. A modified former managed skill is a hard
/// conflict: preserving its bytes is safer than guessing whether the user
/// intended a move.
pub fn migrate_managed_ownership(repo_root: impl AsRef<Path>) -> Result<OwnershipMigrationReport> {
    let _lock = acquire_project_lock(repo_root.as_ref())?;
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    if !managed_manifest_exists(&repo_root)? {
        return Ok(OwnershipMigrationReport::default());
    }

    let baseline = load_managed_baseline(&repo_root)?;
    if baseline.schema_version != LEGACY_MANAGED_STATE_SCHEMA_VERSION {
        return Ok(OwnershipMigrationReport {
            published_schema: Some(baseline.schema_version),
            ..OwnershipMigrationReport::default()
        });
    }

    let mut report = OwnershipMigrationReport::default();
    let mut records = Vec::with_capacity(baseline.records.len());
    let mut payloads = Vec::with_capacity(baseline.records.len());
    let mut live_copies = Vec::new();
    let mut legacy_removals = Vec::new();
    let mut target_paths = HashSet::new();

    for legacy_record in &baseline.records {
        let content = managed_baseline_content(&repo_root, legacy_record)?;
        let mut record = legacy_record.clone();
        record.legacy_baseline_adapter = None;
        record.provenance = ManagedProvenance::default();

        let old_owner = legacy_record.adapter.clone();
        if is_core_path(&legacy_record.relative_path) {
            record.owner = ManagedOwner::Core;
            record.adapter = ManagedOwner::Core.adapter_alias();
            record.provenance = ManagedProvenance {
                source_owner: Some(old_owner),
                ownership_migrated: true,
            };
            report
                .migrated_paths
                .push(legacy_record.relative_path.clone());
        } else if let Some(target) = canonical_skill_target(&legacy_record.relative_path) {
            let source_path = checked_repo_path(&repo_root, &legacy_record.relative_path)?;
            let live = read_bytes(&source_path)?;
            if let Some(live) = &live {
                if sha256_bytes(live) != legacy_record.base_sha256 {
                    bail!(
                        "Legacy managed skill conflict at `{}`: the former Baron-managed copy was modified; preserve it for review",
                        legacy_record.relative_path.display()
                    );
                }
                let canonical_path = checked_repo_path(&repo_root, &target)?;
                if let Some(existing) = read_bytes(&canonical_path)? {
                    if existing != *live {
                        bail!(
                            "Legacy managed skill conflict at `{}`: canonical Core target already contains different bytes",
                            target.display()
                        );
                    }
                } else {
                    live_copies.push((canonical_path, live.clone()));
                }
                legacy_removals.push((source_path, live.clone()));
                record.relative_path = target;
                record.owner = ManagedOwner::Core;
                record.adapter = ManagedOwner::Core.adapter_alias();
                record.provenance = ManagedProvenance {
                    source_owner: Some(old_owner),
                    ownership_migrated: true,
                };
                report
                    .migrated_paths
                    .push(legacy_record.relative_path.clone());
            } else {
                // If the process stopped after staging the Core copy but
                // before publishing the v2 manifest, the old source is gone
                // while the verified canonical bytes remain. Recognize that
                // recoverable intermediate state and finish the ownership
                // transfer on retry. A missing source without a matching
                // Core copy remains adapter-owned; migration must not invent
                // a new live file from the baseline alone.
                let canonical_path = checked_repo_path(&repo_root, &target)?;
                match read_bytes(&canonical_path)? {
                    Some(existing) if sha256_bytes(&existing) == legacy_record.base_sha256 => {
                        record.relative_path = target;
                        record.owner = ManagedOwner::Core;
                        record.adapter = ManagedOwner::Core.adapter_alias();
                        record.provenance = ManagedProvenance {
                            source_owner: Some(old_owner),
                            ownership_migrated: true,
                        };
                        report
                            .migrated_paths
                            .push(legacy_record.relative_path.clone());
                    }
                    Some(_) => {
                        bail!(
                            "Legacy managed skill conflict at `{}`: its source is missing and the staged Core target contains different bytes",
                            legacy_record.relative_path.display()
                        );
                    }
                    None => {
                        record.owner = ManagedOwner::for_payload(
                            &legacy_record.adapter,
                            &legacy_record.relative_path,
                        );
                        record.adapter = record.owner.adapter_alias();
                        report
                            .preserved_paths
                            .push(legacy_record.relative_path.to_string_lossy().to_string());
                    }
                }
            }
        } else {
            record.owner =
                ManagedOwner::for_payload(&legacy_record.adapter, &legacy_record.relative_path);
            record.adapter = record.owner.adapter_alias();
            report
                .preserved_paths
                .push(legacy_record.relative_path.to_string_lossy().to_string());
        }

        validate_relative_path(&record.relative_path)?;
        if !target_paths.insert(record.relative_path.clone()) {
            bail!(
                "Legacy ownership migration found duplicate live path `{}`; preserving the old manifest",
                record.relative_path.display()
            );
        }
        payloads.push(ManagedAssetPayload {
            adapter: record.adapter.clone(),
            relative_path: record.relative_path.clone(),
            merge_kind: record.merge_kind,
            content,
        });
        records.push(record);
    }

    let new_baseline = ManagedBaseline {
        schema_version: MANAGED_STATE_SCHEMA_VERSION,
        installed_version: baseline.installed_version,
        minimum_writer_schema: Some(MANAGED_STATE_SCHEMA_VERSION),
        records,
    };

    for (path, bytes) in &live_copies {
        replace_file(path, bytes)?;
    }

    let mut removed_legacy = Vec::new();
    for (path, expected) in &legacy_removals {
        match read_bytes(path)? {
            Some(current) if current == *expected => {
                if let Err(error) = fs::remove_file(path).with_context(|| {
                    format!(
                        "Could not retire migrated legacy managed skill: {}",
                        path.display()
                    )
                }) {
                    let restore = restore_legacy_files(&removed_legacy);
                    let rollback = rollback_live_copies(&live_copies);
                    return Err(error.context(format!(
                        "Legacy ownership migration stopped before publication; restore={:?}, live_rollback={:?}",
                        restore, rollback
                    )));
                }
                removed_legacy.push((path.clone(), expected.clone()));
            }
            Some(_) => {
                let restore = restore_legacy_files(&removed_legacy);
                let rollback = rollback_live_copies(&live_copies);
                return Err(anyhow!(
                    "Legacy managed skill changed during migration; preserving {} for review (restore={:?}, live_rollback={:?})",
                    path.display(),
                    restore,
                    rollback
                ));
            }
            None => {}
        }
    }

    if let Err(error) = write_baseline(&repo_root, &new_baseline, &payloads) {
        let restore = restore_legacy_files(&removed_legacy);
        let rollback = rollback_live_copies(&live_copies);
        return Err(error.context(format!(
            "Legacy ownership migration was rolled back (legacy_restore={:?}, live_rollback={:?})",
            restore, rollback
        )));
    }

    report.published_schema = Some(MANAGED_STATE_SCHEMA_VERSION);
    Ok(report)
}

fn restore_legacy_files(files: &[(PathBuf, Vec<u8>)]) -> Result<()> {
    for (path, content) in files.iter().rev() {
        replace_file(path, content)?;
    }
    Ok(())
}

fn canonical_skill_target(path: &Path) -> Option<PathBuf> {
    for prefix in [Path::new(".codex/skills"), Path::new(".claude/skills")] {
        if let Ok(relative) = path.strip_prefix(prefix) {
            if !relative.as_os_str().is_empty() {
                return Some(Path::new(".baron/core/skills").join(relative));
            }
        }
    }
    None
}

fn rollback_live_copies(copies: &[(PathBuf, Vec<u8>)]) -> Result<()> {
    for (path, expected) in copies.iter().rev() {
        match read_bytes(path)? {
            Some(current) if current == *expected => {
                fs::remove_file(path).with_context(|| {
                    format!("Could not remove staged Core target: {}", path.display())
                })?;
            }
            Some(_) => bail!(
                "Migrated Core target changed during rollback; preserving {}",
                path.display()
            ),
            None => {}
        }
    }
    Ok(())
}

fn managed_manifest_path(repo_root: &Path) -> Result<PathBuf> {
    checked_state_path(repo_root, Path::new(MANAGED_MANIFEST), false)
}

pub(crate) fn managed_manifest_exists(repo_root: &Path) -> Result<bool> {
    let path = managed_manifest_path(repo_root)?;
    match fs::metadata(&path) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| {
            format!(
                "Could not inspect managed baseline manifest: {}",
                path.display()
            )
        }),
    }
}

pub fn load_managed_baseline(repo_root: impl AsRef<Path>) -> Result<ManagedBaseline> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    let path = managed_manifest_path(&repo_root)?;
    let content = read_text_required(&path)
        .with_context(|| format!("Managed baseline manifest is missing: {}", path.display()))?;
    let mut baseline: ManagedBaseline = serde_json::from_str(&content)
        .with_context(|| format!("Managed baseline manifest is malformed: {}", path.display()))?;
    normalize_loaded_baseline(&mut baseline)?;
    validate_baseline(&baseline)?;
    for record in &baseline.records {
        let base_path = baseline_copy_path(&repo_root, record, false)?;
        if !is_regular_file(&base_path)? {
            bail!(
                "Managed baseline copy is missing for `{}`: {}",
                record.relative_path.display(),
                base_path.display()
            );
        }
        let content = read_text_required(&base_path).with_context(|| {
            format!(
                "Could not read managed baseline copy for `{}`: {}",
                record.relative_path.display(),
                base_path.display()
            )
        })?;
        if sha256(&content) != record.base_sha256 {
            bail!(
                "Managed baseline hash mismatch for `{}`; refuse to use a tampered merge ancestor",
                record.relative_path.display()
            );
        }
    }
    Ok(baseline)
}

fn normalize_loaded_baseline(baseline: &mut ManagedBaseline) -> Result<()> {
    match baseline.schema_version {
        LEGACY_MANAGED_STATE_SCHEMA_VERSION => {
            baseline.minimum_writer_schema = Some(MANAGED_STATE_SCHEMA_VERSION);
            for record in &mut baseline.records {
                let legacy_adapter = record.adapter.trim().to_string();
                if legacy_adapter.is_empty() {
                    bail!("Legacy managed record is missing its adapter owner");
                }
                record.owner = ManagedOwner::for_payload(&legacy_adapter, &record.relative_path);
                record.legacy_baseline_adapter = Some(legacy_adapter);
            }
        }
        MANAGED_STATE_SCHEMA_VERSION => {
            for record in &mut baseline.records {
                if matches!(&record.owner, ManagedOwner::UnsupportedLegacy(name) if name.is_empty())
                {
                    if record.adapter.trim().is_empty() {
                        bail!(
                            "Managed v2 record has neither an owner nor a legacy adapter: {}",
                            record.relative_path.display()
                        );
                    }
                    record.owner =
                        ManagedOwner::for_payload(&record.adapter, &record.relative_path);
                }
                if record.adapter.trim().is_empty() {
                    record.adapter = record.owner.adapter_alias();
                }
            }
        }
        other => bail!(
            "Unsupported managed baseline schema {}; expected {} or {}",
            other,
            LEGACY_MANAGED_STATE_SCHEMA_VERSION,
            MANAGED_STATE_SCHEMA_VERSION
        ),
    }
    Ok(())
}

/// Inspect an existing manifest before replacing it. This keeps a future
/// writer floor or malformed persisted document from being silently hidden by
/// a baseline rewrite. Baseline-copy availability remains the reader's job.
fn ensure_existing_manifest_writable(repo_root: &Path) -> Result<()> {
    if !managed_manifest_exists(repo_root)? {
        return Ok(());
    }
    let path = managed_manifest_path(repo_root)?;
    let content = read_text_required(&path)
        .with_context(|| format!("Managed baseline manifest is missing: {}", path.display()))?;
    let mut baseline: ManagedBaseline = serde_json::from_str(&content)
        .with_context(|| format!("Managed baseline manifest is malformed: {}", path.display()))?;
    normalize_loaded_baseline(&mut baseline)?;
    validate_baseline(&baseline)
}

pub fn managed_baseline_content(
    repo_root: impl AsRef<Path>,
    record: &ManagedAssetRecord,
) -> Result<String> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    let path = baseline_copy_path(&repo_root, record, false)?;
    read_text_required(&path).with_context(|| {
        format!(
            "Could not read managed baseline copy for `{}`: {}",
            record.relative_path.display(),
            path.display()
        )
    })
}

pub fn managed_target_path(repo_root: impl AsRef<Path>, relative_path: &Path) -> Result<PathBuf> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    checked_repo_path(&repo_root, relative_path)
}

pub fn record_managed_baseline(
    repo_root: impl AsRef<Path>,
    payloads: &[ManagedAssetPayload],
    installed_version: &str,
) -> Result<()> {
    let _lock = acquire_project_lock(repo_root.as_ref())?;
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    ensure_existing_manifest_writable(&repo_root)?;
    let baseline = baseline_from_payloads(payloads, installed_version)?;
    write_baseline(&repo_root, &baseline, payloads)
}

pub fn ensure_managed_baseline(
    repo_root: impl AsRef<Path>,
    payloads: &[ManagedAssetPayload],
    installed_version: &str,
) -> Result<()> {
    let _lock = acquire_project_lock(repo_root.as_ref())?;
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    migrate_managed_ownership(&repo_root)?;
    if managed_manifest_exists(&repo_root)? {
        let mut baseline = load_managed_baseline(&repo_root)?;
        let existing = baseline
            .records
            .iter()
            .map(|record| record.relative_path.clone())
            .collect::<HashSet<_>>();
        let additions = payloads
            .iter()
            .filter(|payload| !existing.contains(&payload.relative_path))
            .cloned()
            .collect::<Vec<_>>();
        if additions.is_empty() {
            return Ok(());
        }
        let additional_baseline = baseline_from_payloads(&additions, installed_version)?;
        baseline.records.extend(additional_baseline.records);
        baseline
            .records
            .sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        validate_baseline(&baseline)?;
        write_baseline(&repo_root, &baseline, &additions)
    } else {
        record_managed_baseline(&repo_root, payloads, installed_version)
    }
}

pub fn replace_managed_baseline(
    repo_root: impl AsRef<Path>,
    payloads: &[ManagedAssetPayload],
    installed_version: &str,
) -> Result<()> {
    let _lock = acquire_project_lock(repo_root.as_ref())?;
    record_managed_baseline(repo_root, payloads, installed_version)
}

pub fn plan_managed_update(
    repo_root: impl AsRef<Path>,
    upstream_payloads: &[ManagedAssetPayload],
) -> Result<ManagedUpdatePlan> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    let baseline = load_managed_baseline(&repo_root)?;
    validate_payloads(upstream_payloads)?;
    let upstream = upstream_payloads
        .iter()
        .map(|payload| (payload.relative_path.clone(), payload))
        .collect::<HashMap<_, _>>();
    let mut actions = Vec::new();
    let mut conflicts = Vec::new();
    let mut diagnostics = Vec::new();
    let baseline_keys = baseline
        .records
        .iter()
        .map(|record| record.relative_path.clone())
        .collect::<HashSet<_>>();

    for record in &baseline.records {
        let Some(payload) = upstream.get(&record.relative_path) else {
            diagnostics.push(format!(
                "No upstream managed payload exists for `{}`; preserving local content.",
                record.relative_path.display()
            ));
            actions.push(ManagedUpdateAction {
                adapter: record.adapter.clone(),
                relative_path: record.relative_path.clone(),
                merge_kind: record.merge_kind,
                disposition: UpdateDisposition::KeepLocal,
                resolved_content: None,
                diagnostic: Some("upstream managed asset is absent".to_string()),
            });
            continue;
        };
        if payload.merge_kind != record.merge_kind {
            let diagnostic = format!(
                "Managed merge policy changed for `{}`; staging a conflict instead of guessing.",
                record.relative_path.display()
            );
            diagnostics.push(diagnostic.clone());
            conflicts.push(record.relative_path.clone());
            actions.push(ManagedUpdateAction {
                adapter: record.adapter.clone(),
                relative_path: record.relative_path.clone(),
                merge_kind: record.merge_kind,
                disposition: UpdateDisposition::Conflict,
                resolved_content: None,
                diagnostic: Some(diagnostic),
            });
            continue;
        }
        let base = read_text_required(baseline_copy_path(&repo_root, record, false)?)?;
        let local_path = checked_repo_path(&repo_root, &record.relative_path)?;
        let local = read_text(&local_path)?.unwrap_or_default();
        let action = plan_one(record, &base, &local, &payload.content)?;
        if action.disposition == UpdateDisposition::Conflict {
            conflicts.push(record.relative_path.clone());
            if let Some(diagnostic) = &action.diagnostic {
                diagnostics.push(diagnostic.clone());
            }
        }
        actions.push(action);
    }

    for payload in upstream_payloads {
        if baseline_keys.contains(&payload.relative_path) {
            continue;
        }
        let action = plan_new_upstream_payload(&repo_root, payload)?;
        if action.disposition == UpdateDisposition::Conflict {
            conflicts.push(payload.relative_path.clone());
            if let Some(diagnostic) = &action.diagnostic {
                diagnostics.push(diagnostic.clone());
            }
        }
        actions.push(action);
    }

    actions.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    conflicts.sort();
    conflicts.dedup();
    let managed_paths = baseline
        .records
        .iter()
        .map(|record| record.relative_path.clone())
        .chain(
            upstream_payloads
                .iter()
                .map(|payload| payload.relative_path.clone()),
        )
        .collect::<HashSet<_>>();
    let preserved = collect_preserved_paths(&repo_root, &managed_paths)?;
    if preserved.truncated {
        diagnostics.push(format!(
            "Preserved-path preview reached its bounded limit of {PRESERVED_PATH_PREVIEW_LIMIT}; dependency and generated directories are skipped, and no unlisted user path is managed."
        ));
    }
    Ok(ManagedUpdatePlan {
        actions,
        conflicts,
        preserved_paths: preserved.paths,
        diagnostics,
    })
}

/// Repairs only the managed files represented by the currently embedded
/// Baron assets. It never downloads a release or changes the runtime. Any
/// ambiguous merge leaves every project file and baseline untouched.
pub fn reconcile_installed_managed_assets(
    repo_root: impl AsRef<Path>,
    upstream_payloads: &[ManagedAssetPayload],
    installed_version: &str,
) -> Result<LocalReconcileReport> {
    let _lock = acquire_project_lock(repo_root.as_ref())?;
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    migrate_managed_ownership(&repo_root)?;
    let plan = plan_managed_update(&repo_root, upstream_payloads)?;
    if !plan.conflicts.is_empty() {
        return Ok(LocalReconcileReport {
            applied_paths: Vec::new(),
            conflicts: plan.conflicts,
            preserved_paths: plan.preserved_paths,
        });
    }

    let payloads = upstream_payloads
        .iter()
        .map(|payload| (payload.relative_path.clone(), payload))
        .collect::<HashMap<_, _>>();
    let previous_baseline = load_managed_baseline(&repo_root)?;
    let previous_payloads = previous_baseline
        .records
        .iter()
        .map(|record| {
            Ok(ManagedAssetPayload {
                adapter: record.adapter.clone(),
                relative_path: record.relative_path.clone(),
                merge_kind: record.merge_kind,
                content: managed_baseline_content(&repo_root, record)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let mut seen_targets = HashSet::new();
    let mut rewrites = Vec::new();
    for action in &plan.actions {
        let Some(payload) = payloads.get(&action.relative_path) else {
            continue;
        };
        let target = checked_repo_path(&repo_root, &action.relative_path)?;
        let local = read_text(&target)?;
        let target_exists = local.is_some();
        let replacement = action.resolved_content.clone().or_else(|| {
            (!target_exists && action.disposition == UpdateDisposition::KeepLocal)
                .then(|| payload.content.clone())
        });
        let Some(replacement) = replacement else {
            continue;
        };
        if !seen_targets.insert(action.relative_path.clone()) {
            bail!(
                "Baron local reconciliation refuses duplicate target ownership: {}",
                action.relative_path.display()
            );
        }
        let previous = local.unwrap_or_default();
        if previous != replacement {
            rewrites.push((target, target_exists, previous, replacement));
        }
    }

    let mut applied = Vec::new();
    for (target, _existed, _previous, replacement) in &rewrites {
        if let Err(error) = ensure_safe_target_parent(&repo_root, target)
            .and_then(|_| atomic_write(target, replacement))
        {
            rollback_local_reconcile(&rewrites, &applied)?;
            return Err(error
                .context("Baron local reconciliation failed and restored prior managed targets"));
        }
        applied.push(target.clone());
    }

    if let Err(error) = replace_managed_baseline(&repo_root, upstream_payloads, installed_version) {
        let rollback_targets = rollback_local_reconcile(&rewrites, &applied);
        let rollback_baseline = replace_managed_baseline(
            &repo_root,
            &previous_payloads,
            &previous_baseline.installed_version,
        );
        return match (rollback_targets, rollback_baseline) {
            (Ok(()), Ok(())) => Err(error.context(
                "Baron local reconciliation could not publish its baseline and restored prior state",
            )),
            (target_error, baseline_error) => Err(error.context(format!(
                "Baron local reconciliation failed; target rollback: {}; baseline rollback: {}",
                target_error
                    .map(|_| "ok".to_string())
                    .unwrap_or_else(|rollback| rollback.to_string()),
                baseline_error
                    .map(|_| "ok".to_string())
                    .unwrap_or_else(|rollback| rollback.to_string())
            ))),
        };
    }

    Ok(LocalReconcileReport {
        applied_paths: applied,
        conflicts: Vec::new(),
        preserved_paths: plan.preserved_paths,
    })
}

/// Reconcile only the canonical Core payloads while preserving every existing
/// adapter record in the managed baseline. This keeps Core installation
/// independent from malformed or user-modified adapter integration files.
pub fn core_reconcile_managed_assets(
    repo_root: impl AsRef<Path>,
    upstream_payloads: &[ManagedAssetPayload],
    installed_version: &str,
) -> Result<LocalReconcileReport> {
    let _lock = acquire_project_lock(repo_root.as_ref())?;
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    migrate_managed_ownership(&repo_root)?;
    validate_payloads(upstream_payloads)?;
    let baseline = load_managed_baseline(&repo_root)?;
    let (mut retirement_receipt, previous_receipt_bytes) =
        load_vibe_retirement_receipt(&repo_root)?;
    let upstream = upstream_payloads
        .iter()
        .map(|payload| (payload.relative_path.clone(), payload))
        .collect::<HashMap<_, _>>();
    let core_records = baseline
        .records
        .iter()
        .filter(|record| is_core_path(&record.relative_path))
        .collect::<Vec<_>>();
    let baseline_core_paths = core_records
        .iter()
        .map(|record| record.relative_path.clone())
        .collect::<HashSet<_>>();

    let mut actions = Vec::new();
    let mut conflicts = Vec::new();
    let mut retirement_conflicts = Vec::new();
    let mut retained_legacy_records = HashSet::new();
    let mut retirement_stages = Vec::new();
    let mut recovered_retirements = Vec::new();
    for record in core_records {
        let Some(payload) = upstream.get(&record.relative_path) else {
            if is_retired_vibe_core_path(&record.relative_path) {
                let base = managed_baseline_content(&repo_root, record)?;
                let expected_bytes = base.into_bytes();
                let local_path = checked_repo_path(&repo_root, &record.relative_path)?;
                let archive_relative = retired_vibe_archive_path(&record.relative_path)?;
                let archive_path = checked_repo_path(&repo_root, &archive_relative)?;
                let local = read_bytes(&local_path)?;
                let archived = read_bytes(&archive_path)?;
                let receipt_asset = VibeRetiredAsset {
                    original_path: record.relative_path.to_string_lossy().replace('\\', "/"),
                    archived_path: archive_relative.to_string_lossy().replace('\\', "/"),
                    sha256: sha256_bytes(&expected_bytes),
                };

                match local {
                    None => match archived {
                        Some(bytes) if bytes == expected_bytes => {
                            recovered_retirements.push(receipt_asset);
                        }
                        Some(_) => {
                            retirement_conflicts.push(record.relative_path.clone());
                            retained_legacy_records.insert(record.relative_path.clone());
                        }
                        None => {
                            // The live file is already missing, but the managed baseline is
                            // still the only recoverable copy. Keep its record and require
                            // review instead of silently dropping that recovery source.
                            retirement_conflicts.push(record.relative_path.clone());
                            retained_legacy_records.insert(record.relative_path.clone());
                        }
                    },
                    Some(bytes) if bytes == expected_bytes => match archived {
                        None => retirement_stages.push(VibeRetirementStage {
                            original_path: local_path,
                            archived_path: archive_path,
                            expected_bytes,
                            archive_existed: false,
                        }),
                        Some(archived_bytes) if archived_bytes == expected_bytes => {
                            retirement_stages.push(VibeRetirementStage {
                                original_path: local_path,
                                archived_path: archive_path,
                                expected_bytes,
                                archive_existed: true,
                            });
                        }
                        Some(_) => {
                            retirement_conflicts.push(record.relative_path.clone());
                            retained_legacy_records.insert(record.relative_path.clone());
                        }
                    },
                    Some(_) => {
                        retirement_conflicts.push(record.relative_path.clone());
                        retained_legacy_records.insert(record.relative_path.clone());
                        if archived.as_deref() == Some(expected_bytes.as_slice()) {
                            recovered_retirements.push(receipt_asset);
                        }
                    }
                }
            }
            continue;
        };
        if payload.merge_kind != record.merge_kind {
            conflicts.push(record.relative_path.clone());
            continue;
        }
        let base = managed_baseline_content(&repo_root, record)?;
        let local_path = checked_repo_path(&repo_root, &record.relative_path)?;
        let local = read_text(&local_path)?;
        if local
            .as_deref()
            .is_some_and(|local| local != base && payload.content == base)
        {
            conflicts.push(record.relative_path.clone());
            continue;
        }
        let local = local.unwrap_or_default();
        let action = plan_one(record, &base, &local, &payload.content)?;
        if action.disposition == UpdateDisposition::Conflict {
            conflicts.push(record.relative_path.clone());
        }
        actions.push(action);
    }
    retirement_conflicts.extend(retired_vibe_files_without_baseline(
        &repo_root,
        &baseline_core_paths,
    )?);
    retirement_conflicts.sort();
    retirement_conflicts.dedup();
    for payload in upstream_payloads {
        if baseline_core_paths.contains(&payload.relative_path) {
            continue;
        }
        let action = plan_new_upstream_payload(&repo_root, payload)?;
        if action.disposition == UpdateDisposition::Conflict {
            conflicts.push(payload.relative_path.clone());
        }
        actions.push(action);
    }
    conflicts.extend(retirement_conflicts.iter().cloned());
    conflicts.sort();
    conflicts.dedup();
    let managed_paths = baseline
        .records
        .iter()
        .map(|record| record.relative_path.clone())
        .chain(
            upstream_payloads
                .iter()
                .map(|payload| payload.relative_path.clone()),
        )
        .collect::<HashSet<_>>();
    let preserved = collect_preserved_paths(&repo_root, &managed_paths)?;
    if conflicts
        .iter()
        .any(|path| !retirement_conflicts.contains(path))
    {
        retirement_receipt.review_required_paths = retirement_conflicts
            .iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect();
        if !retirement_receipt.review_required_paths.is_empty()
            || !retirement_receipt.retired_assets.is_empty()
            || !retirement_receipt.pending_assets.is_empty()
        {
            write_vibe_retirement_receipt(&repo_root, &retirement_receipt)?;
        }
        return Ok(LocalReconcileReport {
            applied_paths: Vec::new(),
            conflicts,
            preserved_paths: preserved.paths,
        });
    }

    actions.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    let mut pending_names = retirement_receipt
        .pending_assets
        .iter()
        .map(|item| item.original_path.clone())
        .collect::<HashSet<_>>();
    for item in &recovered_retirements {
        pending_names.remove(&item.original_path);
        retirement_receipt
            .pending_assets
            .retain(|pending| pending.original_path != item.original_path);
        if !retirement_receipt
            .retired_assets
            .iter()
            .any(|existing| existing.original_path == item.original_path)
        {
            retirement_receipt.retired_assets.push(item.clone());
        }
    }
    for stage in &retirement_stages {
        let original = stage
            .original_path
            .strip_prefix(&repo_root)
            .context("Retired Vibe source path escaped the repository")?
            .to_string_lossy()
            .replace('\\', "/");
        let archived = stage
            .archived_path
            .strip_prefix(&repo_root)
            .context("Retired Vibe archive path escaped the repository")?
            .to_string_lossy()
            .replace('\\', "/");
        let receipt_item = VibeRetiredAsset {
            original_path: original.clone(),
            archived_path: archived,
            sha256: sha256_bytes(&stage.expected_bytes),
        };
        let already_archived = retirement_receipt
            .retired_assets
            .iter()
            .any(|item| item.original_path == original);
        if !already_archived && pending_names.insert(original.clone()) {
            retirement_receipt.pending_assets.push(receipt_item);
        }
    }
    retirement_receipt.review_required_paths = retirement_conflicts
        .iter()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .collect();
    retirement_receipt
        .pending_assets
        .sort_by(|left, right| left.original_path.cmp(&right.original_path));
    retirement_receipt.review_required_paths.sort();
    retirement_receipt.review_required_paths.dedup();

    let payloads = upstream_payloads
        .iter()
        .map(|payload| (payload.relative_path.clone(), payload))
        .collect::<HashMap<_, _>>();
    let previous_payloads = baseline
        .records
        .iter()
        .map(|record| {
            Ok(ManagedAssetPayload {
                adapter: record.adapter.clone(),
                relative_path: record.relative_path.clone(),
                merge_kind: record.merge_kind,
                content: managed_baseline_content(&repo_root, record)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut seen_targets = HashSet::new();
    let mut rewrites = Vec::new();
    for action in &actions {
        let Some(payload) = payloads.get(&action.relative_path) else {
            continue;
        };
        let target = checked_repo_path(&repo_root, &action.relative_path)?;
        let local = read_text(&target)?;
        let target_exists = local.is_some();
        let replacement = action.resolved_content.clone().or_else(|| {
            (!target_exists && action.disposition == UpdateDisposition::KeepLocal)
                .then(|| payload.content.clone())
        });
        let Some(replacement) = replacement else {
            continue;
        };
        if !seen_targets.insert(action.relative_path.clone()) {
            bail!(
                "Baron Core reconciliation refuses duplicate target ownership: {}",
                action.relative_path.display()
            );
        }
        let previous = local.unwrap_or_default();
        if previous != replacement {
            rewrites.push((target, target_exists, previous, replacement));
        }
    }

    let mut applied = Vec::new();
    let mut applied_rewrites = Vec::new();
    for (target, _existed, _previous, replacement) in &rewrites {
        if let Err(error) = ensure_safe_target_parent(&repo_root, target)
            .and_then(|_| atomic_write(target, replacement))
        {
            rollback_local_reconcile(&rewrites, &applied_rewrites)?;
            return Err(error
                .context("Baron Core reconciliation failed and restored prior managed targets"));
        }
        applied.push(target.clone());
        applied_rewrites.push(target.clone());
    }

    if let Err(error) = write_vibe_retirement_receipt(&repo_root, &retirement_receipt) {
        let rollback_targets = rollback_local_reconcile(&rewrites, &applied_rewrites);
        return match rollback_targets {
            Ok(()) => Err(error.context("Could not publish Vibe retirement receipt; restored prior Core targets")),
            Err(rollback) => Err(error.context(format!(
                "Could not publish Vibe retirement receipt; Core target rollback also failed: {rollback:#}"
            ))),
        };
    }

    let mut staged = Vec::new();
    for stage in &retirement_stages {
        let stage_result = (|| -> Result<()> {
            ensure_safe_target_parent(&repo_root, &stage.archived_path)?;
            ensure_safe_target_parent(&repo_root, &stage.original_path)?;
            if read_bytes(&stage.original_path)?.as_deref() != Some(stage.expected_bytes.as_slice())
            {
                bail!(
                    "Retired Vibe asset changed during migration: {}",
                    stage.original_path.display()
                );
            }
            if stage.archive_existed {
                if read_bytes(&stage.archived_path)?.as_deref()
                    != Some(stage.expected_bytes.as_slice())
                {
                    bail!(
                        "Retired Vibe archive changed during migration: {}",
                        stage.archived_path.display()
                    );
                }
                fs::remove_file(&stage.original_path).with_context(|| {
                    format!(
                        "Could not retire previously archived Vibe asset: {}",
                        stage.original_path.display()
                    )
                })?;
            } else {
                copy_vibe_asset_without_replacing_archive(
                    &stage.archived_path,
                    &stage.expected_bytes,
                )?;
                if read_bytes(&stage.original_path)?.as_deref()
                    != Some(stage.expected_bytes.as_slice())
                {
                    bail!(
                        "Retired Vibe asset changed during migration: {}",
                        stage.original_path.display()
                    );
                }
                fs::remove_file(&stage.original_path).with_context(|| {
                    format!(
                        "Could not retire unchanged Vibe asset: {}",
                        stage.original_path.display()
                    )
                })?;
            }
            if read_bytes(&stage.archived_path)?.as_deref() != Some(stage.expected_bytes.as_slice())
            {
                restore_missing_vibe_source(&stage.original_path, &stage.expected_bytes)?;
                bail!(
                    "Vibe archive changed during retirement; restored the managed source: {}",
                    stage.original_path.display()
                );
            }
            if read_bytes(&stage.original_path)?.is_some() {
                bail!(
                    "Retired Vibe source reappeared during migration: {}",
                    stage.original_path.display()
                );
            }
            Ok(())
        })();
        if let Err(error) = stage_result {
            let rollback_staged = rollback_vibe_retirement_stages(&repo_root, &staged);
            let rollback_targets = rollback_local_reconcile(&rewrites, &applied_rewrites);
            let rollback_receipt =
                restore_vibe_retirement_receipt(&repo_root, previous_receipt_bytes.as_deref());
            return Err(error.context(format!(
                "Vibe asset retirement stopped and attempted rollback (staged={:?}, targets={:?}, receipt={:?})",
                rollback_staged,
                rollback_targets,
                rollback_receipt
            )));
        }
        staged.push(stage);
        applied.push(stage.original_path.clone());
    }

    let mut merged_payloads = previous_payloads
        .into_iter()
        .filter_map(|previous| {
            if is_core_path(&previous.relative_path) {
                payloads
                    .get(&previous.relative_path)
                    .map(|payload| (*payload).clone())
                    .or_else(|| {
                        retained_legacy_records
                            .contains(&previous.relative_path)
                            .then_some(previous)
                    })
            } else {
                Some(previous)
            }
        })
        .collect::<Vec<_>>();
    for payload in upstream_payloads {
        if !merged_payloads
            .iter()
            .any(|existing| existing.relative_path == payload.relative_path)
        {
            merged_payloads.push(payload.clone());
        }
    }
    if let Err(error) = replace_managed_baseline(&repo_root, &merged_payloads, installed_version) {
        let rollback_staged = rollback_vibe_retirement_stages(&repo_root, &staged);
        let rollback_targets = rollback_local_reconcile(&rewrites, &applied_rewrites);
        let rollback_baseline = replace_managed_baseline(
            &repo_root,
            &baseline_payloads(&repo_root, &baseline)?,
            &baseline.installed_version,
        );
        return match (rollback_staged, rollback_targets, rollback_baseline) {
            (Ok(()), Ok(()), Ok(())) => Err(error.context(
                "Baron Core reconciliation could not publish its baseline and restored prior state",
            )),
            (staged_error, target_error, baseline_error) => {
                let status = |result: &Result<()>| match result {
                    Ok(()) => "ok".to_string(),
                    Err(rollback) => format!("failed: {rollback:#}"),
                };
                Err(error.context(format!(
                    "Baron Core reconciliation failed; retired asset rollback: {}; target rollback: {}; baseline rollback: {}",
                    status(&staged_error),
                    status(&target_error),
                    status(&baseline_error)
                )))
            }
        };
    }
    for stage in &retirement_stages {
        let original = stage
            .original_path
            .strip_prefix(&repo_root)
            .context("Retired Vibe source path escaped the repository")?
            .to_string_lossy()
            .replace('\\', "/");
        let archived = stage
            .archived_path
            .strip_prefix(&repo_root)
            .context("Retired Vibe archive path escaped the repository")?
            .to_string_lossy()
            .replace('\\', "/");
        let item = VibeRetiredAsset {
            original_path: original.clone(),
            archived_path: archived,
            sha256: sha256_bytes(&stage.expected_bytes),
        };
        if !retirement_receipt
            .retired_assets
            .iter()
            .any(|existing| existing.original_path == item.original_path)
        {
            retirement_receipt.retired_assets.push(item);
        }
        retirement_receipt
            .pending_assets
            .retain(|pending| pending.original_path != original);
    }
    retirement_receipt
        .retired_assets
        .sort_by(|left, right| left.original_path.cmp(&right.original_path));
    retirement_receipt
        .pending_assets
        .sort_by(|left, right| left.original_path.cmp(&right.original_path));
    if let Err(error) = write_vibe_retirement_receipt(&repo_root, &retirement_receipt) {
        let rollback_staged = rollback_vibe_retirement_stages(&repo_root, &staged);
        let rollback_targets = rollback_local_reconcile(&rewrites, &applied_rewrites);
        let rollback_baseline = replace_managed_baseline(
            &repo_root,
            &baseline_payloads(&repo_root, &baseline)?,
            &baseline.installed_version,
        );
        let rollback_receipt =
            restore_vibe_retirement_receipt(&repo_root, previous_receipt_bytes.as_deref());
        return Err(error.context(format!(
            "Vibe retirement receipt finalization failed; rollback status staged={:?}, targets={:?}, baseline={:?}, receipt={:?}",
            rollback_staged,
            rollback_targets,
            rollback_baseline,
            rollback_receipt
        )));
    }
    Ok(LocalReconcileReport {
        applied_paths: applied,
        conflicts,
        preserved_paths: preserved.paths,
    })
}

fn rollback_vibe_retirement_stages(
    repo_root: &Path,
    staged: &[&VibeRetirementStage],
) -> Result<()> {
    let mut failures = Vec::new();
    for stage in staged.iter().rev() {
        let result = match read_bytes(&stage.original_path) {
            Ok(Some(bytes)) if bytes == stage.expected_bytes => Ok(()),
            Ok(Some(_)) => Err(anyhow!(
                "Retired Vibe source changed before rollback: {}",
                stage.original_path.display()
            )),
            Ok(None) => match read_bytes(&stage.archived_path) {
                Ok(Some(bytes)) if bytes == stage.expected_bytes => {
                    ensure_safe_target_parent(repo_root, &stage.original_path).and_then(|_| {
                        restore_missing_vibe_source(&stage.original_path, &stage.expected_bytes)
                    })
                }
                Ok(Some(_)) => Err(anyhow!(
                    "Retired Vibe archive changed before rollback: {}",
                    stage.archived_path.display()
                )),
                Ok(None) => Err(anyhow!(
                    "Retired Vibe archive disappeared before rollback: {}",
                    stage.archived_path.display()
                )),
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        };
        if let Err(error) = result {
            failures.push(format!("{}: {error:#}", stage.original_path.display()));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        bail!(
            "Could not restore all retired Vibe assets: {}",
            failures.join("; ")
        )
    }
}

fn copy_vibe_asset_without_replacing_archive(
    archive_path: &Path,
    expected_bytes: &[u8],
) -> Result<()> {
    let mut archive = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(archive_path)
        .with_context(|| {
            format!(
                "Could not create Vibe archive without replacing an existing target: {}",
                archive_path.display()
            )
        })?;
    archive.write_all(expected_bytes)?;
    archive.sync_all()?;
    drop(archive);
    if read_bytes(archive_path)?.as_deref() != Some(expected_bytes) {
        bail!(
            "Vibe archive changed while it was being written: {}",
            archive_path.display()
        );
    }
    Ok(())
}

fn restore_missing_vibe_source(source_path: &Path, expected_bytes: &[u8]) -> Result<()> {
    let mut source = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(source_path)
        .with_context(|| {
            format!(
                "Could not restore missing Vibe source without replacing a concurrent file: {}",
                source_path.display()
            )
        })?;
    source.write_all(expected_bytes)?;
    source.sync_all()?;
    Ok(())
}

fn baseline_payloads(
    repo_root: &Path,
    baseline: &ManagedBaseline,
) -> Result<Vec<ManagedAssetPayload>> {
    baseline
        .records
        .iter()
        .map(|record| {
            Ok(ManagedAssetPayload {
                adapter: record.adapter.clone(),
                relative_path: record.relative_path.clone(),
                merge_kind: record.merge_kind,
                content: managed_baseline_content(repo_root, record)?,
            })
        })
        .collect()
}

fn empty_vibe_retirement_receipt() -> VibeRetirementReceipt {
    VibeRetirementReceipt {
        schema_version: 1,
        legacy_skill_id: "vibe-security-scan".to_string(),
        replacement_skill_id: "security-audit".to_string(),
        retired_assets: Vec::new(),
        pending_assets: Vec::new(),
        review_required_paths: Vec::new(),
    }
}

fn is_retired_vibe_core_path(path: &Path) -> bool {
    path.starts_with(Path::new(RETIRED_VIBE_CORE_PREFIX))
}

fn retired_vibe_archive_path(relative_path: &Path) -> Result<PathBuf> {
    let suffix = relative_path
        .strip_prefix(RETIRED_VIBE_CORE_PREFIX)
        .context("Retired Vibe asset path is outside its exact managed root")?;
    if suffix.as_os_str().is_empty() {
        bail!("Retired Vibe asset path must identify a file below its managed root");
    }
    let archive = Path::new(RETIRED_VIBE_ARCHIVE_ROOT).join(suffix);
    validate_relative_path(&archive)?;
    Ok(archive)
}

fn load_vibe_retirement_receipt(
    repo_root: &Path,
) -> Result<(VibeRetirementReceipt, Option<Vec<u8>>)> {
    let path = checked_repo_path(repo_root, Path::new(RETIRED_VIBE_RECEIPT))?;
    let Some(bytes) = read_bytes(&path)? else {
        return Ok((empty_vibe_retirement_receipt(), None));
    };
    let receipt: VibeRetirementReceipt = serde_json::from_slice(&bytes)
        .context("Vibe retirement receipt is malformed; preserving it and failing closed")?;
    if receipt.schema_version != 1
        || receipt.legacy_skill_id != "vibe-security-scan"
        || receipt.replacement_skill_id != "security-audit"
    {
        bail!("Vibe retirement receipt has an unsupported identity or schema");
    }
    validate_vibe_retirement_receipt(&receipt)?;
    Ok((receipt, Some(bytes)))
}

fn validate_vibe_retirement_receipt(receipt: &VibeRetirementReceipt) -> Result<()> {
    let mut recorded_assets = HashSet::new();
    for asset in receipt
        .retired_assets
        .iter()
        .chain(receipt.pending_assets.iter())
    {
        let original = Path::new(&asset.original_path);
        validate_relative_path(original)?;
        if asset.original_path.contains('\\') || !is_retired_vibe_core_path(original) {
            bail!("Vibe retirement receipt contains a non-canonical legacy asset path");
        }
        let expected_archive = retired_vibe_archive_path(original)?
            .to_string_lossy()
            .replace('\\', "/");
        if asset.archived_path != expected_archive {
            bail!("Vibe retirement receipt archive path does not match its legacy asset");
        }
        if asset.sha256.len() != 64 || !asset.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            bail!("Vibe retirement receipt contains an invalid SHA-256 digest");
        }
        if !recorded_assets.insert(asset.original_path.as_str()) {
            bail!("Vibe retirement receipt contains a duplicate or overlapping asset state");
        }
    }

    let mut review_paths = HashSet::new();
    for path in &receipt.review_required_paths {
        let relative = Path::new(path);
        validate_relative_path(relative)?;
        if path.contains('\\') || !is_retired_vibe_core_path(relative) {
            bail!("Vibe retirement receipt contains a non-canonical review path");
        }
        if !review_paths.insert(path.as_str()) {
            bail!("Vibe retirement receipt contains a duplicate review path");
        }
    }
    Ok(())
}

fn write_vibe_retirement_receipt(repo_root: &Path, receipt: &VibeRetirementReceipt) -> Result<()> {
    validate_vibe_retirement_receipt(receipt)?;
    let target = checked_repo_path(repo_root, Path::new(RETIRED_VIBE_RECEIPT))?;
    ensure_safe_target_parent(repo_root, &target)?;
    let is_empty = receipt.retired_assets.is_empty()
        && receipt.pending_assets.is_empty()
        && receipt.review_required_paths.is_empty();
    match fs::symlink_metadata(&target) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {}
        Ok(_) => bail!("Vibe retirement receipt target is not a regular file"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && is_empty => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let mut bytes = serde_json::to_vec_pretty(receipt)?;
    bytes.push(b'\n');
    replace_file(&target, &bytes)
}

fn restore_vibe_retirement_receipt(repo_root: &Path, previous: Option<&[u8]>) -> Result<()> {
    let target = checked_repo_path(repo_root, Path::new(RETIRED_VIBE_RECEIPT))?;
    match previous {
        Some(bytes) => {
            ensure_safe_target_parent(repo_root, &target)?;
            replace_file(&target, bytes)
        }
        None => match fs::symlink_metadata(&target) {
            Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
                fs::remove_file(&target).with_context(|| {
                    format!(
                        "Could not roll back Vibe retirement receipt {}",
                        target.display()
                    )
                })
            }
            Ok(_) => bail!("Vibe retirement receipt became unsafe during rollback"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        },
    }
}

fn retired_vibe_files_without_baseline(
    repo_root: &Path,
    baseline_paths: &HashSet<PathBuf>,
) -> Result<Vec<PathBuf>> {
    let root_relative = Path::new(RETIRED_VIBE_CORE_PREFIX);
    let root = checked_repo_path(repo_root, root_relative)?;
    match fs::symlink_metadata(&root) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Ok(vec![root_relative.to_path_buf()]);
        }
        Ok(_) => {}
    }
    if is_link_or_reparse_point(&root)? {
        return Ok(vec![root_relative.to_path_buf()]);
    }
    let mut found = Vec::new();
    collect_retired_vibe_files(repo_root, &root, &mut found)?;
    found.retain(|path| !baseline_paths.contains(path));
    found.sort();
    found.dedup();
    Ok(found)
}

fn collect_retired_vibe_files(
    repo_root: &Path,
    directory: &Path,
    found: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() && !file_type.is_symlink() && !is_link_or_reparse_point(&path)? {
            collect_retired_vibe_files(repo_root, &path, found)?;
        } else {
            found.push(
                path.strip_prefix(repo_root)
                    .context("Retired Vibe asset scan escaped the repository")?
                    .to_path_buf(),
            );
        }
    }
    Ok(())
}

pub(crate) fn review_unmanaged_retired_vibe_assets(
    repo_root: impl AsRef<Path>,
    baseline_paths: &HashSet<PathBuf>,
) -> Result<Vec<PathBuf>> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    let conflicts = retired_vibe_files_without_baseline(&repo_root, baseline_paths)?;
    let (mut receipt, _) = load_vibe_retirement_receipt(&repo_root)?;
    receipt.review_required_paths = conflicts
        .iter()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .collect();
    write_vibe_retirement_receipt(&repo_root, &receipt)?;
    Ok(conflicts)
}

fn plan_new_upstream_payload(
    repo_root: &Path,
    payload: &ManagedAssetPayload,
) -> Result<ManagedUpdateAction> {
    let local_path = checked_repo_path(repo_root, &payload.relative_path)?;
    let local = read_text(&local_path).with_context(|| {
        format!(
            "Could not read existing local managed candidate: {}",
            local_path.display()
        )
    })?;
    let local_exists = local.is_some();
    let local = local.unwrap_or_default();
    let (disposition, resolved_content, diagnostic) = match payload.merge_kind {
        ManagedMergeKind::FullText if !local_exists => {
            (UpdateDisposition::TakeUpstream, Some(payload.content.clone()), None)
        }
        ManagedMergeKind::FullText if local == payload.content => {
            (UpdateDisposition::Identical, None, None)
        }
        ManagedMergeKind::FullText => (
            UpdateDisposition::Conflict,
            None,
            Some(format!(
                "A new Baron-managed full-text asset would overwrite existing local content at `{}`.",
                payload.relative_path.display()
            )),
        ),
        ManagedMergeKind::MarkerBlock => {
            let local_managed = managed_content_for_kind(&local, payload.merge_kind)?;
            if local_managed == payload.content {
                (UpdateDisposition::Identical, None, None)
            } else {
                (
                    UpdateDisposition::AutoMerge,
                    Some(replace_delimited_block(
                        &local,
                        MANAGED_START,
                        MANAGED_END,
                        &payload.content,
                    )?),
                    None,
                )
            }
        }
        ManagedMergeKind::RoutingBlock => {
            let local_managed = managed_content_for_kind(&local, payload.merge_kind)?;
            if local_managed == payload.content {
                (UpdateDisposition::Identical, None, None)
            } else {
                (
                    UpdateDisposition::AutoMerge,
                    Some(replace_delimited_block(
                        &local,
                        ROUTING_START,
                        ROUTING_END,
                        &payload.content,
                    )?),
                    None,
                )
            }
        }
        ManagedMergeKind::JsonOwnedEntries => {
            let local_managed = managed_content_for_kind(&local, payload.merge_kind)?;
            if local_managed == payload.content {
                (UpdateDisposition::Identical, None, None)
            } else {
                (
                    UpdateDisposition::AutoMerge,
                    Some(merge_owned_json(&local, &payload.content)?),
                    None,
                )
            }
        }
    };
    Ok(ManagedUpdateAction {
        adapter: payload.adapter.clone(),
        relative_path: payload.relative_path.clone(),
        merge_kind: payload.merge_kind,
        disposition,
        resolved_content,
        diagnostic,
    })
}

pub fn managed_content_for_kind(content: &str, merge_kind: ManagedMergeKind) -> Result<String> {
    match merge_kind {
        ManagedMergeKind::MarkerBlock => {
            extract_delimited_block(content, MANAGED_START, MANAGED_END)
        }
        ManagedMergeKind::RoutingBlock => {
            extract_delimited_block(content, ROUTING_START, ROUTING_END)
        }
        ManagedMergeKind::JsonOwnedEntries => owned_json_content(content),
        ManagedMergeKind::FullText => Ok(content.to_string()),
    }
}

fn plan_one(
    record: &ManagedAssetRecord,
    base: &str,
    local_full: &str,
    upstream_managed: &str,
) -> Result<ManagedUpdateAction> {
    let local_managed = managed_content_for_kind(local_full, record.merge_kind)?;
    let (disposition, resolved_content, diagnostic) = if local_managed == upstream_managed {
        (UpdateDisposition::Identical, None, None)
    } else if local_managed == base {
        let resolved = match record.merge_kind {
            ManagedMergeKind::MarkerBlock => {
                replace_delimited_block(local_full, MANAGED_START, MANAGED_END, upstream_managed)?
            }
            ManagedMergeKind::RoutingBlock => {
                replace_delimited_block(local_full, ROUTING_START, ROUTING_END, upstream_managed)?
            }
            ManagedMergeKind::JsonOwnedEntries => merge_owned_json(local_full, upstream_managed)?,
            ManagedMergeKind::FullText => upstream_managed.to_string(),
        };
        let disposition = match record.merge_kind {
            ManagedMergeKind::MarkerBlock
            | ManagedMergeKind::RoutingBlock
            | ManagedMergeKind::JsonOwnedEntries => UpdateDisposition::AutoMerge,
            ManagedMergeKind::FullText => UpdateDisposition::TakeUpstream,
        };
        (disposition, Some(resolved), None)
    } else if upstream_managed == base {
        (UpdateDisposition::KeepLocal, None, None)
    } else {
        let diagnostic = format!(
            "Both local and upstream changed the Baron-owned {} for `{}`.",
            merge_kind_label(record.merge_kind),
            record.relative_path.display()
        );
        (UpdateDisposition::Conflict, None, Some(diagnostic))
    };
    Ok(ManagedUpdateAction {
        adapter: record.adapter.clone(),
        relative_path: record.relative_path.clone(),
        merge_kind: record.merge_kind,
        disposition,
        resolved_content,
        diagnostic,
    })
}

fn baseline_from_payloads(
    payloads: &[ManagedAssetPayload],
    installed_version: &str,
) -> Result<ManagedBaseline> {
    validate_payloads(payloads)?;
    if installed_version.trim().is_empty() {
        bail!("Managed baseline installed version cannot be empty");
    }
    let mut records = payloads
        .iter()
        .map(|payload| ManagedAssetRecord {
            owner: ManagedOwner::for_payload(&payload.adapter, &payload.relative_path),
            adapter: payload.adapter.clone(),
            relative_path: payload.relative_path.clone(),
            base_sha256: sha256(&payload.content),
            installed_version: installed_version.to_string(),
            merge_kind: payload.merge_kind,
            provenance: ManagedProvenance::default(),
            legacy_baseline_adapter: None,
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(ManagedBaseline {
        schema_version: MANAGED_STATE_SCHEMA_VERSION,
        installed_version: installed_version.to_string(),
        minimum_writer_schema: Some(MANAGED_STATE_SCHEMA_VERSION),
        records,
    })
}

fn write_baseline(
    repo_root: &Path,
    baseline: &ManagedBaseline,
    payloads: &[ManagedAssetPayload],
) -> Result<()> {
    validate_baseline(baseline)?;
    validate_payloads(payloads)?;
    let manifest_path = checked_state_path(repo_root, Path::new(MANAGED_MANIFEST), true)?;
    let mut writes = Vec::new();
    for payload in payloads {
        let record = baseline
            .records
            .iter()
            .find(|record| record.relative_path == payload.relative_path)
            .ok_or_else(|| anyhow!("Managed baseline record is missing for payload"))?;
        let path = baseline_copy_path(repo_root, record, true)?;
        writes.push((path, payload.content.as_bytes().to_vec()));
    }
    let manifest = serde_json::to_string_pretty(baseline)?;
    writes.push((manifest_path, format!("{manifest}\n").into_bytes()));

    let previous = writes
        .iter()
        .map(|(path, _)| Ok((path.clone(), read_bytes(path)?)))
        .collect::<Result<Vec<_>>>()?;
    let mut applied = Vec::new();
    for (path, content) in &writes {
        if let Err(error) = baron_core::safe_io::replace_file(path, content) {
            let rollback = rollback_baseline_files(&previous, &applied);
            return match rollback {
                Ok(()) => {
                    Err(error.context("Managed baseline publication failed and was rolled back"))
                }
                Err(rollback_error) => Err(error.context(format!(
                    "Managed baseline publication failed; rollback also failed: {rollback_error:#}"
                ))),
            };
        }
        applied.push(path.clone());
    }
    Ok(())
}

fn rollback_baseline_files(
    previous: &[(PathBuf, Option<Vec<u8>>)],
    applied: &[PathBuf],
) -> Result<()> {
    let mut failures = Vec::new();
    for path in applied.iter().rev() {
        let Some((_, bytes)) = previous.iter().find(|(candidate, _)| candidate == path) else {
            failures.push(format!("missing rollback record for {}", path.display()));
            continue;
        };
        let result = match bytes {
            Some(bytes) => baron_core::safe_io::replace_file(path, bytes),
            None => match fs::symlink_metadata(path) {
                Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => Err(
                    anyhow!("Rollback target is not a regular file: {}", path.display()),
                ),
                Ok(_) => fs::remove_file(path).with_context(|| {
                    format!("Could not remove new baseline file: {}", path.display())
                }),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error.into()),
            },
        };
        if let Err(error) = result {
            failures.push(format!("{}: {error:#}", path.display()));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        bail!(
            "Managed baseline rollback could not restore all files: {}",
            failures.join("; ")
        )
    }
}

fn validate_baseline(baseline: &ManagedBaseline) -> Result<()> {
    if !matches!(
        baseline.schema_version,
        LEGACY_MANAGED_STATE_SCHEMA_VERSION | MANAGED_STATE_SCHEMA_VERSION
    ) {
        bail!(
            "Unsupported managed baseline schema {}; expected {} or {}",
            baseline.schema_version,
            LEGACY_MANAGED_STATE_SCHEMA_VERSION,
            MANAGED_STATE_SCHEMA_VERSION
        );
    }
    if baseline.schema_version == MANAGED_STATE_SCHEMA_VERSION
        && baseline.minimum_writer_schema != Some(MANAGED_STATE_SCHEMA_VERSION)
    {
        bail!(
            "Managed baseline schema {} requires minimum writer schema {}",
            MANAGED_STATE_SCHEMA_VERSION,
            MANAGED_STATE_SCHEMA_VERSION
        );
    }
    if baseline.installed_version.trim().is_empty() {
        bail!("Managed baseline installed version cannot be empty");
    }
    let mut seen = HashSet::new();
    for record in &baseline.records {
        validate_record_adapter(record)?;
        validate_relative_path(&record.relative_path)?;
        if record.base_sha256.len() != 64
            || !record
                .base_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            bail!(
                "Managed baseline hash for `{}` is invalid",
                record.relative_path.display()
            );
        }
        if !seen.insert(record.relative_path.clone()) {
            bail!(
                "Duplicate live managed owner for `{}`",
                record.relative_path.display()
            );
        }
        if is_core_path(&record.relative_path) && !matches!(record.owner, ManagedOwner::Core) {
            bail!(
                "Canonical Core path `{}` must be owned by Core",
                record.relative_path.display()
            );
        }
    }
    Ok(())
}

fn validate_payloads(payloads: &[ManagedAssetPayload]) -> Result<()> {
    let mut seen = HashSet::new();
    for payload in payloads {
        validate_payload_adapter(&payload.adapter)?;
        validate_relative_path(&payload.relative_path)?;
        if !seen.insert(payload.relative_path.clone()) {
            bail!(
                "Duplicate live managed ownership for `{}`",
                payload.relative_path.display()
            );
        }
    }
    Ok(())
}

fn validate_adapter(adapter: &str) -> Result<()> {
    if adapter.is_empty()
        || !adapter
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        bail!("Managed adapter name is invalid: `{adapter}`");
    }
    Ok(())
}

fn validate_payload_adapter(adapter: &str) -> Result<()> {
    validate_adapter(adapter).or_else(|_| validate_legacy_adapter_component(adapter))
}

fn validate_record_adapter(record: &ManagedAssetRecord) -> Result<()> {
    if matches!(record.owner, ManagedOwner::UnsupportedLegacy(_)) {
        let value = record
            .legacy_baseline_adapter
            .as_deref()
            .unwrap_or(record.owner.as_str());
        validate_legacy_adapter_component(value)
    } else {
        validate_adapter(&record.adapter)
    }
}

fn validate_legacy_adapter_component(adapter: &str) -> Result<()> {
    if adapter.trim().is_empty() {
        bail!("Legacy managed adapter name cannot be empty");
    }
    let mut components = Path::new(adapter).components();
    if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
        bail!("Legacy managed adapter name must be one safe path component");
    }
    Ok(())
}

fn validate_relative_path(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        bail!("Managed path must be a non-empty repository-relative path");
    }
    for component in path.components() {
        if !matches!(component, Component::Normal(_)) {
            bail!("Managed path escapes the repository: {}", path.display());
        }
    }
    Ok(())
}

fn is_core_path(path: &Path) -> bool {
    path == Path::new(".baron/core") || path.starts_with(".baron/core")
}

fn is_regular_file(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.is_file() && !metadata.file_type().is_symlink()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn canonical_repo_root(repo_root: &Path) -> Result<PathBuf> {
    repo_root
        .canonicalize()
        .with_context(|| format!("Could not resolve repository root: {}", repo_root.display()))
}

fn checked_repo_path(repo_root: &Path, relative_path: &Path) -> Result<PathBuf> {
    validate_relative_path(relative_path)?;
    let path = repo_root.join(relative_path);
    let mut current = repo_root.to_path_buf();
    for component in relative_path.components() {
        let Component::Normal(part) = component else {
            unreachable!("validated managed path")
        };
        current.push(part);
        if current.exists() && is_link_or_reparse_point(&current)? {
            bail!(
                "Managed path cannot traverse a symlink or junction: {}",
                relative_path.display()
            );
        }
    }
    Ok(path)
}

fn ensure_safe_target_parent(repo_root: &Path, target: &Path) -> Result<()> {
    let parent = target
        .parent()
        .context("Managed target has no parent directory")?;
    let relative = parent
        .strip_prefix(repo_root)
        .context("Managed target parent escaped the repository")?;
    let mut current = repo_root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            bail!("Managed target parent escapes the repository");
        };
        current.push(part);
        if current.exists() {
            if is_link_or_reparse_point(&current)? || !fs::metadata(&current)?.is_dir() {
                bail!("Managed target parent is unsafe: {}", current.display());
            }
        } else {
            fs::create_dir(&current).or_else(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    Ok(())
                } else {
                    Err(error)
                }
            })?;
            if is_link_or_reparse_point(&current)? || !fs::metadata(&current)?.is_dir() {
                bail!("Managed target parent became unsafe: {}", current.display());
            }
        }
    }
    Ok(())
}

fn rollback_local_reconcile(
    rewrites: &[(PathBuf, bool, String, String)],
    applied: &[PathBuf],
) -> Result<()> {
    let mut failures = Vec::new();
    for target in applied.iter().rev() {
        let Some((_, existed, previous, _)) = rewrites.iter().find(|rewrite| &rewrite.0 == target)
        else {
            failures.push(format!("missing rollback record for {}", target.display()));
            continue;
        };
        let result = if *existed {
            atomic_write(target, previous)
        } else {
            match fs::symlink_metadata(target) {
                Ok(metadata) if metadata.file_type().is_symlink() => bail!(
                    "Managed target became unsafe during rollback: {}",
                    target.display()
                ),
                Ok(metadata) if !metadata.is_file() => bail!(
                    "Managed target is not a regular file during rollback: {}",
                    target.display()
                ),
                Ok(_) => fs::remove_file(target).with_context(|| {
                    format!(
                        "Could not remove local reconciliation target: {}",
                        target.display()
                    )
                }),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error.into()),
            }
        };
        if let Err(error) = result {
            failures.push(format!("{}: {error:#}", target.display()));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        bail!(
            "Baron local reconciliation could not restore all managed targets: {}",
            failures.join("; ")
        )
    }
}

fn baseline_copy_path(
    repo_root: &Path,
    record: &ManagedAssetRecord,
    create_parent: bool,
) -> Result<PathBuf> {
    validate_record_adapter(record)?;
    validate_relative_path(&record.relative_path)?;
    let owner_key = record.owner.baseline_key();
    let storage_key = record
        .legacy_baseline_adapter
        .as_deref()
        .unwrap_or(&owner_key);
    checked_state_path(
        repo_root,
        &Path::new(MANAGED_BASE_DIR)
            .join(storage_key)
            .join(&record.relative_path),
        create_parent,
    )
}

fn checked_state_path(
    repo_root: &Path,
    state_relative: &Path,
    create_parent: bool,
) -> Result<PathBuf> {
    validate_relative_path(state_relative)?;
    let relative = Path::new(".baron")
        .join("managed-state")
        .join(state_relative);
    validate_relative_path(&relative)?;
    let components = relative.components().collect::<Vec<_>>();
    let mut current = repo_root.to_path_buf();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(part) = component else {
            unreachable!("validated managed state path")
        };
        current.push(part);
        let is_final = index + 1 == components.len();
        if current.exists() {
            if is_link_or_reparse_point(&current)? {
                bail!(
                    "Managed baseline state cannot traverse a symlink or junction: {}",
                    relative.display()
                );
            }
            if !is_final && !fs::metadata(&current)?.is_dir() {
                bail!(
                    "Managed baseline state parent is not a directory: {}",
                    current.display()
                );
            }
            continue;
        }
        if !create_parent || is_final {
            break;
        }
        match fs::create_dir(&current) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "Could not create managed baseline state directory: {}",
                        current.display()
                    )
                })
            }
        }
        if is_link_or_reparse_point(&current)? || !fs::metadata(&current)?.is_dir() {
            bail!(
                "Managed baseline state path became unsafe while creating: {}",
                current.display()
            );
        }
    }
    Ok(repo_root.join(relative))
}

fn is_link_or_reparse_point(path: &Path) -> Result<bool> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(true);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;

        Ok(metadata.file_attributes() & 0x0400 != 0)
    }
    #[cfg(not(windows))]
    Ok(false)
}

fn extract_delimited_block(content: &str, start: &str, end: &str) -> Result<String> {
    match delimited_block_bounds(content, start, end, "managed")? {
        Some((begin, finish)) => Ok(content[begin..finish + end.len()].to_string()),
        None => Ok(String::new()),
    }
}

fn replace_delimited_block(
    content: &str,
    start: &str,
    end: &str,
    replacement: &str,
) -> Result<String> {
    match delimited_block_bounds(content, start, end, "managed")? {
        Some((begin, finish)) => {
            let after = finish + end.len();
            Ok(format!(
                "{}{}{}",
                &content[..begin],
                replacement,
                &content[after..]
            ))
        }
        None if content.trim().is_empty() => Ok(format!("{replacement}\n")),
        None => Ok(format!("{}\n\n{replacement}\n", content.trim_end())),
    }
}

fn owned_json_content(content: &str) -> Result<String> {
    if content.trim().is_empty() {
        return Ok("{}".to_string());
    }
    let root: serde_json::Value =
        serde_json::from_str(content).context("Managed JSON content is malformed")?;
    let mut owned = serde_json::Map::new();
    let hook_entries = root
        .get("hooks")
        .and_then(serde_json::Value::as_object)
        .map(|hooks| {
            hooks
                .iter()
                .filter_map(|(event, entries)| {
                    let entries = entries.as_array()?;
                    let managed = entries
                        .iter()
                        .filter(|entry| entry.to_string().contains("baron automation hook"))
                        .cloned()
                        .collect::<Vec<_>>();
                    (!managed.is_empty())
                        .then(|| (event.clone(), serde_json::Value::Array(managed)))
                })
                .collect::<serde_json::Map<_, _>>()
        })
        .unwrap_or_default();
    owned.insert("hooks".to_string(), serde_json::Value::Object(hook_entries));
    Ok(serde_json::to_string(&serde_json::Value::Object(owned))?)
}

fn merge_owned_json(local_content: &str, upstream_owned: &str) -> Result<String> {
    let mut local: serde_json::Value = if local_content.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(local_content).context("Local managed JSON content is malformed")?
    };
    let upstream: serde_json::Value = serde_json::from_str(upstream_owned)
        .context("Upstream managed JSON content is malformed")?;
    let local_object = local
        .as_object_mut()
        .context("Local managed JSON content must be an object")?;
    let upstream_hooks = upstream
        .get("hooks")
        .and_then(serde_json::Value::as_object)
        .cloned()
        .unwrap_or_default();
    let hooks = local_object
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));
    let hooks = hooks
        .as_object_mut()
        .context("Local managed JSON hooks must be an object")?;
    let events = hooks.keys().cloned().collect::<Vec<_>>();
    for event in events {
        let Some(entries) = hooks
            .get_mut(&event)
            .and_then(serde_json::Value::as_array_mut)
        else {
            continue;
        };
        entries.retain(|entry| !entry.to_string().contains("baron automation hook"));
        if let Some(upstream_entries) = upstream_hooks
            .get(&event)
            .and_then(serde_json::Value::as_array)
        {
            entries.extend(upstream_entries.iter().cloned());
        }
    }
    for (event, entries) in upstream_hooks {
        if !hooks.contains_key(&event) {
            hooks.insert(event, entries);
        }
    }
    Ok(format!("{}\n", serde_json::to_string_pretty(&local)?))
}

struct PreservedPathPreview {
    paths: Vec<String>,
    truncated: bool,
}

fn collect_preserved_paths(
    repo_root: &Path,
    managed_paths: &HashSet<PathBuf>,
) -> Result<PreservedPathPreview> {
    let mut paths = BTreeSet::new();
    let mut truncated = false;
    collect_preserved_paths_recursive(
        repo_root,
        repo_root,
        managed_paths,
        &mut paths,
        &mut truncated,
    )?;
    Ok(PreservedPathPreview {
        paths: paths.into_iter().collect(),
        truncated,
    })
}

fn collect_preserved_paths_recursive(
    repo_root: &Path,
    current: &Path,
    managed_paths: &HashSet<PathBuf>,
    paths: &mut BTreeSet<String>,
    truncated: &mut bool,
) -> Result<()> {
    if *truncated {
        return Ok(());
    }
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(repo_root).unwrap().to_path_buf();
        if should_skip_preserved_path(&relative) {
            continue;
        }
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_preserved_paths_recursive(repo_root, &path, managed_paths, paths, truncated)?;
        } else if !managed_paths.contains(&relative) {
            paths.insert(relative.to_string_lossy().replace('\\', "/"));
            if paths.len() >= PRESERVED_PATH_PREVIEW_LIMIT {
                *truncated = true;
                return Ok(());
            }
        }
    }
    Ok(())
}

fn should_skip_preserved_path(relative: &Path) -> bool {
    if relative == ".git" || relative.starts_with(".baron/managed-state") {
        return true;
    }
    relative.file_name().is_some_and(|name| {
        matches!(
            name.to_string_lossy().as_ref(),
            "node_modules" | "target" | "dist" | "build" | ".next" | ".cache" | "vendor"
        )
    })
}

fn merge_kind_label(kind: ManagedMergeKind) -> &'static str {
    match kind {
        ManagedMergeKind::MarkerBlock => "marker block",
        ManagedMergeKind::RoutingBlock => "routing block",
        ManagedMergeKind::JsonOwnedEntries => "JSON-owned entries",
        ManagedMergeKind::FullText => "full managed file",
    }
}

fn sha256(content: &str) -> String {
    sha256_bytes(content.as_bytes())
}

fn sha256_bytes(content: &[u8]) -> String {
    Sha256::digest(content)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn atomic_write(path: &Path, content: &str) -> Result<()> {
    replace_text(path, content)?;
    #[cfg(unix)]
    if content.as_bytes().starts_with(b"#!") {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("Could not set executable mode on {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod vibe_retirement_io_tests {
    use super::*;

    #[test]
    fn archive_copy_and_rollback_source_restore_never_replace_existing_files() {
        let temp = tempfile::tempdir().unwrap();
        let archive = temp.path().join("archive.md");
        let source = temp.path().join("source.md");
        fs::write(&archive, b"existing archive").unwrap();
        fs::write(&source, b"existing local source").unwrap();

        assert!(copy_vibe_asset_without_replacing_archive(&archive, b"managed baseline").is_err());
        assert!(restore_missing_vibe_source(&source, b"managed baseline").is_err());
        assert_eq!(fs::read(&archive).unwrap(), b"existing archive");
        assert_eq!(fs::read(&source).unwrap(), b"existing local source");
    }
}
