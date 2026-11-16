//! NOVA Package Manager CLI

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_complete::{generate, Shell};
use colored::*;

mod commands;
mod config;
mod manifest;
mod registry;
mod resolver;
mod lockfile;
mod fetch;
mod build;
mod publish;
mod audit;

use commands::*;
use config::Config;

#[derive(Parser)]
#[command(name = "nova")]
#[command(about = "NOVA Package Manager - Fast, Safe, Universal", long_about = None)]
#[command(version, author)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Working directory
    #[arg(short, long, global = true)]
    directory: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new NOVA project
    New(NewArgs),

    /// Initialize a new NOVA project in current directory
    Init(InitArgs),

    /// Build the project
    Build(BuildArgs),

    /// Run the project
    Run(RunArgs),

    /// Test the project
    Test(TestArgs),

    /// Benchmark the project
    Bench(BenchArgs),

    /// Check the project (type-check only)
    Check(CheckArgs),

    /// Format source code
    Fmt(FmtArgs),

    /// Lint source code
    Clippy(ClippyArgs),

    /// Generate documentation
    Doc(DocArgs),

    /// Clean build artifacts
    Clean(CleanArgs),

    /// Add a dependency
    Add(AddArgs),

    /// Remove a dependency
    Remove(RemoveArgs),

    /// Update dependencies
    Update(UpdateArgs),

    /// Show dependency tree
    Tree(TreeArgs),

    /// Publish package to registry
    Publish(PublishArgs),

    /// Search packages
    Search(SearchArgs),

    /// Show package metadata
    Metadata(MetadataArgs),

    /// Audit dependencies for vulnerabilities
    Audit(AuditArgs),

    /// Fix issues automatically
    Fix(FixArgs),

    /// Generate shell completions
    Completions(CompletionsArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup colors
    if cli.no_color {
        colored::control::set_override(false);
    }

    // Change directory if specified
    if let Some(dir) = cli.directory {
        std::env::set_current_dir(dir)?;
    }

    // Load config
    let config = Config::load()?;

    // Execute command
    let result = match cli.command {
        Commands::New(args) => cmd_new(args, &config),
        Commands::Init(args) => cmd_init(args, &config),
        Commands::Build(args) => cmd_build(args, &config),
        Commands::Run(args) => cmd_run(args, &config),
        Commands::Test(args) => cmd_test(args, &config),
        Commands::Bench(args) => cmd_bench(args, &config),
        Commands::Check(args) => cmd_check(args, &config),
        Commands::Fmt(args) => cmd_fmt(args, &config),
        Commands::Clippy(args) => cmd_clippy(args, &config),
        Commands::Doc(args) => cmd_doc(args, &config),
        Commands::Clean(args) => cmd_clean(args, &config),
        Commands::Add(args) => cmd_add(args, &config),
        Commands::Remove(args) => cmd_remove(args, &config),
        Commands::Update(args) => cmd_update(args, &config),
        Commands::Tree(args) => cmd_tree(args, &config),
        Commands::Publish(args) => cmd_publish(args, &config),
        Commands::Search(args) => cmd_search(args, &config),
        Commands::Metadata(args) => cmd_metadata(args, &config),
        Commands::Audit(args) => cmd_audit(args, &config),
        Commands::Fix(args) => cmd_fix(args, &config),
        Commands::Completions(args) => cmd_completions(args),
    };

    result
}

fn cmd_completions(args: CompletionsArgs) -> Result<()> {
    let shell = args.shell;
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "nova", &mut std::io::stdout());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}