use crate::concurrent_cache::ConcurrentCache;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::{Duration, Instant};
use std::thread;

pub fn run_cache_benchmark() {
    println!("\n🚀 Cache Performance Benchmark");
    println!("===============================");

    let cache = Arc::new(ConcurrentCache::new(
        "benchmark_cache.log".to_string(),
        Duration::from_secs(30) // Long TTL to focus on concurrency
    ));

    let computation_count = Arc::new(AtomicUsize::new(0));
    let read_count = Arc::new(AtomicUsize::new(0));

    // Benchmark 1: High read contention
    println!("\n📖 Benchmark 1: High Read Contention (100 threads, 10 keys)");
    let start = Instant::now();
    let mut handles = vec![];

    for i in 0..100 {
        let cache_clone = Arc::clone(&cache);
        let comp_count = Arc::clone(&computation_count);
        let read_count_clone = Arc::clone(&read_count);

        let handle = thread::spawn(move || {
            let key = format!("key_{}", i % 10); // 10 different keys
            let _value = cache_clone.get(&key, || {
                comp_count.fetch_add(1, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(10)); // Simulate computation
                Ok(format!("value_for_{}", key))
            }).unwrap();
            read_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    let computations = computation_count.load(Ordering::SeqCst);
    let reads = read_count.load(Ordering::SeqCst);

    println!("  ⏱️  Duration: {:?}", duration);
    println!("  📊 Total reads: {}", reads);
    println!("  🔄 Computations: {} (should be ≤ 10)", computations);
    println!("  📈 Reads/sec: {:.2}", reads as f64 / duration.as_secs_f64());
    println!("  🎯 Cache efficiency: {:.1}%", (1.0 - (computations as f64 / reads as f64)) * 100.0);

    // Reset counters
    computation_count.store(0, Ordering::SeqCst);
    read_count.store(0, Ordering::SeqCst);

    // Benchmark 2: Write-heavy workload
    println!("\n✍️  Benchmark 2: Write-Heavy Workload (50 writers)");
    let start = Instant::now();
    let mut handles = vec![];

    for i in 0..50 {
        let cache_clone = Arc::clone(&cache);

        let handle = thread::spawn(move || {
            let key = format!("write_key_{}", i);
            let value = format!("write_value_{}", i);
            cache_clone.put(key, value).unwrap();
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    println!("  ⏱️  Duration: {:?}", duration);
    println!("  📊 Writes: 50");
    println!("  📈 Writes/sec: {:.2}", 50.0 / duration.as_secs_f64());

    // Benchmark 3: Mixed workload
    println!("\n🔄 Benchmark 3: Mixed Workload (70% reads, 30% writes)");
    let start = Instant::now();
    let mut handles = vec![];

    for i in 0..100 {
        let cache_clone = Arc::clone(&cache);
        let comp_count = Arc::clone(&computation_count);
        let read_count_clone = Arc::clone(&read_count);

        let handle = thread::spawn(move || {
            if i < 70 {
                // Read operation
                let key = format!("mixed_key_{}", i % 5);
                let _value = cache_clone.get(&key, || {
                    comp_count.fetch_add(1, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(5));
                    Ok(format!("mixed_value_{}", key))
                }).unwrap();
                read_count_clone.fetch_add(1, Ordering::SeqCst);
            } else {
                // Write operation
                let key = format!("mixed_write_{}", i);
                let value = format!("mixed_write_value_{}", i);
                cache_clone.put(key, value).unwrap();
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    let computations = computation_count.load(Ordering::SeqCst);
    let _reads = read_count.load(Ordering::SeqCst);

    println!("  ⏱️  Duration: {:?}", duration);
    println!("  📊 Total operations: 100 (70 reads, 30 writes)");
    println!("  🔄 Read computations: {} (should be ≤ 5)", computations);
    println!("  📈 Operations/sec: {:.2}", 100.0 / duration.as_secs_f64());

    println!("\n✅ Benchmark completed!");
    println!("💾 Check 'benchmark_cache.log' for persistence verification");
}