//! Format command

use anyhow::Result;
use std::path::Path;
use std::process::Command;

use crate::config::Config;

pub fn execute(args: super::FmtArgs, _config: &Config) -> Result<()> {
    let project_root = find_project_root()?;

    if args.check {
        println!("{} Checking formatting...", "🔍".blue());
    } else {
        println!("{} Formatting code...", "✨".blue());
    }

    let files = if args.files.is_empty() {
        collect_nova_files(&project_root)?
    } else {
        args.files.clone()
    };

    let mut has_errors = false;

    for file in files {
        let result = format_file(&file, args.check)?;
        if args.check && !result {
            println!("  {} would be reformatted", file.display().to_string().yellow());
            has_errors = true;
        } else if !args.check {
            println!("  {} formatted", file.display().to_string().green());
        }
    }

    if args.check && has_errors {
        anyhow::bail!("Some files need formatting. Run 'nova fmt' to fix.");
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

fn collect_nova_files(root: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "nova" {
                    files.push(entry.path().to_path_buf());
                }
            }
        }
    }
    Ok(files)
}

fn format_file(path: &Path, check: bool) -> Result<bool> {
    let content = std::fs::read_to_string(path)?;
    
    // In a real implementation, this would use the NOVA formatter
    // For now, just check if file ends with newline
    let needs_format = !content.ends_with('\n');
    
    if !check && needs_format {
        let mut new_content = content;
        new_content.push('\n');
        std::fs::write(path, new_content)?;
    }
    
    Ok(needs_format)
}