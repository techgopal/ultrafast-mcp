//! Completion types for MCP 2025-06-18 protocol

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteRequest {
    /// Reference type ("prompts", "resource_templates", "tools", "resources")
    #[serde(rename = "ref")]
    pub ref_type: String,

    /// Reference name (prompt name, resource template URI, tool name, etc.)
    #[serde(rename = "name")]
    pub ref_name: String,

    /// Argument to complete (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argument: Option<String>,

    /// Context for completion (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<CompletionContext>,

    /// Filter criteria (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<CompletionFilter>,

    /// Maximum number of completions to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,
}

/// Context for completion requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionContext {
    /// Current cursor position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor_position: Option<u32>,

    /// Current line content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_content: Option<String>,

    /// Document content around cursor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_content: Option<String>,

    /// Language or file type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Additional context metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Filter criteria for completions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletionFilter {
    /// Prefix to filter by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// Suffix to filter by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,

    /// Regex pattern to filter by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    /// Case sensitivity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_sensitive: Option<bool>,

    /// Minimum length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u32>,

    /// Maximum length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
}

/// Completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteResponse {
    /// Completion results
    pub completion: Completion,

    /// Metadata about the completion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<CompletionMetadata>,
}

/// Completion result set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completion {
    /// Completion values
    pub values: Vec<CompletionValue>,

    /// Total number of possible completions (for pagination)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u32>,

    /// Whether there are more completions available
    #[serde(rename = "hasMore", skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,

    /// Next cursor for pagination
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Individual completion value
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletionValue {
    /// Completion value
    pub value: String,

    /// Optional label (display name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Completion kind (function, variable, class, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<CompletionKind>,

    /// Sort priority (higher = more relevant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,

    /// Insert text (if different from value)
    #[serde(rename = "insertText", skip_serializing_if = "Option::is_none")]
    pub insert_text: Option<String>,

    /// Range to replace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<CompletionRange>,

    /// Documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<CompletionDocumentation>,
}

/// Completion kind
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompletionKind {
    Function,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Method,
    Constructor,
    Field,
    Enum,
    EnumMember,
    Keyword,
    Snippet,
    Text,
    Color,
    File,
    Reference,
    Folder,
    Unit,
    Value,
    Constant,
    Type,
    Namespace,
    Package,
    String,
    Number,
    Boolean,
    Array,
    Object,
    Null,
    Operator,
    TypeParameter,
}

/// Completion range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRange {
    /// Start position
    pub start: u32,
    /// End position
    pub end: u32,
}

/// Completion documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionDocumentation {
    /// Documentation text
    pub text: String,
    /// Documentation kind
    #[serde(rename = "kind", skip_serializing_if = "Option::is_none")]
    pub doc_kind: Option<DocumentationKind>,
}

/// Documentation kind
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentationKind {
    PlainText,
    Markdown,
    Html,
}

/// Completion metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionMetadata {
    /// Completion provider name
    pub provider: String,
    /// Completion trigger characters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_characters: Option<Vec<String>>,
    /// Whether completion is incomplete
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_incomplete: Option<bool>,
    /// Completion statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statistics: Option<CompletionStatistics>,
}

/// Completion statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionStatistics {
    /// Total time taken for completion
    pub total_time_ms: u64,
    /// Number of items processed
    pub items_processed: u32,
    /// Cache hit ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_hit_ratio: Option<f64>,
}

impl CompletionValue {
    /// Create a simple completion value
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: None,
            description: None,
            kind: None,
            priority: None,
            metadata: None,
            insert_text: None,
            range: None,
            documentation: None,
        }
    }

    /// Create a completion value with label
    pub fn with_label(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: Some(label.into()),
            description: None,
            kind: None,
            priority: None,
            metadata: None,
            insert_text: None,
            range: None,
            documentation: None,
        }
    }

    /// Create a completion value with description
    pub fn with_description(value: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: None,
            description: Some(description.into()),
            kind: None,
            priority: None,
            metadata: None,
            insert_text: None,
            range: None,
            documentation: None,
        }
    }

    /// Create a completion value with kind
    pub fn with_kind(value: impl Into<String>, kind: CompletionKind) -> Self {
        Self {
            value: value.into(),
            label: None,
            description: None,
            kind: Some(kind),
            priority: None,
            metadata: None,
            insert_text: None,
            range: None,
            documentation: None,
        }
    }

    /// Create a function completion
    pub fn function(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            value: name.into(),
            label: None,
            description: Some(description.into()),
            kind: Some(CompletionKind::Function),
            priority: Some(1.0),
            metadata: None,
            insert_text: None,
            range: None,
            documentation: None,
        }
    }

    /// Create a variable completion
    pub fn variable(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            value: name.into(),
            label: None,
            description: Some(description.into()),
            kind: Some(CompletionKind::Variable),
            priority: Some(0.8),
            metadata: None,
            insert_text: None,
            range: None,
            documentation: None,
        }
    }

    /// Create a snippet completion
    pub fn snippet(
        trigger: impl Into<String>,
        content: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            value: trigger.into(),
            label: None,
            description: Some(description.into()),
            kind: Some(CompletionKind::Snippet),
            priority: Some(1.2),
            metadata: None,
            insert_text: Some(content.into()),
            range: None,
            documentation: None,
        }
    }
}

impl Completion {
    /// Create a new completion set
    pub fn new(values: Vec<CompletionValue>) -> Self {
        Self {
            values,
            total: None,
            has_more: None,
            next_cursor: None,
        }
    }

    /// Create with metadata
    pub fn with_metadata(
        values: Vec<CompletionValue>,
        total: u32,
        has_more: bool,
    ) -> Self {
        Self {
            values,
            total: Some(total),
            has_more: Some(has_more),
            next_cursor: None,
        }
    }

    /// Create with pagination
    pub fn with_pagination(
        values: Vec<CompletionValue>,
        total: u32,
        has_more: bool,
        next_cursor: String,
    ) -> Self {
        Self {
            values,
            total: Some(total),
            has_more: Some(has_more),
            next_cursor: Some(next_cursor),
        }
    }

    /// Sort completions by priority
    pub fn sort_by_priority(&mut self) {
        self.values.sort_by(|a, b| {
            let a_priority = a.priority.unwrap_or(0.0);
            let b_priority = b.priority.unwrap_or(0.0);
            b_priority.partial_cmp(&a_priority).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Filter completions by prefix
    pub fn filter_by_prefix(&self, prefix: &str) -> Self {
        let filtered_values: Vec<CompletionValue> = self
            .values
            .iter()
            .filter(|cv| cv.value.to_lowercase().starts_with(&prefix.to_lowercase()))
            .cloned()
            .collect();

        let count = filtered_values.len() as u32;
        Self {
            values: filtered_values,
            total: Some(count),
            has_more: Some(false),
            next_cursor: None,
        }
    }
}

impl CompleteRequest {
    /// Create a new completion request
    pub fn new(ref_type: impl Into<String>, ref_name: impl Into<String>) -> Self {
        Self {
            ref_type: ref_type.into(),
            ref_name: ref_name.into(),
            argument: None,
            context: None,
            filter: None,
            max_results: None,
        }
    }

    /// Create with argument
    pub fn with_argument(
        ref_type: impl Into<String>,
        ref_name: impl Into<String>,
        argument: impl Into<String>,
    ) -> Self {
        Self {
            ref_type: ref_type.into(),
            ref_name: ref_name.into(),
            argument: Some(argument.into()),
            context: None,
            filter: None,
            max_results: None,
        }
    }

    /// Add context
    pub fn with_context(mut self, context: CompletionContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Add filter
    pub fn with_filter(mut self, filter: CompletionFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Set max results
    pub fn with_max_results(mut self, max_results: u32) -> Self {
        self.max_results = Some(max_results);
        self
    }
}

impl CompletionContext {
    /// Create a new completion context
    pub fn new() -> Self {
        Self {
            cursor_position: None,
            line_content: None,
            document_content: None,
            language: None,
            metadata: None,
        }
    }

    /// Set cursor position
    pub fn with_cursor_position(mut self, position: u32) -> Self {
        self.cursor_position = Some(position);
        self
    }

    /// Set line content
    pub fn with_line_content(mut self, content: impl Into<String>) -> Self {
        self.line_content = Some(content.into());
        self
    }

    /// Set document content
    pub fn with_document_content(mut self, content: impl Into<String>) -> Self {
        self.document_content = Some(content.into());
        self
    }

    /// Set language
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }
}

impl Default for CompletionContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_value_creation() {
        let cv = CompletionValue::new("test");
        assert_eq!(cv.value, "test");
        assert!(cv.label.is_none());

        let cv = CompletionValue::with_label("test", "Test Label");
        assert_eq!(cv.value, "test");
        assert_eq!(cv.label, Some("Test Label".to_string()));

        let cv = CompletionValue::function("myFunction", "A test function");
        assert_eq!(cv.value, "myFunction");
        assert_eq!(cv.description, Some("A test function".to_string()));
        assert!(matches!(cv.kind, Some(CompletionKind::Function)));
        assert_eq!(cv.priority, Some(1.0));
    }

    #[test]
    fn test_completion_sorting() {
        let mut completion = Completion::new(vec![
            CompletionValue {
                value: "low".to_string(),
                priority: Some(0.5),
                ..Default::default()
            },
            CompletionValue {
                value: "high".to_string(),
                priority: Some(1.0),
                ..Default::default()
            },
            CompletionValue {
                value: "medium".to_string(),
                priority: Some(0.8),
                ..Default::default()
            },
        ]);

        completion.sort_by_priority();
        assert_eq!(completion.values[0].value, "high");
        assert_eq!(completion.values[1].value, "medium");
        assert_eq!(completion.values[2].value, "low");
    }

    #[test]
    fn test_completion_filtering() {
        let completion = Completion::new(vec![
            CompletionValue::new("apple"),
            CompletionValue::new("banana"),
            CompletionValue::new("apricot"),
            CompletionValue::new("cherry"),
        ]);

        let filtered = completion.filter_by_prefix("ap");
        assert_eq!(filtered.values.len(), 2);
        assert!(filtered.values.iter().any(|cv| cv.value == "apple"));
        assert!(filtered.values.iter().any(|cv| cv.value == "apricot"));
    }

    #[test]
    fn test_completion_request_builder() {
        let request = CompleteRequest::new("tools", "myTool")
            .with_context(CompletionContext::new().with_language("rust"))
            .with_filter(CompletionFilter {
                prefix: Some("my".to_string()),
                ..Default::default()
            })
            .with_max_results(10);

        assert_eq!(request.ref_type, "tools");
        assert_eq!(request.ref_name, "myTool");
        assert_eq!(request.context.as_ref().and_then(|c| c.language.as_ref()), Some(&"rust".to_string()));
        assert_eq!(request.max_results, Some(10));
    }
}
