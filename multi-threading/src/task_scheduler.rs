use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// A task is a boxed closure that takes no arguments and returns nothing
pub type Task = Box<dyn FnOnce() + Send + 'static>;

/// Special marker task to signal shutdown
pub struct PoisonPill;

/// Task wrapper that includes metadata for timeout detection
#[derive(Debug)]
struct TaskMetadata {
    id: u64,
    submitted_at: Instant,
}

/// Task with metadata for the scheduler
struct ScheduledTask {
    task: Task,
    metadata: TaskMetadata,
}

/// Configuration for the task scheduler
#[derive(Clone)]
pub struct SchedulerConfig {
    pub num_workers: usize,
    pub timeout_seconds: u64,
    pub enable_work_stealing: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            num_workers: num_cpus::get(),
            timeout_seconds: 30,
            enable_work_stealing: true,
        }
    }
}

/// Individual worker queue for work stealing
struct WorkerQueue {
    queue: VecDeque<ScheduledTask>,
}

impl WorkerQueue {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    fn push(&mut self, task: ScheduledTask) {
        self.queue.push_back(task);
    }

    fn pop(&mut self) -> Option<ScheduledTask> {
        self.queue.pop_front()
    }

    fn steal(&mut self) -> Option<ScheduledTask> {
        // Steal from the back to minimize contention with the owner
        self.queue.pop_back()
    }

    fn len(&self) -> usize {
        self.queue.len()
    }
}

/// Shared state for the task scheduler
struct SchedulerState {
    worker_queues: Vec<Mutex<WorkerQueue>>,
    worker_condvars: Vec<Condvar>,
    shutdown: Mutex<bool>,
    shutdown_condvar: Condvar,
    task_counter: Mutex<u64>,
}

/// Main task scheduler implementation
pub struct TaskScheduler {
    state: Arc<SchedulerState>,
    config: SchedulerConfig,
    worker_handles: Vec<thread::JoinHandle<()>>,
    supervisor_handle: Option<thread::JoinHandle<()>>,
}

impl TaskScheduler {
    /// Create a new task scheduler with the given configuration
    pub fn new(config: SchedulerConfig) -> Self {
        let worker_queues = (0..config.num_workers)
            .map(|_| Mutex::new(WorkerQueue::new()))
            .collect();

        let worker_condvars = (0..config.num_workers)
            .map(|_| Condvar::new())
            .collect();

        let state = Arc::new(SchedulerState {
            worker_queues,
            worker_condvars,
            shutdown: Mutex::new(false),
            shutdown_condvar: Condvar::new(),
            task_counter: Mutex::new(0),
        });

        Self {
            state,
            config,
            worker_handles: Vec::new(),
            supervisor_handle: None,
        }
    }

    /// Start the scheduler with all worker threads and supervisor
    pub fn start(&mut self) {
        // Start worker threads
        for worker_id in 0..self.config.num_workers {
            let state = Arc::clone(&self.state);
            let config = self.config.clone();
            
            let handle = thread::spawn(move || {
                Self::worker_loop(worker_id, state, config);
            });
            
            self.worker_handles.push(handle);
        }

        // Start supervisor thread
        if self.config.timeout_seconds > 0 {
            let state = Arc::clone(&self.state);
            let timeout_duration = Duration::from_secs(self.config.timeout_seconds);
            
            let handle = thread::spawn(move || {
                Self::supervisor_loop(state, timeout_duration);
            });
            
            self.supervisor_handle = Some(handle);
        }
    }

    /// Submit a new task to the scheduler
    pub fn submit<F>(&self, task: F) -> Result<u64, &'static str>
    where
        F: FnOnce() + Send + 'static,
    {
        // Check if scheduler is shutting down
        if *self.state.shutdown.lock().map_err(|_| "Shutdown lock poisoned")? {
            return Err("Scheduler is shutting down");
        }

        // Generate task ID with overflow protection
        let task_id = {
            let mut counter = self.state.task_counter.lock().map_err(|_| "Task counter lock poisoned")?;
            *counter = counter.wrapping_add(1);
            *counter
        };

        let scheduled_task = ScheduledTask {
            task: Box::new(task),
            metadata: TaskMetadata {
                id: task_id,
                submitted_at: Instant::now(),
            },
        };

        // Find the worker queue with the least tasks (load balancing with retry)
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 3;
        
        while attempts < MAX_ATTEMPTS {
            let mut min_queue_size = usize::MAX;
            let mut best_worker = 0;
            let mut best_queue_guard = None;

            // Try to find and lock the best queue atomically
            for (i, queue_mutex) in self.state.worker_queues.iter().enumerate() {
                if let Ok(queue) = queue_mutex.try_lock() {
                    let size = queue.len();
                    if size < min_queue_size {
                        min_queue_size = size;
                        best_worker = i;
                        best_queue_guard = Some(queue);
                        // If we found an empty queue, use it immediately
                        if size == 0 {
                            break;
                        }
                    }
                }
            }

            // If we got a lock on the best queue, use it
            if let Some(mut queue) = best_queue_guard {
                queue.push(scheduled_task);
                // Notify the worker
                self.state.worker_condvars[best_worker].notify_one();
                return Ok(task_id);
            }

            // If we couldn't get any locks, fall back to blocking on the first worker
            if attempts == MAX_ATTEMPTS - 1 {
                let mut queue = self.state.worker_queues[0].lock().map_err(|_| "Worker queue lock poisoned")?;
                queue.push(scheduled_task);
                self.state.worker_condvars[0].notify_one();
                return Ok(task_id);
            }

            attempts += 1;
            // Brief yield before retrying
            std::thread::yield_now();
        }

        Err("Failed to submit task after multiple attempts")
    }

    /// Submit a poison pill to trigger shutdown
    pub fn submit_poison_pill(&self) {
        self.submit_poison_pill_safe();
    }

    /// Shutdown the scheduler and wait for all threads to complete
    pub fn shutdown(self) {
        // Signal shutdown first
        {
            let shutdown_result = self.state.shutdown.lock();
            match shutdown_result {
                Ok(mut shutdown) => {
                    *shutdown = true;
                    self.state.shutdown_condvar.notify_all();
                }
                Err(_) => {
                    eprintln!("Warning: Shutdown lock poisoned during shutdown");
                    return;
                }
            }
        }

        // Submit poison pills to ensure all workers wake up
        // Use a separate method that handles errors gracefully
        self.submit_poison_pill_safe();

        // Wait for all worker threads with panic detection
        let mut worker_panics = 0;
        for (i, handle) in self.worker_handles.into_iter().enumerate() {
            match handle.join() {
                Ok(_) => {
                    // Worker completed successfully
                }
                Err(_) => {
                    eprintln!("Warning: Worker thread {} panicked during shutdown", i);
                    worker_panics += 1;
                }
            }
        }

        // Wait for supervisor thread with panic detection
        if let Some(handle) = self.supervisor_handle {
            match handle.join() {
                Ok(_) => {
                    // Supervisor completed successfully
                }
                Err(_) => {
                    eprintln!("Warning: Supervisor thread panicked during shutdown");
                }
            }
        }

        if worker_panics > 0 {
            eprintln!("Warning: {} worker threads panicked during shutdown", worker_panics);
        }
    }

    /// Submit poison pills with error handling
    fn submit_poison_pill_safe(&self) {
        for (i, queue_mutex) in self.state.worker_queues.iter().enumerate() {
            match queue_mutex.lock() {
                Ok(mut queue) => {
                    let poison_task = ScheduledTask {
                        task: Box::new(|| {
                            // This is the poison pill task - it will trigger shutdown
                        }),
                        metadata: TaskMetadata {
                            id: u64::MAX, // Special ID for poison pill
                            submitted_at: Instant::now(),
                        },
                    };
                    queue.push(poison_task);
                    
                    // Notify the worker
                    self.state.worker_condvars[i].notify_one();
                }
                Err(_) => {
                    eprintln!("Warning: Could not submit poison pill to worker {}, queue lock poisoned", i);
                    // Try to notify anyway in case the worker is waiting
                    self.state.worker_condvars[i].notify_one();
                }
            }
        }
    }

    /// Worker thread main loop
    fn worker_loop(worker_id: usize, state: Arc<SchedulerState>, config: SchedulerConfig) {
        loop {
            // Check for shutdown with error handling
            let should_shutdown = match state.shutdown.lock() {
                Ok(shutdown) => *shutdown,
                Err(_) => {
                    eprintln!("Worker {}: Shutdown lock poisoned, exiting", worker_id);
                    return;
                }
            };
            
            if should_shutdown {
                break;
            }

            // Try to get a task from our own queue first
            let task = {
                let queue_mutex = &state.worker_queues[worker_id];
                let condvar = &state.worker_condvars[worker_id];
                
                let mut queue_guard = match queue_mutex.lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        eprintln!("Worker {}: Queue lock poisoned, exiting", worker_id);
                        return;
                    }
                };
                
                // Wait for tasks if queue is empty
                loop {
                    if queue_guard.len() > 0 {
                        break queue_guard.pop();
                    }
                    
                    // Check shutdown again before waiting
                    let should_shutdown = match state.shutdown.lock() {
                        Ok(shutdown) => *shutdown,
                        Err(_) => {
                            eprintln!("Worker {}: Shutdown lock poisoned while waiting, exiting", worker_id);
                            return;
                        }
                    };
                    
                    if should_shutdown {
                        return;
                    }
                    
                    queue_guard = match condvar.wait(queue_guard) {
                        Ok(guard) => guard,
                        Err(_) => {
                            eprintln!("Worker {}: Condition variable wait failed, exiting", worker_id);
                            return;
                        }
                    };
                }
            };

            if let Some(scheduled_task) = task {
                // Check if this is a poison pill
                if scheduled_task.metadata.id == u64::MAX {
                    break;
                }

                // Execute the task with panic protection
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    (scheduled_task.task)();
                })).unwrap_or_else(|_| {
                    eprintln!("Worker {}: Task {} panicked during execution", worker_id, scheduled_task.metadata.id);
                });
                continue;
            }

            // Try work stealing if enabled and no task found
            if config.enable_work_stealing {
                let stolen_task = Self::try_steal_work(worker_id, &state);
                if let Some(scheduled_task) = stolen_task {
                    // Check if this is a poison pill
                    if scheduled_task.metadata.id == u64::MAX {
                        break;
                    }

                    // Execute the stolen task with panic protection
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        (scheduled_task.task)();
                    })).unwrap_or_else(|_| {
                        eprintln!("Worker {}: Stolen task {} panicked during execution", worker_id, scheduled_task.metadata.id);
                    });
                    continue;
                }
            }

            // No work found, yield to other threads
            thread::yield_now();
        }
    }

    /// Try to steal work from other worker queues
    fn try_steal_work(worker_id: usize, state: &Arc<SchedulerState>) -> Option<ScheduledTask> {
        let num_workers = state.worker_queues.len();
        
        // Try to steal from other workers in round-robin fashion
        for i in 1..num_workers {
            let target_worker = (worker_id + i) % num_workers;
            
            if let Ok(mut queue) = state.worker_queues[target_worker].try_lock() {
                if let Some(task) = queue.steal() {
                    // Don't steal poison pills - they are meant for specific workers
                    if task.metadata.id == u64::MAX {
                        // Put the poison pill back at the end of the queue
                        queue.push(task);
                        continue;
                    }
                    return Some(task);
                }
            }
        }
        
        None
    }

    /// Supervisor thread main loop for timeout detection
    fn supervisor_loop(state: Arc<SchedulerState>, timeout_duration: Duration) {
        let check_interval = Duration::from_millis(500); // More responsive checking
        
        loop {
            // Check for shutdown with error handling
            let should_shutdown = match state.shutdown.lock() {
                Ok(shutdown) => *shutdown,
                Err(_) => {
                    eprintln!("Supervisor: Shutdown lock poisoned, exiting");
                    return;
                }
            };
            
            if should_shutdown {
                break;
            }

            // Sleep with shorter intervals for more responsive shutdown
            thread::sleep(check_interval);

            // Check all queues for stale tasks
            let now = Instant::now();
            for (worker_id, queue_mutex) in state.worker_queues.iter().enumerate() {
                // Use try_lock to avoid blocking and reduce contention
                if let Ok(queue) = queue_mutex.try_lock() {
                    for (task_index, scheduled_task) in queue.queue.iter().enumerate() {
                        let age = now.duration_since(scheduled_task.metadata.submitted_at);
                        if age > timeout_duration {
                            eprintln!(
                                "Warning: Task {} in worker {} queue position {} has been waiting for {:?} (timeout: {:?})",
                                scheduled_task.metadata.id,
                                worker_id,
                                task_index,
                                age,
                                timeout_duration
                            );
                        }
                    }
                }
                // If we can't get the lock, skip this queue to avoid blocking
            }
        }
    }
}

/// Helper function to get the number of logical CPUs
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn test_scheduler_creation() {
        let config = SchedulerConfig::default();
        let scheduler = TaskScheduler::new(config);
        assert_eq!(scheduler.config.num_workers, num_cpus::get());
    }

    #[test]
    fn test_task_submission_and_execution() {
        let config = SchedulerConfig {
            num_workers: 2,
            timeout_seconds: 5,
            enable_work_stealing: true,
        };
        
        let mut scheduler = TaskScheduler::new(config);
        scheduler.start();

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        // Submit a task
        let task_id = scheduler.submit(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }).unwrap();

        assert!(task_id > 0);

        // Wait a bit for task execution
        thread::sleep(Duration::from_millis(100));

        // Check that task was executed
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        scheduler.shutdown();
    }

    #[test]
    fn test_multiple_tasks() {
        let config = SchedulerConfig {
            num_workers: 2,
            timeout_seconds: 5,
            enable_work_stealing: true,
        };
        
        let mut scheduler = TaskScheduler::new(config);
        scheduler.start();

        let counter = Arc::new(AtomicUsize::new(0));
        
        // Submit multiple tasks
        for _ in 0..10 {
            let counter_clone = Arc::clone(&counter);
            scheduler.submit(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }).unwrap();
        }

        // Wait for all tasks to complete
        thread::sleep(Duration::from_millis(200));

        assert_eq!(counter.load(Ordering::SeqCst), 10);

        scheduler.shutdown();
    }

    #[test]
    fn test_shutdown_after_poison_pill() {
        let config = SchedulerConfig {
            num_workers: 1,
            timeout_seconds: 5,
            enable_work_stealing: false,
        };
        
        let mut scheduler = TaskScheduler::new(config);
        scheduler.start();

        // Submit a poison pill
        scheduler.submit_poison_pill();

        // Scheduler should shut down cleanly
        scheduler.shutdown();
    }
}