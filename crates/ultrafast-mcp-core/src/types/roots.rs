//! Roots types for MCP
//!
//! Filesystem boundary management for security-conscious path validation

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// A filesystem root that defines boundary for file operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Root {
    /// The URI of the root (typically file://)
    pub uri: String,
    /// Optional human-readable name for the root
    pub name: Option<String>,
    /// Optional security configuration for this root
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<RootSecurityConfig>,
}

/// Security configuration for a root
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RootSecurityConfig {
    /// Whether to allow read operations
    #[serde(default = "default_true")]
    pub allow_read: bool,
    /// Whether to allow write operations
    #[serde(default = "default_false")]
    pub allow_write: bool,
    /// Whether to allow execute operations
    #[serde(default = "default_false")]
    pub allow_execute: bool,
    /// Maximum file size allowed (in bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_file_size: Option<u64>,
    /// Allowed file extensions (if specified, only these extensions are allowed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_extensions: Option<Vec<String>>,
    /// Blocked file extensions (these extensions are always blocked)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_extensions: Option<Vec<String>>,
    /// Blocked directory names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_directories: Option<Vec<String>>,
}

fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}

impl Default for RootSecurityConfig {
    fn default() -> Self {
        Self {
            allow_read: true,
            allow_write: false,
            allow_execute: false,
            max_file_size: Some(100 * 1024 * 1024), // 100MB default
            allowed_extensions: None,
            blocked_extensions: Some(vec![
                "exe".to_string(),
                "bat".to_string(),
                "cmd".to_string(),
                "scr".to_string(),
                "pif".to_string(),
                "com".to_string(),
                "dll".to_string(),
                "so".to_string(),
                "dylib".to_string(),
            ]),
            blocked_directories: Some(vec![
                ".git".to_string(),
                ".svn".to_string(),
                ".hg".to_string(),
                "node_modules".to_string(),
                "__pycache__".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
            ]),
        }
    }
}

/// Request to list available roots
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListRootsRequest {
    // No parameters needed for listing roots
}

/// Response containing available roots
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListRootsResponse {
    /// List of available roots
    pub roots: Vec<Root>,
}

/// Notification sent when the list of roots changes
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RootListChangedNotification {
    /// Updated list of roots
    pub roots: Vec<Root>,
}

/// Operation type for security validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootOperation {
    Read,
    Write,
    Execute,
    List,
}

/// Comprehensive security validator for roots
#[derive(Debug, Clone)]
pub struct RootSecurityValidator {
    /// Maximum allowed path depth
    max_path_depth: usize,
    /// Global blocked paths (always forbidden)
    global_blocked_paths: HashSet<String>,
    /// Global blocked patterns (regex patterns)
    global_blocked_patterns: Vec<regex::Regex>,
}

impl Default for RootSecurityValidator {
    fn default() -> Self {
        let mut global_blocked_paths = HashSet::new();

        // System directories that should never be accessible
        let system_paths = [
            "/etc/passwd",
            "/etc/shadow",
            "/etc/sudoers",
            "/proc",
            "/sys",
            "/dev",
            "/boot",
            "/Windows/System32",
            "/Windows/SysWOW64",
            "C:\\Windows\\System32",
            "C:\\Windows\\SysWOW64",
        ];

        for path in &system_paths {
            global_blocked_paths.insert(path.to_string());
        }

        // Compile blocked patterns
        let pattern_strings = [
            r".*\.key$",       // Private keys
            r".*\.pem$",       // Certificates
            r".*\.p12$",       // PKCS12 files
            r".*\.keystore$",  // Java keystores
            r".*\.env$",       // Environment files
            r".*\.secret$",    // Secret files
            r".*password.*",   // Password files
            r".*credential.*", // Credential files
        ];

        let mut global_blocked_patterns = Vec::new();
        for pattern in &pattern_strings {
            if let Ok(regex) = regex::Regex::new(pattern) {
                global_blocked_patterns.push(regex);
            }
        }

        Self {
            max_path_depth: 20,
            global_blocked_paths,
            global_blocked_patterns,
        }
    }
}

impl RootSecurityValidator {
    /// Create a new validator with custom settings
    pub fn new(max_path_depth: usize) -> Self {
        Self {
            max_path_depth,
            ..Default::default()
        }
    }

    /// Add a globally blocked path
    pub fn add_blocked_path(&mut self, path: String) {
        self.global_blocked_paths.insert(path);
    }

    /// Add a globally blocked pattern
    pub fn add_blocked_pattern(&mut self, pattern: &str) -> Result<(), regex::Error> {
        let regex = regex::Regex::new(pattern)?;
        self.global_blocked_patterns.push(regex);
        Ok(())
    }

    /// Validate a path access request against a root
    pub fn validate_access(
        &self,
        root: &Root,
        target_path: &str,
        operation: RootOperation,
    ) -> Result<(), RootSecurityError> {
        // Check global blocks and patterns first (before path validation)
        self.validate_global_blocks(target_path)?;
        self.validate_file_extension(target_path, &root.security)?;
        self.validate_directory_access(target_path, &root.security)?;
        self.validate_operation_permissions(operation, &root.security)?;

        // Then do path validation and depth checks
        self.validate_path_within_root(&root.uri, target_path)?;
        self.validate_path_depth_relative(&root.uri, target_path)?;

        Ok(())
    }

    /// Validate that a path is within the allowed root and secure
    pub fn validate_path_within_root(
        &self,
        root_uri: &str,
        path: &str,
    ) -> Result<(), RootSecurityError> {
        let root_path = uri_to_path(root_uri)?;
        let target_path = uri_to_path(path)?;

        // Check for directory traversal patterns FIRST (before path resolution)
        let path_str = target_path.to_string_lossy();
        if path_str.contains("..") || path_str.contains("./") || path_str.contains(".\\") {
            return Err(RootSecurityError::DirectoryTraversalAttempt);
        }

        // Try to canonicalize paths if they exist, otherwise do basic validation
        let (canonical_root, canonical_target) =
            match (root_path.canonicalize(), target_path.canonicalize()) {
                (Ok(root), Ok(target)) => (root, target),
                (Ok(root), Err(_)) => {
                    // Target doesn't exist, validate against parent structure
                    let mut parent = target_path.parent();
                    while let Some(p) = parent {
                        if let Ok(canonical_parent) = p.canonicalize() {
                            let relative_path = target_path.strip_prefix(p).unwrap_or(&target_path);
                            return self.validate_basic_path_security(
                                &root,
                                &canonical_parent.join(relative_path),
                            );
                        }
                        parent = p.parent();
                    }
                    // No existing parent found, use basic validation
                    return self.validate_basic_path_security(&root_path, &target_path);
                }
                (Err(_), _) => {
                    // Root doesn't exist (common in tests), use basic validation
                    return self.validate_basic_path_security(&root_path, &target_path);
                }
            };

        // Check if target is within root
        if !canonical_target.starts_with(&canonical_root) {
            return Err(RootSecurityError::PathOutsideRoot);
        }

        Ok(())
    }

    /// Basic path security validation for when canonicalization isn't available
    fn validate_basic_path_security(
        &self,
        root_path: &Path,
        target_path: &Path,
    ) -> Result<(), RootSecurityError> {
        // Normalize paths by removing . and .. components
        let normalized_root = self.normalize_path_components(root_path);
        let normalized_target = self.normalize_path_components(target_path);

        // Check if target is within root
        if !normalized_target.starts_with(&normalized_root) {
            return Err(RootSecurityError::PathOutsideRoot);
        }

        // Check for directory traversal patterns
        let path_str = target_path.to_string_lossy();
        if path_str.contains("..") {
            return Err(RootSecurityError::DirectoryTraversalAttempt);
        }

        Ok(())
    }

    /// Normalize path by processing components to remove . and ..
    fn normalize_path_components(&self, path: &Path) -> PathBuf {
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                std::path::Component::Normal(name) => normalized.push(name),
                std::path::Component::RootDir => normalized.push("/"),
                std::path::Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
                std::path::Component::ParentDir => {
                    // Don't allow going above root
                    if normalized.parent().is_some() {
                        normalized.pop();
                    }
                }
                std::path::Component::CurDir => {
                    // Skip current directory references
                }
            }
        }
        normalized
    }

    /// Validate path depth relative to root
    fn validate_path_depth_relative(
        &self,
        root_uri: &str,
        target_path: &str,
    ) -> Result<(), RootSecurityError> {
        let root_path = uri_to_path(root_uri)?;
        let target_path = uri_to_path(target_path)?;

        // Calculate relative path depth
        let relative_depth = if let Ok(relative) = target_path.strip_prefix(&root_path) {
            relative.components().count()
        } else {
            // If we can't calculate relative path, use absolute depth
            target_path.components().count()
        };

        if relative_depth > self.max_path_depth {
            return Err(RootSecurityError::PathTooDeep(
                relative_depth,
                self.max_path_depth,
            ));
        }

        Ok(())
    }

    /// Validate against global blocked paths and patterns
    fn validate_global_blocks(&self, path: &str) -> Result<(), RootSecurityError> {
        let path_lower = path.to_lowercase();

        // Check global blocked paths
        if self
            .global_blocked_paths
            .iter()
            .any(|blocked| path_lower.contains(&blocked.to_lowercase()))
        {
            return Err(RootSecurityError::GloballyBlockedPath);
        }

        // Check global blocked patterns
        for pattern in &self.global_blocked_patterns {
            if pattern.is_match(&path_lower) {
                return Err(RootSecurityError::MatchesBlockedPattern);
            }
        }

        Ok(())
    }

    /// Validate file extension against security config
    fn validate_file_extension(
        &self,
        path: &str,
        security: &Option<RootSecurityConfig>,
    ) -> Result<(), RootSecurityError> {
        let path = Path::new(path);
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        if let Some(security_config) = security {
            // Check if extension is explicitly blocked
            if let Some(ref blocked) = security_config.blocked_extensions {
                if let Some(ref ext) = extension {
                    if blocked
                        .iter()
                        .any(|blocked_ext| blocked_ext.to_lowercase() == *ext)
                    {
                        return Err(RootSecurityError::BlockedFileExtension(ext.clone()));
                    }
                }
            }

            // Check if only specific extensions are allowed
            if let Some(ref allowed) = security_config.allowed_extensions {
                if let Some(ref ext) = extension {
                    if !allowed
                        .iter()
                        .any(|allowed_ext| allowed_ext.to_lowercase() == *ext)
                    {
                        return Err(RootSecurityError::FileExtensionNotAllowed(ext.clone()));
                    }
                } else {
                    // File has no extension, but allowed extensions are specified
                    return Err(RootSecurityError::FileExtensionNotAllowed(
                        "(none)".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate directory access against security config
    fn validate_directory_access(
        &self,
        path: &str,
        security: &Option<RootSecurityConfig>,
    ) -> Result<(), RootSecurityError> {
        if let Some(security_config) = security {
            if let Some(ref blocked_dirs) = security_config.blocked_directories {
                let path = Path::new(path);
                for component in path.components() {
                    if let Some(dir_name) = component.as_os_str().to_str() {
                        if blocked_dirs
                            .iter()
                            .any(|blocked| blocked.to_lowercase() == dir_name.to_lowercase())
                        {
                            return Err(RootSecurityError::BlockedDirectory(dir_name.to_string()));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate operation permissions
    fn validate_operation_permissions(
        &self,
        operation: RootOperation,
        security: &Option<RootSecurityConfig>,
    ) -> Result<(), RootSecurityError> {
        if let Some(security_config) = security {
            match operation {
                RootOperation::Read | RootOperation::List => {
                    if !security_config.allow_read {
                        return Err(RootSecurityError::OperationNotAllowed("read".to_string()));
                    }
                }
                RootOperation::Write => {
                    if !security_config.allow_write {
                        return Err(RootSecurityError::OperationNotAllowed("write".to_string()));
                    }
                }
                RootOperation::Execute => {
                    if !security_config.allow_execute {
                        return Err(RootSecurityError::OperationNotAllowed(
                            "execute".to_string(),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate file size (if the file exists)
    pub fn validate_file_size(
        &self,
        path: &Path,
        security: &Option<RootSecurityConfig>,
    ) -> Result<(), RootSecurityError> {
        if let Some(security_config) = security {
            if let Some(max_size) = security_config.max_file_size {
                if let Ok(metadata) = std::fs::metadata(path) {
                    if metadata.len() > max_size {
                        return Err(RootSecurityError::FileTooLarge(metadata.len(), max_size));
                    }
                }
            }
        }

        Ok(())
    }
}

/// Legacy function for backward compatibility
pub fn validate_path_within_root(root_uri: &str, path: &str) -> Result<(), RootSecurityError> {
    let validator = RootSecurityValidator::default();
    validator.validate_path_within_root(root_uri, path)
}

/// Convert a file:// URI to a PathBuf
fn uri_to_path(uri: &str) -> Result<PathBuf, RootSecurityError> {
    if let Some(stripped) = uri.strip_prefix("file://") {
        // Handle Windows paths that might have drive letters
        #[cfg(windows)]
        {
            if stripped.len() >= 3 && stripped.chars().nth(0).unwrap() == '/' {
                let without_slash = &stripped[1..];
                if without_slash.chars().nth(1) == Some(':') {
                    return Ok(PathBuf::from(without_slash));
                }
            }
        }

        Ok(PathBuf::from(stripped))
    } else {
        Err(RootSecurityError::InvalidUri)
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RootSecurityError {
    #[error("Path is outside the allowed root")]
    PathOutsideRoot,
    #[error("Directory traversal attempt detected")]
    DirectoryTraversalAttempt,
    #[error("Invalid URI format")]
    InvalidUri,
    #[error("Invalid root path")]
    InvalidRootPath,
    #[error("Path depth {0} exceeds maximum allowed depth {1}")]
    PathTooDeep(usize, usize),
    #[error("Path is globally blocked for security reasons")]
    GloballyBlockedPath,
    #[error("Path matches a blocked security pattern")]
    MatchesBlockedPattern,
    #[error("File extension '{0}' is blocked")]
    BlockedFileExtension(String),
    #[error("File extension '{0}' is not in the allowed list")]
    FileExtensionNotAllowed(String),
    #[error("Directory '{0}' is blocked")]
    BlockedDirectory(String),
    #[error("Operation '{0}' is not allowed")]
    OperationNotAllowed(String),
    #[error("File size {0} bytes exceeds maximum allowed size {1} bytes")]
    FileTooLarge(u64, u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_root(uri: &str, security: Option<RootSecurityConfig>) -> Root {
        Root {
            uri: uri.to_string(),
            name: Some("test".to_string()),
            security,
        }
    }

    #[test]
    fn test_valid_path_within_root() {
        let root = "file:///tmp/root";
        let path = "file:///tmp/root/subdir/file.txt";
        assert_eq!(validate_path_within_root(root, path), Ok(()));
    }

    #[test]
    fn test_path_outside_root() {
        let root = "file:///tmp/root";
        let path = "file:///tmp/other/file.txt";
        let err = validate_path_within_root(root, path).unwrap_err();
        assert_eq!(err, RootSecurityError::PathOutsideRoot);
    }

    #[test]
    fn test_directory_traversal_attempt() {
        let root = "file:///tmp/root";
        let path = "file:///tmp/root/../etc/passwd";
        let err = validate_path_within_root(root, path).unwrap_err();
        assert_eq!(err, RootSecurityError::DirectoryTraversalAttempt);
    }

    #[test]
    fn test_invalid_uri() {
        let root = "/tmp/root";
        let path = "file:///tmp/root/file.txt";
        let err = validate_path_within_root(root, path).unwrap_err();
        assert_eq!(err, RootSecurityError::InvalidUri);
    }

    #[test]
    fn test_comprehensive_security_validator() {
        let validator = RootSecurityValidator::default();
        let root = create_test_root("file:///tmp/root", Some(RootSecurityConfig::default()));

        // Test valid access
        assert!(validator
            .validate_access(&root, "file:///tmp/root/test.txt", RootOperation::Read)
            .is_ok());
    }

    #[test]
    fn test_path_depth_validation() {
        let validator = RootSecurityValidator::new(3);
        let root = create_test_root("file:///tmp", None);

        // Path within depth limit should pass
        assert!(validator
            .validate_access(&root, "file:///tmp/a/b/c.txt", RootOperation::Read)
            .is_ok());

        // Path exceeding depth limit should fail
        let err = validator
            .validate_access(&root, "file:///tmp/a/b/c/d/e.txt", RootOperation::Read)
            .unwrap_err();
        assert!(matches!(err, RootSecurityError::PathTooDeep(_, _)));
    }

    #[test]
    fn test_global_blocked_paths() {
        let validator = RootSecurityValidator::default();
        let root = create_test_root("file:///", None);

        // System paths should be blocked
        let err = validator
            .validate_access(&root, "file:///etc/passwd", RootOperation::Read)
            .unwrap_err();
        assert_eq!(err, RootSecurityError::GloballyBlockedPath);
    }

    #[test]
    fn test_blocked_file_extensions() {
        let security_config = RootSecurityConfig {
            blocked_extensions: Some(vec!["exe".to_string(), "bat".to_string()]),
            ..Default::default()
        };

        let validator = RootSecurityValidator::default();
        let root = create_test_root("file:///tmp/root", Some(security_config));

        // Blocked extension should fail
        let err = validator
            .validate_access(&root, "file:///tmp/root/malware.exe", RootOperation::Read)
            .unwrap_err();
        assert!(matches!(err, RootSecurityError::BlockedFileExtension(_)));

        // Safe extension should pass
        assert!(validator
            .validate_access(&root, "file:///tmp/root/document.txt", RootOperation::Read)
            .is_ok());
    }

    #[test]
    fn test_allowed_file_extensions() {
        let security_config = RootSecurityConfig {
            allowed_extensions: Some(vec!["txt".to_string(), "md".to_string()]),
            blocked_extensions: None,
            ..Default::default()
        };

        let validator = RootSecurityValidator::default();
        let root = create_test_root("file:///tmp/root", Some(security_config));

        // Allowed extension should pass
        assert!(validator
            .validate_access(&root, "file:///tmp/root/document.txt", RootOperation::Read)
            .is_ok());

        // Non-allowed extension should fail
        let err = validator
            .validate_access(&root, "file:///tmp/root/script.py", RootOperation::Read)
            .unwrap_err();
        assert!(matches!(err, RootSecurityError::FileExtensionNotAllowed(_)));
    }

    #[test]
    fn test_blocked_directories() {
        let security_config = RootSecurityConfig {
            blocked_directories: Some(vec![".git".to_string(), "node_modules".to_string()]),
            ..Default::default()
        };

        let validator = RootSecurityValidator::default();
        let root = create_test_root("file:///tmp/root", Some(security_config));

        // Blocked directory should fail
        let err = validator
            .validate_access(&root, "file:///tmp/root/.git/config", RootOperation::Read)
            .unwrap_err();
        assert!(matches!(err, RootSecurityError::BlockedDirectory(_)));

        // Safe directory should pass
        assert!(validator
            .validate_access(&root, "file:///tmp/root/src/main.rs", RootOperation::Read)
            .is_ok());
    }

    #[test]
    fn test_operation_permissions() {
        let security_config = RootSecurityConfig {
            allow_read: true,
            allow_write: false,
            allow_execute: false,
            ..Default::default()
        };

        let validator = RootSecurityValidator::default();
        let root = create_test_root("file:///tmp/root", Some(security_config));

        // Read operation should pass
        assert!(validator
            .validate_access(&root, "file:///tmp/root/file.txt", RootOperation::Read)
            .is_ok());

        // Write operation should fail
        let err = validator
            .validate_access(&root, "file:///tmp/root/file.txt", RootOperation::Write)
            .unwrap_err();
        assert!(matches!(err, RootSecurityError::OperationNotAllowed(_)));

        // Execute operation should fail
        let err = validator
            .validate_access(&root, "file:///tmp/root/script.sh", RootOperation::Execute)
            .unwrap_err();
        assert!(matches!(err, RootSecurityError::OperationNotAllowed(_)));
    }

    #[test]
    fn test_security_patterns() {
        let validator = RootSecurityValidator::default();
        let root = create_test_root("file:///tmp/root", None);

        // Files matching security patterns should be blocked
        let test_cases = [
            "file:///tmp/root/private.key",
            "file:///tmp/root/cert.pem",
            "file:///tmp/root/passwords.txt",
            "file:///tmp/root/.env",
        ];

        for path in &test_cases {
            let result = validator.validate_access(&root, path, RootOperation::Read);
            assert!(result.is_err(), "Path {} should be blocked", path);
        }
    }

    #[test]
    fn test_custom_blocked_paths_and_patterns() {
        let mut validator = RootSecurityValidator::default();
        validator.add_blocked_path("/custom/blocked".to_string());
        validator.add_blocked_pattern(r".*\.secret$").unwrap();

        let root = create_test_root("file:///tmp/root", None);

        // Custom blocked path should fail
        let err = validator
            .validate_access(
                &root,
                "file:///custom/blocked/file.txt",
                RootOperation::Read,
            )
            .unwrap_err();
        assert_eq!(err, RootSecurityError::GloballyBlockedPath);

        // Custom pattern should fail
        let err = validator
            .validate_access(&root, "file:///tmp/root/data.secret", RootOperation::Read)
            .unwrap_err();
        assert_eq!(err, RootSecurityError::MatchesBlockedPattern);
    }

    #[test]
    fn test_root_security_config_default() {
        let config = RootSecurityConfig::default();

        assert!(config.allow_read);
        assert!(!config.allow_write);
        assert!(!config.allow_execute);
        assert_eq!(config.max_file_size, Some(100 * 1024 * 1024));
        assert!(config.blocked_extensions.is_some());
        assert!(config.blocked_directories.is_some());
    }

    #[test]
    fn test_windows_path_handling() {
        #[cfg(windows)]
        {
            let uri = "file:///C:/Users/test/file.txt";
            let path = uri_to_path(uri).unwrap();
            assert_eq!(path, PathBuf::from("C:/Users/test/file.txt"));
        }
    }

    #[test]
    fn test_comprehensive_security_validation() {
        let security_config = RootSecurityConfig {
            allow_read: true,
            allow_write: false,
            blocked_extensions: Some(vec!["exe".to_string()]),
            blocked_directories: Some(vec![".git".to_string()]),
            ..Default::default()
        };

        let validator = RootSecurityValidator::new(10);
        let root = create_test_root("file:///tmp/safe", Some(security_config));

        // Valid file should pass all checks
        assert!(validator
            .validate_access(&root, "file:///tmp/safe/doc.txt", RootOperation::Read)
            .is_ok());

        // Test multiple failure scenarios
        assert!(validator
            .validate_access(&root, "file:///etc/passwd", RootOperation::Read)
            .is_err());
        assert!(validator
            .validate_access(&root, "file:///tmp/safe/malware.exe", RootOperation::Read)
            .is_err());
        assert!(validator
            .validate_access(&root, "file:///tmp/safe/.git/config", RootOperation::Read)
            .is_err());
        assert!(validator
            .validate_access(&root, "file:///tmp/safe/doc.txt", RootOperation::Write)
            .is_err());
    }
}
