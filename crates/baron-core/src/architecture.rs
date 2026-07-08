use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::{ProjectConfig, ProjectPlatform};
use crate::platform::platform_name;

const START: &str = "<!-- baron:architecture:start -->";
const END: &str = "<!-- baron:architecture:end -->";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectureReport {
    pub current_architecture: PathBuf,
    pub project_structure: PathBuf,
    pub boundaries: PathBuf,
    pub dependency_rules: PathBuf,
    pub expansion_rules: PathBuf,
}

pub fn ensure_architecture_governor(
    repo_root: impl AsRef<Path>,
    config: &ProjectConfig,
) -> Result<ArchitectureReport> {
    let repo_root = repo_root.as_ref();
    let root = repo_root.join("docs/baron/architecture");
    let current_architecture = root.join("CURRENT_ARCHITECTURE.md");
    let project_structure = root.join("PROJECT_STRUCTURE.md");
    let boundaries = root.join("BOUNDARIES.md");
    let dependency_rules = root.join("DEPENDENCY_RULES.md");
    let expansion_rules = root.join("EXPANSION_RULES.md");
    let primary = config.platform.unwrap_or(ProjectPlatform::Unknown);
    let extensions = extension_names(&config.platform_extensions);
    let top_level = observed_top_level(repo_root)?;

    upsert(
        &current_architecture,
        &format!(
            "# Current Baron Architecture\n\n- Primary platform: `{}`\n- Extension platforms: {}\n- Architecture source: repo evidence plus explicit project configuration\n- Status: descriptive contract; Baron does not rearrange existing code automatically\n",
            platform_name(primary), extensions
        ),
    )?;
    upsert(
        &project_structure,
        &format!(
            "# Project Structure Contract\n\n## Observed Top-Level Paths\n{}\n\n## Adaptive Rule\n\n- Keep existing paths unless an approved migration plan, dry-run inventory, rollback path, and proof justify movement.\n- Recommendations describe responsibilities; they do not force framework-incompatible folder names.\n- New top-level modules need one responsibility, an owner, allowed dependencies, and a validation path.\n- Baron must not move existing paths automatically.\n\n## Suggested Responsibility Areas\n{}\n",
            markdown_list(&top_level), responsibility_areas(primary, &config.platform_extensions)
        ),
    )?;
    upsert(
        &boundaries,
        &render_boundaries(primary, &config.platform_extensions),
    )?;
    upsert(&dependency_rules, &render_dependency_rules())?;
    upsert(
        &expansion_rules,
        &render_expansion_rules(primary, &config.platform_extensions),
    )?;

    Ok(ArchitectureReport {
        current_architecture,
        project_structure,
        boundaries,
        dependency_rules,
        expansion_rules,
    })
}

pub fn render_architecture_context(repo_root: impl AsRef<Path>) -> String {
    let path = repo_root
        .as_ref()
        .join("docs/baron/architecture/CURRENT_ARCHITECTURE.md");
    if !path.is_file() {
        return String::new();
    }
    let content = fs::read_to_string(path).unwrap_or_default();
    format!(
        "## Architecture Governor\n\n{}\n",
        content.chars().take(1_800).collect::<String>()
    )
}

fn observed_top_level(repo_root: &Path) -> Result<Vec<String>> {
    let mut values = fs::read_dir(repo_root)?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            (!matches!(
                name.as_str(),
                ".git" | ".baron" | ".codex" | ".claude" | "target" | "node_modules"
            ) && name != "docs")
                .then_some(name)
        })
        .collect::<Vec<_>>();
    values.sort();
    Ok(values)
}

fn responsibility_areas(primary: ProjectPlatform, extensions: &[ProjectPlatform]) -> String {
    let mut areas = areas_for(primary).to_vec();
    for extension in extensions {
        for area in areas_for(*extension) {
            if !areas.contains(area) {
                areas.push(area);
            }
        }
    }
    areas
        .iter()
        .map(|area| format!("- {area}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn areas_for(platform: ProjectPlatform) -> &'static [&'static str] {
    match platform {
        ProjectPlatform::Frontend => &[
            "frontend product surfaces",
            "shared UI and design tokens",
            "browser tests",
        ],
        ProjectPlatform::Backend => &[
            "backend services and domain logic",
            "database/migrations",
            "API contracts and integration tests",
        ],
        ProjectPlatform::Fullstack => &[
            "frontend product surfaces",
            "backend services",
            "database/migrations",
            "shared API/data contracts",
            "infrastructure and end-to-end tests",
        ],
        ProjectPlatform::Mobile => &[
            "mobile applications",
            "shared mobile domain/client code",
            "backend contracts",
            "device/emulator tests",
        ],
        ProjectPlatform::Desktop => &[
            "desktop shell/UI",
            "cross-platform core",
            "OS adapters",
            "packaging/installers",
        ],
        ProjectPlatform::Tool => &[
            "reusable core",
            "CLI/automation adapters",
            "installers/releases",
            "contract fixtures",
        ],
        ProjectPlatform::Library => &[
            "public API",
            "private implementation",
            "consumer fixtures",
            "examples/docs",
        ],
        ProjectPlatform::Data => &[
            "schemas/contracts",
            "pipelines/jobs",
            "quality/lineage",
            "backfill/recovery",
        ],
        ProjectPlatform::Cloud => &[
            "infrastructure definitions",
            "services/functions",
            "policy/secrets",
            "deploy/rollback",
        ],
        ProjectPlatform::Unknown => {
            &["observed modules only; classify before adding a preferred structure"]
        }
    }
}

fn render_boundaries(primary: ProjectPlatform, extensions: &[ProjectPlatform]) -> String {
    format!(
        "# Architecture Boundaries\n\n- Primary owner: `{}`\n- Extensions: {}\n- UI clients may depend on declared contracts, never backend/database internals.\n- Backend owns authorization and transaction decisions.\n- Database migrations are versioned and reviewed with rollback/data-impact proof.\n- Shared contracts have one declared owner and compatibility tests.\n- Infrastructure depends on deployable interfaces, not application internals.\n",
        platform_name(primary), extension_names(extensions)
    )
}

fn render_dependency_rules() -> String {
    "# Dependency Rules\n\n- Dependencies point toward stable domain and declared contract layers.\n- Cross-platform clients share contracts, not UI or runtime-specific internals.\n- Cycles, duplicate contract ownership, and hidden database access are drift.\n- New dependencies require purpose, owner, security/license review, and verification.\n- Existing repository conventions remain authoritative unless a verified correction plan replaces them.\n".to_string()
}

fn render_expansion_rules(primary: ProjectPlatform, extensions: &[ProjectPlatform]) -> String {
    format!(
        "# Expansion Rules\n\n- Current primary: `{}`\n- Current extensions: {}\n- `baron init --<platform>` on an initialized project adds an extension; it does not replace the primary.\n- Reconcile shared contracts, auth, data ownership, deployment, and tests before adding code.\n- Existing files are never moved or deleted merely to match a suggested layout.\n- Structural migration requires plan, dry-run inventory, rollback, and proof.\n- Architecture drift produces a correction proposal; automatic destructive restructuring is forbidden.\n",
        platform_name(primary), extension_names(extensions)
    )
}

fn extension_names(values: &[ProjectPlatform]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values
            .iter()
            .map(|value| format!("`{}`", platform_name(*value)))
            .collect::<Vec<_>>()
            .join(", ")
    }
}
fn markdown_list(values: &[String]) -> String {
    if values.is_empty() {
        "- none observed".to_string()
    } else {
        values
            .iter()
            .map(|value| format!("- `{value}`"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn upsert(path: &Path, body: &str) -> Result<()> {
    let existing = fs::read_to_string(path).unwrap_or_default();
    let block = format!("{START}\n{}{END}", body.trim_end().to_string() + "\n");
    let content = match (existing.find(START), existing.find(END)) {
        (Some(start), Some(end)) if start < end => format!(
            "{}{}{}",
            &existing[..start],
            block,
            &existing[end + END.len()..]
        ),
        _ if existing.trim().is_empty() => format!("{block}\n"),
        _ => format!("{}\n\n{block}\n", existing.trim_end()),
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content).with_context(|| format!("Could not write {}", path.display()))
}
