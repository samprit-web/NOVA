//! CLI Commands

pub mod new;
pub mod init;
pub mod build;
pub mod run;
pub mod test;
pub mod bench;
pub mod check;
pub mod fmt;
pub mod clippy;
pub mod doc;
pub mod clean;
pub mod add;
pub mod remove;
pub mod update;
pub mod tree;
pub mod publish;
pub mod search;
pub mod metadata;
pub mod audit;
pub mod fix;
pub mod completions;

use anyhow::Result;
use clap::Args;

use crate::config::Config;

// New command
#[derive(Args)]
pub struct NewArgs {
    /// Project name
    pub name: String,

    /// Project path
    pub path: Option<std::path::PathBuf>,

    /// Project template
    #[arg(short, long, default_value = "bin")]
    pub template: String,

    /// Initialize git repository
    #[arg(long, default_value = "true")]
    pub git: bool,
}

pub fn cmd_new(args: NewArgs, config: &Config) -> Result<()> {
    new::execute(args, config)
}

// Init command
#[derive(Args)]
pub struct InitArgs {
    /// Project name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Project template
    #[arg(short, long, default_value = "bin")]
    pub template: String,

    /// Initialize git repository
    #[arg(long, default_value = "true")]
    pub git: bool,
}

pub fn cmd_init(args: InitArgs, config: &Config) -> Result<()> {
    init::execute(args, config)
}

// Build command
#[derive(Args)]
pub struct BuildArgs {
    /// Build profile
    #[arg(short, long, default_value = "dev")]
    pub profile: String,

    /// Target triple
    #[arg(long)]
    pub target: Option<String>,

    /// Build all targets
    #[arg(long)]
    pub all_targets: bool,

    /// Number of parallel jobs
    #[arg(short, long)]
    pub jobs: Option<usize>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

pub fn cmd_build(args: BuildArgs, config: &Config) -> Result<()> {
    build::execute(args, config)
}

// Run command
#[derive(Args)]
pub struct RunArgs {
    /// Build profile
    #[arg(short, long, default_value = "dev")]
    pub profile: String,

    /// Binary name
    #[arg(short, long)]
    pub bin: Option<String>,

    /// Example name
    #[arg(long)]
    pub example: Option<String>,

    /// Arguments to pass to the binary
    #[arg(last = true)]
    pub args: Vec<String>,
}

pub fn cmd_run(args: RunArgs, config: &Config) -> Result<()> {
    run::execute(args, config)
}

// Test command
#[derive(Args)]
pub struct TestArgs {
    /// Test profile
    #[arg(short, long, default_value = "test")]
    pub profile: String,

    /// Test filter
    #[arg(short, long)]
    pub filter: Option<String>,

    /// Run ignored tests
    #[arg(long)]
    pub ignored: bool,

    /// Run only ignored tests
    #[arg(long)]
    pub ignored_only: bool,

    /// Number of test threads
    #[arg(short, long)]
    pub threads: Option<usize>,
}

pub fn cmd_test(args: TestArgs, config: &Config) -> Result<()> {
    test::execute(args, config)
}

// Bench command
#[derive(Args)]
pub struct BenchArgs {
    /// Benchmark filter
    #[arg(short, long)]
    pub filter: Option<String>,
}

pub fn cmd_bench(args: BenchArgs, config: &Config) -> Result<()> {
    bench::execute(args, config)
}

// Check command
#[derive(Args)]
pub struct CheckArgs {
    /// Check all targets
    #[arg(long)]
    pub all_targets: bool,
}

pub fn cmd_check(args: CheckArgs, config: &Config) -> Result<()> {
    check::execute(args, config)
}

// Fmt command
#[derive(Args)]
pub struct FmtArgs {
    /// Check only, don't write changes
    #[arg(long)]
    pub check: bool,

    /// Files to format
    pub files: Vec<std::path::PathBuf>,
}

pub fn cmd_fmt(args: FmtArgs, config: &Config) -> Result<()> {
    fmt::execute(args, config)
}

// Clippy command
#[derive(Args)]
pub struct ClippyArgs {
    /// Fix warnings automatically
    #[arg(long)]
    pub fix: bool,

    /// Allow specific lints
    #[arg(long)]
    pub allow: Vec<String>,

    /// Warn specific lints
    #[arg(long)]
    pub warn: Vec<String>,

    /// Deny specific lints
    #[arg(long)]
    pub deny: Vec<String>,
}

pub fn cmd_clippy(args: ClippyArgs, config: &Config) -> Result<()> {
    clippy::execute(args, config)
}

// Doc command
#[derive(Args)]
pub struct DocArgs {
    /// Open docs in browser
    #[arg(long)]
    pub open: bool,

    /// Document private items
    #[arg(long)]
    pub private: bool,
}

pub fn cmd_doc(args: DocArgs, config: &Config) -> Result<()> {
    doc::execute(args, config)
}

// Clean command
#[derive(Args)]
pub struct CleanArgs {
    /// Package to clean
    #[arg(short, long)]
    pub package: Option<String>,
}

pub fn cmd_clean(args: CleanArgs, config: &Config) -> Result<()> {
    clean::execute(args, config)
}

// Add command
#[derive(Args)]
pub struct AddArgs {
    /// Package specification (name@version)
    pub packages: Vec<String>,

    /// Add as dev dependency
    #[arg(long)]
    pub dev: bool,

    /// Add as build dependency
    #[arg(long)]
    pub build: bool,

    /// Optional features
    #[arg(short, long)]
    pub features: Vec<String>,

    /// Git repository URL
    #[arg(long)]
    pub git: Option<String>,

    /// Git branch
    #[arg(long)]
    pub branch: Option<String>,

    /// Git tag
    #[arg(long)]
    pub tag: Option<String>,

    /// Git rev
    #[arg(long)]
    pub rev: Option<String>,

    /// Local path
    #[arg(long)]
    pub path: Option<std::path::PathBuf>,
}

pub fn cmd_add(args: AddArgs, config: &Config) -> Result<()> {
    add::execute(args, config)
}

// Remove command
#[derive(Args)]
pub struct RemoveArgs {
    /// Packages to remove
    pub packages: Vec<String>,
}

pub fn cmd_remove(args: RemoveArgs, config: &Config) -> Result<()> {
    remove::execute(args, config)
}

// Update command
#[derive(Args)]
pub struct UpdateArgs {
    /// Packages to update
    pub packages: Vec<String>,

    /// Update to latest compatible versions
    #[arg(long)]
    pub latest: bool,
}

pub fn cmd_update(args: UpdateArgs, config: &Config) -> Result<()> {
    update::execute(args, config)
}

// Tree command
#[derive(Args)]
pub struct TreeArgs {
    /// Show depth
    #[arg(short, long)]
    pub depth: Option<usize>,

    /// Show only dependencies
    #[arg(long)]
    pub deps: bool,
}

pub fn cmd_tree(args: TreeArgs, config: &Config) -> Result<()> {
    tree::execute(args, config)
}

// Publish command
#[derive(Args)]
pub struct PublishArgs {
    /// Package to publish
    #[arg(short, long)]
    pub package: Option<String>,

    /// Registry to publish to
    #[arg(long)]
    pub registry: Option<String>,

    /// Dry run
    #[arg(long)]
    pub dry_run: bool,

    /// Allow dirty working directory
    #[arg(long)]
    pub allow_dirty: bool,
}

pub fn cmd_publish(args: PublishArgs, config: &Config) -> Result<()> {
    publish::execute(args, config)
}

// Search command
#[derive(Args)]
pub struct SearchArgs {
    /// Search query
    pub query: String,

    /// Limit results
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
}

pub fn cmd_search(args: SearchArgs, config: &Config) -> Result<()> {
    search::execute(args, config)
}

// Metadata command
#[derive(Args)]
pub struct MetadataArgs {
    /// Output format
    #[arg(short, long, default_value = "json")]
    pub format: String,

    /// No deps
    #[arg(long)]
    pub no_deps: bool,
}

pub fn cmd_metadata(args: MetadataArgs, config: &Config) -> Result<()> {
    metadata::execute(args, config)
}

// Audit command
#[derive(Args)]
pub struct AuditArgs {
    /// Output format
    #[arg(short, long, default_value = "human")]
    pub format: String,

    /// Ignore specific vulnerabilities
    #[arg(long)]
    pub ignore: Vec<String>,
}

pub fn cmd_audit(args: AuditArgs, config: &Config) -> Result<()> {
    audit::execute(args, config)
}

// Fix command
#[derive(Args)]
pub struct FixArgs {
    /// Fix all
    #[arg(long)]
    pub all: bool,

    /// Specific fixes
    #[arg(long)]
    pub fix: Vec<String>,
}

pub fn cmd_fix(args: FixArgs, config: &Config) -> Result<()> {
    fix::execute(args, config)
}

// Completions command
#[derive(Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    pub shell: clap_complete::Shell,
}