# Concurrent In-Memory Cache with Expiration and Write-Through

This implementation provides a thread-safe in-memory cache in Rust with expiration and write-through capabilities.

## Features

- **Thread-Safe**: Multiple readers can access the cache concurrently using `RwLock`
- **Expiration**: Cache entries automatically expire after a configurable TTL
- **Write-Through**: All cache writes are persisted to a backing store
- **Per-Key Locking**: Only one thread can recompute a specific key at a time
- **Garbage Collection**: Automatic cleanup of expired entries
- **Blocking Reads**: Readers block until expired entries are recomputed

## Key Design Decisions

### Thread Safety
- Uses `Arc<RwLock<HashMap>>` for the main cache to allow multiple concurrent readers
- Uses `Arc<Mutex<HashMap<Key, Arc<Mutex<()>>>>>` for per-key locks during recomputation
- Separate `Mutex` for backing store writes to prevent file corruption

### Cache Entry Management
- Each entry includes value and expiration timestamp
- Configurable TTL (Time-To-Live) for entries
- Automatic expiration checking on reads

### Recomputation Strategy
- Cache misses or expired entries trigger recomputation
- Per-key locking ensures only one thread computes each key
- Other threads block and receive the computed value

## Usage Example

```rust
use concurrent_cache::ConcurrentCache;
use std::time::Duration;
use std::sync::Arc;

// Create cache with 5-second TTL
let cache = Arc::new(ConcurrentCache::new(
    "backing_store.log".to_string(),
    Duration::from_secs(5)
));

// Start garbage collector
let _gc_handle = cache.start_garbage_collector(Duration::from_secs(3));

// Get value with recomputation function
let value = cache.get(&"my_key".to_string(), || {
    // Expensive computation here
    Ok("computed_value".to_string())
}).unwrap();

// Direct put (also writes through to backing store)
cache.put("another_key".to_string(), "direct_value".to_string()).unwrap();
```

## Demonstration

Run `cargo run` to see a comprehensive demonstration that shows:

1. **Multiple Reader Threads**: 8 threads requesting 3 keys concurrently
2. **Recomputation Blocking**: Only one thread computes each key, others block and reuse
3. **Writer Threads**: Direct cache insertions with write-through
4. **Expiration Testing**: Entries expire and are recomputed
5. **Garbage Collection**: Automatic cleanup of expired entries

## Test Suite

Run `cargo test` to execute tests covering:

- Basic cache operations
- Expiration functionality
- Concurrent access patterns
- High contention scenarios (20 threads, 1 key)
- Garbage collection

## Performance Characteristics

- **Read-Heavy Workloads**: Excellent with RwLock allowing concurrent reads
- **Write Performance**: Sequential due to per-key locks and write-through
- **Memory Management**: Automatic cleanup prevents memory leaks
- **Scalability**: Handles high thread contention gracefully

## Thread Safety Guarantees

1. **Data Race Free**: All shared data protected by appropriate synchronization
2. **Deadlock Free**: Lock ordering prevents deadlocks
3. **Consistent Reads**: Readers always get valid, non-corrupted data
4. **Atomic Updates**: Cache and backing store updates are atomic per key

## Files Created

- `cache_backing_store.log`: Persistent write-through storage
- `test_cache*.log`: Test artifacts (ignored in .gitignore)

This implementation satisfies all requirements from the original issue:
- ✅ Multiple concurrent readers
- ✅ Smaller number of writers
- ✅ Entry expiration
- ✅ Blocking on cache miss/expiration
- ✅ Per-key recomputation locking
- ✅ Write-through persistence
- ✅ Garbage collection