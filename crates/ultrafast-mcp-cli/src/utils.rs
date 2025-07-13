use anyhow::Result;
use colored::*;
use std::path::Path;

/// Utility functions for the CLI
/// Check if a directory looks like an MCP project
#[allow(dead_code)]
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
#[allow(dead_code)]
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

/// Validate project name
#[allow(dead_code)]
pub fn validate_project_name(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("Project name cannot be empty");
    }

    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        anyhow::bail!(
            "Project name can only contain alphanumeric characters, hyphens, and underscores"
        );
    }

    if name.starts_with('-') || name.ends_with('-') {
        anyhow::bail!("Project name cannot start or end with a hyphen");
    }

    Ok(())
}

/// Print success message
#[allow(dead_code)]
pub fn print_success(message: &str) {
    println!("{} {}", "✅".green(), message);
}

/// Print error message
#[allow(dead_code)]
pub fn print_error(message: &str) {
    eprintln!("{} {}", "❌".red(), message);
}

/// Print warning message
#[allow(dead_code)]
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠️".yellow(), message);
}

/// Print info message
#[allow(dead_code)]
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ️".blue(), message);
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
    fn test_is_mcp_project() {
        // This test would need a real MCP project directory
        // For now, just test that it doesn't panic
        let temp_dir = std::env::temp_dir();
        let _result = is_mcp_project(&temp_dir);
    }
}
