use std::fs;

use baron_core::config::{initialize_project_with_options, AdapterKind, ProjectPlatform};
use baron_core::context::{compile_context_for_task, ContextTarget};
use baron_core::platform::{ensure_platform_intelligence, profile_for};
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

#[test]
fn every_public_platform_has_a_deep_baron_profile() {
    for platform in [
        ProjectPlatform::Frontend,
        ProjectPlatform::Backend,
        ProjectPlatform::Fullstack,
        ProjectPlatform::Mobile,
        ProjectPlatform::Desktop,
        ProjectPlatform::Tool,
        ProjectPlatform::Library,
        ProjectPlatform::Data,
        ProjectPlatform::Cloud,
        ProjectPlatform::Unknown,
    ] {
        let profile = profile_for(platform);
        assert!(!profile.product_concerns.is_empty(), "{platform:?}");
        assert!(!profile.architecture_priorities.is_empty(), "{platform:?}");
        assert!(!profile.failure_modes.is_empty(), "{platform:?}");
        assert!(!profile.security_expectations.is_empty(), "{platform:?}");
        assert!(!profile.performance_expectations.is_empty(), "{platform:?}");
        assert!(!profile.skill_routing.is_empty(), "{platform:?}");
        assert!(!profile.agent_routing.is_empty(), "{platform:?}");
        assert!(profile.verification_layers.len() >= 2, "{platform:?}");
        assert!(!profile.release_proof.is_empty(), "{platform:?}");
    }
}

#[test]
fn generated_profile_uses_repo_evidence_and_preserves_unknowns() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("app");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(
        repo.join("package.json"),
        r#"{"scripts":{"build":"next build","test":"vitest"},"dependencies":{"next":"15.0.0"}}"#,
    )
    .unwrap();
    fs::write(repo.join("next.config.ts"), "export default {};").unwrap();
    let config = initialize_project_with_options(
        &repo,
        Some(AdapterKind::Codex),
        &vault,
        Some(ProjectPlatform::Fullstack),
    )
    .unwrap();

    let report = ensure_platform_intelligence(&repo, &config).unwrap();

    assert!(report.project_profile.exists());
    assert!(report.stack_map.exists());
    assert!(report.quality_gates.exists());
    let stack = fs::read_to_string(report.stack_map).unwrap();
    assert!(stack.contains("Next.js"));
    assert!(stack.contains("npm run build"));
    assert!(stack.contains("Unknown"));
    assert!(!stack.contains("Supabase is configured"));
}

#[test]
fn compact_context_loads_only_task_relevant_active_profile() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("app");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let config = initialize_project_with_options(
        &repo,
        Some(AdapterKind::Codex),
        &vault,
        Some(ProjectPlatform::Fullstack),
    )
    .unwrap();
    ensure_platform_intelligence(&repo, &config).unwrap();
    ensure_vault(&vault, &repo).unwrap();

    let output = compile_context_for_task(
        &repo,
        &vault,
        ContextTarget::Codex,
        Some("implement backend API authorization"),
    )
    .unwrap();

    assert!(output.contains("## Platform Intelligence"));
    assert!(output.contains("fullstack"));
    assert!(output.contains("backend"));
    assert!(!output.contains("Desktop shell integration"));
    assert!(output.len() < 24_000);
}
