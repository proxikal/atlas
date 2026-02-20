//! Interpreter caching for optimized variable and function lookups
//!
//! Provides caching layers to reduce repeated scope chain traversals
//! and function body lookups during interpretation.

use std::collections::HashMap;

/// Cache entry recording where a variable was found
#[derive(Debug, Clone, Copy)]
pub struct VarLocation {
    /// Scope index in the locals stack (None = globals)
    pub scope_index: Option<usize>,
    /// Generation counter when this entry was cached
    pub generation: u64,
}

/// Environment lookup cache for reducing scope chain traversals
///
/// Caches the location (scope index) where variables were last found.
/// Uses a generation counter to invalidate stale entries when scopes change.
#[derive(Debug, Default)]
pub struct LookupCache {
    /// Variable name -> location mapping
    entries: HashMap<String, VarLocation>,
    /// Current generation (incremented on scope changes)
    generation: u64,
    /// Statistics for cache performance monitoring
    stats: CacheStats,
}

/// Cache performance statistics
#[derive(Debug, Default, Clone, Copy)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of stale entries encountered
    pub stale: u64,
}

impl LookupCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            generation: 0,
            stats: CacheStats::default(),
        }
    }

    /// Record that a scope has been entered (invalidates some cache entries)
    pub fn scope_entered(&mut self) {
        self.generation += 1;
    }

    /// Record that a scope has been exited (invalidates some cache entries)
    pub fn scope_exited(&mut self) {
        self.generation += 1;
    }

    /// Look up a cached variable location
    ///
    /// Returns Some(location) if the cache entry is valid, None otherwise.
    pub fn get(&mut self, name: &str) -> Option<VarLocation> {
        if let Some(entry) = self.entries.get(name) {
            // Check if entry is still valid (within current generation window)
            // We allow entries from recent generations to remain valid
            // since shadowing is relatively rare
            if entry.generation >= self.generation.saturating_sub(2) {
                self.stats.hits += 1;
                return Some(*entry);
            }
            self.stats.stale += 1;
        }
        self.stats.misses += 1;
        None
    }

    /// Cache a variable location
    pub fn insert(&mut self, name: String, scope_index: Option<usize>) {
        self.entries.insert(
            name,
            VarLocation {
                scope_index,
                generation: self.generation,
            },
        );
    }

    /// Clear all cache entries (full invalidation)
    pub fn clear(&mut self) {
        self.entries.clear();
        self.generation = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats
    }

    /// Reset statistics counters
    pub fn reset_stats(&mut self) {
        self.stats = CacheStats::default();
    }

    /// Calculate cache hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.stats.hits + self.stats.misses;
        if total == 0 {
            0.0
        } else {
            (self.stats.hits as f64 / total as f64) * 100.0
        }
    }
}

/// Function lookup cache for faster function call resolution
///
/// Caches function existence checks to avoid repeated HashMap lookups.
#[derive(Debug, Default)]
pub struct FunctionCache {
    /// Function name -> exists in function_bodies
    known_functions: HashMap<String, bool>,
    /// Statistics
    stats: CacheStats,
}

impl FunctionCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            known_functions: HashMap::new(),
            stats: CacheStats::default(),
        }
    }

    /// Check if a function is known to exist
    pub fn is_known(&mut self, name: &str) -> Option<bool> {
        if let Some(&exists) = self.known_functions.get(name) {
            self.stats.hits += 1;
            Some(exists)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Record that a function exists (or doesn't)
    pub fn record(&mut self, name: String, exists: bool) {
        self.known_functions.insert(name, exists);
    }

    /// Invalidate cache for a specific function (e.g., when defined)
    pub fn invalidate(&mut self, name: &str) {
        self.known_functions.remove(name);
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.known_functions.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats
    }
}

/// Combined interpreter cache
#[derive(Debug, Default)]
pub struct InterpreterCache {
    /// Variable lookup cache
    pub lookup: LookupCache,
    /// Function lookup cache
    pub functions: FunctionCache,
}

impl InterpreterCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            lookup: LookupCache::new(),
            functions: FunctionCache::new(),
        }
    }

    /// Clear all caches
    pub fn clear(&mut self) {
        self.lookup.clear();
        self.functions.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_cache_basic() {
        let mut cache = LookupCache::new();

        // Initially empty
        assert!(cache.get("x").is_none());

        // Insert and retrieve
        cache.insert("x".to_string(), Some(0));
        let loc = cache.get("x").unwrap();
        assert_eq!(loc.scope_index, Some(0));
    }

    #[test]
    fn test_lookup_cache_generation() {
        let mut cache = LookupCache::new();

        cache.insert("x".to_string(), Some(0));
        assert!(cache.get("x").is_some());

        // Scope change should not immediately invalidate
        cache.scope_entered();
        assert!(cache.get("x").is_some());

        // Multiple scope changes eventually invalidate
        cache.scope_entered();
        cache.scope_entered();
        cache.scope_entered();
        assert!(cache.get("x").is_none());
    }

    #[test]
    fn test_lookup_cache_stats() {
        let mut cache = LookupCache::new();

        // Miss
        cache.get("x");
        assert_eq!(cache.stats().misses, 1);

        // Insert
        cache.insert("x".to_string(), Some(0));

        // Hit
        cache.get("x");
        assert_eq!(cache.stats().hits, 1);

        // Hit rate
        assert_eq!(cache.hit_rate(), 50.0);
    }

    #[test]
    fn test_function_cache_basic() {
        let mut cache = FunctionCache::new();

        // Initially unknown
        assert!(cache.is_known("foo").is_none());

        // Record and check
        cache.record("foo".to_string(), true);
        assert_eq!(cache.is_known("foo"), Some(true));

        // Invalidate
        cache.invalidate("foo");
        assert!(cache.is_known("foo").is_none());
    }

    #[test]
    fn test_interpreter_cache_combined() {
        let mut cache = InterpreterCache::new();

        cache.lookup.insert("x".to_string(), Some(0));
        cache.functions.record("foo".to_string(), true);

        assert!(cache.lookup.get("x").is_some());
        assert_eq!(cache.functions.is_known("foo"), Some(true));

        cache.clear();

        assert!(cache.lookup.get("x").is_none());
        assert!(cache.functions.is_known("foo").is_none());
    }

    #[test]
    fn test_lookup_cache_globals() {
        let mut cache = LookupCache::new();

        // Global variable (scope_index = None)
        cache.insert("global".to_string(), None);
        let loc = cache.get("global").unwrap();
        assert_eq!(loc.scope_index, None);
    }

    #[test]
    fn test_hit_rate_zero_accesses() {
        let cache = LookupCache::new();
        assert_eq!(cache.hit_rate(), 0.0);
    }

    #[test]
    fn test_reset_stats() {
        let mut cache = LookupCache::new();
        cache.insert("x".to_string(), Some(0));
        cache.get("x");
        cache.get("y");

        assert_eq!(cache.stats().hits, 1);
        assert_eq!(cache.stats().misses, 1);

        cache.reset_stats();

        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 0);
    }
}
