use serde::{Deserialize, Serialize};

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client name
    pub name: String,
    /// Client version
    pub version: String,
    /// Optional additional information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional homepage URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    /// Optional repository URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// Optional author information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    /// Optional license information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

impl Default for ClientInfo {
    fn default() -> Self {
        Self {
            name: "default-client".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            homepage: None,
            repository: None,
            authors: None,
            license: None,
        }
    }
}

impl ClientInfo {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            description: None,
            homepage: None,
            repository: None,
            authors: None,
            license: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_homepage(mut self, homepage: String) -> Self {
        self.homepage = Some(homepage);
        self
    }

    pub fn with_repository(mut self, repository: String) -> Self {
        self.repository = Some(repository);
        self
    }

    pub fn with_authors(mut self, authors: Vec<String>) -> Self {
        self.authors = Some(authors);
        self
    }

    pub fn with_license(mut self, license: String) -> Self {
        self.license = Some(license);
        self
    }
}

// Re-export protocol capabilities for convenience
pub use crate::protocol::capabilities::{
    ClientCapabilities, ElicitationCapability, RootsCapability, SamplingCapability,
};
