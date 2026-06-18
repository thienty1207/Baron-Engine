use std::fs;
use std::path::Path;

use baron_core::asset_lifecycle::{
    audit_runtime_assets, quarantine_failing_assets, stage_skill_update,
};
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn audit_flags_external_runtime_links_and_thin_skills() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    write(
        &repo.join(".codex/skills/superpowers/SKILL.md"),
        "---\nname: superpowers\ndescription: Use when planning TDD debugging review verification.\n---\n\n# Superpowers\n\nSuperpowers is the workflow core. Proof and trace are required.\n",
    );
    write(
        &repo.join(".codex/skills/api-and-interface-design/SKILL.md"),
        "---\nname: api-and-interface-design\ndescription: Use when API work.\n---\n\nRead https://github.com/example/skill and design APIs.\n",
    );
    write(
        &repo.join(".codex/agents/code-reviewer.toml"),
        "name = \"code-reviewer\"\ndescription = \"Core review gate.\"\ndeveloper_instructions = \"\"\"\nEvidence only. Do not invoke other subagents. Proof and trace required.\n\"\"\"\n",
    );

    let report = audit_runtime_assets(repo).unwrap();

    assert!(!report.passed);
    assert!(report
        .items
        .iter()
        .any(|item| item.name == "api-and-interface-design" && item.external_runtime_link));
    assert!(report
        .items
        .iter()
        .any(|item| item.name == "api-and-interface-design" && item.thin));
}

#[test]
fn quarantine_moves_failing_custom_assets_without_touching_superpowers() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    write(
        &repo.join(".codex/skills/superpowers/SKILL.md"),
        "---\nname: superpowers\ndescription: Use when planning TDD debugging review verification.\n---\n\n# Superpowers\n\nSuperpowers is the workflow core. Proof and trace are required.\n",
    );
    write(
        &repo.join(".codex/skills/weak-custom/SKILL.md"),
        "---\nname: weak-custom\ndescription: Use when weak.\n---\n\nRead https://github.com/example/weak and do it.\n",
    );

    let result = quarantine_failing_assets(repo).unwrap();

    assert_eq!(result.quarantined.len(), 1);
    assert!(!repo.join(".codex/skills/weak-custom/SKILL.md").exists());
    assert!(repo.join(".codex/skills/superpowers/SKILL.md").exists());
    assert!(repo.join(".baron/quarantine/asset-lifecycle").exists());
}

#[test]
fn staged_skill_update_requires_human_review_and_does_not_overwrite_runtime_skill() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let runtime_path = repo.join(".codex/skills/rust-api/SKILL.md");
    write(
        &runtime_path,
        "---\nname: rust-api\ndescription: Use when Rust API work.\n---\n\n# Rust API\n\nExisting runtime skill.\n",
    );

    let staged = stage_skill_update(
        repo,
        "rust-api",
        "Add Axum extractor validation guidance",
        "---\nname: rust-api\ndescription: Use when Rust API work.\n---\n\n# Rust API\n\nNew proposal.\n",
    )
    .unwrap();

    let runtime = fs::read_to_string(runtime_path).unwrap();
    assert!(runtime.contains("Existing runtime skill"));
    assert!(staged.proposal_path.exists());
    assert!(staged.diff_path.exists());
    let metadata = fs::read_to_string(staged.metadata_path).unwrap();
    assert!(metadata.contains("\"status\": \"proposed\""));
    assert!(metadata.contains("\"approvalRequired\": true"));
}
