// Concurrent In-Memory Cache with Expiration and Write-Through
// Clean main file focused on cache functionality
mod lookAtThisClass;
mod task_scheduler;
mod timeout_test;
mod work_stealing_demo;

use std::thread;
use std::time::Duration;
use rand::random;
use std::fs::File;
use std::io::prelude::*;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use task_scheduler::{TaskScheduler, SchedulerConfig};
use timeout_test::test_timeout_detection;
use work_stealing_demo::demonstrate_work_stealing;

// Big Chungus Devs inc
fn multi_threaded_fizz_buzz(){
    let handle = thread::spawn(|| {
        for i in 1..10 {
            if i % 2 == 0 { println!("fizz");}else { println!("buzz"); }
            thread::sleep(Duration::from_millis(1));
        }
    });

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
    println!("=== Rust Multi-threading Experiments ===\n");

    // Run the original examples
    println!("1. Running original MNTD example:");
    let output = mntd(1000);
    write_vec_to_file(output).unwrap();
    println!("   MNTD output written to Vector.txt\n");

    // Demonstrate the new concurrent task scheduler
    println!("2. Demonstrating Concurrent Task Scheduler:");
    demonstrate_task_scheduler();

    // Test timeout detection specifically
    println!("3. Testing Timeout Detection:");
    test_timeout_detection();

    // Demonstrate work stealing
    println!("4. Demonstrating Work Stealing:");
    demonstrate_work_stealing();

    println!("=== All experiments completed ===");
}

/// Demonstrate the concurrent task scheduler with various scenarios
fn demonstrate_task_scheduler() {
    // Configuration for the scheduler
    let config = SchedulerConfig {
        num_workers: 4,
        timeout_seconds: 2,
        enable_work_stealing: true,
    };

    println!("   Creating scheduler with {} workers, {}s timeout, work stealing: {}",
             config.num_workers, config.timeout_seconds, config.enable_work_stealing);

    let mut scheduler = TaskScheduler::new(config);
    scheduler.start();

    // Shared counter to demonstrate task execution
    let task_counter = Arc::new(AtomicUsize::new(0));
    let start_time = std::time::Instant::now();

    println!("   Submitting 20 computational tasks...");

    // Submit multiple computational tasks
    for i in 0..20 {
        let counter = Arc::clone(&task_counter);
        let task_id = i;
        
        scheduler.submit(move || {
            // Simulate some computational work
            let mut sum = 0u64;
            for j in 0..1000000 {
                sum += j as u64;
            }
            
            counter.fetch_add(1, Ordering::SeqCst);
            println!("     Task {} completed (sum: {})", task_id, sum);
        }).unwrap();
    }

    // Submit some I/O-bound tasks
    println!("   Submitting 5 I/O-bound tasks...");
    for i in 0..5 {
        let counter = Arc::clone(&task_counter);
        let task_id = 20 + i;
        
        scheduler.submit(move || {
            // Simulate I/O work
            thread::sleep(Duration::from_millis(100));
            counter.fetch_add(1, Ordering::SeqCst);
            println!("     I/O Task {} completed", task_id);
        }).unwrap();
    }

    // Submit a task that takes longer to demonstrate timeout detection
    println!("   Submitting a slow task to test timeout detection...");
    let counter = Arc::clone(&task_counter);
    scheduler.submit(move || {
        println!("     Slow task starting...");
        thread::sleep(Duration::from_secs(3)); // This should trigger timeout warning
        counter.fetch_add(1, Ordering::SeqCst);
        println!("     Slow task completed");
    }).unwrap();

    // Wait for most tasks to complete
    println!("   Waiting for tasks to complete...");
    while task_counter.load(Ordering::SeqCst) < 25 {
        thread::sleep(Duration::from_millis(50));
    }

    let elapsed = start_time.elapsed();
    println!("   All 26 tasks completed in {:?}", elapsed);

    // Demonstrate clean shutdown
    println!("   Shutting down scheduler...");
    scheduler.shutdown();
    println!("   Scheduler shut down cleanly");
}