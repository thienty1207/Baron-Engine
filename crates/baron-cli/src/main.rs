use std::path::PathBuf;

use anyhow::{bail, Result};
use baron_adapters::{shadow_preview, AgentAdapter};
use baron_core::survey::{render_project_atlas, survey_repository};
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
        }) => {
            if !shadow {
                bail!("Phase 1 supports init only with --shadow; no files will be written.");
            }
            let adapter = parse_adapter(codex, claude, agent)?;
            let repo_path = repo_path.unwrap_or(std::env::current_dir()?);
            let _survey = survey_repository(repo_path)?;
            print!("{}", shadow_preview(adapter).to_markdown());
        }
        None => {
            println!("{} {}", product_name(), phase());
            println!("Run `baron --help` for commands.");
        }
    }
    Ok(())
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
