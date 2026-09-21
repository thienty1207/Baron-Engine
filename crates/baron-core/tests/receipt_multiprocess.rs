use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use baron_core::execution_receipt::{
    execute_command_for_identity, load_receipts, load_verified_receipt,
    load_verified_receipts_strict, ExecutionRequest,
};
use baron_core::identity::project_id_for_path;
use baron_core::operation::{LifecycleIdentity, SupportedAdapter};
use tempfile::tempdir;

fn command(repo: &std::path::Path) -> ExecutionRequest {
    #[cfg(windows)]
    let (executable, arguments) = ("cmd", vec!["/C".to_string(), "exit 0".to_string()]);
    #[cfg(not(windows))]
    let (executable, arguments) = ("sh", vec!["-c".to_string(), "exit 0".to_string()]);
    ExecutionRequest {
        capability: "multiprocess-test".to_string(),
        provider: "trusted-runner".to_string(),
        executable: executable.to_string(),
        arguments,
        working_directory: repo.to_path_buf(),
        timeout: Duration::from_secs(5),
    }
}

fn identity(repo: &Path, index: &str) -> LifecycleIdentity {
    let project_id = project_id_for_path(repo).unwrap();
    LifecycleIdentity::resolve(
        &project_id,
        &format!("multiprocess worker {index}"),
        SupportedAdapter::Codex,
        Some(&format!("session-worker-{index}")),
        Some(&format!("request-worker-{index}")),
    )
    .unwrap()
}

#[test]
fn verified_receipt_api_is_the_cross_process_authority_entrypoint() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let machine_home = temp.path().join("machine-home");
    fs::create_dir_all(&repo).unwrap();
    let receipt = Command::new(env::current_exe().unwrap())
        .args(["--exact", "receipt_worker", "--nocapture"])
        .env("BARON_RECEIPT_WORKER", "1")
        .env("BARON_RECEIPT_REPO", &repo)
        .env("BARON_RECEIPT_WORKER_INDEX", "single")
        .env("BARON_HOME", &machine_home)
        .env("BARON_RECEIPT_SKIP_BARRIER", "1")
        .output()
        .unwrap();
    assert!(
        receipt.status.success(),
        "{}",
        String::from_utf8_lossy(&receipt.stderr)
    );
    let receipt_id = load_receipts(&repo)
        .unwrap()
        .pop()
        .map(|receipt| receipt.receipt_id)
        .unwrap();
    let previous_home = env::var_os("BARON_HOME");
    env::set_var("BARON_HOME", &machine_home);
    let verified = load_verified_receipt(&repo, &receipt_id).unwrap();
    match previous_home {
        Some(value) => env::set_var("BARON_HOME", value),
        None => env::remove_var("BARON_HOME"),
    }
    assert_eq!(verified.receipt_id, receipt_id);
}

#[test]
fn receipt_worker() {
    if env::var_os("BARON_RECEIPT_WORKER").is_none() {
        return;
    }
    let repo = std::path::PathBuf::from(env::var_os("BARON_RECEIPT_REPO").unwrap());
    let index = env::var("BARON_RECEIPT_WORKER_INDEX").unwrap();
    if env::var_os("BARON_RECEIPT_SKIP_BARRIER").is_none() {
        let ready_dir = std::path::PathBuf::from(env::var_os("BARON_RECEIPT_READY_DIR").unwrap());
        let release = std::path::PathBuf::from(env::var_os("BARON_RECEIPT_RELEASE").unwrap());
        fs::write(ready_dir.join(format!("ready-{index}")), b"ready").unwrap();
        while !release.exists() {
            thread::sleep(Duration::from_millis(5));
        }
    }
    let receipt =
        execute_command_for_identity(command(&repo), &identity(&repo, &index), "proof").unwrap();
    println!("WORKER_RECEIPT={}", receipt.receipt_id);
}

#[test]
fn receipt_verifier() {
    if env::var_os("BARON_RECEIPT_VERIFIER").is_none() {
        return;
    }
    let repo = std::path::PathBuf::from(env::var_os("BARON_RECEIPT_REPO").unwrap());
    let expected = env::var("BARON_RECEIPT_EXPECTED")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let verified = load_verified_receipts_strict(&repo).unwrap();
    assert_eq!(verified.len(), expected);
    let key_ids = verified
        .iter()
        .map(|receipt| receipt.authority_key_id.clone().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(key_ids.len(), 1);
    println!("VERIFIED_RECEIPTS={}", verified.len());
}

#[test]
fn multiprocess_receipt_append_is_lossless_and_authoritative() {
    const WORKER_COUNT: usize = 8;
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let machine_home = temp.path().join("machine-home");
    let ready_dir = temp.path().join("ready");
    let release = temp.path().join("release");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&ready_dir).unwrap();

    let test_binary = env::current_exe().unwrap();
    let mut children = Vec::new();
    for index in 0..WORKER_COUNT {
        let child = Command::new(&test_binary)
            .args(["--exact", "receipt_worker", "--nocapture"])
            .env("BARON_RECEIPT_WORKER", "1")
            .env("BARON_RECEIPT_REPO", &repo)
            .env("BARON_RECEIPT_WORKER_INDEX", index.to_string())
            .env("BARON_HOME", &machine_home)
            .env("BARON_RECEIPT_READY_DIR", &ready_dir)
            .env("BARON_RECEIPT_RELEASE", &release)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        children.push(child);
    }

    let deadline = Instant::now() + Duration::from_secs(10);
    while fs::read_dir(&ready_dir).unwrap().count() < WORKER_COUNT {
        assert!(
            Instant::now() < deadline,
            "workers did not reach the first-start barrier"
        );
        thread::sleep(Duration::from_millis(5));
    }
    fs::write(&release, b"release").unwrap();

    for child in children {
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "worker failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("WORKER_RECEIPT=receipt-"),
            "worker did not emit a receipt marker:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }

    let receipts = load_receipts(&repo).unwrap();
    assert_eq!(receipts.len(), WORKER_COUNT);
    let receipt_ids = receipts
        .iter()
        .map(|receipt| receipt.receipt_id.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(receipt_ids.len(), WORKER_COUNT);
    assert!(receipts.iter().all(|receipt| receipt.schema_version == 2));
    assert!(receipts
        .iter()
        .all(|receipt| receipt.authority_signature.is_some()));

    let output = Command::new(&test_binary)
        .args(["--exact", "receipt_verifier", "--nocapture"])
        .env("BARON_RECEIPT_VERIFIER", "1")
        .env("BARON_RECEIPT_REPO", &repo)
        .env("BARON_RECEIPT_EXPECTED", WORKER_COUNT.to_string())
        .env("BARON_HOME", &machine_home)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "verifier failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("VERIFIED_RECEIPTS=8"));
}
