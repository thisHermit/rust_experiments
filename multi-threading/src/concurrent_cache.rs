use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::thread;
use std::fs::{File, OpenOptions};
use std::io::{Write};
use std::hash::Hash;
use std::fmt::Debug;

/// Cache entry with value and expiration time
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

impl<V> CacheEntry<V> {
    fn new(value: V, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// A thread-safe in-memory cache with expiration and write-through
pub struct ConcurrentCache<K, V> 
where
    K: Clone + Hash + Eq + Debug + Send + Sync + 'static,
    V: Clone + Debug + Send + Sync + 'static,
{
    cache: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    recompute_locks: Arc<Mutex<HashMap<K, Arc<Mutex<()>>>>>,
    backing_store_lock: Arc<Mutex<()>>,
    backing_store_path: String,
    default_ttl: Duration,
}

impl<K, V> ConcurrentCache<K, V>
where
    K: Clone + Hash + Eq + Debug + Send + Sync + 'static,
    V: Clone + Debug + Send + Sync + 'static,
{
    /// Create a new concurrent cache with specified TTL and backing store path
    pub fn new(backing_store_path: String, default_ttl: Duration) -> Self {
        // Ensure backing store file exists
        if let Ok(_) = File::create(&backing_store_path) {
            // File created or already exists
        }

        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            recompute_locks: Arc::new(Mutex::new(HashMap::new())),
            backing_store_lock: Arc::new(Mutex::new(())),
            backing_store_path,
            default_ttl,
        }
    }

    /// Get a value from the cache, blocking if it needs recomputation
    pub fn get<F>(&self, key: &K, recompute_fn: F) -> Result<V, String>
    where
        F: FnOnce() -> Result<V, String>,
    {
        // First, try to read from cache
        {
            let cache_read = self.cache.read().map_err(|e| format!("Cache read lock error: {}", e))?;
            if let Some(entry) = cache_read.get(key) {
                if !entry.is_expired() {
                    return Ok(entry.value.clone());
                }
            }
        }

        // Cache miss or expired - need to recompute
        // Get or create a lock for this specific key
        let key_lock = {
            let mut locks = self.recompute_locks.lock().map_err(|e| format!("Recompute locks error: {}", e))?;
            locks.entry(key.clone()).or_insert_with(|| Arc::new(Mutex::new(()))).clone()
        };

        // Acquire the key-specific lock to ensure only one thread recomputes this key
        let _key_guard = key_lock.lock().map_err(|e| format!("Key lock error: {}", e))?;

        // Double-check: another thread might have computed it while we were waiting
        {
            let cache_read = self.cache.read().map_err(|e| format!("Cache read lock error: {}", e))?;
            if let Some(entry) = cache_read.get(key) {
                if !entry.is_expired() {
                    return Ok(entry.value.clone());
                }
            }
        }

        // Recompute the value
        let new_value = recompute_fn()?;

        // Store in cache and write-through to backing store
        self.put(key.clone(), new_value.clone())?;

        Ok(new_value)
    }

    /// Put a value into the cache with write-through to backing store
    pub fn put(&self, key: K, value: V) -> Result<(), String> {
        // Write to cache
        {
            let mut cache_write = self.cache.write().map_err(|e| format!("Cache write lock error: {}", e))?;
            cache_write.insert(key.clone(), CacheEntry::new(value.clone(), self.default_ttl));
        }

        // Write-through to backing store
        self.write_to_backing_store(&key, &value)?;

        Ok(())
    }

    /// Write to the simulated backing store
    fn write_to_backing_store(&self, key: &K, value: &V) -> Result<(), String> {
        // Use backing store lock to prevent concurrent writes from corrupting the file
        let _lock = self.backing_store_lock.lock().map_err(|e| format!("Backing store lock error: {}", e))?;
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.backing_store_path)
            .map_err(|e| format!("Failed to open backing store: {}", e))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?
            .as_secs();

        writeln!(file, "{}: {:?} -> {:?}", timestamp, key, value)
            .map_err(|e| format!("Failed to write to backing store: {}", e))?;

        Ok(())
    }

    /// Start the garbage collector thread
    pub fn start_garbage_collector(&self, gc_interval: Duration) -> thread::JoinHandle<()> {
        let cache_clone = Arc::clone(&self.cache);
        
        thread::spawn(move || {
            loop {
                thread::sleep(gc_interval);
                
                if let Ok(mut cache_write) = cache_clone.write() {
                    cache_write.retain(|_, entry| !entry.is_expired());
                    println!("Garbage collection completed. Cache size: {}", cache_write.len());
                }
            }
        })
    }

    /// Get current cache size (for monitoring)
    pub fn size(&self) -> usize {
        self.cache.read().map(|cache| cache.len()).unwrap_or(0)
    }

    /// Clear all expired entries manually
    pub fn cleanup_expired(&self) -> Result<usize, String> {
        let mut cache_write = self.cache.write().map_err(|e| format!("Cache write lock error: {}", e))?;
        let initial_size = cache_write.len();
        cache_write.retain(|_, entry| !entry.is_expired());
        let final_size = cache_write.len();
        Ok(initial_size - final_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_cache_basic_operations() {
        let cache = ConcurrentCache::new("test_cache.log".to_string(), Duration::from_secs(5));
        
        // Test put and get
        cache.put("key1".to_string(), "value1".to_string()).unwrap();
        
        let value = cache.get(&"key1".to_string(), || Ok("recomputed".to_string())).unwrap();
        assert_eq!(value, "value1");
    }

    #[test]
    fn test_cache_expiration() {
        let cache = ConcurrentCache::new("test_cache_exp.log".to_string(), Duration::from_millis(100));
        
        cache.put("key1".to_string(), "value1".to_string()).unwrap();
        
        // Should get cached value
        let value = cache.get(&"key1".to_string(), || Ok("recomputed".to_string())).unwrap();
        assert_eq!(value, "value1");
        
        // Wait for expiration
        thread::sleep(Duration::from_millis(150));
        
        // Should get recomputed value
        let value = cache.get(&"key1".to_string(), || Ok("recomputed".to_string())).unwrap();
        assert_eq!(value, "recomputed");
    }

    #[test]
    fn test_concurrent_access() {
        let cache = Arc::new(ConcurrentCache::new("test_cache_concurrent.log".to_string(), Duration::from_secs(1)));
        let counter = Arc::new(AtomicUsize::new(0));
        
        let mut handles = vec![];
        
        // Spawn multiple reader threads
        for i in 0..5 {
            let cache_clone = Arc::clone(&cache);
            let counter_clone = Arc::clone(&counter);
            
            let handle = thread::spawn(move || {
                let key = format!("key{}", i % 2); // Use only 2 keys to force contention
                let value = cache_clone.get(&key, || {
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                    Ok(format!("computed_value_{}", i))
                }).unwrap();
                
                println!("Thread {} got value: {}", i, value);
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Should have computed at most 2 values (one for each key)
        let computations = counter.load(Ordering::SeqCst);
        assert!(computations <= 2, "Too many computations: {}", computations);
    }
}