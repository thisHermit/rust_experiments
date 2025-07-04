mod lookAtThisClass;
mod task_scheduler;
mod timeout_test;

use std::thread;
use std::time::Duration;
use rand::random;
use std::fs::File;
use std::io::prelude::*;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use task_scheduler::{TaskScheduler, SchedulerConfig};
use timeout_test::test_timeout_detection;

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



fn main() {
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