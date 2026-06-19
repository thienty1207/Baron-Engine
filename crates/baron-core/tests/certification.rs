use std::fs;
use std::path::Path;

use baron_core::certification::{
    latest_certification_status, run_certification, CertificationProfile,
};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::vault::vault_context_without_create;
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn certification_proves_scale_isolation_and_cache_recovery() {
    let temp = tempdir().unwrap();
    let vault = temp.path().join("Vault");
    let repo = temp.path().join("main").join("same-name");
    let second_repo = temp.path().join("other").join("same-name");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&second_repo).unwrap();

    write(
        &repo.join("package.json"),
        r#"{"scripts":{"build":"vite build","test":"vitest"}}"#,
    );
    write(
        &repo.join("src/auth/login.ts"),
        "export const login = true;\n",
    );
    for index in 0..1_200 {
        write(
            &repo.join(format!("src/modules/module-{index}.ts")),
            &format!("export const value{index} = {index};\n"),
        );
    }
    for index in 0..180 {
        write(
            &repo.join(format!("docs/history/2020/week-{index:03}.md")),
            &format!("# Week {index}\n\n- Verified historical auth proof {index}.\n"),
        );
    }

    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    initialize_project(&second_repo, AdapterKind::Codex, &vault).unwrap();
    let context = vault_context_without_create(&vault, &repo).unwrap();
    let other_context = vault_context_without_create(&vault, &second_repo).unwrap();
    write(
        &context.project_root.join("Facts.md"),
        "# Facts\n\n- Current project auth memory is verified and must rank first.\n",
    );
    write(
        &other_context.project_root.join("Facts.md"),
        "# Facts\n\n- Other project auth memory must not contaminate current recall.\n",
    );
    write(&context.index_path, "not a sqlite database");

    let report = run_certification(&repo, &vault, CertificationProfile::Smoke).unwrap();

    assert!(report.passed, "{:#?}", report);
    assert!(report
        .checks
        .iter()
        .any(|check| check.id == "large-repo-survey" && check.passed));
    assert!(report
        .checks
        .iter()
        .any(|check| check.id == "shared-vault-firewall" && check.passed));
    assert!(report
        .checks
        .iter()
        .any(|check| check.id == "cache-corruption-recovery" && check.passed));
    assert!(report
        .checks
        .iter()
        .any(|check| check.id == "release-readiness" && check.passed));
    assert!(report
        .checks
        .iter()
        .any(|check| check.id == "autopilot-readiness" && check.passed));
    assert!(report
        .checks
        .iter()
        .any(|check| check.id == "runtime-backend-policy" && check.passed));
    assert!(report.markdown_path.is_file());
    assert!(report.json_path.is_file());

    let markdown = fs::read_to_string(&report.markdown_path).unwrap();
    assert!(markdown.contains("# Baron Certification"));
    assert!(markdown.contains("Memory firewall"));
    assert!(markdown.contains("Release readiness"));
    assert!(markdown.contains("autopilot"));
    assert!(markdown.contains("runtime"));

    let status = latest_certification_status(&repo).unwrap();
    assert!(status.contains("latest certification passed"));
    assert!(status.contains("3.1.1"));
}
