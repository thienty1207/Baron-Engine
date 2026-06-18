use std::io::Read;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use baron_adapters::{install_adapter, shadow_preview, AgentAdapter};
use baron_core::asset_lifecycle::{
    audit_runtime_assets, quarantine_failing_assets, stage_skill_update,
};
use baron_core::automation::{
    automation_status, handle_hook, reconcile, record_lifecycle_event, AutomationEvent, HookAdapter,
};
use baron_core::capability::{
    check_capabilities, load_capability_state, load_registry, register_provider, remove_provider,
    CapabilityExecutionEvidence, CapabilityProvider, CheckOptions, Presence, ProviderKind,
    Requirement,
};
use baron_core::certification::{
    latest_certification_status, render_certification_report, run_certification,
    CertificationProfile,
};
use baron_core::config::{
    find_project_root, initialize_project, initialize_project_with_options, load_project_config,
    resolve_vault_path_for_repo, set_project_platform, setup_machine_vault, AdapterKind,
    ProjectPlatform,
};
use baron_core::context::{compile_context_for_task, compile_context_why, ContextTarget};
use baron_core::continuity::{continuity_status, record_continuity_checkpoint};
use baron_core::control_plane::{
    gate_evidence_status, record_gate_evidence, route_task, validate_control_plane,
};
use baron_core::firewall::{compact_memory_brief, recall, render_recall};
use baron_core::harness::{
    harness_status, record_decision, record_friction, start_or_resume_intake,
};
use baron_core::harness_improvement::{
    audit_harness, propose_improvements, record_improvement_outcome, record_intervention,
    verify_open_stories,
};
use baron_core::memory::{build_memory_index, load_memory_records};
use baron_core::migration::{
    execute_agent_bootstrap_migration, inventory_agent_bootstrap, migration_status,
    render_migration_inventory, rollback_migration,
};
use baron_core::plan::{
    complete_plan, interrupt_plan, plan_status, start_or_resume_plan, update_plan,
};
use baron_core::proof::{proof_status, record_proof, record_proof_with_capabilities};
use baron_core::release::{load_and_verify_release_metadata, write_release_metadata};
use baron_core::session::{import_sessions, import_state_summary};
use baron_core::session_replay::{
    index_session_replay, replay_session_context, search_session_replay,
};
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
        #[arg(long)]
        codex: bool,
        #[arg(long)]
        claude: bool,
        #[arg(long = "agent")]
        agent: bool,
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
                println!("- Custom assets preserved: yes");
            } else if let Some(platform) = platform {
                let config = set_project_platform(&repo_path, platform)?;
                println!("# Baron Platform Focus\n");
                println!("- Project: `{}`", config.project_slug);
                println!("- Platform focus: `{}`", platform_name(platform));
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
            for adapter in &adapters {
                install_adapter(&repo_root, *adapter)?;
            }
            let names = adapters
                .iter()
                .map(|adapter| adapter_name(*adapter))
                .collect::<Vec<_>>()
                .join(", ");
            println!("# Baron Adapter Update\n");
            println!("- Project: `{}`", config.project_slug);
            println!("- Updated adapters: `{}`", names);
            println!("- User content and custom assets preserved: yes");
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
                let context = ensure_vault(vault_path, repo_path)?;
                build_memory_index(&context)?;
                print!("{}", compact_memory_brief(&context)?);
            }
            MemoryCommands::ImportSessions { repo_path, vault } => {
                let repo_path = resolve_repo_root(repo_path.unwrap_or(std::env::current_dir()?))?;
                let vault_path = resolve_command_vault(vault, &repo_path)?;
                let context = ensure_vault(vault_path, &repo_path)?;
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
            let context = ensure_vault(vault_path, repo_path)?;
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
            if why {
                print!("{}", compile_context_why(repo_path, vault_path, target)?);
            } else {
                let output =
                    compile_context_for_task(&repo_path, &vault_path, target, task.as_deref())?;
                let vault_context = ensure_vault(&vault_path, &repo_path)?;
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
                let context = ensure_vault(vault_path, repo_root)?;
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
                let context = ensure_vault(vault_path, repo_root)?;
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
                let context = ensure_vault(vault_path, repo_root)?;
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
                let report = reconcile(&repo_root)?;
                println!("# Baron Automation Reconciliation\n");
                println!("- Passed: `{}`", if report.passed { "yes" } else { "no" });
                println!("- Active plan: `{}`", report.active_plan);
                println!("- Gaps: {}", list_or_none(&report.gaps));
            }
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
            ReleaseCommands::Verify { artifacts_dir } => {
                let manifest = load_and_verify_release_metadata(&artifacts_dir)?;
                println!("# Baron Release Verification\n");
                println!("- Release assets verified");
                println!("- Version: `{}`", manifest.version);
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
    let vault = ensure_vault(vault_path, &repo_root)?;
    Ok((repo_root, vault))
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
