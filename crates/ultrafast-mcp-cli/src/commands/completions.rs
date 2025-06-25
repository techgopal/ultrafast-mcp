use clap::{Args, ValueEnum};
use anyhow::Result;
use crate::config::Config;

/// Generate shell completions
#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,

    /// Output file (defaults to stdout)
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

pub async fn execute(args: CompletionsArgs, _config: Option<Config>) -> Result<()> {
    use clap::CommandFactory;
    use clap_complete::generate;
    use std::io;

    let mut cmd = crate::Cli::command();
    let name = cmd.get_name().to_string();

    if let Some(output_path) = args.output {
        println!("📝 Generating completions for {:?} to: {}", args.shell, output_path.display());
        let mut file = std::fs::File::create(&output_path)?;

        match args.shell {
            Shell::Bash => generate(clap_complete::shells::Bash, &mut cmd, name, &mut file),
            Shell::Zsh => generate(clap_complete::shells::Zsh, &mut cmd, name, &mut file),
            Shell::Fish => generate(clap_complete::shells::Fish, &mut cmd, name, &mut file),
            Shell::PowerShell => generate(clap_complete::shells::PowerShell, &mut cmd, name, &mut file),
            Shell::Elvish => generate(clap_complete::shells::Elvish, &mut cmd, name, &mut file),
        }
        println!("✅ Completions generated successfully");
    } else {
        println!("# Completions for {:?}", args.shell);
        match args.shell {
            Shell::Bash => generate(clap_complete::shells::Bash, &mut cmd, name, &mut io::stdout()),
            Shell::Zsh => generate(clap_complete::shells::Zsh, &mut cmd, name, &mut io::stdout()),
            Shell::Fish => generate(clap_complete::shells::Fish, &mut cmd, name, &mut io::stdout()),
            Shell::PowerShell => generate(clap_complete::shells::PowerShell, &mut cmd, name, &mut io::stdout()),
            Shell::Elvish => generate(clap_complete::shells::Elvish, &mut cmd, name, &mut io::stdout()),
        }
    }

    Ok(())
}
