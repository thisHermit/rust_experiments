use std::time::Duration;
use std::thread;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use crate::task_scheduler::{TaskScheduler, SchedulerConfig};

/// Test timeout detection specifically
pub fn test_timeout_detection() {
    println!("=== Testing Timeout Detection ===");
    
    let config = SchedulerConfig {
        num_workers: 1, // Use only 1 worker to ensure task queuing
        timeout_seconds: 1, // Very short timeout for testing
        enable_work_stealing: false,
    };

    let mut scheduler = TaskScheduler::new(config);
    scheduler.start();

    let counter = Arc::new(AtomicUsize::new(0));

    // Submit a blocking task first to occupy the worker
    println!("Submitting blocking task to occupy worker...");
    let counter_clone = Arc::clone(&counter);
    scheduler.submit(move || {
        thread::sleep(Duration::from_secs(5)); // Long blocking task
        counter_clone.fetch_add(1, Ordering::SeqCst);
        println!("Blocking task completed");
    }).unwrap();

    // Submit a task that will be queued and should trigger timeout warning
    println!("Submitting queued task that should trigger timeout warning...");
    let counter_clone = Arc::clone(&counter);
    scheduler.submit(move || {
        counter_clone.fetch_add(1, Ordering::SeqCst);
        println!("Queued task completed (should have generated timeout warning)");
    }).unwrap();

    // Wait to see timeout detection in action
    println!("Waiting 3 seconds to see timeout warning (should appear after 1 second)...");
    thread::sleep(Duration::from_secs(3));

    // Wait for all tasks to complete
    while counter.load(Ordering::SeqCst) < 2 {
        thread::sleep(Duration::from_millis(100));
    }

    println!("Both tasks completed, shutting down...");
    scheduler.shutdown();
    println!("=== Timeout Detection Test Completed ===\n");
}