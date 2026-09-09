use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::load_project_config;
use crate::continuity::continuity_status;
use crate::control_plane::route_task;
use crate::intent::intent_status;
use crate::plan::plan_status;
use crate::proof::latest_proof;
use crate::trace::latest_trace_score;
use crate::vault::VaultContext;
use crate::work_shape::decide_work_shape;

pub const TASK_STATE_SCHEMA_VERSION: u32 = 1;
const MAX_FIELD_CHARS: usize = 900;
const MAX_LIST_ITEMS: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskStateProjection {
    pub schema_version: u32,
    pub project_id: String,
    pub task_id: String,
    pub task: String,
    pub original_intent: Option<String>,
    pub intent: Option<String>,
    pub target_behavior: Option<String>,
    pub constraints: Vec<String>,
    pub non_goals: Vec<String>,
    pub current_plan: Option<String>,
    pub continuity: Option<String>,
    pub recovery: Option<String>,
    pub last_successful_step: Option<String>,
    pub affected_files: Vec<String>,
    pub route: Option<TaskStateRoute>,
    pub mandatory_gates: Vec<String>,
    pub proof_state: Option<String>,
    pub trace_state: Option<String>,
    pub proof_trace_state: Option<String>,
    pub blockers: Vec<String>,
    pub unknowns: Vec<String>,
    pub next_action: String,
    pub resumed: bool,
    pub conflicts: Vec<String>,
    pub stale_recovery: bool,
    pub truncated: Vec<String>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskStateRoute {
    pub explanation: String,
    pub selected_skills: Vec<String>,
    pub selected_agents: Vec<String>,
}

pub fn compile_task_state(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    task: Option<&str>,
) -> Result<TaskStateProjection> {
    let repo_root = repo_root.as_ref();
    let task = task
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("current repository state");
    let task_id = task_id(&vault.project_id, task);
    let intent_source = intent_status(repo_root).unwrap_or_default();
    let plan_source = plan_status(repo_root).unwrap_or_default();
    let continuity_source = continuity_status(repo_root, vault).unwrap_or_default();
    let recovery_path = repo_root.join("docs/baron/continuity/CURRENT_RECOVERY.md");
    let continuity_path = repo_root.join("docs/baron/continuity/CURRENT.md");
    let recovery_source = read_bounded(&recovery_path, MAX_FIELD_CHARS * 2);

    let intent_title = field(&intent_source, "- Title: ");
    let target_behavior = field(&intent_source, "- Target behavior: ")
        .or_else(|| section_first_line(&intent_source, "## Target Behavior"));
    let constraints = section_list(&intent_source, "## Constraints");
    let non_goals = section_list(&intent_source, "## Non-Goals");
    let plan_title = field(&plan_source, "- Title: ");
    let plan_status = field(&plan_source, "- Status: ");
    let plan_next = field(&plan_source, "- Next action: ");
    let continuity_task = field(&continuity_source, "- Current task: ");
    let continuity_next = field(&continuity_source, "- Next action: ");
    let recovery_outcome = field(&recovery_source, "- Outcome: ");
    let recovery_root_cause = section_first_line(&recovery_source, "## Root Cause");
    let last_successful_step = section_first_line(&recovery_source, "## Last Successful Step")
        .or_else(|| section_first_line(&continuity_source, "## Last Successful Step"));
    let recovery_next = section_first_line(&recovery_source, "## Safe Next Action");
    let stale_recovery = recovery_is_stale(&continuity_path, &recovery_path);

    let work_shape = decide_work_shape(repo_root, task).ok();
    let route_result = work_shape
        .as_ref()
        .map(|decision| route_task(repo_root, task, decision.risk));
    let route = route_result.as_ref().and_then(|result| {
        result.as_ref().ok().map(|report| TaskStateRoute {
            explanation: bounded(&report.explanation),
            selected_skills: report
                .selected_skills
                .iter()
                .map(|item| item.name.clone())
                .collect(),
            selected_agents: report.mandatory_agents.clone(),
        })
    });

    let mut blockers = Vec::new();
    if let Some(Err(error)) = route_result {
        blockers.push(format!("control-plane route unavailable: {error}"));
    }
    if let Some(outcome) = recovery_outcome.as_deref() {
        if matches!(outcome, "failed" | "blocked" | "interrupted") {
            if let Some(root_cause) = recovery_root_cause.as_deref() {
                blockers.push(format!("recovery blocker: {}", bounded(root_cause)));
            }
        }
        if matches!(outcome, "failed" | "blocked" | "interrupted")
            && recovery_next.is_none()
            && !stale_recovery
        {
            blockers.push("recovery state has no safe next action".to_string());
        }
    }
    let conflicts = detect_conflicts(intent_title.as_deref(), plan_title.as_deref());
    if !conflicts.is_empty() {
        blockers.push("intent and plan describe incompatible targets".to_string());
    }

    let mut unknowns = Vec::new();
    if intent_title.is_none() {
        unknowns.push("current intent is unknown".to_string());
    }
    if plan_title.is_none() || plan_status.is_none() {
        unknowns.push("current plan/work state is incomplete".to_string());
    }
    if continuity_task.is_none() && recovery_source.trim().is_empty() {
        unknowns.push("continuity and recovery state are unknown".to_string());
    }
    if route.is_none() {
        unknowns.push("current route is unknown".to_string());
    }

    let proof_state =
        latest_proof(repo_root)?.map(|proof| format!("{}: {}", proof.id, bounded(&proof.summary)));
    let trace_state = latest_trace_score(repo_root)?.map(|trace| {
        format!(
            "{}/{}; passed={}",
            trace.achieved.as_str(),
            trace.required.as_str(),
            trace.passed
        )
    });
    let mandatory_gates = route
        .as_ref()
        .map(|route| route.selected_agents.clone())
        .unwrap_or_default();
    let proof_required = work_shape
        .as_ref()
        .map(|decision| decision.proof_required)
        .unwrap_or(false);
    if proof_required && proof_state.is_none() {
        unknowns.push("required proof has not been recorded".to_string());
    }
    if proof_required && trace_state.is_none() {
        unknowns.push("required trace has not been recorded".to_string());
    }
    let proof_trace_state = Some(bounded(&format!(
        "proof={}; trace={}",
        proof_state.as_deref().unwrap_or("unknown"),
        trace_state.as_deref().unwrap_or("unknown")
    )));

    let mut sources = vec![
        ".baron/project.toml".to_string(),
        "docs/baron/harness/CURRENT_INTENT.md".to_string(),
        "docs/baron/plans/CURRENT.md".to_string(),
        "docs/baron/continuity/CURRENT.md".to_string(),
        "docs/baron/continuity/CURRENT_RECOVERY.md".to_string(),
        "docs/baron/proofs/INDEX.md".to_string(),
        "docs/baron/traces/INDEX.md".to_string(),
    ];
    if load_project_config(repo_root).is_err() {
        unknowns.push("project configuration is unavailable".to_string());
        sources.retain(|source| source != ".baron/project.toml");
    }

    let mut affected_files = list_value(&continuity_source, "- Changed files: ");
    affected_files.extend(section_list(&recovery_source, "## Affected Files"));
    affected_files.sort();
    affected_files.dedup();
    affected_files.truncate(MAX_LIST_ITEMS);

    let next_action = if !stale_recovery {
        recovery_next
            .clone()
            .or(continuity_next.clone())
            .or(plan_next.clone())
            .unwrap_or_else(|| "unknown; inspect current authorities before acting".to_string())
    } else {
        continuity_next
            .clone()
            .or(plan_next.clone())
            .unwrap_or_else(|| {
                "recovery is stale; reconcile current continuity before acting".to_string()
            })
    };
    let resumed = matches!(plan_status.as_deref(), Some("interrupted" | "in_progress"))
        || matches!(
            recovery_outcome.as_deref(),
            Some("failed" | "blocked" | "interrupted")
        )
        || continuity_task.is_some();

    Ok(TaskStateProjection {
        schema_version: TASK_STATE_SCHEMA_VERSION,
        project_id: vault.project_id.clone(),
        task_id,
        task: bounded(task),
        original_intent: Some(bounded(intent_title.as_deref().unwrap_or(task))),
        intent: intent_title.map(|value| bounded(&value)),
        target_behavior: target_behavior.map(|value| bounded(&value)),
        constraints,
        non_goals,
        current_plan: plan_title.map(|title| {
            bounded(&format!(
                "title={title}; status={}; next={}",
                plan_status.as_deref().unwrap_or("unknown"),
                plan_next.as_deref().unwrap_or("unknown")
            ))
        }),
        continuity: continuity_task.map(|current| {
            bounded(&format!(
                "task={current}; next={}",
                continuity_next.as_deref().unwrap_or("unknown")
            ))
        }),
        recovery: if recovery_source.trim().is_empty() {
            None
        } else {
            Some(bounded(&format!(
                "outcome={}; next={}; stale={stale_recovery}",
                recovery_outcome.as_deref().unwrap_or("unknown"),
                recovery_next.as_deref().unwrap_or("unknown")
            )))
        },
        last_successful_step: last_successful_step.map(|value| bounded(&value)),
        affected_files,
        route,
        mandatory_gates,
        proof_state: proof_state.map(|value| bounded(&value)),
        trace_state: trace_state.map(|value| bounded(&value)),
        proof_trace_state,
        blockers: bounded_list(blockers),
        unknowns: bounded_list(unknowns),
        next_action: bounded(&next_action),
        resumed,
        conflicts: bounded_list(conflicts),
        stale_recovery,
        truncated: Vec::new(),
        sources,
    })
}

pub fn render_task_state(state: &TaskStateProjection, max_chars: usize) -> String {
    let mut output = String::new();
    output.push_str("## Task State\n\n");
    output.push_str(&format!("- Schema: {}\n", state.schema_version));
    output.push_str(&format!(
        "- Project identity: {}\n",
        bounded_to(&state.project_id, 120)
    ));
    output.push_str(&format!(
        "- Task identity: {}\n",
        bounded_to(&state.task_id, 120)
    ));
    output.push_str(&format!("- Task: {}\n", bounded_to(&state.task, 300)));
    output.push_str(&format!(
        "- Original intent: {}\n",
        bounded_to(state.original_intent.as_deref().unwrap_or("unknown"), 300)
    ));
    output.push_str(&format!(
        "- Intent: {}\n",
        bounded_to(state.intent.as_deref().unwrap_or("unknown"), 300)
    ));
    output.push_str(&format!(
        "- Target behavior: {}\n",
        bounded_to(state.target_behavior.as_deref().unwrap_or("unknown"), 300)
    ));
    output.push_str(&format!(
        "- Constraints: {}\n",
        list_or_unknown_limited(&state.constraints, 360)
    ));
    output.push_str(&format!(
        "- Non-goals: {}\n",
        list_or_unknown_limited(&state.non_goals, 360)
    ));
    output.push_str(&format!(
        "- Current plan/work state: {}\n",
        bounded_to(state.current_plan.as_deref().unwrap_or("unknown"), 300)
    ));
    output.push_str(&format!(
        "- Recovery: {}\n",
        bounded_to(state.recovery.as_deref().unwrap_or("unknown"), 300)
    ));
    output.push_str(&format!(
        "- Blockers: {}\n",
        list_or_unknown_limited(&state.blockers, 360)
    ));
    output.push_str(&format!(
        "- Next action: {}\n",
        bounded_to(&state.next_action, 300)
    ));
    output.push_str(&format!(
        "- Safe next action: {}\n",
        bounded_to(&state.next_action, 300)
    ));
    let route = state.route.as_ref().map(|route| {
        bounded_to(
            &format!(
                "{}; skills={}; agents={}",
                route.explanation,
                list_or_unknown_limited(&route.selected_skills, 220),
                list_or_unknown_limited(&route.selected_agents, 220)
            ),
            500,
        )
    });
    output.push_str(&format!(
        "- Route: {}\n",
        route.as_deref().unwrap_or("unknown")
    ));
    output.push_str(&format!(
        "- Mandatory proof/completion gates: {}\n",
        list_or_unknown_limited(&state.mandatory_gates, 360)
    ));
    output.push_str(&format!(
        "- Continuity: {}\n",
        bounded_to(state.continuity.as_deref().unwrap_or("unknown"), 300)
    ));
    output.push_str(&format!(
        "- Last successful step: {}\n",
        bounded_to(
            state.last_successful_step.as_deref().unwrap_or("unknown"),
            300
        )
    ));
    output.push_str(&format!(
        "- Affected files: {}\n",
        list_or_unknown_limited(&state.affected_files, 360)
    ));
    output.push_str(&format!(
        "- Proof state: {}\n",
        bounded_to(state.proof_state.as_deref().unwrap_or("unknown"), 260)
    ));
    output.push_str(&format!(
        "- Trace state: {}\n",
        bounded_to(state.trace_state.as_deref().unwrap_or("unknown"), 260)
    ));
    output.push_str(&format!(
        "- Proof/trace state: {}\n",
        bounded_to(state.proof_trace_state.as_deref().unwrap_or("unknown"), 300)
    ));
    output.push_str(&format!(
        "- Unknowns: {}\n",
        list_or_unknown_limited(&state.unknowns, 360)
    ));
    output.push_str(&format!("- Resumed: {}\n", state.resumed));
    output.push_str(&format!(
        "- Conflicts: {}\n",
        list_or_unknown_limited(&state.conflicts, 360)
    ));
    output.push_str(&format!("- Stale recovery: {}\n", state.stale_recovery));
    output.push_str(&format!(
        "- Sources: {}\n",
        list_or_unknown_limited(&state.sources, 360)
    ));
    output.push('\n');
    if output.chars().count() > max_chars {
        let bounded_output = output.chars().take(max_chars).collect::<String>();
        return format!(
            "{}\n- Task State fields are individually bounded; rendering budget reached.\n",
            bounded_output.trim_end()
        );
    }
    output
}

fn task_id(project_id: &str, task: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(project_id.as_bytes());
    digest.update([0]);
    digest.update(task.as_bytes());
    let digest = digest.finalize();
    let suffix = digest
        .iter()
        .take(10)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("task-{suffix}")
}

fn bounded(value: &str) -> String {
    let mut output = value.chars().take(MAX_FIELD_CHARS).collect::<String>();
    if value.chars().count() > MAX_FIELD_CHARS {
        output.push_str("...");
    }
    output
}

fn bounded_list(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| bounded(&value))
        .take(MAX_LIST_ITEMS)
        .collect()
}

fn list_or_unknown_limited(values: &[String], limit: usize) -> String {
    if values.is_empty() {
        "unknown".to_string()
    } else {
        bounded_to(&values.join("; "), limit)
    }
}

fn bounded_to(value: &str, limit: usize) -> String {
    let mut output = value.chars().take(limit).collect::<String>();
    if value.chars().count() > limit {
        output.push_str("...");
    }
    output
}

fn read_bounded(path: &Path, limit: usize) -> String {
    fs::read_to_string(path)
        .map(|content| content.chars().take(limit).collect())
        .unwrap_or_default()
}

fn field(source: &str, prefix: &str) -> Option<String> {
    source.lines().find_map(|line| {
        line.strip_prefix(prefix)
            .map(|value| value.trim().trim_matches(char::from(96)).trim().to_string())
    })
}

fn section_first_line(source: &str, heading: &str) -> Option<String> {
    let mut active = false;
    for line in source.lines() {
        if line.trim() == heading {
            active = true;
            continue;
        }
        if active && line.starts_with("## ") {
            break;
        }
        if active && !line.trim().is_empty() {
            return Some(line.trim().trim_start_matches("- ").to_string());
        }
    }
    None
}

fn section_list(source: &str, heading: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut active = false;
    for line in source.lines() {
        if line.trim() == heading {
            active = true;
            continue;
        }
        if active && line.starts_with("## ") {
            break;
        }
        if active {
            if let Some(value) = line.trim().strip_prefix("- ") {
                values.push(value.trim().to_string());
            }
        }
    }
    bounded_list(values)
}

fn list_value(source: &str, prefix: &str) -> Vec<String> {
    source
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty() && *value != "none")
                .map(ToString::to_string)
                .collect()
        })
        .map(bounded_list)
        .unwrap_or_default()
}

fn detect_conflicts(intent: Option<&str>, plan: Option<&str>) -> Vec<String> {
    let (Some(intent), Some(plan)) = (intent, plan) else {
        return Vec::new();
    };
    let intent_words = meaningful_words(intent);
    let plan_words = meaningful_words(plan);
    if intent_words.is_empty() || plan_words.is_empty() || !intent_words.is_disjoint(&plan_words) {
        Vec::new()
    } else {
        vec![format!(
            "intent target {} conflicts with plan title {}",
            intent, plan
        )]
    }
}

fn meaningful_words(value: &str) -> BTreeSet<String> {
    value
        .to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| word.len() >= 4)
        .filter(|word| !matches!(*word, "task" | "current" | "implement" | "update"))
        .map(ToString::to_string)
        .collect()
}

fn recovery_is_stale(current: &Path, recovery: &Path) -> bool {
    let Ok(current_time) = fs::metadata(current).and_then(|metadata| metadata.modified()) else {
        return false;
    };
    let Ok(recovery_time) = fs::metadata(recovery).and_then(|metadata| metadata.modified()) else {
        return false;
    };
    current_time > recovery_time
}
