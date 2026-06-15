use std::fs;

use baron_core::survey::{survey_repository, ProjectType};
use tempfile::tempdir;

fn write(path: &std::path::Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn detects_stack_hints_from_common_project_files() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    write(
        &root.join("package.json"),
        r#"{"scripts":{"build":"next build","test":"vitest"},"dependencies":{"next":"latest"}}"#,
    );
    write(
        &root.join("Cargo.toml"),
        "[package]\nname = \"api\"\nversion = \"0.1.0\"\n",
    );
    write(&root.join("go.mod"), "module example.com/api\n");
    write(
        &root.join("pyproject.toml"),
        "[project]\nname = \"worker\"\n",
    );
    write(&root.join("Dockerfile"), "FROM rust:1\n");
    write(&root.join("next.config.js"), "module.exports = {}\n");
    fs::create_dir_all(root.join("supabase")).unwrap();
    fs::create_dir_all(root.join(".github/workflows")).unwrap();

    let survey = survey_repository(root).unwrap();
    let hints: Vec<_> = survey
        .stack_hints
        .iter()
        .map(|item| item.label.as_str())
        .collect();

    assert!(hints.contains(&"Node package"));
    assert!(hints.contains(&"Rust crate"));
    assert!(hints.contains(&"Go module"));
    assert!(hints.contains(&"Python project"));
    assert!(hints.contains(&"Docker"));
    assert!(hints.contains(&"Next.js"));
    assert!(hints.contains(&"Supabase"));
    assert!(hints.contains(&"GitHub Actions"));
}

#[test]
fn classifies_mixed_frontend_backend_fixture_as_fullstack() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    write(
        &root.join("package.json"),
        r#"{"scripts":{"dev":"vite --host 0.0.0.0","test":"vitest"}}"#,
    );
    write(&root.join("vite.config.ts"), "export default {}\n");
    write(
        &root.join("src/pages/index.tsx"),
        "export default function Home() {}\n",
    );
    write(
        &root.join("Cargo.toml"),
        "[package]\nname = \"backend\"\nversion = \"0.1.0\"\n",
    );
    write(&root.join("src/main.rs"), "fn main() {}\n");

    let survey = survey_repository(root).unwrap();

    assert_eq!(survey.project_type, ProjectType::Fullstack);
}

#[test]
fn detects_security_and_data_risky_surfaces() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    write(&root.join("src/auth/login.rs"), "pub fn login() {}\n");
    write(
        &root.join("src/payment/checkout.rs"),
        "pub fn charge() {}\n",
    );
    write(&root.join("src/upload/file.rs"), "pub fn upload() {}\n");
    write(
        &root.join("migrations/001_create_users.sql"),
        "create table users(id int);\n",
    );

    let survey = survey_repository(root).unwrap();
    let labels: Vec<_> = survey
        .risky_surfaces
        .iter()
        .map(|item| item.label.as_str())
        .collect();

    assert!(labels.contains(&"Auth or session surface"));
    assert!(labels.contains(&"Payment or billing surface"));
    assert!(labels.contains(&"Upload or file handling surface"));
    assert!(labels.contains(&"Migration or data model surface"));
}

#[test]
fn missing_build_and_test_commands_are_unknowns_not_guesses() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    write(&root.join("README.md"), "# legacy project\n");

    let survey = survey_repository(root).unwrap();

    assert!(survey.commands.is_empty());
    assert!(survey
        .unknowns
        .iter()
        .any(|item| item == "No build command detected"));
    assert!(survey
        .unknowns
        .iter()
        .any(|item| item == "No test command detected"));
}

#[test]
fn survey_does_not_hide_risky_paths_after_six_thousand_entries() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("large-repo");
    fs::create_dir_all(repo.join("src/generated")).unwrap();
    for index in 0..6_050 {
        fs::write(
            repo.join("src/generated").join(format!("{index:05}.txt")),
            "generated",
        )
        .unwrap();
    }
    fs::create_dir_all(repo.join("zzz/security")).unwrap();
    fs::write(repo.join("zzz/security/payment-upload.rs"), "fn main() {}").unwrap();

    let survey = survey_repository(&repo).unwrap();

    assert!(survey
        .risky_surfaces
        .iter()
        .any(|surface| surface.path.contains("payment-upload.rs")));
}
