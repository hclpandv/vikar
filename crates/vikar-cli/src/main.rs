//! The `vikar` binary: loads a playbook, validates it, compiles it into
//! runnable [`vikar_core::Task`]s, and either prints the plan (`vikar plan`)
//! or executes it (`vikar play`).
//!
//! This crate is where the compile-time task-kind and provider registries
//! live (see [`compile`] and [`providers`]) — deliberately compile-time
//! `match` statements rather than dynamic plugin loading, matching the
//! "single static binary, zero infra" design constraint. Dynamic loading is
//! a real feature for later; it has nothing to do with proving Vikar's
//! core hook and would be a rabbit hole to build now.

mod compile;
mod env_file;
mod init;
mod providers;
mod tasks;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use vikar_config::Playbook;

#[derive(Parser)]
#[command(
    name = "vikar",
    version,
    about = "Dispatch a workforce of substitute agents, declared in YAML."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scaffold a `.agents/` workspace in the current directory: an
    /// `.env` for API keys (gitignored), a `.gitignore`, and two demo
    /// playbooks — one that needs no API key at all, one that calls a
    /// real hosted model — so there's something runnable immediately,
    /// for you to build your own flows on top of.
    Init {
        /// Overwrite any files that already exist under `.agents/`.
        #[arg(long)]
        force: bool,
    },

    /// Parse and validate a playbook, printing the resolved plan without
    /// running anything. Useful for sanity-checking a playbook (and for
    /// CI: exits non-zero on a bad playbook, no API keys required).
    Plan {
        /// Path to the playbook YAML file.
        playbook: PathBuf,
    },

    /// Parse, validate, and execute a playbook.
    Play {
        /// Path to the playbook YAML file.
        playbook: PathBuf,

        /// Print machine-readable JSON instead of a human-readable trace.
        /// Reserved for pipeline use — see the project README's "zero
        /// infra" notes on exit codes and structured output.
        #[arg(long)]
        output_json: bool,
    },
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .without_time()
        .init();

    let cli = Cli::parse();

    // Load `.agents/.env` (or plain `.env`), if one exists, before any
    // subcommand runs — providers read API keys via `std::env::var`, and
    // this is what makes keys placed by `vikar init` (or by hand) actually
    // visible without the user having to `export` them manually.
    env_file::load();

    let result = match cli.command {
        Command::Init { force } => init::run(force),
        Command::Plan { playbook } => run_plan(&playbook),
        Command::Play {
            playbook,
            output_json,
        } => {
            let rt = tokio::runtime::Runtime::new().expect("failed to start async runtime");
            rt.block_on(run_play(&playbook, output_json))
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

/// `vikar plan <file>` — load, validate, print the plan. No execution,
/// no network access, no API keys needed.
fn run_plan(path: &PathBuf) -> anyhow::Result<()> {
    let playbook = Playbook::load(path)?;
    playbook.validate()?;

    println!("Playbook: {}", path.display());
    println!(
        "Models declared: {}",
        if playbook.models.is_empty() {
            "(none)".to_string()
        } else {
            playbook
                .models
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        }
    );
    println!("Tasks ({}):", playbook.tasks.len());
    for (i, task) in playbook.tasks.iter().enumerate() {
        print_task_plan(&task_kind_line(task), 1, i + 1);
        if let vikar_config::TaskConfig::Until { tasks, .. } = task {
            for (j, inner) in tasks.iter().enumerate() {
                print_task_plan(&task_kind_line(inner), 2, j + 1);
            }
        }
    }
    println!(
        "\nPlaybook is valid. Run with `vikar play {}`.",
        path.display()
    );
    Ok(())
}

fn print_task_plan(line: &str, depth: usize, index: usize) {
    let indent = "  ".repeat(depth);
    println!("{indent}{index}. {line}");
}

fn task_kind_line(task: &vikar_config::TaskConfig) -> String {
    use vikar_config::TaskConfig::*;
    match task {
        Ask { model, register, .. } => format!(
            "[ask]   name={} model={model} register={}",
            task.display_name(),
            register.as_deref().unwrap_or("-")
        ),
        Run { command, register, .. } => format!(
            "[run]   name={} command=`{command}` register={}",
            task.display_name(),
            register.as_deref().unwrap_or("-")
        ),
        Until { condition, max_iterations, tasks, .. } => format!(
            "[until] name={} condition=`{condition}` max_iterations={max_iterations} ({} nested task(s))",
            task.display_name(),
            tasks.len()
        ),
    }
}

/// `vikar play <file>` — load, validate, compile, execute.
async fn run_play(path: &PathBuf, output_json: bool) -> anyhow::Result<()> {
    let playbook = Playbook::load(path)?;
    playbook.validate()?;

    let models = providers::build_models(&playbook.models)?;
    let runtime_tasks = compile::compile_tasks(&playbook.tasks, &models)?;
    let runner = vikar_core::Runner::new(runtime_tasks);

    let ctx = runner.run().await?;

    if output_json {
        let json = serde_json::json!({ "status": "ok", "context": ctx.as_json() });
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        println!("\nPlaybook completed successfully.");
    }

    Ok(())
}
