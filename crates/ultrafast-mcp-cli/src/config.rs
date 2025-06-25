use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use anyhow::{Result, Context};

/// CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Project settings
    pub project: ProjectConfig,
    /// Server configurations
    pub servers: HashMap<String, ServerConfig>,
    /// Client configurations
    pub clients: HashMap<String, ClientConfig>,
    /// Template configurations
    pub templates: TemplateConfig,
    /// Development settings
    pub dev: DevConfig,
}

/// Project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name
    pub name: String,
    /// Project version
    pub version: String,
    /// Project description
    pub description: Option<String>,
    /// Author information
    pub author: Option<String>,
    /// License
    pub license: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// Server capabilities
    pub capabilities: ServerCapabilities,
    /// Transport configuration
    pub transport: TransportConfig,
    /// Tool configurations
    pub tools: Vec<ToolConfig>,
    /// Resource configurations
    pub resources: Vec<ResourceConfig>,
}

/// Server capabilities configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Experimental capabilities
    pub experimental: HashMap<String, serde_json::Value>,
    /// Logging capability
    pub logging: Option<LoggingCapability>,
    /// Prompts capability
    pub prompts: Option<PromptsCapability>,
    /// Resources capability
    pub resources: Option<ResourcesCapability>,
    /// Tools capability
    pub tools: Option<ToolsCapability>,
}

/// Logging capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingCapability {
    /// Supported log levels
    pub levels: Vec<String>,
}

/// Prompts capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    /// Supports list changes
    pub list_changed: bool,
}

/// Resources capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    /// Supports subscribe
    pub subscribe: bool,
    /// Supports list changes
    pub list_changed: bool,
}

/// Tools capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    /// Supports list changes
    pub list_changed: bool,
}

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Transport type (stdio, http, etc.)
    pub transport_type: String,
    /// Additional configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: Option<String>,
    /// Input schema
    pub input_schema: serde_json::Value,
    /// Handler configuration
    pub handler: HandlerConfig,
}

/// Resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Resource URI
    pub uri: String,
    /// Resource name
    pub name: String,
    /// Resource description
    pub description: Option<String>,
    /// Resource MIME type
    pub mime_type: Option<String>,
    /// Handler configuration
    pub handler: HandlerConfig,
}

/// Handler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerConfig {
    /// Handler type
    pub handler_type: String,
    /// Handler configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Client name
    pub name: String,
    /// Client version
    pub version: String,
    /// Client capabilities
    pub capabilities: ClientCapabilities,
    /// Server connection settings
    pub server: ServerConnectionConfig,
}

/// Client capabilities configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Experimental capabilities
    pub experimental: HashMap<String, serde_json::Value>,
    /// Sampling capability
    pub sampling: Option<SamplingCapability>,
}

/// Sampling capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingCapability {
    /// Sampling configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Server connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConnectionConfig {
    /// Server endpoint
    pub endpoint: String,
    /// Transport configuration
    pub transport: TransportConfig,
    /// Connection timeout
    pub timeout: Option<u64>,
    /// Retry configuration
    pub retry: Option<RetryConfig>,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Retry delay in milliseconds
    pub delay_ms: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

/// Template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Template directory
    pub template_dir: Option<String>,
    /// Available templates
    pub templates: HashMap<String, TemplateInfo>,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            template_dir: None,
            templates: HashMap::new(),
        }
    }
}

/// Template information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Template path
    pub path: String,
    /// Template variables
    pub variables: HashMap<String, TemplateVariable>,
}

/// Template variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable description
    pub description: String,
    /// Default value
    pub default: Option<String>,
    /// Whether the variable is required
    pub required: bool,
}

/// Development configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevConfig {
    /// Watch configuration
    pub watch: WatchConfig,
    /// Hot reload settings
    pub hot_reload: bool,
    /// Development server port
    pub port: Option<u16>,
    /// Log level for development
    pub log_level: String,
}

impl Default for DevConfig {
    fn default() -> Self {
        Self {
            watch: WatchConfig::default(),
            hot_reload: true,
            port: Some(8080),
            log_level: "info".to_string(),
        }
    }
}

/// Watch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    /// Directories to watch
    pub directories: Vec<String>,
    /// File patterns to watch
    pub patterns: Vec<String>,
    /// File patterns to ignore
    pub ignore_patterns: Vec<String>,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            directories: vec!["src".to_string(), "templates".to_string()],
            patterns: vec!["**/*.rs".to_string(), "**/*.toml".to_string()],
            ignore_patterns: vec!["target/**".to_string(), "**/*.tmp".to_string()],
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project: ProjectConfig {
                name: "my-mcp-project".to_string(),
                version: "0.1.0".to_string(),
                description: None,
                author: None,
                license: Some("MIT".to_string()),
                repository: None,
            },
            servers: HashMap::new(),
            clients: HashMap::new(),
            templates: TemplateConfig {
                template_dir: None,
                templates: HashMap::new(),
            },
            dev: DevConfig {
                watch: WatchConfig {
                    directories: vec!["src".to_string(), "templates".to_string()],
                    patterns: vec!["**/*.rs".to_string(), "**/*.toml".to_string()],
                    ignore_patterns: vec!["target/**".to_string(), "**/*.tmp".to_string()],
                },
                hot_reload: true,
                port: Some(8080),
                log_level: "info".to_string(),
            },
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        
        Ok(config)
    }

    /// Save configuration to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;
        
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        
        Ok(())
    }

    /// Get default configuration file path
    pub fn default_path() -> Result<std::path::PathBuf> {
        let mut path = dirs::config_dir()
            .context("Failed to get config directory")?;
        path.push("ultrafast-mcp");
        path.push("config.toml");
        Ok(path)
    }

    /// Create default configuration file
    pub fn create_default() -> Result<()> {
        let path = Self::default_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }
        
        let config = Self::default();
        config.save(&path)?;
        
        Ok(())
    }
}
