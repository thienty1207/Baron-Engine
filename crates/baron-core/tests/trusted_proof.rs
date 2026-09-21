#[cfg(windows)]
use std::time::Duration;

#[cfg(windows)]
use baron_core::execution_receipt::{
    execute_command_for_identity, ExecutionRequest, ReceiptContext,
};
#[cfg(windows)]
use baron_core::operation::{AuthoritativeLifecycleIdentity, SupportedAdapter};
#[cfg(windows)]
use baron_core::proof::record_proof_from_receipt_bound;
#[cfg(windows)]
use baron_core::vault::ensure_vault;

#[cfg(windows)]
#[test]
fn proof_can_reference_only_a_current_trusted_receipt() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    std::fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "trusted proof",
        SupportedAdapter::Codex,
        Some("session-trusted-proof"),
        Some("request-trusted-proof"),
    )
    .unwrap();
    let binding = ReceiptContext::for_identity(&identity, "proof").unwrap();
    let receipt = execute_command_for_identity(
        ExecutionRequest {
            capability: "test".to_string(),
            provider: "cmd".to_string(),
            executable: "cmd".to_string(),
            arguments: vec!["/C".to_string(), "exit 0".to_string()],
            working_directory: repo.clone(),
            timeout: Duration::from_secs(5),
        },
        &identity,
        &binding.gate_kind,
    )
    .unwrap();
    let proof =
        record_proof_from_receipt_bound(&repo, &context, &receipt.receipt_id, &binding).unwrap();
    assert!(std::fs::read_to_string(proof.repo_path)
        .unwrap()
        .contains("Trusted Execution Receipt"));
}
