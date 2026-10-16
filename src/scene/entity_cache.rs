/// Entity State Caching System
///
/// AutoCAD Strategy: Cache expensive computations (tessellation, color, visibility)
/// and only recompute when marked dirty
///
/// Problem it solves:
/// - Hatch upload: 145ms/frame (retessellating 100K+ hatches every frame!)
/// - Wire rendering: 611K wires being recomputed every frame
/// - Text rendering: 372K vertices recomputed every frame
///
/// Solution:
/// - Only retessellate if entity changed (mark dirty)
/// - Cache tessellation result
/// - Use frame counter to track what's stale
/// - Reduce from 100% retessellation to <1% per frame

use std::collections::HashMap;

/// Cache for expensive entity computations
#[derive(Clone, Debug)]
pub struct EntityCache {
    /// Current frame number
    frame: u64,
    /// Entity tessellation cache
    tessellation_cache: HashMap<u64, CachedTessellation>,
    /// Entities marked dirty (need retessellation)
    dirty_entities: std::collections::HashSet<u64>,
    /// Statistics
    pub stats: CacheStats,
}

#[derive(Clone, Debug, Default)]
pub struct CacheStats {
    pub tessellations_total: u64,
    pub tessellations_cached: u64,
    pub tessellations_recomputed: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub dirty_marks: u64,
}

#[derive(Clone, Debug)]
pub struct CachedTessellation {
    /// Cached geometry data (stored as-is, type depends on entity)
    pub geometry: Vec<u8>,
    /// Frame number when tessellated
    pub cached_at_frame: u64,
    /// Whether geometry needs update
    pub needs_update: bool,
}

impl EntityCache {
    pub fn new() -> Self {
        Self {
            frame: 0,
            tessellation_cache: HashMap::new(),
            dirty_entities: std::collections::HashSet::new(),
            stats: CacheStats::default(),
        }
    }

    /// Advance to next frame
    pub fn next_frame(&mut self) {
        self.frame += 1;
        // Keep cache, just track frame (age data)
    }

    /// Check if entity needs retessellation
    pub fn needs_retessellation(&self, entity_handle: u64) -> bool {
        // Needs tessellation if:
        // 1. Not in cache yet
        // 2. Marked dirty
        if self.dirty_entities.contains(&entity_handle) {
            return true;
        }
        if let Some(cached) = self.tessellation_cache.get(&entity_handle) {
            !cached.needs_update
        } else {
            true // Not cached = needs tessellation
        }
    }

    /// Get cached tessellation if available
    pub fn get_cached_tessellation(&self, entity_handle: u64) -> Option<Vec<u8>> {
        if self.dirty_entities.contains(&entity_handle) {
            return None; // Dirty = don't use cache
        }
        self.tessellation_cache.get(&entity_handle).map(|t| t.geometry.clone())
    }

    /// Store tessellation in cache
    pub fn cache_tessellation(&mut self, entity_handle: u64, geometry: Vec<u8>) {
        self.tessellation_cache.insert(
            entity_handle,
            CachedTessellation {
                geometry,
                cached_at_frame: self.frame,
                needs_update: false,
            },
        );
        self.dirty_entities.remove(&entity_handle);
        self.stats.tessellations_recomputed += 1;
    }

    /// Mark entity as dirty (needs retessellation)
    pub fn mark_dirty(&mut self, entity_handle: u64) {
        self.dirty_entities.insert(entity_handle);
        if let Some(cached) = self.tessellation_cache.get_mut(&entity_handle) {
            cached.needs_update = true;
        }
        self.stats.dirty_marks += 1;
    }

    /// Clear cache (e.g., when document changes)
    pub fn clear(&mut self) {
        self.tessellation_cache.clear();
        self.dirty_entities.clear();
    }

    /// Get number of cached entities
    pub fn cached_count(&self) -> usize {
        self.tessellation_cache.len()
    }

    /// Get number of dirty entities
    pub fn dirty_count(&self) -> usize {
        self.dirty_entities.len()
    }

    /// Estimate memory usage
    pub fn memory_usage_bytes(&self) -> usize {
        self.tessellation_cache.values()
            .map(|t| t.geometry.len())
            .sum()
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            tessellations_total: self.stats.tessellations_total,
            tessellations_cached: self.cached_count() as u64,
            tessellations_recomputed: self.stats.tessellations_recomputed,
            cache_hits: self.stats.cache_hits,
            cache_misses: self.stats.cache_misses,
            dirty_marks: self.stats.dirty_marks,
        }
    }

    /// Age out old cache entries (optional - remove entries not used in N frames)
    pub fn age_out_old(&mut self, max_age_frames: u64) {
        let current = self.frame;
        self.tessellation_cache.retain(|_, cached| {
            current - cached.cached_at_frame <= max_age_frames
        });
    }
}

/// Layer change tracking (marks all entities on layer as dirty)
pub struct LayerChangeTracker {
    /// Last frame each layer was modified
    layer_change_frames: HashMap<String, u64>,
    current_frame: u64,
}

impl LayerChangeTracker {
    pub fn new() -> Self {
        Self {
            layer_change_frames: HashMap::new(),
            current_frame: 0,
        }
    }

    pub fn next_frame(&mut self) {
        self.current_frame += 1;
    }

    /// Mark layer as changed
    pub fn mark_layer_changed(&mut self, layer_name: &str) {
        self.layer_change_frames.insert(
            layer_name.to_string(),
            self.current_frame,
        );
    }

    /// Check if layer was changed in current frame
    pub fn layer_changed_this_frame(&self, layer_name: &str) -> bool {
        self.layer_change_frames.get(layer_name)
            .map(|&frame| frame == self.current_frame)
            .unwrap_or(false)
    }

    /// Check if layer was changed recently
    pub fn layer_changed_recently(&self, layer_name: &str, frames_back: u64) -> bool {
        self.layer_change_frames.get(layer_name)
            .map(|&frame| self.current_frame - frame <= frames_back)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caching_avoids_retessellation() {
        let mut cache = EntityCache::new();
        let handle = 1u64;

        // First tessellation: not cached
        assert!(cache.needs_retessellation(handle));

        // Cache result
        cache.cache_tessellation(handle, vec![1, 2, 3]);

        // Second frame: should use cache
        cache.next_frame();
        assert!(!cache.needs_retessellation(handle));
        assert!(cache.get_cached_tessellation(handle).is_some());

        println!("Cached 1 entity - avoid 1 tessellation");
        assert_eq!(cache.cached_count(), 1);
    }

    #[test]
    fn test_dirty_marking_forces_retessellation() {
        let mut cache = EntityCache::new();
        let handle = 1u64;

        cache.cache_tessellation(handle, vec![1, 2, 3]);
        cache.next_frame();
        assert!(!cache.needs_retessellation(handle));

        // Mark dirty
        cache.mark_dirty(handle);
        assert!(cache.needs_retessellation(handle));

        println!("Entity marked dirty - tessellation forced");
    }

    #[test]
    fn test_large_cache_reduces_tessellation_work() {
        let mut cache = EntityCache::new();

        // Simulate 487 entities (all cached)
        for i in 0..487 {
            cache.cache_tessellation(i as u64, vec![1, 2, 3]);
        }

        cache.next_frame();

        // Count how many need retessellation
        let mut need_tessellation = 0;
        for i in 0..487 {
            if cache.needs_retessellation(i as u64) {
                need_tessellation += 1;
            }
        }

        println!("Cache prevents {} retessellations (0% needed)", 487 - need_tessellation);
        assert_eq!(need_tessellation, 0, "All cached entities should be reused");
    }

    #[test]
    fn test_memory_tracking() {
        let mut cache = EntityCache::new();

        // Add 100 entities with 1KB geometry each
        for i in 0..100 {
            cache.cache_tessellation(i as u64, vec![0u8; 1024]);
        }

        let memory = cache.memory_usage_bytes();
        println!("Cache using {}MB memory", memory / (1024 * 1024));
        assert!(memory > 100_000, "Should track memory usage");
    }
}
