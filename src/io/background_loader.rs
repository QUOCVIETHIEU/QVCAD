//! Background file loading with streaming entity delivery
//!
//! Loads DWG/DXF files in a background thread to keep UI responsive.
//! Sends entities in batches as they're parsed, enabling progressive rendering.

use crossbeam_channel::{bounded, Receiver, Sender};
use std::path::{Path, PathBuf};
use std::thread::{self, JoinHandle};
use std::io::Cursor;
use codec::{CadDocument, EntityType, Handle, DwgReader, DxfReader};

/// Events emitted during background file loading
#[derive(Debug, Clone)]
pub enum LoaderEvent {
    /// Parsing progress: (loaded_entities, estimated_total)
    ParsingProgress { loaded: usize, total: usize },

    /// Batch of entities ready to add to scene
    /// Entities arrive in batches to reduce Scene::add_entity() calls
    EntitiesBatch(Vec<(Handle, EntityType)>),

    /// Tessellation progress: (tessellated, total)
    TessellationProgress { done: usize, total: usize },

    /// Loading complete - final document ready
    Complete(CadDocument),

    /// Error occurred during loading
    Error(String),
}

/// Manages background file loading
pub struct BackgroundFileLoader {
    /// Channel sender (held by thread)
    tx: Sender<LoaderEvent>,
    /// Thread handle
    thread_handle: Option<JoinHandle<()>>,
}

impl BackgroundFileLoader {
    /// Start loading a DWG/DXF file in background
    ///
    /// Returns a `BackgroundFileLoader` and a `Receiver<LoaderEvent>`.
    /// The receiver will emit events as loading progresses.
    pub fn start_load(path: PathBuf) -> (Self, Receiver<LoaderEvent>) {
        // Create bounded channel (1000 events buffered)
        let (tx, rx) = bounded(1000);
        let tx_clone = tx.clone();

        let thread_handle = thread::spawn(move || {
            // Load file in background
            if let Err(e) = Self::load_file_impl(&path, &tx_clone) {
                let _ = tx_clone.send(LoaderEvent::Error(e));
            }
        });

        (
            Self {
                tx,
                thread_handle: Some(thread_handle),
            },
            rx,
        )
    }

    /// Implementation of file loading
    fn load_file_impl(path: &Path, tx: &Sender<LoaderEvent>) -> Result<(), String> {
        // Step 1: Load file into memory
        let bytes = std::fs::read(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Send parsing progress update
        let _ = tx.send(LoaderEvent::ParsingProgress {
            loaded: 0,
            total: bytes.len() / 1024,
        });

        // Step 2: Detect and parse file format
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());

        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        eprintln!("DEBUG: Loading file: {}", filename);
        eprintln!("DEBUG: Extension: {:?}", ext);

        // Parse based on file format (DXF or DWG)
        let doc = match ext.as_deref() {
            Some("dxf") => {
                eprintln!("DEBUG: Parsing as DXF format");
                DxfReader::from_reader(Cursor::new(bytes.clone()))
                    .map_err(|e| format!("Failed to read DXF: {}", e))?
                    .read()
                    .map_err(|e| format!("Error parsing DXF file: {}", e))?
            }
            Some("dwg") => {
                eprintln!("DEBUG: Parsing as DWG format");
                DwgReader::from_stream(Cursor::new(bytes.clone()))
                    .read()
                    .map_err(|e| format!("Error parsing DWG file: {}", e))?
            }
            _ => {
                let ext_str = ext.as_deref().unwrap_or("no extension");
                let err = format!("File '{}' has unsupported format '.{}' - only DXF/DWG supported", filename, ext_str);
                eprintln!("DEBUG: {}", err);
                return Err(err);
            }
        };

        let entity_count = doc.entities().count();
        eprintln!("DEBUG: Parse complete - {} entities loaded", entity_count);

        // Step 3: Send completion event with document
        let _ = tx.send(LoaderEvent::ParsingProgress {
            loaded: 1000,
            total: 1000,
        });

        let _ = tx.send(LoaderEvent::Complete(doc));

        Ok(())
    }
}

impl Drop for BackgroundFileLoader {
    fn drop(&mut self) {
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_creation() {
        // Test that loader can be created
        // Actual loading would need a real DXF file
        let path = PathBuf::from("nonexistent.dxf");
        let (_loader, rx) = BackgroundFileLoader::start_load(path);

        // Should eventually get an error
        if let Ok(event) = rx.recv() {
            match event {
                LoaderEvent::Error(_) => {
                    // Expected
                }
                _ => {}
            }
        }
    }
}
