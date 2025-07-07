use std::time::Duration;
use std::thread;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use crate::task_scheduler::{TaskScheduler, SchedulerConfig};

/// Demonstrate work stealing functionality
pub fn demonstrate_work_stealing() {
    println!("=== Demonstrating Work Stealing ===");
    
    let config = SchedulerConfig {
        num_workers: 4,
        timeout_seconds: 5,
        enable_work_stealing: true,
    };

    let mut scheduler = TaskScheduler::new(config);
    scheduler.start();

    let counter = Arc::new(AtomicUsize::new(0));

    println!("Submitting 50 tasks to demonstrate work stealing across 4 workers...");

    // Submit many small tasks that will be distributed via work stealing
    for i in 0..50 {
        let counter_clone = Arc::clone(&counter);
        
        scheduler.submit(move || {
            // Simulate some work
            thread::sleep(Duration::from_millis(10));
            
            counter_clone.fetch_add(1, Ordering::SeqCst);
            if i % 10 == 0 {
                println!("  Task {} completed", i);
            }
        }).unwrap();
    }

    // Wait for all tasks to complete
    while counter.load(Ordering::SeqCst) < 50 {
        thread::sleep(Duration::from_millis(10));
    }

    println!("All 50 tasks completed! Work stealing ensured efficient distribution.");
    println!("Note: With work stealing enabled, idle workers steal tasks from busy workers,");
    println!("      ensuring optimal CPU utilization and minimal task starvation.");

    scheduler.shutdown();
    println!("=== Work Stealing Demonstration Completed ===\n");
}