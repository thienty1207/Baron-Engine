use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use baron_adapters::{install_adapter, shadow_preview, AgentAdapter};
use baron_core::config::{
    find_project_root, initialize_project, load_project_config, resolve_vault_path_for_repo,
    AdapterKind,
};
use baron_core::context::{compile_context_for_task, compile_context_why, ContextTarget};
use baron_core::firewall::{compact_memory_brief, recall, render_recall};
use baron_core::memory::{build_memory_index, load_memory_records};
use baron_core::survey::{render_project_atlas, survey_repository};
use baron_core::vault::{ensure_vault, resolve_vault_path, vault_context_without_create};
use baron_core::{phase, product_name};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "baron", about = "Baron Engine")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
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
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },
    Recall {
        query: String,
        repo_path: Option<PathBuf>,
        #[arg(long)]
        vault: Option<PathBuf>,
    },
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
        }) => {
            let adapter = parse_adapter(codex, claude, agent)?;
            let repo_path = repo_path.unwrap_or(std::env::current_dir()?);
            let _survey = survey_repository(&repo_path)?;
            if shadow {
                print!("{}", shadow_preview(adapter).to_markdown());
            } else {
                let repo_root = repo_path.canonicalize()?;
                let vault_path = resolve_vault_path_for_repo(vault, &repo_root)?;
                initialize_project(&repo_root, adapter_kind(adapter), &vault_path)?;
                let context = ensure_vault(&vault_path, &repo_root)?;
                build_memory_index(&context)?;
                let report = install_adapter(&repo_root, adapter)?;
                println!("# Baron Adapter Init\n");
                println!("- Project: `{}`", context.project_slug);
                println!("- Adapter initialized: `{}`", report.adapter);
                println!("- Vault: `{}`", context.vault_root.display());
                println!("- Managed files: {}", report.managed_files.len());
                println!("- Custom assets preserved: yes");
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
                print!(
                    "{}",
                    compile_context_for_task(repo_path, vault_path, target, task.as_deref(),)?
                );
            }
        }
        None => {
            println!("{} {}", product_name(), phase());
            println!("Run `baron --help` for commands.");
        }
    }
    Ok(())
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

fn parse_adapter(codex: bool, claude: bool, agent: bool) -> Result<AgentAdapter> {
    match (codex as u8) + (claude as u8) + (agent as u8) {
        1 if codex => Ok(AgentAdapter::Codex),
        1 if claude => Ok(AgentAdapter::Claude),
        1 if agent => Ok(AgentAdapter::Generic),
        0 => bail!("Choose one adapter: --codex, --claude, or --agent."),
        _ => bail!("Choose only one adapter: --codex, --claude, or --agent."),
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
    println!("- Firewall: current project first, approved global second, cross-project blocked unless explicit");
    println!("\nNo files were written.");
    Ok(())
}

fn render_memory_index(
    context: &baron_core::vault::VaultContext,
    report: &baron_core::memory::MemoryIndexReport,
) -> String {
    format!(
        "# Baron Memory Index\n\n- Vault: `{}`\n- Project slug: `{}`\n- Index: `{}`\n- Total records: {}\n- Current project records: {}\n- Cross-project records: {}\n- Approved global records: {}\n- Global candidate records: {}\n- Wrote Vault cache only; target repo files were not written.\n",
        context.vault_root.display(),
        context.project_slug,
        context.index_path.display(),
        report.total_records,
        report.current_project_records,
        report.cross_project_records,
        report.global_verified_records,
        report.global_candidate_records
    )
}
