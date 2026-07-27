use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use baron_adapters::{
    load_managed_baseline, managed_baseline_content, plan_managed_update, replace_managed_baseline,
    ManagedAssetPayload, ManagedMergeKind, UpdateDisposition,
};
use baron_core::config::{load_project_config, ProjectConfig};
use baron_core::release::sha256_file;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::self_update::UpdateCandidate;

const TRANSACTION_SCHEMA_VERSION: u32 = 1;
const STATE_FILE: &str = "state.json";
const STATE_SEAL_FILE: &str = "state.sha256";
const MAX_TRANSACTION_PACKETS: usize = 512;
const MAX_TRANSACTION_FILE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_TRANSACTION_DIRECTORIES: usize = 64;
static TRANSACTION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Discovered,
    Downloaded,
    Verified,
    Planned,
    Conflict,
    ProjectActivated,
    RuntimePending,
    Completed,
    RolledBack,
    Aborted,
}

impl TransactionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Discovered => "discovered",
            Self::Downloaded => "downloaded",
            Self::Verified => "verified",
            Self::Planned => "planned",
            Self::Conflict => "conflict",
            Self::ProjectActivated => "project_activated",
            Self::RuntimePending => "runtime_pending",
            Self::Completed => "completed",
            Self::RolledBack => "rolled_back",
            Self::Aborted => "aborted",
        }
    }

    fn allows_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Discovered, Self::Downloaded)
                | (Self::Downloaded, Self::Verified)
                | (Self::Verified, Self::Planned | Self::Conflict)
                | (Self::Conflict, Self::Planned | Self::Aborted)
                | (
                    Self::Planned,
                    Self::ProjectActivated | Self::RolledBack | Self::Aborted
                )
                | (
                    Self::ProjectActivated,
                    Self::RuntimePending | Self::RolledBack
                )
                | (Self::RuntimePending, Self::Completed | Self::RolledBack)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionPacket {
    pub adapter: String,
    pub relative_path: PathBuf,
    pub merge_kind: ManagedMergeKind,
    pub disposition: UpdateDisposition,
    pub target_existed: bool,
    pub has_upstream: bool,
    pub base_sha256: String,
    pub local_sha256: String,
    pub upstream_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateTransaction {
    pub schema_version: u32,
    pub transaction_id: String,
    pub repo_root_fingerprint: String,
    pub project_id: String,
    pub project_slug: String,
    pub adapters: Vec<String>,
    pub source_version: String,
    pub target_version: String,
    pub source_revision: String,
    pub target: String,
    pub candidate_relative_path: PathBuf,
    pub candidate_sha256: String,
    pub candidate_size_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_binary_path: Option<PathBuf>,
    pub status: TransactionStatus,
    pub packets: Vec<TransactionPacket>,
    pub last_checkpoint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionPaths {
    pub root: PathBuf,
    pub state_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingTransaction {
    pub state_path: PathBuf,
    pub transaction: UpdateTransaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyFailurePoint {
    AfterManagedStateBackup,
    BeforeManagedWrite(usize),
    AfterManagedWrite(usize),
    BeforeBaselineReplacement,
    AfterBaselineReplacement,
}

pub fn create_verified_transaction(
    repo_root: &Path,
    config: &ProjectConfig,
    candidate: &UpdateCandidate,
    source_version: &str,
    adapters: &[String],
) -> Result<(TransactionPaths, UpdateTransaction)> {
    let repo_root = canonical_repo_root(repo_root)?;
    validate_config(config)?;
    if source_version.trim().is_empty() {
        bail!("Baron update transaction source version cannot be empty");
    }
    let update_root = safe_update_root(&repo_root)?;
    let candidate_path = checked_existing_path(&update_root, &candidate.staged_path)?;
    let candidate_relative_path = candidate_path
        .strip_prefix(&update_root)
        .context("Verified candidate escaped the Baron update workspace")?
        .to_path_buf();
    validate_relative_path(&candidate_relative_path)?;
    let candidate_metadata = fs::metadata(&candidate_path)?;
    if !candidate_metadata.is_file()
        || candidate_metadata.len() != candidate.size_bytes
        || sha256_file(&candidate_path)? != candidate.sha256
    {
        bail!("Verified candidate no longer matches its recorded identity");
    }
    let adapters = normalize_adapters(adapters)?;
    let transaction_id = next_transaction_id()?;
    let transactions_root = safe_child_directory(&update_root, "transactions")?;
    let root = safe_child_directory(&transactions_root, &transaction_id)?;
    let paths = TransactionPaths {
        state_path: root.join(STATE_FILE),
        root,
    };
    let transaction = UpdateTransaction {
        schema_version: TRANSACTION_SCHEMA_VERSION,
        transaction_id,
        repo_root_fingerprint: path_fingerprint(&repo_root)?,
        project_id: config.project_id.clone(),
        project_slug: config.project_slug.clone(),
        adapters,
        source_version: source_version.to_string(),
        target_version: candidate.version.clone(),
        source_revision: candidate.source_revision.clone(),
        target: candidate.target.clone(),
        candidate_relative_path,
        candidate_sha256: candidate.sha256.clone(),
        candidate_size_bytes: candidate.size_bytes,
        runtime_binary_path: None,
        status: TransactionStatus::Verified,
        packets: Vec::new(),
        last_checkpoint: "candidate_verified".to_string(),
    };
    write_transaction(&paths, &transaction)?;
    Ok((paths, transaction))
}

pub fn plan_candidate_transaction(
    repo_root: &Path,
    state_path: &Path,
    expected_project_id: &str,
    candidate_version: &str,
    candidate_payloads: &[ManagedAssetPayload],
) -> Result<UpdateTransaction> {
    let repo_root = canonical_repo_root(repo_root)?;
    let paths = transaction_paths_for_state(&repo_root, state_path)?;
    let mut transaction = load_transaction(&repo_root, &paths)?;
    validate_transaction_identity(&repo_root, &transaction, expected_project_id)?;
    if transaction.status != TransactionStatus::Verified {
        bail!(
            "Baron candidate may only plan a verified transaction; current status is `{}`",
            transaction.status.as_str()
        );
    }
    if transaction.target_version != candidate_version {
        bail!("Baron candidate version does not match the frozen transaction target version");
    }
    verify_candidate_identity(&repo_root, &transaction)?;
    validate_payload_set(candidate_payloads, &transaction.adapters)?;

    let plan = plan_managed_update(&repo_root, candidate_payloads)?;
    if plan.actions.len() > MAX_TRANSACTION_PACKETS {
        bail!("Baron update transaction exceeds the bounded managed packet limit");
    }
    let baseline = load_managed_baseline(&repo_root)?;
    let baseline_by_key = baseline
        .records
        .iter()
        .map(|record| {
            (
                (record.adapter.clone(), record.relative_path.clone()),
                record,
            )
        })
        .collect::<HashMap<_, _>>();
    let payload_by_key = candidate_payloads
        .iter()
        .map(|payload| {
            (
                (payload.adapter.clone(), payload.relative_path.clone()),
                payload,
            )
        })
        .collect::<HashMap<_, _>>();

    let mut packet_targets = BTreeSet::new();
    let mut packets = Vec::new();
    for action in &plan.actions {
        validate_adapter(&action.adapter)?;
        validate_relative_path(&action.relative_path)?;
        if !packet_targets.insert(action.relative_path.clone()) {
            bail!(
                "Baron update transaction refuses duplicate live managed target ownership: {}",
                action.relative_path.display()
            );
        }
        let key = (action.adapter.clone(), action.relative_path.clone());
        let base = baseline_by_key
            .get(&key)
            .map(|record| managed_baseline_content(&repo_root, record))
            .transpose()?
            .unwrap_or_default();
        let target = checked_repo_target(&repo_root, &action.relative_path, false)?;
        let target_existed = target.exists();
        let local = if target_existed {
            read_bounded_text(&target, "managed target")?
        } else {
            String::new()
        };
        let upstream = payload_by_key
            .get(&key)
            .map(|payload| payload.content.clone())
            .unwrap_or_default();
        let has_upstream = payload_by_key.contains_key(&key);
        let resolved = match action.disposition {
            UpdateDisposition::Conflict => String::new(),
            _ => action
                .resolved_content
                .clone()
                .unwrap_or_else(|| local.clone()),
        };
        let resolved_sha256 =
            (action.disposition != UpdateDisposition::Conflict).then(|| sha256_text(&resolved));
        let packet = TransactionPacket {
            adapter: action.adapter.clone(),
            relative_path: action.relative_path.clone(),
            merge_kind: action.merge_kind,
            disposition: action.disposition,
            target_existed,
            has_upstream,
            base_sha256: sha256_text(&base),
            local_sha256: sha256_text(&local),
            upstream_sha256: sha256_text(&upstream),
            resolved_sha256,
        };
        write_packet(&paths, "BASE", &packet, &base)?;
        write_packet(&paths, "LOCAL", &packet, &local)?;
        write_packet(&paths, "UPSTREAM", &packet, &upstream)?;
        write_packet(&paths, "RESOLVED", &packet, &resolved)?;
        write_packet(&paths, "backups/project", &packet, &local)?;
        packets.push(packet);
    }
    transaction.packets = packets;
    transition(
        &mut transaction,
        if plan.conflicts.is_empty() {
            TransactionStatus::Planned
        } else {
            TransactionStatus::Conflict
        },
    )?;
    transaction.last_checkpoint = if plan.conflicts.is_empty() {
        "candidate_plan_staged".to_string()
    } else {
        "conflict_packets_staged".to_string()
    };
    write_transaction(&paths, &transaction)?;
    Ok(transaction)
}

pub fn continue_transaction(
    repo_root: &Path,
    state_path: &Path,
    expected_project_id: &str,
) -> Result<UpdateTransaction> {
    apply_transaction_with_failure(repo_root, state_path, expected_project_id, None)
}

pub fn abort_transaction(
    repo_root: &Path,
    state_path: &Path,
    expected_project_id: &str,
) -> Result<()> {
    let repo_root = canonical_repo_root(repo_root)?;
    let paths = transaction_paths_for_state(&repo_root, state_path)?;
    let transaction = load_transaction(&repo_root, &paths)?;
    validate_transaction_identity(&repo_root, &transaction, expected_project_id)?;
    if !matches!(
        transaction.status,
        TransactionStatus::Verified | TransactionStatus::Planned | TransactionStatus::Conflict
    ) {
        bail!(
            "Baron can abort only an unapplied update transaction; current status is `{}`",
            transaction.status.as_str()
        );
    }
    remove_transaction_directory(&repo_root, &paths.root)
}

pub fn recover_transaction(
    repo_root: &Path,
    state_path: &Path,
    expected_project_id: &str,
) -> Result<UpdateTransaction> {
    let repo_root = canonical_repo_root(repo_root)?;
    let paths = transaction_paths_for_state(&repo_root, state_path)?;
    let transaction = load_transaction(&repo_root, &paths)?;
    validate_transaction_identity(&repo_root, &transaction, expected_project_id)?;
    match transaction.status {
        TransactionStatus::ProjectActivated | TransactionStatus::RuntimePending => {
            rollback_transaction(&repo_root, &paths, transaction)
        }
        _ => Ok(transaction),
    }
}

pub fn mark_runtime_pending(
    repo_root: &Path,
    state_path: &Path,
    expected_project_id: &str,
    installed_binary: &Path,
) -> Result<UpdateTransaction> {
    let repo_root = canonical_repo_root(repo_root)?;
    let paths = transaction_paths_for_state(&repo_root, state_path)?;
    let mut transaction = load_transaction(&repo_root, &paths)?;
    validate_transaction_identity(&repo_root, &transaction, expected_project_id)?;
    if transaction.status != TransactionStatus::ProjectActivated {
        bail!(
            "Baron may mark runtime pending only after project activation; current status is `{}`",
            transaction.status.as_str()
        );
    }
    let installed_binary = installed_binary.canonicalize().with_context(|| {
        format!(
            "Could not resolve the active Baron runtime for transaction handoff: {}",
            installed_binary.display()
        )
    })?;
    if !fs::metadata(&installed_binary)?.is_file() || is_link_or_reparse_point(&installed_binary)? {
        bail!("Active Baron runtime is not a safe regular file for transaction handoff");
    }
    transaction.runtime_binary_path = Some(installed_binary);
    transition(&mut transaction, TransactionStatus::RuntimePending)?;
    transaction.last_checkpoint = "runtime_handoff_prepared".to_string();
    write_transaction(&paths, &transaction)?;
    Ok(transaction)
}

pub fn complete_transaction(
    repo_root: &Path,
    state_path: &Path,
    expected_project_id: &str,
    runtime_proof: &str,
) -> Result<UpdateTransaction> {
    let repo_root = canonical_repo_root(repo_root)?;
    let paths = transaction_paths_for_state(&repo_root, state_path)?;
    let mut transaction = load_transaction(&repo_root, &paths)?;
    validate_transaction_identity(&repo_root, &transaction, expected_project_id)?;
    if transaction.status != TransactionStatus::RuntimePending {
        bail!(
            "Baron may complete an update only after runtime handoff is pending; current status is `{}`",
            transaction.status.as_str()
        );
    }
    if runtime_proof.trim().is_empty() {
        bail!("Baron update completion requires runtime verification evidence");
    }
    write_receipt(&repo_root, &paths, &transaction, runtime_proof)?;
    transition(&mut transaction, TransactionStatus::Completed)?;
    transaction.last_checkpoint = "receipt_written".to_string();
    write_transaction(&paths, &transaction)?;
    Ok(transaction)
}

pub fn inspect_transaction(
    repo_root: &Path,
    state_path: &Path,
    expected_project_id: &str,
) -> Result<UpdateTransaction> {
    let repo_root = canonical_repo_root(repo_root)?;
    let paths = transaction_paths_for_state(&repo_root, state_path)?;
    let transaction = load_transaction(&repo_root, &paths)?;
    validate_transaction_identity(&repo_root, &transaction, expected_project_id)?;
    Ok(transaction)
}

pub fn candidate_for_transaction(
    repo_root: &Path,
    state_path: &Path,
    expected_project_id: &str,
) -> Result<UpdateCandidate> {
    let repo_root = canonical_repo_root(repo_root)?;
    let transaction = inspect_transaction(&repo_root, state_path, expected_project_id)?;
    verify_candidate_identity(&repo_root, &transaction)?;
    let update_root = safe_update_root(&repo_root)?;
    let staged_path = checked_existing_path(
        &update_root,
        &update_root.join(&transaction.candidate_relative_path),
    )?;
    Ok(UpdateCandidate {
        version: transaction.target_version,
        source_revision: transaction.source_revision,
        target: transaction.target,
        executable_name: staged_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("baron")
            .to_string(),
        sha256: transaction.candidate_sha256,
        size_bytes: transaction.candidate_size_bytes,
        staged_path,
    })
}

#[cfg(target_os = "windows")]
pub fn runtime_binary_for_transaction(
    repo_root: &Path,
    state_path: &Path,
    expected_project_id: &str,
) -> Result<PathBuf> {
    let repo_root = canonical_repo_root(repo_root)?;
    let transaction = inspect_transaction(&repo_root, state_path, expected_project_id)?;
    let path = transaction
        .runtime_binary_path
        .context("Baron update transaction has no frozen runtime handoff path")?;
    if !path.is_absolute() || !path.is_file() || is_link_or_reparse_point(&path)? {
        bail!("Baron update transaction runtime handoff path is unavailable or unsafe");
    }
    Ok(path)
}

pub fn recover_incomplete_transactions(
    repo_root: &Path,
    expected_project_id: &str,
) -> Result<Vec<String>> {
    let repo_root = canonical_repo_root(repo_root)?;
    let update_root = safe_update_root(&repo_root)?;
    let transactions_root = update_root.join("transactions");
    if !transactions_root.exists() {
        return Ok(Vec::new());
    }
    if is_link_or_reparse_point(&transactions_root)? || !fs::metadata(&transactions_root)?.is_dir()
    {
        bail!("Baron update transaction root is unsafe");
    }
    let mut recovered = Vec::new();
    let mut entries =
        fs::read_dir(&transactions_root)?.collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    if entries.len() > MAX_TRANSACTION_DIRECTORIES {
        bail!("Baron update transaction workspace exceeds the bounded directory limit");
    }
    for entry in entries {
        let root = entry.path();
        if is_link_or_reparse_point(&root)? || !fs::metadata(&root)?.is_dir() {
            bail!(
                "Baron update transaction entry is unsafe: {}",
                root.display()
            );
        }
        let state_path = root.join(STATE_FILE);
        if !state_path.is_file() {
            continue;
        }
        let transaction = inspect_transaction(&repo_root, &state_path, expected_project_id)?;
        if matches!(
            transaction.status,
            TransactionStatus::ProjectActivated | TransactionStatus::RuntimePending
        ) {
            let recovered_transaction =
                recover_transaction(&repo_root, &state_path, expected_project_id)?;
            recovered.push(recovered_transaction.transaction_id);
        }
    }
    Ok(recovered)
}

pub fn pending_transaction(
    repo_root: &Path,
    expected_project_id: &str,
) -> Result<Option<PendingTransaction>> {
    let repo_root = canonical_repo_root(repo_root)?;
    let update_root = safe_update_root(&repo_root)?;
    let transactions_root = update_root.join("transactions");
    if !transactions_root.exists() {
        return Ok(None);
    }
    if is_link_or_reparse_point(&transactions_root)? || !fs::metadata(&transactions_root)?.is_dir()
    {
        bail!("Baron update transaction root is unsafe");
    }
    let mut entries =
        fs::read_dir(&transactions_root)?.collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    if entries.len() > MAX_TRANSACTION_DIRECTORIES {
        bail!("Baron update transaction workspace exceeds the bounded directory limit");
    }
    let mut pending = Vec::new();
    for entry in entries {
        let root = entry.path();
        if is_link_or_reparse_point(&root)? || !fs::metadata(&root)?.is_dir() {
            bail!(
                "Baron update transaction entry is unsafe: {}",
                root.display()
            );
        }
        let state_path = root.join(STATE_FILE);
        if !state_path.is_file() {
            continue;
        }
        let transaction = inspect_transaction(&repo_root, &state_path, expected_project_id)?;
        if matches!(
            transaction.status,
            TransactionStatus::Verified | TransactionStatus::Planned | TransactionStatus::Conflict
        ) {
            pending.push(PendingTransaction {
                state_path,
                transaction,
            });
        }
    }
    if pending.len() > 1 {
        bail!(
            "Baron found multiple pending update transactions; refuse ambiguous activation: {}",
            pending
                .iter()
                .map(|pending| pending.transaction.transaction_id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(pending.pop())
}

fn apply_transaction_with_failure(
    repo_root: &Path,
    state_path: &Path,
    expected_project_id: &str,
    failure: Option<ApplyFailurePoint>,
) -> Result<UpdateTransaction> {
    let repo_root = canonical_repo_root(repo_root)?;
    let paths = transaction_paths_for_state(&repo_root, state_path)?;
    let mut transaction = load_transaction(&repo_root, &paths)?;
    validate_transaction_identity(&repo_root, &transaction, expected_project_id)?;
    verify_frozen_transaction(&repo_root, &paths, &mut transaction)?;
    if transaction.status == TransactionStatus::Conflict {
        freeze_conflict_resolutions(&paths, &mut transaction)?;
        transition(&mut transaction, TransactionStatus::Planned)?;
        transaction.last_checkpoint = "conflict_resolutions_frozen".to_string();
        write_transaction(&paths, &transaction)?;
    }
    if transaction.status != TransactionStatus::Planned {
        bail!(
            "Baron may apply only a planned transaction; current status is `{}`",
            transaction.status.as_str()
        );
    }
    if let Err(error) = apply_project_changes(&repo_root, &paths, &mut transaction, failure) {
        let rollback_result = rollback_transaction(&repo_root, &paths, transaction);
        return match rollback_result {
            Ok(_) => Err(error.context("Baron update project activation failed and was rolled back")),
            Err(rollback_error) => Err(error.context(format!(
                "Baron update project activation failed; automatic rollback also failed: {rollback_error:#}"
            ))),
        };
    }
    Ok(transaction)
}

fn apply_project_changes(
    repo_root: &Path,
    paths: &TransactionPaths,
    transaction: &mut UpdateTransaction,
    failure: Option<ApplyFailurePoint>,
) -> Result<()> {
    snapshot_managed_state(repo_root, paths)?;
    maybe_fail(failure, ApplyFailurePoint::AfterManagedStateBackup)?;
    for (index, packet) in transaction.packets.iter().enumerate() {
        let resolved = read_packet(paths, "RESOLVED", packet)?;
        let local = read_packet(paths, "LOCAL", packet)?;
        if resolved == local {
            continue;
        }
        maybe_fail(failure, ApplyFailurePoint::BeforeManagedWrite(index))?;
        let target = checked_repo_target(repo_root, &packet.relative_path, true)?;
        atomic_write_text(&target, &resolved)?;
        transaction.last_checkpoint = format!("managed_write:{index}");
        write_transaction(paths, transaction)?;
        maybe_fail(failure, ApplyFailurePoint::AfterManagedWrite(index))?;
    }
    maybe_fail(failure, ApplyFailurePoint::BeforeBaselineReplacement)?;
    let payloads = transaction
        .packets
        .iter()
        .filter(|packet| packet.has_upstream)
        .map(|packet| {
            Ok(ManagedAssetPayload {
                adapter: packet.adapter.clone(),
                relative_path: packet.relative_path.clone(),
                merge_kind: packet.merge_kind,
                content: read_packet(paths, "UPSTREAM", packet)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    replace_managed_baseline(repo_root, &payloads, &transaction.target_version)?;
    transaction.last_checkpoint = "baseline_replaced".to_string();
    write_transaction(paths, transaction)?;
    maybe_fail(failure, ApplyFailurePoint::AfterBaselineReplacement)?;
    transition(transaction, TransactionStatus::ProjectActivated)?;
    transaction.last_checkpoint = "project_activated".to_string();
    write_transaction(paths, transaction)
}

fn rollback_transaction(
    repo_root: &Path,
    paths: &TransactionPaths,
    mut transaction: UpdateTransaction,
) -> Result<UpdateTransaction> {
    validate_transaction(&transaction)?;
    for packet in &transaction.packets {
        let target = checked_repo_target(repo_root, &packet.relative_path, true)?;
        if packet.target_existed {
            let backup = read_packet(paths, "backups/project", packet)?;
            atomic_write_text(&target, &backup)?;
        } else if target.exists() {
            fs::remove_file(&target).with_context(|| {
                format!(
                    "Could not remove rolled-back managed target: {}",
                    target.display()
                )
            })?;
        }
    }
    restore_managed_state(repo_root, paths)?;
    if transaction.status != TransactionStatus::RolledBack {
        if !matches!(
            transaction.status,
            TransactionStatus::Planned
                | TransactionStatus::ProjectActivated
                | TransactionStatus::RuntimePending
        ) {
            bail!(
                "Baron cannot roll back transaction status `{}`",
                transaction.status.as_str()
            );
        }
        transition(&mut transaction, TransactionStatus::RolledBack)?;
    }
    transaction.last_checkpoint = "rollback_restored".to_string();
    write_transaction(paths, &transaction)?;
    Ok(transaction)
}

fn freeze_conflict_resolutions(
    paths: &TransactionPaths,
    transaction: &mut UpdateTransaction,
) -> Result<()> {
    for packet in &mut transaction.packets {
        if packet.disposition != UpdateDisposition::Conflict {
            continue;
        }
        let resolved = read_packet(paths, "RESOLVED", packet)?;
        if resolved.trim().is_empty() {
            bail!(
                "Conflict resolution is missing for managed target `{}`",
                packet.relative_path.display()
            );
        }
        packet.resolved_sha256 = Some(sha256_text(&resolved));
    }
    Ok(())
}

fn verify_frozen_transaction(
    repo_root: &Path,
    paths: &TransactionPaths,
    transaction: &mut UpdateTransaction,
) -> Result<()> {
    validate_transaction(transaction)?;
    verify_candidate_identity(repo_root, transaction)?;
    for packet in &transaction.packets {
        verify_packet_hash(paths, "BASE", packet, &packet.base_sha256)?;
        verify_packet_hash(paths, "LOCAL", packet, &packet.local_sha256)?;
        verify_packet_hash(paths, "UPSTREAM", packet, &packet.upstream_sha256)?;
        let resolved = read_packet(paths, "RESOLVED", packet)?;
        if let Some(expected) = &packet.resolved_sha256 {
            if sha256_text(&resolved) != *expected {
                bail!(
                    "Frozen resolved packet changed for `{}`; create a new transaction instead of continuing stale state",
                    packet.relative_path.display()
                );
            }
        }
        let target = checked_repo_target(repo_root, &packet.relative_path, false)?;
        let current = if target.exists() {
            read_bounded_text(&target, "managed target")?
        } else {
            String::new()
        };
        if sha256_text(&current) != packet.local_sha256 {
            bail!(
                "Managed target changed after transaction planning for `{}`; refuse stale continuation",
                packet.relative_path.display()
            );
        }
    }
    Ok(())
}

fn verify_candidate_identity(repo_root: &Path, transaction: &UpdateTransaction) -> Result<()> {
    let update_root = safe_update_root(repo_root)?;
    let candidate = checked_existing_path(
        &update_root,
        &update_root.join(&transaction.candidate_relative_path),
    )?;
    let metadata = fs::metadata(&candidate)?;
    if !metadata.is_file()
        || metadata.len() != transaction.candidate_size_bytes
        || sha256_file(&candidate)? != transaction.candidate_sha256
    {
        bail!("Verified candidate changed after transaction staging; refuse continuation");
    }
    Ok(())
}

fn validate_transaction_identity(
    repo_root: &Path,
    transaction: &UpdateTransaction,
    expected_project_id: &str,
) -> Result<()> {
    validate_transaction(transaction)?;
    let config = load_project_config(repo_root)?;
    if config.project_id != transaction.project_id || expected_project_id != transaction.project_id
    {
        bail!("Baron update transaction project identity changed; refuse continuation");
    }
    if path_fingerprint(repo_root)? != transaction.repo_root_fingerprint {
        bail!("Baron update transaction repository location changed; refuse continuation");
    }
    Ok(())
}

fn validate_transaction(transaction: &UpdateTransaction) -> Result<()> {
    if transaction.schema_version != TRANSACTION_SCHEMA_VERSION {
        bail!(
            "Unsupported Baron update transaction schema {}; expected {}",
            transaction.schema_version,
            TRANSACTION_SCHEMA_VERSION
        );
    }
    if transaction.transaction_id.is_empty()
        || transaction.project_id.is_empty()
        || transaction.project_slug.is_empty()
        || transaction.source_version.is_empty()
        || transaction.target_version.is_empty()
        || transaction.source_revision.len() != 40
        || !transaction
            .source_revision
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || transaction.candidate_sha256.len() != 64
        || !transaction
            .candidate_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("Baron update transaction identity metadata is invalid");
    }
    validate_relative_path(&transaction.candidate_relative_path)?;
    if matches!(
        transaction.status,
        TransactionStatus::RuntimePending | TransactionStatus::Completed
    ) && transaction.runtime_binary_path.is_none()
    {
        bail!("Baron update transaction runtime handoff path is missing");
    }
    if transaction.packets.len() > MAX_TRANSACTION_PACKETS {
        bail!("Baron update transaction exceeds the bounded managed packet limit");
    }
    let mut targets = BTreeSet::new();
    for packet in &transaction.packets {
        validate_adapter(&packet.adapter)?;
        validate_relative_path(&packet.relative_path)?;
        if !targets.insert(packet.relative_path.clone()) {
            bail!(
                "Baron update transaction contains duplicate live managed target ownership: {}",
                packet.relative_path.display()
            );
        }
        for hash in [
            &packet.base_sha256,
            &packet.local_sha256,
            &packet.upstream_sha256,
        ] {
            validate_sha256(hash)?;
        }
        if let Some(hash) = &packet.resolved_sha256 {
            validate_sha256(hash)?;
        }
    }
    Ok(())
}

fn validate_config(config: &ProjectConfig) -> Result<()> {
    if config.project_id.trim().is_empty() || config.project_slug.trim().is_empty() {
        bail!("Baron project configuration has no stable project identity");
    }
    Ok(())
}

fn validate_payload_set(payloads: &[ManagedAssetPayload], adapters: &[String]) -> Result<()> {
    if payloads.is_empty() {
        bail!("Baron candidate did not render any managed assets");
    }
    let allowed = adapters.iter().collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    for payload in payloads {
        validate_adapter(&payload.adapter)?;
        validate_relative_path(&payload.relative_path)?;
        if !allowed.contains(&payload.adapter) {
            bail!("Baron candidate rendered an unregistered adapter payload");
        }
        if !seen.insert((payload.adapter.clone(), payload.relative_path.clone())) {
            bail!("Baron candidate rendered duplicate managed payload ownership");
        }
    }
    Ok(())
}

fn normalize_adapters(adapters: &[String]) -> Result<Vec<String>> {
    if adapters.is_empty() {
        bail!("Baron update transaction requires at least one registered adapter");
    }
    let mut normalized = BTreeSet::new();
    for adapter in adapters {
        validate_adapter(adapter)?;
        normalized.insert(adapter.clone());
    }
    Ok(normalized.into_iter().collect())
}

fn transition(transaction: &mut UpdateTransaction, next: TransactionStatus) -> Result<()> {
    if !transaction.status.allows_transition_to(next) {
        bail!(
            "Invalid Baron update transaction transition: `{}` -> `{}`",
            transaction.status.as_str(),
            next.as_str()
        );
    }
    transaction.status = next;
    Ok(())
}

fn next_transaction_id() -> Result<String> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("System clock is before the Unix epoch")?
        .as_millis();
    let sequence = TRANSACTION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    Ok(format!("txn-{}-{millis}-{sequence}", std::process::id()))
}

fn transaction_paths_for_state(repo_root: &Path, state_path: &Path) -> Result<TransactionPaths> {
    let update_root = safe_update_root(repo_root)?;
    let state_path = checked_existing_path(&update_root, state_path)?;
    if state_path.file_name().and_then(|name| name.to_str()) != Some(STATE_FILE) {
        bail!("Baron update transaction state file must be named {STATE_FILE}");
    }
    let root = state_path
        .parent()
        .context("Baron update transaction state has no parent directory")?
        .to_path_buf();
    let transactions_root = update_root.join("transactions");
    let transactions_root = checked_existing_path(&update_root, &transactions_root)?;
    if !root.starts_with(&transactions_root) || root == transactions_root {
        bail!("Baron update transaction state escaped the transaction workspace");
    }
    Ok(TransactionPaths { root, state_path })
}

fn safe_update_root(repo_root: &Path) -> Result<PathBuf> {
    let repo_root = canonical_repo_root(repo_root)?;
    let baron = safe_child_directory(&repo_root, ".baron")?;
    safe_child_directory(&baron, "update")
}

fn safe_child_directory(parent: &Path, name: &str) -> Result<PathBuf> {
    if name.is_empty() || Path::new(name).components().count() != 1 {
        bail!("Baron update workspace component is invalid");
    }
    let path = parent.join(name);
    if path.exists() {
        if is_link_or_reparse_point(&path)? || !fs::metadata(&path)?.is_dir() {
            bail!(
                "Baron update workspace cannot traverse a symlink, junction, or file: {}",
                path.display()
            );
        }
        return Ok(path);
    }
    fs::create_dir(&path).or_else(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            Ok(())
        } else {
            Err(error)
        }
    })?;
    if is_link_or_reparse_point(&path)? || !fs::metadata(&path)?.is_dir() {
        bail!(
            "Baron update workspace became unsafe while creating: {}",
            path.display()
        );
    }
    Ok(path)
}

fn checked_existing_path(root: &Path, path: &Path) -> Result<PathBuf> {
    let root = root
        .canonicalize()
        .with_context(|| format!("Could not resolve update workspace: {}", root.display()))?;
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Could not resolve staged update path: {}", path.display()))?;
    if !canonical.starts_with(&root) {
        bail!("Baron update path escaped the update workspace");
    }
    let relative = canonical.strip_prefix(&root).expect("checked prefix");
    validate_relative_path(relative)?;
    let mut current = root;
    for component in relative.components() {
        let Component::Normal(part) = component else {
            unreachable!("validated update path")
        };
        current.push(part);
        if is_link_or_reparse_point(&current)? {
            bail!("Baron update path cannot traverse a symlink or junction");
        }
    }
    Ok(canonical)
}

fn checked_repo_target(
    repo_root: &Path,
    relative_path: &Path,
    create_parent: bool,
) -> Result<PathBuf> {
    validate_relative_path(relative_path)?;
    let repo_root = canonical_repo_root(repo_root)?;
    let mut current = repo_root.clone();
    let components = relative_path.components().collect::<Vec<_>>();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(part) = component else {
            unreachable!("validated managed target")
        };
        current.push(part);
        let is_final = index + 1 == components.len();
        if current.exists() {
            if is_link_or_reparse_point(&current)? {
                bail!(
                    "Managed target cannot traverse a symlink or junction: {}",
                    relative_path.display()
                );
            }
            if !is_final && !fs::metadata(&current)?.is_dir() {
                bail!(
                    "Managed target parent is not a directory: {}",
                    current.display()
                );
            }
            continue;
        }
        if !create_parent || is_final {
            break;
        }
        fs::create_dir(&current).or_else(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(error)
            }
        })?;
        if is_link_or_reparse_point(&current)? || !fs::metadata(&current)?.is_dir() {
            bail!(
                "Managed target parent became unsafe while creating: {}",
                current.display()
            );
        }
    }
    Ok(repo_root.join(relative_path))
}

fn write_packet(
    paths: &TransactionPaths,
    section: &str,
    packet: &TransactionPacket,
    content: &str,
) -> Result<()> {
    let path = packet_path(paths, section, packet, true)?;
    atomic_write_text(&path, content)
}

fn read_packet(
    paths: &TransactionPaths,
    section: &str,
    packet: &TransactionPacket,
) -> Result<String> {
    let path = packet_path(paths, section, packet, false)?;
    read_bounded_text(&path, "transaction packet")
}

fn packet_path(
    paths: &TransactionPaths,
    section: &str,
    packet: &TransactionPacket,
    create_parent: bool,
) -> Result<PathBuf> {
    validate_adapter(&packet.adapter)?;
    validate_relative_path(&packet.relative_path)?;
    let mut path = paths.root.clone();
    for component in section
        .split('/')
        .chain(std::iter::once(packet.adapter.as_str()))
    {
        path = if create_parent {
            safe_child_directory(&path, component)?
        } else {
            checked_child_directory(&path, component)?
        };
    }
    let mut parent = path;
    let components = packet.relative_path.components().collect::<Vec<_>>();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(part) = component else {
            unreachable!("validated packet path")
        };
        let next = parent.join(part);
        let is_final = index + 1 == components.len();
        if is_final {
            return Ok(next);
        }
        if next.exists() {
            if is_link_or_reparse_point(&next)? || !fs::metadata(&next)?.is_dir() {
                bail!("Baron update packet path is unsafe: {}", next.display());
            }
        } else if create_parent {
            parent = safe_child_directory(&parent, &part.to_string_lossy())?;
            continue;
        } else {
            bail!("Baron update packet is missing: {}", next.display());
        }
        parent = next;
    }
    unreachable!("managed relative path has at least one component")
}

fn checked_child_directory(parent: &Path, name: &str) -> Result<PathBuf> {
    if name.is_empty() || Path::new(name).components().count() != 1 {
        bail!("Baron update workspace component is invalid");
    }
    let path = parent.join(name);
    if !path.exists() {
        bail!(
            "Baron update packet directory is missing: {}",
            path.display()
        );
    }
    if is_link_or_reparse_point(&path)? || !fs::metadata(&path)?.is_dir() {
        bail!(
            "Baron update packet directory is unsafe: {}",
            path.display()
        );
    }
    Ok(path)
}

fn snapshot_managed_state(repo_root: &Path, paths: &TransactionPaths) -> Result<()> {
    let source = repo_root.join(".baron/managed-state");
    if !source.exists() {
        bail!("Managed baseline state is missing before transaction activation");
    }
    validate_safe_tree(&source)?;
    let destination = paths.root.join("backups/managed-state");
    if destination.exists() {
        validate_safe_tree(&destination)?;
        return Ok(());
    }
    copy_tree(&source, &destination)
}

fn restore_managed_state(repo_root: &Path, paths: &TransactionPaths) -> Result<()> {
    let backup = paths.root.join("backups/managed-state");
    if !backup.exists() {
        bail!("Managed baseline backup is missing; Baron refuses an incomplete rollback");
    }
    validate_safe_tree(&backup)?;
    let destination = repo_root.join(".baron/managed-state");
    if destination.exists() {
        validate_safe_tree(&destination)?;
        fs::remove_dir_all(&destination).with_context(|| {
            format!(
                "Could not remove changed managed baseline: {}",
                destination.display()
            )
        })?;
    }
    copy_tree(&backup, &destination)
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    validate_safe_tree(source)?;
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)?;
        if metadata.file_type().is_symlink() || is_link_or_reparse_point(&source_path)? {
            bail!(
                "Baron update backup cannot copy a symlink or junction: {}",
                source_path.display()
            );
        }
        if metadata.is_dir() {
            copy_tree(&source_path, &destination_path)?;
        } else if metadata.is_file() {
            fs::copy(&source_path, &destination_path)?;
        } else {
            bail!("Baron update backup encountered an unsupported filesystem entry");
        }
    }
    Ok(())
}

fn validate_safe_tree(root: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(root)?;
    if metadata.file_type().is_symlink() || is_link_or_reparse_point(root)? {
        bail!(
            "Baron update state cannot traverse a symlink or junction: {}",
            root.display()
        );
    }
    if !metadata.is_dir() {
        bail!("Baron update state must be a directory: {}", root.display());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() || is_link_or_reparse_point(&path)? {
            bail!(
                "Baron update state cannot traverse a symlink or junction: {}",
                path.display()
            );
        }
        if metadata.is_dir() {
            validate_safe_tree(&path)?;
        }
    }
    Ok(())
}

fn write_receipt(
    repo_root: &Path,
    _paths: &TransactionPaths,
    transaction: &UpdateTransaction,
    runtime_proof: &str,
) -> Result<()> {
    let update_root = safe_update_root(repo_root)?;
    let receipts = safe_child_directory(&update_root, "receipts")?;
    let receipt = serde_json::json!({
        "schema_version": TRANSACTION_SCHEMA_VERSION,
        "transaction_id": transaction.transaction_id,
        "project_id": transaction.project_id,
        "target_version": transaction.target_version,
        "candidate_sha256": transaction.candidate_sha256,
        "runtime_proof": runtime_proof,
        "status": "completed"
    });
    atomic_write_text(
        &receipts.join(format!("{}.json", transaction.transaction_id)),
        &format!("{}\n", serde_json::to_string_pretty(&receipt)?),
    )
}

fn write_transaction(paths: &TransactionPaths, transaction: &UpdateTransaction) -> Result<()> {
    validate_transaction(transaction)?;
    let content = format!("{}\n", serde_json::to_string_pretty(transaction)?);
    atomic_write_text(&paths.state_path, &content)?;
    atomic_write_text(
        &paths.root.join(STATE_SEAL_FILE),
        &format!("{}\n", sha256_text(&content)),
    )
}

fn load_transaction(repo_root: &Path, paths: &TransactionPaths) -> Result<UpdateTransaction> {
    let state = read_bounded_text(&paths.state_path, "transaction state")?;
    let seal = read_bounded_text(&paths.root.join(STATE_SEAL_FILE), "transaction state seal")?;
    if seal.trim() != sha256_text(&state) {
        bail!("Baron update transaction metadata changed after it was sealed; refuse continuation");
    }
    let transaction: UpdateTransaction =
        serde_json::from_str(&state).context("Baron update transaction state is malformed")?;
    validate_transaction_identity(repo_root, &transaction, &transaction.project_id)?;
    Ok(transaction)
}

fn remove_transaction_directory(repo_root: &Path, root: &Path) -> Result<()> {
    let update_root = safe_update_root(repo_root)?;
    let root = checked_existing_path(&update_root, root)?;
    let transactions = checked_existing_path(&update_root, &update_root.join("transactions"))?;
    if !root.starts_with(&transactions) || root == transactions {
        bail!("Baron refuses to remove an update path outside its transaction workspace");
    }
    validate_safe_tree(&root)?;
    fs::remove_dir_all(root).context("Could not remove staged Baron update transaction")
}

fn maybe_fail(point: Option<ApplyFailurePoint>, expected: ApplyFailurePoint) -> Result<()> {
    if point == Some(expected) {
        bail!("Injected Baron update transaction failure at {expected:?}");
    }
    Ok(())
}

fn verify_packet_hash(
    paths: &TransactionPaths,
    section: &str,
    packet: &TransactionPacket,
    expected: &str,
) -> Result<()> {
    let content = read_packet(paths, section, packet)?;
    if sha256_text(&content) != expected {
        bail!(
            "Frozen {section} packet changed for `{}`; refuse stale continuation",
            packet.relative_path.display()
        );
    }
    Ok(())
}

fn atomic_write_text(path: &Path, content: &str) -> Result<()> {
    let parent = path
        .parent()
        .context("Baron update file has no parent directory")?;
    if !parent.exists() {
        fs::create_dir_all(parent)?;
    }
    let temporary = parent.join(format!(
        ".{}.baron-tmp-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("state"),
        TRANSACTION_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = fs::File::create(&temporary)
        .with_context(|| format!("Could not write Baron update file: {}", temporary.display()))?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;
    drop(file);
    if let Err(rename_error) = fs::rename(&temporary, path) {
        if path.exists() {
            fs::remove_file(path).with_context(|| {
                format!("Could not replace Baron update file: {}", path.display())
            })?;
            fs::rename(&temporary, path)?;
        } else {
            return Err(rename_error).with_context(|| {
                format!("Could not activate Baron update file: {}", path.display())
            });
        }
    }
    Ok(())
}

fn read_bounded_text(path: &Path, label: &str) -> Result<String> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Could not inspect Baron {label}: {}", path.display()))?;
    if !metadata.is_file() || metadata.len() > MAX_TRANSACTION_FILE_BYTES {
        bail!("Baron {label} is missing, not a file, or exceeds the bounded size limit");
    }
    fs::read_to_string(path)
        .with_context(|| format!("Could not read Baron {label}: {}", path.display()))
}

fn canonical_repo_root(repo_root: &Path) -> Result<PathBuf> {
    let root = repo_root.canonicalize().with_context(|| {
        format!(
            "Could not resolve Baron project root: {}",
            repo_root.display()
        )
    })?;
    if !fs::metadata(&root)?.is_dir() {
        bail!("Baron project root is not a directory");
    }
    Ok(root)
}

fn path_fingerprint(path: &Path) -> Result<String> {
    Ok(sha256_text(&path.canonicalize()?.to_string_lossy()))
}

fn sha256_text(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn validate_sha256(value: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("Baron update transaction hash is invalid");
    }
    Ok(())
}

fn validate_adapter(adapter: &str) -> Result<()> {
    if adapter.is_empty()
        || !adapter
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        bail!("Baron update transaction adapter is invalid");
    }
    Ok(())
}

fn validate_relative_path(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        bail!("Baron update transaction path must be repository-relative");
    }
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!(
            "Baron update transaction path escapes its workspace: {}",
            path.display()
        );
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use baron_adapters::record_managed_baseline;
    use baron_core::config::{AdapterKind, AutomationConfig};
    use tempfile::tempdir;

    const SOURCE_REVISION: &str = "0123456789abcdef0123456789abcdef01234567";

    fn config() -> ProjectConfig {
        ProjectConfig {
            schema_version: 4,
            project_id: "project-identity".to_string(),
            project_slug: "project".to_string(),
            platform: None,
            platform_extensions: Vec::new(),
            adapters: vec![AdapterKind::Codex],
            automation: AutomationConfig::default(),
        }
    }

    fn initialize_repo(repo: &Path) {
        fs::create_dir_all(repo.join(".baron")).unwrap();
        fs::write(
            repo.join(".baron/project.toml"),
            "schema_version = 4\nproject_id = \"project-identity\"\nproject_slug = \"project\"\nadapters = [\"codex\"]\n\n[automation]\ncontext = true\nplan = true\nharness = true\nproof = true\ntrace = true\n",
        )
        .unwrap();
    }

    fn payload(content: &str) -> ManagedAssetPayload {
        ManagedAssetPayload {
            adapter: "codex".to_string(),
            relative_path: PathBuf::from("AGENTS.md"),
            merge_kind: ManagedMergeKind::FullText,
            content: content.to_string(),
        }
    }

    fn staged_candidate(repo: &Path) -> UpdateCandidate {
        let candidate_dir = repo.join(".baron/update/candidate");
        fs::create_dir_all(&candidate_dir).unwrap();
        let staged_path = candidate_dir.join("baron.exe");
        fs::write(&staged_path, "candidate").unwrap();
        UpdateCandidate {
            version: "3.4.0".to_string(),
            source_revision: SOURCE_REVISION.to_string(),
            target: "x86_64-pc-windows-msvc".to_string(),
            executable_name: "baron.exe".to_string(),
            sha256: sha256_file(&staged_path).unwrap(),
            size_bytes: fs::metadata(&staged_path).unwrap().len(),
            staged_path,
        }
    }

    fn planned_transaction(
        repo: &Path,
        local: &str,
        upstream: &str,
    ) -> (TransactionPaths, UpdateTransaction) {
        initialize_repo(repo);
        let base = payload("base");
        record_managed_baseline(repo, &[base], "3.3.0").unwrap();
        fs::write(repo.join("AGENTS.md"), local).unwrap();
        let candidate = staged_candidate(repo);
        let (paths, _) = create_verified_transaction(
            repo,
            &config(),
            &candidate,
            "3.3.0",
            &["codex".to_string()],
        )
        .unwrap();
        let transaction = plan_candidate_transaction(
            repo,
            &paths.state_path,
            "project-identity",
            "3.4.0",
            &[payload(upstream)],
        )
        .unwrap();
        (paths, transaction)
    }

    #[test]
    fn transaction_statuses_are_explicit_and_monotonic() {
        assert_eq!(TransactionStatus::Verified.as_str(), "verified");
        assert_eq!(TransactionStatus::Conflict.as_str(), "conflict");
        assert_eq!(TransactionStatus::Completed.as_str(), "completed");
        assert!(TransactionStatus::Verified.allows_transition_to(TransactionStatus::Planned));
        assert!(!TransactionStatus::Completed.allows_transition_to(TransactionStatus::Planned));
    }

    #[test]
    fn conflict_stages_base_local_upstream_and_empty_resolution_without_live_write() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        let (paths, transaction) = planned_transaction(&repo, "local", "upstream");

        assert_eq!(transaction.status, TransactionStatus::Conflict);
        assert_eq!(fs::read_to_string(repo.join("AGENTS.md")).unwrap(), "local");
        let packet = &transaction.packets[0];
        assert_eq!(read_packet(&paths, "BASE", packet).unwrap(), "base");
        assert_eq!(read_packet(&paths, "LOCAL", packet).unwrap(), "local");
        assert_eq!(read_packet(&paths, "UPSTREAM", packet).unwrap(), "upstream");
        assert_eq!(read_packet(&paths, "RESOLVED", packet).unwrap(), "");
        assert!(packet.resolved_sha256.is_none());
    }

    #[test]
    fn continuation_refuses_a_changed_live_target_before_any_write() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        let (paths, transaction) = planned_transaction(&repo, "local", "upstream");
        fs::write(
            packet_path(&paths, "RESOLVED", &transaction.packets[0], false).unwrap(),
            "resolved",
        )
        .unwrap();
        fs::write(repo.join("AGENTS.md"), "changed-after-plan").unwrap();

        let error = continue_transaction(&repo, &paths.state_path, "project-identity")
            .unwrap_err()
            .to_string();

        assert!(error.contains("changed after transaction planning"));
        assert_eq!(
            fs::read_to_string(repo.join("AGENTS.md")).unwrap(),
            "changed-after-plan"
        );
    }

    #[test]
    fn continuation_rejects_a_missing_packet_without_recreating_staging_directories() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        let (paths, transaction) = planned_transaction(&repo, "base", "upstream");
        let packet_root = paths
            .root
            .join("BASE")
            .join(&transaction.packets[0].adapter);
        fs::remove_dir_all(&packet_root).unwrap();

        let error = continue_transaction(&repo, &paths.state_path, "project-identity")
            .unwrap_err()
            .to_string();

        assert!(error.contains("packet directory is missing"));
        assert!(!packet_root.exists());
        assert_eq!(fs::read_to_string(repo.join("AGENTS.md")).unwrap(), "base");
    }

    #[test]
    fn abort_removes_only_unapplied_staged_state() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        let (paths, _) = planned_transaction(&repo, "local", "upstream");

        abort_transaction(&repo, &paths.state_path, "project-identity").unwrap();

        assert!(!paths.root.exists());
        assert_eq!(fs::read_to_string(repo.join("AGENTS.md")).unwrap(), "local");
        assert!(repo.join(".baron/managed-state/manifest.json").is_file());
    }

    #[test]
    fn every_project_activation_checkpoint_rolls_back_project_and_managed_baseline() {
        for failure in [
            ApplyFailurePoint::AfterManagedStateBackup,
            ApplyFailurePoint::BeforeManagedWrite(0),
            ApplyFailurePoint::AfterManagedWrite(0),
            ApplyFailurePoint::BeforeBaselineReplacement,
            ApplyFailurePoint::AfterBaselineReplacement,
        ] {
            let temp = tempdir().unwrap();
            let repo = temp.path().join("repo");
            fs::create_dir_all(&repo).unwrap();
            let (paths, transaction) = planned_transaction(&repo, "base", "upstream");
            assert_eq!(transaction.status, TransactionStatus::Planned);

            let error = apply_transaction_with_failure(
                &repo,
                &paths.state_path,
                "project-identity",
                Some(failure),
            )
            .unwrap_err()
            .to_string();

            assert!(error.contains("rolled back"), "failure point: {failure:?}");
            assert_eq!(
                fs::read_to_string(repo.join("AGENTS.md")).unwrap(),
                "base",
                "failure point: {failure:?}"
            );
            assert_eq!(
                load_managed_baseline(&repo).unwrap().installed_version,
                "3.3.0",
                "failure point: {failure:?}"
            );
            assert_eq!(
                load_transaction(
                    &repo,
                    &transaction_paths_for_state(&repo, &paths.state_path).unwrap()
                )
                .unwrap()
                .status,
                TransactionStatus::RolledBack,
                "failure point: {failure:?}"
            );
        }
    }

    #[test]
    fn recovery_rolls_back_an_applied_transaction_without_claiming_success() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        let (paths, _) = planned_transaction(&repo, "base", "upstream");
        let activated = continue_transaction(&repo, &paths.state_path, "project-identity").unwrap();
        assert_eq!(activated.status, TransactionStatus::ProjectActivated);
        assert_eq!(
            fs::read_to_string(repo.join("AGENTS.md")).unwrap(),
            "upstream"
        );

        let recovered = recover_transaction(&repo, &paths.state_path, "project-identity").unwrap();

        assert_eq!(recovered.status, TransactionStatus::RolledBack);
        assert_eq!(fs::read_to_string(repo.join("AGENTS.md")).unwrap(), "base");
        assert_eq!(
            load_managed_baseline(&repo).unwrap().installed_version,
            "3.3.0"
        );
    }

    #[test]
    fn startup_recovery_scans_and_rolls_back_every_incomplete_applied_transaction() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        let (paths, _) = planned_transaction(&repo, "base", "upstream");
        let activated = continue_transaction(&repo, &paths.state_path, "project-identity").unwrap();
        assert_eq!(activated.status, TransactionStatus::ProjectActivated);

        let recovered = recover_incomplete_transactions(&repo, "project-identity").unwrap();

        assert_eq!(recovered, vec![activated.transaction_id]);
        assert_eq!(fs::read_to_string(repo.join("AGENTS.md")).unwrap(), "base");
        assert_eq!(
            inspect_transaction(&repo, &paths.state_path, "project-identity")
                .unwrap()
                .status,
            TransactionStatus::RolledBack
        );
    }
}
