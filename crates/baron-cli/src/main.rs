use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};

mod self_update;
mod update_transaction;

use crate::self_update::CandidateBinaryInspector;
use anyhow::{bail, Context, Result};
use baron_adapters::{
    install_adapter, managed_payloads_for_adapter, plan_managed_update,
    reconcile_installed_managed_assets, shadow_preview, AgentAdapter, ManagedUpdatePlan,
    UpdateDisposition,
};
use baron_core::architecture::ensure_architecture_governor;
use baron_core::asset_lifecycle::{
    audit_runtime_assets, quarantine_failing_assets, stage_skill_update,
};
use baron_core::authority::classify_request;
use baron_core::automation::{
    automation_status, handle_hook, reconcile, record_lifecycle_event, AutomationEvent, HookAdapter,
};
use baron_core::autopilot::{
    approve_candidate, autopilot_status, reject_candidate, review_after_task,
};
use baron_core::capability::{
    check_capabilities, load_capability_state, load_registry, register_provider, remove_provider,
    runtime_backend_report, BackendSafety, CapabilityExecutionEvidence, CapabilityProvider,
    CheckOptions, Presence, ProviderKind, Requirement,
};
use baron_core::certification::{
    latest_certification_status, render_certification_report, run_certification,
    CertificationProfile,
};
use baron_core::code_graph::{
    code_graph_cache_root, ensure_code_map_capability, graph_state_freshness,
    load_code_graph_state, verify_graph_hit_source, write_code_graph_query_cache,
    CodeGraphProvider, GraphFreshness, QueryLimits, SourceVerification,
};
use baron_core::config::{
    find_project_root, initialize_project, initialize_project_with_options, load_project_config,
    resolve_vault_path_for_repo, set_project_platform, setup_machine_vault, AdapterKind,
    ProjectPlatform,
};
use baron_core::context::{compile_context_for_task, compile_context_why, ContextTarget};
use baron_core::continuity::{
    continuity_status, record_continuity_checkpoint, record_recovery, RecoveryInput,
    RecoveryOutcome,
};
use baron_core::control_plane::{
    gate_evidence_status, record_gate_evidence, route_task, validate_control_plane,
};
use baron_core::firewall::{compact_memory_brief, recall, render_recall};
use baron_core::graphify::{GraphifyProvider, SUPPORTED_GRAPHIFY_VERSION};
use baron_core::harness::{
    ensure_harness_workspace, harness_status, record_decision, record_friction,
    start_or_resume_intake,
};
use baron_core::harness_improvement::{
    audit_harness, propose_improvements, record_improvement_outcome, record_intervention,
    verify_open_stories,
};
use baron_core::intent::{intent_status, record_intent, IntentBriefInput};
use baron_core::memory::{build_memory_index, load_memory_records};
use baron_core::migration::{
    execute_agent_bootstrap_migration, inventory_agent_bootstrap, migration_status,
    render_migration_inventory, rollback_migration,
};
use baron_core::plan::{
    complete_plan, interrupt_plan, plan_status, start_or_resume_plan, update_plan,
};
use baron_core::platform::{ensure_platform_intelligence, platform_name as core_platform_name};
use baron_core::proof::{proof_status, record_proof, record_proof_with_capabilities};
use baron_core::release::{
    load_and_verify_release_metadata, verify_release_identity, write_release_metadata,
};
use baron_core::review_gate::{close_finding, record_finding, review_status, ReviewFindingInput};
use baron_core::session::{import_sessions, import_state_summary};
use baron_core::session_replay::{
    index_session_replay, replay_session_context, search_session_replay,
};
use baron_core::state_guard::require_coherent_execution_state;
use baron_core::survey::{render_project_atlas, survey_repository};
use baron_core::trace::{record_trace, score_trace, TraceOutcome};
use baron_core::vault::{ensure_vault, resolve_vault_path, vault_context_without_create};
use baron_core::{phase, product_name};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "baron", about = "Baron Engine", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Setup {
        #[arg(long, num_args = 0..=1, default_missing_value = ".")]
        vault: Option<PathBuf>,
    },
    #[command(hide = true)]
    Survey {
        repo_path: Option<PathBuf>,
        #[arg(long = "json")]
        json: bool,
    },
    Init {
        repo_path: Option<PathBuf>,
        #[arg(long)]
        codex: bool,
        #[arg(long)]
        claude: bool,
        #[arg(long = "agent")]
        agent: bool,
        #[arg(long)]
        shadow: bool,
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long)]
        frontend: bool,
        #[arg(long)]
        backend: bool,
        #[arg(long)]
        fullstack: bool,
        #[arg(long)]
        mobile: bool,
        #[arg(long)]
        desktop: bool,
        #[arg(long = "tool")]
        tool_platform: bool,
        #[arg(long)]
        library: bool,
        #[arg(long)]
        data: bool,
        #[arg(long)]
        cloud: bool,
        #[arg(long)]
        unknown: bool,
    },
    Update {
        repo_path: Option<PathBuf>,
        #[arg(long, hide = true)]
        codex: bool,
        #[arg(long, hide = true)]
        claude: bool,
        #[arg(long = "agent", hide = true)]
        agent: bool,
        #[arg(long, hide = true)]
        dry_run: bool,
        #[arg(long, hide = true, requires = "dry_run")]
        installed: bool,
        #[arg(long, hide = true, conflicts_with_all = ["dry_run", "installed"])]
        verify_candidate: bool,
        #[arg(long, hide = true, requires = "verify_candidate")]
        candidate_dir: Option<PathBuf>,
        #[arg(long = "candidate-plan", hide = true, requires = "transaction", conflicts_with_all = ["dry_run", "installed", "verify_candidate", "continue_update", "abort_update"])]
        candidate_plan: bool,
        #[arg(long = "continue", hide = true, requires = "transaction", conflicts_with_all = ["dry_run", "installed", "verify_candidate", "candidate_plan", "abort_update"])]
        continue_update: bool,
        #[arg(long = "runtime-binary", hide = true, requires = "continue_update")]
        runtime_binary: Option<PathBuf>,
        #[arg(long = "runtime-parent-pid", hide = true, requires = "continue_update")]
        runtime_parent_pid: Option<u32>,
        #[arg(long = "abort", hide = true, requires = "transaction", conflicts_with_all = ["dry_run", "installed", "verify_candidate", "candidate_plan", "continue_update"])]
        abort_update: bool,
        #[arg(long = "baron-finalize", hide = true, requires_all = ["transaction", "parent_pid"], conflicts_with_all = ["dry_run", "installed", "verify_candidate", "candidate_plan", "continue_update", "abort_update"])]
        finalize_update: bool,
        #[arg(long, hide = true, requires = "finalize_update")]
        parent_pid: Option<u32>,
        #[arg(long, hide = true)]
        transaction: Option<PathBuf>,
    },
    #[command(hide = true)]
    Authority {
        #[command(subcommand)]
        command: AuthorityCommands,
    },
    #[command(hide = true)]
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },
    #[command(hide = true)]
    Recall {
        query: String,
        repo_path: Option<PathBuf>,
        #[arg(long)]
        vault: Option<PathBuf>,
    },
    #[command(hide = true)]
    Context {
        repo_path: Option<PathBuf>,
        #[arg(long)]
        codex: bool,
        #[arg(long)]
        claude: bool,
        #[arg(long = "agent")]
        agent: bool,
        #[arg(long)]
        why: bool,
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        vault: Option<PathBuf>,
    },
    #[command(hide = true)]
    Plan {
        #[command(subcommand)]
        command: PlanCommands,
    },
    #[command(hide = true)]
    Harness {
        #[command(subcommand)]
        command: HarnessCommands,
    },
    #[command(hide = true)]
    Proof {
        #[command(subcommand)]
        command: ProofCommands,
    },
    #[command(hide = true)]
    Trace {
        #[command(subcommand)]
        command: TraceCommands,
    },
    #[command(hide = true)]
    Review {
        #[command(subcommand)]
        command: ReviewCommands,
    },
    #[command(hide = true)]
    Migrate {
        #[command(subcommand)]
        command: MigrationCommands,
    },
    #[command(hide = true)]
    Capability {
        #[command(subcommand)]
        command: CapabilityCommands,
    },
    #[command(hide = true)]
    ControlPlane {
        #[command(subcommand)]
        command: ControlPlaneCommands,
    },
    #[command(hide = true)]
    Asset {
        #[command(subcommand)]
        command: AssetCommands,
    },
    #[command(hide = true)]
    SessionReplay {
        #[command(subcommand)]
        command: SessionReplayCommands,
    },
    #[command(hide = true)]
    Autopilot {
        #[command(subcommand)]
        command: AutopilotCommands,
    },
    #[command(hide = true)]
    Runtime {
        #[command(subcommand)]
        command: RuntimeCommands,
    },
    #[command(hide = true)]
    Certify {
        #[command(subcommand)]
        command: CertifyCommands,
    },
    #[command(hide = true)]
    Automation {
        #[command(subcommand)]
        command: AutomationCommands,
    },
    #[command(hide = true)]
    Continuity {
        #[command(subcommand)]
        command: ContinuityCommands,
    },
    #[command(hide = true)]
    Release {
        #[command(subcommand)]
        command: ReleaseCommands,
    },
}

#[derive(Debug, Subcommand)]
enum AuthorityCommands {
    Classify {
        request: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum MemoryCommands {
    Status {
        repo_path: Option<PathBuf>,
        #[arg(long)]
        vault: Option<PathBuf>,
    },
    Index {
        repo_path: Option<PathBuf>,
        #[arg(long)]
        vault: Option<PathBuf>,
    },
    Compact {
        repo_path: Option<PathBuf>,
        #[arg(long)]
        vault: Option<PathBuf>,
    },
    ImportSessions {
        repo_path: Option<PathBuf>,
        #[arg(long)]
        vault: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum PlanCommands {
    Status {
        repo_path: Option<PathBuf>,
    },
    Start {
        title: String,
        repo_path: Option<PathBuf>,
    },
    Update {
        note: String,
        repo_path: Option<PathBuf>,
    },
    Interrupt {
        state: String,
        repo_path: Option<PathBuf>,
    },
    Complete {
        verification: String,
        repo_path: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum HarnessCommands {
    Status {
        repo_path: Option<PathBuf>,
    },
    IntentStatus {
        repo_path: Option<PathBuf>,
    },
    Intent {
        title: String,
        repo_path: Option<PathBuf>,
        #[arg(long = "current")]
        current_behavior: String,
        #[arg(long = "target")]
        target_behavior: String,
        #[arg(long)]
        scope: String,
        #[arg(long = "non-goal")]
        non_goals: Vec<String>,
        #[arg(long)]
        constraint: Vec<String>,
        #[arg(long)]
        decision: Vec<String>,
        #[arg(long = "proof")]
        required_proof: String,
        #[arg(long = "unknown")]
        unknowns: Vec<String>,
        #[arg(long)]
        confirmed: bool,
    },
    Audit {
        repo_path: Option<PathBuf>,
    },
    VerifyAll {
        repo_path: Option<PathBuf>,
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
    Intake {
        title: String,
        repo_path: Option<PathBuf>,
    },
    Decision {
        summary: String,
        repo_path: Option<PathBuf>,
    },
    Friction {
        summary: String,
        repo_path: Option<PathBuf>,
    },
    Intervention {
        summary: String,
        repo_path: Option<PathBuf>,
    },
    Propose {
        repo_path: Option<PathBuf>,
    },
    Outcome {
        proposal_id: String,
        outcome: String,
        repo_path: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum ProofCommands {
    Status {
        repo_path: Option<PathBuf>,
    },
    Record {
        summary: String,
        repo_path: Option<PathBuf>,
        #[arg(long = "capability-evidence")]
        capability_evidence: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
enum TraceCommands {
    Record {
        summary: String,
        repo_path: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutcomeArg::Completed)]
        outcome: OutcomeArg,
    },
    Score {
        repo_path: Option<PathBuf>,
        #[arg(long)]
        id: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum ReviewCommands {
    Status {
        repo_path: Option<PathBuf>,
    },
    Finding {
        summary: String,
        repo_path: Option<PathBuf>,
        #[arg(long)]
        severity: String,
        #[arg(long)]
        evidence: Vec<String>,
        #[arg(long = "affected-file")]
        affected_files: Vec<String>,
    },
    Close {
        id: String,
        repo_path: Option<PathBuf>,
        #[arg(long = "fix-evidence")]
        fix_evidence: String,
        #[arg(long)]
        verification: String,
    },
}

#[derive(Debug, Subcommand)]
enum MigrationCommands {
    AgentBootstrap {
        repo_path: Option<PathBuf>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        vault: Option<PathBuf>,
    },
    Status {
        repo_path: Option<PathBuf>,
    },
    Rollback {
        #[arg(long)]
        id: String,
        repo_path: Option<PathBuf>,
        #[arg(long)]
        vault: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum CapabilityCommands {
    Register {
        capability: String,
        repo_path: Option<PathBuf>,
        #[arg(long)]
        name: String,
        #[arg(long, value_enum)]
        kind: ProviderKindArg,
        #[arg(long)]
        required: bool,
        #[arg(long)]
        command: Option<String>,
        #[arg(long = "scan")]
        scan_target: Option<String>,
        #[arg(long = "adapter", value_enum)]
        adapters: Vec<AdapterArg>,
        #[arg(long)]
        description: String,
    },
    Check {
        capability: Option<String>,
        repo_path: Option<PathBuf>,
        #[arg(long, value_enum)]
        adapter: Option<AdapterArg>,
        #[arg(long)]
        json: bool,
    },
    List {
        repo_path: Option<PathBuf>,
        #[arg(long, value_enum)]
        adapter: Option<AdapterArg>,
        #[arg(long)]
        json: bool,
    },
    Remove {
        capability: String,
        repo_path: Option<PathBuf>,
        #[arg(long)]
        name: String,
    },
}

#[derive(Debug, Subcommand)]
enum ControlPlaneCommands {
    Status {
        repo_path: Option<PathBuf>,
    },
    Route {
        task: String,
        repo_path: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = RiskLaneArg::Medium)]
        risk: RiskLaneArg,
    },
    RecordGate {
        agent: String,
        summary: String,
        repo_path: Option<PathBuf>,
    },
    Evidence {
        repo_path: Option<PathBuf>,
        #[arg(long = "required")]
        required: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
enum AssetCommands {
    Audit {
        repo_path: Option<PathBuf>,
    },
    Quarantine {
        repo_path: Option<PathBuf>,
    },
    ProposeSkill {
        skill: String,
        reason: String,
        content_path: PathBuf,
        repo_path: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum SessionReplayCommands {
    Index {
        repo_path: Option<PathBuf>,
        #[arg(long)]
        vault: Option<PathBuf>,
    },
    Search {
        query: String,
        repo_path: Option<PathBuf>,
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long, default_value_t = 8)]
        limit: usize,
    },
    Replay {
        message_id: String,
        repo_path: Option<PathBuf>,
        #[arg(long)]
        vault: Option<PathBuf>,
        #[arg(long, default_value_t = 2)]
        radius: usize,
    },
}

#[derive(Debug, Subcommand)]
enum AutopilotCommands {
    Status {
        repo_path: Option<PathBuf>,
    },
    Review {
        summary: String,
        repo_path: Option<PathBuf>,
    },
    Approve {
        candidate_id: String,
        repo_path: Option<PathBuf>,
    },
    Reject {
        candidate_id: String,
        repo_path: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum RuntimeCommands {
    Check {
        repo_path: Option<PathBuf>,
        #[arg(long, value_enum)]
        adapter: Option<AdapterArg>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum CertifyCommands {
    Run {
        repo_path: Option<PathBuf>,
        #[arg(long)]
        vault: PathBuf,
        #[arg(long, value_enum, default_value_t = CertificationProfileArg::Smoke)]
        profile: CertificationProfileArg,
    },
    Status {
        repo_path: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum AutomationCommands {
    Status {
        repo_path: Option<PathBuf>,
    },
    Reconcile {
        repo_path: Option<PathBuf>,
    },
    Hook {
        #[arg(value_enum)]
        event: AutomationEventArg,
        repo_path: Option<PathBuf>,
        #[arg(long, value_enum)]
        adapter: AdapterArg,
    },
    #[command(name = "code-map")]
    CodeMap {
        #[command(subcommand)]
        command: CodeMapCommands,
    },
}

#[derive(Debug, Subcommand)]
enum CodeMapCommands {
    Status {
        repo_path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    Refresh {
        repo_path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    Query {
        question: String,
        repo_path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ContinuityCommands {
    Status {
        repo_path: Option<PathBuf>,
    },
    Checkpoint {
        note: String,
        repo_path: Option<PathBuf>,
    },
    Recover {
        root_cause: String,
        repo_path: Option<PathBuf>,
        #[arg(long, value_enum)]
        outcome: RecoveryOutcomeArg,
        #[arg(long = "last-success")]
        last_successful_step: String,
        #[arg(long)]
        evidence: Vec<String>,
        #[arg(long = "affected-file")]
        affected_files: Vec<String>,
        #[arg(long = "next-action")]
        next_action: String,
        #[arg(long = "retry-condition")]
        retry_conditions: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
enum ReleaseCommands {
    Metadata {
        artifacts_dir: PathBuf,
        #[arg(long)]
        release_version: Option<String>,
        #[arg(long)]
        source_revision: String,
    },
    Verify {
        artifacts_dir: PathBuf,
        #[arg(long)]
        expected_version: String,
        #[arg(long)]
        expected_source_revision: String,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutcomeArg {
    Completed,
    Partial,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RecoveryOutcomeArg {
    Failed,
    Blocked,
    Interrupted,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ProviderKindArg {
    Cli,
    Binary,
    Mcp,
    Skill,
    Http,
    AgentAdapter,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RiskLaneArg {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum AdapterArg {
    Codex,
    Claude,
    Agent,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CertificationProfileArg {
    Smoke,
    Release,
    Extreme,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum AutomationEventArg {
    SessionStart,
    Prompt,
    Checkpoint,
    ContextCompiled,
    PlanStarted,
    HarnessStarted,
    ProofRecorded,
    TraceScored,
    Stop,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Authority { command }) => match command {
            AuthorityCommands::Classify { request, json } => {
                let decision = classify_request(&request);
                if json {
                    println!("{}", serde_json::to_string_pretty(&decision)?);
                } else {
                    println!("# Baron Request Authority\n");
                    println!("- Authority: `{}`", decision.authority.as_str());
                    println!(
                        "- Mutation allowed: `{}`",
                        if decision.mutation_allowed() {
                            "yes"
                        } else {
                            "no"
                        }
                    );
                    println!("- Reason: {}", decision.reason);
                    println!("- Next action: {}", decision.next_action);
                }
            }
        },
        Some(Commands::Setup { vault }) => {
            let vault_path = vault.unwrap_or(std::env::current_dir()?);
            let configured = setup_machine_vault(&vault_path)?;
            println!("# Baron Setup\n");
            println!("- Default Vault: `{}`", configured.display());
            println!("- Machine config: `~/.baron/config.toml`");
            println!("- Next: run `baron init --codex --fullstack` inside a project folder.");
        }
        Some(Commands::Survey { repo_path, json }) => {
            let repo_path = repo_path.unwrap_or(std::env::current_dir()?);
            let survey = survey_repository(repo_path)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&survey)?);
            } else {
                print!("{}", render_project_atlas(&survey));
            }
        }
        Some(Commands::Init {
            repo_path,
            codex,
            claude,
            agent,
            shadow,
            vault,
            frontend,
            backend,
            fullstack,
            mobile,
            desktop,
            tool_platform,
            library,
            data,
            cloud,
            unknown,
        }) => {
            let adapter = selected_adapter(codex, claude, agent)?;
            let platform = parse_platform(
                frontend,
                backend,
                fullstack,
                mobile,
                desktop,
                tool_platform,
                library,
                data,
                cloud,
                unknown,
            )?;
            let repo_path = repo_path.unwrap_or(std::env::current_dir()?);
            if shadow {
                let adapter = adapter.context(
                    "Choose exactly one adapter for shadow init: --codex, --claude, or --agent",
                )?;
                print!("{}", shadow_preview(adapter).to_markdown());
            } else if let Some(adapter) = adapter {
                let repo_root = repo_path.canonicalize()?;
                let vault_path = resolve_vault_path_for_repo(vault, &repo_root)?;
                let config = initialize_project_with_options(
                    &repo_root,
                    Some(adapter_kind(adapter)),
                    &vault_path,
                    platform,
                )?;
                let context = ensure_vault(&vault_path, &repo_root)?;
                build_memory_index(&context)?;
                ensure_platform_intelligence(&repo_root, &config)?;
                ensure_architecture_governor(&repo_root, &config)?;
                ensure_harness_workspace(&repo_root, &context)?;
                ensure_code_map_capability(&repo_root)?;
                let report = install_adapter(&repo_root, adapter)?;
                println!("# Baron Adapter Init\n");
                println!("- Project: `{}`", context.project_slug);
                println!("- Adapter initialized: `{}`", report.adapter);
                println!("- Vault: `{}`", context.vault_root.display());
                println!(
                    "- Platform focus: `{}`",
                    config
                        .platform
                        .map(platform_name)
                        .unwrap_or("auto-detected")
                );
                println!("- Managed files: {}", report.managed_files.len());
                println!(
                    "- Extension platforms: {}",
                    platform_list(&config.platform_extensions)
                );
                println!("- Custom assets preserved: yes");
            } else if let Some(platform) = platform {
                let config = set_project_platform(&repo_path, platform)?;
                let repo_root = find_project_root(&repo_path)?;
                ensure_platform_intelligence(&repo_root, &config)?;
                ensure_architecture_governor(&repo_root, &config)?;
                ensure_code_map_capability(&repo_root)?;
                println!("# Baron Platform Focus\n");
                println!("- Project: `{}`", config.project_slug);
                println!(
                    "- Primary platform: `{}`",
                    config.platform.map(platform_name).unwrap_or("unknown")
                );
                println!(
                    "- Extension platforms: {}",
                    platform_list(&config.platform_extensions)
                );
                println!("- Adapter files were not changed.");
            } else {
                bail!(
                    "Choose an adapter (--codex, --claude, --agent), a platform (--fullstack, --backend, --frontend, --mobile, --desktop, --tool, --library, --data, --cloud), or both."
                );
            }
        }
        Some(Commands::Update {
            repo_path,
            codex,
            claude,
            agent,
            dry_run,
            installed,
            verify_candidate,
            candidate_dir,
            candidate_plan,
            continue_update,
            runtime_binary,
            runtime_parent_pid,
            abort_update,
            finalize_update,
            parent_pid,
            transaction,
        }) => {
            let start = repo_path.unwrap_or(std::env::current_dir()?);
            let repo_root = find_project_root(&start)?;
            let config = load_project_config(&repo_root)?;
            let requested = selected_adapter(codex, claude, agent)?;
            let adapters = match requested {
                Some(adapter) => {
                    let kind = adapter_kind(adapter);
                    if !config.adapters.contains(&kind) {
                        bail!(
                            "Adapter `{}` is not registered. Run `baron init --{}` first.",
                            adapter_name(adapter),
                            adapter_name(adapter)
                        );
                    }
                    vec![adapter]
                }
                None => config.adapters.iter().copied().map(agent_adapter).collect(),
            };
            let names = adapters
                .iter()
                .map(|adapter| adapter_name(*adapter))
                .collect::<Vec<_>>()
                .join(", ");
            // A later ordinary update must never silently skip an interrupted
            // project activation. Continue/finalize own their active state;
            // every other update entry point restores it before doing new work.
            if !dry_run && !continue_update && !finalize_update && !abort_update {
                let recovered = update_transaction::recover_incomplete_transactions(
                    &repo_root,
                    &config.project_id,
                )?;
                if !recovered.is_empty() {
                    eprintln!(
                        "Baron recovered incomplete update transaction(s): {}",
                        recovered.join(", ")
                    );
                }
            }
            if finalize_update {
                let state_path = transaction
                    .context("Baron delayed finalizer requires --transaction <state-path>.")?;
                #[cfg(target_os = "windows")]
                {
                    self_update::wait_for_parent_exit(
                        parent_pid.expect("Clap requires --parent-pid"),
                    )?;
                    let candidate = update_transaction::candidate_for_transaction(
                        &repo_root,
                        &state_path,
                        &config.project_id,
                    )?;
                    let installed_binary = update_transaction::runtime_binary_for_transaction(
                        &repo_root,
                        &state_path,
                        &config.project_id,
                    )?;
                    let handoff = self_update::prepare_runtime_handoff(
                        &repo_root,
                        &candidate,
                        &installed_binary,
                    )?;
                    let finalize_result = self_update::finalize_windows_handoff(
                        &handoff,
                        &self_update::ProcessBinaryInspector,
                        &candidate.version,
                    );
                    if let Err(error) = finalize_result {
                        let _ = update_transaction::recover_transaction(
                            &repo_root,
                            &state_path,
                            &config.project_id,
                        );
                        return Err(error.context(
                            "Baron delayed finalizer failed; project activation was rolled back",
                        ));
                    }
                    let completed = match update_transaction::complete_transaction(
                        &repo_root,
                        &state_path,
                        &config.project_id,
                        &format!("{} --version matched", installed_binary.display()),
                    ) {
                        Ok(completed) => completed,
                        Err(error) => {
                            let _ = self_update::rollback_windows_handoff(&handoff);
                            let _ = update_transaction::recover_transaction(
                                &repo_root,
                                &state_path,
                                &config.project_id,
                            );
                            return Err(error.context("Baron finalizer receipt failed; runtime and project activation were rolled back"));
                        }
                    };
                    println!("# Baron Delayed Update Finalizer\n");
                    println!("- Transaction: `{}`", completed.transaction_id);
                    println!("- Status: `{}`", completed.status.as_str());
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = parent_pid;
                    bail!("The delayed Baron finalizer is supported only on Windows");
                }
            } else if candidate_plan {
                let state_path = transaction
                    .context("Baron candidate planning requires --transaction <state-path>.")?;
                let payloads = adapters
                    .iter()
                    .map(|adapter| managed_payloads_for_adapter(*adapter))
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>();
                let planned = update_transaction::plan_candidate_transaction(
                    &repo_root,
                    &state_path,
                    &config.project_id,
                    env!("CARGO_PKG_VERSION"),
                    &payloads,
                )?;
                println!("# Baron Candidate Update Plan\n");
                println!("- Transaction: `{}`", planned.transaction_id);
                println!("- Status: `{}`", planned.status.as_str());
                println!("- Managed packets: {}", planned.packets.len());
                println!(
                    "- Candidate rendered only staged packets; project files remain unchanged."
                );
            } else if continue_update {
                let state_path = transaction
                    .context("Baron update continuation requires --transaction <state-path>.")?;
                let applied = update_transaction::continue_transaction(
                    &repo_root,
                    &state_path,
                    &config.project_id,
                )?;
                let candidate = update_transaction::candidate_for_transaction(
                    &repo_root,
                    &state_path,
                    &config.project_id,
                )?;
                // A verified candidate runs the hidden continuation protocol,
                // but it must replace the original installed Baron binary, not
                // the staged candidate process that is rendering new assets.
                let installed_binary = runtime_binary.unwrap_or(std::env::current_exe()?);
                let runtime_pending = update_transaction::mark_runtime_pending(
                    &repo_root,
                    &state_path,
                    &config.project_id,
                    &installed_binary,
                )?;
                #[cfg(not(target_os = "windows"))]
                let _ = (&runtime_pending, runtime_parent_pid);
                #[cfg(target_os = "windows")]
                let (runtime_status, runtime_message) = {
                    let finalizer = self_update::launch_windows_finalizer(
                        &candidate,
                        &repo_root,
                        &state_path,
                        runtime_parent_pid.unwrap_or(std::process::id()),
                    );
                    match finalizer {
                        Ok(path) => (
                            runtime_pending.status.as_str().to_string(),
                            format!("delayed finalizer launched at `{}`", path.display()),
                        ),
                        Err(error) => {
                            let _ = update_transaction::recover_transaction(
                                &repo_root,
                                &state_path,
                                &config.project_id,
                            );
                            return Err(error.context("Could not launch delayed finalizer; project activation was rolled back"));
                        }
                    }
                };
                #[cfg(not(target_os = "windows"))]
                let (runtime_status, runtime_message) = {
                    let handoff = self_update::prepare_runtime_handoff(
                        &repo_root,
                        &candidate,
                        &installed_binary,
                    )?;
                    if let Err(error) = self_update::activate_unix_handoff(&handoff) {
                        let _ = update_transaction::recover_transaction(
                            &repo_root,
                            &state_path,
                            &config.project_id,
                        );
                        return Err(error.context(
                            "Baron runtime activation failed; project activation was rolled back",
                        ));
                    }
                    let verification = verify_activated_runtime_version(
                        &self_update::ProcessBinaryInspector,
                        &installed_binary,
                        &candidate.version,
                    );
                    if let Err(error) = verification {
                        let _ = self_update::rollback_unix_handoff(&handoff);
                        let _ = update_transaction::recover_transaction(
                            &repo_root,
                            &state_path,
                            &config.project_id,
                        );
                        return Err(error.context("Activated Baron runtime failed verification; project activation was rolled back"));
                    }
                    let completed = if let Err(error) = update_transaction::complete_transaction(
                        &repo_root,
                        &state_path,
                        &config.project_id,
                        &format!("{} --version matched", installed_binary.display()),
                    ) {
                        let _ = self_update::rollback_unix_handoff(&handoff);
                        let _ = update_transaction::recover_transaction(
                            &repo_root,
                            &state_path,
                            &config.project_id,
                        );
                        return Err(error.context("Baron runtime receipt failed; runtime and project activation were rolled back"));
                    } else {
                        update_transaction::inspect_transaction(
                            &repo_root,
                            &state_path,
                            &config.project_id,
                        )?
                    };
                    (
                        completed.status.as_str().to_string(),
                        "atomic Unix runtime activation completed".to_string(),
                    )
                };
                println!("# Baron Update Transaction\n");
                println!("- Transaction: `{}`", applied.transaction_id);
                println!("- Status: `{runtime_status}`");
                println!("- Project managed assets: activated transactionally");
                println!("- Runtime activation: {runtime_message}");
            } else if abort_update {
                let state_path = transaction
                    .context("Baron update abort requires --transaction <state-path>.")?;
                update_transaction::abort_transaction(&repo_root, &state_path, &config.project_id)?;
                println!("# Baron Update Transaction\n");
                println!("- Status: `aborted`");
                println!("- Project managed assets: unchanged");
                println!("- Vault and user-owned files: unchanged");
            } else if verify_candidate {
                let target = self_update::current_release_target()?;
                let inspector = self_update::ProcessBinaryInspector;
                let candidate = if let Some(directory) = candidate_dir {
                    let source = self_update::DirectoryCandidateSource::new(directory);
                    self_update::stage_verified_candidate(
                        &repo_root,
                        &source,
                        &inspector,
                        env!("CARGO_PKG_VERSION"),
                        target,
                    )?
                } else {
                    let source = self_update::HttpsCandidateSource::github_release()?;
                    self_update::stage_verified_candidate(
                        &repo_root,
                        &source,
                        &inspector,
                        env!("CARGO_PKG_VERSION"),
                        target,
                    )?
                };
                let handoff = self_update::prepare_runtime_handoff(
                    &repo_root,
                    &candidate,
                    &std::env::current_exe()?,
                )?;
                let adapter_names = adapters
                    .iter()
                    .map(|adapter| adapter_name(*adapter).to_string())
                    .collect::<Vec<_>>();
                let (paths, _) = update_transaction::create_verified_transaction(
                    &repo_root,
                    &config,
                    &candidate,
                    env!("CARGO_PKG_VERSION"),
                    &adapter_names,
                )?;
                println!("# Baron Verified Update Candidate\n");
                println!("- Version: `{}`", candidate.version);
                println!("- Target: `{}`", candidate.target);
                println!("- Source revision: `{}`", candidate.source_revision);
                println!("- Staged path: `{}`", candidate.staged_path.display());
                println!(
                    "- Handoff prepared: `{}`",
                    self_update::handoff_label(&handoff)
                );
                println!("- Transaction state: `{}`", paths.state_path.display());
                println!("- Runtime activation: not performed");
                println!("- Project managed files: unchanged");
            } else if dry_run {
                let payloads = adapters
                    .iter()
                    .map(|adapter| managed_payloads_for_adapter(*adapter))
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>();
                let plan = plan_managed_update(&repo_root, &payloads)?;
                let candidate_label = if installed {
                    "currently installed embedded assets"
                } else {
                    "running Baron embedded assets"
                };
                print!(
                    "{}",
                    render_safe_update_preview(
                        &config.project_slug,
                        &names,
                        candidate_label,
                        &plan
                    )
                );
            } else {
                let running_binary = std::env::current_exe().context(
                    "Could not resolve the installed Baron runtime for the verified update protocol",
                )?;
                let pending =
                    update_transaction::pending_transaction(&repo_root, &config.project_id)?;
                let (state_path, mut transaction, candidate) = if let Some(pending) = pending {
                    let candidate = update_transaction::candidate_for_transaction(
                        &repo_root,
                        &pending.state_path,
                        &config.project_id,
                    )?;
                    (pending.state_path, pending.transaction, candidate)
                } else {
                    let target = self_update::current_release_target()?;
                    let source = self_update::HttpsCandidateSource::github_release()?;
                    let candidate = self_update::stage_verified_candidate(
                        &repo_root,
                        &source,
                        &self_update::ProcessBinaryInspector,
                        env!("CARGO_PKG_VERSION"),
                        target,
                    )?;
                    let adapter_names = adapters
                        .iter()
                        .map(|adapter| adapter_name(*adapter).to_string())
                        .collect::<Vec<_>>();
                    let (paths, transaction) = update_transaction::create_verified_transaction(
                        &repo_root,
                        &config,
                        &candidate,
                        env!("CARGO_PKG_VERSION"),
                        &adapter_names,
                    )?;
                    (paths.state_path, transaction, candidate)
                };

                if transaction.status == update_transaction::TransactionStatus::Verified {
                    self_update::invoke_verified_candidate(
                        &candidate,
                        &repo_root,
                        &candidate_plan_arguments(&repo_root, &state_path, &transaction.adapters)?,
                    )?;
                    transaction = update_transaction::inspect_transaction(
                        &repo_root,
                        &state_path,
                        &config.project_id,
                    )?;
                }

                if transaction.status == update_transaction::TransactionStatus::Conflict {
                    println!("# Baron Update Needs Review\n");
                    println!("- Transaction: `{}`", transaction.transaction_id);
                    println!("- Project managed assets: unchanged");
                    println!("- Runtime: unchanged");
                    println!(
                        "- Staged conflict packets: `{}`",
                        state_path
                            .parent()
                            .expect("transaction state has parent")
                            .display()
                    );
                    println!("- Next: resolve the staged `RESOLVED/` packets with an agent, then run `baron update` again.");
                    return Ok(());
                }
                if transaction.status != update_transaction::TransactionStatus::Planned {
                    bail!(
                        "Baron update candidate did not produce an actionable planned transaction; current status is `{}`",
                        transaction.status.as_str()
                    );
                }

                self_update::invoke_verified_candidate(
                    &candidate,
                    &repo_root,
                    &candidate_continue_arguments(&repo_root, &state_path, &running_binary),
                )?;
                let completed = update_transaction::inspect_transaction(
                    &repo_root,
                    &state_path,
                    &config.project_id,
                )?;
                println!("# Baron Update\n");
                println!("- Project: `{}`", config.project_slug);
                println!("- Transaction: `{}`", completed.transaction_id);
                println!("- Project managed assets: activated transactionally");
                match completed.status {
                    update_transaction::TransactionStatus::Completed => {
                        println!("- Runtime: verified and activated");
                        println!("- Status: `completed`");
                    }
                    update_transaction::TransactionStatus::RuntimePending => {
                        println!("- Runtime: delayed Windows finalizer will activate after this Baron process exits");
                        println!("- Status: `runtime_pending`");
                    }
                    status => bail!(
                        "Baron candidate returned without a completed or pending runtime handoff; current status is `{}`",
                        status.as_str()
                    ),
                }
            }
        }
        Some(Commands::Memory { command }) => match command {
            MemoryCommands::Status { repo_path, vault } => {
                let repo_path = resolve_repo_root(repo_path.unwrap_or(std::env::current_dir()?))?;
                let vault_path = resolve_command_vault(vault, &repo_path)?;
                print_memory_status(repo_path, vault_path)?;
            }
            MemoryCommands::Index { repo_path, vault } => {
                let repo_path = resolve_repo_root(repo_path.unwrap_or(std::env::current_dir()?))?;
                let vault_path = resolve_command_vault(vault, &repo_path)?;
                let context = ensure_vault(vault_path, repo_path)?;
                let report = build_memory_index(&context)?;
                print!("{}", render_memory_index(&context, &report));
            }
            MemoryCommands::Compact { repo_path, vault } => {
                let repo_path = resolve_repo_root(repo_path.unwrap_or(std::env::current_dir()?))?;
                let vault_path = resolve_command_vault(vault, &repo_path)?;
                let context = coherent_or_bootstrap_context(&repo_path, &vault_path)?;
                build_memory_index(&context)?;
                print!("{}", compact_memory_brief(&context)?);
            }
            MemoryCommands::ImportSessions { repo_path, vault } => {
                let repo_path = resolve_repo_root(repo_path.unwrap_or(std::env::current_dir()?))?;
                let vault_path = resolve_command_vault(vault, &repo_path)?;
                let context = coherent_or_bootstrap_context(&repo_path, &vault_path)?;
                let report = import_sessions(&repo_path, &context, 20)?;
                build_memory_index(&context)?;
                println!("# Baron Session Import\n");
                println!("- Roots checked: {}", report.roots_checked);
                println!("- Files checked: {}", report.files_checked);
                println!("- Imported: {}", report.imported);
                println!("- Deduplicated: {}", report.deduplicated);
                println!("- Skipped unmatched: {}", report.skipped_unmatched);
                println!("- Skipped noise: {}", report.skipped_noise);
                println!("- State: `{}`", report.state_path.display());
            }
        },
        Some(Commands::Recall {
            query,
            repo_path,
            vault,
        }) => {
            let repo_path = resolve_repo_root(repo_path.unwrap_or(std::env::current_dir()?))?;
            let vault_path = resolve_command_vault(vault, &repo_path)?;
            let context = coherent_or_bootstrap_context(&repo_path, &vault_path)?;
            build_memory_index(&context)?;
            print!("{}", render_recall(&recall(&context, &query, 8)?));
        }
        Some(Commands::Context {
            repo_path,
            codex,
            claude,
            agent,
            why,
            task,
            vault,
        }) => {
            let repo_path = resolve_repo_root(repo_path.unwrap_or(std::env::current_dir()?))?;
            let vault_path = resolve_command_vault(vault, &repo_path)?;
            let default = load_project_config(&repo_path)
                .ok()
                .and_then(|config| config.adapters.first().copied())
                .map(agent_adapter)
                .map(context_target);
            let target = parse_context_target(codex, claude, agent, why, default)?;
            let vault_context = coherent_or_bootstrap_context(&repo_path, &vault_path)?;
            if why {
                print!("{}", compile_context_why(repo_path, vault_path, target)?);
            } else {
                let output =
                    compile_context_for_task(&repo_path, &vault_path, target, task.as_deref())?;
                record_lifecycle_event(
                    &vault_context,
                    hook_adapter_for_repo(&repo_path),
                    AutomationEvent::ContextCompiled,
                )?;
                print!("{}", output);
            }
        }
        Some(Commands::Plan { command }) => match command {
            PlanCommands::Status { repo_path } => {
                let repo_root = configured_repo(repo_path)?;
                print!("{}", plan_status(repo_root)?);
            }
            PlanCommands::Start { title, repo_path } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                let plan = start_or_resume_plan(&repo_root, &vault, &title)?;
                record_lifecycle_event(
                    &vault,
                    hook_adapter_for_repo(&repo_root),
                    AutomationEvent::PlanStarted,
                )?;
                println!("# Baron Plan Start\n");
                println!("- Title: {}", plan.title);
                println!("- Risk: `{}`", plan.risk.as_str());
                println!(
                    "- Action: {}",
                    if plan.resumed { "resumed" } else { "created" }
                );
                println!("- Plan: `{}`", plan.repo_path.display());
            }
            PlanCommands::Update { note, repo_path } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                update_plan(&repo_root, &vault, &note)?;
                println!("# Baron Plan Update\n\n- Progress recorded.");
            }
            PlanCommands::Interrupt { state, repo_path } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                interrupt_plan(&repo_root, &vault, &state)?;
                println!("# Baron Plan Interrupt\n\n- Last known state recorded.");
            }
            PlanCommands::Complete {
                verification,
                repo_path,
            } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                complete_plan(&repo_root, &vault, &verification)?;
                println!("# Baron Plan Complete\n\n- Completion gate passed.");
            }
        },
        Some(Commands::Harness { command }) => match command {
            HarnessCommands::Status { repo_path } => {
                let repo_root = configured_repo(repo_path)?;
                print!("{}", harness_status(repo_root)?);
            }
            HarnessCommands::IntentStatus { repo_path } => {
                let repo_root = configured_repo(repo_path)?;
                print!("{}", intent_status(repo_root)?);
            }
            HarnessCommands::Intent {
                title,
                repo_path,
                current_behavior,
                target_behavior,
                scope,
                non_goals,
                constraint,
                decision,
                required_proof,
                unknowns,
                confirmed,
            } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                let brief = record_intent(
                    &repo_root,
                    &vault,
                    IntentBriefInput {
                        title,
                        current_behavior,
                        target_behavior,
                        scope,
                        non_goals,
                        constraints: constraint,
                        decisions: decision,
                        required_proof,
                        unknowns,
                        confirmed,
                    },
                )?;
                println!("# Baron Harness Intent\n");
                println!("- Intent ID: `{}`", brief.id);
                println!("- Title: {}", brief.title);
                println!("- Risk: `{}`", brief.risk.as_str());
                println!(
                    "- Confirmation: `{}`",
                    if brief.confirmed {
                        "confirmed"
                    } else {
                        "needs_confirmation"
                    }
                );
                println!(
                    "- Action: {}",
                    if brief.resumed { "resumed" } else { "created" }
                );
                println!("- Repo: `{}`", brief.repo_path.display());
                println!("- Vault: `{}`", brief.vault_path.display());
            }
            HarnessCommands::Audit { repo_path } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                let audit = audit_harness(&repo_root, &vault)?;
                println!("# Baron Harness Audit\n");
                println!("- Context-read score: {}", audit.context_read_score);
                println!("- Open friction: {}", audit.open_friction_count);
                println!("- Diagnostics: {}", list_or_none(&audit.diagnostics));
            }
            HarnessCommands::VerifyAll { repo_path, limit } => {
                let repo_root = configured_repo(repo_path)?;
                let report = verify_open_stories(&repo_root, limit)?;
                println!("# Baron Harness Story Verification\n");
                println!("- Checked stories: {}", report.checked_count);
                println!("- Proof gaps: {}", list_or_none(&report.proof_gaps));
            }
            HarnessCommands::Intake { title, repo_path } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                let story = start_or_resume_intake(&repo_root, &vault, &title)?;
                record_lifecycle_event(
                    &vault,
                    hook_adapter_for_repo(&repo_root),
                    AutomationEvent::HarnessStarted,
                )?;
                println!("# Baron Harness Intake\n");
                println!("- Title: {}", story.title);
                println!("- Risk: `{}`", story.risk.as_str());
                println!(
                    "- Action: {}",
                    if story.resumed { "resumed" } else { "created" }
                );
            }
            HarnessCommands::Decision { summary, repo_path } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                record_decision(&repo_root, &vault, &summary)?;
                println!("# Baron Harness Decision\n\n- Decision recorded.");
            }
            HarnessCommands::Friction { summary, repo_path } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                record_friction(&repo_root, &vault, &summary)?;
                println!("# Baron Harness Friction\n\n- Friction recorded.");
            }
            HarnessCommands::Intervention { summary, repo_path } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                let record = record_intervention(&repo_root, &vault, &summary)?;
                println!("# Baron Harness Intervention\n");
                println!("- Intervention recorded");
                println!("- Repo: `{}`", record.repo_path.display());
                println!("- Vault: `{}`", record.vault_path.display());
            }
            HarnessCommands::Propose { repo_path } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                let proposal = propose_improvements(&repo_root, &vault)?;
                println!("# Baron Harness Improvement Proposals\n");
                println!("- Proposals: {}", proposal.proposal_count);
                println!("- IDs: {}", list_or_none(&proposal.proposal_ids));
                println!("- Human approval: human approval required before core policy or architecture changes");
                println!("- Repo: `{}`", proposal.repo_path.display());
                println!("- Vault: `{}`", proposal.vault_path.display());
            }
            HarnessCommands::Outcome {
                proposal_id,
                outcome,
                repo_path,
            } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                record_improvement_outcome(&repo_root, &vault, &proposal_id, &outcome)?;
                println!("# Baron Harness Improvement Outcome\n");
                println!("- Outcome recorded");
                println!("- Proposal: `{proposal_id}`");
            }
        },
        Some(Commands::Proof { command }) => match command {
            ProofCommands::Status { repo_path } => {
                let repo_root = configured_repo(repo_path)?;
                print!("{}", proof_status(repo_root)?);
            }
            ProofCommands::Record {
                summary,
                repo_path,
                capability_evidence,
            } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                let capability_evidence = capability_evidence
                    .iter()
                    .map(|value| parse_capability_evidence(value))
                    .collect::<Result<Vec<_>>>()?;
                let proof = if capability_evidence.is_empty() {
                    record_proof(&repo_root, &vault, &summary)?
                } else {
                    record_proof_with_capabilities(
                        &repo_root,
                        &vault,
                        &summary,
                        &capability_evidence,
                    )?
                };
                record_lifecycle_event(
                    &vault,
                    hook_adapter_for_repo(&repo_root),
                    AutomationEvent::ProofRecorded,
                )?;
                println!("# Baron Proof Record\n");
                println!("- Proof ID: `{}`", proof.id);
                println!("- Evidence: {}", proof.summary);
                println!(
                    "- Capability gate: `{}`",
                    if proof.capability_gate_passed {
                        "passed"
                    } else {
                        "failed"
                    }
                );
                if !proof.capability_gaps.is_empty() {
                    println!("- Capability gaps: {}", proof.capability_gaps.join(", "));
                }
            }
        },
        Some(Commands::Trace { command }) => match command {
            TraceCommands::Record {
                summary,
                repo_path,
                outcome,
            } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                let trace = record_trace(&repo_root, &vault, &summary, outcome.into())?;
                println!("# Baron Trace Record\n");
                println!("- Trace ID: `{}`", trace.id);
                println!("- Score status: `unscored`");
            }
            TraceCommands::Score { repo_path, id } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                let score = score_trace(&repo_root, &vault, id.as_deref())?;
                record_lifecycle_event(
                    &vault,
                    hook_adapter_for_repo(&repo_root),
                    AutomationEvent::TraceScored,
                )?;
                println!("# Baron Trace Score\n");
                println!("- Achieved: `{}`", score.achieved.as_str());
                println!("- Required: `{}`", score.required.as_str());
                println!("- Passed: `{}`", if score.passed { "yes" } else { "no" });
                println!(
                    "- Missing: {}",
                    if score.missing_fields.is_empty() {
                        "none".to_string()
                    } else {
                        score.missing_fields.join(", ")
                    }
                );
                println!(
                    "- Warnings: {}",
                    if score.warnings.is_empty() {
                        "none".to_string()
                    } else {
                        score.warnings.join(", ")
                    }
                );
                if !score.passed {
                    bail!(
                        "Trace quality gate failed: required `{}`, achieved `{}`.",
                        score.required.as_str(),
                        score.achieved.as_str()
                    );
                }
            }
        },
        Some(Commands::Review { command }) => match command {
            ReviewCommands::Status { repo_path } => {
                let repo_root = configured_repo(repo_path)?;
                print!("{}", review_status(repo_root)?);
            }
            ReviewCommands::Finding {
                summary,
                repo_path,
                severity,
                evidence,
                affected_files,
            } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                let finding = record_finding(
                    &repo_root,
                    &vault,
                    ReviewFindingInput {
                        severity,
                        summary,
                        evidence,
                        affected_files,
                    },
                )?;
                println!("# Baron Review Finding\n");
                println!("- Finding ID: `{}`", finding.id);
                println!("- Status: `open`");
            }
            ReviewCommands::Close {
                id,
                repo_path,
                fix_evidence,
                verification,
            } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                close_finding(&repo_root, &vault, &id, &fix_evidence, &verification)?;
                println!("# Baron Review Finding Closure\n");
                println!("- Finding ID: `{id}`");
                println!("- Status: `closed`");
            }
        },
        Some(Commands::Migrate { command }) => match command {
            MigrationCommands::AgentBootstrap {
                repo_path,
                dry_run,
                vault,
            } => {
                let repo_path = repo_path.unwrap_or(std::env::current_dir()?);
                if dry_run {
                    let inventory = inventory_agent_bootstrap(&repo_path, vault.as_deref())?;
                    print!("{}", render_migration_inventory(&inventory));
                } else {
                    let receipt = execute_agent_bootstrap_migration(
                        &repo_path,
                        vault.as_deref(),
                        |repo_root, vault_root| {
                            initialize_project(repo_root, AdapterKind::Codex, vault_root)?;
                            install_adapter(repo_root, AgentAdapter::Codex)?;
                            let context = ensure_vault(vault_root, repo_root)?;
                            build_memory_index(&context)?;
                            Ok(())
                        },
                    )?;
                    println!("# Baron Agent Bootstrap Migration\n");
                    println!("- Migration ID: `{}`", receipt.migration_id);
                    println!("- Status: `{}`", receipt.status);
                    println!("- Imported: {}", receipt.imported_count);
                    println!("- Quarantined: {}", receipt.quarantined_count);
                    println!("- Removed: {}", receipt.removed_count);
                    println!("- Backup: `{}`", receipt.backup_root.display());
                    println!("- Runtime dependency on Agent Bootstrap: none");
                }
            }
            MigrationCommands::Status { repo_path } => {
                let repo_path = repo_path.unwrap_or(std::env::current_dir()?);
                print!("{}", migration_status(repo_path)?);
            }
            MigrationCommands::Rollback {
                id,
                repo_path,
                vault,
            } => {
                let repo_path = repo_path.unwrap_or(std::env::current_dir()?);
                let vault = if let Some(vault) = vault {
                    vault
                } else {
                    resolve_vault_path_for_repo(None, &repo_path)?
                };
                let report = rollback_migration(&repo_path, &vault, &id)?;
                println!("# Baron Migration Rollback\n");
                println!("- Migration ID: `{}`", report.migration_id);
                println!("- Status: `{}`", report.status);
                println!("- Restored paths: {}", report.restored_count);
            }
        },
        Some(Commands::Capability { command }) => match command {
            CapabilityCommands::Register {
                capability,
                repo_path,
                name,
                kind,
                required,
                command,
                scan_target,
                adapters,
                description,
            } => {
                let repo_root = configured_repo(repo_path)?;
                let normalized_name = baron_core::capability::normalize_identifier(&name)
                    .context("Provider name must contain letters or numbers")?;
                let provider = CapabilityProvider {
                    name,
                    capability,
                    kind: kind.into(),
                    requirement: if required {
                        Requirement::Required
                    } else {
                        Requirement::Optional
                    },
                    command,
                    scan_target,
                    adapters: adapters.into_iter().map(Into::into).collect(),
                    description,
                };
                let registry = register_provider(&repo_root, provider)?;
                let registered = registry
                    .providers
                    .iter()
                    .find(|provider| provider.name == normalized_name)
                    .context("Provider was not registered")?;
                println!("# Baron Capability Register\n");
                println!("- Capability: `{}`", registered.capability);
                println!("- Provider: `{}`", registered.name);
                println!("- Kind: `{}`", provider_kind_name(registered.kind));
                println!(
                    "- Requirement: `{}`",
                    requirement_name(registered.requirement)
                );
            }
            CapabilityCommands::Check {
                capability,
                repo_path,
                adapter,
                json,
            } => {
                let repo_root = configured_repo(repo_path)?;
                let adapter = resolve_capability_adapter(&repo_root, adapter)?;
                let state = check_capabilities(
                    &repo_root,
                    CheckOptions {
                        adapter,
                        capability,
                        allow_network: true,
                    },
                )?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&state)?);
                } else {
                    print!("{}", render_capability_check(&state));
                }
            }
            CapabilityCommands::List {
                repo_path,
                adapter,
                json,
            } => {
                let repo_root = configured_repo(repo_path)?;
                let registry = load_registry(&repo_root)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&registry)?);
                } else {
                    let adapter = resolve_capability_adapter(&repo_root, adapter)?;
                    let state = load_capability_state(&repo_root)?;
                    print!(
                        "{}",
                        render_capability_list(&registry, state.as_ref(), adapter)
                    );
                }
            }
            CapabilityCommands::Remove {
                capability,
                repo_path,
                name,
            } => {
                let repo_root = configured_repo(repo_path)?;
                let removed = remove_provider(&repo_root, &capability, &name)?;
                println!("# Baron Capability Remove\n");
                println!("- Capability: `{}`", capability);
                println!("- Provider: `{}`", name);
                println!("- Removed: `{}`", if removed { "yes" } else { "no" });
            }
        },
        Some(Commands::ControlPlane { command }) => match command {
            ControlPlaneCommands::Status { repo_path } => {
                let repo_root = configured_repo(repo_path)?;
                let report = validate_control_plane(&repo_root)?;
                println!("# Baron Control Plane Status\n");
                println!("- Passed: `{}`", if report.passed { "yes" } else { "no" });
                println!(
                    "- Workflow owner: `{}`",
                    report
                        .workflow_owner
                        .unwrap_or_else(|| "missing".to_string())
                );
                println!(
                    "- Mandatory agents: {}",
                    if report.mandatory_agents.is_empty() {
                        "none".to_string()
                    } else {
                        report.mandatory_agents.join(", ")
                    }
                );
                println!("- Diagnostics: {}", list_or_none(&report.diagnostics));
            }
            ControlPlaneCommands::Route {
                task,
                repo_path,
                risk,
            } => {
                let repo_root = configured_repo(repo_path)?;
                let route = route_task(&repo_root, &task, risk.into())?;
                println!("# Baron Control Plane Route\n");
                println!("- Task: `{}`", task);
                println!("- Explanation: {}", route.explanation);
                println!("\n## Selected Skills\n");
                for skill in &route.selected_skills {
                    println!("- `{}`: {}", skill.name, skill.reason);
                }
                println!("\n## Mandatory Agent Gates\n");
                for agent in &route.mandatory_agents {
                    println!("- `{agent}`");
                }
                println!("\n## Optional Agents\n");
                if route.optional_agents.is_empty() {
                    println!("- none");
                } else {
                    for agent in &route.optional_agents {
                        println!("- `{}`: {}", agent.name, agent.reason);
                    }
                }
                println!("\n## Skipped\n");
                if route.skipped.is_empty() {
                    println!("- none");
                } else {
                    for skipped in &route.skipped {
                        println!("- {skipped}");
                    }
                }
            }
            ControlPlaneCommands::RecordGate {
                agent,
                summary,
                repo_path,
            } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                let evidence = record_gate_evidence(&repo_root, &vault, &agent, &summary)?;
                println!("# Baron Control Plane Gate Evidence\n");
                println!("- Gate evidence recorded");
                println!("- Agent: `{}`", evidence.agent);
                println!("- Repo: `{}`", evidence.repo_path.display());
                println!("- Vault: `{}`", evidence.vault_path.display());
            }
            ControlPlaneCommands::Evidence {
                repo_path,
                required,
            } => {
                let repo_root = configured_repo(repo_path)?;
                let required = if required.is_empty() {
                    vec![
                        "code-reviewer".to_string(),
                        "security-auditor".to_string(),
                        "test-engineer".to_string(),
                    ]
                } else {
                    required
                };
                let status = gate_evidence_status(&repo_root, &required)?;
                println!("# Baron Control Plane Evidence\n");
                println!("- Passed: `{}`", if status.passed { "yes" } else { "no" });
                println!("- Required: {}", required.join(", "));
                println!("- Missing: {}", list_or_none(&status.missing_agents));
            }
        },
        Some(Commands::Asset { command }) => match command {
            AssetCommands::Audit { repo_path } => {
                let repo_root = configured_repo(repo_path)?;
                let report = audit_runtime_assets(&repo_root)?;
                println!("# Baron Asset Audit\n");
                println!("- Passed: `{}`", if report.passed { "yes" } else { "no" });
                println!("- Items: {}", report.items.len());
                println!("- Diagnostics: {}", list_or_none(&report.diagnostics));
            }
            AssetCommands::Quarantine { repo_path } => {
                let repo_root = configured_repo(repo_path)?;
                let report = quarantine_failing_assets(&repo_root)?;
                println!("# Baron Asset Quarantine\n");
                println!("- Quarantined: {}", report.quarantined.len());
                println!(
                    "- Managed failures skipped: {}",
                    report.skipped_managed.len()
                );
                println!(
                    "- Quarantine root: `{}`",
                    repo_root
                        .join(".baron/quarantine/asset-lifecycle")
                        .display()
                );
            }
            AssetCommands::ProposeSkill {
                skill,
                reason,
                content_path,
                repo_path,
            } => {
                let repo_root = configured_repo(repo_path)?;
                let content = std::fs::read_to_string(&content_path).with_context(|| {
                    format!("Could not read proposal body: {}", content_path.display())
                })?;
                let staged = stage_skill_update(&repo_root, &skill, &reason, &content)?;
                println!("# Baron Skill Update Proposal\n");
                println!("- Skill: `{skill}`");
                println!("- Approval required: `yes`");
                println!("- Proposal: `{}`", staged.proposal_path.display());
                println!("- Diff: `{}`", staged.diff_path.display());
                println!("- Metadata: `{}`", staged.metadata_path.display());
            }
        },
        Some(Commands::SessionReplay { command }) => match command {
            SessionReplayCommands::Index { repo_path, vault } => {
                let repo_root = resolve_repo_root(repo_path.unwrap_or(std::env::current_dir()?))?;
                let vault_path = resolve_command_vault(vault, &repo_root)?;
                let context = coherent_or_bootstrap_context(&repo_root, &vault_path)?;
                let report = index_session_replay(&context)?;
                println!("# Baron Session Replay Index\n");
                println!("- Sources: {}", report.indexed_sources);
                println!("- Messages: {}", report.indexed_messages);
                println!("- Index: `{}`", report.index_path.display());
            }
            SessionReplayCommands::Search {
                query,
                repo_path,
                vault,
                limit,
            } => {
                let repo_root = resolve_repo_root(repo_path.unwrap_or(std::env::current_dir()?))?;
                let vault_path = resolve_command_vault(vault, &repo_root)?;
                let context = coherent_or_bootstrap_context(&repo_root, &vault_path)?;
                index_session_replay(&context)?;
                let hits = search_session_replay(&context, &query, limit)?;
                println!("# Baron Session Replay Search\n");
                println!("- Query: `{query}`");
                println!("- Hits: {}", hits.len());
                for hit in hits {
                    println!(
                        "- `{}` {} {}: {}",
                        hit.message_id,
                        hit.source_path,
                        hit.role,
                        hit.text.split_whitespace().collect::<Vec<_>>().join(" ")
                    );
                }
            }
            SessionReplayCommands::Replay {
                message_id,
                repo_path,
                vault,
                radius,
            } => {
                let repo_root = resolve_repo_root(repo_path.unwrap_or(std::env::current_dir()?))?;
                let vault_path = resolve_command_vault(vault, &repo_root)?;
                let context = coherent_or_bootstrap_context(&repo_root, &vault_path)?;
                let replay = replay_session_context(&context, &message_id, radius)?;
                println!("# Baron Session Replay\n");
                println!("- Project: `{}`", replay.project_slug);
                println!("- Source: `{}`", replay.source_path);
                for message in replay.messages {
                    println!(
                        "\n## {} {}\n\n{}",
                        message.ordinal, message.role, message.text
                    );
                }
            }
        },
        Some(Commands::Autopilot { command }) => match command {
            AutopilotCommands::Status { repo_path } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                print!("{}", autopilot_status(&repo_root, &vault)?);
            }
            AutopilotCommands::Review { summary, repo_path } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                let review = review_after_task(&repo_root, &vault, &summary)?;
                println!("# Baron Autopilot Review\n");
                println!("- Candidate count: {}", review.candidate_count);
                println!(
                    "- Approval required: `{}`",
                    if review.approval_required {
                        "yes"
                    } else {
                        "no"
                    }
                );
                println!("- Candidate IDs: {}", list_or_none(&review.candidate_ids));
                println!(
                    "- Observed automation: {}",
                    list_or_none(&review.observed_automation)
                );
                println!("- Repo candidates: `{}`", review.repo_path.display());
                println!("- Vault candidates: `{}`", review.vault_path.display());
            }
            AutopilotCommands::Approve {
                candidate_id,
                repo_path,
            } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                approve_candidate(&repo_root, &vault, &candidate_id)?;
                println!("# Baron Autopilot Candidate Approval\n");
                println!("- Candidate: `{candidate_id}`");
                println!("- Status: `approved`");
                println!("- Trusted fact: `not automatic`");
            }
            AutopilotCommands::Reject {
                candidate_id,
                repo_path,
            } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                reject_candidate(&repo_root, &vault, &candidate_id)?;
                println!("# Baron Autopilot Candidate Rejection\n");
                println!("- Candidate: `{candidate_id}`");
                println!("- Status: `rejected`");
            }
        },
        Some(Commands::Runtime { command }) => match command {
            RuntimeCommands::Check {
                repo_path,
                adapter,
                json,
            } => {
                let repo_root = configured_repo(repo_path)?;
                let adapter = resolve_capability_adapter(&repo_root, adapter)?;
                let report = runtime_backend_report(&repo_root, adapter)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    print!("{}", render_runtime_check(&report));
                }
            }
        },
        Some(Commands::Certify { command }) => match command {
            CertifyCommands::Run {
                repo_path,
                vault,
                profile,
            } => {
                let repo_root = resolve_repo_root(repo_path.unwrap_or(std::env::current_dir()?))?;
                let report = run_certification(&repo_root, &vault, profile.into())?;
                print!("{}", render_certification_report(&report));
                println!("- Markdown report: `{}`", report.markdown_path.display());
                println!("- JSON report: `{}`", report.json_path.display());
                if !report.passed {
                    bail!("Baron certification failed");
                }
            }
            CertifyCommands::Status { repo_path } => {
                let repo_root = resolve_repo_root(repo_path.unwrap_or(std::env::current_dir()?))?;
                print!("{}", latest_certification_status(repo_root)?);
            }
        },
        Some(Commands::Automation { command }) => match command {
            AutomationCommands::Status { repo_path } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                print!("{}", automation_status(&repo_root, &vault)?);
            }
            AutomationCommands::Reconcile { repo_path } => {
                let repo_root = configured_repo(repo_path)?;
                let config = load_project_config(&repo_root)?;
                let payloads = config
                    .adapters
                    .iter()
                    .map(|adapter| managed_payloads_for_adapter(agent_adapter(*adapter)))
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>();
                let assets = reconcile_installed_managed_assets(
                    &repo_root,
                    &payloads,
                    env!("CARGO_PKG_VERSION"),
                )?;
                let automation_evidence_recorded = match execution_context(Some(repo_root.clone()))
                {
                    Ok((_, vault)) => {
                        // Reconcile is the local managed-refresh path. Restore only
                        // missing Baron-owned support documents before recording the
                        // checkpoint; each helper preserves user-written content.
                        ensure_platform_intelligence(&repo_root, &config)?;
                        ensure_architecture_governor(&repo_root, &config)?;
                        ensure_harness_workspace(&repo_root, &vault)?;
                        ensure_code_map_capability(&repo_root)?;
                        record_lifecycle_event(
                            &vault,
                            hook_adapter_for_repo(&repo_root),
                            AutomationEvent::Checkpoint,
                        )?;
                        true
                    }
                    Err(_) => false,
                };
                let report = reconcile(&repo_root)?;
                let applied = assets
                    .applied_paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>();
                let conflicts = assets
                    .conflicts
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>();
                println!("# Baron Automation Reconciliation\n");
                println!("- Passed: `{}`", if report.passed { "yes" } else { "no" });
                println!("- Active plan: `{}`", report.active_plan);
                println!("- Gaps: {}", list_or_none(&report.gaps));
                println!(
                    "- Local managed assets repaired: {}",
                    list_or_none(&applied)
                );
                println!(
                    "- Managed conflicts preserved: {}",
                    list_or_none(&conflicts)
                );
                println!(
                    "- Automation evidence recorded: {}",
                    if automation_evidence_recorded {
                        "yes"
                    } else {
                        "no (state is not coherent)"
                    }
                );
                println!("- Remote release download: not attempted");
                println!("- Runtime replacement: not attempted");
            }
            AutomationCommands::CodeMap { command } => match command {
                CodeMapCommands::Status { repo_path, json } => {
                    let repo_root = configured_repo(repo_path)?;
                    let report = code_map_status(&repo_root)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    } else {
                        print!("{}", render_code_map_status(&report));
                    }
                }
                CodeMapCommands::Refresh { repo_path, json } => {
                    let repo_root = configured_repo(repo_path)?;
                    let provider = GraphifyProvider::new("graphify");
                    let cache_root = code_graph_cache_root(&repo_root)?;
                    let state = provider.refresh(&repo_root, &cache_root)?;
                    let report = CodeMapRefreshReport {
                        provider: "graphify-local".to_string(),
                        version: state.provider_version.clone(),
                        source_fingerprint: state.source_fingerprint.clone(),
                        graph_size_bytes: state.graph_size_bytes,
                        action: "query".to_string(),
                    };
                    if json {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    } else {
                        println!("# Baron Code Map Refresh\n");
                        println!("- Provider: `{}`", report.provider);
                        println!("- Version: `{}`", report.version);
                        println!("- Graph size: {} bytes", report.graph_size_bytes);
                        println!("- Next AI action: query this task, then verify source.");
                    }
                }
                CodeMapCommands::Query {
                    question,
                    repo_path,
                    json,
                } => {
                    let repo_root = configured_repo(repo_path)?;
                    let provider = GraphifyProvider::new("graphify");
                    let cache_root = code_graph_cache_root(&repo_root)?;
                    let hits = provider.query(
                        &repo_root,
                        &cache_root,
                        &question,
                        QueryLimits::default(),
                    )?;
                    let state = load_code_graph_state(&repo_root)?.context(
                        "No local code map is available; Survey fallback remains active",
                    )?;
                    write_code_graph_query_cache(&repo_root, &state, &question, hits.clone())?;
                    let hits = hits
                        .into_iter()
                        .map(|hit| {
                            let verification = verify_graph_hit_source(&repo_root, &hit)?;
                            Ok(CodeMapQueryHit { hit, verification })
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let report = CodeMapQueryReport {
                        provider: "graphify-local".to_string(),
                        action: "verify_source".to_string(),
                        hits,
                    };
                    if json {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    } else {
                        print!("{}", render_code_map_query(&report));
                    }
                }
            },
            AutomationCommands::Hook {
                event,
                repo_path,
                adapter,
            } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                let mut payload = String::new();
                std::io::stdin().read_to_string(&mut payload)?;
                println!(
                    "{}",
                    handle_hook(&repo_root, &vault, adapter.into(), event.into(), &payload)?
                );
            }
        },
        Some(Commands::Continuity { command }) => match command {
            ContinuityCommands::Status { repo_path } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                print!("{}", continuity_status(&repo_root, &vault)?);
            }
            ContinuityCommands::Checkpoint { note, repo_path } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                let packet = record_continuity_checkpoint(
                    &repo_root,
                    &vault,
                    &note,
                    adapter_kind_name(
                        load_project_config(&repo_root)?
                            .adapters
                            .first()
                            .copied()
                            .unwrap_or(AdapterKind::Generic),
                    ),
                )?;
                println!("# Baron Continuity Checkpoint\n");
                println!("- Note: {}", note);
                println!("- Repo packet: `{}`", packet.repo_path.display());
                println!("- Vault packet: `{}`", packet.vault_path.display());
            }
            ContinuityCommands::Recover {
                root_cause,
                repo_path,
                outcome,
                last_successful_step,
                evidence,
                affected_files,
                next_action,
                retry_conditions,
            } => {
                let (repo_root, vault) = execution_context(repo_path)?;
                let outcome = match outcome {
                    RecoveryOutcomeArg::Failed => RecoveryOutcome::Failed,
                    RecoveryOutcomeArg::Blocked => RecoveryOutcome::Blocked,
                    RecoveryOutcomeArg::Interrupted => RecoveryOutcome::Interrupted,
                };
                let packet = record_recovery(
                    &repo_root,
                    &vault,
                    RecoveryInput {
                        outcome,
                        root_cause,
                        last_successful_step,
                        evidence,
                        affected_files,
                        next_action,
                        retry_conditions,
                    },
                )?;
                println!("# Baron Actionable Recovery\n");
                println!("- Recovery ID: `{}`", packet.id);
                println!("- Outcome: `{}`", packet.outcome.as_str());
                println!(
                    "- Action: {}",
                    if packet.resumed { "resumed" } else { "created" }
                );
                println!("- Repo: `{}`", packet.repo_path.display());
                println!("- Vault: `{}`", packet.vault_path.display());
            }
        },
        Some(Commands::Release { command }) => match command {
            ReleaseCommands::Metadata {
                artifacts_dir,
                release_version,
                source_revision,
            } => {
                let version =
                    release_version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
                let manifest = write_release_metadata(&artifacts_dir, &version, &source_revision)?;
                println!("# Baron Release Metadata\n");
                println!("- Release metadata generated");
                println!("- Version: `{}`", manifest.version);
                println!("- Source revision: `{}`", manifest.source_revision);
                println!("- Artifacts: {}", manifest.artifacts.len());
            }
            ReleaseCommands::Verify {
                artifacts_dir,
                expected_version,
                expected_source_revision,
            } => {
                let manifest = load_and_verify_release_metadata(&artifacts_dir)?;
                verify_release_identity(&manifest, &expected_version, &expected_source_revision)?;
                println!("# Baron Release Verification\n");
                println!("- Release assets verified");
                println!("- Version: `{}`", manifest.version);
                println!("- Source revision: `{}`", manifest.source_revision);
                println!("- Artifacts: {}", manifest.artifacts.len());
            }
        },
        None => {
            println!("{} {}", product_name(), phase());
            println!("Run `baron --help` for commands.");
        }
    }
    Ok(())
}

fn render_safe_update_preview(
    project_slug: &str,
    adapters: &str,
    candidate_label: &str,
    plan: &ManagedUpdatePlan,
) -> String {
    let mut output = String::from("# Baron Safe Update Preview\n\n");
    output.push_str(&format!("- Project: `{project_slug}`\n"));
    output.push_str(&format!("- Registered adapters: `{adapters}`\n"));
    output.push_str(&format!("- Candidate: {candidate_label}\n"));
    output.push_str("- No project files were written.\n");
    output.push_str("- Source, plans, Harness state, Vault memory, and custom assets are outside this preview.\n\n");

    let changes = plan
        .actions
        .iter()
        .filter(|action| action.disposition != UpdateDisposition::Identical)
        .collect::<Vec<_>>();
    output.push_str(&format!(
        "- Managed assets checked: {}\n",
        plan.actions.len()
    ));
    output.push_str(&format!(
        "- Conflicts requiring a decision: {}\n",
        plan.conflicts.len()
    ));
    output.push_str(&format!(
        "- User-owned paths preserved: {}\n",
        plan.preserved_paths.len()
    ));
    if changes.is_empty() {
        output
            .push_str("- Result: all managed assets already match the running Baron candidate.\n");
        return output;
    }
    output.push_str("- Actions needing attention:\n");
    for action in changes.iter().take(20) {
        output.push_str(&format!(
            "  - `[{}] {}`: `{}`\n",
            action.adapter,
            action.relative_path.display(),
            update_disposition_label(action.disposition)
        ));
    }
    if changes.len() > 20 {
        output.push_str(&format!(
            "  - {} more managed actions omitted from this bounded preview.\n",
            changes.len() - 20
        ));
    }
    output
}

#[cfg_attr(target_os = "windows", allow(dead_code))]
fn verify_activated_runtime_version(
    inspector: &dyn CandidateBinaryInspector,
    installed_binary: &Path,
    candidate_version: &str,
) -> Result<()> {
    let reported = inspector.reported_version(installed_binary)?;
    let expected = format!("baron {candidate_version}");
    if reported == expected {
        Ok(())
    } else {
        bail!("Activated Baron runtime reported `{reported}`; expected `{expected}`")
    }
}

fn update_disposition_label(disposition: UpdateDisposition) -> &'static str {
    match disposition {
        UpdateDisposition::TakeUpstream => "take_upstream",
        UpdateDisposition::KeepLocal => "keep_local",
        UpdateDisposition::Identical => "identical",
        UpdateDisposition::AutoMerge => "auto_merge",
        UpdateDisposition::Conflict => "conflict",
    }
}

fn resolve_capability_adapter(
    repo_root: &std::path::Path,
    requested: Option<AdapterArg>,
) -> Result<AdapterKind> {
    if let Some(adapter) = requested {
        return Ok(adapter.into());
    }
    load_project_config(repo_root)?
        .adapters
        .first()
        .copied()
        .context("No registered adapter is available for capability checks")
}

fn render_capability_check(state: &baron_core::capability::CapabilityState) -> String {
    let mut output = format!(
        "# Baron Capability Check\n\n- Adapter: `{}`\n- Checked: {}\n",
        adapter_kind_name(state.adapter),
        state.checked_at
    );
    if state.observations.is_empty() {
        output.push_str("- No providers registered.\n");
    }
    for observation in &state.observations {
        output.push_str(&format!(
            "\n## {} / {}\n\n- Kind: `{}`\n- Requirement: `{}`\n- Presence: `{}`\n- Compatible: `{}`\n- Evidence: {}\n",
            observation.capability,
            observation.provider,
            provider_kind_name(observation.kind),
            requirement_name(observation.requirement),
            presence_name(observation.presence),
            if observation.compatible { "yes" } else { "no" },
            observation.evidence
        ));
    }
    output.push_str(&format!(
        "\n- Required gaps: {}\n- Optional gaps: {}\n",
        list_or_none(&state.required_gaps),
        list_or_none(&state.optional_gaps)
    ));
    output
}

fn render_runtime_check(report: &baron_core::capability::RuntimeBackendReport) -> String {
    let mut output = format!(
        "# Baron Runtime Backend Check\n\n- Adapter: `{}`\n- Passed: `{}`\n- Rule: provider availability is not execution proof.\n",
        adapter_kind_name(report.adapter),
        if report.passed { "yes" } else { "no" }
    );
    if report.providers.is_empty() {
        output.push_str("- No providers registered.\n");
    }
    for provider in &report.providers {
        output.push_str(&format!(
            "\n## {} / {}\n\n- Kind: `{}`\n- Requirement: `{}`\n- Presence: `{}`\n- Compatible: `{}`\n- Policy: `{}`\n- Execution evidence: `{}`\n- Evidence: {}\n- Recommendation: {}\n",
            provider.capability,
            provider.provider,
            provider_kind_name(provider.kind),
            requirement_name(provider.requirement),
            presence_name(provider.presence),
            if provider.compatible { "yes" } else { "no" },
            backend_safety_name(provider.safety),
            presence_name(provider.execution_evidence),
            provider.evidence,
            provider.recommendation
        ));
    }
    output.push_str(&format!(
        "\n- Blocking gaps: {}\n- Warnings: {}\n- Recommendations: {}\n",
        list_or_none(&report.blocking_gaps),
        list_or_none(&report.warnings),
        list_or_none(&report.recommendations)
    ));
    output
}

fn render_capability_list(
    registry: &baron_core::capability::CapabilityRegistry,
    state: Option<&baron_core::capability::CapabilityState>,
    adapter: AdapterKind,
) -> String {
    let state = state.filter(|state| state.adapter == adapter);
    let mut output = format!(
        "# Baron Capability Registry\n\n- Adapter view: `{}`\n",
        adapter_kind_name(adapter)
    );
    if registry.providers.is_empty() {
        output.push_str("- No providers registered.\n");
        return output;
    }
    output.push_str(
        "\n| Capability | Provider | Kind | Requirement | Last presence | Compatible |\n| --- | --- | --- | --- | --- | --- |\n",
    );
    for provider in &registry.providers {
        let observation = state.and_then(|state| {
            state
                .observations
                .iter()
                .find(|item| item.provider == provider.name)
        });
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            provider.capability,
            provider.name,
            provider_kind_name(provider.kind),
            requirement_name(provider.requirement),
            observation
                .map(|item| presence_name(item.presence))
                .unwrap_or("unknown"),
            observation
                .map(|item| if item.compatible { "yes" } else { "no" })
                .unwrap_or("unknown")
        ));
    }
    output
}

fn backend_safety_name(safety: BackendSafety) -> &'static str {
    match safety {
        BackendSafety::Safe => "safe",
        BackendSafety::NeedsConfirmation => "needs_confirmation",
        BackendSafety::Unsafe => "unsafe",
        BackendSafety::Unknown => "unknown",
    }
}

fn provider_kind_name(kind: ProviderKind) -> &'static str {
    match kind {
        ProviderKind::Cli => "cli",
        ProviderKind::Binary => "binary",
        ProviderKind::Mcp => "mcp",
        ProviderKind::Skill => "skill",
        ProviderKind::Http => "http",
        ProviderKind::AgentAdapter => "agent_adapter",
    }
}

fn requirement_name(requirement: Requirement) -> &'static str {
    match requirement {
        Requirement::Optional => "optional",
        Requirement::Required => "required",
    }
}

fn presence_name(presence: Presence) -> &'static str {
    match presence {
        Presence::Present => "present",
        Presence::Missing => "missing",
        Presence::Unknown => "unknown",
    }
}

fn adapter_kind_name(adapter: AdapterKind) -> &'static str {
    match adapter {
        AdapterKind::Codex => "codex",
        AdapterKind::Claude => "claude",
        AdapterKind::Generic => "agent",
    }
}

fn list_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn parse_capability_evidence(value: &str) -> Result<CapabilityExecutionEvidence> {
    let mut parts = value.splitn(3, '|').map(str::trim);
    let capability = parts.next().unwrap_or_default();
    let provider = parts.next().unwrap_or_default();
    let summary = parts.next().unwrap_or_default();
    if capability.is_empty() || provider.is_empty() || summary.is_empty() {
        bail!("Capability evidence must use `<capability>|<provider>|<result summary>`.");
    }
    Ok(CapabilityExecutionEvidence {
        capability: capability.to_string(),
        provider: provider.to_string(),
        summary: summary.to_string(),
    })
}

fn parse_context_target(
    codex: bool,
    claude: bool,
    agent: bool,
    allow_default: bool,
    default: Option<ContextTarget>,
) -> Result<ContextTarget> {
    match (codex as u8) + (claude as u8) + (agent as u8) {
        1 if codex => Ok(ContextTarget::Codex),
        1 if claude => Ok(ContextTarget::Claude),
        1 if agent => Ok(ContextTarget::Generic),
        0 if allow_default => Ok(default.unwrap_or(ContextTarget::Generic)),
        0 if default.is_some() => Ok(default.expect("checked above")),
        0 => bail!("Choose one context target: --codex, --claude, or --agent."),
        _ => bail!("Choose only one context target: --codex, --claude, or --agent."),
    }
}

fn selected_adapter(codex: bool, claude: bool, agent: bool) -> Result<Option<AgentAdapter>> {
    match (codex as u8) + (claude as u8) + (agent as u8) {
        1 if codex => Ok(Some(AgentAdapter::Codex)),
        1 if claude => Ok(Some(AgentAdapter::Claude)),
        1 if agent => Ok(Some(AgentAdapter::Generic)),
        0 => Ok(None),
        _ => bail!("Choose only one adapter: --codex, --claude, or --agent."),
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_platform(
    frontend: bool,
    backend: bool,
    fullstack: bool,
    mobile: bool,
    desktop: bool,
    tool: bool,
    library: bool,
    data: bool,
    cloud: bool,
    unknown: bool,
) -> Result<Option<ProjectPlatform>> {
    let selected = [
        (frontend, ProjectPlatform::Frontend),
        (backend, ProjectPlatform::Backend),
        (fullstack, ProjectPlatform::Fullstack),
        (mobile, ProjectPlatform::Mobile),
        (desktop, ProjectPlatform::Desktop),
        (tool, ProjectPlatform::Tool),
        (library, ProjectPlatform::Library),
        (data, ProjectPlatform::Data),
        (cloud, ProjectPlatform::Cloud),
        (unknown, ProjectPlatform::Unknown),
    ]
    .into_iter()
    .filter_map(|(enabled, platform)| enabled.then_some(platform))
    .collect::<Vec<_>>();
    match selected.len() {
        0 => Ok(None),
        1 => Ok(selected.first().copied()),
        _ => bail!("Choose at most one platform focus flag."),
    }
}

fn platform_name(platform: ProjectPlatform) -> &'static str {
    core_platform_name(platform)
}

fn platform_list(platforms: &[ProjectPlatform]) -> String {
    if platforms.is_empty() {
        "none".to_string()
    } else {
        platforms
            .iter()
            .map(|platform| format!("`{}`", platform_name(*platform)))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn adapter_kind(adapter: AgentAdapter) -> AdapterKind {
    match adapter {
        AgentAdapter::Codex => AdapterKind::Codex,
        AgentAdapter::Claude => AdapterKind::Claude,
        AgentAdapter::Generic => AdapterKind::Generic,
    }
}

fn agent_adapter(adapter: AdapterKind) -> AgentAdapter {
    match adapter {
        AdapterKind::Codex => AgentAdapter::Codex,
        AdapterKind::Claude => AgentAdapter::Claude,
        AdapterKind::Generic => AgentAdapter::Generic,
    }
}

fn context_target(adapter: AgentAdapter) -> ContextTarget {
    match adapter {
        AgentAdapter::Codex => ContextTarget::Codex,
        AgentAdapter::Claude => ContextTarget::Claude,
        AgentAdapter::Generic => ContextTarget::Generic,
    }
}

fn adapter_name(adapter: AgentAdapter) -> &'static str {
    match adapter {
        AgentAdapter::Codex => "codex",
        AgentAdapter::Claude => "claude",
        AgentAdapter::Generic => "agent",
    }
}

fn candidate_adapter_flags(adapters: &[String]) -> Result<Vec<OsString>> {
    let mut flags = Vec::new();
    for adapter in adapters {
        let flag = match adapter.as_str() {
            "codex" => "--codex",
            "claude" => "--claude",
            "agent" => "--agent",
            _ => bail!("Baron update transaction contains an unsupported adapter: {adapter}"),
        };
        flags.push(OsString::from(flag));
    }
    Ok(flags)
}

fn candidate_plan_arguments(
    repo_root: &std::path::Path,
    state_path: &std::path::Path,
    adapters: &[String],
) -> Result<Vec<OsString>> {
    let mut arguments = vec![
        OsString::from("update"),
        repo_root.as_os_str().to_os_string(),
        OsString::from("--candidate-plan"),
        OsString::from("--transaction"),
        state_path.as_os_str().to_os_string(),
    ];
    arguments.extend(candidate_adapter_flags(adapters)?);
    Ok(arguments)
}

fn candidate_continue_arguments(
    repo_root: &std::path::Path,
    state_path: &std::path::Path,
    runtime_binary: &std::path::Path,
) -> Vec<OsString> {
    vec![
        OsString::from("update"),
        repo_root.as_os_str().to_os_string(),
        OsString::from("--continue"),
        OsString::from("--transaction"),
        state_path.as_os_str().to_os_string(),
        OsString::from("--runtime-binary"),
        runtime_binary.as_os_str().to_os_string(),
        OsString::from("--runtime-parent-pid"),
        OsString::from(std::process::id().to_string()),
    ]
}

fn resolve_repo_root(path: PathBuf) -> Result<PathBuf> {
    find_project_root(&path).or_else(|_| {
        path.canonicalize()
            .with_context(|| format!("Could not resolve repo path: {}", path.display()))
    })
}

fn resolve_command_vault(vault: Option<PathBuf>, repo_root: &PathBuf) -> Result<PathBuf> {
    resolve_vault_path_for_repo(vault.clone(), repo_root).or_else(|_| resolve_vault_path(vault))
}

fn configured_repo(repo_path: Option<PathBuf>) -> Result<PathBuf> {
    let start = repo_path.unwrap_or(std::env::current_dir()?);
    find_project_root(start)
}

fn execution_context(
    repo_path: Option<PathBuf>,
) -> Result<(PathBuf, baron_core::vault::VaultContext)> {
    let repo_root = configured_repo(repo_path)?;
    let vault_path = resolve_vault_path_for_repo(None, &repo_root)?;
    let vault = require_coherent_execution_state(&repo_root, vault_path)?;
    Ok((repo_root, vault))
}

#[derive(Debug, serde::Serialize)]
struct CodeMapStatusReport {
    provider: String,
    supported_version: String,
    present: bool,
    detected_version: Option<String>,
    freshness: String,
    action: String,
    diagnostics: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct CodeMapRefreshReport {
    provider: String,
    version: String,
    source_fingerprint: String,
    graph_size_bytes: u64,
    action: String,
}

#[derive(Debug, serde::Serialize)]
struct CodeMapQueryHit {
    hit: baron_core::code_graph::CodeGraphHit,
    verification: SourceVerification,
}

#[derive(Debug, serde::Serialize)]
struct CodeMapQueryReport {
    provider: String,
    action: String,
    hits: Vec<CodeMapQueryHit>,
}

fn code_map_status(repo_root: &std::path::Path) -> Result<CodeMapStatusReport> {
    let provider = GraphifyProvider::new("graphify");
    let probe = provider.probe(repo_root)?;
    let mut diagnostics = probe.diagnostics;
    let freshness = match load_code_graph_state(repo_root) {
        Ok(Some(state)) => match graph_state_freshness(repo_root, &state) {
            Ok(freshness) => freshness,
            Err(_) => {
                diagnostics.push("local code-map state could not be validated".to_string());
                GraphFreshness::Invalid
            }
        },
        Ok(None) => GraphFreshness::Missing,
        Err(_) => {
            diagnostics.push("local code-map state is invalid".to_string());
            GraphFreshness::Invalid
        }
    };
    let compatible = probe.present && probe.version.as_deref() == Some(SUPPORTED_GRAPHIFY_VERSION);
    let action = if !compatible {
        "survey_fallback"
    } else if freshness == GraphFreshness::Fresh {
        "query"
    } else {
        "refresh"
    };
    Ok(CodeMapStatusReport {
        provider: "graphify-local".to_string(),
        supported_version: SUPPORTED_GRAPHIFY_VERSION.to_string(),
        present: probe.present,
        detected_version: probe.version,
        freshness: graph_freshness_name(freshness).to_string(),
        action: action.to_string(),
        diagnostics,
    })
}

fn render_code_map_status(report: &CodeMapStatusReport) -> String {
    format!(
        "# Baron Code Map Status\n\n- Provider: `{}`\n- Supported version: `{}`\n- Present: `{}`\n- Detected version: `{}`\n- Cache freshness: `{}`\n- AI action: `{}`\n- Diagnostics: {}\n",
        report.provider,
        report.supported_version,
        if report.present { "yes" } else { "no" },
        report.detected_version.as_deref().unwrap_or("unknown"),
        report.freshness,
        report.action,
        list_or_none(&report.diagnostics),
    )
}

fn render_code_map_query(report: &CodeMapQueryReport) -> String {
    let mut output = format!(
        "# Baron Code Map Query\n\n- Provider: `{}`\n- Next action: `{}`\n",
        report.provider, report.action
    );
    if report.hits.is_empty() {
        output.push_str("- No bounded graph hits returned. Survey fallback remains active.\n");
        return output;
    }
    for item in &report.hits {
        output.push_str(&format!(
            "\n## {}\n\n- Source: `{}`\n- Confidence: `{}`\n- Verification: `{}`\n- Evidence: {}\n- Rule: {}\n",
            item.hit.label,
            item.hit.source_file.as_deref().unwrap_or("unknown"),
            graph_confidence_name(item.hit.confidence),
            source_verification_name(item.verification.status),
            item.verification.evidence.as_deref().unwrap_or("none"),
            item.verification.message,
        ));
    }
    output
}

fn graph_freshness_name(freshness: GraphFreshness) -> &'static str {
    match freshness {
        GraphFreshness::Fresh => "fresh",
        GraphFreshness::Stale => "stale",
        GraphFreshness::Missing => "missing",
        GraphFreshness::Invalid => "invalid",
    }
}

fn graph_confidence_name(confidence: baron_core::code_graph::GraphConfidence) -> &'static str {
    match confidence {
        baron_core::code_graph::GraphConfidence::Extracted => "extracted",
        baron_core::code_graph::GraphConfidence::Inferred => "inferred",
    }
}

fn source_verification_name(
    status: baron_core::code_graph::SourceVerificationStatus,
) -> &'static str {
    match status {
        baron_core::code_graph::SourceVerificationStatus::Verified => "verified",
        baron_core::code_graph::SourceVerificationStatus::Advisory => "advisory",
        baron_core::code_graph::SourceVerificationStatus::MissingSource => "missing_source",
        baron_core::code_graph::SourceVerificationStatus::MissingEvidence => "missing_evidence",
    }
}

fn coherent_or_bootstrap_context(
    repo_root: &std::path::Path,
    vault_path: &std::path::Path,
) -> Result<baron_core::vault::VaultContext> {
    if repo_root.join(".baron/project.toml").is_file() {
        require_coherent_execution_state(repo_root, vault_path)
    } else {
        ensure_vault(vault_path, repo_root)
    }
}

fn hook_adapter_for_repo(repo_root: &std::path::Path) -> HookAdapter {
    match load_project_config(repo_root)
        .ok()
        .and_then(|config| config.adapters.first().copied())
    {
        Some(AdapterKind::Codex) => HookAdapter::Codex,
        Some(AdapterKind::Claude) => HookAdapter::Claude,
        Some(AdapterKind::Generic) | None => HookAdapter::Agent,
    }
}

impl From<OutcomeArg> for TraceOutcome {
    fn from(value: OutcomeArg) -> Self {
        match value {
            OutcomeArg::Completed => TraceOutcome::Completed,
            OutcomeArg::Partial => TraceOutcome::Partial,
            OutcomeArg::Blocked => TraceOutcome::Blocked,
            OutcomeArg::Failed => TraceOutcome::Failed,
        }
    }
}

impl From<ProviderKindArg> for ProviderKind {
    fn from(value: ProviderKindArg) -> Self {
        match value {
            ProviderKindArg::Cli => ProviderKind::Cli,
            ProviderKindArg::Binary => ProviderKind::Binary,
            ProviderKindArg::Mcp => ProviderKind::Mcp,
            ProviderKindArg::Skill => ProviderKind::Skill,
            ProviderKindArg::Http => ProviderKind::Http,
            ProviderKindArg::AgentAdapter => ProviderKind::AgentAdapter,
        }
    }
}

impl From<RiskLaneArg> for baron_core::risk::RiskLane {
    fn from(value: RiskLaneArg) -> Self {
        match value {
            RiskLaneArg::Low => Self::Low,
            RiskLaneArg::Medium => Self::Medium,
            RiskLaneArg::High => Self::High,
        }
    }
}

impl From<AdapterArg> for AdapterKind {
    fn from(value: AdapterArg) -> Self {
        match value {
            AdapterArg::Codex => AdapterKind::Codex,
            AdapterArg::Claude => AdapterKind::Claude,
            AdapterArg::Agent => AdapterKind::Generic,
        }
    }
}

impl From<AdapterArg> for HookAdapter {
    fn from(value: AdapterArg) -> Self {
        match value {
            AdapterArg::Codex => HookAdapter::Codex,
            AdapterArg::Claude => HookAdapter::Claude,
            AdapterArg::Agent => HookAdapter::Agent,
        }
    }
}

impl From<CertificationProfileArg> for CertificationProfile {
    fn from(value: CertificationProfileArg) -> Self {
        match value {
            CertificationProfileArg::Smoke => Self::Smoke,
            CertificationProfileArg::Release => Self::Release,
            CertificationProfileArg::Extreme => Self::Extreme,
        }
    }
}

impl From<AutomationEventArg> for AutomationEvent {
    fn from(value: AutomationEventArg) -> Self {
        match value {
            AutomationEventArg::SessionStart => AutomationEvent::SessionStart,
            AutomationEventArg::Prompt => AutomationEvent::Prompt,
            AutomationEventArg::Checkpoint => AutomationEvent::Checkpoint,
            AutomationEventArg::ContextCompiled => AutomationEvent::ContextCompiled,
            AutomationEventArg::PlanStarted => AutomationEvent::PlanStarted,
            AutomationEventArg::HarnessStarted => AutomationEvent::HarnessStarted,
            AutomationEventArg::ProofRecorded => AutomationEvent::ProofRecorded,
            AutomationEventArg::TraceScored => AutomationEvent::TraceScored,
            AutomationEventArg::Stop => AutomationEvent::Stop,
        }
    }
}

fn print_memory_status(repo_path: PathBuf, vault_path: PathBuf) -> Result<()> {
    let context = vault_context_without_create(&vault_path, &repo_path)?;
    let vault_exists = context.vault_root.exists();
    let project_exists = context.project_root.exists();
    let index_exists = context.index_path.exists();
    let records = if index_exists {
        load_memory_records(&context)?
    } else {
        Vec::new()
    };

    println!("# Baron Memory Status\n");
    println!("- Vault: `{}`", context.vault_root.display());
    println!(
        "- Vault exists: {}",
        if vault_exists { "yes" } else { "no" }
    );
    println!("- Project slug: `{}`", context.project_slug);
    println!(
        "- Project capsule exists: {}",
        if project_exists { "yes" } else { "no" }
    );
    println!("- Index: `{}`", context.index_path.display());
    println!(
        "- Index exists: {}",
        if index_exists { "yes" } else { "no" }
    );
    println!("- Records: {}", records.len());
    let (imported_sessions, skipped_sessions, last_import) = if project_exists {
        import_state_summary(&context)?
    } else {
        (0, 0, None)
    };
    println!("- Imported sessions: {}", imported_sessions);
    println!("- Skipped session sources: {}", skipped_sessions);
    println!(
        "- Last session import: {}",
        last_import.unwrap_or_else(|| "never".to_string())
    );
    println!("- Firewall: current project first, approved global second, cross-project blocked unless explicit");
    println!("\nNo files were written.");
    Ok(())
}

fn render_memory_index(
    context: &baron_core::vault::VaultContext,
    report: &baron_core::memory::MemoryIndexReport,
) -> String {
    format!(
        "# Baron Memory Index\n\n- Vault: `{}`\n- Project slug: `{}`\n- Index: `{}`\n- Total sources: {}\n- Reused sources: {}\n- Refreshed sources: {}\n- Deleted sources: {}\n- Total records: {}\n- Current project records: {}\n- Cross-project records: {}\n- Approved global records: {}\n- Global candidate records: {}\n- Wrote Vault cache only; target repo files were not written.\n",
        context.vault_root.display(),
        context.project_slug,
        context.index_path.display(),
        report.total_sources,
        report.reused_sources,
        report.refreshed_sources,
        report.deleted_sources,
        report.total_records,
        report.current_project_records,
        report.cross_project_records,
        report.global_verified_records,
        report.global_candidate_records
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use anyhow::Result;

    use super::self_update::CandidateBinaryInspector;
    use super::verify_activated_runtime_version;

    struct StaticInspector(&'static str);

    impl CandidateBinaryInspector for StaticInspector {
        fn reported_version(&self, _candidate_path: &Path) -> Result<String> {
            Ok(self.0.to_string())
        }
    }

    #[test]
    fn activated_runtime_requires_the_exact_release_version() {
        let accepted = StaticInspector("baron 3.6.0");
        assert!(verify_activated_runtime_version(&accepted, Path::new("baron"), "3.6.0").is_ok());

        let rejected = StaticInspector("baron 3.5.0");
        let error = verify_activated_runtime_version(&rejected, Path::new("baron"), "3.6.0")
            .expect_err("a mismatched activated runtime must be rejected");
        assert!(error
            .to_string()
            .contains("Activated Baron runtime reported `baron 3.5.0`; expected `baron 3.6.0`"));
    }
}
