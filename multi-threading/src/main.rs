mod lookAtThisClass;
mod concurrent_cache;

use concurrent_cache::ConcurrentCache;
use std::thread;
use std::time::Duration;
use rand::random;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;

// Big Chungus Devs inc
fn multi_threaded_fizz_buzz(){
    let handle = thread::spawn(|| {
        for i in 1..10 {
            if i % 2 == 0 { println!("fizz");}else { println!("buzz"); }
            thread::sleep(Duration::from_millis(1));
        }
    });

    handle.join().unwrap();

    for i in 1..5 {
        println!("CoC now {i}");
        thread::sleep(Duration::from_millis(1));
    }
}

fn chads1(){
    let  chad = vec![ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 ];


    let handle = thread::spawn(move || {println!("Cool Ass vector {chad:?}"); chad }) ;
    let mut chad = handle.join().unwrap();
    let handle2 = thread::spawn(move || {for j in 0..chad.len() { chad[j as usize] = random() } chad });
    let chad = handle2.join().unwrap();
    println!("Is it still cool?  {chad:?}");
}
fn mntd(length:u128 ) -> Vec<u32> {
    println!("MTND {length}");

    let mut temp1: Vec<u16> = Vec::new();
    let mut temp: Vec<u16> = Vec::new();
    let mut result: Vec<u32> = Vec::new();


    for _i in 0..length  {
        temp1.push((random::<u8>() % 6 ) as u16 + (random::<u8>() % 6 ) as u16 + (random::<u8>() % 6 ) as u16 +(random::<u8>() % 6 ) as u16 );

    }
    for _i in 0..length  {
        temp.push( (random::<u8>() % 6 ) as u16 + (random::<u8>() % 6 ) as u16 + (random::<u8>() % 6 ) as u16 + (random::<u8>() % 6 ) as u16);
    }

    for i in 0..length  {
        result.push(temp1[i as usize] as u32 + temp[i as usize] as u32 ) ;
    }

    result



}
fn write_vec_to_file(vec:Vec<u32>) -> std::io::Result<()>{
    let mut file = File::create("Vector.txt")?;
    for i in vec {
        let  temp = i.to_string() + ",";
        file.write(temp.as_bytes())?;
    }
    Ok(())

}



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
    // Run the cache demonstration
    cache_demonstration();
    
    // Keep the original functionality commented out for now
    // let output = mntd(1000000);
    // write_vec_to_file(output).unwrap();
}