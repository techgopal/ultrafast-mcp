use crate::config::Config;
use crate::templates::Template;
use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::path::PathBuf;

/// Initialize a new MCP project
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Project name
    #[arg(value_name = "NAME")]
    pub name: Option<String>,

    /// Project directory (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// Project template to use
    #[arg(short, long, default_value = "basic")]
    pub template: String,

    /// Project description
    #[arg(short, long)]
    pub description: Option<String>,

    /// Author name
    #[arg(short, long)]
    pub author: Option<String>,

    /// License
    #[arg(short, long, default_value = "MIT")]
    pub license: String,

    /// Skip interactive prompts
    #[arg(short, long)]
    pub yes: bool,

    /// Force overwrite existing files
    #[arg(short, long)]
    pub force: bool,
}

pub async fn execute(args: InitArgs, config: Option<Config>) -> Result<()> {
    println!("{}", "Initializing new MCP project...".green().bold());

    // Determine project name
    let project_name = match args.name {
        Some(name) => name,
        None => {
            if args.yes {
                "my-mcp-project".to_string()
            } else {
                prompt_for_input("Project name", Some("my-mcp-project"))?
            }
        }
    };

    // Determine project directory
    let project_dir = match args.path {
        Some(path) => path,
        None => std::env::current_dir()?.join(&project_name),
    };

    // Check if directory exists and handle force flag
    if project_dir.exists() && !args.force {
        if !args.yes {
            let response = prompt_for_confirmation(&format!(
                "Directory '{}' already exists. Continue?",
                project_dir.display()
            ))?;
            if !response {
                println!("{}", "Initialization cancelled.".yellow());
                return Ok(());
            }
        } else {
            anyhow::bail!("Directory already exists. Use --force to overwrite.");
        }
    }

    // Create project directory
    std::fs::create_dir_all(&project_dir)
        .with_context(|| format!("Failed to create directory: {}", project_dir.display()))?;

    println!("📁 Created project directory: {}", project_dir.display());

    // Get additional project information
    let description = match args.description {
        Some(desc) => desc,
        None => {
            if args.yes {
                format!("An MCP project: {project_name}")
            } else {
                prompt_for_input(
                    "Description",
                    Some(&format!("An MCP project: {project_name}")),
                )?
            }
        }
    };

    let author = match args.author {
        Some(author) => author,
        None => {
            if args.yes {
                "Unknown".to_string()
            } else {
                prompt_for_input("Author", Some("Unknown"))?
            }
        }
    };

    // Load template
    let template = Template::load(&args.template, config.as_ref())
        .with_context(|| format!("Failed to load template: {}", args.template))?;

    // Create template context
    let mut context = std::collections::HashMap::new();
    context.insert("project_name".to_string(), project_name.clone());
    context.insert("description".to_string(), description);
    context.insert("author".to_string(), author);
    context.insert("license".to_string(), args.license);
    context.insert("version".to_string(), "0.1.0".to_string());

    // Generate project from template
    template
        .generate(&project_dir, &context)
        .context("Failed to generate project from template")?;

    println!("✅ Project initialized successfully!");
    println!();
    println!("Next steps:");
    println!("  cd {project_name}");
    println!("  mcp dev");
    println!();
    println!("For more information, run: mcp info");

    Ok(())
}

fn prompt_for_input(prompt: &str, default: Option<&str>) -> Result<String> {
    use std::io::{self, Write};

    let prompt_text = if let Some(default) = default {
        format!("{prompt} [{default}]: ")
    } else {
        format!("{prompt}: ")
    };

    print!("{prompt_text}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        if let Some(default) = default {
            Ok(default.to_string())
        } else {
            anyhow::bail!("Input is required")
        }
    } else {
        Ok(input.to_string())
    }
}

fn prompt_for_confirmation(prompt: &str) -> Result<bool> {
    use std::io::{self, Write};

    print!("{prompt} [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    Ok(matches!(input.as_str(), "y" | "yes"))
}
