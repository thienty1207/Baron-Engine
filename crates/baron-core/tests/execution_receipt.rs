#[cfg(windows)]
use std::time::Duration;

#[cfg(windows)]
use baron_core::execution_receipt::{
    execute_command, load_receipts, receipt_is_current, ExecutionRequest, ExecutionResult,
};

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
