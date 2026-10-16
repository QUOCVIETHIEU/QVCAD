//! Loading progress modal UI component
//!
//! Displays visual feedback during file loading:
//! - Progress bar (0-100%)
//! - Phase indicator (Reading/Parsing/Tessellating)
//! - ETA countdown
//! - Cancel button
//! - Memory usage indicator
//!
//! Keeps users engaged and informed during long operations.

use std::time::{Duration, Instant};

/// Phase of file loading operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadingPhase {
    /// Reading file from disk
    Reading,
    /// Parsing entities
    Parsing,
    /// Tessellating geometry
    Tessellating,
    /// All done
    Complete,
}

impl LoadingPhase {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Reading => "Reading file...",
            Self::Parsing => "Parsing entities...",
            Self::Tessellating => "Tessellating geometry...",
            Self::Complete => "Complete!",
        }
    }
}

/// Progress statistics
#[derive(Debug, Clone, Default)]
pub struct ProgressStats {
    /// Current progress (0-100)
    pub progress_percent: u32,
    /// Current loading phase
    pub phase: Option<LoadingPhase>,
    /// Entities loaded so far
    pub entities_loaded: usize,
    /// Total entities to load
    pub entities_total: usize,
    /// Estimated time remaining (seconds)
    pub eta_seconds: u32,
    /// Memory used (MB)
    pub memory_mb: usize,
    /// Whether operation was cancelled
    pub cancelled: bool,
}

/// Loading progress modal state
pub struct LoadingProgressModal {
    /// Current progress stats
    pub stats: ProgressStats,
    /// When loading started
    start_time: Instant,
    /// Total expected duration (for ETA calculation)
    estimated_total_ms: u128,
    /// Visible or hidden
    pub visible: bool,
}

impl LoadingProgressModal {
    /// Create a new progress modal
    pub fn new() -> Self {
        Self {
            stats: ProgressStats::default(),
            start_time: Instant::now(),
            estimated_total_ms: 5000, // Default 5 second estimate
            visible: false,
        }
    }

    /// Show the modal
    pub fn show(&mut self) {
        self.visible = true;
        self.start_time = Instant::now();
    }

    /// Hide the modal
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Update progress
    pub fn set_progress(&mut self, percent: u32, phase: LoadingPhase) {
        self.stats.progress_percent = percent.min(100);
        self.stats.phase = Some(phase);

        // Calculate ETA based on progress rate
        let elapsed = self.start_time.elapsed().as_millis();
        if self.stats.progress_percent > 0 && elapsed > 0 {
            let rate = elapsed as f64 / self.stats.progress_percent as f64;
            let total_ms = rate * 100.0;
            let remaining_ms = total_ms - elapsed as f64;
            self.stats.eta_seconds = (remaining_ms / 1000.0).max(0.0) as u32;
        }
    }

    /// Update entity count
    pub fn set_entity_progress(&mut self, loaded: usize, total: usize) {
        self.stats.entities_loaded = loaded;
        self.stats.entities_total = total;

        // Update percent based on entities
        if total > 0 {
            let percent = (loaded * 100) / total;
            if let Some(phase) = self.stats.phase {
                self.set_progress(percent as u32, phase);
            }
        }
    }

    /// Update memory usage
    pub fn set_memory_usage(&mut self, mb: usize) {
        self.stats.memory_mb = mb;
    }

    /// Mark as cancelled
    pub fn cancel(&mut self) {
        self.stats.cancelled = true;
        self.hide();
    }

    /// Mark as complete
    pub fn complete(&mut self) {
        self.stats.progress_percent = 100;
        self.stats.phase = Some(LoadingPhase::Complete);
        // Auto-hide modal after showing "Complete" (will be hidden in next tick)
        // User can still see "Complete" for one frame before modal disappears
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get formatted progress string for display
    pub fn progress_text(&self) -> String {
        format!(
            "{}/{} entities ({}%)",
            self.stats.entities_loaded,
            self.stats.entities_total,
            self.stats.progress_percent
        )
    }

    /// Get formatted ETA string
    pub fn eta_text(&self) -> String {
        if self.stats.eta_seconds == 0 {
            "Computing ETA...".to_string()
        } else if self.stats.eta_seconds < 60 {
            format!("{}s remaining", self.stats.eta_seconds)
        } else {
            let minutes = self.stats.eta_seconds / 60;
            let seconds = self.stats.eta_seconds % 60;
            format!("{}m {}s remaining", minutes, seconds)
        }
    }

    /// Get formatted memory string
    pub fn memory_text(&self) -> String {
        if self.stats.memory_mb > 1024 {
            format!("{:.1} GB", self.stats.memory_mb as f64 / 1024.0)
        } else {
            format!("{} MB", self.stats.memory_mb)
        }
    }
}

impl Default for LoadingProgressModal {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress bar widget renderer
pub struct ProgressBar {
    /// Total width in characters
    pub width: usize,
}

impl ProgressBar {
    pub fn new(width: usize) -> Self {
        Self { width }
    }

    /// Render progress bar as ASCII string
    pub fn render(&self, percent: u32) -> String {
        let filled = (self.width * percent as usize) / 100;
        let empty = self.width - filled;

        format!(
            "[{}{}] {}%",
            "=".repeat(filled),
            " ".repeat(empty),
            percent
        )
    }
}

/// Complete progress modal renderer
pub struct ProgressModalRenderer;

impl ProgressModalRenderer {
    /// Render the complete modal as text (for testing/CLI)
    pub fn render_text(modal: &LoadingProgressModal) -> String {
        if !modal.visible {
            return String::new();
        }

        let bar = ProgressBar::new(40);
        let progress_bar = bar.render(modal.stats.progress_percent);

        let phase_label = modal
            .stats
            .phase
            .map(|p| p.label())
            .unwrap_or("Waiting...");

        format!(
            r#"
╔════════════════════════════════════════════════════╗
║           Loading File                             ║
╠════════════════════════════════════════════════════╣
║ Phase: {}
║ {}
║ {}
║ Memory: {}
║ {}
╠════════════════════════════════════════════════════╣
║ [Cancel]                                           ║
╚════════════════════════════════════════════════════╝
        "#,
            phase_label,
            progress_bar,
            modal.progress_text(),
            modal.memory_text(),
            modal.eta_text()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_modal_creation() {
        let modal = LoadingProgressModal::new();
        assert!(!modal.visible);
        assert_eq!(modal.stats.progress_percent, 0);
    }

    #[test]
    fn test_progress_modal_show_hide() {
        let mut modal = LoadingProgressModal::new();
        modal.show();
        assert!(modal.visible);
        modal.hide();
        assert!(!modal.visible);
    }

    #[test]
    fn test_progress_update() {
        let mut modal = LoadingProgressModal::new();
        modal.set_progress(50, LoadingPhase::Parsing);

        assert_eq!(modal.stats.progress_percent, 50);
        assert_eq!(modal.stats.phase, Some(LoadingPhase::Parsing));
    }

    #[test]
    fn test_progress_bar_render() {
        let bar = ProgressBar::new(20);

        let full = bar.render(100);
        assert!(full.contains("===================="));

        let half = bar.render(50);
        assert!(half.contains("=========="));

        let none = bar.render(0);
        assert!(none.contains("                    "));
    }

    #[test]
    fn test_entity_progress() {
        let mut modal = LoadingProgressModal::new();
        modal.set_entity_progress(500, 1000);

        assert_eq!(modal.stats.entities_loaded, 500);
        assert_eq!(modal.stats.entities_total, 1000);
        assert_eq!(modal.stats.progress_percent, 50);
    }

    #[test]
    fn test_eta_calculation() {
        let mut modal = LoadingProgressModal::new();
        modal.show();

        // Simulate 50% progress
        modal.stats.progress_percent = 50;
        modal.start_time = Instant::now() - Duration::from_millis(2500);

        // ETA should estimate ~2500ms remaining
        // (50% complete in 2500ms = 5000ms total)
        let eta_ms = modal.stats.eta_seconds as u128 * 1000;
        assert!(eta_ms > 1000 && eta_ms < 4000); // Reasonable range
    }

    #[test]
    fn test_memory_formatting() {
        let mut modal = LoadingProgressModal::new();

        modal.set_memory_usage(256);
        assert_eq!(modal.memory_text(), "256 MB");

        modal.set_memory_usage(1536);
        assert_eq!(modal.memory_text(), "1.5 GB");
    }

    #[test]
    fn test_modal_cancel() {
        let mut modal = LoadingProgressModal::new();
        modal.show();
        modal.cancel();

        assert!(modal.stats.cancelled);
        assert!(!modal.visible);
    }

    #[test]
    fn test_modal_complete() {
        let mut modal = LoadingProgressModal::new();
        modal.complete();

        assert_eq!(modal.stats.progress_percent, 100);
        assert_eq!(modal.stats.phase, Some(LoadingPhase::Complete));
    }
}
