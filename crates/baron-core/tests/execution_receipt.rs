#[cfg(windows)]
use std::time::Duration;

#[cfg(windows)]
use baron_core::execution_receipt::{
    execute_command, execute_command_for_identity, load_receipts, load_verified_receipt,
    receipt_is_current, ExecutionRequest, ExecutionResult, ReceiptContext,
};
#[cfg(windows)]
use baron_core::identity::project_id_for_path;
#[cfg(windows)]
use baron_core::operation::{AuthoritativeLifecycleIdentity, SupportedAdapter};

#[cfg(windows)]
#[test]
fn trusted_runner_records_current_passing_receipt() {
    let temp = tempfile::tempdir().unwrap();
    let receipt = execute_command(ExecutionRequest {
        capability: "test".to_string(),
        provider: "powershell".to_string(),
        executable: "cmd".to_string(),
        arguments: vec!["/C".to_string(), "exit 0".to_string()],
        working_directory: temp.path().to_path_buf(),
        timeout: Duration::from_secs(5),
    })
    .unwrap();
    assert_eq!(receipt.result, ExecutionResult::Passed);
    assert!(receipt_is_current(temp.path(), &receipt).unwrap());
    assert_eq!(load_receipts(temp.path()).unwrap().len(), 1);
    assert_eq!(
        receipt.receipt_id.strip_prefix("receipt-").unwrap().len(),
        32
    );
}

#[cfg(windows)]
#[test]
fn authoritative_receipt_is_signed_schema_v2_and_verifiable_after_reload() {
    let temp = tempfile::tempdir().unwrap();
    let identity = AuthoritativeLifecycleIdentity::resolve(
        &project_id_for_path(temp.path()).unwrap(),
        "receipt fixture",
        SupportedAdapter::Codex,
        Some("session-receipt"),
        Some("request-receipt"),
    );
    let identity = identity.unwrap();
    let binding = ReceiptContext::for_identity(&identity, "proof").unwrap();
    let receipt = execute_command_for_identity(
        ExecutionRequest {
            capability: "test".to_string(),
            provider: "cmd".to_string(),
            executable: "cmd".to_string(),
            arguments: vec!["/C".to_string(), "exit 0".to_string()],
            working_directory: temp.path().to_path_buf(),
            timeout: Duration::from_secs(5),
        },
        &identity,
        &binding.gate_kind,
    )
    .unwrap();
    assert_eq!(receipt.schema_version, 2);
    assert!(receipt.authority_key_id.is_some());
    assert!(receipt.authority_signature.is_some());
    let verified = load_verified_receipt(temp.path(), &receipt.receipt_id).unwrap();
    assert_eq!(verified.receipt_id, receipt.receipt_id);
}

#[cfg(windows)]
#[test]
fn failed_runner_receipt_cannot_be_a_pass() {
    let temp = tempfile::tempdir().unwrap();
    let receipt = execute_command(ExecutionRequest {
        capability: "test".to_string(),
        provider: "cmd".to_string(),
        executable: "cmd".to_string(),
        arguments: vec!["/C".to_string(), "exit 7".to_string()],
        working_directory: temp.path().to_path_buf(),
        timeout: Duration::from_secs(5),
    })
    .unwrap();
    assert_eq!(receipt.result, ExecutionResult::Failed);
    assert!(!receipt_is_current(temp.path(), &receipt).unwrap());
}

#[cfg(windows)]
#[test]
fn runner_bounds_large_output_without_deadlocking() {
    let temp = tempfile::tempdir().unwrap();
    let receipt = execute_command(ExecutionRequest {
        capability: "test-output".to_string(),
        provider: "cmd".to_string(),
        executable: "cmd".to_string(),
        arguments: vec![
            "/C".to_string(),
            "for /L %i in (1,1,10000) do @echo 012345678901234567890123456789".to_string(),
        ],
        working_directory: temp.path().to_path_buf(),
        timeout: Duration::from_secs(5),
    })
    .unwrap();
    assert_eq!(receipt.result, ExecutionResult::Passed);
    assert!(receipt.stdout_excerpt.contains("output truncated"));
}

#[cfg(windows)]
#[test]
fn source_change_and_tampering_invalidate_receipt() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("README.md"), "before\n").unwrap();
    let receipt = execute_command(ExecutionRequest {
        capability: "test".to_string(),
        provider: "cmd".to_string(),
        executable: "cmd".to_string(),
        arguments: vec!["/C".to_string(), "exit 0".to_string()],
        working_directory: temp.path().to_path_buf(),
        timeout: Duration::from_secs(5),
    })
    .unwrap();
    assert!(receipt_is_current(temp.path(), &receipt).unwrap());
    std::fs::write(temp.path().join("README.md"), "after\n").unwrap();
    assert!(!receipt_is_current(temp.path(), &receipt).unwrap());
    let mut tampered = receipt;
    tampered.stdout_excerpt.push_str("tampered");
    assert!(!receipt_is_current(temp.path(), &tampered).unwrap());
}
