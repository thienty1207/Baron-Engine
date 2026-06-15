use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Frontend,
    Backend,
    Fullstack,
    Tool,
    Desktop,
    Mobile,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurveyItem {
    pub label: String,
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurveyCommand {
    pub kind: String,
    pub command: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoSurvey {
    pub repo_path: String,
    pub repo_root: String,
    pub git_present: bool,
    pub git_root: Option<String>,
    pub project_type: ProjectType,
    pub stack_hints: Vec<SurveyItem>,
    pub entrypoints: Vec<SurveyItem>,
    pub commands: Vec<SurveyCommand>,
    pub risky_surfaces: Vec<SurveyItem>,
    pub read_first: Vec<String>,
    pub docs_needing_review: Vec<SurveyItem>,
    pub unknowns: Vec<String>,
}

pub fn survey_repository(path: impl AsRef<Path>) -> Result<RepoSurvey> {
    let repo_path = path.as_ref();
    let repo_root = repo_path
        .canonicalize()
        .with_context(|| format!("Could not resolve repo path: {}", repo_path.display()))?;
    let git_root = find_git_root(&repo_root);
    let git_present = git_root.is_some();
    let entries = collect_entries(&repo_root)?;
    let files: BTreeSet<String> = entries
        .iter()
        .filter(|entry| entry.is_file)
        .map(|entry| entry.relative_path.clone())
        .collect();
    let paths: BTreeSet<String> = entries
        .iter()
        .map(|entry| entry.relative_path.clone())
        .collect();

    let stack_hints = detect_stack_hints(&paths);
    let entrypoints = detect_entrypoints(&files);
    let commands = detect_commands(&repo_root, &files);
    let risky_surfaces = detect_risky_surfaces(&paths);
    let read_first = detect_read_first(&files);
    let docs_needing_review = detect_docs_needing_review(&files);
    let project_type = classify_project(&paths, &stack_hints, &entrypoints, &risky_surfaces);
    let unknowns = detect_unknowns(git_present, &commands);

    Ok(RepoSurvey {
        repo_path: normalize_path(&repo_root),
        repo_root: normalize_path(git_root.as_deref().unwrap_or(&repo_root)),
        git_present,
        git_root: git_root.as_ref().map(normalize_path),
        project_type,
        stack_hints,
        entrypoints,
        commands,
        risky_surfaces,
        read_first,
        docs_needing_review,
        unknowns,
    })
}

pub fn render_project_atlas(survey: &RepoSurvey) -> String {
    let mut output = String::new();
    output.push_str("# Project Atlas\n\n");
    output.push_str("## Overview\n\n");
    output.push_str(&format!("- Repo path: `{}`\n", survey.repo_path));
    output.push_str(&format!("- Repo root: `{}`\n", survey.repo_root));
    output.push_str(&format!(
        "- Git: {}\n",
        if survey.git_present {
            "detected"
        } else {
            "not detected"
        }
    ));
    output.push_str(&format!(
        "- Project type: `{}`\n\n",
        project_type_label(survey.project_type)
    ));

    push_items(&mut output, "Stack Hints", &survey.stack_hints);
    push_items(&mut output, "Entrypoints", &survey.entrypoints);
    push_commands(&mut output, &survey.commands);
    push_items(&mut output, "Risky Surfaces", &survey.risky_surfaces);
    push_strings(&mut output, "Read First", &survey.read_first);
    push_items(
        &mut output,
        "Docs Needing Review",
        &survey.docs_needing_review,
    );
    push_strings(&mut output, "Unknowns", &survey.unknowns);
    output.push_str("## Shadow Safety\n\n");
    output.push_str("- Survey mode is read-only.\n");
    output.push_str("- Project Atlas is printed to stdout in Phase 1.\n");
    output.push_str("- No files were written.\n");
    output
}

fn push_items(output: &mut String, title: &str, items: &[SurveyItem]) {
    output.push_str(&format!("## {}\n\n", title));
    if items.is_empty() {
        output.push_str("- none detected\n\n");
        return;
    }
    for item in items {
        output.push_str(&format!(
            "- {}: `{}` - {}\n",
            item.label, item.path, item.reason
        ));
    }
    output.push('\n');
}

fn push_commands(output: &mut String, commands: &[SurveyCommand]) {
    output.push_str("## Commands\n\n");
    if commands.is_empty() {
        output.push_str("- none detected\n\n");
        return;
    }
    for command in commands {
        output.push_str(&format!(
            "- {}: `{}` (source: `{}`)\n",
            command.kind, command.command, command.source
        ));
    }
    output.push('\n');
}

fn push_strings(output: &mut String, title: &str, items: &[String]) {
    output.push_str(&format!("## {}\n\n", title));
    if items.is_empty() {
        output.push_str("- none\n\n");
        return;
    }
    for item in items {
        output.push_str(&format!("- {}\n", item));
    }
    output.push('\n');
}

#[derive(Debug)]
struct Entry {
    relative_path: String,
    is_file: bool,
}

fn collect_entries(root: &Path) -> Result<Vec<Entry>> {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .parents(true);

    let mut entries = Vec::new();
    for result in builder
        .filter_entry(|entry| !is_heavy_path(entry.path()))
        .build()
    {
        let entry = match result {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if path == root {
            continue;
        }
        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };
        let relative_path = normalize_relative(relative);
        if relative_path.is_empty() {
            continue;
        }
        entries.push(Entry {
            relative_path,
            is_file: entry
                .file_type()
                .map(|file_type| file_type.is_file())
                .unwrap_or(false),
        });
    }
    Ok(entries)
}

fn is_heavy_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(|name| {
            matches!(
                name,
                ".git"
                    | "node_modules"
                    | "target"
                    | "dist"
                    | "build"
                    | ".next"
                    | ".cache"
                    | "vendor"
            )
        })
        .unwrap_or(false)
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    for candidate in start.ancestors() {
        if candidate.join(".git").exists() {
            return Some(candidate.to_path_buf());
        }
    }
    None
}

fn detect_stack_hints(paths: &BTreeSet<String>) -> Vec<SurveyItem> {
    let mut items = Vec::new();
    add_if_path(
        paths,
        &mut items,
        "package.json",
        "Node package",
        "JavaScript/TypeScript package manifest",
    );
    add_if_path(
        paths,
        &mut items,
        "Cargo.toml",
        "Rust crate",
        "Rust workspace or crate manifest",
    );
    add_if_path(
        paths,
        &mut items,
        "go.mod",
        "Go module",
        "Go module manifest",
    );
    add_if_path(
        paths,
        &mut items,
        "pyproject.toml",
        "Python project",
        "Python project manifest",
    );
    add_if_path(
        paths,
        &mut items,
        "Dockerfile",
        "Docker",
        "Container build file",
    );
    add_if_any(
        paths,
        &mut items,
        "Next.js",
        "Next.js config or app router surface",
        |path| {
            path == "next.config.js"
                || path == "next.config.mjs"
                || path == "next.config.ts"
                || path.starts_with("app/")
        },
    );
    add_if_any(paths, &mut items, "Vite", "Vite config detected", |path| {
        path == "vite.config.js" || path == "vite.config.ts" || path == "vite.config.mjs"
    });
    add_if_any(
        paths,
        &mut items,
        "Supabase",
        "Supabase workspace detected",
        |path| path == "supabase" || path.starts_with("supabase/"),
    );
    add_if_any(
        paths,
        &mut items,
        "GitHub Actions",
        "CI workflow folder detected",
        |path| path == ".github/workflows" || path.starts_with(".github/workflows/"),
    );
    add_if_any(
        paths,
        &mut items,
        "Database migrations",
        "Migration folder detected",
        |path| path.starts_with("migrations/") || path.contains("/migrations/"),
    );
    items
}

fn detect_entrypoints(files: &BTreeSet<String>) -> Vec<SurveyItem> {
    let candidates = [
        (
            "src/main.tsx",
            "Frontend entrypoint",
            "React or browser app entrypoint",
        ),
        (
            "src/main.ts",
            "TypeScript entrypoint",
            "TypeScript application entrypoint",
        ),
        ("src/App.tsx", "Frontend app shell", "React app shell"),
        ("src/main.rs", "Rust binary entrypoint", "Rust main binary"),
        (
            "src/lib.rs",
            "Rust library entrypoint",
            "Rust library surface",
        ),
        ("main.go", "Go entrypoint", "Go main package"),
        ("cmd/main.go", "Go command entrypoint", "Go command package"),
        ("main.py", "Python entrypoint", "Python main module"),
        ("app.py", "Python app entrypoint", "Python app module"),
    ];
    candidates
        .iter()
        .filter(|(path, _, _)| files.contains(*path))
        .map(|(path, label, reason)| item(*label, *path, *reason))
        .collect()
}

fn detect_commands(root: &Path, files: &BTreeSet<String>) -> Vec<SurveyCommand> {
    let mut commands = Vec::new();
    if files.contains("package.json") {
        commands.extend(package_commands(root.join("package.json")));
    }
    if files.contains("Cargo.toml") {
        commands.push(command("build", "cargo build", "Cargo.toml"));
        commands.push(command("test", "cargo test", "Cargo.toml"));
    }
    if files.contains("go.mod") {
        commands.push(command("build", "go build ./...", "go.mod"));
        commands.push(command("test", "go test ./...", "go.mod"));
    }
    if files.contains("pyproject.toml") {
        commands.push(command("test", "pytest", "pyproject.toml"));
    }
    dedupe_commands(commands)
}

fn package_commands(path: PathBuf) -> Vec<SurveyCommand> {
    let Ok(content) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return Vec::new();
    };
    let scripts = json.get("scripts").and_then(|value| value.as_object());
    let mut commands = Vec::new();
    if scripts.and_then(|scripts| scripts.get("build")).is_some() {
        commands.push(command("build", "npm run build", "package.json"));
    }
    if scripts.and_then(|scripts| scripts.get("test")).is_some() {
        commands.push(command("test", "npm test", "package.json"));
    }
    if scripts.and_then(|scripts| scripts.get("dev")).is_some() {
        commands.push(command("dev", "npm run dev", "package.json"));
    }
    commands
}

fn detect_risky_surfaces(paths: &BTreeSet<String>) -> Vec<SurveyItem> {
    let mut items = Vec::new();
    add_risk(
        paths,
        &mut items,
        "Auth or session surface",
        ["auth", "login", "session", "jwt", "token"],
    );
    add_risk(
        paths,
        &mut items,
        "Payment or billing surface",
        ["payment", "billing", "subscription", "checkout"],
    );
    add_risk(
        paths,
        &mut items,
        "Upload or file handling surface",
        ["upload", "multipart", "storage"],
    );
    add_risk(
        paths,
        &mut items,
        "Migration or data model surface",
        ["migration", "migrations", "schema"],
    );
    add_risk(
        paths,
        &mut items,
        "Secret or environment surface",
        [".env", "secret", "credential"],
    );
    items
}

fn detect_read_first(files: &BTreeSet<String>) -> Vec<String> {
    [
        "AGENTS.md",
        "CLAUDE.md",
        "README.md",
        "docs/BARON_STATUS.md",
        "package.json",
        "Cargo.toml",
        "go.mod",
        "pyproject.toml",
    ]
    .iter()
    .filter(|path| files.contains(**path))
    .map(|path| path.to_string())
    .collect()
}

fn detect_docs_needing_review(files: &BTreeSet<String>) -> Vec<SurveyItem> {
    let mut docs = Vec::new();
    if files.contains("README.md") {
        docs.push(item(
            "README",
            "README.md",
            "Useful orientation, but validate freshness before treating as truth",
        ));
    }
    for path in files
        .iter()
        .filter(|path| path.starts_with("docs/"))
        .take(10)
    {
        docs.push(item(
            "Project docs",
            path,
            "Review freshness before relying on this doc",
        ));
    }
    docs
}

fn classify_project(
    paths: &BTreeSet<String>,
    stack_hints: &[SurveyItem],
    entrypoints: &[SurveyItem],
    risky_surfaces: &[SurveyItem],
) -> ProjectType {
    let has_frontend = has_label(stack_hints, "Next.js")
        || has_label(stack_hints, "Vite")
        || paths.contains("index.html")
        || paths
            .iter()
            .any(|path| path.starts_with("app/") || path.starts_with("pages/"))
        || entrypoints
            .iter()
            .any(|item| item.label.contains("Frontend"));
    let has_backend = has_label(stack_hints, "Rust crate")
        || has_label(stack_hints, "Go module")
        || has_label(stack_hints, "Python project")
        || has_label(stack_hints, "Database migrations")
        || paths
            .iter()
            .any(|path| path.starts_with("api/") || path.contains("/api/"))
        || risky_surfaces
            .iter()
            .any(|item| item.label.contains("Migration"));
    let has_desktop = paths.iter().any(|path| {
        path.starts_with("src-tauri/") || path == "tauri.conf.json" || path.contains("electron")
    });
    let has_mobile = paths.iter().any(|path| {
        path.starts_with("android/") || path.starts_with("ios/") || path == "react-native.config.js"
    });
    let has_tool = has_label(stack_hints, "Rust crate")
        || has_label(stack_hints, "Go module")
        || paths
            .iter()
            .any(|path| path.starts_with("bin/") || path.starts_with("scripts/"));

    if has_mobile {
        ProjectType::Mobile
    } else if has_desktop {
        ProjectType::Desktop
    } else if has_frontend && has_backend {
        ProjectType::Fullstack
    } else if has_frontend {
        ProjectType::Frontend
    } else if has_backend {
        ProjectType::Backend
    } else if has_tool {
        ProjectType::Tool
    } else {
        ProjectType::Unknown
    }
}

fn detect_unknowns(git_present: bool, commands: &[SurveyCommand]) -> Vec<String> {
    let mut unknowns = Vec::new();
    if !git_present {
        unknowns.push("No git repository detected".to_string());
    }
    if !commands.iter().any(|command| command.kind == "build") {
        unknowns.push("No build command detected".to_string());
    }
    if !commands.iter().any(|command| command.kind == "test") {
        unknowns.push("No test command detected".to_string());
    }
    unknowns
}

fn add_if_path(
    paths: &BTreeSet<String>,
    items: &mut Vec<SurveyItem>,
    path: &str,
    label: &str,
    reason: &str,
) {
    if paths.contains(path) {
        items.push(item(label, path, reason));
    }
}

fn add_if_any(
    paths: &BTreeSet<String>,
    items: &mut Vec<SurveyItem>,
    label: &str,
    reason: &str,
    predicate: impl Fn(&str) -> bool,
) {
    if let Some(path) = paths.iter().find(|path| predicate(path)) {
        items.push(item(label, path, reason));
    }
}

fn add_risk<const N: usize>(
    paths: &BTreeSet<String>,
    items: &mut Vec<SurveyItem>,
    label: &str,
    needles: [&str; N],
) {
    if let Some(path) = paths.iter().find(|path| {
        let lower = path.to_lowercase();
        needles.iter().any(|needle| lower.contains(needle))
    }) {
        items.push(item(label, path, "High-care area for AI-assisted changes"));
    }
}

fn item(
    label: impl Into<String>,
    path: impl Into<String>,
    reason: impl Into<String>,
) -> SurveyItem {
    SurveyItem {
        label: label.into(),
        path: path.into(),
        reason: reason.into(),
    }
}

fn command(
    kind: impl Into<String>,
    command: impl Into<String>,
    source: impl Into<String>,
) -> SurveyCommand {
    SurveyCommand {
        kind: kind.into(),
        command: command.into(),
        source: source.into(),
    }
}

fn dedupe_commands(commands: Vec<SurveyCommand>) -> Vec<SurveyCommand> {
    let mut seen = BTreeSet::new();
    commands
        .into_iter()
        .filter(|command| {
            seen.insert((
                command.kind.clone(),
                command.command.clone(),
                command.source.clone(),
            ))
        })
        .collect()
}

fn has_label(items: &[SurveyItem], label: &str) -> bool {
    items.iter().any(|item| item.label == label)
}

fn project_type_label(project_type: ProjectType) -> &'static str {
    match project_type {
        ProjectType::Frontend => "frontend",
        ProjectType::Backend => "backend",
        ProjectType::Fullstack => "fullstack",
        ProjectType::Tool => "tool",
        ProjectType::Desktop => "desktop",
        ProjectType::Mobile => "mobile",
        ProjectType::Unknown => "unknown",
    }
}

fn normalize_path(path: impl AsRef<Path>) -> String {
    let normalized = path.as_ref().to_string_lossy().replace('\\', "/");
    normalized
        .strip_prefix("//?/")
        .unwrap_or(&normalized)
        .to_string()
}

fn normalize_relative(path: &Path) -> String {
    normalize_path(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_required_markdown_sections() {
        let survey = RepoSurvey {
            repo_path: "repo".to_string(),
            repo_root: "repo".to_string(),
            git_present: false,
            git_root: None,
            project_type: ProjectType::Unknown,
            stack_hints: Vec::new(),
            entrypoints: Vec::new(),
            commands: Vec::new(),
            risky_surfaces: Vec::new(),
            read_first: Vec::new(),
            docs_needing_review: Vec::new(),
            unknowns: vec!["No build command detected".to_string()],
        };
        let markdown = render_project_atlas(&survey);
        assert!(markdown.contains("## Overview"));
        assert!(markdown.contains("## Stack Hints"));
        assert!(markdown.contains("## Shadow Safety"));
    }
}
