use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};

use crate::capability::{
    evaluate_execution_evidence, evaluate_execution_evidence_for_operation, load_capability_state,
    record_runtime_execution, CapabilityExecutionEvidence,
};
use crate::execution_receipt::{
    load_verified_receipt, receipt_matches_verified_context, ReceiptContext,
    VerifiedExecutionReceipt,
};
use crate::harness::{current_harness_risk, update_current_validation_evidence};
use crate::operation::{OperationContext, SupportedAdapter};
use crate::risk::RiskLane;
use crate::safe_io::{
    acquire_project_lock, append_text, artifact_instance_id, create_new_text, read_text,
    replace_text,
};
use crate::vault::VaultContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofRecord {
    pub id: String,
    pub summary: String,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
    pub capability_gate_passed: bool,
    pub capability_gaps: Vec<String>,
    pub capability_warnings: Vec<String>,
    pub receipt_id: Option<String>,
    pub source_fingerprint: Option<String>,
    pub binding: Option<ReceiptContext>,
}

pub fn record_proof(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    summary: &str,
) -> Result<ProofRecord> {
    record_proof_internal(repo_root, vault, None, summary, &[], None, None)
}

pub fn record_proof_from_receipt(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    receipt_id: &str,
) -> Result<ProofRecord> {
    let _ = (repo_root, vault, receipt_id);
    bail!("explicit proof receipt binding is required; use record_proof_from_receipt_bound")
}

/// Record proof from a receipt whose operation identity is explicitly bound to
/// the proof request.  A generic receipt reference is intentionally rejected so
/// a receipt from another task, adapter, or gate cannot become proof for the
/// current operation.
pub fn record_proof_from_receipt_bound(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    receipt_id: &str,
    binding: &ReceiptContext,
) -> Result<ProofRecord> {
    let repo_root = repo_root.as_ref();
    if binding.gate_kind.trim() != "proof" {
        bail!(
            "proof receipt kind `{}` is not authorized for proof recording",
            binding.gate_kind.trim()
        );
    }
    let receipt = load_verified_receipt(repo_root, receipt_id)?;
    if !receipt_matches_verified_context(&receipt, binding)? {
        bail!(
            "Trusted execution receipt `{}` is stale, failed, mismatched, replayed, or tampered",
            receipt.receipt_id,
        );
    }
    let operation = operation_from_binding(binding)?;
    record_proof_internal(
        repo_root,
        vault,
        Some(&operation),
        &format!(
            "trusted execution receipt {} passed for {} via {}",
            receipt.receipt_id, receipt.capability, receipt.provider
        ),
        &[],
        Some((&receipt, binding)),
        Some(binding),
    )
}

/// Record proof with a complete operation binding but without a trusted
/// execution receipt. This is suitable for Low-risk operation evidence; a
/// Medium/High completion still requires a current receipt-bound proof.
pub fn record_proof_for_operation(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    operation: &OperationContext,
    summary: &str,
) -> Result<ProofRecord> {
    let binding = complete_operation_binding(vault, operation)?
        .context("complete operation identity is required for bound proof recording")?;
    record_proof_internal(
        repo_root,
        vault,
        Some(operation),
        summary,
        &[],
        None,
        Some(&binding),
    )
}

pub fn record_proof_with_capabilities(
    _repo_root: impl AsRef<Path>,
    _vault: &VaultContext,
    _summary: &str,
    _capability_evidence: &[CapabilityExecutionEvidence],
) -> Result<ProofRecord> {
    bail!("explicit adapter identity is required for capability evidence")
}

/// Record proof with the adapter and correlation identity supplied by the
/// current operation. Capability evidence is never evaluated against the
/// project's serialized `active_adapter` value.
pub fn record_proof_with_capabilities_for_operation(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    operation: &OperationContext,
    summary: &str,
    capability_evidence: &[CapabilityExecutionEvidence],
) -> Result<ProofRecord> {
    let binding = complete_operation_binding(vault, operation)?;
    record_proof_internal(
        repo_root,
        vault,
        Some(operation),
        summary,
        capability_evidence,
        None,
        binding.as_ref(),
    )
}

fn record_proof_internal(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    operation: Option<&OperationContext>,
    summary: &str,
    capability_evidence: &[CapabilityExecutionEvidence],
    trusted_receipt: Option<(&VerifiedExecutionReceipt, &ReceiptContext)>,
    binding: Option<&ReceiptContext>,
) -> Result<ProofRecord> {
    let repo_root = repo_root.as_ref();
    let date = Local::now().format("%Y-%m-%d").to_string();
    let _lock = acquire_project_lock(repo_root)?;
    let id = artifact_instance_id(&date)?;
    let repo_path = repo_root
        .join("docs/baron/proofs")
        .join(&date)
        .join(format!("{id}.md"));
    let vault_path = vault
        .project_root
        .join("Proofs")
        .join(&date)
        .join(format!("{id}.md"));
    let capability_gate = if let Some(operation) = operation {
        evaluate_execution_evidence_for_operation(repo_root, operation, capability_evidence)?
    } else if let Some(state) = load_capability_state(repo_root)? {
        let adapter = state
            .adapter
            .supported()
            .context("explicit adapter identity is required for capability evidence")?;
        evaluate_execution_evidence(repo_root, adapter, capability_evidence)?
    } else {
        crate::capability::CapabilityGate {
            passed: true,
            gaps: Vec::new(),
            warnings: Vec::new(),
        }
    };
    let binding = binding
        .or_else(|| trusted_receipt.map(|(_, binding)| binding))
        .cloned();
    let content = render_proof(ProofView {
        id: &id,
        summary,
        capability_evidence,
        gate_passed: capability_gate.passed,
        gaps: &capability_gate.gaps,
        warnings: &capability_gate.warnings,
        operation,
        binding: binding.as_ref(),
        trusted_receipt: trusted_receipt.map(|(receipt, _)| receipt),
    });
    preflight_publication_paths(repo_root, vault, &repo_path, &vault_path)?;
    record_runtime_execution(repo_root, capability_evidence)?;
    create_new_text(&vault_path, &content)?;
    create_new_text(&repo_path, &content)?;
    append(
        &repo_root.join("docs/baron/proofs/INDEX.md"),
        "# Baron Proof Index\n\n",
        &format!("- `{id}` - {}", summary.trim()),
    )?;
    append(
        &vault.project_root.join("Proofs/INDEX.md"),
        "# Baron Proof Index\n\n",
        &format!("- `{id}` - {}", summary.trim()),
    )?;
    let verified =
        proof_satisfies_risk(summary, current_harness_risk(repo_root)) && capability_gate.passed;
    update_current_validation_evidence(repo_root, vault, summary.trim(), verified)?;
    Ok(ProofRecord {
        id,
        summary: summary.trim().to_string(),
        repo_path,
        vault_path,
        capability_gate_passed: capability_gate.passed,
        capability_gaps: capability_gate.gaps,
        capability_warnings: capability_gate.warnings,
        receipt_id: trusted_receipt.map(|(receipt, _)| receipt.receipt_id.clone()),
        source_fingerprint: trusted_receipt.map(|(receipt, _)| receipt.source_fingerprint.clone()),
        binding,
    })
}

pub fn proof_status(repo_root: impl AsRef<Path>) -> Result<String> {
    match latest_proof(repo_root.as_ref())? {
        Some(proof) => Ok(format!(
            "# Baron Proof Status\n\n- Latest proof: `{}`\n- Evidence: {}\n",
            proof.id, proof.summary
        )),
        None => Ok("# Baron Proof Status\n\n- Latest proof: none\n".to_string()),
    }
}

pub fn latest_proof(repo_root: &Path) -> Result<Option<ProofRecord>> {
    let root = repo_root.join("docs/baron/proofs");
    let Some(path) = latest_markdown(&root)? else {
        return Ok(None);
    };
    if path.file_name().and_then(|value| value.to_str()) == Some("INDEX.md") {
        return Ok(None);
    }
    Ok(Some(parse_proof(&path)?))
}

/// Find a proof by its exact persisted ID. This is an authority selector;
/// unlike [`latest_proof`], it never falls back to a repository-global newest
/// artifact.
pub fn proof_by_id(repo_root: &Path, proof_id: &str) -> Result<Option<ProofRecord>> {
    let proof_id = proof_id.trim();
    if proof_id.is_empty() {
        return Ok(None);
    }
    for path in proof_paths(repo_root)? {
        if path.file_stem().and_then(|value| value.to_str()) == Some(proof_id) {
            return Ok(Some(parse_proof(&path)?));
        }
        let proof = parse_proof(&path)?;
        if proof.id == proof_id {
            return Ok(Some(proof));
        }
    }
    Ok(None)
}

/// Find the newest proof that carries the exact operation binding. Unbound
/// and partially bound legacy records are intentionally excluded.
pub fn proof_for_operation(
    repo_root: &Path,
    expected: &ReceiptContext,
) -> Result<Option<ProofRecord>> {
    expected.validate()?;
    let mut paths = proof_paths(repo_root)?;
    paths.sort();
    for path in paths.into_iter().rev() {
        let proof = parse_proof(&path)?;
        let Some(binding) = proof.binding.as_ref() else {
            continue;
        };
        if operation_binding_matches(binding, expected) && binding.gate_kind.trim() == "proof" {
            return Ok(Some(proof));
        }
    }
    Ok(None)
}

/// Return the complete operation binding persisted in a proof. A legacy
/// proof returns `None` and remains diagnostic-only.
pub fn proof_operation_binding(proof: &ProofRecord) -> Option<ReceiptContext> {
    proof.binding.clone()
}

struct ProofView<'a> {
    id: &'a str,
    summary: &'a str,
    capability_evidence: &'a [CapabilityExecutionEvidence],
    gate_passed: bool,
    gaps: &'a [String],
    warnings: &'a [String],
    operation: Option<&'a OperationContext>,
    binding: Option<&'a ReceiptContext>,
    trusted_receipt: Option<&'a VerifiedExecutionReceipt>,
}

fn render_proof(view: ProofView<'_>) -> String {
    let operation_identity = if let Some(binding) = view.binding {
        format!(
            "- Receipt ID: `{}`\n- Task ID: `{}`\n- Operation ID: `{}`\n- Adapter: `{}`\n- Session ID: `{}`\n- Request ID: `{}`\n- Gate kind: `{}`\n- Source fingerprint: `{}`\n",
            view.trusted_receipt
                .map(|receipt| receipt.receipt_id.as_str())
                .unwrap_or("none"),
            binding.task_id.trim(),
            binding.operation_id.trim(),
            binding.adapter.trim(),
            binding.session_id.trim(),
            binding.request_id.trim(),
            binding.gate_kind.trim(),
            view.trusted_receipt
                .map(|receipt| receipt.source_fingerprint.as_str())
                .unwrap_or("none"),
        )
    } else {
        view.operation
            .map(|operation| {
                format!(
                    "- Adapter: `{}`\n- Session ID: `{}`\n- Request ID: `{}`\n",
                    operation.adapter.as_str(),
                    operation.session_id.as_deref().unwrap_or("none"),
                    operation.request_id.as_deref().unwrap_or("none")
                )
            })
            .unwrap_or_default()
    };
    let mut content = format!(
        "# Baron Proof\n\n- Proof ID: `{}`\n- Recorded: {}\n- Capability gate: `{}`\n{}\n## Evidence\n\n{}\n\n## Capability Execution Evidence\n\n",
        view.id,
        now(),
        if view.gate_passed { "passed" } else { "failed" },
        operation_identity,
        view.summary.trim()
    );
    if view.capability_evidence.is_empty() {
        content.push_str("- none recorded\n");
    } else {
        for evidence in view.capability_evidence {
            content.push_str(&format!(
                "- `{}` via `{}` - {}\n",
                evidence.capability.trim(),
                evidence.provider.trim(),
                evidence.summary.trim()
            ));
        }
    }
    content.push_str("\n## Capability Gaps\n\n");
    push_bullets(&mut content, view.gaps);
    content.push_str("\n## Capability Warnings\n\n");
    push_bullets(&mut content, view.warnings);
    if let Some(receipt) = view.trusted_receipt {
        content.push_str(&format!(
            "\n## Trusted Execution Receipt\n\n- Receipt ID: `{}`\n- Source: Baron-owned execution runner\n- Task ID: `{}`\n- Operation ID: `{}`\n- Adapter: `{}`\n- Session ID: `{}`\n- Request ID: `{}`\n- Gate kind: `{}`\n- Source fingerprint: `{}`\n",
            receipt.receipt_id,
            receipt.task_id.as_deref().unwrap_or("none"),
            receipt.operation_id.as_deref().unwrap_or("none"),
            receipt.adapter.as_deref().unwrap_or("none"),
            receipt.session_id.as_deref().unwrap_or("none"),
            receipt.request_id.as_deref().unwrap_or("none"),
            receipt.gate_kind.as_deref().unwrap_or("none"),
            receipt.source_fingerprint,
        ));
    }
    content
}

fn push_bullets(content: &mut String, values: &[String]) {
    if values.is_empty() {
        content.push_str("- none\n");
    } else {
        for value in values {
            content.push_str(&format!("- {value}\n"));
        }
    }
}

fn section_body(content: &str, heading: &str) -> String {
    content
        .split(heading)
        .nth(1)
        .and_then(|value| value.split("\n## ").next())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn bullet_section(content: &str, heading: &str) -> Vec<String> {
    section_body(content, heading)
        .lines()
        .filter_map(|line| line.strip_prefix("- "))
        .filter(|value| *value != "none")
        .map(str::to_string)
        .collect()
}

pub fn proof_satisfies_risk(summary: &str, risk: RiskLane) -> bool {
    let lower = summary.to_lowercase();
    if lower.trim().is_empty() {
        return false;
    }
    if risk == RiskLane::Low {
        return true;
    }
    let verification = ["passed", "verified", "test", "build", "smoke"]
        .iter()
        .any(|term| lower.contains(term));
    if !verification {
        return false;
    }
    if risk == RiskLane::Medium {
        return true;
    }
    [
        "security",
        "authorization",
        "permission",
        "tenant",
        "rls",
        "migration",
        "data impact",
        "payment",
        "upload",
    ]
    .iter()
    .any(|term| lower.contains(term))
}

fn latest_markdown(root: &Path) -> Result<Option<PathBuf>> {
    if !root.exists() {
        return Ok(None);
    }
    let mut files = Vec::new();
    collect_markdown(root, &mut files)?;
    files.retain(|path| path.file_name().and_then(|value| value.to_str()) != Some("INDEX.md"));
    files.sort();
    Ok(files.pop())
}

fn collect_markdown(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_markdown(&path, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
            files.push(path);
        }
    }
    Ok(())
}

fn append(path: &Path, header: &str, item: &str) -> Result<()> {
    let content = match read_text(path)? {
        Some(content) => content,
        None => {
            replace_text(path, header)?;
            header.to_string()
        }
    };
    let separator = if content.is_empty() || content.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    append_text(path, &format!("{separator}{item}\n"))
}

/// Returns whether a proof points at a Baron-owned receipt that is still valid
/// for the current project source and exact proof operation. Free-form proof
/// remains readable for migration and low-risk reporting, but it is not
/// completion evidence for a medium/high-risk plan.
pub fn proof_has_current_receipt(repo_root: &Path, proof: &ProofRecord) -> Result<bool> {
    let Some((receipt_id, binding)) = proof_receipt_context(proof)? else {
        return Ok(false);
    };
    let Ok(receipt) = load_verified_receipt(repo_root, &receipt_id) else {
        return Ok(false);
    };
    receipt_matches_verified_context(&receipt, &binding)
}

/// Read the explicit receipt binding recorded in a proof.  A proof without a
/// complete binding remains readable diagnostic history, but cannot authorize
/// a medium/high-risk completion or provide a task scope for quality gates.
pub fn proof_receipt_context(proof: &ProofRecord) -> Result<Option<(String, ReceiptContext)>> {
    let content = fs::read_to_string(&proof.repo_path)?;
    let Some(receipt_id) = parse_receipt_id(&content) else {
        return Ok(None);
    };
    let binding =
        parse_operation_binding(&content)?.context("proof receipt binding fields are missing")?;
    Ok(Some((receipt_id, binding)))
}

fn parse_proof(path: &Path) -> Result<ProofRecord> {
    let content = fs::read_to_string(path)?;
    let id = content
        .lines()
        .find_map(|line| line.strip_prefix("- Proof ID: `"))
        .and_then(|value| value.strip_suffix('`'))
        .unwrap_or("unknown")
        .to_string();
    let summary = section_body(&content, "## Evidence");
    let capability_gate_passed = !content.contains("- Capability gate: `failed`");
    let capability_gaps = bullet_section(&content, "## Capability Gaps");
    let capability_warnings = bullet_section(&content, "## Capability Warnings");
    Ok(ProofRecord {
        id,
        summary,
        repo_path: path.to_path_buf(),
        vault_path: PathBuf::new(),
        capability_gate_passed,
        capability_gaps,
        capability_warnings,
        receipt_id: parse_receipt_id(&content),
        source_fingerprint: optional_field_value(&content, "- Source fingerprint: `"),
        binding: parse_operation_binding(&content)?,
    })
}

fn parse_receipt_id(content: &str) -> Option<String> {
    optional_field_value(content, "- Receipt ID: `").filter(|value| value != "none")
}

fn parse_operation_binding(content: &str) -> Result<Option<ReceiptContext>> {
    let fields = [
        optional_field_value(content, "- Task ID: `"),
        optional_field_value(content, "- Operation ID: `"),
        optional_field_value(content, "- Adapter: `"),
        optional_field_value(content, "- Session ID: `"),
        optional_field_value(content, "- Request ID: `"),
        optional_field_value(content, "- Gate kind: `"),
    ];
    let present = fields.iter().filter(|value| value.is_some()).count();
    if present == 0 {
        return Ok(None);
    }
    if present != fields.len() {
        bail!("proof operation binding is incomplete");
    }
    let [task_id, operation_id, adapter, session_id, request_id, gate_kind] = fields;
    let binding = ReceiptContext::new(
        task_id.expect("checked task ID"),
        operation_id.expect("checked operation ID"),
        adapter.expect("checked adapter"),
        session_id.expect("checked session ID"),
        request_id.expect("checked request ID"),
        gate_kind.expect("checked gate kind"),
    );
    binding.validate()?;
    Ok(Some(binding))
}

fn optional_field_value(content: &str, prefix: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .and_then(|value| value.strip_suffix('`'))
        .map(str::to_string)
}

fn complete_operation_binding(
    vault: &VaultContext,
    operation: &OperationContext,
) -> Result<Option<ReceiptContext>> {
    let (Some(task_id), Some(operation_id), Some(session_id), Some(request_id)) = (
        operation.task_id.as_deref(),
        operation.operation_id.as_deref(),
        operation.session_id.as_deref(),
        operation.request_id.as_deref(),
    ) else {
        return Ok(None);
    };
    let binding = ReceiptContext::new(
        task_id,
        operation_id,
        operation.adapter.as_str(),
        session_id,
        request_id,
        "proof",
    );
    binding.validate()?;
    operation
        .lifecycle_identity(&vault.project_id)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    Ok(Some(binding))
}

fn operation_from_binding(binding: &ReceiptContext) -> Result<OperationContext> {
    let adapter = SupportedAdapter::parse(&binding.adapter)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    Ok(OperationContext::new(adapter)
        .with_task_id(binding.task_id.clone())
        .with_operation_id(binding.operation_id.clone())
        .with_session_id(binding.session_id.clone())
        .with_request_id(binding.request_id.clone()))
}

fn operation_binding_matches(left: &ReceiptContext, right: &ReceiptContext) -> bool {
    left.task_id == right.task_id
        && left.operation_id == right.operation_id
        && left.adapter == right.adapter
        && left.session_id == right.session_id
        && left.request_id == right.request_id
}

fn proof_paths(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let root = repo_root.join("docs/baron/proofs");
    if !root.exists() {
        return Ok(Vec::new());
    }
    let root_metadata = fs::symlink_metadata(&root)?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    collect_markdown(&root, &mut paths)?;
    paths.retain(|path| path.file_name().and_then(|value| value.to_str()) != Some("INDEX.md"));
    Ok(paths)
}

fn preflight_publication_paths(
    repo_root: &Path,
    vault: &VaultContext,
    repo_path: &Path,
    vault_path: &Path,
) -> Result<()> {
    let paths = vec![
        repo_path.to_path_buf(),
        vault_path.to_path_buf(),
        repo_root.join("docs/baron/proofs/INDEX.md"),
        vault.project_root.join("Proofs/INDEX.md"),
        repo_root.join("docs/baron/harness/TEST_MATRIX.md"),
        vault.project_root.join("ProductHarness/TEST_MATRIX.md"),
    ];
    for path in paths {
        validate_publication_path(&path)?;
    }
    for path in [repo_path, vault_path] {
        match fs::symlink_metadata(path) {
            Ok(_) => bail!(
                "Proof publication target already exists; refusing overwrite: {}",
                path.display()
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn validate_publication_path(path: &Path) -> Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            bail!(
                "Proof publication target is not a regular file: {}",
                path.display()
            );
        }
    }
    let mut current = path.parent();
    while let Some(parent) = current {
        match fs::symlink_metadata(parent) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    bail!(
                        "Proof publication parent is not a real directory: {}",
                        parent.display()
                    )
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        current = parent.parent();
    }
    Ok(())
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}
