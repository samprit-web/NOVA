//! Build command

use anyhow::Result;
use std::process::Command;
use std::path::Path;

use crate::config::Config;

pub fn execute(args: super::BuildArgs, _config: &Config) -> Result<()> {
    println!("{} Building project...", "⚙".blue());

    // Find project root
    let project_root = find_project_root()?;
    
    // Parse nova.toml
    let manifest_path = project_root.join("nova.toml");
    if !manifest_path.exists() {
        anyhow::bail!("nova.toml not found. Are you in a NOVA project?");
    }

    let manifest = parse_manifest(&manifest_path)?;

    // Determine target
    let target = args.target.as_deref().unwrap_or("native");

    // Build
    let output = build_project(&project_root, &manifest, &args)?;

    println!("{} Build successful!", "✓".green());
    if let Some(path) = output {
        println!("  Output: {}", path.display());
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
    anyhow::bail!("Could not find nova.toml in current directory or parents");
}

fn parse_manifest(path: &Path) -> Result<Manifest> {
    let content = std::fs::read_to_string(path)?;
    let manifest: Manifest = toml::from_str(&content)?;
    Ok(manifest)
}

#[derive(serde::Deserialize)]
struct Manifest {
    package: Package,
    dependencies: Option<std::collections::HashMap<String, Dependency>>,
}

#[derive(serde::Deserialize)]
struct Package {
    name: String,
    version: String,
    edition: String,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum Dependency {
    Simple(String),
    Detailed(DetailedDependency),
}

#[derive(serde::Deserialize)]
struct DetailedDependency {
    version: String,
    features: Option<Vec<String>>,
    optional: Option<bool>,
    git: Option<String>,
    branch: Option<String>,
    tag: Option<String>,
    rev: Option<String>,
    path: Option<String>,
}

fn build_project(project_root: &Path, manifest: &Manifest, args: &super::BuildArgs) -> Result<Option<std::path::PathBuf>> {
    // For now, just invoke the compiler
    // In a real implementation, this would:
    // 1. Resolve dependencies
    // 2. Download/compile dependencies
    // 3. Compile the project
    // 4. Link everything
    
    let profile = &args.profile;
    let target_dir = project_root.join("target").join(profile);
    std::fs::create_dir_all(&target_dir)?;

    // Simulate compilation
    println!("  Compiling {} v{}", manifest.package.name, manifest.package.version);
    
    // In reality, we'd call the compiler here
    // let mut compiler = nova_compiler::Compiler::new(...);
    // compiler.compile_file(project_root.join("src/main.nova"))?;
    
    // For demo, create a dummy output
    let output_path = target_dir.join(&manifest.package.name);
    #[cfg(windows)]
    let output_path = output_path.with_extension("exe");
    
    std::fs::write(&output_path, b"NOVA binary placeholder")?;

    Ok(Some(output_path))
}