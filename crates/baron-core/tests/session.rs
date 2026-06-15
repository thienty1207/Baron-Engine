use std::fs;
use std::sync::Mutex;

use baron_core::context::{compile_context, ContextTarget};
use baron_core::firewall::recall;
use baron_core::memory::build_memory_index;
use baron_core::session::import_sessions;
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn imports_matched_codex_and_claude_sessions_with_redaction_and_deduplication() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let other = temp.path().join("other");
    let vault = temp.path().join("Vault");
    let codex = temp.path().join("codex-sessions");
    let claude = temp.path().join("claude-sessions");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&other).unwrap();
    fs::create_dir_all(&codex).unwrap();
    fs::create_dir_all(&claude).unwrap();
    let repo_text = repo.canonicalize().unwrap().to_string_lossy().to_string();
    let other_text = other.canonicalize().unwrap().to_string_lossy().to_string();
    fs::write(
        codex.join("matched.jsonl"),
        format!(
            "{}\n{}\n{}\n{}\n",
            serde_json::json!({"type":"session_meta","payload":{"cwd":repo_text}}),
            serde_json::json!({"type":"response_item","payload":{"role":"user","content":[{"type":"input_text","text":"Review auth token=supersecret123"}]}}),
            serde_json::json!({"type":"response_item","payload":{"role":"assistant","content":[{"type":"output_text","text":"Decision: use Supabase RLS tenant isolation."}]}}),
            serde_json::json!({"type":"tool_call","name":"shell","arguments":"secret noise"})
        ),
    )
    .unwrap();
    fs::write(
        claude.join("matched.jsonl"),
        format!(
            "{}\n{}\n{}\n",
            serde_json::json!({"type":"user","cwd":repo_text,"message":{"role":"user","content":"Need a customer data security review"}}),
            serde_json::json!({"type":"assistant","message":{"role":"assistant","content":"Next action: verify RLS policies."}}),
            serde_json::json!({"type":"system","message":"developer prompt must not be imported"})
        ),
    )
    .unwrap();
    fs::write(
        codex.join("unmatched.jsonl"),
        format!(
            "{}\n{}",
            serde_json::json!({"type":"session_meta","payload":{"cwd":other_text}}),
            serde_json::json!({"type":"response_item","payload":{"role":"user","content":"Other project secret"}})
        ),
    )
    .unwrap();
    std::env::set_var("BARON_CODEX_SESSIONS_ROOT", &codex);
    std::env::set_var("BARON_CLAUDE_SESSIONS_ROOT", &claude);

    let context = ensure_vault(&vault, &repo).unwrap();
    let first = import_sessions(&repo, &context, 20).unwrap();
    let second = import_sessions(&repo, &context, 20).unwrap();
    let imported = fs::read_dir(context.project_root.join("Sessions/Imported"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    let combined = imported
        .iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    std::env::remove_var("BARON_CODEX_SESSIONS_ROOT");
    std::env::remove_var("BARON_CLAUDE_SESSIONS_ROOT");

    assert_eq!(first.imported, 2);
    assert_eq!(second.imported, 0);
    assert!(second.deduplicated >= 2);
    assert!(combined.contains("Supabase RLS tenant isolation"));
    assert!(combined.contains("Need a customer data security review"));
    assert!(combined.contains("[REDACTED]"));
    assert!(!combined.contains("supersecret123"));
    assert!(!combined.contains("tool_call"));
    assert!(!combined.contains("developer prompt"));
    assert!(!combined.contains("Other project secret"));

    build_memory_index(&context).unwrap();
    assert!(recall(&context, "bảo mật dữ liệu khách hàng", 5)
        .unwrap()
        .results
        .iter()
        .any(|hit| hit.record.kind.as_str() == "session"));
}

#[test]
fn context_automatically_imports_matched_sessions() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    let codex = temp.path().join("sessions");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&codex).unwrap();
    let repo_text = repo.canonicalize().unwrap().to_string_lossy().to_string();
    fs::write(
        codex.join("session.jsonl"),
        format!(
            "{}\n{}",
            serde_json::json!({"type":"session_meta","payload":{"cwd":repo_text}}),
            serde_json::json!({"type":"response_item","payload":{"role":"user","content":"Remember the durable session decision"}})
        ),
    )
    .unwrap();
    std::env::set_var("BARON_CODEX_SESSIONS_ROOT", &codex);
    std::env::set_var(
        "BARON_CLAUDE_SESSIONS_ROOT",
        temp.path().join("missing-claude"),
    );

    let output = compile_context(&repo, &vault, ContextTarget::Codex).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    std::env::remove_var("BARON_CODEX_SESSIONS_ROOT");
    std::env::remove_var("BARON_CLAUDE_SESSIONS_ROOT");

    assert!(context.project_root.join("Sessions/Imported").exists());
    assert!(output.contains("durable session decision"));
}
