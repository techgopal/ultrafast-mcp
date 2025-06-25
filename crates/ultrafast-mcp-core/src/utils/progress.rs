use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Progress information for long-running operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    /// Unique identifier for this progress
    pub id: String,
    /// Current progress value
    pub current: u64,
    /// Total expected value (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Current status
    pub status: ProgressStatus,
    /// Timestamp of last update (Unix timestamp)
    pub updated_at: u64,
    /// Additional metadata
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Progress status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgressStatus {
    /// Operation is starting
    Starting,
    /// Operation is in progress
    Running,
    /// Operation completed successfully
    Completed,
    /// Operation failed
    Failed,
    /// Operation was cancelled
    Cancelled,
    /// Operation is paused
    Paused,
}

impl Progress {
    /// Create a new progress instance
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            current: 0,
            total: None,
            description: None,
            status: ProgressStatus::Starting,
            updated_at: current_timestamp(),
            metadata: HashMap::new(),
        }
    }

    /// Set the total expected value
    pub fn with_total(mut self, total: u64) -> Self {
        self.total = Some(total);
        self.updated_at = current_timestamp();
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self.updated_at = current_timestamp();
        self
    }

    /// Set the current progress
    pub fn with_current(mut self, current: u64) -> Self {
        self.current = current;
        self.updated_at = current_timestamp();
        self
    }

    /// Set the status
    pub fn with_status(mut self, status: ProgressStatus) -> Self {
        self.status = status;
        self.updated_at = current_timestamp();
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self.updated_at = current_timestamp();
        self
    }

    /// Update the current progress
    pub fn update(&mut self, current: u64) {
        self.current = current;
        self.updated_at = current_timestamp();
    }

    /// Update with description
    pub fn update_with_description(&mut self, current: u64, description: impl Into<String>) {
        self.current = current;
        self.description = Some(description.into());
        self.updated_at = current_timestamp();
    }

    /// Mark as completed
    pub fn complete(&mut self) {
        if let Some(total) = self.total {
            self.current = total;
        }
        self.status = ProgressStatus::Completed;
        self.updated_at = current_timestamp();
    }

    /// Mark as failed
    pub fn fail(&mut self, error: Option<String>) {
        self.status = ProgressStatus::Failed;
        if let Some(error) = error {
            self.metadata.insert("error".to_string(), serde_json::Value::String(error));
        }
        self.updated_at = current_timestamp();
    }

    /// Mark as cancelled
    pub fn cancel(&mut self) {
        self.status = ProgressStatus::Cancelled;
        self.updated_at = current_timestamp();
    }

    /// Get progress percentage (0-100) if total is known
    pub fn percentage(&self) -> Option<f64> {
        self.total.map(|total| {
            if total == 0 {
                100.0
            } else {
                (self.current as f64 / total as f64) * 100.0
            }
        })
    }

    /// Check if the operation is finished
    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            ProgressStatus::Completed | ProgressStatus::Failed | ProgressStatus::Cancelled
        )
    }

    /// Check if the operation is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, ProgressStatus::Running | ProgressStatus::Starting)
    }

    /// Get the age of this progress update
    pub fn age(&self) -> Duration {
        let now = current_timestamp();
        Duration::from_secs(now.saturating_sub(self.updated_at))
    }
}

/// Progress tracker for managing multiple progress instances
#[derive(Debug, Default)]
pub struct ProgressTracker {
    progress_map: HashMap<String, Progress>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Start tracking a new progress
    pub fn start(&mut self, id: impl Into<String>) -> &mut Progress {
        let id = id.into();
        let progress = Progress::new(id.clone()).with_status(ProgressStatus::Running);
        self.progress_map.entry(id.clone()).or_insert(progress)
    }

    /// Get a progress by ID
    pub fn get(&self, id: &str) -> Option<&Progress> {
        self.progress_map.get(id)
    }

    /// Get a mutable progress by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Progress> {
        self.progress_map.get_mut(id)
    }

    /// Update progress
    pub fn update(&mut self, id: &str, current: u64) -> Option<&Progress> {
        if let Some(progress) = self.progress_map.get_mut(id) {
            progress.update(current);
            Some(progress)
        } else {
            None
        }
    }

    /// Complete a progress
    pub fn complete(&mut self, id: &str) -> Option<&Progress> {
        if let Some(progress) = self.progress_map.get_mut(id) {
            progress.complete();
            Some(progress)
        } else {
            None
        }
    }

    /// Fail a progress
    pub fn fail(&mut self, id: &str, error: Option<String>) -> Option<&Progress> {
        if let Some(progress) = self.progress_map.get_mut(id) {
            progress.fail(error);
            Some(progress)
        } else {
            None
        }
    }

    /// Cancel a progress
    pub fn cancel(&mut self, id: &str) -> Option<&Progress> {
        if let Some(progress) = self.progress_map.get_mut(id) {
            progress.cancel();
            Some(progress)
        } else {
            None
        }
    }

    /// Remove a progress
    pub fn remove(&mut self, id: &str) -> Option<Progress> {
        self.progress_map.remove(id)
    }

    /// Get all progress instances
    pub fn all(&self) -> impl Iterator<Item = &Progress> {
        self.progress_map.values()
    }

    /// Get all active progress instances
    pub fn active(&self) -> impl Iterator<Item = &Progress> {
        self.progress_map.values().filter(|p| p.is_active())
    }

    /// Get all finished progress instances
    pub fn finished(&self) -> impl Iterator<Item = &Progress> {
        self.progress_map.values().filter(|p| p.is_finished())
    }

    /// Clean up old finished progress instances
    pub fn cleanup_finished(&mut self, max_age: Duration) {
        let cutoff = current_timestamp() - max_age.as_secs();
        self.progress_map.retain(|_, progress| {
            !progress.is_finished() || progress.updated_at > cutoff
        });
    }

    /// Get the number of tracked progress instances
    pub fn len(&self) -> usize {
        self.progress_map.len()
    }

    /// Check if the tracker is empty
    pub fn is_empty(&self) -> bool {
        self.progress_map.is_empty()
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Progress notification for MCP protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressNotification {
    /// Progress information
    #[serde(flatten)]
    pub progress: Progress,
}

impl ProgressNotification {
    /// Create a new progress notification
    pub fn new(progress: Progress) -> Self {
        Self { progress }
    }
}

impl From<Progress> for ProgressNotification {
    fn from(progress: Progress) -> Self {
        Self::new(progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_creation() {
        let progress = Progress::new("test")
            .with_total(100)
            .with_description("Test operation")
            .with_current(25);

        assert_eq!(progress.id, "test");
        assert_eq!(progress.current, 25);
        assert_eq!(progress.total, Some(100));
        assert_eq!(progress.description, Some("Test operation".to_string()));
        assert_eq!(progress.status, ProgressStatus::Starting);
    }

    #[test]
    fn test_progress_percentage() {
        let mut progress = Progress::new("test").with_total(100);
        progress.update(25);
        assert_eq!(progress.percentage(), Some(25.0));

        progress.update(50);
        assert_eq!(progress.percentage(), Some(50.0));

        let progress_no_total = Progress::new("test");
        assert_eq!(progress_no_total.percentage(), None);
    }

    #[test]
    fn test_progress_status() {
        let mut progress = Progress::new("test").with_status(ProgressStatus::Running);
        assert!(progress.is_active());
        assert!(!progress.is_finished());

        progress.complete();
        assert!(progress.is_finished());
        assert!(!progress.is_active());
        assert_eq!(progress.status, ProgressStatus::Completed);
    }

    #[test]
    fn test_progress_tracker() {
        let mut tracker = ProgressTracker::new();
        
        // Start tracking
        let progress = tracker.start("test1");
        progress.update(50);
        
        // Check active
        assert_eq!(tracker.active().count(), 1);
        assert_eq!(tracker.finished().count(), 0);
        
        // Complete
        tracker.complete("test1");
        assert_eq!(tracker.active().count(), 0);
        assert_eq!(tracker.finished().count(), 1);
        
        // Test update
        tracker.start("test2");
        let updated = tracker.update("test2", 75);
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().current, 75);
    }

    #[test]
    fn test_progress_cleanup() {
        let mut tracker = ProgressTracker::new();
        
        // Add some progress
        tracker.start("test1");
        tracker.complete("test1");
        
        // Cleanup should remove finished progress
        tracker.cleanup_finished(Duration::from_secs(0));
        assert_eq!(tracker.len(), 0);
    }
}
