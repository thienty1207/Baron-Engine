use std::fs;
use std::path::{Path, PathBuf};

use baron_core::config::{initialize_project, AdapterKind};
use baron_core::harness::{current_harness_title_for_operation, start_or_resume_intake};
use baron_core::operation::{LifecycleIdentity, OperationContext, SupportedAdapter};
use baron_core::plan::start_or_resume_plan_for_operation;
use baron_core::vault::{ensure_vault, VaultContext};
use tempfile::{tempdir, TempDir};

struct Fixture {
    _temp: TempDir,
    repo: PathBuf,
    vault: VaultContext,
    operation: OperationContext,
    plan: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("demo");
        let vault = temp.path().join("Vault");
        fs::create_dir_all(&repo).unwrap();
        initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
        let vault = ensure_vault(&vault, &repo).unwrap();
        let operation = operation(&vault.project_id, "fix README alpha", "request-a");
        let plan =
            start_or_resume_plan_for_operation(&repo, &vault, "fix README alpha", &operation)
                .unwrap();
        start_or_resume_intake(&repo, &vault, "fix README alpha").unwrap();
        Self {
            _temp: temp,
            repo,
            vault,
            operation,
            plan: plan.repo_path,
        }
    }

    fn title(&self) -> anyhow::Result<Option<String>> {
        current_harness_title_for_operation(&self.repo, &self.operation)
    }

    fn current(&self) -> PathBuf {
        self.repo.join("docs/baron/harness/CURRENT.md")
    }
}

fn operation(project: &str, task: &str, request: &str) -> OperationContext {
    let identity = LifecycleIdentity::resolve(
        project,
        task,
        SupportedAdapter::Codex,
        Some("session"),
        Some(request),
    )
    .unwrap();
    OperationContext::from_identity(&identity)
}

fn rewrite(path: &Path, old: &str, new: &str) {
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains(old), "fixture must contain {old}");
    fs::write(path, content.replace(old, new)).unwrap();
}

#[test]
fn exact_story_ownership_ignores_another_tasks_plan_current() {
    let fixture = Fixture::new();
    let other = operation(&fixture.vault.project_id, "fix README beta", "request-b");
    start_or_resume_plan_for_operation(&fixture.repo, &fixture.vault, "fix README beta", &other)
        .unwrap();
    assert_eq!(
        fixture.title().unwrap().as_deref(),
        Some("fix README alpha")
    );
    assert_eq!(
        current_harness_title_for_operation(&fixture.repo, &other).unwrap(),
        None
    );
    fs::write(
        fixture.repo.join("docs/baron/plans/CURRENT.md"),
        "malformed presentation",
    )
    .unwrap();
    assert_eq!(
        fixture.title().unwrap().as_deref(),
        Some("fix README alpha")
    );
}

#[test]
fn identified_operations_resolve_their_own_story_after_b_updates_current() {
    let fixture = Fixture::new();
    let other = operation(&fixture.vault.project_id, "fix README beta", "request-b");
    start_or_resume_plan_for_operation(&fixture.repo, &fixture.vault, "fix README beta", &other)
        .unwrap();
    start_or_resume_intake(&fixture.repo, &fixture.vault, "fix README beta").unwrap();
    assert_eq!(
        fixture.title().unwrap().as_deref(),
        Some("fix README alpha")
    );
    assert_eq!(
        current_harness_title_for_operation(&fixture.repo, &other)
            .unwrap()
            .as_deref(),
        Some("fix README beta")
    );
}

#[test]
fn a_matching_projection_cannot_claim_another_story_file() {
    let fixture = Fixture::new();
    start_or_resume_intake(&fixture.repo, &fixture.vault, "fix README beta").unwrap();
    rewrite(
        &fixture.current(),
        "- Title: fix README beta",
        "- Title: fix README alpha",
    );
    assert_eq!(
        fixture.title().unwrap().as_deref(),
        Some("fix README alpha"),
        "the exact plan task and story file, not CURRENT, determine ownership"
    );
}

#[test]
fn a_story_requires_exact_indexed_active_ownership() {
    let fixture = Fixture::new();
    fs::write(
        fixture.repo.join("docs/baron/plans/ACTIVE.md"),
        "# Baron Active Plan Index\n",
    )
    .unwrap();
    assert_eq!(fixture.title().unwrap(), None);
}

#[test]
fn a_second_unindexed_same_task_plan_prevents_story_attachment() {
    let fixture = Fixture::new();
    let other = operation(&fixture.vault.project_id, "fix README alpha", "request-b");
    let content = fs::read_to_string(&fixture.plan)
        .unwrap()
        .replace(
            fixture.operation.operation_id.as_deref().unwrap(),
            other.operation_id.as_deref().unwrap(),
        )
        .replace("request-a", "request-b");
    fs::write(
        fixture.repo.join("docs/baron/plans/unindexed-second.md"),
        content,
    )
    .unwrap();
    assert_eq!(fixture.title().unwrap(), None);
}

#[test]
fn compact_crlf_frontmatter_cannot_hide_a_second_active_task_owner() {
    let fixture = Fixture::new();
    let other = operation(&fixture.vault.project_id, "fix README alpha", "request-b");
    let content = fs::read_to_string(&fixture.plan)
        .unwrap()
        .replace(
            fixture.operation.operation_id.as_deref().unwrap(),
            other.operation_id.as_deref().unwrap(),
        )
        .replace("request-a", "request-b")
        .replace(": ", ":")
        .replace('\n', "\r\n");
    fs::write(
        fixture.repo.join("docs/baron/plans/unindexed-second.md"),
        content,
    )
    .unwrap();
    assert_eq!(fixture.title().unwrap(), None);
}

#[test]
fn valid_crlf_plan_frontmatter_keeps_unique_story_ownership() {
    let fixture = Fixture::new();
    rewrite(&fixture.plan, "\n", "\r\n");
    assert_eq!(
        fixture.title().unwrap().as_deref(),
        Some("fix README alpha")
    );
}

#[test]
fn ownership_rejects_identity_from_another_project() {
    let fixture = Fixture::new();
    let foreign = operation("another-project", "fix README alpha", "request-a");
    assert!(current_harness_title_for_operation(&fixture.repo, &foreign).is_err());
}

#[test]
fn partial_identity_is_not_story_authority() {
    let fixture = Fixture::new();
    assert_eq!(
        current_harness_title_for_operation(
            &fixture.repo,
            &OperationContext::new(SupportedAdapter::Codex)
        )
        .unwrap(),
        None
    );
}

#[test]
fn duplicate_story_projection_fields_do_not_change_operation_story_authority() {
    let fixture = Fixture::new();
    let mut content = fs::read_to_string(fixture.current()).unwrap();
    content.push_str("- Title: fix README alpha\n");
    fs::write(fixture.current(), content).unwrap();
    assert_eq!(
        fixture.title().unwrap().as_deref(),
        Some("fix README alpha")
    );
}

#[test]
fn malformed_active_index_is_not_story_authority() {
    let fixture = Fixture::new();
    fs::write(
        fixture.repo.join("docs/baron/plans/ACTIVE.md"),
        "# Baron Active Plan Index\n<!-- BARON:ACTIVE-PLAN invalid-json -->\n",
    )
    .unwrap();
    assert!(fixture.title().is_err());
}

#[test]
fn missing_actual_story_gives_no_authority() {
    let fixture = Fixture::new();
    let content = fs::read_to_string(fixture.current()).unwrap();
    let story_path = content
        .lines()
        .find_map(|line| {
            line.strip_prefix("- Story: `")
                .and_then(|value| value.strip_suffix('`'))
        })
        .unwrap();
    fs::remove_file(fixture.repo.join(story_path)).unwrap();
    // A valid managed path to an absent story is unknown, not guessed.
    let content = content
        .lines()
        .map(|line| {
            if line.starts_with("- Story: ") {
                "- Story: `docs/baron/harness/stories/missing.md`"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(fixture.current(), content).unwrap();
    assert_eq!(fixture.title().unwrap(), None);
}

#[test]
fn harness_projection_risk_cannot_lower_the_owned_story_risk() {
    let fixture = Fixture::new();
    rewrite(&fixture.current(), "- Risk: `low`", "- Risk: `high`");
    assert_eq!(
        fixture.title().unwrap().as_deref(),
        Some("fix README alpha")
    );
}
