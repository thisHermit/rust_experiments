// Concurrent In-Memory Cache with Expiration and Write-Through
// Clean main file focused on cache functionality

mod concurrent_cache;
mod benchmark;
mod original_experiments;

use concurrent_cache::ConcurrentCache;
use benchmark::run_cache_benchmark;
use original_experiments::run_original_experiments;
use std::thread;
use std::time::Duration;
use std::sync::Arc;



fn cache_demonstration() {
    println!("=== Concurrent Cache with Expiration and Write-Through Demo ===");
    
    // Create cache with 5-second TTL
    let cache = Arc::new(ConcurrentCache::new(
        "cache_backing_store.log".to_string(),
        Duration::from_secs(5)
    ));
    
    // Start garbage collector thread
    let _gc_handle = cache.start_garbage_collector(Duration::from_secs(3));
    
    println!("Cache created with 5-second TTL and garbage collection every 3 seconds");
    
    // Simulate expensive computation
    let expensive_computation = |key: &str| -> Result<String, String> {
        println!("🔄 Computing expensive value for key: {}", key);
        thread::sleep(Duration::from_millis(500)); // Simulate work
        Ok(format!("computed_value_for_{}", key))
    };
    
    let mut handles = vec![];
    
    // Spawn multiple reader threads
    println!("\n📖 Spawning multiple reader threads...");
    for i in 0..8 {
        let cache_clone = Arc::clone(&cache);
        let key = format!("key_{}", i % 3); // Use only 3 keys to create contention
        
        let handle = thread::spawn(move || {
            let thread_id = i;
            println!("Thread {} requesting key: {}", thread_id, key);
            
            let result = cache_clone.get(&key, || expensive_computation(&key));
            
            match result {
                Ok(value) => println!("✅ Thread {} got: {} -> {}", thread_id, key, value),
                Err(e) => println!("❌ Thread {} error: {}", thread_id, e),
            }
        });
        
        handles.push(handle);
        
        // Stagger thread starts slightly
        thread::sleep(Duration::from_millis(50));
    }
    
    // Spawn a few writer threads
    println!("\n✍️  Spawning writer threads...");
    for i in 0..3 {
        let cache_clone = Arc::clone(&cache);
        let key = format!("writer_key_{}", i);
        let value = format!("writer_value_{}", i);
        
        let handle = thread::spawn(move || {
            println!("Writer thread {} inserting: {} -> {}", i, key, value);
            
            if let Err(e) = cache_clone.put(key.clone(), value.clone()) {
                println!("❌ Writer {} error: {}", i, e);
            } else {
                println!("✅ Writer {} successfully wrote: {} -> {}", i, key, value);
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("\n📊 Cache size after operations: {}", cache.size());
    
    // Test expiration
    println!("\n⏰ Testing expiration...");
    println!("Waiting 6 seconds for entries to expire...");
    thread::sleep(Duration::from_secs(6));
    
    // Try to access an expired key
    let result = cache.get(&"key_0".to_string(), || expensive_computation("key_0"));
    match result {
        Ok(value) => println!("✅ Got value after expiration (should be recomputed): {}", value),
        Err(e) => println!("❌ Error accessing expired key: {}", e),
    }
    
    // Manual cleanup to see how many expired
    match cache.cleanup_expired() {
        Ok(removed) => println!("🧹 Manual cleanup removed {} expired entries", removed),
        Err(e) => println!("❌ Cleanup error: {}", e),
    }
    
    println!("📊 Final cache size: {}", cache.size());
    println!("\n✅ Cache demonstration completed!");
    println!("💾 Check 'cache_backing_store.log' for write-through persistence");
}

fn main() {
    println!("🦀 Rust Experiments - Concurrent Cache Implementation 🦀");
    println!("=====================================================");
    
    // Check command line arguments for what to run
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "cache" => {
                cache_demonstration();
            },
            "benchmark" => {
                run_cache_benchmark();
            },
            "original" => {
                run_original_experiments();
            },
            "all" => {
                cache_demonstration();
                println!("\n{}", "=".repeat(50));
                run_cache_benchmark();
                println!("\n{}", "=".repeat(50));
                run_original_experiments();
            },
            _ => {
                print_usage();
            }
        }
    } else {
        // Default: run cache demonstration and benchmarks
        cache_demonstration();
        println!("\n{}", "=".repeat(50));
        run_cache_benchmark();
    }
}

fn print_usage() {
    println!("Usage: cargo run [option]");
    println!("Options:");
    println!("  cache     - Run cache demonstration only");
    println!("  benchmark - Run cache benchmarks only");
    println!("  original  - Run original threading experiments");
    println!("  all       - Run everything");
    println!("  (no args) - Run cache demo and benchmarks (default)");
}