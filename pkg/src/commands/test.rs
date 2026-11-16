//! Test command

use anyhow::Result;
use std::process::Command;
use std::path::Path;

use crate::config::Config;

pub fn execute(args: super::TestArgs, config: &Config) -> Result<()> {
    println!("{} Running tests...", "🧪".blue());

    let project_root = find_project_root()?;
    
    // Build test profile
    let build_args = super::BuildArgs {
        profile: "test".to_string(),
        target: None,
        all_targets: true,
        jobs: None,
        verbose: false,
    };
    super::build::execute(build_args, config)?;

    // Run tests
    let test_bin = project_root.join("target").join("test").join("tests");
    
    #[cfg(windows)]
    let test_bin = test_bin.with_extension("exe");

    if !test_bin.exists() {
        println!("  No tests found");
        return Ok(());
    }

    let mut cmd = Command::new(&test_bin);
    
    if let Some(filter) = args.filter {
        cmd.arg("--filter").arg(filter);
    }
    if args.ignored {
        cmd.arg("--ignored");
    }
    if args.ignored_only {
        cmd.arg("--ignored-only");
    }
    if let Some(threads) = args.threads {
        cmd.arg("--threads").arg(threads.to_string());
    }

    let status = cmd.status()?;
    
    if !status.success() {
        anyhow::bail!("Tests failed");
    }

    println!("{} All tests passed!", "✓".green());
    Ok(())
}

fn find_project_root() -> Result<std::path::PathBuf> {
    let mut current = std::env::current_dir()?;
    loop {
        if current.join("nova.toml").exists() {
            return Ok(current);
        }
        if !current.pop() {
            break;
        }
    }
    anyhow::bail!("Could not find nova.toml");
}