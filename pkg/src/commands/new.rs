//! New project command

use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::config::Config;

pub fn execute(args: super::NewArgs, _config: &Config) -> Result<()> {
    let project_path = args.path.as_deref().unwrap_or_else(|| Path::new(&args.name));
    
    if project_path.exists() {
        anyhow::bail!("Directory '{}' already exists", project_path.display());
    }

    println!("{} Creating new project: {}", "✓".green(), args.name.bold());
    
    fs::create_dir_all(project_path)?;

    // Create project structure
    create_project_structure(project_path, &args)?;

    // Initialize git if requested
    if args.git {
        init_git(project_path)?;
    }

    println!("{} Project '{}' created successfully!", "✓".green(), args.name.bold());
    println!("\nNext steps:");
    println!("  cd {}", project_path.display());
    println!("  nova build");
    println!("  nova run");

    Ok(())
}

fn create_project_structure(project_path: &Path, args: &super::NewArgs) -> Result<()> {
    // Create Cargo.toml equivalent (nova.toml)
    let manifest = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2026"
description = "A NOVA project"
license = "MIT OR Apache-2.0"
repository = "https://github.com/user/{}"

[dependencies]
"#, args.name, args.name);

    fs::write(project_path.join("nova.toml"), manifest)?;

    // Create src directory
    let src_dir = project_path.join("src");
    fs::create_dir_all(&src_dir)?;

    // Create main.nova
    let main_content = match args.template.as_str() {
        "bin" => r#"fn main() {
    println!("Hello, NOVA! 🚀")
}
"#,
        "lib" => r#"/// A NOVA library
pub fn hello() -> String {
    "Hello from NOVA library!".to_string()
}
"#,
        "wasm" => r#"// WASM entry point
#[no_mangle]
pub extern "C" fn main() {
    println!("Hello from NOVA WASM!")
}
"#,
        "actor" => r#"use nova::task::actor;

actor Counter {
    state: i64 = 0

    fn increment(&mut self) -> i64 {
        self.state += 1
        self.state
    }

    fn get(&self) -> i64 {
        self.state
    }
}

fn main() {
    let counter = Counter::new();
    println!("Counter: {}", counter.increment());
}
"#,
        _ => r#"fn main() {
    println!("Hello, NOVA!")
}
"#,
    };

    fs::write(src_dir.join("main.nova"), main_content)?;

    // Create .gitignore
    let gitignore = r#"# NOVA build artifacts
target/
*.nvbc

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db
"#;
    fs::write(project_path.join(".gitignore"), gitignore)?;

    // Create README.md
    let readme = format!(r#"# {}

A NOVA project.

## Building

```bash
nova build
```

## Running

```bash
nova run
```

## Testing

```bash
nova test
```
"#, args.name);
    fs::write(project_path.join("README.md"), readme)?;

    // Create tests directory
    let tests_dir = project_path.join("tests");
    fs::create_dir_all(&tests_dir)?;

    let test_content = r#"#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(2 + 2, 4);
    }
}
"#;
    fs::write(tests_dir.join("main_test.nova"), test_content)?;

    // Create examples directory
    let examples_dir = project_path.join("examples");
    fs::create_dir_all(&examples_dir)?;

    let example_content = r#"fn main() {
    println!("Example!")
}
"#;
    fs::write(examples_dir.join("example.nova"), example_content)?;

    // Create benches directory
    let benches_dir = project_path.join("benches");
    fs::create_dir_all(&benches_dir)?;

    let bench_content = r#"use nova::bench;

#[bench]
fn bench_example(b: &mut Bencher) {
    b.iter(|| {
        2 + 2
    });
}
"#;
    fs::write(benches_dir.join("bench.nova"), bench_content)?;

    Ok(())
}

fn init_git(project_path: &Path) -> Result<()> {
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(project_path)
        .output()?;
    Ok(())
}