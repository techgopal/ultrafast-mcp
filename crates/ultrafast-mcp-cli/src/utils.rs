use anyhow::Result;
use std::path::Path;
use colored::*;

/// Utility functions for the CLI

/// Check if a directory looks like an MCP project
pub fn is_mcp_project(dir: &Path) -> bool {
    let cargo_toml = dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return false;
    }
    
    // Check if Cargo.toml contains MCP dependencies
    if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
        content.contains("ultrafast-mcp")
    } else {
        false
    }
}

/// Find the project root directory
pub fn find_project_root() -> Result<std::path::PathBuf> {
    let mut current = std::env::current_dir()?;
    
    loop {
        if is_mcp_project(&current) {
            return Ok(current);
        }
        
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            anyhow::bail!("Not in an MCP project directory");
        }
    }
}

/// Pretty print JSON with syntax highlighting
pub fn pretty_print_json(value: &serde_json::Value) -> String {
    // Simple pretty printing - in a real implementation you'd add syntax highlighting
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "Invalid JSON".to_string())
}

/// Format file size in human readable format
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Create a progress bar
pub fn create_progress_bar(total: u64, message: &str) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new(total);
    pb.set_style(
        match indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}") {
            Ok(style) => style.progress_chars("#>-"),
            Err(_) => indicatif::ProgressStyle::default_bar(),
        }
    );
    pb.set_message(message.to_string());
    pb
}

/// Print success message
pub fn print_success(message: &str) {
    println!("{} {}", "✅".green(), message);
}

/// Print error message
pub fn print_error(message: &str) {
    eprintln!("{} {}", "❌".red(), message);
}

/// Print warning message
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠️".yellow(), message);
}

/// Print info message
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ️".blue(), message);
}

/// Validate project name
pub fn validate_project_name(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("Project name cannot be empty");
    }
    
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        anyhow::bail!("Project name can only contain alphanumeric characters, hyphens, and underscores");
    }
    
    if name.starts_with('-') || name.ends_with('-') {
        anyhow::bail!("Project name cannot start or end with a hyphen");
    }
    
    Ok(())
}

/// Get the user's name for git config
pub fn get_git_user_name() -> Option<String> {
    std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

/// Get the user's email for git config
pub fn get_git_user_email() -> Option<String> {
    std::process::Command::new("git")
        .args(["config", "user.email"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

/// Check if a command exists in PATH
pub fn command_exists(command: &str) -> bool {
    std::process::Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get the current directory name
pub fn current_dir_name() -> Option<String> {
    std::env::current_dir()
        .ok()
        .and_then(|path| path.file_name().map(|name| name.to_string_lossy().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_project_name() {
        assert!(validate_project_name("my-project").is_ok());
        assert!(validate_project_name("my_project").is_ok());
        assert!(validate_project_name("project123").is_ok());
        
        assert!(validate_project_name("").is_err());
        assert!(validate_project_name("-project").is_err());
        assert!(validate_project_name("project-").is_err());
        assert!(validate_project_name("my project").is_err());
        assert!(validate_project_name("my@project").is_err());
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(100), "100 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
    }
}
