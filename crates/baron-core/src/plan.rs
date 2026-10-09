use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::control_plane::gate_evidence_status_strict_for_operation_in_generation;
use crate::execution_receipt::ReceiptContext;
use crate::operation::{task_id_for_task, LifecycleIdentity, OperationContext, SupportedAdapter};
use crate::proof::{
    proof_for_operation_in_generation, proof_has_current_receipt, proof_operation_binding,
    proof_satisfies_risk,
};
use crate::risk::{classify_risk, RiskLane};
use crate::safe_io::{
    acquire_project_lock, artifact_instance_id, create_new_text, read_text, read_text_required,
    replace_text,
};
use crate::trace::{
    latest_trace_score_for_operation_in_vault_with_risk_and_generation,
    latest_trace_score_for_operation_with_risk_and_generation, TraceOperationBinding, TraceTier,
};
use crate::vault::{canonical_project_id, VaultContext};

const MANAGED_PLAN_ROOT: &str = "docs/baron/plans";
const ACTIVE_PLAN_INDEX_FILE: &str = "ACTIVE.md";
const ACTIVE_PLAN_MARKER: &str = "<!-- BARON:ACTIVE-PLAN ";
const PLAN_TRANSITION_JOURNAL: &str = ".baron/plan-transition.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanRecord {
    pub title: String,
    pub risk: RiskLane,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
    pub resumed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionEvidenceStatus {
    pub passed: bool,
    pub issues: Vec<String>,
}

/// Identity captured when a plan is started from a concrete Baron operation.
/// Legacy title-only plans remain readable, but medium/high-risk completion
/// cannot authorize them without this complete binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanOperationBinding {
    pub task_id: String,
    pub operation_id: String,
    pub adapter: String,
    pub session_id: String,
    pub request_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PlanTransitionJournal {
    schema_version: u32,
    plan_path: String,
    source_hash: String,
    vault_source_hash: Option<String>,
    target_hash: String,
    source_status: String,
    status: String,
    updated_at: String,
    verification: Option<String>,
    progress_note: String,
    title: String,
    current_title: String,
    risk: RiskLane,
    task_id: String,
    binding: Option<PlanOperationBinding>,
    #[serde(default)]
    authority_generation: Option<String>,
    next_action: String,
}

impl PlanOperationBinding {
    /// Copy a validated lifecycle tuple without deriving new identity.
    pub fn from_identity(identity: &LifecycleIdentity) -> Self {
        Self {
            task_id: identity.task_id().to_string(),
            operation_id: identity.operation_id().to_string(),
            adapter: identity.adapter().as_str().to_string(),
            session_id: identity.session_id().to_string(),
            request_id: identity.request_id().to_string(),
        }
    }

    pub fn to_operation_context(&self) -> Result<OperationContext> {
        let adapter =
            self.adapter
                .parse()
                .map_err(|error: crate::operation::OperationIdentityError| {
                    anyhow::anyhow!(error.to_string())
                })?;
        Ok(OperationContext::new(adapter)
            .with_task_id(self.task_id.clone())
            .with_operation_id(self.operation_id.clone())
            .with_session_id(self.session_id.clone())
            .with_request_id(self.request_id.clone()))
    }

    pub fn proof_binding(&self) -> ReceiptContext {
        ReceiptContext::new(
            self.task_id.clone(),
            self.operation_id.clone(),
            self.adapter.clone(),
            self.session_id.clone(),
            self.request_id.clone(),
            "proof",
        )
    }

    fn trace_binding(&self, proof_id: &str) -> TraceOperationBinding {
        TraceOperationBinding {
            task_id: self.task_id.clone(),
            operation_id: self.operation_id.clone(),
            adapter: self.adapter.clone(),
            session_id: self.session_id.clone(),
            request_id: self.request_id.clone(),
            proof_id: proof_id.to_string(),
        }
    }
}

fn binding_for_identity(
    vault: &VaultContext,
    identity: &LifecycleIdentity,
) -> Result<PlanOperationBinding> {
    if identity.project_id() != vault.project_id {
        bail!(
            "plan lifecycle identity project `{}` does not match Vault project `{}`",
            identity.project_id(),
            vault.project_id
        );
    }
    Ok(PlanOperationBinding::from_identity(identity))
}

/// Durable per-operation lookup for identified active plans.
///
/// `CURRENT.md` remains a human-facing projection of the most recently
/// started/resumed plan. It cannot be the only authority when multiple
/// identified operations are active in one project, so this small managed
/// index preserves the exact plan path for each operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ActivePlanIndexEntry {
    task_id: String,
    operation_id: String,
    adapter: String,
    session_id: String,
    request_id: String,
    plan_path: String,
    status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    authority_generation: Option<String>,
}

impl ActivePlanIndexEntry {
    fn from_plan(
        binding: &PlanOperationBinding,
        repo_root: &Path,
        plan_path: &Path,
        status: &str,
        authority_generation: Option<String>,
    ) -> Self {
        Self {
            task_id: binding.task_id.clone(),
            operation_id: binding.operation_id.clone(),
            adapter: binding.adapter.clone(),
            session_id: binding.session_id.clone(),
            request_id: binding.request_id.clone(),
            plan_path: normalize(plan_path, repo_root),
            status: status.to_string(),
            authority_generation,
        }
    }

    fn binding(&self) -> Result<PlanOperationBinding> {
        self.adapter
            .parse::<SupportedAdapter>()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        Ok(PlanOperationBinding {
            task_id: self.task_id.clone(),
            operation_id: self.operation_id.clone(),
            adapter: self.adapter.clone(),
            session_id: self.session_id.clone(),
            request_id: self.request_id.clone(),
        })
    }
}

/// Canonical active-plan metadata after managed plan authority validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivePlanAuthority {
    pub title: String,
    pub risk: RiskLane,
    pub binding: Option<PlanOperationBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndexedPlanAuthority {
    pub title: String,
    pub risk: RiskLane,
    pub binding: Option<PlanOperationBinding>,
    pub authority_generation: Option<String>,
}

impl From<IndexedPlanAuthority> for ActivePlanAuthority {
    fn from(authority: IndexedPlanAuthority) -> Self {
        Self {
            title: authority.title,
            risk: authority.risk,
            binding: authority.binding,
        }
    }
}

/// Resolve correctness-sensitive plan authority from one exact lifecycle
/// binding. `CURRENT.md` is never consulted: ACTIVE and the managed plan's
/// canonical frontmatter provide authority independently of the presentation.
pub fn active_plan_authority_for_binding(
    repo_root: impl AsRef<Path>,
    binding: &PlanOperationBinding,
) -> Result<Option<ActivePlanAuthority>> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    active_plan_authority_for_binding_locked(repo_root, binding)
}

/// Resolve exact authority only from the durable ACTIVE index. This is for
/// correctness-sensitive ingress that requires an explicit persisted
/// operation selector; unlike the compatibility API above, it never discovers
/// an unindexed pre-ACTIVE plan by scanning plan frontmatter.
pub fn indexed_active_plan_authority_for_binding(
    repo_root: impl AsRef<Path>,
    binding: &PlanOperationBinding,
) -> Result<Option<ActivePlanAuthority>> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    Ok(indexed_active_plan_authority_for_binding_locked(repo_root, binding)?.map(Into::into))
}

pub(crate) fn indexed_active_plan_authority_for_binding_locked(
    repo_root: &Path,
    binding: &PlanOperationBinding,
) -> Result<Option<IndexedPlanAuthority>> {
    let entries = load_active_plan_index(repo_root)?;
    let mut matches = entries
        .iter()
        .filter(|entry| is_active_plan_status(&entry.status))
        .filter_map(|entry| match entry.binding() {
            Ok(actual) if actual == *binding => Some(Ok(entry)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>>>()?;
    if matches.len() > 1 {
        bail!(
            "ACTIVE index contains multiple plans for operation `{}`",
            binding.operation_id
        );
    }
    let Some(entry) = matches.pop() else {
        return Ok(None);
    };
    let path = resolve_managed_plan_path(repo_root, &entry.plan_path)?;
    let mut active = load_identified_active_plan(repo_root, &path, binding)?;
    if active.status != entry.status || active.authority_generation != entry.authority_generation {
        bail!(
            "Baron active plan index status or authority generation does not match {}",
            entry.plan_path
        );
    }
    active.authority_indexed = true;
    active.ensure_authority()?;
    Ok(Some(IndexedPlanAuthority {
        title: active.title,
        risk: active.risk,
        binding: active.binding,
        authority_generation: active.authority_generation,
    }))
}

/// Return every validated active operation bound to one host adapter/session.
/// Session-only hook events use this to reject a correlation when another
/// operation in the same host session has no matching turn identifier.
pub(crate) fn active_plan_bindings_for_session(
    repo_root: impl AsRef<Path>,
    adapter: SupportedAdapter,
    session_id: &str,
) -> Result<Vec<PlanOperationBinding>> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    Ok(discover_identified_active_plans(repo_root)?
        .into_iter()
        .filter_map(|active| {
            active.binding.filter(|binding| {
                binding.adapter == adapter.as_str() && binding.session_id == session_id
            })
        })
        .collect())
}

/// Resolve the plan authority carried by an operation-bound trace. The trace
/// module uses the locked variant for its publication revalidation.
pub fn active_plan_authority_for_trace_binding(
    repo_root: impl AsRef<Path>,
    binding: &TraceOperationBinding,
) -> Result<Option<ActivePlanAuthority>> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    active_plan_authority_for_trace_binding_locked(repo_root, binding)
}

pub(crate) fn active_plan_authority_for_trace_binding_locked(
    repo_root: &Path,
    binding: &TraceOperationBinding,
) -> Result<Option<ActivePlanAuthority>> {
    let plan_binding = PlanOperationBinding {
        task_id: binding.task_id.clone(),
        operation_id: binding.operation_id.clone(),
        adapter: binding.adapter.clone(),
        session_id: binding.session_id.clone(),
        request_id: binding.request_id.clone(),
    };
    active_plan_authority_for_binding_locked(repo_root, &plan_binding)
}

pub(crate) fn indexed_active_plan_authority_for_trace_binding_locked(
    repo_root: &Path,
    binding: &TraceOperationBinding,
) -> Result<Option<IndexedPlanAuthority>> {
    let plan_binding = PlanOperationBinding {
        task_id: binding.task_id.clone(),
        operation_id: binding.operation_id.clone(),
        adapter: binding.adapter.clone(),
        session_id: binding.session_id.clone(),
        request_id: binding.request_id.clone(),
    };
    indexed_active_plan_authority_for_binding_locked(repo_root, &plan_binding)
}

fn active_plan_authority_for_binding_locked(
    repo_root: &Path,
    binding: &PlanOperationBinding,
) -> Result<Option<ActivePlanAuthority>> {
    let Some(active) = active_plan_for_binding(repo_root, binding)? else {
        return Ok(None);
    };
    active.ensure_authority()?;
    Ok(Some(ActivePlanAuthority {
        title: active.title,
        risk: active.risk,
        binding: active.binding,
    }))
}

pub fn active_plan_authority(repo_root: impl AsRef<Path>) -> Result<Option<ActivePlanAuthority>> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    if let Some(active) = sole_active_managed_plan(repo_root)? {
        if active.binding.is_some() {
            active.ensure_authority()?;
            return Ok(Some(ActivePlanAuthority {
                title: active.title,
                risk: active.risk,
                binding: active.binding,
            }));
        }
    }

    // Keep the historical CURRENT-linked path for unbound legacy plans. An
    // identified operation is selected from validated managed plan authority
    // above, never accepted or vetoed by the presentation pointer.
    let Some(active) = resolve_legacy_active_plan(repo_root)? else {
        return Ok(None);
    };
    active.ensure_authority()?;
    Ok(Some(ActivePlanAuthority {
        title: active.title,
        risk: active.risk,
        binding: active.binding,
    }))
}

/// Return the active plan's persisted operation binding for ingress adapters.
/// Missing bindings remain explicit instead of being inferred from the latest
/// repository artifact.
pub fn active_plan_operation_binding(
    repo_root: impl AsRef<Path>,
) -> Result<Option<PlanOperationBinding>> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    if let Some(active) = sole_active_managed_plan(repo_root)? {
        if active.binding.is_some() {
            active.ensure_authority()?;
            let binding = active
                .binding
                .context("identified active plan has no binding")?;
            indexed_active_plan_authority_for_binding_locked(repo_root, &binding)?
                .context("identified active plan is not present in the validated ACTIVE index")?;
            return Ok(Some(binding));
        }
    }

    // Preserve CURRENT-based compatibility and integrity checks for legacy
    // unbound plans. An identified active operation above never uses CURRENT
    // to select or veto its binding.
    Ok(resolve_legacy_active_plan(repo_root)?.and_then(|active| active.binding))
}

/// Inspect only canonical managed plans, without consulting the legacy UI
/// projection. Used by recovery ingress to preserve unbound diagnostic packets
/// while refusing ambiguous concurrent operation selection.
pub(crate) fn managed_active_plan_operation_binding(
    repo_root: &Path,
) -> Result<Option<PlanOperationBinding>> {
    let _lock = acquire_project_lock(repo_root)?;
    let Some(active) = sole_active_managed_plan(repo_root)? else {
        return Ok(None);
    };
    active.ensure_authority()?;
    Ok(active.binding)
}

/// Evaluate the current active plan using the same scoped completion evidence
/// consumed by plan completion and completion-integrity diagnostics. A
/// completed plan is not an active reconciliation target. Identified active
/// operations are resolved from validated managed authority, independently of
/// the human-facing CURRENT projection.
pub fn active_plan_completion_evidence_status(
    repo_root: impl AsRef<Path>,
) -> Result<Option<CompletionEvidenceStatus>> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    match sole_active_managed_plan(repo_root) {
        Ok(Some(active)) if active.binding.is_some() && !active.authority_indexed => {
            return Ok(Some(CompletionEvidenceStatus {
                passed: false,
                issues: vec!["identified plan is missing indexed ACTIVE authority".to_string()],
            }));
        }
        Ok(Some(active)) if active.binding.is_some() => {
            return Ok(Some(completion_evidence_status(repo_root, &active, None)?));
        }
        Err(error) => {
            return Ok(Some(CompletionEvidenceStatus {
                passed: false,
                issues: vec![error.to_string()],
            }));
        }
        Ok(_) => {}
    }
    match resolve_legacy_active_plan(repo_root) {
        Ok(Some(active)) => Ok(Some(completion_evidence_status(repo_root, &active, None)?)),
        Ok(None)
            if read_text(repo_root.join("docs/baron/plans/CURRENT.md"))?.is_some()
                && active_plan(repo_root)?.is_none() =>
        {
            Ok(Some(CompletionEvidenceStatus {
                passed: false,
                issues: vec!["active plan is missing".to_string()],
            }))
        }
        Ok(None) => Ok(None),
        Err(error) => Ok(Some(CompletionEvidenceStatus {
            passed: false,
            issues: vec![error.to_string()],
        })),
    }
}

/// Evaluate completion evidence for one exact lifecycle identity. A missing
/// operation plan is a failed authority lookup, never a reason to inspect the
/// latest CURRENT projection.
pub fn active_plan_completion_evidence_status_for_identity(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    identity: &LifecycleIdentity,
) -> Result<Option<CompletionEvidenceStatus>> {
    let binding = binding_for_identity(vault, identity)?;
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    let active = match active_plan_for_indexed_binding(repo_root, &binding) {
        Ok(Some(active)) => active,
        Ok(None) => {
            return Ok(Some(CompletionEvidenceStatus {
                passed: false,
                issues: vec![format!(
                    "active plan for operation `{}` is missing",
                    binding.operation_id
                )],
            }));
        }
        Err(error) => {
            return Ok(Some(CompletionEvidenceStatus {
                passed: false,
                issues: vec![error.to_string()],
            }));
        }
    };
    Ok(Some(completion_evidence_status(
        repo_root,
        &active,
        Some(vault),
    )?))
}

pub fn start_or_resume_plan(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    title: &str,
) -> Result<PlanRecord> {
    start_or_resume_plan_internal(repo_root.as_ref(), vault, title, None)
}

/// Start or resume a plan while persisting the exact operation identity that
/// will be required by medium/high-risk completion. This is the operation-
/// aware entry point used by trusted Baron integrations.
pub fn start_or_resume_plan_for_operation(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    title: &str,
    operation: &OperationContext,
) -> Result<PlanRecord> {
    let identity = operation
        .lifecycle_identity_for_task(&vault.project_id, title)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    start_or_resume_plan_for_identity(repo_root, vault, title, &identity)
}

/// Start or resume a plan from one complete canonical lifecycle identity.
/// Legacy title-only plan files remain readable, but this entry point never
/// upgrades an unbound plan by inference.
pub fn start_or_resume_plan_for_identity(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    title: &str,
    identity: &LifecycleIdentity,
) -> Result<PlanRecord> {
    if identity.project_id() != vault.project_id {
        bail!(
            "plan lifecycle identity project `{}` does not match Vault project `{}`",
            identity.project_id(),
            vault.project_id
        );
    }
    identity
        .validate_task(title)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let binding = PlanOperationBinding::from_identity(identity);
    start_or_resume_plan_internal(repo_root.as_ref(), vault, title, Some(&binding))
}

pub fn update_plan_for_identity(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    note: &str,
    identity: &LifecycleIdentity,
) -> Result<()> {
    let binding = binding_for_identity(vault, identity)?;
    update_plan_for_binding(repo_root.as_ref(), vault, note, &binding)
}

pub fn interrupt_plan_for_identity(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    state: &str,
    identity: &LifecycleIdentity,
) -> Result<()> {
    let binding = binding_for_identity(vault, identity)?;
    interrupt_plan_for_binding(repo_root.as_ref(), vault, state, &binding)
}

pub fn complete_plan_for_identity(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    verification_summary: &str,
    identity: &LifecycleIdentity,
) -> Result<()> {
    let binding = binding_for_identity(vault, identity)?;
    complete_plan_for_binding(repo_root.as_ref(), vault, verification_summary, &binding)
}

fn start_or_resume_plan_internal(
    repo_root: &Path,
    vault: &VaultContext,
    title: &str,
    binding: Option<&PlanOperationBinding>,
) -> Result<PlanRecord> {
    let title = title.trim();
    let _lock = acquire_project_lock(repo_root)?;
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    if let Some(binding) = binding {
        validate_shared_operation_plan_ownership(repo_root, vault, binding)?;
    }
    ensure_active_plan_vault_index_consistent(repo_root, vault)?;
    let matching_active = match binding {
        Some(requested) => active_plan_for_new_operation(repo_root, requested)?,
        None => resolve_legacy_active_plan(repo_root)?,
    };
    if matching_active.is_none() {
        if let Some(requested) = binding {
            let mut paths = Vec::new();
            collect_managed_plan_files(&repo_root.join(MANAGED_PLAN_ROOT), &mut paths)?;
            for path in paths {
                let text = read_text_required(&path)?;
                if !claims_baron_plan(&text) {
                    continue;
                }
                let metadata = load_plan_file_metadata(&path)?;
                if is_active_plan_status(&metadata.status)
                    && metadata.title.eq_ignore_ascii_case(title)
                {
                    match metadata.operation_binding()? {
                        None => bail!(
                            "Cannot authorize identified plan `{title}` from a legacy unbound plan"
                        ),
                        Some(existing) if existing != *requested => bail!(
                            "Cannot resume plan `{title}` under a different operation identity"
                        ),
                        Some(_) => {
                            bail!("active plan authority lookup is inconsistent for `{title}`")
                        }
                    }
                }
            }
        }
    }
    if let Some(active) = matching_active {
        active.ensure_authority()?;
        if (binding.is_some() || active.title.eq_ignore_ascii_case(title))
            && active.status != "completed"
        {
            if let Some(requested) = binding {
                if active.binding.as_ref() != Some(requested) {
                    bail!("Cannot resume plan `{title}` under a different operation identity");
                }
            }
            publish_plan_status_transition(
                repo_root,
                vault,
                PlanStatusTransition {
                    status: "in_progress",
                    verification: None,
                    progress_note: "Plan resumed.",
                    current_title: title,
                    next_action: "continue from last known state",
                    authority_generation: if binding.is_some() && !active.authority_indexed {
                        Some(artifact_instance_id(&today())?)
                    } else {
                        None
                    },
                },
                &active,
            )?;
            return Ok(PlanRecord {
                title: title.to_string(),
                risk: active.risk,
                repo_path: active.path.clone(),
                vault_path: vault_plan_path(repo_root, vault, &active.path)?,
                resumed: true,
            });
        }
        if binding.is_none() && active.status != "completed" {
            bail!(
                "Cannot start unbound plan `{title}` while active plan `{}` exists; provide an explicit operation identity",
                active.title
            );
        }
    }
    let risk = classify_risk(title);
    let date = today();
    let instance_id = artifact_instance_id(&date)?;
    let authority_generation = binding.map(|_| instance_id.as_str());
    let repo_path = repo_root
        .join("docs/baron/plans")
        .join(&date)
        .join(format!("{date}-{}-{instance_id}.md", slugify(title)));
    let vault_path = vault_plan_path(repo_root, vault, &repo_path)?;
    let content = plan_content(
        &vault.project_id,
        title,
        risk,
        binding,
        authority_generation,
    )?;
    create_new_text(&repo_path, &content)?;
    create_new_text(&vault_path, &content)?;
    append_unique(
        &repo_root.join("docs/baron/plans/INDEX.md"),
        "# Baron Plan Index\n\n",
        &format!(
            "- [{}]({}) - status: `in_progress` - risk: `{}`",
            title,
            normalize(&repo_path, repo_root),
            risk.as_str()
        ),
    )?;
    append_unique(
        &vault.project_root.join("Plans/INDEX.md"),
        "# Baron Plan Index\n\n",
        &format!(
            "- [{}]({}) - status: `in_progress` - risk: `{}`",
            title,
            normalize(&vault_path, &vault.project_root),
            risk.as_str()
        ),
    )?;
    if let Some(binding) = binding {
        upsert_active_plan_index(repo_root, vault, binding, &repo_path, "in_progress")?;
    }
    write_current(
        repo_root,
        vault,
        CurrentPlanView {
            title,
            risk,
            status: "in_progress",
            plan_path: &repo_path,
            next_action: "continue from current task scope",
            verification: "not_run",
            binding,
            task_id: None,
        },
    )?;
    Ok(PlanRecord {
        title: title.to_string(),
        risk,
        repo_path,
        vault_path,
        resumed: false,
    })
}

pub fn update_plan(repo_root: impl AsRef<Path>, vault: &VaultContext, note: &str) -> Result<()> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    ensure_active_plan_vault_index_consistent(repo_root, vault)?;
    let active = require_legacy_active_plan(repo_root)?;
    update_plan_state(repo_root, vault, note, &active)
}

fn update_plan_for_binding(
    repo_root: &Path,
    vault: &VaultContext,
    note: &str,
    binding: &PlanOperationBinding,
) -> Result<()> {
    let _lock = acquire_project_lock(repo_root)?;
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    ensure_active_plan_vault_index_consistent(repo_root, vault)?;
    let active = require_active_plan_for_binding(repo_root, binding)?;
    update_plan_state(repo_root, vault, note, &active)
}

fn update_plan_state(
    repo_root: &Path,
    vault: &VaultContext,
    note: &str,
    active: &ActivePlan,
) -> Result<()> {
    append_progress(&active.path, note.trim())?;
    mirror_plan(repo_root, vault, &active.path)?;
    if let Some(binding) = active.binding.as_ref() {
        upsert_active_plan_index(repo_root, vault, binding, &active.path, &active.status)?;
    }
    write_current(
        repo_root,
        vault,
        CurrentPlanView {
            title: &active.title,
            risk: active.risk,
            status: &active.status,
            plan_path: &active.path,
            next_action: note.trim(),
            verification: "not_run",
            binding: active.binding.as_ref(),
            task_id: Some(&active.task_id),
        },
    )
}

pub fn interrupt_plan(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    state: &str,
) -> Result<()> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    ensure_active_plan_vault_index_consistent(repo_root, vault)?;
    let active = require_legacy_active_plan(repo_root)?;
    interrupt_plan_state(repo_root, vault, state, &active)
}

fn interrupt_plan_for_binding(
    repo_root: &Path,
    vault: &VaultContext,
    state: &str,
    binding: &PlanOperationBinding,
) -> Result<()> {
    let _lock = acquire_project_lock(repo_root)?;
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    ensure_active_plan_vault_index_consistent(repo_root, vault)?;
    let active = require_active_plan_for_binding(repo_root, binding)?;
    interrupt_plan_state(repo_root, vault, state, &active)
}

struct PlanStatusTransition<'a> {
    status: &'a str,
    verification: Option<&'a str>,
    progress_note: &'a str,
    current_title: &'a str,
    next_action: &'a str,
    authority_generation: Option<String>,
}

fn publish_plan_status_transition(
    repo_root: &Path,
    vault: &VaultContext,
    transition: PlanStatusTransition<'_>,
    active: &ActivePlan,
) -> Result<()> {
    let PlanStatusTransition {
        status,
        verification,
        progress_note,
        current_title,
        next_action,
        authority_generation,
    } = transition;
    let allowed_transition = matches!(
        (active.status.as_str(), status),
        ("in_progress", "in_progress" | "interrupted" | "completed")
            | ("interrupted", "in_progress" | "interrupted" | "completed")
    );
    if !allowed_transition {
        bail!(
            "Plan status transition `{}` -> `{status}` is not supported",
            active.status
        );
    }
    if status == "completed" && verification.is_none_or(|value| value.trim().is_empty()) {
        bail!("Plan completion requires a non-empty verification summary.");
    }

    let source = read_text_required(&active.path)?;
    let updated_at = now();
    let verification = verification.map(single_line);
    let target = plan_transition_content(
        &source,
        status,
        &updated_at,
        verification.as_deref(),
        progress_note,
        authority_generation.as_deref(),
    );
    let plan_path = normalize(&active.path, repo_root);
    let vault_path = vault_plan_path(repo_root, vault, &active.path)?;
    let vault_source_hash = read_text(&vault_path)?.map(|content| plan_content_hash(&content));
    let journal = PlanTransitionJournal {
        schema_version: 1,
        plan_path,
        source_hash: plan_content_hash(&source),
        vault_source_hash,
        target_hash: plan_content_hash(&target),
        source_status: active.status.clone(),
        status: status.to_string(),
        updated_at,
        verification,
        progress_note: progress_note.to_string(),
        title: active.title.clone(),
        current_title: current_title.to_string(),
        risk: active.risk,
        task_id: active.task_id.clone(),
        binding: active.binding.clone(),
        authority_generation,
        next_action: next_action.to_string(),
    };
    let journal_path = repo_root.join(PLAN_TRANSITION_JOURNAL);
    if read_text(&journal_path)?.is_some() {
        bail!("A pending plan status transition must be recovered before another transition");
    }
    let serialized = format!("{}\n", serde_json::to_string_pretty(&journal)?);
    write(&journal_path, &serialized)?;
    recover_pending_plan_transition(repo_root, vault)
}

/// Complete an interrupted status publication from its write-ahead intent.
/// Callers hold checkout then Vault capsule locks before invoking this helper.
fn recover_pending_plan_transition(repo_root: &Path, vault: &VaultContext) -> Result<()> {
    let journal_path = repo_root.join(PLAN_TRANSITION_JOURNAL);
    let Some(serialized) = read_text(&journal_path)? else {
        return Ok(());
    };
    let journal: PlanTransitionJournal = serde_json::from_str(&serialized)
        .context("Pending plan status transition journal is malformed")?;
    if journal.schema_version != 1
        || !matches!(
            journal.source_status.as_str(),
            "in_progress" | "interrupted"
        )
        || !matches!(
            journal.status.as_str(),
            "in_progress" | "interrupted" | "completed"
        )
    {
        bail!("Pending plan status transition journal has an unsupported schema or status");
    }
    if journal.status == "completed"
        && journal
            .verification
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
    {
        bail!("Pending plan completion journal has no verification summary");
    }

    let plan_path = resolve_managed_plan_path(repo_root, &journal.plan_path)?;
    if normalize(&plan_path, repo_root) != journal.plan_path {
        bail!("Pending plan transition path is not canonical");
    }
    let current = read_text_required(&plan_path)?;
    let current_hash = plan_content_hash(&current);
    let target = if current_hash == journal.source_hash {
        let source_metadata = load_plan_file_metadata(&plan_path)?;
        validate_transition_metadata(
            repo_root,
            &source_metadata,
            &journal,
            &journal.source_status,
        )?;
        let target = plan_transition_content(
            &current,
            &journal.status,
            &journal.updated_at,
            journal.verification.as_deref(),
            &journal.progress_note,
            journal.authority_generation.as_deref(),
        );
        if plan_content_hash(&target) != journal.target_hash {
            bail!("Pending plan transition target hash does not match its journal");
        }
        write(&plan_path, &target)?;
        target
    } else if current_hash == journal.target_hash {
        current
    } else {
        bail!(
            "Managed plan changed during a pending status transition; preserving it for recovery: {}",
            plan_path.display()
        );
    };

    let metadata = load_plan_file_metadata(&plan_path)?;
    validate_transition_metadata(repo_root, &metadata, &journal, &journal.status)?;
    let binding = metadata.operation_binding()?;
    if journal.status == "completed" {
        let active = ActivePlan {
            title: metadata.title.clone(),
            path: plan_path.clone(),
            status: metadata.status.clone(),
            risk: metadata.risk,
            task_id: metadata.task_id.clone(),
            binding: binding.clone(),
            linked_status: Some(metadata.status.clone()),
            authority_issues: Vec::new(),
            authority_generation: metadata.authority_generation.clone(),
            authority_indexed: false,
        };
        if let Some(issue) = completion_evidence_status(repo_root, &active, Some(vault))?
            .issues
            .into_iter()
            .next()
        {
            bail!("Pending plan completion recovery is blocked: {issue}");
        }
    }

    let vault_path = vault_plan_path(repo_root, vault, &plan_path)?;
    let vault_content = read_text(&vault_path).with_context(|| {
        format!(
            "Pending plan transition cannot validate its Vault mirror: {}",
            vault_path.display()
        )
    })?;
    let vault_hash = vault_content.as_deref().map(plan_content_hash);
    if vault_hash != journal.vault_source_hash
        && vault_hash.as_deref() != Some(journal.target_hash.as_str())
    {
        bail!(
            "Vault plan mirror changed during a pending status transition; preserving it for recovery: {}",
            vault_path.display()
        );
    }
    write(&vault_path, &target)?;
    update_plan_indexes(
        repo_root,
        vault,
        &journal.title,
        &plan_path,
        journal.risk,
        &journal.status,
    )?;
    if let Some(binding) = binding.as_ref() {
        upsert_active_plan_index(repo_root, vault, binding, &plan_path, &journal.status)?;
    }
    write_current(
        repo_root,
        vault,
        CurrentPlanView {
            title: &journal.current_title,
            risk: journal.risk,
            status: &journal.status,
            plan_path: &plan_path,
            next_action: &journal.next_action,
            verification: journal.verification.as_deref().unwrap_or("not_run"),
            binding: binding.as_ref(),
            task_id: Some(&journal.task_id),
        },
    )?;

    if read_text_required(&journal_path)? != serialized {
        bail!("Pending plan transition journal changed during recovery");
    }
    fs::remove_file(&journal_path).with_context(|| {
        format!(
            "Could not clear recovered plan transition: {}",
            journal_path.display()
        )
    })?;
    Ok(())
}

fn validate_transition_metadata(
    repo_root: &Path,
    metadata: &PlanFileMetadata,
    journal: &PlanTransitionJournal,
    expected_status: &str,
) -> Result<()> {
    if metadata.status != expected_status
        || metadata.title != journal.title
        || metadata.risk != journal.risk
        || metadata.task_id != journal.task_id
        || metadata.operation_binding()? != journal.binding
        || validate_linked_plan_authority(repo_root, metadata)? != journal.binding
    {
        bail!("Pending plan transition metadata does not match canonical plan authority");
    }
    Ok(())
}

fn plan_transition_content(
    content: &str,
    status: &str,
    updated_at: &str,
    verification: Option<&str>,
    progress_note: &str,
    authority_generation: Option<&str>,
) -> String {
    let mut found_authority_generation = false;
    let mut updated = content
        .lines()
        .map(|line| {
            if line.starts_with("status: ") {
                format!("status: {status}")
            } else if line.starts_with("updated: ") {
                format!("updated: {updated_at}")
            } else if line.starts_with("authority_generation: ") {
                found_authority_generation = true;
                authority_generation
                    .map(|generation| format!("authority_generation: {generation}"))
                    .unwrap_or_else(|| line.to_string())
            } else if line.starts_with("verification: ") {
                verification
                    .map(|value| format!("verification: {value}"))
                    .unwrap_or_else(|| line.to_string())
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>();
    if let Some(generation) = authority_generation.filter(|_| !found_authority_generation) {
        if let Some(index) = updated
            .iter()
            .position(|line| line.starts_with("created: "))
        {
            updated.insert(index, format!("authority_generation: {generation}"));
        }
    }
    let updated = updated.join("\n");
    format!("{updated}\n- {updated_at} - {progress_note}\n")
}

fn plan_content_hash(content: &str) -> String {
    format!("{:x}", Sha256::digest(content.as_bytes()))
}

fn interrupt_plan_state(
    repo_root: &Path,
    vault: &VaultContext,
    state: &str,
    active: &ActivePlan,
) -> Result<()> {
    let state = state.trim();
    publish_plan_status_transition(
        repo_root,
        vault,
        PlanStatusTransition {
            status: "interrupted",
            verification: None,
            progress_note: &format!("Interrupted: {state}"),
            current_title: &active.title,
            next_action: state,
            authority_generation: None,
        },
        active,
    )
}

pub fn complete_plan(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    verification_summary: &str,
) -> Result<()> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    ensure_active_plan_vault_index_consistent(repo_root, vault)?;
    let active = require_legacy_active_plan(repo_root)?;
    complete_plan_state(repo_root, vault, verification_summary, &active)
}

fn complete_plan_for_binding(
    repo_root: &Path,
    vault: &VaultContext,
    verification_summary: &str,
    binding: &PlanOperationBinding,
) -> Result<()> {
    let _lock = acquire_project_lock(repo_root)?;
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    ensure_active_plan_vault_index_consistent(repo_root, vault)?;
    let active = require_active_plan_for_binding(repo_root, binding)?;
    complete_plan_state(repo_root, vault, verification_summary, &active)
}

fn complete_plan_state(
    repo_root: &Path,
    vault: &VaultContext,
    verification_summary: &str,
    active: &ActivePlan,
) -> Result<()> {
    if let Some(issue) = completion_evidence_status(repo_root, active, Some(vault))?
        .issues
        .into_iter()
        .next()
    {
        bail!("Plan completion blocked: {issue}.");
    }
    if verification_summary.trim().is_empty() {
        bail!("Plan completion requires a non-empty verification summary.");
    }
    publish_plan_status_transition(
        repo_root,
        vault,
        PlanStatusTransition {
            status: "completed",
            verification: Some(verification_summary.trim()),
            progress_note: &format!(
                "Completed with verification: {}",
                verification_summary.trim()
            ),
            current_title: &active.title,
            next_action: "start the next explicit task",
            authority_generation: None,
        },
        active,
    )
}

/// Evaluate every completion-sensitive artifact from one active plan scope.
/// Both completion and post-completion integrity diagnostics call this helper
/// so they cannot disagree about which proof, trace, or gates are authoritative.
fn completion_evidence_status(
    repo_root: &Path,
    active: &ActivePlan,
    vault: Option<&VaultContext>,
) -> Result<CompletionEvidenceStatus> {
    if !active.authority_issues.is_empty() {
        return Ok(CompletionEvidenceStatus {
            passed: false,
            issues: active.authority_issues.clone(),
        });
    }
    let issues = completion_evidence_issues(repo_root, active, vault)?;
    Ok(CompletionEvidenceStatus {
        passed: issues.is_empty(),
        issues,
    })
}

fn completion_evidence_issues(
    repo_root: &Path,
    active: &ActivePlan,
    vault: Option<&VaultContext>,
) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    let Some(expected_binding) = active.binding.as_ref() else {
        issues.push("proof is missing".to_string());
        issues.push("passing trace is missing".to_string());
        issues.push("active plan operation binding is missing".to_string());
        return Ok(issues);
    };

    let proof_binding = expected_binding.proof_binding();
    let proof = proof_for_operation_in_generation(
        repo_root,
        &proof_binding,
        active.authority_generation.as_deref(),
    )?;
    if let Some(proof) = proof.as_ref() {
        if !proof_satisfies_risk(&proof.summary, active.risk) {
            issues.push(format!(
                "proof does not satisfy `{}` risk requirements",
                active.risk.as_str()
            ));
        }
        let Some(actual_binding) = proof_operation_binding(proof) else {
            issues.push("proof operation identity does not match the active plan".to_string());
            return Ok(issues);
        };
        if !same_operation_binding(&actual_binding, &proof_binding)
            || actual_binding.gate_kind.trim() != "proof"
        {
            issues.push("proof operation identity does not match the active plan".to_string());
        }
        if active.risk != RiskLane::Low && !proof_has_current_receipt(repo_root, proof)? {
            issues.push(
                "medium/high-risk proof must reference a current trusted execution receipt"
                    .to_string(),
            );
        }
    } else {
        issues.push("proof is missing".to_string());
    }

    if active.risk != RiskLane::Low {
        let required_agents = [
            "code-reviewer".to_string(),
            "security-auditor".to_string(),
            "test-engineer".to_string(),
        ];
        let gate_operation = expected_binding.to_operation_context()?;
        let gate_status = gate_evidence_status_strict_for_operation_in_generation(
            repo_root,
            &required_agents,
            &gate_operation,
            active.authority_generation.as_deref(),
        )?;
        if !gate_status.passed {
            issues.push(format!(
                "trusted quality-gate receipts are missing for {}",
                gate_status.missing_agents.join(", ")
            ));
        }
    }

    if let Some(proof) = proof {
        let trace_binding = expected_binding.trace_binding(&proof.id);
        let trace = match vault {
            Some(vault) => latest_trace_score_for_operation_in_vault_with_risk_and_generation(
                repo_root,
                vault,
                &trace_binding,
                active.risk,
                active.authority_generation.as_deref(),
            )?,
            None => latest_trace_score_for_operation_with_risk_and_generation(
                repo_root,
                &trace_binding,
                active.risk,
                active.authority_generation.as_deref(),
            )?,
        };
        match trace {
            Some(trace) if trace.passed && trace.achieved >= required_tier(active.risk) => {}
            _ => issues.push("passing trace is missing".to_string()),
        }
    } else {
        issues.push("passing trace is missing".to_string());
    }
    Ok(issues)
}

fn same_operation_binding(left: &ReceiptContext, right: &ReceiptContext) -> bool {
    left.task_id == right.task_id
        && left.operation_id == right.operation_id
        && left.adapter == right.adapter
        && left.session_id == right.session_id
        && left.request_id == right.request_id
}

/// Context view from exact canonical plan authority, never CURRENT.
pub fn plan_status_for_identity(
    repo_root: impl AsRef<Path>,
    identity: &LifecycleIdentity,
) -> Result<String> {
    plan_status_for_identity_with_policy(repo_root.as_ref(), identity, false)
}

/// Current-state consumers must not surface a frontmatter-only plan when its
/// indexed ACTIVE authority is missing. The compatibility reader above keeps
/// legacy identified-plan diagnostics available without granting them current
/// authority.
pub(crate) fn indexed_plan_status_for_identity(
    repo_root: impl AsRef<Path>,
    identity: &LifecycleIdentity,
) -> Result<String> {
    plan_status_for_identity_with_policy(repo_root.as_ref(), identity, true)
}

fn plan_status_for_identity_with_policy(
    repo_root: &Path,
    identity: &LifecycleIdentity,
    require_indexed_authority: bool,
) -> Result<String> {
    let _lock = acquire_project_lock(repo_root)?;
    let binding = PlanOperationBinding::from_identity(identity);
    let active_plan = if require_indexed_authority {
        active_plan_for_indexed_binding(repo_root, &binding)?
    } else {
        active_plan_for_binding(repo_root, &binding)?
    };
    let Some(active) = active_plan else {
        return Ok(
            "# Baron Plan Status\n\n- Active plan: unknown for this operation\n".to_string(),
        );
    };
    active.ensure_authority()?;
    let body = read_text_required(&active.path)?;
    Ok(format!("# Baron Plan Status\n\n- Title: {}\n- Risk: `{}`\n- Status: `{}`\n- Plan: `{}`\n- Task ID: `{}`\n- Operation ID: `{}`\n\n{}",
        active.title, active.risk.as_str(), active.status, normalize(&active.path, repo_root),
        binding.task_id, binding.operation_id, body))
}

pub fn plan_status(repo_root: impl AsRef<Path>) -> Result<String> {
    let repo_root = repo_root.as_ref();
    let path = repo_root.join("docs/baron/plans/CURRENT.md");
    let Some(current) = read_text(&path)? else {
        return Ok("# Baron Plan Status\n\n- Active plan: none\n".to_string());
    };
    let mut output = format!("# Baron Plan Status\n\n{current}");
    let active = active_plan(repo_root)?;
    let authority_issues = active
        .as_ref()
        .map(|plan| plan.authority_issues.clone())
        .unwrap_or_default();
    let linked_completed = active
        .as_ref()
        .is_some_and(|plan| plan.status == "completed");
    let should_report_integrity = !authority_issues.is_empty()
        || linked_completed
        || (active.is_none() && !current.trim().is_empty());
    if should_report_integrity {
        let issues = if authority_issues.is_empty() {
            completion_integrity_issues(repo_root, &current)?
        } else {
            authority_issues
        };
        if issues.is_empty() {
            output.push_str("\n## Completion Integrity\n\n- Completion integrity: `passed`\n");
        } else {
            output.push_str("\n## Completion Integrity\n\n- Completion integrity: `failed`\n");
            for issue in issues {
                output.push_str(&format!("- {issue}\n"));
            }
        }
    } else {
        output.push_str("\n## Completion Integrity\n\n- Completion integrity: `not_applicable`\n");
    }
    Ok(output)
}

fn completion_integrity_issues(repo_root: &Path, current: &str) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    let verification = field(current, "- Verification: ").unwrap_or_default();
    if verification.trim().is_empty() || verification.trim() == "not_run" {
        issues.push("verification evidence is missing".to_string());
    }
    let active = active_plan(repo_root)?;
    match active {
        Some(plan) if plan.path.is_file() => {
            let body = read_text_required(&plan.path)?;
            if plan.linked_status.as_deref() != Some("completed")
                || !body.lines().any(|line| line == "status: completed")
            {
                issues.push("plan file is not marked completed".to_string());
            }
            let plan_verification = body
                .lines()
                .find_map(|line| line.strip_prefix("verification: "))
                .unwrap_or_default();
            if plan_verification.trim().is_empty() || plan_verification.trim() == "not_run" {
                issues.push("plan verification evidence is missing".to_string());
            }
            issues.extend(completion_evidence_status(repo_root, &plan, None)?.issues);
        }
        _ => issues.push("linked plan file is missing".to_string()),
    }
    Ok(issues)
}

fn write_current(repo_root: &Path, vault: &VaultContext, view: CurrentPlanView<'_>) -> Result<()> {
    let task_id = view
        .task_id
        .map(str::to_owned)
        .or_else(|| view.binding.map(|binding| binding.task_id.clone()))
        .map_or_else(
            || {
                task_id_for_task(&vault.project_id, view.title)
                    .map_err(|error| anyhow::anyhow!(error.to_string()))
            },
            Ok,
        )?;
    let operation_identity = view
        .binding
        .map(|binding| {
            format!(
                "- Operation ID: `{}`\n- Adapter: `{}`\n- Session ID: `{}`\n- Request ID: `{}`\n",
                binding.operation_id, binding.adapter, binding.session_id, binding.request_id
            )
        })
        .unwrap_or_default();
    let content = format!(
        "# Current Baron Plan\n\n\
- Title: {}\n\
- Plan: `{}`\n\
- Status: `{}`\n\
- Risk: `{}`\n\
- Task ID: `{}`\n\
{}\
- Verification: {}\n\
- Next action: {}\n\
- Updated: {}\n\n\
## Rules\n\n\
- Silence or shutdown never means completed.\n\
- Completion requires risk-appropriate proof and a passing trace score.\n",
        view.title,
        normalize(view.plan_path, repo_root),
        view.status,
        view.risk.as_str(),
        task_id,
        operation_identity,
        view.verification,
        view.next_action,
        now()
    );
    write(&repo_root.join("docs/baron/plans/CURRENT.md"), &content)?;
    write(&vault.project_root.join("Plans/CURRENT.md"), &content)
}

fn active_plan_index_path(repo_root: &Path) -> PathBuf {
    repo_root
        .join(MANAGED_PLAN_ROOT)
        .join(ACTIVE_PLAN_INDEX_FILE)
}

fn active_plan_vault_index_path(vault: &VaultContext) -> PathBuf {
    vault
        .project_root
        .join("Plans")
        .join(ACTIVE_PLAN_INDEX_FILE)
}

fn read_active_plan_index_entries(repo_root: &Path) -> Result<Vec<ActivePlanIndexEntry>> {
    read_active_plan_index_entries_at(&active_plan_index_path(repo_root))
}

fn read_active_plan_vault_index_entries(vault: &VaultContext) -> Result<Vec<ActivePlanIndexEntry>> {
    read_active_plan_index_entries_at(&active_plan_vault_index_path(vault))
}

fn read_active_plan_index_entries_at(path: &Path) -> Result<Vec<ActivePlanIndexEntry>> {
    let Some(content) = read_text(path)? else {
        return Ok(Vec::new());
    };
    let mut entries = Vec::new();
    for line in content.lines() {
        let Some(json) = line
            .strip_prefix(ACTIVE_PLAN_MARKER)
            .and_then(|value| value.strip_suffix(" -->"))
        else {
            continue;
        };
        let entry: ActivePlanIndexEntry = serde_json::from_str(json)
            .with_context(|| "Baron active plan index contains malformed entry")?;
        entry.binding()?;
        parse_plan_status(&entry.status)?;
        if !is_safe_plan_path(&entry.plan_path) {
            bail!(
                "Baron active plan index contains an unsafe plan path: {}",
                entry.plan_path
            );
        }
        entries.push(entry);
    }
    for (index, entry) in entries.iter().enumerate() {
        let binding = entry.binding()?;
        for previous in entries.iter().take(index) {
            if previous.binding()? == binding {
                bail!(
                    "Baron active plan index contains duplicate operation `{}`",
                    binding.operation_id
                );
            }
            if previous.plan_path == entry.plan_path {
                bail!(
                    "Baron active plan index claims one path for conflicting entries: {}",
                    entry.plan_path
                );
            }
        }
    }
    Ok(entries)
}

fn ensure_active_plan_vault_index_consistent(repo_root: &Path, vault: &VaultContext) -> Result<()> {
    recover_pending_plan_transition(repo_root, vault)?;
    let repo_entries = read_active_plan_index_entries(repo_root)?;
    let vault_entries = read_active_plan_vault_index_entries(vault)?;
    for local in &repo_entries {
        let local_binding = local.binding()?;
        for shared in &vault_entries {
            let shared_binding = shared.binding()?;
            if shared_binding == local_binding {
                if shared.plan_path != local.plan_path
                    || shared.status != local.status
                    || shared.authority_generation != local.authority_generation
                {
                    bail!(
                        "shared Vault ACTIVE entry for operation `{}` conflicts with this checkout",
                        local_binding.operation_id
                    );
                }
            } else if shared.plan_path == local.plan_path {
                bail!(
                    "shared Vault ACTIVE path is already bound to another operation: {}",
                    local.plan_path
                );
            }
        }
    }
    Ok(())
}

fn validate_shared_operation_plan_ownership(
    repo_root: &Path,
    vault: &VaultContext,
    binding: &PlanOperationBinding,
) -> Result<()> {
    let Some(shared) = read_active_plan_vault_index_entries(vault)?
        .into_iter()
        .find(|entry| {
            entry
                .binding()
                .is_ok_and(|entry_binding| entry_binding == *binding)
        })
    else {
        return Ok(());
    };

    if shared.status == "completed" {
        bail!(
            "operation `{}` is already completed; a new lifecycle requires a fresh operation identity",
            binding.operation_id
        );
    }
    if !is_safe_plan_path(&shared.plan_path) {
        bail!(
            "shared Vault ACTIVE entry for operation `{}` contains an unsafe plan path",
            binding.operation_id
        );
    }
    let candidate = repo_root.join(&shared.plan_path);
    if !candidate.is_file() {
        bail!(
            "shared Vault ACTIVE entry for operation `{}` points to a plan path not present in this checkout; refusing to create a competing plan",
            binding.operation_id
        );
    }
    let path = resolve_managed_plan_path(repo_root, &shared.plan_path)?;

    let metadata = load_plan_file_metadata(&path)?;
    if metadata.operation_binding()?.as_ref() != Some(binding)
        || metadata.status != shared.status
        || metadata.authority_generation != shared.authority_generation
    {
        bail!(
            "shared Vault ACTIVE entry for operation `{}` conflicts with its local plan authority",
            binding.operation_id
        );
    }
    Ok(())
}

fn load_active_plan_index(repo_root: &Path) -> Result<Vec<ActivePlanIndexEntry>> {
    let entries = read_active_plan_index_entries(repo_root)?;
    for entry in &entries {
        validate_active_plan_index_entry(repo_root, entry)?;
    }
    Ok(entries)
}

fn validate_active_plan_index_entry(repo_root: &Path, entry: &ActivePlanIndexEntry) -> Result<()> {
    let binding = entry.binding()?;
    let path = resolve_managed_plan_path(repo_root, &entry.plan_path)?;
    let metadata = load_plan_file_metadata(&path).with_context(|| {
        format!(
            "Baron active plan index points to malformed plan: {}",
            entry.plan_path
        )
    })?;
    if metadata.status != entry.status {
        bail!(
            "Baron active plan index status does not match {}",
            entry.plan_path
        );
    }
    if metadata.operation_binding()?.as_ref() != Some(&binding) {
        bail!(
            "Baron active plan index authority identity does not match {}",
            entry.plan_path
        );
    }
    if metadata.authority_generation != entry.authority_generation {
        bail!(
            "Baron active plan index authority generation does not match {}",
            entry.plan_path
        );
    }
    if validate_linked_plan_authority(repo_root, &metadata)?.as_ref() != Some(&binding) {
        bail!(
            "Baron active plan index canonical authority does not match {}",
            entry.plan_path
        );
    }
    Ok(())
}

fn write_active_plan_index(
    repo_root: &Path,
    vault: &VaultContext,
    entries: Vec<ActivePlanIndexEntry>,
    updated_binding: &PlanOperationBinding,
) -> Result<()> {
    let repo_content = serialize_active_plan_index_entries(entries.clone())?;
    let mut vault_entries = read_active_plan_vault_index_entries(vault)?;
    for entry in &entries {
        let binding = entry.binding()?;
        let mut matching_index = None;
        for (index, shared) in vault_entries.iter().enumerate() {
            if shared.binding()? == binding {
                matching_index = Some(index);
                break;
            }
        }
        if let Some(index) = matching_index {
            let shared = &vault_entries[index];
            if shared.plan_path != entry.plan_path {
                bail!(
                    "shared Vault ACTIVE entry for operation `{}` has conflicting plan paths",
                    binding.operation_id
                );
            }
            if &binding == updated_binding {
                vault_entries[index] = entry.clone();
            } else if shared.status != entry.status
                || shared.authority_generation != entry.authority_generation
            {
                bail!(
                    "shared Vault ACTIVE entry for operation `{}` has a stale status",
                    binding.operation_id
                );
            }
        } else {
            if vault_entries
                .iter()
                .any(|shared| shared.plan_path == entry.plan_path)
            {
                bail!(
                    "shared Vault ACTIVE path is already bound to another operation: {}",
                    entry.plan_path
                );
            }
            vault_entries.push(entry.clone());
        }
    }
    let vault_content = serialize_active_plan_index_entries(vault_entries)?;
    write(&active_plan_index_path(repo_root), &repo_content)?;
    write(&active_plan_vault_index_path(vault), &vault_content)
}

fn serialize_active_plan_index_entries(mut entries: Vec<ActivePlanIndexEntry>) -> Result<String> {
    entries.sort_by(|left, right| {
        left.operation_id
            .cmp(&right.operation_id)
            .then_with(|| left.plan_path.cmp(&right.plan_path))
    });
    let mut content = String::from(
        "# Baron Active Plan Index\n\n<!-- CURRENT.md is a presentation pointer; operation entries below are lookup authority. -->\n",
    );
    for entry in entries {
        content.push_str(ACTIVE_PLAN_MARKER);
        content.push_str(&serde_json::to_string(&entry)?);
        content.push_str(" -->\n");
    }
    Ok(content)
}

fn upsert_active_plan_index(
    repo_root: &Path,
    vault: &VaultContext,
    binding: &PlanOperationBinding,
    plan_path: &Path,
    status: &str,
) -> Result<()> {
    let authority_generation = load_plan_file_metadata(plan_path)?.authority_generation;
    let mut entries = Vec::new();
    for entry in read_active_plan_index_entries(repo_root)? {
        let existing = entry.binding()?;
        if existing == *binding {
            // The caller has already validated this exact plan before changing
            // its status. Its row is intentionally replaced after publication,
            // so a stale status in this one entry is not treated as corruption.
            continue;
        }
        validate_active_plan_index_entry(repo_root, &entry)?;
        entries.push(entry);
    }
    entries.push(ActivePlanIndexEntry::from_plan(
        binding,
        repo_root,
        plan_path,
        status,
        authority_generation,
    ));
    write_active_plan_index(repo_root, vault, entries, binding)
}

fn active_plan_for_binding(
    repo_root: &Path,
    binding: &PlanOperationBinding,
) -> Result<Option<ActivePlan>> {
    active_plan_for_binding_with_completion_policy(repo_root, binding, false, true)
}

fn active_plan_for_indexed_binding(
    repo_root: &Path,
    binding: &PlanOperationBinding,
) -> Result<Option<ActivePlan>> {
    active_plan_for_binding_with_completion_policy(repo_root, binding, false, false)
}

fn active_plan_for_new_operation(
    repo_root: &Path,
    binding: &PlanOperationBinding,
) -> Result<Option<ActivePlan>> {
    active_plan_for_binding_with_completion_policy(repo_root, binding, true, true)
}

fn active_plan_for_binding_with_completion_policy(
    repo_root: &Path,
    binding: &PlanOperationBinding,
    reject_completed: bool,
    allow_frontmatter_fallback: bool,
) -> Result<Option<ActivePlan>> {
    let mut matches = Vec::new();
    for entry in load_active_plan_index(repo_root)? {
        if entry.binding()? != *binding {
            continue;
        }
        if entry.status == "completed" {
            if reject_completed {
                bail!(
                    "operation `{}` is already completed; a new lifecycle requires a fresh operation identity",
                    binding.operation_id
                );
            }
            continue;
        }
        if !is_active_plan_status(&entry.status) {
            continue;
        }
        let path = resolve_managed_plan_path(repo_root, &entry.plan_path)?;
        let mut active = load_identified_active_plan(repo_root, &path, binding)?;
        if active.status != entry.status
            || active.authority_generation != entry.authority_generation
        {
            bail!(
                "Baron active plan index status or authority generation does not match {}",
                entry.plan_path
            );
        }
        active.authority_indexed = true;
        matches.push(active);
    }
    if matches.len() > 1 {
        bail!(
            "Baron active plan index contains multiple active paths for operation `{}`",
            binding.operation_id
        );
    }
    if let Some(active) = matches.pop() {
        return Ok(Some(active));
    }

    if !allow_frontmatter_fallback {
        return Ok(None);
    }

    // Existing identified plans created before ACTIVE.md was introduced remain
    // readable. Discover one by validated Baron frontmatter, then the caller
    // registers the exact path under the same project lock before publishing.
    find_identified_active_plan(repo_root, binding, reject_completed)
}

fn load_identified_active_plan(
    repo_root: &Path,
    path: &Path,
    expected: &PlanOperationBinding,
) -> Result<ActivePlan> {
    let metadata = load_plan_file_metadata(path)?;
    let actual = metadata
        .operation_binding()?
        .context("identified active plan is missing its operation binding")?;
    if actual != *expected {
        bail!("identified active plan operation binding does not match lookup");
    }
    let validated = validate_linked_plan_authority(repo_root, &metadata)?;
    if validated.as_ref() != Some(expected) {
        bail!("identified active plan failed canonical operation validation");
    }
    Ok(ActivePlan {
        title: metadata.title,
        path: path.to_path_buf(),
        status: metadata.status.clone(),
        risk: metadata.risk,
        task_id: metadata.task_id,
        binding: Some(actual),
        linked_status: Some(metadata.status),
        authority_issues: Vec::new(),
        authority_generation: metadata.authority_generation,
        authority_indexed: false,
    })
}

fn find_identified_active_plan(
    repo_root: &Path,
    expected: &PlanOperationBinding,
    reject_completed: bool,
) -> Result<Option<ActivePlan>> {
    let mut paths = Vec::new();
    collect_managed_plan_files(&repo_root.join(MANAGED_PLAN_ROOT), &mut paths)?;
    let mut matches = Vec::new();
    for path in paths {
        let content = read_text_required(&path)?;
        if !claims_baron_plan(&content) {
            continue;
        }
        let metadata = load_plan_file_metadata(&path)
            .with_context(|| format!("managed Baron plan is malformed: {}", path.display()))?;
        if reject_completed
            && metadata.status == "completed"
            && metadata.operation_binding()?.as_ref() == Some(expected)
        {
            load_identified_active_plan(repo_root, &path, expected)?;
            bail!(
                "operation `{}` is already completed; a new lifecycle requires a fresh operation identity",
                expected.operation_id
            );
        }
        if !is_active_plan_status(&metadata.status) {
            continue;
        }
        let Some(binding) = metadata.operation_binding()? else {
            continue;
        };
        if binding == *expected {
            matches.push(load_identified_active_plan(repo_root, &path, expected)?);
        }
    }
    if matches.len() > 1 {
        bail!(
            "multiple active Baron plans match operation `{}`",
            expected.operation_id
        );
    }
    Ok(matches.pop())
}

fn collect_managed_plan_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !root.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_managed_plan_files(&path, files)?;
        } else if file_type.is_file()
            && path.extension().is_some_and(|extension| extension == "md")
            && !matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("CURRENT.md" | "INDEX.md" | "ACTIVE.md")
            )
        {
            files.push(path);
        }
    }
    Ok(())
}

fn claims_baron_plan(content: &str) -> bool {
    let mut lines = content.lines();
    if lines.next() != Some("---") {
        return false;
    }
    for line in lines {
        if line == "---" {
            return false;
        }
        if line
            .strip_prefix("type:")
            .is_some_and(|value| value.trim() == "baron-plan")
        {
            return true;
        }
    }
    false
}

fn active_plan(repo_root: &Path) -> Result<Option<ActivePlan>> {
    let current_path = repo_root.join("docs/baron/plans/CURRENT.md");
    let Some(content) = read_text(&current_path)? else {
        return Ok(None);
    };
    let mut authority_issues = Vec::new();
    let current_title = field(&content, "- Title: ")
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    let path = match field(&content, "- Plan: `")
        .and_then(|value| value.strip_suffix('`').map(str::to_string))
    {
        Some(value) => match resolve_managed_plan_path(repo_root, &value) {
            Ok(path) => Some(path),
            Err(error) => {
                authority_issues.push(format!("CURRENT managed plan path is invalid: {error}"));
                // Keep a concrete ActivePlan so every caller reports a failed
                // authority state (including reconcile and Stop) rather than
                // treating an invalid pointer as an absent plan. Mutation is
                // still blocked by `authority_issues` before this path is used.
                Some(current_path.clone())
            }
        },
        None => None,
    };
    let current_status = match current_backtick_field(&content, "- Status: `") {
        Some(value) => match parse_plan_status(&value) {
            Ok(status) => status,
            Err(error) => {
                authority_issues.push(format!("CURRENT status is invalid: {error}"));
                value
            }
        },
        None => {
            authority_issues.push("CURRENT status is missing".to_string());
            "unknown".to_string()
        }
    };
    let mut status = current_status.clone();
    let risk = match current_backtick_field(&content, "- Risk: `") {
        Some(value) => match parse_risk_lane(&value) {
            Ok(risk) => risk,
            Err(error) => {
                authority_issues.push(format!("CURRENT risk is invalid: {error}"));
                RiskLane::Medium
            }
        },
        None => {
            authority_issues.push("CURRENT risk is missing".to_string());
            RiskLane::Medium
        }
    };
    let task_id = current_backtick_field(&content, "- Task ID: `");
    let operation_id = current_backtick_field(&content, "- Operation ID: `");
    let adapter = current_backtick_field(&content, "- Adapter: `");
    let session_id = current_backtick_field(&content, "- Session ID: `");
    let request_id = current_backtick_field(&content, "- Request ID: `");
    let (current_binding, binding_issues) = current_operation_binding(
        task_id.clone(),
        operation_id,
        adapter,
        session_id,
        request_id,
    );
    authority_issues.extend(binding_issues);

    Ok(path.map(|path| {
        let mut title = current_title;
        let mut canonical_risk = risk;
        let mut linked_task_id = task_id.clone();
        let mut binding = current_binding;
        let mut linked_status = None;
        let mut authority_generation = None;
        match load_plan_file_metadata(&path) {
            Ok(metadata) => {
                authority_generation = metadata.authority_generation.clone();
                linked_status = Some(metadata.status.clone());
                let linked_binding = match metadata.operation_binding() {
                    Ok(linked_binding) => linked_binding,
                    Err(error) => {
                        authority_issues.push(error.to_string());
                        None
                    }
                };
                compare_current_with_linked_plan(
                    CurrentPlanProjection {
                        title: &title,
                        risk: canonical_risk,
                        status: &status,
                        task_id: task_id.as_deref(),
                        binding: binding.as_ref(),
                    },
                    &metadata,
                    linked_binding.as_ref(),
                    &mut authority_issues,
                );
                if let Err(error) = validate_linked_plan_authority(repo_root, &metadata) {
                    authority_issues.push(format!("linked plan authority is invalid: {error}"));
                }
                title = metadata.title.clone();
                status = metadata.status.clone();
                canonical_risk = metadata.risk;
                linked_task_id = Some(metadata.task_id.clone());
                binding = linked_binding;
            }
            Err(error) => {
                authority_issues.push(format!("linked plan metadata is unavailable: {error}"));
            }
        }
        ActivePlan {
            title,
            path,
            status,
            risk: canonical_risk,
            task_id: linked_task_id.unwrap_or_default(),
            binding,
            linked_status,
            authority_issues,
            authority_generation,
            authority_indexed: false,
        }
    }))
}

fn require_active_plan_for_binding(
    repo_root: &Path,
    binding: &PlanOperationBinding,
) -> Result<ActivePlan> {
    indexed_active_plan_authority_for_binding(repo_root, binding)?
        .context("identified mutation requires validated ACTIVE authority; explicitly start/resume the legacy plan first")?;
    let active = active_plan_for_binding(repo_root, binding)?.with_context(|| {
        format!(
            "No active Baron plan for operation `{}`.",
            binding.operation_id
        )
    })?;
    active.ensure_authority()?;
    Ok(active)
}

fn require_legacy_active_plan(repo_root: &Path) -> Result<ActivePlan> {
    let active = resolve_legacy_active_plan(repo_root)?
        .context("No active Baron plan. Run `baron plan start \"<title>\"`.")?;
    if let Some(binding) = active.binding.as_ref() {
        indexed_active_plan_authority_for_binding(repo_root, binding)?
            .context("identified mutation requires validated ACTIVE authority; explicitly start/resume the legacy plan first")?;
    }
    Ok(active)
}

fn resolve_legacy_active_plan(repo_root: &Path) -> Result<Option<ActivePlan>> {
    // Count managed active plans before consulting CURRENT. For an identified
    // operation, the exact ACTIVE/frontmatter binding is authority; a stale,
    // missing, or malformed presentation pointer cannot veto the sole plan.
    let managed_active = discover_active_managed_plans(repo_root)?;
    if managed_active.len() > 1 {
        bail!(
            "legacy plan mutation is ambiguous: {} managed active plans exist; provide an exact operation identity",
            managed_active.len()
        );
    }
    if let Some(candidate) = managed_active.into_iter().next() {
        if candidate.binding.is_some() {
            return Ok(Some(candidate));
        }

        // Unbound legacy plans still require the compatibility pointer to
        // identify the exact path because they have no lifecycle identity.
        let current = active_plan(repo_root)?;
        let Some(current) = current else {
            return Ok(None);
        };
        current.ensure_authority()?;
        if current.path != candidate.path {
            bail!("legacy plan mutation is ambiguous: CURRENT.md does not identify the sole unbound managed plan");
        }
        return Ok(Some(candidate));
    }

    // With no managed active operation, retain historical CURRENT behavior for
    // out-of-tree and pre-identity legacy plans.
    let current = active_plan(repo_root)?;
    let Some(current) = current else {
        return Ok(None);
    };
    current.ensure_authority()?;
    if !is_active_plan_status(&current.status) {
        return Ok(None);
    }
    Ok(Some(current))
}

fn discover_active_managed_plans(repo_root: &Path) -> Result<Vec<ActivePlan>> {
    // Validate every managed ACTIVE entry before scanning plan files, so a
    // malformed indexed authority cannot be hidden by the CURRENT projection.
    let _ = load_active_plan_index(repo_root)?;
    let mut paths = Vec::new();
    collect_managed_plan_files(&repo_root.join(MANAGED_PLAN_ROOT), &mut paths)?;
    let mut matches = Vec::new();
    for path in paths {
        let content = read_text_required(&path)?;
        if !claims_baron_plan(&content) {
            continue;
        }
        let metadata = load_plan_file_metadata(&path)
            .with_context(|| format!("managed Baron plan is malformed: {}", path.display()))?;
        if !is_active_plan_status(&metadata.status) {
            continue;
        }
        let binding = metadata.operation_binding()?;
        let active = if let Some(binding) = binding {
            load_identified_active_plan(repo_root, &path, &binding)?
        } else {
            validate_linked_plan_authority(repo_root, &metadata)?;
            ActivePlan {
                title: metadata.title,
                path: path.clone(),
                status: metadata.status.clone(),
                risk: metadata.risk,
                task_id: metadata.task_id,
                binding: None,
                linked_status: Some(metadata.status),
                authority_issues: Vec::new(),
                authority_generation: metadata.authority_generation,
                authority_indexed: false,
            }
        };
        if matches.iter().any(|existing: &ActivePlan| {
            existing.path == active.path
                || (active.binding.is_some() && existing.binding == active.binding)
        }) {
            bail!(
                "multiple active Baron plans claim one operation or path: {}",
                path.display()
            );
        }
        matches.push(active);
    }
    Ok(matches)
}

fn sole_active_managed_plan(repo_root: &Path) -> Result<Option<ActivePlan>> {
    let mut active_plans = discover_active_managed_plans(repo_root)?;
    if active_plans.len() > 1 {
        bail!(
            "operation selection is ambiguous: {} managed active plans exist; provide an exact operation identity",
            active_plans.len()
        );
    }
    Ok(active_plans.pop())
}

fn discover_identified_active_plans(repo_root: &Path) -> Result<Vec<ActivePlan>> {
    // Validate every managed ACTIVE entry before scanning legacy pre-ACTIVE
    // plans. This prevents an unrelated malformed entry from being silently
    // ignored by a legacy mutation.
    let _ = load_active_plan_index(repo_root)?;
    let mut paths = Vec::new();
    collect_managed_plan_files(&repo_root.join(MANAGED_PLAN_ROOT), &mut paths)?;
    let mut matches = Vec::new();
    for path in paths {
        let content = read_text_required(&path)?;
        if !claims_baron_plan(&content) {
            continue;
        }
        let metadata = load_plan_file_metadata(&path)
            .with_context(|| format!("managed Baron plan is malformed: {}", path.display()))?;
        if !is_active_plan_status(&metadata.status) {
            continue;
        }
        let Some(binding) = metadata.operation_binding()? else {
            continue;
        };
        let active = load_identified_active_plan(repo_root, &path, &binding)?;
        if matches.iter().any(|existing: &ActivePlan| {
            existing.path == active.path || existing.binding == active.binding
        }) {
            bail!(
                "multiple active Baron plans claim one operation or path: {}",
                path.display()
            );
        }
        matches.push(active);
    }
    Ok(matches)
}

fn current_backtick_field(content: &str, prefix: &str) -> Option<String> {
    field(content, prefix).and_then(|value| {
        value
            .strip_suffix('`')
            .map(|value| value.trim().to_string())
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlanFileMetadata {
    title: String,
    status: String,
    risk: RiskLane,
    task_id: String,
    operation_id: Option<String>,
    adapter: Option<String>,
    session_id: Option<String>,
    request_id: Option<String>,
    authority_generation: Option<String>,
}

impl PlanFileMetadata {
    fn operation_binding(&self) -> Result<Option<PlanOperationBinding>> {
        match (
            self.operation_id.as_ref(),
            self.adapter.as_ref(),
            self.session_id.as_ref(),
            self.request_id.as_ref(),
        ) {
            (None, None, None, None) => Ok(None),
            (Some(operation_id), Some(adapter), Some(session_id), Some(request_id))
                if !operation_id.trim().is_empty()
                    && !adapter.trim().is_empty()
                    && !session_id.trim().is_empty()
                    && !request_id.trim().is_empty() =>
            {
                adapter
                    .parse::<SupportedAdapter>()
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
                Ok(Some(PlanOperationBinding {
                    task_id: self.task_id.clone(),
                    operation_id: operation_id.clone(),
                    adapter: adapter.clone(),
                    session_id: session_id.clone(),
                    request_id: request_id.clone(),
                }))
            }
            _ => bail!("linked plan operation metadata is incomplete"),
        }
    }
}

fn load_plan_file_metadata(path: &Path) -> Result<PlanFileMetadata> {
    let content = read_text_required(path)?;
    let fields = leading_plan_frontmatter(&content)?;
    let document_type = required_frontmatter_field(&fields, "type")?;
    if document_type != "baron-plan" {
        bail!("linked plan document type must be `baron-plan`");
    }
    let title = required_frontmatter_field(&fields, "title")?;
    let status = parse_plan_status(&required_frontmatter_field(&fields, "status")?)?;
    let risk = parse_risk_lane(&required_frontmatter_field(&fields, "risk")?)?;
    let task_id = required_frontmatter_field(&fields, "task_id")?;
    if task_id.is_empty() {
        bail!("linked plan task_id is empty");
    }
    let authority_generation = optional_frontmatter_field(&fields, "authority_generation")?;
    if authority_generation
        .as_deref()
        .is_some_and(|generation| generation.trim().is_empty())
    {
        bail!("linked plan authority generation is empty");
    }
    Ok(PlanFileMetadata {
        title,
        status,
        risk,
        task_id,
        operation_id: optional_frontmatter_field(&fields, "operation_id")?,
        adapter: optional_frontmatter_field(&fields, "adapter")?,
        session_id: optional_frontmatter_field(&fields, "session_id")?,
        request_id: optional_frontmatter_field(&fields, "request_id")?,
        authority_generation,
    })
}

fn leading_plan_frontmatter(content: &str) -> Result<Vec<(&str, &str)>> {
    let mut lines = content.lines();
    if lines.next() != Some("---") {
        bail!("linked plan frontmatter is missing or not leading");
    }

    let mut fields = Vec::new();
    let mut closed = false;
    for line in lines {
        if line == "---" {
            closed = true;
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let (key, value) = line
            .split_once(':')
            .with_context(|| format!("linked plan frontmatter line is malformed: {line}"))?;
        let key = key.trim();
        if key.is_empty() {
            bail!("linked plan frontmatter contains an empty field name");
        }
        fields.push((key, value.trim()));
    }
    if !closed {
        bail!("linked plan frontmatter is not closed");
    }
    Ok(fields)
}

fn required_frontmatter_field(fields: &[(&str, &str)], key: &str) -> Result<String> {
    optional_frontmatter_field(fields, key)?
        .filter(|value| !value.is_empty())
        .with_context(|| format!("linked plan metadata is missing `{key}: `"))
}

fn optional_frontmatter_field(fields: &[(&str, &str)], key: &str) -> Result<Option<String>> {
    let values = fields
        .iter()
        .filter_map(|(field, value)| (*field == key).then_some(*value))
        .collect::<Vec<_>>();
    if values.len() > 1 {
        bail!("linked plan authority field `{key}` is duplicated");
    }
    Ok(values.first().map(|value| (*value).to_string()))
}

fn validate_linked_plan_authority(
    repo_root: &Path,
    metadata: &PlanFileMetadata,
) -> Result<Option<PlanOperationBinding>> {
    let expected_risk = classify_risk(&metadata.title);
    if metadata.risk != expected_risk {
        bail!("linked plan risk does not match canonical classifier");
    }

    let project_id = canonical_project_id(repo_root)?;
    let expected_task_id = task_id_for_task(&project_id, &metadata.title)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    let binding = metadata.operation_binding()?;
    let Some(binding) = binding else {
        if metadata.task_id != expected_task_id
            && metadata.task_id != format!("task-{}", slugify(&metadata.title))
        {
            bail!("linked plan task_id does not match canonical task or legacy format");
        }
        return Ok(None);
    };

    if metadata.task_id != expected_task_id {
        bail!("linked plan task_id does not match canonical task");
    }

    let adapter = SupportedAdapter::parse(&binding.adapter)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let identity = LifecycleIdentity::from_parts_checked(
        project_id,
        binding.task_id.clone(),
        binding.operation_id.clone(),
        adapter,
        binding.session_id.clone(),
        binding.request_id.clone(),
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    identity
        .validate_task(&metadata.title)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    Ok(Some(binding))
}

fn parse_risk_lane(value: &str) -> Result<RiskLane> {
    match value.trim() {
        "low" => Ok(RiskLane::Low),
        "medium" => Ok(RiskLane::Medium),
        "high" => Ok(RiskLane::High),
        other => bail!("unsupported risk lane `{other}`"),
    }
}

fn parse_plan_status(value: &str) -> Result<String> {
    match value.trim() {
        "in_progress" | "interrupted" | "needs_correction" | "blocked" | "completed" => {
            Ok(value.trim().to_string())
        }
        other => bail!("unsupported plan status `{other}`"),
    }
}

fn is_active_plan_status(value: &str) -> bool {
    matches!(
        value,
        "in_progress" | "interrupted" | "needs_correction" | "blocked"
    )
}

fn current_operation_binding(
    task_id: Option<String>,
    operation_id: Option<String>,
    adapter: Option<String>,
    session_id: Option<String>,
    request_id: Option<String>,
) -> (Option<PlanOperationBinding>, Vec<String>) {
    let fields = [
        operation_id.as_ref(),
        adapter.as_ref(),
        session_id.as_ref(),
        request_id.as_ref(),
    ];
    if fields.iter().all(Option::is_none) {
        return (None, Vec::new());
    }
    if task_id.is_none()
        || fields.iter().any(Option::is_none)
        || [
            operation_id.as_deref(),
            adapter.as_deref(),
            session_id.as_deref(),
            request_id.as_deref(),
        ]
        .iter()
        .any(|value| value.is_some_and(str::is_empty))
    {
        return (
            None,
            vec!["CURRENT operation metadata is incomplete".to_string()],
        );
    }
    let adapter = adapter.expect("checked above");
    if let Err(error) = adapter.parse::<SupportedAdapter>() {
        return (None, vec![format!("CURRENT adapter is invalid: {error}")]);
    }
    (
        Some(PlanOperationBinding {
            task_id: task_id.expect("checked above"),
            operation_id: operation_id.expect("checked above"),
            adapter,
            session_id: session_id.expect("checked above"),
            request_id: request_id.expect("checked above"),
        }),
        Vec::new(),
    )
}

struct CurrentPlanProjection<'a> {
    title: &'a str,
    status: &'a str,
    risk: RiskLane,
    task_id: Option<&'a str>,
    binding: Option<&'a PlanOperationBinding>,
}

fn compare_current_with_linked_plan(
    current: CurrentPlanProjection<'_>,
    linked: &PlanFileMetadata,
    linked_binding: Option<&PlanOperationBinding>,
    issues: &mut Vec<String>,
) {
    if current.title != linked.title {
        issues.push("CURRENT title does not match linked plan title".to_string());
    }
    if current.risk != linked.risk {
        issues.push("CURRENT risk does not match linked plan risk".to_string());
    }
    if current.status != linked.status {
        issues.push("CURRENT status does not match linked plan status".to_string());
    }
    match current.task_id {
        Some(current_task_id) if current_task_id == linked.task_id => {}
        Some(_) => issues.push("CURRENT task_id does not match linked plan task_id".to_string()),
        None => issues.push("CURRENT task_id is missing from linked plan authority".to_string()),
    }
    match (current.binding, linked_binding) {
        (None, None) => {}
        (Some(current), Some(linked)) => {
            if current.operation_id != linked.operation_id {
                issues.push("CURRENT operation_id does not match linked plan".to_string());
            }
            if current.adapter != linked.adapter {
                issues.push("CURRENT adapter does not match linked plan".to_string());
            }
            if current.session_id != linked.session_id {
                issues.push("CURRENT session_id does not match linked plan".to_string());
            }
            if current.request_id != linked.request_id {
                issues.push("CURRENT request_id does not match linked plan".to_string());
            }
        }
        _ => issues.push("CURRENT and linked plan operation binding state differ".to_string()),
    }
}

fn field(content: &str, prefix: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .map(str::to_string)
}

fn single_line(value: &str) -> String {
    value.replace(['\r', '\n'], " ").trim().to_string()
}

fn append_progress(path: &Path, note: &str) -> Result<()> {
    let mut content = read_text_required(path)?;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&format!("- {} - {}\n", now(), note));
    write(path, &content)
}

fn mirror_plan(repo_root: &Path, vault: &VaultContext, plan_path: &Path) -> Result<()> {
    let content = read_text_required(plan_path)?;
    write(&vault_plan_path(repo_root, vault, plan_path)?, &content)
}

fn vault_plan_path(repo_root: &Path, vault: &VaultContext, repo_path: &Path) -> Result<PathBuf> {
    let relative = repo_path
        .strip_prefix(repo_root.join(MANAGED_PLAN_ROOT))
        .context("plan path is outside Baron plan root")?;
    if relative.as_os_str().is_empty()
        || !relative
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        bail!("plan path is not a regular managed Baron plan path");
    }
    Ok(vault.project_root.join("Plans").join(relative))
}

fn plan_content(
    project_id: &str,
    title: &str,
    risk: RiskLane,
    binding: Option<&PlanOperationBinding>,
    authority_generation: Option<&str>,
) -> Result<String> {
    let task_id = binding.map(|binding| binding.task_id.clone()).map_or_else(
        || task_id_for_task(project_id, title).map_err(|error| anyhow::anyhow!(error.to_string())),
        Ok,
    )?;
    let operation_identity = binding
        .map(|binding| {
            format!(
                "operation_id: {}\nadapter: {}\nsession_id: {}\nrequest_id: {}\n",
                binding.operation_id, binding.adapter, binding.session_id, binding.request_id
            )
        })
        .unwrap_or_default();
    let authority_identity = authority_generation
        .map(|generation| format!("authority_generation: {generation}\n"))
        .unwrap_or_default();
    Ok(format!(
        "---\n\
type: baron-plan\n\
title: {title}\n\
status: in_progress\n\
risk: {}\n\
task_id: {task_id}\n\
{operation_identity}\
{authority_identity}\
created: {}\n\
updated: {}\n\
verification: not_run\n\
---\n\n\
# {title}\n\n\
## Goal\n\n{title}\n\n\
## Scope\n\n- Work tied to this task only.\n\n\
## Checklist\n\n\
- [ ] Define the implementation path.\n\
- [ ] Implement the requested change.\n\
- [ ] Record risk-appropriate proof.\n\
- [ ] Record and score the execution trace.\n\n\
## Progress Log\n\n\
- {} - Plan started.\n",
        risk.as_str(),
        today(),
        now(),
        now()
    ))
}

fn required_tier(risk: RiskLane) -> TraceTier {
    match risk {
        RiskLane::Low => TraceTier::Minimal,
        RiskLane::Medium => TraceTier::Standard,
        RiskLane::High => TraceTier::Detailed,
    }
}

fn append_unique(path: &Path, header: &str, item: &str) -> Result<()> {
    let mut content = fs::read_to_string(path).unwrap_or_else(|_| header.to_string());
    if content.contains(item) {
        return Ok(());
    }
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(item);
    content.push('\n');
    write(path, &content)
}

fn update_plan_indexes(
    repo_root: &Path,
    vault: &VaultContext,
    title: &str,
    repo_path: &Path,
    risk: RiskLane,
    status: &str,
) -> Result<()> {
    let vault_path = vault_plan_path(repo_root, vault, repo_path)?;
    replace_plan_index_row(
        &repo_root.join("docs/baron/plans/INDEX.md"),
        title,
        &normalize(repo_path, repo_root),
        risk,
        status,
    )?;
    replace_plan_index_row(
        &vault.project_root.join("Plans/INDEX.md"),
        title,
        &normalize(&vault_path, &vault.project_root),
        risk,
        status,
    )
}

fn replace_plan_index_row(
    path: &Path,
    title: &str,
    relative_path: &str,
    risk: RiskLane,
    status: &str,
) -> Result<()> {
    let row = format!(
        "- [{title}]({relative_path}) - status: `{status}` - risk: `{}`",
        risk.as_str()
    );
    let mut content =
        fs::read_to_string(path).unwrap_or_else(|_| "# Baron Plan Index\n\n".to_string());
    let mut replaced = false;
    let mut lines = content
        .lines()
        .map(|line| {
            if plan_index_link(line) == Some(relative_path) {
                replaced = true;
                row.clone()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>();
    if !replaced {
        lines.push(row);
    }
    content = lines.join("\n");
    content.push('\n');
    write(path, &content)
}

fn plan_index_link(line: &str) -> Option<&str> {
    let start = line.find("](")? + 2;
    let end = line[start..].find(')')? + start;
    Some(&line[start..end])
}

fn write(path: &Path, content: &str) -> Result<()> {
    replace_text(path, content).with_context(|| format!("Could not write {}", path.display()))
}

fn is_safe_plan_path(value: &str) -> bool {
    let path = Path::new(value);
    !value.trim().is_empty()
        && !value.contains('\\')
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn resolve_managed_plan_path(repo_root: &Path, value: &str) -> Result<PathBuf> {
    if !is_safe_plan_path(value) {
        bail!("path is not a safe relative path: {value}");
    }

    let plan_root = repo_root.join(MANAGED_PLAN_ROOT);
    let candidate = repo_root.join(value);
    // The safe-I/O read rejects missing entries, directories, non-regular
    // entries, symlinks, reparse points, and unsafe parent components before
    // the target can be treated as authority.
    read_text_required(&candidate)?;
    let canonical_root = plan_root.canonicalize().with_context(|| {
        format!(
            "could not resolve managed Baron plan root: {}",
            plan_root.display()
        )
    })?;
    let canonical_candidate = candidate.canonicalize().with_context(|| {
        format!(
            "could not resolve managed Baron plan path: {}",
            candidate.display()
        )
    })?;
    if canonical_candidate.strip_prefix(&canonical_root).is_err() {
        bail!("path resolves outside managed Baron plan root: {value}");
    }
    Ok(candidate)
}

fn normalize(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut dash = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            dash = false;
        } else if !dash && !slug.is_empty() {
            slug.push('-');
            dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "plan".to_string()
    } else {
        slug
    }
}

struct ActivePlan {
    title: String,
    path: PathBuf,
    status: String,
    risk: RiskLane,
    task_id: String,
    binding: Option<PlanOperationBinding>,
    linked_status: Option<String>,
    authority_issues: Vec<String>,
    authority_generation: Option<String>,
    authority_indexed: bool,
}

impl ActivePlan {
    fn ensure_authority(&self) -> Result<()> {
        if let Some(issue) = self.authority_issues.first() {
            bail!("Active Baron plan authority mismatch: {issue}");
        }
        Ok(())
    }
}

struct CurrentPlanView<'a> {
    title: &'a str,
    risk: RiskLane,
    status: &'a str,
    plan_path: &'a Path,
    next_action: &'a str,
    verification: &'a str,
    binding: Option<&'a PlanOperationBinding>,
    task_id: Option<&'a str>,
}
