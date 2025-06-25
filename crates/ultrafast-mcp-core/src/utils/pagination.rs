use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Cursor-based pagination for MCP list operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cursor {
    /// Opaque cursor value
    pub value: String,
    /// Optional metadata for the cursor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl Cursor {
    /// Create a new cursor with a value
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            metadata: None,
        }
    }

    /// Create a cursor with metadata
    pub fn with_metadata(value: impl Into<String>, metadata: HashMap<String, serde_json::Value>) -> Self {
        Self {
            value: value.into(),
            metadata: Some(metadata),
        }
    }

    /// Get the cursor value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Get cursor metadata
    pub fn metadata(&self) -> Option<&HashMap<String, serde_json::Value>> {
        self.metadata.as_ref()
    }
}

/// Pagination parameters for list requests
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaginationParams {
    /// Maximum number of items to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
}

impl PaginationParams {
    /// Create new pagination parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the limit
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the cursor
    pub fn with_cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = Some(cursor);
        self
    }

    /// Get the effective limit, using a default if not set
    pub fn effective_limit(&self, default: u32) -> u32 {
        self.limit.unwrap_or(default)
    }
}

/// Pagination information in responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    /// Total number of items (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    /// Cursor for the next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<Cursor>,
    /// Whether there are more items
    pub has_more: bool,
}

impl PaginationInfo {
    /// Create pagination info indicating no more items
    pub fn no_more() -> Self {
        Self {
            total: None,
            next_cursor: None,
            has_more: false,
        }
    }

    /// Create pagination info with a next cursor
    pub fn with_next(cursor: Cursor) -> Self {
        Self {
            total: None,
            next_cursor: Some(cursor),
            has_more: true,
        }
    }

    /// Create pagination info with total count
    pub fn with_total(total: u64, next_cursor: Option<Cursor>) -> Self {
        Self {
            total: Some(total),
            next_cursor: next_cursor.clone(),
            has_more: next_cursor.is_some(),
        }
    }
}

/// A paginated list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedList<T> {
    /// The items in this page
    pub items: Vec<T>,
    /// Pagination information
    #[serde(flatten)]
    pub pagination: PaginationInfo,
}

impl<T> PaginatedList<T> {
    /// Create a new paginated list
    pub fn new(items: Vec<T>, pagination: PaginationInfo) -> Self {
        Self { items, pagination }
    }

    /// Create a complete list (no pagination)
    pub fn complete(items: Vec<T>) -> Self {
        Self {
            items,
            pagination: PaginationInfo::no_more(),
        }
    }

    /// Create a paginated list with next cursor
    pub fn with_next(items: Vec<T>, next_cursor: Cursor) -> Self {
        Self {
            items,
            pagination: PaginationInfo::with_next(next_cursor),
        }
    }

    /// Get the number of items in this page
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if this page is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Check if there are more pages
    pub fn has_more(&self) -> bool {
        self.pagination.has_more
    }

    /// Get the next cursor if available
    pub fn next_cursor(&self) -> Option<&Cursor> {
        self.pagination.next_cursor.as_ref()
    }
}

impl<T> IntoIterator for PaginatedList<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<T> AsRef<[T]> for PaginatedList<T> {
    fn as_ref(&self) -> &[T] {
        &self.items
    }
}

/// Helper for building paginated responses
pub struct PaginationBuilder<T> {
    items: Vec<T>,
    limit: Option<u32>,
    total: Option<u64>,
}

impl<T> PaginationBuilder<T> {
    /// Create a new pagination builder
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            limit: None,
            total: None,
        }
    }

    /// Set the page limit
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the total count
    pub fn with_total(mut self, total: u64) -> Self {
        self.total = Some(total);
        self
    }

    /// Build the paginated list
    pub fn build<F>(self, next_cursor_fn: F) -> PaginatedList<T>
    where
        F: FnOnce(&[T]) -> Option<Cursor>,
    {
        let has_more = if let Some(limit) = self.limit {
            self.items.len() > limit as usize
        } else {
            false
        };

        let items = if let Some(limit) = self.limit {
            self.items.into_iter().take(limit as usize).collect()
        } else {
            self.items
        };

        let next_cursor = if has_more {
            next_cursor_fn(&items)
        } else {
            None
        };

        let pagination = if let Some(total) = self.total {
            PaginationInfo::with_total(total, next_cursor)
        } else if let Some(cursor) = next_cursor {
            PaginationInfo::with_next(cursor)
        } else {
            PaginationInfo::no_more()
        };

        PaginatedList::new(items, pagination)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_creation() {
        let cursor = Cursor::new("abc123");
        assert_eq!(cursor.value(), "abc123");
        assert!(cursor.metadata().is_none());

        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), serde_json::Value::String("value".to_string()));
        let cursor = Cursor::with_metadata("def456", metadata.clone());
        assert_eq!(cursor.value(), "def456");
        assert_eq!(cursor.metadata(), Some(&metadata));
    }

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams::new()
            .with_limit(10)
            .with_cursor(Cursor::new("cursor123"));
        
        assert_eq!(params.limit, Some(10));
        assert_eq!(params.cursor.as_ref().unwrap().value(), "cursor123");
        assert_eq!(params.effective_limit(50), 10);
        
        let params = PaginationParams::new();
        assert_eq!(params.effective_limit(50), 50);
    }

    #[test]
    fn test_paginated_list() {
        let items = vec![1, 2, 3, 4, 5];
        let list = PaginatedList::complete(items.clone());
        
        assert_eq!(list.len(), 5);
        assert!(!list.has_more());
        assert!(list.next_cursor().is_none());
        
        let list = PaginatedList::with_next(items, Cursor::new("next"));
        assert!(list.has_more());
        assert_eq!(list.next_cursor().unwrap().value(), "next");
    }

    #[test]
    fn test_pagination_builder() {
        let items = vec![1, 2, 3, 4, 5];
        let list = PaginationBuilder::new(items)
            .with_limit(3)
            .build(|items| {
                if !items.is_empty() {
                    Some(Cursor::new(format!("after_{}", items.last().unwrap())))
                } else {
                    None
                }
            });
        
        assert_eq!(list.len(), 3);
        assert!(list.has_more());
        assert_eq!(list.next_cursor().unwrap().value(), "after_3");
    }
}
