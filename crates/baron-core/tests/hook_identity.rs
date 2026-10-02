//! Native schema snapshot checked 2026-10-01:
//! https://learn.chatgpt.com/docs/hooks (unversioned Codex reference)
//! https://code.claude.com/docs/en/hooks (prompt_id: Claude Code >=2.1.196).
//! Common: session_id, transcript_path, cwd, hook_event_name.
//! Host   Event                 Turn key    Prompt text
//! Codex  SessionStart          absent      absent (source only)
//! Codex  UserPromptSubmit      turn_id     prompt
//! Codex  PreCompact/PostToolUse turn_id    absent (trigger/tool fields)
//! Codex  Stop                  turn_id     absent (last_assistant_message)
//! Claude SessionStart          prompt_id*  absent (source/model)
//! Claude UserPromptSubmit      prompt_id*  prompt
//! Claude PreCompact/PostToolUse prompt_id* absent (trigger/tool fields)
//! Claude Stop                  prompt_id*  absent (last_assistant_message)
//! * Common field absent before first input; older example schemas omit it.
//!
//! request_id is a Baron compatibility input, not a claimed Claude host field.

use std::fs;
use std::path::Path;

use baron_core::automation::{handle_hook, AutomationEvent, HookAdapter};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::operation::{LifecycleIdentity, OperationContext, SupportedAdapter};
use baron_core::plan::{complete_plan_for_identity, start_or_resume_plan_for_identity};
use baron_core::proof::record_proof_for_operation;
use baron_core::trace::{
    record_trace_for_operation, score_trace, TraceOperationBinding, TraceOutcome,
};
use baron_core::vault::{ensure_vault, VaultContext};
use serde_json::{json, Value};
use tempfile::tempdir;

const MAP: &str = ".baron/state/hook-correlations.json";

fn project() -> (tempfile::TempDir, std::path::PathBuf, VaultContext) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    (temp, repo, context)
}

fn deliver(
    repo: &Path,
    vault: &VaultContext,
    adapter: HookAdapter,
    event: AutomationEvent,
    payload: Value,
) -> Value {
    serde_json::from_str(&handle_hook(repo, vault, adapter, event, &payload.to_string()).unwrap())
        .unwrap()
}

fn host(
    adapter: HookAdapter,
    event: &str,
    session: &str,
    turn: Option<&str>,
    task: Option<&str>,
) -> Value {
    let mut value = json!({"session_id":session,"transcript_path":"/workspace/transcript.jsonl","cwd":"/workspace","hook_event_name":event});
    if let Some(turn) = turn {
        value[if adapter == HookAdapter::Codex {
            "turn_id"
        } else {
            "prompt_id"
        }] = json!(turn);
    }
    if let Some(task) = task {
        value["prompt"] = json!(task);
    }
    if event == "Stop" {
        value["stop_hook_active"] = json!(false);
        value["last_assistant_message"] = json!("Work ready for verification");
    }
    value
}

fn passing(repo: &Path, vault: &VaultContext, identity: &LifecycleIdentity) {
    let operation = OperationContext::from_identity(identity);
    let proof =
        record_proof_for_operation(repo, vault, &operation, "README verification passed").unwrap();
    let trace = record_trace_for_operation(
        repo,
        vault,
        "README task completed",
        TraceOutcome::Completed,
        &TraceOperationBinding::from_operation(&operation, &proof.id).unwrap(),
    )
    .unwrap();
    assert!(score_trace(repo, vault, Some(&trace.id)).unwrap().passed);
}

fn establish(
    repo: &Path,
    vault: &VaultContext,
    adapter: HookAdapter,
    session: &str,
    turn: Option<&str>,
    task: &str,
) -> LifecycleIdentity {
    let prompt = deliver(
        repo,
        vault,
        adapter,
        AutomationEvent::UserPromptSubmit,
        host(adapter, "UserPromptSubmit", session, turn, Some(task)),
    );
    assert_ne!(prompt["decision"], "block", "{prompt}");
    let identity = LifecycleIdentity::resolve(
        &vault.project_id,
        task,
        if adapter == HookAdapter::Codex {
            SupportedAdapter::Codex
        } else {
            SupportedAdapter::Claude
        },
        Some(session),
        prompt["baron"]["request_id"].as_str(),
    )
    .unwrap();
    assert_eq!(prompt["baron"]["operation_id"], identity.operation_id());
    if let Some(turn) = turn {
        assert_eq!(prompt["baron"]["request_id"], turn);
    }
    start_or_resume_plan_for_identity(repo, vault, task, &identity).unwrap();
    identity
}

fn matrix(first_adapter: HookAdapter, a_passes: bool) {
    let (_temp, repo, vault) = project();
    let second_adapter = if first_adapter == HookAdapter::Codex {
        HookAdapter::Claude
    } else {
        HookAdapter::Codex
    };
    let a_turn = "550e8400-e29b-41d4-a716-446655440000";
    let b_turn = "550e8400-e29b-41d4-a716-446655440001";
    let a = establish(
        &repo,
        &vault,
        first_adapter,
        "session-a",
        Some(a_turn),
        "fix README alpha typo",
    );
    if a_passes {
        passing(&repo, &vault, &a);
    }
    let b = establish(
        &repo,
        &vault,
        second_adapter,
        "session-b",
        Some(b_turn),
        "fix README beta typo",
    );
    if !a_passes {
        passing(&repo, &vault, &b);
    }
    let current = fs::read(repo.join("docs/baron/plans/CURRENT.md")).unwrap();
    // A fresh Vault handle and cleared response cache model a new process;
    // the CLI regression additionally runs actual separate processes.
    let fresh = ensure_vault(&vault.vault_root, &repo).unwrap();
    let compact = deliver(
        &repo,
        &fresh,
        first_adapter,
        AutomationEvent::PreCompact,
        host(first_adapter, "PreCompact", "session-a", Some(a_turn), None),
    );
    assert_eq!(compact["baron"]["operation_id"], a.operation_id());
    let stop_payload = host(first_adapter, "Stop", "session-a", Some(a_turn), None);
    let stop = deliver(
        &repo,
        &fresh,
        first_adapter,
        AutomationEvent::Stop,
        stop_payload.clone(),
    );
    assert_eq!(stop["baron"]["task_id"], a.task_id());
    assert_eq!(stop["baron"]["operation_id"], a.operation_id());
    assert_eq!(stop["baron"]["session_id"], "session-a");
    assert_eq!(stop["baron"]["request_id"], a_turn);
    assert_eq!(stop["baron"]["reconciliation_passed"], a_passes);
    assert_eq!(stop["decision"] == "block", !a_passes);
    assert_eq!(stop["completed"], false);
    assert_eq!(
        stop,
        deliver(
            &repo,
            &fresh,
            first_adapter,
            AutomationEvent::Stop,
            stop_payload
        )
    );
    assert_eq!(
        current,
        fs::read(repo.join("docs/baron/plans/CURRENT.md")).unwrap()
    );
}

#[test]
fn codex_taskless_stop_a_pass_b_fail_current_b() {
    matrix(HookAdapter::Codex, true);
}
#[test]
fn codex_taskless_stop_a_fail_b_pass_current_b() {
    matrix(HookAdapter::Codex, false);
}
#[test]
fn claude_taskless_stop_a_pass_b_fail_current_b() {
    matrix(HookAdapter::Claude, true);
}
#[test]
fn claude_taskless_stop_a_fail_b_pass_current_b() {
    matrix(HookAdapter::Claude, false);
}

#[test]
fn old_claude_session_only_is_durable_unambiguous_and_retry_stable() {
    let (_temp, repo, vault) = project();
    let a = establish(
        &repo,
        &vault,
        HookAdapter::Claude,
        "old-session",
        None,
        "fix README alpha typo",
    );
    passing(&repo, &vault, &a);
    let stop_payload = host(HookAdapter::Claude, "Stop", "old-session", None, None);
    let stop = deliver(
        &repo,
        &vault,
        HookAdapter::Claude,
        AutomationEvent::Stop,
        stop_payload.clone(),
    );
    assert_eq!(stop["baron"]["operation_id"], a.operation_id());
    assert_eq!(stop["baron"]["reconciliation_passed"], true);
    assert_eq!(
        stop,
        deliver(
            &repo,
            &vault,
            HookAdapter::Claude,
            AutomationEvent::Stop,
            stop_payload,
        )
    );
    establish(
        &repo,
        &vault,
        HookAdapter::Claude,
        "old-session",
        None,
        "fix README beta typo",
    );
    let ambiguous = deliver(
        &repo,
        &vault,
        HookAdapter::Claude,
        AutomationEvent::Stop,
        host(HookAdapter::Claude, "Stop", "old-session", None, None),
    );
    assert_eq!(ambiguous["decision"], "block");
    assert!(ambiguous.to_string().contains("ambiguous"), "{ambiguous}");
}

#[test]
fn old_claude_same_text_prompts_are_distinct_and_completed_mapping_is_not_resurrected() {
    for completed in [false, true] {
        let (_temp, repo, vault) = project();
        let a = establish(
            &repo,
            &vault,
            HookAdapter::Claude,
            "same-session",
            None,
            "fix README alpha typo",
        );
        if completed {
            passing(&repo, &vault, &a);
            complete_plan_for_identity(&repo, &vault, "README verified", &a).unwrap();
        }
        let second = deliver(
            &repo,
            &vault,
            HookAdapter::Claude,
            AutomationEvent::UserPromptSubmit,
            host(
                HookAdapter::Claude,
                "UserPromptSubmit",
                "same-session",
                None,
                Some("fix README alpha typo"),
            ),
        );
        assert_ne!(
            second["baron"]["operation_id"],
            a.operation_id(),
            "session+task is not a stable turn key: {second}"
        );
        assert_ne!(second["baron"]["request_id"], a.request_id());
        let stop = deliver(
            &repo,
            &vault,
            HookAdapter::Claude,
            AutomationEvent::Stop,
            host(HookAdapter::Claude, "Stop", "same-session", None, None),
        );
        assert_eq!(stop["decision"], "block");
        assert!(stop.to_string().contains("ambiguous"), "{stop}");
    }
}

#[test]
fn unknown_turn_and_wrong_host_never_use_session_or_current() {
    let (_temp, repo, vault) = project();
    establish(
        &repo,
        &vault,
        HookAdapter::Codex,
        "s",
        Some("t"),
        "fix README typo",
    );
    for (adapter, session, turn) in [
        (HookAdapter::Codex, "s", "unknown"),
        (HookAdapter::Codex, "other", "t"),
        (HookAdapter::Claude, "s", "t"),
    ] {
        let stop = deliver(
            &repo,
            &vault,
            adapter,
            AutomationEvent::Stop,
            host(adapter, "Stop", session, Some(turn), None),
        );
        assert_eq!(stop["decision"], "block");
        assert!(stop.to_string().contains("correlation"), "{stop}");
        assert_ne!(stop["baron"]["reconciliation_passed"], true);
    }
}

#[test]
fn conflicting_aliases_and_invalid_identity_are_rejected_before_writes() {
    let (_temp, repo, vault) = project();
    for payload in [
        json!({"session_id":"s","turn_id":"a","requestId":"b","prompt":"fix README"}),
        json!({"session_id":"s","sessionId":"other","turn_id":"a","prompt":"fix README"}),
        json!({"session_id":"s","turn_id":123,"prompt":"fix README"}),
        json!({"session_id":"s","turn_id":" ","prompt":"fix README"}),
    ] {
        let result = handle_hook(
            &repo,
            &vault,
            HookAdapter::Codex,
            AutomationEvent::UserPromptSubmit,
            &payload.to_string(),
        );
        assert!(result.is_err(), "accepted invalid aliases: {result:?}");
        assert!(!repo.join(MAP).exists());
        assert!(!repo.join(".baron/cache/automation-dedup.json").exists());
    }
    let result = handle_hook(&repo,&vault,HookAdapter::Claude,AutomationEvent::UserPromptSubmit,&json!({"session_id":"s","prompt_id":"550e8400-e29b-41d4-a716-446655440000","promptId":"550e8400-e29b-41d4-a716-446655440001","prompt":"fix README"}).to_string());
    assert!(result.is_err());
}

fn load_map(repo: &Path) -> Value {
    assert!(
        repo.join(MAP).exists(),
        "Prompt did not persist lifecycle correlation"
    );
    serde_json::from_slice(&fs::read(repo.join(MAP)).unwrap()).unwrap()
}

fn authority_side_effects(repo: &Path, vault: &VaultContext) -> Vec<Option<Vec<u8>>> {
    [
        repo.join("docs/baron/continuity/CURRENT.md"),
        repo.join("docs/baron/continuity/CURRENT_RECOVERY.md"),
        repo.join(".baron/cache/automation-dedup.json"),
        vault
            .project_root
            .join("Artifacts/automation-journal.jsonl"),
    ]
    .into_iter()
    .map(|path| fs::read(path).ok())
    .collect()
}

#[test]
fn malformed_duplicate_stale_and_task_conflicting_maps_block_cached_stop() {
    for corruption in [
        "malformed",
        "duplicate",
        "project",
        "task",
        "operation",
        "session",
        "turn",
        "timestamp",
        "version",
        "identity_whitespace",
    ] {
        let (_temp, repo, vault) = project();
        let a = establish(
            &repo,
            &vault,
            HookAdapter::Codex,
            "s",
            Some("t"),
            "fix README typo",
        );
        passing(&repo, &vault, &a);
        let payload = host(HookAdapter::Codex, "Stop", "s", Some("t"), None);
        assert_eq!(
            deliver(
                &repo,
                &vault,
                HookAdapter::Codex,
                AutomationEvent::Stop,
                payload.clone()
            )["baron"]["reconciliation_passed"],
            true
        );
        let mut map = load_map(&repo);
        match corruption {
            "duplicate" => {
                let entry = map["entries"][0].clone();
                map["entries"].as_array_mut().unwrap().push(entry);
            }
            "project" => map["project_id"] = json!("foreign-project"),
            "task" => map["entries"][0]["canonical_task"] = json!("a different task"),
            "operation" => map["entries"][0]["identity"]["operation_id"] = json!("op-foreign"),
            "session" => map["entries"][0]["host_session_id"] = json!("foreign-session"),
            "turn" => map["entries"][0]["host_turn_id"] = json!("foreign-turn"),
            "timestamp" => map["entries"][0]["created_at"] = json!("invalid"),
            "version" => map["schema_version"] = json!(99),
            "identity_whitespace" => {
                let project = map["entries"][0]["identity"]["project_id"]
                    .as_str()
                    .unwrap()
                    .to_string();
                map["entries"][0]["identity"]["project_id"] = json!(format!(" {project} "));
            }
            _ => {}
        }
        let bytes = if corruption == "malformed" {
            "{broken".to_string()
        } else {
            map.to_string()
        };
        fs::write(repo.join(MAP), &bytes).unwrap();
        let before = authority_side_effects(&repo, &vault);
        let stop = deliver(
            &repo,
            &vault,
            HookAdapter::Codex,
            AutomationEvent::Stop,
            payload,
        );
        assert_eq!(stop["decision"], "block", "{corruption}: {stop}");
        assert_ne!(stop["baron"]["reconciliation_passed"], true);
        assert_eq!(fs::read_to_string(repo.join(MAP)).unwrap(), bytes);
        assert_eq!(
            before,
            authority_side_effects(&repo, &vault),
            "corrupt correlation wrote journal/continuity/dedup: {corruption}"
        );
    }
}

#[test]
fn malformed_nested_identity_container_is_rejected_before_recursion_bypass() {
    let (_temp, repo, vault) = project();
    let result = handle_hook(&repo, &vault, HookAdapter::Codex, AutomationEvent::UserPromptSubmit,
        &json!({"session_id":"s","turn_id":"t","prompt":"fix README","request":42,"baron_recursion_depth":1}).to_string());
    assert!(
        result.is_err(),
        "malformed request container was accepted: {result:?}"
    );
    assert!(!repo.join(MAP).exists());
}

#[test]
fn old_claude_completed_turn_cannot_be_replaced_by_new_session_only_authority() {
    let (_temp, repo, vault) = project();
    let a = establish(
        &repo,
        &vault,
        HookAdapter::Claude,
        "old-session",
        None,
        "fix README alpha typo",
    );
    passing(&repo, &vault, &a);
    complete_plan_for_identity(&repo, &vault, "README verified", &a).unwrap();
    let b = establish(
        &repo,
        &vault,
        HookAdapter::Claude,
        "old-session",
        None,
        "fix README beta typo",
    );
    passing(&repo, &vault, &b);
    let stop = deliver(
        &repo,
        &vault,
        HookAdapter::Claude,
        AutomationEvent::Stop,
        host(HookAdapter::Claude, "Stop", "old-session", None, None),
    );
    assert_eq!(
        stop["decision"], "block",
        "a delayed Stop(A) must not be selected as B: {stop}"
    );
    assert!(stop.to_string().contains("ambiguous"), "{stop}");
}

#[test]
fn old_claude_session_only_stop_cannot_hide_unmapped_active_operation() {
    let (_temp, repo, vault) = project();
    let completed = establish(
        &repo,
        &vault,
        HookAdapter::Claude,
        "shared-session",
        None,
        "fix README alpha typo",
    );
    passing(&repo, &vault, &completed);
    complete_plan_for_identity(&repo, &vault, "README verified", &completed).unwrap();

    let active_without_hook_mapping = LifecycleIdentity::resolve(
        &vault.project_id,
        "fix README beta typo",
        SupportedAdapter::Claude,
        Some("shared-session"),
        Some("cli-created-operation"),
    )
    .unwrap();
    start_or_resume_plan_for_identity(
        &repo,
        &vault,
        "fix README beta typo",
        &active_without_hook_mapping,
    )
    .unwrap();

    let stop = deliver(
        &repo,
        &vault,
        HookAdapter::Claude,
        AutomationEvent::Stop,
        host(HookAdapter::Claude, "Stop", "shared-session", None, None),
    );
    assert_eq!(
        stop["decision"], "block",
        "session-only Stop must not resolve to a completed mapping while another operation is active: {stop}"
    );
    assert_ne!(stop["baron"]["reconciliation_passed"], true);
    assert!(stop.to_string().contains("ambiguous"), "{stop}");
}

#[test]
fn native_prompt_without_prompt_text_is_rejected() {
    let (_temp, repo, vault) = project();
    for adapter in [HookAdapter::Codex, HookAdapter::Claude] {
        let payload = host(
            adapter,
            "UserPromptSubmit",
            "s",
            Some("550e8400-e29b-41d4-a716-446655440000"),
            None,
        );
        let result = handle_hook(
            &repo,
            &vault,
            adapter,
            AutomationEvent::UserPromptSubmit,
            &payload.to_string(),
        );
        assert!(
            result.is_err(),
            "native Prompt must not invent task text: {result:?}"
        );
        assert!(!repo.join(MAP).exists());
    }
}

#[test]
fn active_frontmatter_corruption_blocks_cached_stop() {
    for corruption in ["request", "unsafe", "duplicate", "shared_path", "status"] {
        let (_temp, repo, vault) = project();
        let a = establish(
            &repo,
            &vault,
            HookAdapter::Codex,
            "s",
            Some("t"),
            "fix README typo",
        );
        passing(&repo, &vault, &a);
        let payload = host(HookAdapter::Codex, "Stop", "s", Some("t"), None);
        assert_eq!(
            deliver(
                &repo,
                &vault,
                HookAdapter::Codex,
                AutomationEvent::Stop,
                payload.clone()
            )["baron"]["reconciliation_passed"],
            true
        );
        let index = repo.join("docs/baron/plans/ACTIVE.md");
        let content = fs::read_to_string(&index).unwrap();
        let row = content
            .lines()
            .find(|line| line.starts_with("<!-- BARON:ACTIVE-PLAN "))
            .unwrap();
        let mut entry: Value = serde_json::from_str(
            row.strip_prefix("<!-- BARON:ACTIVE-PLAN ")
                .unwrap()
                .strip_suffix(" -->")
                .unwrap(),
        )
        .unwrap();
        if corruption == "completed" {
            complete_plan_for_identity(&repo, &vault, "README verified", &a).unwrap();
        } else if corruption == "request" {
            let path = repo.join(entry["plan_path"].as_str().unwrap());
            let text = fs::read_to_string(&path).unwrap();
            fs::write(path, text.replace("request_id: t", "request_id: foreign")).unwrap();
        } else {
            match corruption {
                "unsafe" => entry["plan_path"] = json!("../outside.md"),
                "status" => entry["status"] = json!("unrecognized"),
                "shared_path" => entry["operation_id"] = json!("op-other"),
                _ => {}
            }
            let changed = format!("<!-- BARON:ACTIVE-PLAN {} -->", entry);
            let output = if matches!(corruption, "duplicate" | "shared_path") {
                format!("{content}\n{changed}\n")
            } else {
                content.replace(row, &changed)
            };
            fs::write(index, output).unwrap();
        }
        let stop = deliver(
            &repo,
            &vault,
            HookAdapter::Codex,
            AutomationEvent::Stop,
            payload,
        );
        assert_eq!(stop["decision"], "block", "{corruption}: {stop}");
        assert_ne!(stop["baron"]["reconciliation_passed"], true);
    }
}

#[test]
fn completed_cleanup_requires_current_active_frontmatter_agreement_before_any_write() {
    let (_temp, repo, vault) = project();
    let a = establish(
        &repo,
        &vault,
        HookAdapter::Codex,
        "completed",
        Some("completed"),
        "fix README alpha typo",
    );
    passing(&repo, &vault, &a);
    complete_plan_for_identity(&repo, &vault, "README verified", &a).unwrap();
    let index = repo.join("docs/baron/plans/ACTIVE.md");
    let text = fs::read_to_string(&index).unwrap();
    fs::write(
        &index,
        text.replace("\"status\":\"completed\"", "\"status\":\"in_progress\""),
    )
    .unwrap();
    let map = fs::read(repo.join(MAP)).unwrap();
    let before = authority_side_effects(&repo, &vault);
    let result = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::UserPromptSubmit,
        &host(
            HookAdapter::Codex,
            "UserPromptSubmit",
            "next",
            Some("next"),
            Some("fix README beta typo"),
        )
        .to_string(),
    );
    assert!(
        result.is_err(),
        "stale completed ACTIVE row was pruned: {result:?}"
    );
    assert_eq!(map, fs::read(repo.join(MAP)).unwrap());
    assert_eq!(before, authority_side_effects(&repo, &vault));
}

#[test]
fn stop_for_exact_completed_operation_does_not_block_host_shutdown() {
    let (_temp, repo, vault) = project();
    let identity = establish(
        &repo,
        &vault,
        HookAdapter::Codex,
        "finished-session",
        Some("finished-turn"),
        "fix README and verify completion",
    );
    passing(&repo, &vault, &identity);
    complete_plan_for_identity(&repo, &vault, "README verification passed", &identity).unwrap();

    let stop = deliver(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::Stop,
        host(
            HookAdapter::Codex,
            "Stop",
            "finished-session",
            Some("finished-turn"),
            None,
        ),
    );

    assert_ne!(
        stop["decision"], "block",
        "completed Stop was blocked: {stop}"
    );
}

#[test]
fn stop_for_completed_operation_still_rejects_frontmatter_identity_mismatch() {
    let (_temp, repo, vault) = project();
    let identity = establish(
        &repo,
        &vault,
        HookAdapter::Codex,
        "finished-session",
        Some("finished-turn"),
        "fix README and verify completion",
    );
    passing(&repo, &vault, &identity);
    complete_plan_for_identity(&repo, &vault, "README verification passed", &identity).unwrap();

    let index = fs::read_to_string(repo.join("docs/baron/plans/ACTIVE.md")).unwrap();
    let row = index
        .lines()
        .find(|line| line.starts_with("<!-- BARON:ACTIVE-PLAN "))
        .unwrap();
    let entry: Value = serde_json::from_str(
        row.strip_prefix("<!-- BARON:ACTIVE-PLAN ")
            .unwrap()
            .strip_suffix(" -->")
            .unwrap(),
    )
    .unwrap();
    let plan_path = repo.join(entry["plan_path"].as_str().unwrap());
    let plan = fs::read_to_string(&plan_path).unwrap();
    fs::write(
        &plan_path,
        plan.replace("request_id: finished-turn", "request_id: foreign-turn"),
    )
    .unwrap();

    let stop = deliver(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::Stop,
        host(
            HookAdapter::Codex,
            "Stop",
            "finished-session",
            Some("finished-turn"),
            None,
        ),
    );

    assert_eq!(
        stop["decision"], "block",
        "mismatched completed authority was allowed: {stop}"
    );
}

#[test]
fn native_turn_aliases_normalize_intentionally_and_claude_does_not_use_codex_turn() {
    let (_temp, repo, vault) = project();
    let task = "fix README alpha typo";
    let first = deliver(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::UserPromptSubmit,
        json!({"sessionId":"s","requestId":"t","turn_id":"t","turnId":"t","prompt":task}),
    );
    let retry = deliver(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::UserPromptSubmit,
        json!({"session_id":"s","turn_id":"t","prompt":task}),
    );
    assert_eq!(first, retry);
    let claude = deliver(
        &repo,
        &vault,
        HookAdapter::Claude,
        AutomationEvent::UserPromptSubmit,
        json!({"session_id":"s","promptId":"550e8400-e29b-41d4-a716-446655440000","turn_id":"irrelevant-codex-field","prompt":task}),
    );
    assert_eq!(
        claude["baron"]["request_id"],
        "550e8400-e29b-41d4-a716-446655440000"
    );
    assert_ne!(
        claude["baron"]["operation_id"],
        first["baron"]["operation_id"]
    );
    let stable = deliver(
        &repo,
        &vault,
        HookAdapter::Claude,
        AutomationEvent::UserPromptSubmit,
        json!({"session_id":"s","prompt_id":"550e8400-e29b-41d4-a716-446655440000","prompt":task}),
    );
    assert_eq!(stable, claude);
}

#[test]
fn bounded_retention_preserves_live_and_pending_correlations_and_reclaims_completed() {
    let (_temp, repo, vault) = project();
    let a = establish(
        &repo,
        &vault,
        HookAdapter::Codex,
        "live",
        Some("live"),
        "fix README live typo",
    );
    let done = establish(
        &repo,
        &vault,
        HookAdapter::Codex,
        "done",
        Some("done"),
        "fix README done typo",
    );
    passing(&repo, &vault, &done);
    complete_plan_for_identity(&repo, &vault, "README verified", &done).unwrap();
    let mut map = load_map(&repo);
    let template = map["entries"][0].clone();
    for index in 0..254 {
        let identity = LifecycleIdentity::resolve(
            &vault.project_id,
            "pending task",
            SupportedAdapter::Codex,
            Some("pending"),
            Some(&format!("pending-{index}")),
        )
        .unwrap();
        let mut entry = template.clone();
        entry["identity"] = serde_json::to_value(identity).unwrap();
        entry["canonical_task"] = json!("pending task");
        entry["host_session_id"] = json!("pending");
        entry["host_turn_id"] = json!(format!("pending-{index}"));
        map["entries"].as_array_mut().unwrap().push(entry);
    }
    fs::write(repo.join(MAP), map.to_string()).unwrap();
    deliver(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::UserPromptSubmit,
        host(
            HookAdapter::Codex,
            "UserPromptSubmit",
            "new",
            Some("new"),
            Some("fix README new typo"),
        ),
    );
    let retained = load_map(&repo);
    assert!(retained["entries"].as_array().unwrap().len() <= 256);
    assert!(retained["entries"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["identity"]["operation_id"] == a.operation_id()));
    assert!(!retained["entries"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["identity"]["operation_id"] == done.operation_id()));
    let before = fs::read(repo.join(MAP)).unwrap();
    let overflow = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::UserPromptSubmit,
        &host(
            HookAdapter::Codex,
            "UserPromptSubmit",
            "overflow",
            Some("overflow"),
            Some("fix README overflow"),
        )
        .to_string(),
    );
    assert!(
        overflow.is_err(),
        "capacity must fail closed when only live/pending mappings remain"
    );
    assert_eq!(before, fs::read(repo.join(MAP)).unwrap());
}
