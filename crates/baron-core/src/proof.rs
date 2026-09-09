use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};

use crate::capability::{
    evaluate_execution_evidence, evaluate_execution_evidence_for_operation, load_capability_state,
    record_runtime_execution, CapabilityExecutionEvidence,
};
use crate::execution_receipt::{load_receipts, receipt_matches_context, ReceiptContext};
use crate::harness::{current_harness_risk, update_current_validation_evidence};
use crate::operation::OperationContext;
use crate::risk::RiskLane;
use crate::safe_io::replace_text;
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
}

pub fn record_proof(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    summary: &str,
) -> Result<ProofRecord> {
    record_proof_internal(repo_root, vault, None, summary, &[])
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
    let receipt = load_receipts(repo_root)?
        .into_iter()
        .find(|receipt| receipt.receipt_id == receipt_id.trim())
        .with_context(|| format!("Trusted execution receipt not found: {}", receipt_id.trim()))?;
    if !receipt_matches_context(repo_root, &receipt, binding)? {
        bail!(
            "Trusted execution receipt `{}` is stale, failed, mismatched, replayed, or tampered",
            receipt.receipt_id,
        );
    }
    let proof = record_proof(
        repo_root,
        vault,
        &format!(
            "trusted execution receipt {} passed for {} via {}",
            receipt.receipt_id, receipt.capability, receipt.provider
        ),
    )?;
    append_receipt_reference(&proof.repo_path, &receipt, binding)?;
    append_receipt_reference(&proof.vault_path, &receipt, binding)?;
    Ok(proof)
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
    record_proof_internal(
        repo_root,
        vault,
        Some(operation),
        summary,
        capability_evidence,
    )
}

fn record_proof_internal(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    operation: Option<&OperationContext>,
    summary: &str,
    capability_evidence: &[CapabilityExecutionEvidence],
) -> Result<ProofRecord> {
    let repo_root = repo_root.as_ref();
    let id = Local::now().format("%Y%m%d%H%M%S%3f").to_string();
    let date = Local::now().format("%Y-%m-%d").to_string();
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
    record_runtime_execution(repo_root, capability_evidence)?;
    let content = render_proof(
        &id,
        summary,
        capability_evidence,
        capability_gate.passed,
        &capability_gate.gaps,
        &capability_gate.warnings,
        operation,
    );
    write(&repo_path, &content)?;
    write(&vault_path, &content)?;
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
    let content = fs::read_to_string(&path)?;
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
    Ok(Some(ProofRecord {
        id,
        summary,
        repo_path: path,
        vault_path: PathBuf::new(),
        capability_gate_passed,
        capability_gaps,
        capability_warnings,
    }))
}

fn render_proof(
    id: &str,
    summary: &str,
    capability_evidence: &[CapabilityExecutionEvidence],
    gate_passed: bool,
    gaps: &[String],
    warnings: &[String],
    operation: Option<&OperationContext>,
) -> String {
    let operation_identity = operation
        .map(|operation| {
            format!(
                "- Adapter: `{}`\n- Session ID: `{}`\n- Request ID: `{}`\n",
                operation.adapter.as_str(),
                operation.session_id.as_deref().unwrap_or("none"),
                operation.request_id.as_deref().unwrap_or("none")
            )
        })
        .unwrap_or_default();
    let mut content = format!(
        "# Baron Proof\n\n- Proof ID: `{id}`\n- Recorded: {}\n- Capability gate: `{}`\n{}\n## Evidence\n\n{}\n\n## Capability Execution Evidence\n\n",
        now(),
        if gate_passed { "passed" } else { "failed" },
        operation_identity,
        summary.trim()
    );
    if capability_evidence.is_empty() {
        content.push_str("- none recorded\n");
    } else {
        for evidence in capability_evidence {
            content.push_str(&format!(
                "- `{}` via `{}` - {}\n",
                evidence.capability.trim(),
                evidence.provider.trim(),
                evidence.summary.trim()
            ));
        }
    }
    content.push_str("\n## Capability Gaps\n\n");
    push_bullets(&mut content, gaps);
    content.push_str("\n## Capability Warnings\n\n");
    push_bullets(&mut content, warnings);
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
    let mut content = fs::read_to_string(path).unwrap_or_else(|_| header.to_string());
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(item);
    content.push('\n');
    write(path, &content)
}

fn write(path: &Path, content: &str) -> Result<()> {
    replace_text(path, content).with_context(|| format!("Could not write {}", path.display()))
}

/// Returns whether a proof points at a Baron-owned receipt that is still valid
/// for the current project source and exact proof operation. Free-form proof
/// remains readable for migration and low-risk reporting, but it is not
/// completion evidence for a medium/high-risk plan.
pub fn proof_has_current_receipt(repo_root: &Path, proof: &ProofRecord) -> Result<bool> {
    let Some((receipt_id, binding)) = proof_receipt_context(proof)? else {
        return Ok(false);
    };
    let Some(receipt) = load_receipts(repo_root)?
        .into_iter()
        .find(|receipt| receipt.receipt_id == receipt_id)
    else {
        return Ok(false);
    };
    receipt_matches_context(repo_root, &receipt, &binding)
}

/// Read the explicit receipt binding recorded in a proof.  A proof without a
/// complete binding remains readable diagnostic history, but cannot authorize
/// a medium/high-risk completion or provide a task scope for quality gates.
pub fn proof_receipt_context(proof: &ProofRecord) -> Result<Option<(String, ReceiptContext)>> {
    let content = fs::read_to_string(&proof.repo_path)?;
    let Some(receipt_id) = content
        .lines()
        .find_map(|line| line.strip_prefix("- Receipt ID: `"))
        .and_then(|value| value.strip_suffix('`'))
    else {
        return Ok(None);
    };
    let binding = ReceiptContext::new(
        field_value(&content, "- Task ID: `")?,
        field_value(&content, "- Operation ID: `")?,
        field_value(&content, "- Adapter: `")?,
        field_value(&content, "- Session ID: `")?,
        field_value(&content, "- Request ID: `")?,
        field_value(&content, "- Gate kind: `")?,
    );
    Ok(Some((receipt_id.to_string(), binding)))
}

fn field_value(content: &str, prefix: &str) -> Result<String> {
    content
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .and_then(|value| value.strip_suffix('`'))
        .map(str::to_string)
        .with_context(|| format!("proof receipt binding field is missing: {prefix}"))
}

fn append_receipt_reference(
    path: &Path,
    receipt: &crate::execution_receipt::ExecutionReceipt,
    binding: &ReceiptContext,
) -> Result<()> {
    let mut content = fs::read_to_string(path)?;
    content.push_str(&format!(
        "\n## Trusted Execution Receipt\n\n- Receipt ID: `{}`\n- Source: Baron-owned execution runner\n- Task ID: `{}`\n- Operation ID: `{}`\n- Adapter: `{}`\n- Session ID: `{}`\n- Request ID: `{}`\n- Gate kind: `{}`\n- Source fingerprint: `{}`\n",
        receipt.receipt_id,
        binding.task_id.trim(),
        binding.operation_id.trim(),
        binding.adapter.trim(),
        binding.session_id.trim(),
        binding.request_id.trim(),
        binding.gate_kind.trim(),
        receipt.source_fingerprint,
    ));
    write(path, &content)
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}
