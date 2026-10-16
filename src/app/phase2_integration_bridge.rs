//! Phase 2 Integration Bridge
//!
//! Bridges Phase 2 components (BackgroundFileLoader, ProgressModal, etc.)
//! with the existing app message loop without modifying the main Message enum
//! extensively.
//!
//! This creates a cleaner separation and makes it easier to test Phase 2
//! independently of the main app structure.

use std::path::PathBuf;
use crossbeam_channel::Receiver;
use std::time::Instant;
use codec::CadDocument;

use crate::io::background_loader::{BackgroundFileLoader, LoaderEvent};
use crate::scene::streaming::StreamingEntityBatcher;
use crate::scene::progressive_tessellation::ProgressiveTessellator;
use crate::ui::loading_progress::{LoadingProgressModal, LoadingPhase};

/// Phase 2 Integration State - encapsulates all Phase 2 components
/// Add this as a field in your App struct:
/// ```ignore
/// pub struct App {
///     // ... existing fields ...
///     #[cfg(not(target_arch = "wasm32"))]
///     phase2: Phase2IntegrationBridge,
/// }
/// ```
pub struct Phase2IntegrationBridge {
    /// Active background loader (if any)
    pub loader: Option<BackgroundFileLoader>,

    /// Event receiver from background loader
    pub event_rx: Option<Receiver<LoaderEvent>>,

    /// Progress modal state
    pub modal: LoadingProgressModal,

    /// Entity batcher for reducing geometry bumps
    pub batcher: StreamingEntityBatcher,

    /// Progressive tessellator
    pub tessellator: ProgressiveTessellator,

    /// Whether we're currently loading
    is_loading: bool,

    /// Loaded document ready to be added to scene
    loaded_document: Option<CadDocument>,

    /// Timestamp of last poll (for monitoring)
    last_poll: Option<Instant>,
}

impl Phase2IntegrationBridge {
    /// Create new Phase 2 integration
    pub fn new() -> Self {
        Self {
            loader: None,
            event_rx: None,
            modal: LoadingProgressModal::new(),
            batcher: StreamingEntityBatcher::new(1000),
            tessellator: ProgressiveTessellator::new(4, Default::default()),
            is_loading: false,
            loaded_document: None,
            last_poll: None,
        }
    }

    /// Start loading a file (call this when user picks a file)
    /// Replaces the synchronous open_path_with_phase call
    pub fn start_load(&mut self, path: PathBuf) -> bool {
        if self.is_loading {
            eprintln!("Already loading a file");
            return false;
        }

        // Spawn background loader
        let (loader, rx) = BackgroundFileLoader::start_load(path.clone());

        self.loader = Some(loader);
        self.event_rx = Some(rx);
        self.is_loading = true;

        // Show progress modal
        self.modal.show();
        self.modal.set_progress(0, LoadingPhase::Reading);

        true
    }

    /// Poll for events from background loader
    /// Call this once per frame (in your Tick message handler)
    /// Returns true if loading is complete
    pub fn poll_events(&mut self) -> bool {
        self.last_poll = Some(Instant::now());

        if let Some(rx) = &self.event_rx {
            // Non-blocking receive all available events
            while let Ok(event) = rx.try_recv() {
                match event {
                    LoaderEvent::ParsingProgress { loaded, total } => {
                        self.modal.set_entity_progress(loaded, total);
                        self.modal.set_progress(
                            (loaded * 100 / total).min(100) as u32,
                            LoadingPhase::Parsing,
                        );
                    }

                    LoaderEvent::EntitiesBatch(entities) => {
                        // Add entities to batcher
                        for (handle, entity_type) in entities {
                            if self.batcher.add_entity(handle, entity_type) {
                                // Batch is full - would call scene.add_entities() here
                                // For now, just track that it happened
                            }
                        }

                        // Update progress
                        let total = self.modal.stats.entities_total;
                        if total > 0 {
                            let loaded = self.modal.stats.entities_loaded;
                            self.modal.set_entity_progress(loaded, total);
                        }
                    }

                    LoaderEvent::TessellationProgress { done, total } => {
                        self.modal.set_progress(
                            (done * 100 / total).min(100) as u32,
                            LoadingPhase::Tessellating,
                        );
                        self.modal.set_entity_progress(done, total);
                    }

                    LoaderEvent::Complete(doc) => {
                        // Mark complete
                        self.modal.complete();

                        // Store loaded document for scene integration
                        self.loaded_document = Some(doc);

                        // Flush any remaining batch
                        let remaining = self.batcher.flush();
                        if !remaining.is_empty() {
                            // Would call: scene.add_entities(remaining)
                        }

                        // Hide modal after a brief moment (next tick will show "Complete" then hide)
                        // For now, hide immediately so user sees completion message in logs/status
                        self.modal.hide();

                        // Reset state
                        self.is_loading = false;
                        self.loader = None;
                        self.event_rx = None;

                        return true; // Signal complete
                    }

                    LoaderEvent::Error(err) => {
                        eprintln!("Loading error: {}", err);
                        self.modal.hide();
                        self.is_loading = false;
                        self.loader = None;
                        self.event_rx = None;
                        return true; // Signal error (stop polling)
                    }
                }
            }
        }

        false // Still loading
    }

    /// Cancel ongoing load
    pub fn cancel(&mut self) {
        self.loader = None;
        self.event_rx = None;
        self.modal.cancel();
        self.batcher = StreamingEntityBatcher::new(1000);
        self.is_loading = false;
    }

    /// Check if currently loading
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    /// Get progress text for UI
    pub fn progress_text(&self) -> String {
        if !self.modal.visible {
            return String::new();
        }

        format!(
            "{} - {} ({}MB)",
            self.modal
                .stats
                .phase
                .map(|p| p.label())
                .unwrap_or("..."),
            self.modal.progress_text(),
            self.modal.stats.memory_mb
        )
    }

    /// Get ETA text for UI
    pub fn eta_text(&self) -> String {
        self.modal.eta_text()
    }

    /// Get memory text for UI
    pub fn memory_text(&self) -> String {
        self.modal.memory_text()
    }

    /// Get phase info
    pub fn current_phase(&self) -> Option<LoadingPhase> {
        self.modal.stats.phase
    }

    /// Get progress percentage
    pub fn progress_percent(&self) -> u32 {
        self.modal.stats.progress_percent
    }

    /// Check if modal should be visible
    pub fn modal_visible(&self) -> bool {
        self.modal.visible
    }

    /// Get loaded document if available (takes ownership, clears internal state)
    pub fn take_loaded_document(&mut self) -> Option<CadDocument> {
        self.loaded_document.take()
    }
}

impl Default for Phase2IntegrationBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = Phase2IntegrationBridge::new();
        assert!(!bridge.is_loading());
        assert!(!bridge.modal_visible());
    }

    #[test]
    fn test_modal_visibility() {
        let mut bridge = Phase2IntegrationBridge::new();

        // Not visible initially
        assert!(!bridge.modal_visible());

        // Show on load
        bridge.modal.show();
        assert!(bridge.modal_visible());

        // Hide on cancel
        bridge.cancel();
        assert!(!bridge.modal_visible());
    }

    #[test]
    fn test_progress_updates() {
        let mut bridge = Phase2IntegrationBridge::new();
        bridge.modal.show();

        bridge.modal.set_progress(50, LoadingPhase::Parsing);
        assert_eq!(bridge.progress_percent(), 50);

        bridge.modal.complete();
        assert_eq!(bridge.progress_percent(), 100);
    }

    #[test]
    fn test_text_getters() {
        let mut bridge = Phase2IntegrationBridge::new();
        bridge.modal.show();
        bridge.modal.set_progress(0, LoadingPhase::Reading);

        let progress = bridge.progress_text();
        assert!(!progress.is_empty());

        let eta = bridge.eta_text();
        assert!(!eta.is_empty());
    }
}
