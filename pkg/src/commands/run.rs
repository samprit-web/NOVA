//! Run command

use anyhow::Result;
use std::process::Command;
use std::path::Path;

use crate::config::Config;

pub fn execute(args: super::RunArgs, config: &Config) -> Result<()> {
    // First build
    let build_args = super::BuildArgs {
        profile: args.profile.clone(),
        target: None,
        all_targets: false,
        jobs: None,
        verbose: false,
    };
    super::build::execute(build_args, config)?;

    // Find binary
    let project_root = find_project_root()?;
    let manifest = parse_manifest(&project_root.join("nova.toml"))?;
    
    let profile = &args.profile;
    let target_dir = project_root.join("target").join(profile);
    
    let binary_name = args.bin.unwrap_or_else(|| manifest.package.name.clone());
    let binary_path = target_dir.join(&binary_name);
    
    #[cfg(windows)]
    let binary_path = binary_path.with_extension("exe");

    if !binary_path.exists() {
        anyhow::bail!("Binary not found: {}. Run 'nova build' first.", binary_path.display());
    }

    // Run binary
    println!("{} Running {}...", "▶".green(), binary_name.bold());
    
    let mut cmd = Command::new(&binary_path);
    cmd.args(&args.args);
    cmd.current_dir(&project_root);
    
    let status = cmd.status()?;
    
    if !status.success() {
        anyhow::bail!("Process exited with code: {:?}", status.code());
    }

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

fn parse_manifest(path: &Path) -> Result<Manifest> {
    let content = std::fs::read_to_string(path)?;
    let manifest: Manifest = toml::from_str(&content)?;
    Ok(manifest)
}

#[derive(serde::Deserialize)]
struct Manifest {
    package: Package,
}

#[derive(serde::Deserialize)]
struct Package {
    name: String,
    version: String,
    edition: String,
}