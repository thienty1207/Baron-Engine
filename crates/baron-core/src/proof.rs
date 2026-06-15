use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{Local, SecondsFormat};

use crate::capability::{
    default_adapter, evaluate_execution_evidence, CapabilityExecutionEvidence,
};
use crate::harness::{current_harness_risk, update_current_validation_evidence};
use crate::risk::RiskLane;
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
    record_proof_with_capabilities(repo_root, vault, summary, &[])
}

pub fn record_proof_with_capabilities(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
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
    let capability_gate = match default_adapter(repo_root) {
        Ok(adapter) => evaluate_execution_evidence(repo_root, adapter, capability_evidence)?,
        Err(_) => crate::capability::CapabilityGate {
            passed: true,
            gaps: Vec::new(),
            warnings: Vec::new(),
        },
    };
    let content = render_proof(
        &id,
        summary,
        capability_evidence,
        capability_gate.passed,
        &capability_gate.gaps,
        &capability_gate.warnings,
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
) -> String {
    let mut content = format!(
        "# Baron Proof\n\n- Proof ID: `{id}`\n- Recorded: {}\n- Capability gate: `{}`\n\n## Evidence\n\n{}\n\n## Capability Execution Evidence\n\n",
        now(),
        if gate_passed { "passed" } else { "failed" },
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
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content).with_context(|| format!("Could not write {}", path.display()))
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}
