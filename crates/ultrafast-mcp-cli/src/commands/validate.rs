use clap::Args;
use anyhow::{Result, Context};
use colored::*;
use crate::config::Config;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::fs;

/// Validate MCP schemas and configurations
#[derive(Debug, Args)]
pub struct ValidateArgs {
    /// Path to validate
    #[arg(value_name = "PATH")]
    pub path: Option<std::path::PathBuf>,

    /// Validate schema only
    #[arg(long)]
    pub schema_only: bool,

    /// Output format
    #[arg(long, default_value = "human")]
    pub format: String,

    /// Strict validation mode
    #[arg(long)]
    pub strict: bool,

    /// Fix issues automatically where possible
    #[arg(long)]
    pub fix: bool,
}

#[derive(Debug)]
struct ValidationResult {
    passed: usize,
    warnings: usize,
    errors: usize,
    issues: Vec<ValidationIssue>,
}

#[derive(Debug)]
struct ValidationIssue {
    level: ValidationLevel,
    file: Option<PathBuf>,
    message: String,
    suggestion: Option<String>,
}

#[derive(Debug, Clone)]
enum ValidationLevel {
    Error,
    Warning,
    #[allow(dead_code)]
    Info,
}

pub async fn execute(args: ValidateArgs, config: Option<Config>) -> Result<()> {
    println!("{}", "Validating MCP project...".green().bold());
    
    let path = match args.path {
        Some(ref path) => path.clone(),
        None => std::env::current_dir().context("Failed to get current directory")?,
    };
    println!("📁 Validating: {}", path.display());
    
    let mut result = ValidationResult {
        passed: 0,
        warnings: 0,
        errors: 0,
        issues: Vec::new(),
    };
    
    // Run validation checks
    validate_project_structure(&path, &args, &mut result).await?;
    validate_cargo_files(&path, &args, &mut result).await?;
    validate_mcp_config(&path, &config, &args, &mut result).await?;
    
    if !args.schema_only {
        validate_source_code(&path, &args, &mut result).await?;
        validate_examples(&path, &args, &mut result).await?;
    }
    
    // Output results
    output_results(&result, &args)?;
    
    // Apply fixes if requested
    if args.fix && !result.issues.is_empty() {
        apply_fixes(&result, &path, &args).await?;
    }
    
    if result.errors > 0 {
        anyhow::bail!("Validation failed with {} error(s)", result.errors);
    }
    
    Ok(())
}

async fn validate_project_structure(path: &Path, _args: &ValidateArgs, result: &mut ValidationResult) -> Result<()> {
    println!("🏗️  Validating project structure...");
    
    // Check for essential files
    let essential_files = [
        "Cargo.toml",
        "README.md",
    ];
    
    for file in &essential_files {
        let file_path = path.join(file);
        if file_path.exists() {
            result.passed += 1;
            println!("   ✅ {}", file);
        } else {
            result.errors += 1;
            result.issues.push(ValidationIssue {
                level: ValidationLevel::Error,
                file: Some(file_path.clone()),
                message: format!("Missing essential file: {}", file),
                suggestion: Some(format!("Create {}", file)),
            });
        }
    }
    
    // Check for crates directory if it's a workspace
    let cargo_toml_path = path.join("Cargo.toml");
    if cargo_toml_path.exists() {
        let cargo_content = fs::read_to_string(&cargo_toml_path)?;
        if cargo_content.contains("[workspace]") {
            let crates_dir = path.join("crates");
            if crates_dir.exists() {
                result.passed += 1;
                println!("   ✅ crates directory found");
            } else {
                result.warnings += 1;
                result.issues.push(ValidationIssue {
                    level: ValidationLevel::Warning,
                    file: Some(crates_dir),
                    message: "Workspace detected but no crates directory found".to_string(),
                    suggestion: Some("Consider organizing crates in a crates/ directory".to_string()),
                });
            }
        }
    }
    
    Ok(())
}

async fn validate_cargo_files(path: &Path, args: &ValidateArgs, result: &mut ValidationResult) -> Result<()> {
    println!("📦 Validating Cargo files...");
    
    let cargo_toml_path = path.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        result.errors += 1;
        result.issues.push(ValidationIssue {
            level: ValidationLevel::Error,
            file: Some(cargo_toml_path),
            message: "Cargo.toml not found".to_string(),
            suggestion: Some("Initialize with 'cargo init'".to_string()),
        });
        return Ok(());
    }
    
    let cargo_content = fs::read_to_string(&cargo_toml_path)?;
    let cargo_toml: toml::Value = cargo_content.parse()
        .context("Failed to parse Cargo.toml")?;
    
    // Check package metadata
    if let Some(package) = cargo_toml.get("package") {
        check_package_field(package, "name", result, args.strict);
        check_package_field(package, "version", result, args.strict);
        check_package_field(package, "edition", result, args.strict);
        
        if args.strict {
            check_package_field(package, "description", result, true);
            check_package_field(package, "license", result, true);
            check_package_field(package, "repository", result, true);
        }
    }
    
    // Check dependencies
    if let Some(deps) = cargo_toml.get("dependencies") {
        validate_dependencies(deps, result, "dependencies")?;
    }
    
    // Check dev dependencies
    if let Some(dev_deps) = cargo_toml.get("dev-dependencies") {
        validate_dependencies(dev_deps, result, "dev-dependencies")?;
    }
    
    result.passed += 1;
    println!("   ✅ Cargo.toml is valid");
    
    Ok(())
}

async fn validate_mcp_config(path: &Path, config: &Option<Config>, _args: &ValidateArgs, result: &mut ValidationResult) -> Result<()> {
    println!("⚙️  Validating MCP configuration...");
    
    // Check for mcp.toml or mcp.json
    let config_files = ["mcp.toml", "mcp.json", ".mcp.toml", ".mcp.json"];
    let mut config_found = false;
    
    for config_file in &config_files {
        let config_path = path.join(config_file);
        if config_path.exists() {
            config_found = true;
            validate_config_file(&config_path, result).await?;
            break;
        }
    }
    
    if !config_found {
        result.warnings += 1;
        result.issues.push(ValidationIssue {
            level: ValidationLevel::Warning,
            file: None,
            message: "No MCP configuration file found".to_string(),
            suggestion: Some("Consider creating mcp.toml for project configuration".to_string()),
        });
    }
    
    // Validate loaded config if available
    if let Some(config) = config {
        validate_loaded_config(config, result)?;
    }
    
    Ok(())
}

async fn validate_source_code(path: &Path, _args: &ValidateArgs, result: &mut ValidationResult) -> Result<()> {
    println!("🦀 Validating source code...");
    
    let src_dir = path.join("src");
    if !src_dir.exists() {
        result.errors += 1;
        result.issues.push(ValidationIssue {
            level: ValidationLevel::Error,
            file: Some(src_dir),
            message: "src directory not found".to_string(),
            suggestion: Some("Create src directory with main.rs or lib.rs".to_string()),
        });
        return Ok(());
    }
    
    // Check for main.rs or lib.rs
    let main_rs = src_dir.join("main.rs");
    let lib_rs = src_dir.join("lib.rs");
    
    if !main_rs.exists() && !lib_rs.exists() {
        result.errors += 1;
        result.issues.push(ValidationIssue {
            level: ValidationLevel::Error,
            file: Some(src_dir.clone()),
            message: "Neither main.rs nor lib.rs found in src/".to_string(),
            suggestion: Some("Create main.rs for binary or lib.rs for library".to_string()),
        });
    } else {
        result.passed += 1;
        if main_rs.exists() {
            println!("   ✅ main.rs found");
        }
        if lib_rs.exists() {
            println!("   ✅ lib.rs found");
        }
    }
    
    Ok(())
}

async fn validate_examples(path: &Path, _args: &ValidateArgs, result: &mut ValidationResult) -> Result<()> {
    println!("📚 Validating examples...");
    
    let examples_dir = path.join("examples");
    if examples_dir.exists() {
        let mut example_count = 0;
        
        for entry in fs::read_dir(&examples_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                example_count += 1;
                let example_path = entry.path();
                let cargo_toml = example_path.join("Cargo.toml");
                
                if cargo_toml.exists() {
                    println!("   ✅ Example: {}", entry.file_name().to_string_lossy());
                } else {
                    result.warnings += 1;
                    result.issues.push(ValidationIssue {
                        level: ValidationLevel::Warning,
                        file: Some(cargo_toml),
                        message: format!("Example {} missing Cargo.toml", entry.file_name().to_string_lossy()),
                        suggestion: Some("Add Cargo.toml to example directory".to_string()),
                    });
                }
            }
        }
        
        if example_count > 0 {
            result.passed += 1;
            println!("   ✅ Found {} example(s)", example_count);
        }
    } else {
        result.warnings += 1;
        result.issues.push(ValidationIssue {
            level: ValidationLevel::Warning,
            file: Some(examples_dir),
            message: "No examples directory found".to_string(),
            suggestion: Some("Consider adding examples to demonstrate usage".to_string()),
        });
    }
    
    Ok(())
}

fn check_package_field(package: &toml::Value, field: &str, result: &mut ValidationResult, required: bool) {
    if package.get(field).is_some() {
        result.passed += 1;
    } else if required {
        result.errors += 1;
        result.issues.push(ValidationIssue {
            level: ValidationLevel::Error,
            file: None,
            message: format!("Missing required package field: {}", field),
            suggestion: Some(format!("Add {} field to [package] section", field)),
        });
    } else {
        result.warnings += 1;
        result.issues.push(ValidationIssue {
            level: ValidationLevel::Warning,
            file: None,
            message: format!("Missing recommended package field: {}", field),
            suggestion: Some(format!("Consider adding {} field to [package] section", field)),
        });
    }
}

fn validate_dependencies(deps: &toml::Value, result: &mut ValidationResult, section: &str) -> Result<()> {
    if let Some(deps_table) = deps.as_table() {
        for (name, spec) in deps_table {
            // Check for problematic version specifications
            if let Some(version_str) = spec.as_str() {
                if version_str == "*" {
                    result.warnings += 1;
                    result.issues.push(ValidationIssue {
                        level: ValidationLevel::Warning,
                        file: None,
                        message: format!("Wildcard version for dependency '{}' in {}", name, section),
                        suggestion: Some("Use specific version constraints".to_string()),
                    });
                }
            }
        }
        
        result.passed += 1;
    }
    
    Ok(())
}

async fn validate_config_file(config_path: &Path, result: &mut ValidationResult) -> Result<()> {
    let content = fs::read_to_string(config_path)?;
    
    // Try to parse as appropriate format
    match config_path.extension().and_then(|s| s.to_str()) {
        Some("toml") => {
            match content.parse::<toml::Value>() {
                Ok(_) => {
                    result.passed += 1;
                    println!("   ✅ {} is valid TOML", config_path.file_name().unwrap().to_string_lossy());
                }
                Err(e) => {
                    result.errors += 1;
                    result.issues.push(ValidationIssue {
                        level: ValidationLevel::Error,
                        file: Some(config_path.to_path_buf()),
                        message: format!("Invalid TOML: {}", e),
                        suggestion: Some("Fix TOML syntax errors".to_string()),
                    });
                }
            }
        }
        Some("json") => {
            match serde_json::from_str::<Value>(&content) {
                Ok(_) => {
                    result.passed += 1;
                    println!("   ✅ {} is valid JSON", config_path.file_name().unwrap().to_string_lossy());
                }
                Err(e) => {
                    result.errors += 1;
                    result.issues.push(ValidationIssue {
                        level: ValidationLevel::Error,
                        file: Some(config_path.to_path_buf()),
                        message: format!("Invalid JSON: {}", e),
                        suggestion: Some("Fix JSON syntax errors".to_string()),
                    });
                }
            }
        }
        _ => {
            result.warnings += 1;
            result.issues.push(ValidationIssue {
                level: ValidationLevel::Warning,
                file: Some(config_path.to_path_buf()),
                message: "Unknown config file format".to_string(),
                suggestion: Some("Use .toml or .json extension".to_string()),
            });
        }
    }
    
    Ok(())
}

fn validate_loaded_config(config: &Config, result: &mut ValidationResult) -> Result<()> {
    // Validate project configuration
    if config.project.name.is_empty() {
        result.errors += 1;
        result.issues.push(ValidationIssue {
            level: ValidationLevel::Error,
            file: None,
            message: "Project name is empty".to_string(),
            suggestion: Some("Set project.name in configuration".to_string()),
        });
    }
    
    // Validate servers
    for (name, server) in &config.servers {
        if server.name != *name {
            result.warnings += 1;
            result.issues.push(ValidationIssue {
                level: ValidationLevel::Warning,
                file: None,
                message: format!("Server name mismatch: key '{}' vs name '{}'", name, server.name),
                suggestion: Some("Ensure server key matches server name".to_string()),
            });
        }
    }
    
    result.passed += 1;
    println!("   ✅ Loaded configuration is valid");
    
    Ok(())
}

fn output_results(result: &ValidationResult, args: &ValidateArgs) -> Result<()> {
    println!("\n📊 Validation Results:");
    
    match args.format.as_str() {
        "human" => output_human_format(result),
        "json" => output_json_format(result)?,
        _ => anyhow::bail!("Unsupported output format: {}", args.format),
    }
    
    Ok(())
}

fn output_human_format(result: &ValidationResult) {
    println!("   Passed:   {}", result.passed.to_string().green());
    println!("   Warnings: {}", result.warnings.to_string().yellow());
    println!("   Errors:   {}", result.errors.to_string().red());
    println!("   Total:    {}", (result.passed + result.warnings + result.errors));
    
    if !result.issues.is_empty() {
        println!("\n🔍 Issues found:");
        for issue in &result.issues {
            let level_str = match issue.level {
                ValidationLevel::Error => "❌ ERROR".red(),
                ValidationLevel::Warning => "⚠️  WARN".yellow(),
                ValidationLevel::Info => "ℹ️  INFO".blue(),
            };
            
            println!("   {} {}", level_str, issue.message);
            
            if let Some(file) = &issue.file {
                println!("      File: {}", file.display().to_string().dimmed());
            }
            
            if let Some(suggestion) = &issue.suggestion {
                println!("      Suggestion: {}", suggestion.green().dimmed());
            }
            
            println!();
        }
    }
}

fn output_json_format(result: &ValidationResult) -> Result<()> {
    let json_result = serde_json::json!({
        "summary": {
            "passed": result.passed,
            "warnings": result.warnings,
            "errors": result.errors,
            "total": result.passed + result.warnings + result.errors
        },
        "issues": result.issues.iter().map(|issue| {
            serde_json::json!({
                "level": match issue.level {
                    ValidationLevel::Error => "error",
                    ValidationLevel::Warning => "warning",
                    ValidationLevel::Info => "info",
                },
                "file": issue.file.as_ref().map(|p| p.to_string_lossy()),
                "message": issue.message,
                "suggestion": issue.suggestion
            })
        }).collect::<Vec<_>>()
    });
    
    println!("{}", serde_json::to_string_pretty(&json_result)?);
    Ok(())
}

async fn apply_fixes(result: &ValidationResult, _path: &Path, args: &ValidateArgs) -> Result<()> {
    if args.fix {
        println!("\n🔧 Applying automatic fixes...");
        
        let fixable_issues: Vec<_> = result.issues.iter()
            .filter(|issue| issue.suggestion.is_some())
            .collect();
        
        if fixable_issues.is_empty() {
            println!("   No automatic fixes available");
        } else {
            println!("   {} automatic fix(es) available", fixable_issues.len());
            // TODO: Implement specific fixes based on issue types
            println!("   ⚠️  Automatic fixing not yet implemented");
        }
    }
    
    Ok(())
}
