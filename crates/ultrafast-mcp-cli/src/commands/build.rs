use crate::config::Config;
use anyhow::Result;
use clap::Args;
use colored::*;

/// Build the project
#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Build profile
    #[arg(long, default_value = "dev")]
    pub profile: String,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Target directory
    #[arg(long)]
    pub target_dir: Option<std::path::PathBuf>,
}

pub async fn execute(args: BuildArgs, _config: Option<Config>) -> Result<()> {
    println!("{}", "Building MCP project...".green().bold());

    let mut cmd = tokio::process::Command::new("cargo");
    cmd.arg("build");

    if args.profile == "release" {
        cmd.arg("--release");
    }

    if args.verbose {
        cmd.arg("--verbose");
    }

    if let Some(target_dir) = args.target_dir {
        cmd.arg("--target-dir").arg(target_dir);
    }

    let output = cmd.output().await?;

    if output.status.success() {
        println!("✅ Build completed successfully");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Build failed:\n{}", stderr);
    }

    Ok(())
}
