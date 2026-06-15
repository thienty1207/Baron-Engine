use std::fs;
use std::path::Path;

use baron_core::capability::{
    check_capabilities, register_provider, CapabilityProvider, CheckOptions, ProviderKind,
    Requirement,
};
use baron_core::config::AdapterKind;
use baron_core::context::{
    compile_context, compile_context_for_task, compile_context_why, ContextTarget,
};
use baron_core::memory::build_memory_index;
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn codex_context_bundle_includes_survey_memory_and_skipped_context() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("tomoty");
    let legacy = temp.path().join("legacy-crm");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&legacy).unwrap();
    write(&repo.join("AGENTS.md"), "# Agent Guide\n");
    write(
        &repo.join("package.json"),
        r#"{"scripts":{"build":"vite build","test":"vitest"}}"#,
    );
    write(&repo.join("src/main.tsx"), "console.log('app');\n");

    let tomoty_context = ensure_vault(&vault, &repo).unwrap();
    let legacy_context = ensure_vault(&vault, &legacy).unwrap();
    write(
        &tomoty_context.project_root.join("Facts.md"),
        "# Facts\n\n- Homepage context uses TomoTy design proof.\n",
    );
    write(
        &legacy_context.project_root.join("Facts.md"),
        "# Facts\n\n- Homepage context uses legacy CRM proof.\n",
    );
    build_memory_index(&tomoty_context).unwrap();

    let bundle = compile_context(&repo, &vault, ContextTarget::Codex).unwrap();

    assert!(bundle.contains("# Baron Context Bundle - Codex"));
    assert!(bundle.contains("## Project Atlas"));
    assert!(bundle.contains("Project type: `frontend`"));
    assert!(bundle.contains("AGENTS.md"));
    assert!(bundle.contains("npm run build"));
    assert!(bundle.contains("Homepage context uses TomoTy"));
    assert!(!bundle.contains("Homepage context uses legacy CRM"));
    assert!(bundle.contains("## Skipped Context"));
    assert!(bundle.contains("full Vault scan"));
    assert!(bundle.len() <= 20_000);
}

#[test]
fn why_output_explains_loaded_and_skipped_context() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("baron-demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    build_memory_index(&context).unwrap();

    let why = compile_context_why(&repo, &vault, ContextTarget::Generic).unwrap();

    assert!(why.contains("# Context Selection Why"));
    assert!(why.contains("Loaded: repo survey"));
    assert!(why.contains("Loaded: Memory Firewall Brief"));
    assert!(why.contains("Skipped: full Vault scan"));
    assert!(why.contains("Skipped: adapter refresh because init/update owns managed files"));
    assert!(!why.contains("Phase 3 only"));
}

#[test]
fn adapter_context_headings_are_distinct() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    build_memory_index(&context).unwrap();

    assert!(compile_context(&repo, &vault, ContextTarget::Codex)
        .unwrap()
        .contains("For Codex"));
    assert!(compile_context(&repo, &vault, ContextTarget::Claude)
        .unwrap()
        .contains("For Claude"));
    assert!(compile_context(&repo, &vault, ContextTarget::Generic)
        .unwrap()
        .contains("For generic agents"));
}

#[test]
fn task_focus_changes_risk_and_context_guidance() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    let bundle = compile_context_for_task(
        &repo,
        &vault,
        ContextTarget::Codex,
        Some("implement auth login and tenant permissions"),
    )
    .unwrap();

    assert!(bundle.contains("Task: `implement auth login and tenant permissions`"));
    assert!(bundle.contains("Risk lane: `high`"));
    assert!(bundle.contains("security evidence"));
}

#[test]
fn task_focus_selects_relevant_memory_instead_of_first_indexed_records() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    for index in 0..8 {
        write(
            &context
                .project_root
                .join("Notes")
                .join(format!("a-{index}.md")),
            &format!("# Note\n\n- Unrelated typography note number {index}.\n"),
        );
    }
    write(
        &context.project_root.join("Decisions.md"),
        "# Decisions\n\n- Supabase RLS tenant isolation protects customer records.\n",
    );

    let output = compile_context_for_task(
        &repo,
        &vault,
        ContextTarget::Codex,
        Some("bảo mật dữ liệu khách hàng"),
    )
    .unwrap();

    assert!(output.contains("Supabase RLS tenant isolation"));
    assert!(!output.contains("Unrelated typography note number 7"));
}

#[test]
fn context_loads_current_execution_state_without_loading_history() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    write(
        &repo.join("docs/superpowers/plans/CURRENT.md"),
        "# Current Plan\n\n- Status: in_progress\n- Next: verify context compiler\n",
    );
    write(
        &repo.join("docs/superpowers/plans/2026-06-01-old-plan.md"),
        "# Historical Plan\n\nThis body must not enter compact context.\n",
    );

    let bundle = compile_context(&repo, &vault, ContextTarget::Generic).unwrap();

    assert!(bundle.contains("Status: in_progress"));
    assert!(bundle.contains("Next: verify context compiler"));
    assert!(!bundle.contains("This body must not enter compact context"));
}

#[test]
fn context_stays_bounded_when_execution_state_is_large() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    write(
        &repo.join("docs/superpowers/plans/CURRENT.md"),
        &format!("# Current Plan\n\n{}", "large-state-line\n".repeat(5_000)),
    );

    let bundle = compile_context(&repo, &vault, ContextTarget::Codex).unwrap();

    assert!(bundle.chars().count() <= 20_000);
    assert!(bundle.matches("large-state-line").count() < 200);
    assert!(bundle.contains("## Skipped Context"));
}

#[test]
fn context_prefers_baron_plan_and_loads_bounded_execution_evidence() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join("docs/baron/plans")).unwrap();
    fs::create_dir_all(repo.join("docs/baron/harness")).unwrap();
    fs::create_dir_all(repo.join("docs/baron/proofs")).unwrap();
    fs::create_dir_all(repo.join("docs/baron/traces")).unwrap();
    write(
        &repo.join("docs/baron/plans/CURRENT.md"),
        "# Current Baron Plan\n\n- Title: auth login\n- Status: `in_progress`\n",
    );
    write(
        &repo.join("docs/baron/harness/CURRENT.md"),
        "# Current Product Harness\n\n- Title: auth login\n- Risk: `high`\n",
    );
    write(
        &repo.join("docs/baron/proofs/INDEX.md"),
        "# Baron Proof Index\n\n- latest auth tests passed\n",
    );
    write(
        &repo.join("docs/baron/traces/INDEX.md"),
        "# Baron Trace Index\n\n- latest detailed trace passed\n",
    );

    let bundle = compile_context(&repo, &vault, ContextTarget::Codex).unwrap();

    assert!(bundle.contains("docs/baron/plans/CURRENT.md"));
    assert!(bundle.contains("## Product Harness State"));
    assert!(bundle.contains("Risk: `high`"));
    assert!(bundle.contains("## Proof And Trace State"));
    assert!(bundle.contains("latest auth tests passed"));
    assert!(bundle.contains("latest detailed trace passed"));
}

#[test]
fn context_loads_bounded_cached_capability_summary_for_active_adapter() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    register_provider(
        &repo,
        CapabilityProvider {
            name: "git-cli".to_string(),
            capability: "source-control".to_string(),
            kind: ProviderKind::Cli,
            requirement: Requirement::Required,
            command: Some("git".to_string()),
            scan_target: None,
            adapters: Vec::new(),
            description: "Provides repository state and change evidence.".to_string(),
        },
    )
    .unwrap();
    check_capabilities(
        &repo,
        CheckOptions {
            adapter: AdapterKind::Codex,
            capability: None,
            allow_network: false,
        },
    )
    .unwrap();

    let bundle = compile_context(&repo, &vault, ContextTarget::Codex).unwrap();

    assert!(bundle.contains("## Capability Summary"));
    assert!(bundle.contains("source-control"));
    assert!(bundle.contains("git-cli"));
    assert!(bundle.contains("Presence: `present`"));
    assert!(bundle.contains("Presence does not prove execution"));
    assert!(bundle.chars().count() <= 20_000);
}

#[test]
fn context_why_explains_capability_cache_without_claiming_execution() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    register_provider(
        &repo,
        CapabilityProvider {
            name: "security-skill".to_string(),
            capability: "security-scan".to_string(),
            kind: ProviderKind::Skill,
            requirement: Requirement::Optional,
            command: None,
            scan_target: Some(".codex/skills/vibe-security-scan".to_string()),
            adapters: vec![AdapterKind::Codex],
            description: "Provides defensive repository security review guidance.".to_string(),
        },
    )
    .unwrap();

    let why = compile_context_why(&repo, &vault, ContextTarget::Codex).unwrap();

    assert!(why.contains("Capability Registry"));
    assert!(why.contains("presence cache"));
    assert!(why.contains("tool execution evidence"));
}
