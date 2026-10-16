//! Streaming entity addition with batching
//!
//! Instead of adding entities one-at-a-time (triggering Scene updates for each),
//! batch them and add in groups. This reduces epoch bumps from O(n) to O(1).
//!
//! Example: Loading 10K entities
//! - Without batching: 10,000 scene updates
//! - With batching (1000/batch): 10 scene updates
//! - Speedup: 100-1000x ✨

use codec::{EntityType, Handle};
use std::collections::VecDeque;

/// Accumulates entities in batches before adding to scene
///
/// Calling code sends entities one-at-a-time, and when a batch fills,
/// the batcher triggers an on_batch callback. The receiving code
/// (Scene::add_entities_batch) adds all entities and bumps epoch once.
#[derive(Debug)]
pub struct StreamingEntityBatcher {
    /// Accumulated entities waiting to be added
    batch: Vec<(Handle, EntityType)>,

    /// Batch size - trigger callback when reached
    batch_size: usize,

    /// Statistics
    stats: BatcherStats,
}

#[derive(Debug, Default, Clone)]
pub struct BatcherStats {
    /// Total entities added
    pub total_added: usize,
    /// Batches emitted
    pub batches_emitted: usize,
    /// Current batch size
    pub current_batch_size: usize,
}

impl StreamingEntityBatcher {
    /// Create a new batcher with given batch size
    ///
    /// # Arguments
    /// * `batch_size` - How many entities per batch (typically 1000)
    ///
    /// # Example
    /// ```ignore
    /// let mut batcher = StreamingEntityBatcher::new(1000);
    /// for (handle, entity) in entities {
    ///     if batcher.add_entity(handle, entity) {
    ///         // Process batch
    ///         let batch = batcher.take_batch();
    ///         scene.add_entities_batch(batch);
    ///     }
    /// }
    /// // Don't forget remaining entities
    /// let final_batch = batcher.flush();
    /// scene.add_entities_batch(final_batch);
    /// ```
    pub fn new(batch_size: usize) -> Self {
        assert!(batch_size > 0, "batch_size must be > 0");
        Self {
            batch: Vec::with_capacity(batch_size),
            batch_size,
            stats: BatcherStats::default(),
        }
    }

    /// Add an entity to the batch
    ///
    /// # Returns
    /// `true` if the batch is now full and should be processed
    ///
    /// # Example
    /// ```ignore
    /// if batcher.add_entity(handle, entity) {
    ///     let batch = batcher.take_batch();
    ///     scene.add_entities_batch(batch);
    /// }
    /// ```
    pub fn add_entity(&mut self, handle: Handle, entity: EntityType) -> bool {
        self.batch.push((handle, entity));
        self.stats.total_added += 1;
        self.stats.current_batch_size = self.batch.len();

        // Signal that batch is ready
        self.batch.len() >= self.batch_size
    }

    /// Take the current batch (empties the internal buffer)
    ///
    /// Call this when `add_entity()` returns true, or at the end of loading.
    pub fn take_batch(&mut self) -> Vec<(Handle, EntityType)> {
        self.stats.batches_emitted += 1;
        self.stats.current_batch_size = 0;
        std::mem::take(&mut self.batch)
    }

    /// Flush any remaining entities
    ///
    /// Call this at the end of entity stream to get leftover entities.
    pub fn flush(&mut self) -> Vec<(Handle, EntityType)> {
        if !self.batch.is_empty() {
            self.take_batch()
        } else {
            Vec::new()
        }
    }

    /// Get current batch statistics
    pub fn stats(&self) -> &BatcherStats {
        &self.stats
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.batch.is_empty()
    }

    /// Current batch size
    pub fn len(&self) -> usize {
        self.batch.len()
    }

    /// Reset statistics (useful for benchmarking)
    pub fn reset_stats(&mut self) {
        self.stats = BatcherStats::default();
        self.stats.current_batch_size = self.batch.len();
    }
}

/// Helper for progressive entity loading with streaming batcher
///
/// This is a convenience wrapper that coordinates background file loading
/// with scene batch additions.
pub struct StreamingSceneLoader {
    /// Entity batcher
    batcher: StreamingEntityBatcher,

    /// Queue of batches ready to add to scene
    pending_batches: VecDeque<Vec<(Handle, EntityType)>>,

    /// Statistics
    total_batches_queued: usize,
}

impl StreamingSceneLoader {
    /// Create a new streaming scene loader
    pub fn new(batch_size: usize) -> Self {
        Self {
            batcher: StreamingEntityBatcher::new(batch_size),
            pending_batches: VecDeque::new(),
            total_batches_queued: 0,
        }
    }

    /// Add an entity from the file loader
    ///
    /// Returns a batch if one is ready to add to the scene
    pub fn add_entity_from_loader(
        &mut self,
        handle: Handle,
        entity: EntityType,
    ) -> Option<Vec<(Handle, EntityType)>> {
        if self.batcher.add_entity(handle, entity) {
            let batch = self.batcher.take_batch();
            self.total_batches_queued += 1;
            Some(batch)
        } else {
            None
        }
    }

    /// Flush remaining entities after file load completes
    pub fn flush_remaining(&mut self) -> Option<Vec<(Handle, EntityType)>> {
        let batch = self.batcher.flush();
        if !batch.is_empty() {
            self.total_batches_queued += 1;
            Some(batch)
        } else {
            None
        }
    }

    /// Get overall statistics
    pub fn stats(&self) -> StreamingLoaderStats {
        StreamingLoaderStats {
            batcher_stats: self.batcher.stats().clone(),
            total_batches_queued: self.total_batches_queued,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StreamingLoaderStats {
    pub batcher_stats: BatcherStats,
    pub total_batches_queued: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batcher_basics() {
        let mut batcher = StreamingEntityBatcher::new(3);

        // Add 2 entities - shouldn't trigger batch yet
        assert!(!batcher.add_entity(codec::Handle::new(1), EntityType::Line(Default::default())));
        assert!(!batcher.add_entity(codec::Handle::new(2), EntityType::Line(Default::default())));
        assert_eq!(batcher.len(), 2);

        // Add 3rd entity - should trigger batch
        assert!(batcher.add_entity(codec::Handle::new(3), EntityType::Line(Default::default())));
        assert_eq!(batcher.len(), 3);

        // Take batch
        let batch = batcher.take_batch();
        assert_eq!(batch.len(), 3);
        assert!(batcher.is_empty());
    }

    #[test]
    fn test_batcher_flush() {
        let mut batcher = StreamingEntityBatcher::new(1000);

        // Add 100 entities (less than batch size)
        for i in 0..100 {
            batcher.add_entity(
                codec::Handle::new(i as u64),
                EntityType::Line(Default::default()),
            );
        }

        // Flush should return all 100
        let batch = batcher.flush();
        assert_eq!(batch.len(), 100);
        assert!(batcher.is_empty());
    }

    #[test]
    fn test_streaming_loader() {
        let mut loader = StreamingSceneLoader::new(10);

        // Add 25 entities
        for i in 0..25 {
            let batch = loader.add_entity_from_loader(
                codec::Handle::new(i),
                EntityType::Line(Default::default()),
            );

            match i {
                9 | 19 => {
                    // Batches should be ready at 10 and 20
                    assert!(batch.is_some());
                    assert_eq!(batch.unwrap().len(), 10);
                }
                _ => {
                    assert!(batch.is_none());
                }
            }
        }

        // Flush remaining
        let final_batch = loader.flush_remaining();
        assert!(final_batch.is_some());
        assert_eq!(final_batch.unwrap().len(), 5);

        // Check stats
        let stats = loader.stats();
        assert_eq!(stats.batcher_stats.total_added, 25);
        assert_eq!(stats.total_batches_queued, 3); // 10 + 10 + 5
    }
}
