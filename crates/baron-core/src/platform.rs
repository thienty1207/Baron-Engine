use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::{ProjectConfig, ProjectPlatform};
use crate::survey::survey_repository;

const START: &str = "<!-- baron:platform:start -->";
const END: &str = "<!-- baron:platform:end -->";

#[derive(Debug, Clone, Copy)]
pub struct PlatformProfile {
    pub platform: ProjectPlatform,
    pub product_concerns: &'static [&'static str],
    pub architecture_priorities: &'static [&'static str],
    pub failure_modes: &'static [&'static str],
    pub security_expectations: &'static [&'static str],
    pub performance_expectations: &'static [&'static str],
    pub skill_routing: &'static [&'static str],
    pub agent_routing: &'static [&'static str],
    pub verification_layers: &'static [&'static str],
    pub release_proof: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformReport {
    pub project_profile: PathBuf,
    pub stack_map: PathBuf,
    pub guidelines: PathBuf,
    pub quality_gates: PathBuf,
    pub profile_files: Vec<PathBuf>,
}

pub fn profile_for(platform: ProjectPlatform) -> PlatformProfile {
    match platform {
        ProjectPlatform::Frontend => profile(platform, FRONTEND),
        ProjectPlatform::Backend => profile(platform, BACKEND),
        ProjectPlatform::Fullstack => profile(platform, FULLSTACK),
        ProjectPlatform::Mobile => profile(platform, MOBILE),
        ProjectPlatform::Desktop => profile(platform, DESKTOP),
        ProjectPlatform::Tool => profile(platform, TOOL),
        ProjectPlatform::Library => profile(platform, LIBRARY),
        ProjectPlatform::Data => profile(platform, DATA),
        ProjectPlatform::Cloud => profile(platform, CLOUD),
        ProjectPlatform::Unknown => profile(platform, UNKNOWN),
    }
}

pub fn ensure_platform_intelligence(
    repo_root: impl AsRef<Path>,
    config: &ProjectConfig,
) -> Result<PlatformReport> {
    let repo_root = repo_root.as_ref();
    let survey = survey_repository(repo_root)?;
    let platforms = active_platforms(config);
    let root = repo_root.join("docs/baron/platform");
    let project_profile = root.join("PROJECT_PROFILE.md");
    let stack_map = root.join("STACK_MAP.md");
    let guidelines = root.join("ENGINEERING_GUIDELINES.md");
    let quality_gates = root.join("QUALITY_GATES.md");

    upsert_managed(
        &project_profile,
        &format!(
            "# Baron Project Profile\n\n- Primary platform: `{}`\n- Extension platforms: {}\n- Survey project type: `{:?}`\n- Rule: repo evidence and explicit project rules override generic profile guidance.\n- Unknown facts remain unknown.\n",
            platform_name(platforms[0]),
            extension_names(&platforms),
            survey.project_type
        ),
    )?;
    upsert_managed(&stack_map, &render_stack_map(&survey))?;
    upsert_managed(&guidelines, &render_guidelines(&platforms))?;
    upsert_managed(&quality_gates, &render_quality_gates(&platforms))?;

    let mut profile_files = Vec::new();
    for platform in platforms {
        let path = root
            .join("profiles")
            .join(format!("{}.md", platform_name(platform)));
        upsert_managed(&path, &render_profile(profile_for(platform)))?;
        profile_files.push(path);
    }

    Ok(PlatformReport {
        project_profile,
        stack_map,
        guidelines,
        quality_gates,
        profile_files,
    })
}

pub fn render_platform_context(repo_root: impl AsRef<Path>, task: Option<&str>) -> String {
    let repo_root = repo_root.as_ref();
    let Ok(config) = crate::config::load_project_config(repo_root) else {
        return String::new();
    };
    let platforms = active_platforms(&config);
    let lens = task_lens(task);
    let selected = platforms
        .iter()
        .copied()
        .find(|platform| platform_name(*platform) == lens)
        .unwrap_or(platforms[0]);
    let profile_path = repo_root
        .join("docs/baron/platform/profiles")
        .join(format!("{}.md", platform_name(selected)));
    let stack_path = repo_root.join("docs/baron/platform/STACK_MAP.md");
    let profile = bounded_read(&profile_path, 2_600);
    let stack = bounded_read(&stack_path, 1_400);
    format!(
        "## Platform Intelligence\n\n- Primary: `{}`\n- Task lens: `{}`\n- Loaded profile: `{}`\n\n{}\n{}\n",
        platform_name(platforms[0]),
        lens,
        platform_name(selected),
        profile,
        stack
    )
}

fn active_platforms(config: &ProjectConfig) -> Vec<ProjectPlatform> {
    let mut platforms = vec![config.platform.unwrap_or(ProjectPlatform::Unknown)];
    for platform in &config.platform_extensions {
        if !platforms.contains(platform) {
            platforms.push(*platform);
        }
    }
    platforms
}

fn profile(platform: ProjectPlatform, values: ProfileValues) -> PlatformProfile {
    PlatformProfile {
        platform,
        product_concerns: values.0,
        architecture_priorities: values.1,
        failure_modes: values.2,
        security_expectations: values.3,
        performance_expectations: values.4,
        skill_routing: values.5,
        agent_routing: values.6,
        verification_layers: values.7,
        release_proof: values.8,
    }
}

type ProfileValues = (
    &'static [&'static str],
    &'static [&'static str],
    &'static [&'static str],
    &'static [&'static str],
    &'static [&'static str],
    &'static [&'static str],
    &'static [&'static str],
    &'static [&'static str],
    &'static [&'static str],
);

const FRONTEND: ProfileValues = (
    &[
        "user journeys",
        "responsive states",
        "accessibility and content hierarchy",
    ],
    &[
        "feature-owned UI",
        "stable design tokens",
        "typed API boundaries",
    ],
    &[
        "layout overflow",
        "missing loading/error states",
        "client/server state drift",
    ],
    &[
        "safe rendering",
        "browser auth boundaries",
        "dependency and CSP review",
    ],
    &[
        "render cost",
        "bundle/loading budget",
        "image and font delivery",
    ],
    &["frontend-design", "performance-optimization when measured"],
    &[
        "code-reviewer",
        "test-engineer",
        "security-auditor for browser security",
    ],
    &[
        "component/DOM proof",
        "mobile and desktop browser proof",
        "build and accessibility checks",
    ],
    &[
        "critical journeys pass",
        "responsive evidence exists",
        "no unresolved blocking review finding",
    ],
);
const BACKEND: ProfileValues = (
    &[
        "API behavior",
        "data integrity",
        "operational failure handling",
    ],
    &[
        "clear service boundaries",
        "transaction ownership",
        "versioned contracts",
    ],
    &[
        "authorization gaps",
        "partial writes",
        "retry/idempotency errors",
    ],
    &[
        "server-side authorization",
        "input validation",
        "secret and audit handling",
    ],
    &[
        "latency and query cost",
        "bounded concurrency",
        "backpressure and caching",
    ],
    &[
        "api-and-interface-design",
        "vibe-security-scan",
        "observability-and-instrumentation",
    ],
    &["code-reviewer", "security-auditor", "test-engineer"],
    &[
        "unit/domain proof",
        "integration/database proof",
        "API and failure-path smoke",
    ],
    &[
        "contract compatibility proven",
        "migration/rollback known",
        "security and operations proof recorded",
    ],
);
const FULLSTACK: ProfileValues = (
    &[
        "end-to-end journeys",
        "shared contracts",
        "deployment and data consistency",
    ],
    &[
        "frontend/backend/database ownership",
        "single declared shared contracts",
        "independent deploy boundaries",
    ],
    &[
        "contract drift",
        "auth split-brain",
        "cross-layer partial rollout",
    ],
    &[
        "end-to-end authorization",
        "tenant/data isolation",
        "secret boundaries by runtime",
    ],
    &[
        "browser plus API latency",
        "query and bundle budgets",
        "cache invalidation",
    ],
    &[
        "frontend-design",
        "api-and-interface-design",
        "vibe-security-scan",
        "observability-and-instrumentation",
    ],
    &["code-reviewer", "security-auditor", "test-engineer"],
    &[
        "layer tests",
        "contract/integration proof",
        "end-to-end critical journey",
    ],
    &[
        "all changed layers verified",
        "shared contract compatibility proven",
        "deploy and rollback path recorded",
    ],
);
const MOBILE: ProfileValues = (
    &[
        "device journeys",
        "offline/interrupted behavior",
        "store and OS lifecycle",
    ],
    &[
        "platform-neutral domain core",
        "native boundary isolation",
        "versioned backend contracts",
    ],
    &[
        "lifecycle state loss",
        "permission denial gaps",
        "offline sync conflict",
    ],
    &[
        "secure local storage",
        "deep-link validation",
        "least-privilege permissions",
    ],
    &[
        "startup and frame stability",
        "network/battery budget",
        "memory pressure",
    ],
    &[
        "api-and-interface-design",
        "performance-optimization when measured",
        "vibe-security-scan",
    ],
    &["code-reviewer", "security-auditor", "test-engineer"],
    &[
        "domain/unit proof",
        "emulator/simulator proof",
        "device lifecycle and network-state proof",
    ],
    &[
        "supported OS matrix passes",
        "permissions and offline states verified",
        "store-ready artifact checks pass",
    ],
);
const DESKTOP: ProfileValues = (
    &[
        "desktop workflows",
        "filesystem/OS integration",
        "install and update safety",
    ],
    &[
        "UI/core separation",
        "OS adapters",
        "signed packaging and rollback",
    ],
    &[
        "path/permission errors",
        "update corruption",
        "platform-specific behavior drift",
    ],
    &["safe file access", "IPC validation", "credential storage"],
    &[
        "startup and memory use",
        "large-file responsiveness",
        "background work bounds",
    ],
    &[
        "performance-optimization when measured",
        "vibe-security-scan",
        "deprecation-and-migration",
    ],
    &["code-reviewer", "security-auditor", "test-engineer"],
    &[
        "core tests",
        "OS integration smoke",
        "installer/update/rollback proof",
    ],
    &[
        "platform matrix passes",
        "installer integrity verified",
        "user data survives update/rollback",
    ],
);
const TOOL: ProfileValues = (
    &[
        "predictable CLI UX",
        "automation compatibility",
        "safe install/update",
    ],
    &[
        "library/CLI separation",
        "stable machine-readable output",
        "atomic filesystem changes",
    ],
    &[
        "partial writes",
        "shell quoting errors",
        "breaking exit/output contracts",
    ],
    &[
        "path validation",
        "safe command execution",
        "checksum and secret handling",
    ],
    &[
        "startup/scan bounds",
        "streaming output",
        "large-repo behavior",
    ],
    &[
        "api-and-interface-design",
        "deprecation-and-migration",
        "performance-optimization when measured",
    ],
    &["code-reviewer", "security-auditor", "test-engineer"],
    &[
        "unit/core proof",
        "CLI contract tests",
        "cross-platform install and smoke",
    ],
    &[
        "exit/output compatibility proven",
        "supported platform installers pass",
        "rollback and data safety verified",
    ],
);
const LIBRARY: ProfileValues = (
    &[
        "consumer API",
        "compatibility",
        "documentation and examples",
    ],
    &[
        "small public surface",
        "internal encapsulation",
        "semantic versioning",
    ],
    &[
        "silent breaking change",
        "feature interaction",
        "example drift",
    ],
    &[
        "safe defaults",
        "input/resource bounds",
        "dependency provenance",
    ],
    &[
        "benchmarkable hot paths",
        "allocation/size cost",
        "compile/import cost",
    ],
    &[
        "api-and-interface-design",
        "deprecation-and-migration",
        "performance-optimization when measured",
    ],
    &[
        "code-reviewer",
        "security-auditor when exposed",
        "test-engineer",
    ],
    &[
        "public API tests",
        "consumer fixture",
        "compatibility and docs examples",
    ],
    &[
        "versioning decision recorded",
        "consumer fixture passes",
        "migration notes exist for breaking change",
    ],
);
const DATA: ProfileValues = (
    &[
        "data correctness",
        "lineage and reproducibility",
        "backfill/recovery",
    ],
    &[
        "schema ownership",
        "idempotent stages",
        "immutable raw inputs and versioned outputs",
    ],
    &[
        "duplicate/lost records",
        "schema drift",
        "non-reproducible backfill",
    ],
    &[
        "data access control",
        "PII minimization",
        "safe serialization and retention",
    ],
    &[
        "throughput and memory bounds",
        "partition/query efficiency",
        "incremental processing",
    ],
    &[
        "observability-and-instrumentation",
        "deprecation-and-migration",
        "vibe-security-scan",
    ],
    &["code-reviewer", "security-auditor", "test-engineer"],
    &[
        "transformation tests",
        "quality/lineage checks",
        "backfill and rollback rehearsal",
    ],
    &[
        "quality thresholds pass",
        "schema compatibility proven",
        "replay/backfill evidence recorded",
    ],
);
const CLOUD: ProfileValues = (
    &["reliability", "cost and operability", "safe deployment"],
    &[
        "least-privilege components",
        "declarative infrastructure",
        "observable rollback boundaries",
    ],
    &[
        "permission drift",
        "region/provider failure",
        "irreversible rollout",
    ],
    &[
        "IAM least privilege",
        "secret rotation",
        "network and supply-chain controls",
    ],
    &[
        "capacity and cost bounds",
        "cold-start/latency",
        "autoscaling and retry storms",
    ],
    &[
        "observability-and-instrumentation",
        "vibe-security-scan",
        "deprecation-and-migration",
    ],
    &["code-reviewer", "security-auditor", "test-engineer"],
    &[
        "policy/static checks",
        "staging integration proof",
        "deploy/rollback and alert proof",
    ],
    &[
        "change plan and rollback verified",
        "security policy passes",
        "monitoring and ownership exist",
    ],
);
const UNKNOWN: ProfileValues = (
    &[
        "discover product purpose",
        "identify users and critical behavior",
    ],
    &[
        "map existing boundaries before proposing structure",
        "preserve working behavior",
    ],
    &[
        "false assumptions",
        "destructive normalization",
        "unverified commands",
    ],
    &[
        "identify trust boundaries before editing",
        "do not infer auth or data policy",
    ],
    &["measure before optimizing", "keep scans bounded"],
    &["Superpowers first; load domain skills only from evidence"],
    &[
        "code-reviewer",
        "test-engineer",
        "security-auditor only when risk evidence exists",
    ],
    &["survey and existing tests", "smallest safe smoke"],
    &[
        "project type clarified",
        "unknowns recorded",
        "existing behavior preserved",
    ],
);

fn render_profile(profile: PlatformProfile) -> String {
    format!(
        "# {} Engineering Profile\n\n## Product Concerns\n{}\n\n## Architecture Priorities\n{}\n\n## Common Failure Modes\n{}\n\n## Security Expectations\n{}\n\n## Performance Expectations\n{}\n\n## Skill Routing\n{}\n\n## Agent Routing\n{}\n\n## Verification Layers\n{}\n\n## Release Proof\n{}\n",
        title(platform_name(profile.platform)),
        list(profile.product_concerns), list(profile.architecture_priorities),
        list(profile.failure_modes), list(profile.security_expectations),
        list(profile.performance_expectations), list(profile.skill_routing),
        list(profile.agent_routing), list(profile.verification_layers), list(profile.release_proof)
    )
}

fn render_stack_map(survey: &crate::survey::RepoSurvey) -> String {
    format!(
        "# Baron Stack Map\n\n## Detected Stack\n{}\n\n## Entrypoints\n{}\n\n## Commands\n{}\n\n## Unknown\n{}\n\nDetected means observed in repository files. Missing facts are not inferred.\n",
        survey.stack_hints.iter().map(|item| format!("- {} (`{}`)", item.label, item.path)).collect::<Vec<_>>().join("\n").if_empty("- none detected"),
        survey.entrypoints.iter().map(|item| format!("- {} (`{}`)", item.label, item.path)).collect::<Vec<_>>().join("\n").if_empty("- none detected"),
        survey.commands.iter().map(|item| format!("- {}: `{}`", item.kind, item.command)).collect::<Vec<_>>().join("\n").if_empty("- none detected"),
        survey.unknowns.join("\n- ").prefix_list()
    )
}

fn render_guidelines(platforms: &[ProjectPlatform]) -> String {
    let bodies = platforms
        .iter()
        .map(|platform| render_profile(profile_for(*platform)))
        .collect::<Vec<_>>()
        .join("\n");
    format!("# Baron Platform Engineering Guidelines\n\n{bodies}\n")
}

fn render_quality_gates(platforms: &[ProjectPlatform]) -> String {
    let mut lines = vec!["# Baron Platform Quality Gates".to_string(), String::new()];
    for platform in platforms {
        let profile = profile_for(*platform);
        lines.push(format!("## {}", title(platform_name(*platform))));
        lines.push(list(profile.verification_layers));
        lines.push(
            "- Core gates remain code-reviewer, security-auditor, and test-engineer when routed."
                .to_string(),
        );
        lines.push(String::new());
    }
    lines.join("\n")
}

fn upsert_managed(path: &Path, body: &str) -> Result<()> {
    let existing = fs::read_to_string(path).unwrap_or_default();
    let block = format!("{START}\n{}{END}", body.trim_end().to_string() + "\n");
    let content = match (existing.find(START), existing.find(END)) {
        (Some(start), Some(end)) if start < end => {
            let after = end + END.len();
            format!("{}{}{}", &existing[..start], block, &existing[after..])
        }
        _ if existing.trim().is_empty() => format!("{block}\n"),
        _ => format!("{}\n\n{block}\n", existing.trim_end()),
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content).with_context(|| format!("Could not write {}", path.display()))
}

fn bounded_read(path: &Path, limit: usize) -> String {
    let value =
        fs::read_to_string(path).unwrap_or_else(|_| "- profile not generated yet".to_string());
    value.chars().take(limit).collect()
}

fn task_lens(task: Option<&str>) -> &'static str {
    let task = task.unwrap_or_default().to_lowercase();
    for (name, words) in [
        ("mobile", &["mobile", "android", "ios", "device"][..]),
        (
            "frontend",
            &["frontend", "ui", "page", "component", "responsive"],
        ),
        ("backend", &["backend", "api", "server", "database", "auth"]),
        ("cloud", &["cloud", "deploy", "infrastructure", "iam"]),
        ("data", &["pipeline", "dataset", "etl", "analytics"]),
    ] {
        if words.iter().any(|word| task.contains(word)) {
            return name;
        }
    }
    "primary"
}

pub fn platform_name(platform: ProjectPlatform) -> &'static str {
    match platform {
        ProjectPlatform::Frontend => "frontend",
        ProjectPlatform::Backend => "backend",
        ProjectPlatform::Fullstack => "fullstack",
        ProjectPlatform::Mobile => "mobile",
        ProjectPlatform::Desktop => "desktop",
        ProjectPlatform::Tool => "tool",
        ProjectPlatform::Library => "library",
        ProjectPlatform::Data => "data",
        ProjectPlatform::Cloud => "cloud",
        ProjectPlatform::Unknown => "unknown",
    }
}

fn extension_names(platforms: &[ProjectPlatform]) -> String {
    if platforms.len() < 2 {
        "none".to_string()
    } else {
        platforms[1..]
            .iter()
            .map(|p| format!("`{}`", platform_name(*p)))
            .collect::<Vec<_>>()
            .join(", ")
    }
}
fn list(values: &[&str]) -> String {
    values
        .iter()
        .map(|v| format!("- {v}"))
        .collect::<Vec<_>>()
        .join("\n")
}
fn title(value: &str) -> String {
    let mut chars = value.chars();
    chars
        .next()
        .map(|c| format!("{}{}", c.to_ascii_uppercase(), chars.as_str()))
        .unwrap_or_default()
}

trait StringListExt {
    fn if_empty(self, fallback: &str) -> String;
    fn prefix_list(self) -> String;
}
impl StringListExt for String {
    fn if_empty(self, fallback: &str) -> String {
        if self.is_empty() {
            fallback.to_string()
        } else {
            self
        }
    }
    fn prefix_list(self) -> String {
        if self.is_empty() {
            "- none".to_string()
        } else {
            format!("- {self}")
        }
    }
}
