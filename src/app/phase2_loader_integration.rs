//! Phase 2 Integration: Background file loading with progress modal
//!
//! This module demonstrates how to integrate the three optimization components:
//! 1. BackgroundFileLoader (background I/O)
//! 2. StreamingEntityBatcher (entity batching)
//! 3. ProgressiveTessellator (coarse-first display)
//! 4. LoadingProgressModal (UI feedback)
//!
//! # Integration Points
//!
//! When user selects file to open:
//! 1. Spawn BackgroundFileLoader on background thread
//! 2. Get Receiver<LoaderEvent> channel
//! 3. Show LoadingProgressModal
//! 4. Poll receiver each frame for events
//! 5. Update modal with progress
//! 6. When complete, add to scene

use std::path::PathBuf;
use crossbeam_channel::Receiver;

use crate::io::background_loader::{BackgroundFileLoader, LoaderEvent};
use crate::scene::streaming::StreamingEntityBatcher;
use crate::scene::progressive_tessellation::ProgressiveTessellator;
use crate::ui::loading_progress::{LoadingProgressModal, LoadingPhase};

/// Orchestrates the entire file loading process
/// This would typically be embedded in the main App struct
pub struct Phase2FileLoadingSystem {
    /// Active background loader (if any)
    pub loader: Option<BackgroundFileLoader>,
    /// Event receiver from loader
    pub event_rx: Option<Receiver<LoaderEvent>>,
    /// Progress modal state
    pub modal: LoadingProgressModal,
    /// Entity batcher for reducing geometry bumps
    pub batcher: StreamingEntityBatcher,
    /// Progressive tessellator for immediate display
    pub tessellator: ProgressiveTessellator,
}

impl Phase2FileLoadingSystem {
    /// Create new loading system
    pub fn new() -> Self {
        Self {
            loader: None,
            event_rx: None,
            modal: LoadingProgressModal::new(),
            batcher: StreamingEntityBatcher::new(1000),
            tessellator: ProgressiveTessellator::new(4, Default::default()),
        }
    }

    /// Start loading a file
    /// Call this when user picks a file (e.g., in Message::OpenPathPicked handler)
    pub fn start_load(&mut self, path: PathBuf) {
        // Spawn background loader
        let (loader, rx) = BackgroundFileLoader::start_load(path.clone());

        // Store loader and receiver
        self.loader = Some(loader);
        self.event_rx = Some(rx);

        // Show progress modal
        self.modal.show();
        self.modal.set_progress(0, LoadingPhase::Reading);
    }

    /// Poll for events from background loader
    /// Call this once per frame (e.g., in Message::Tick handler)
    /// Returns true if loading is complete
    pub fn poll_events(&mut self) -> bool {
        if let Some(rx) = &self.event_rx {
            // Non-blocking receive
            while let Ok(event) = rx.try_recv() {
                match event {
                    LoaderEvent::ParsingProgress { loaded, total } => {
                        self.modal.set_entity_progress(loaded, total);
                        self.modal
                            .set_progress((loaded * 100 / total).min(100) as u32, LoadingPhase::Parsing);
                    }

                    LoaderEvent::EntitiesBatch(entities) => {
                        // Add to batcher (not directly to scene!)
                        for (handle, entity_type) in entities {
                            // Check if batch should be flushed
                            if self.batcher.add_entity(handle, entity_type) {
                                // Batch is full, would call:
                                // scene.add_entities(batch)
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
                        self.modal.set_progress((done * 100 / total).min(100) as u32, LoadingPhase::Tessellating);
                        self.modal.set_entity_progress(done, total);
                    }

                    LoaderEvent::Complete(_doc) => {
                        // Mark complete
                        self.modal.complete();

                        // Flush any remaining batch
                        let remaining = self.batcher.flush();
                        if !remaining.is_empty() {
                            // Would call: scene.add_entities(remaining)
                        }

                        return true; // Signal complete
                    }

                    LoaderEvent::Error(err) => {
                        eprintln!("Loading error: {}", err);
                        self.modal.hide();
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
    }

    /// Check if loading is active
    pub fn is_loading(&self) -> bool {
        self.loader.is_some() && self.event_rx.is_some()
    }

    /// Get progress text for display
    pub fn progress_text(&self) -> String {
        if !self.modal.visible {
            return String::new();
        }

        format!(
            "{} - {} ({} MB)",
            self.modal
                .stats
                .phase
                .map(|p| p.label())
                .unwrap_or("..."),
            self.modal.progress_text(),
            self.modal.stats.memory_mb
        )
    }

    /// Get ETA text for display
    pub fn eta_text(&self) -> String {
        self.modal.eta_text()
    }
}

impl Default for Phase2FileLoadingSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Example: How to use in app update loop
///
/// ```ignore
/// pub enum Message {
///     // ... existing messages ...
///     OpenPathPicked(Option<(PathBuf, u64)>),
///     FileLoadingTick, // Sent regularly (e.g., every frame)
///     CancelLoading,
/// }
///
/// pub struct App {
///     // ... existing fields ...
///     file_loading: Phase2FileLoadingSystem,
/// }
///
/// impl App {
///     fn update(&mut self, msg: Message) -> Task<Message> {
///         match msg {
///             Message::OpenPathPicked(Some((path, _size))) => {
///                 // Start loading!
///                 self.file_loading.start_load(path);
///                 Task::none()
///             }
///
///             Message::FileLoadingTick => {
///                 // Poll for progress
///                 let complete = self.file_loading.poll_events();
///                 if complete {
///                     // Handle completion
///                     // - Add entities to scene
///                     // - Close modal
///                     // - Update viewport
///                 }
///                 Task::none()
///             }
///
///             Message::CancelLoading => {
///                 self.file_loading.cancel();
///                 Task::none()
///             }
///
///             _ => Task::none(),
///         }
///     }
/// }
/// ```

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase2_system_creation() {
        let system = Phase2FileLoadingSystem::new();
        assert!(!system.is_loading());
        assert!(!system.modal.visible);
    }

    #[test]
    fn test_modal_visibility() {
        let mut system = Phase2FileLoadingSystem::new();
        let fake_path = PathBuf::from("test.dxf");

        // Note: start_load would spawn a thread
        // For testing, we just check state management
        system.modal.show();
        assert!(system.modal.visible);

        system.modal.hide();
        assert!(!system.modal.visible);
    }

    #[test]
    fn test_progress_updates() {
        let mut system = Phase2FileLoadingSystem::new();
        system.modal.show();

        // Simulate progress
        system
            .modal
            .set_entity_progress(500, 1000);
        assert_eq!(system.modal.stats.entities_loaded, 500);
        assert_eq!(system.modal.stats.progress_percent, 50);

        // Simulate completion
        system.modal.complete();
        assert_eq!(system.modal.stats.progress_percent, 100);
    }

    #[test]
    fn test_cancel() {
        let mut system = Phase2FileLoadingSystem::new();
        system.modal.show();
        assert!(system.modal.visible);

        system.cancel();
        assert!(!system.modal.visible);
        assert!(!system.is_loading());
    }

    #[test]
    fn test_progress_text() {
        let mut system = Phase2FileLoadingSystem::new();
        system.modal.show();
        system
            .modal
            .set_entity_progress(1000, 10000);
        system.modal.set_progress(10, LoadingPhase::Parsing);

        let text = system.progress_text();
        assert!(!text.is_empty());
        assert!(text.contains("Parsing"));
    }
}
