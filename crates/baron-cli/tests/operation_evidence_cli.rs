use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use baron_core::operation::{LifecycleIdentity, OperationContext, SupportedAdapter};
use baron_core::proof::{proof_for_operation, record_proof_for_operation};
use baron_core::trace::{record_trace_for_operation, TraceOperationBinding, TraceOutcome};
use baron_core::vault::{ensure_vault, VaultContext};
use predicates::str::contains;
use tempfile::{tempdir, TempDir};

const TASK_A: &str = "fix README alpha typo";
const TASK_B: &str = "backend login security beta";

fn successful_command() -> (&'static str, &'static [&'static str]) {
    #[cfg(windows)]
    {
        ("cmd", &["/C", "exit 0"])
    }
    #[cfg(not(windows))]
    {
        ("sh", &["-c", "exit 0"])
    }
}

struct Fixture {
    _temp: TempDir,
    repo: PathBuf,
    vault: VaultContext,
    home: PathBuf,
    a: LifecycleIdentity,
}

impl Fixture {
    fn new(two: bool) -> Self {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault_path = temp.path().join("Vault");
        let home = temp.path().join("machine");
        fs::create_dir_all(&repo).unwrap();
        Command::cargo_bin("baron")
            .unwrap()
            .args([
                "init",
                repo.to_str().unwrap(),
                "--codex",
                "--vault",
                vault_path.to_str().unwrap(),
            ])
            .assert()
            .success();
        let vault = ensure_vault(&vault_path, &repo).unwrap();
        let a = LifecycleIdentity::resolve(
            &vault.project_id,
            TASK_A,
            SupportedAdapter::Codex,
            Some("session-a"),
            Some("request-a"),
        )
        .unwrap();
        let fixture = Self {
            _temp: temp,
            repo,
            vault,
            home,
            a,
        };
        fixture
            .command()
            .args([
                "plan",
                "start",
                TASK_A,
                "--adapter",
                "codex",
                "--session-id",
                "session-a",
                "--request-id",
                "request-a",
            ])
            .assert()
            .success();
        if two {
            fixture
                .command()
                .args([
                    "plan",
                    "start",
                    TASK_B,
                    "--adapter",
                    "claude",
                    "--session-id",
                    "session-b",
                    "--request-id",
                    "request-b",
                ])
                .assert()
                .success();
        }
        fixture
    }

    fn command(&self) -> Command {
        let mut command = Command::cargo_bin("baron").unwrap();
        command
            .current_dir(&self.repo)
            .env("BARON_HOME", &self.home);
        command
    }

    fn selector(&self) -> [&str; 8] {
        [
            "--task",
            TASK_A,
            "--adapter",
            "codex",
            "--session-id",
            "session-a",
            "--request-id",
            "request-a",
        ]
    }

    fn proof(&self) -> baron_core::proof::ProofRecord {
        record_proof_for_operation(
            &self.repo,
            &self.vault,
            &OperationContext::from_identity(&self.a),
            "README alpha verification passed",
        )
        .unwrap()
    }

    fn trace(&self) -> baron_core::trace::TraceRecord {
        let proof = self.proof();
        record_trace_for_operation(
            &self.repo,
            &self.vault,
            "alpha README corrected",
            TraceOutcome::Completed,
            &TraceOperationBinding::from_operation(
                &OperationContext::from_identity(&self.a),
                &proof.id,
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn snapshot(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut snapshot = BTreeMap::new();
        collect(&self.repo, &mut snapshot);
        collect(&self.vault.vault_root, &mut snapshot);
        snapshot
    }
}

fn collect(root: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
    if !root.exists() {
        return;
    }
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, files);
        } else if path.file_name().unwrap() != ".baron-mutation.lock" {
            files.insert(path.clone(), fs::read(path).unwrap());
        }
    }
}

fn field(output: &[u8], label: &str) -> String {
    String::from_utf8_lossy(output)
        .lines()
        .find_map(|line| {
            line.strip_prefix(&format!("- {label}: `"))
                .and_then(|value| value.strip_suffix('`'))
        })
        .unwrap()
        .to_string()
}

#[test]
fn recovery_cli_selects_a_while_current_is_b() {
    let f = Fixture::new(true);
    let proof = f.proof();
    f.command()
        .args([
            "continuity",
            "recover",
            "A-only failure",
            "--outcome",
            "failed",
            "--last-success",
            "A-only verified step",
            "--next-action",
            "retry A-only check",
            "--affected-file",
            "README.md",
        ])
        .args(f.selector())
        .assert()
        .success();
    let content =
        fs::read_to_string(f.repo.join("docs/baron/continuity/CURRENT_RECOVERY.md")).unwrap();
    assert!(content.contains(&format!("- Operation ID: `{}`", f.a.operation_id())));
    assert!(content.contains(&proof.id));
    assert!(content.contains(TASK_A));
    assert!(!content.contains(TASK_B));
}

#[test]
fn recovery_cli_no_selector_fails_before_publishing_with_two_active_plans() {
    let f = Fixture::new(true);
    let before = f.snapshot();
    f.command()
        .args([
            "continuity",
            "recover",
            "ambiguous failure",
            "--outcome",
            "failed",
            "--last-success",
            "previous step",
            "--next-action",
            "retry check",
        ])
        .assert()
        .failure()
        .stderr(contains("ambiguous"));
    assert_eq!(f.snapshot(), before);
    f.command()
        .args([
            "continuity",
            "recover",
            "partial identity failure",
            "--outcome",
            "failed",
            "--last-success",
            "previous step",
            "--next-action",
            "retry check",
            "--task",
            TASK_A,
        ])
        .assert()
        .failure()
        .stderr(contains("together"));
    assert_eq!(f.snapshot(), before);
}

#[test]
fn proof_record_explicit_a_never_uses_current_b() {
    let f = Fixture::new(true);
    f.command()
        .args(["proof", "record", "README alpha passed"])
        .args(f.selector())
        .assert()
        .success();
    let binding =
        baron_core::execution_receipt::ReceiptContext::for_identity(&f.a, "proof").unwrap();
    assert!(proof_for_operation(&f.repo, &binding).unwrap().is_some());
    assert!(
        fs::read_to_string(f.repo.join("docs/baron/plans/CURRENT.md"))
            .unwrap()
            .contains(TASK_B)
    );
}

#[test]
fn receipt_for_a_records_while_current_is_claude_b() {
    let f = Fixture::new(true);
    let (command, command_arguments) = successful_command();
    let output = f
        .command()
        .args([
            "proof",
            "execute",
            "--capability",
            "test",
            "--provider",
            "trusted-runner",
        ])
        .args(f.selector())
        .args([command, "--"])
        .args(command_arguments)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let receipt = field(&output, "Receipt");
    f.command()
        .args([
            "proof",
            "record",
            "trusted alpha passed",
            "--receipt",
            &receipt,
            "--task-id",
            f.a.task_id(),
            "--operation-id",
            f.a.operation_id(),
            "--adapter",
            "codex",
            "--session-id",
            "session-a",
            "--request-id",
            "request-a",
            "--gate-kind",
            "proof",
        ])
        .assert()
        .success();
    let proof = proof_for_operation(
        &f.repo,
        &baron_core::execution_receipt::ReceiptContext::for_identity(&f.a, "proof").unwrap(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(proof.receipt_id.as_deref(), Some(receipt.as_str()));
}

#[test]
fn valid_receipt_cannot_publish_proof_without_exact_active_operation() {
    let f = Fixture::new(true);
    let (command, command_arguments) = successful_command();
    let output = f
        .command()
        .args([
            "proof",
            "execute",
            "--capability",
            "test",
            "--provider",
            "trusted-runner",
        ])
        .args(f.selector())
        .args([command, "--"])
        .args(command_arguments)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let receipt = field(&output, "Receipt");

    let active = f.repo.join("docs/baron/plans/ACTIVE.md");
    let content = fs::read_to_string(&active).unwrap();
    let rows = content
        .lines()
        .filter(|line| !line.contains(&format!("\"operation_id\":\"{}\"", f.a.operation_id())))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_ne!(
        content, rows,
        "fixture must remove operation A's ACTIVE row"
    );
    fs::write(active, rows).unwrap();
    let before = f.snapshot();
    f.command()
        .args([
            "proof",
            "record",
            "trusted alpha passed",
            "--receipt",
            &receipt,
            "--task-id",
            f.a.task_id(),
            "--operation-id",
            f.a.operation_id(),
            "--adapter",
            "codex",
            "--session-id",
            "session-a",
            "--request-id",
            "request-a",
            "--gate-kind",
            "proof",
        ])
        .assert()
        .failure();
    assert_eq!(f.snapshot(), before);
}

#[test]
fn trace_record_and_auto_score_select_a_plan_risk_and_proof() {
    let f = Fixture::new(true);
    let proof = f.proof();
    let output = f
        .command()
        .args(["trace", "record", "README alpha corrected"])
        .args(f.selector())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let id = field(&output, "Trace ID");
    let expected =
        TraceOperationBinding::from_operation(&OperationContext::from_identity(&f.a), &proof.id)
            .unwrap();
    let trace = baron_core::trace::trace_for_operation(&f.repo, &expected)
        .unwrap()
        .unwrap();
    assert_eq!(trace.id, id);
    assert_eq!(trace.proof_id.as_deref(), Some(proof.id.as_str()));
    assert_eq!(trace.binding.unwrap().operation_id, f.a.operation_id());
    f.command()
        .args(["trace", "score"])
        .args(f.selector())
        .assert()
        .success()
        .stdout(contains("Required: `minimal`"))
        .stdout(contains("Passed: `yes`"));
}

#[test]
fn ambiguous_cli_evidence_ingress_does_not_mutate_repo_or_vault() {
    let f = Fixture::new(true);
    let trace = f.trace();
    for args in [
        vec!["proof", "record", "must not publish"],
        vec!["trace", "record", "must not publish"],
        vec!["trace", "score"],
    ] {
        let before = f.snapshot();
        f.command()
            .args(args)
            .assert()
            .failure()
            .stderr(contains("ambiguous"));
        assert_eq!(f.snapshot(), before);
    }
    // An explicit artifact ID is exact authority, not automatic/latest selection.
    f.command()
        .args(["trace", "score", "--id", &trace.id])
        .assert()
        .success();
}

#[test]
fn partial_or_wrong_selector_fails_before_evidence_publication() {
    let f = Fixture::new(true);
    for args in [
        vec!["proof", "record", "must not publish", "--task", TASK_A],
        vec![
            "trace",
            "record",
            "must not publish",
            "--task",
            TASK_A,
            "--adapter",
            "codex",
            "--session-id",
            "wrong",
            "--request-id",
            "request-a",
        ],
    ] {
        let before = f.snapshot();
        f.command().args(args).assert().failure();
        assert_eq!(f.snapshot(), before);
    }
}

#[test]
fn single_active_legacy_evidence_ingress_remains_compatible_without_current() {
    let f = Fixture::new(false);
    fs::remove_file(f.repo.join("docs/baron/plans/CURRENT.md")).unwrap();
    f.command()
        .args(["proof", "record", "README alpha passed"])
        .assert()
        .success();
    let binding =
        baron_core::execution_receipt::ReceiptContext::for_identity(&f.a, "proof").unwrap();
    assert!(proof_for_operation(&f.repo, &binding).unwrap().is_some());
    f.command()
        .args(["trace", "record", "README corrected"])
        .assert()
        .success();
    f.command()
        .args(["trace", "score"])
        .assert()
        .success()
        .stdout(contains("Passed: `yes`"));
}

#[test]
fn explicit_cli_selector_ignores_corrupt_current_but_not_active_authority() {
    let f = Fixture::new(true);
    fs::write(
        f.repo.join("docs/baron/plans/CURRENT.md"),
        "presentation is unavailable\n",
    )
    .unwrap();
    f.command()
        .args(["proof", "record", "README alpha passed"])
        .args(f.selector())
        .assert()
        .success();
    f.command()
        .args(["trace", "record", "README corrected"])
        .args(f.selector())
        .assert()
        .success();
    f.command()
        .args(["trace", "score"])
        .args(f.selector())
        .assert()
        .success();
    let active = f.repo.join("docs/baron/plans/ACTIVE.md");
    let text = fs::read_to_string(&active).unwrap();
    fs::write(&active, text.replace("session-a", "forged-session")).unwrap();
    let before = f.snapshot();
    f.command()
        .args(["proof", "record", "must not publish"])
        .args(f.selector())
        .assert()
        .failure();
    assert_eq!(f.snapshot(), before);
}

#[test]
fn ambiguous_cli_ingress_still_fails_without_current_projection() {
    let f = Fixture::new(true);
    f.trace();
    fs::remove_file(f.repo.join("docs/baron/plans/CURRENT.md")).unwrap();
    for args in [
        vec!["proof", "record", "must not publish"],
        vec!["trace", "record", "must not publish"],
        vec!["trace", "score"],
    ] {
        let before = f.snapshot();
        f.command()
            .args(args)
            .assert()
            .failure()
            .stderr(contains("ambiguous"));
        assert_eq!(f.snapshot(), before);
    }
}
