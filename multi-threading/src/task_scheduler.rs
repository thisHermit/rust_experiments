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
        if *self.state.shutdown.lock().unwrap() {
            return Err("Scheduler is shutting down");
        }

        // Generate task ID
        let mut counter = self.state.task_counter.lock().unwrap();
        *counter += 1;
        let task_id = *counter;
        drop(counter);

        let scheduled_task = ScheduledTask {
            task: Box::new(task),
            metadata: TaskMetadata {
                id: task_id,
                submitted_at: Instant::now(),
            },
        };

        // Find the worker queue with the least tasks (load balancing)
        let mut min_queue_size = usize::MAX;
        let mut best_worker = 0;

        for (i, queue_mutex) in self.state.worker_queues.iter().enumerate() {
            if let Ok(queue) = queue_mutex.try_lock() {
                let size = queue.len();
                if size < min_queue_size {
                    min_queue_size = size;
                    best_worker = i;
                }
            }
        }

        // Submit to the best worker queue
        let mut queue = self.state.worker_queues[best_worker].lock().unwrap();
        queue.push(scheduled_task);
        
        // Notify the worker
        self.state.worker_condvars[best_worker].notify_one();

        Ok(task_id)
    }

    /// Submit a poison pill to trigger shutdown
    pub fn submit_poison_pill(&self) {
        // Submit poison pills to all worker queues
        for (i, queue_mutex) in self.state.worker_queues.iter().enumerate() {
            let mut queue = queue_mutex.lock().unwrap();
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
    }

    /// Shutdown the scheduler and wait for all threads to complete
    pub fn shutdown(self) {
        // Signal shutdown
        {
            let mut shutdown = self.state.shutdown.lock().unwrap();
            *shutdown = true;
            self.state.shutdown_condvar.notify_all();
        }

        // Submit poison pills to ensure all workers wake up
        self.submit_poison_pill();

        // Wait for all worker threads
        for handle in self.worker_handles {
            let _ = handle.join();
        }

        // Wait for supervisor thread
        if let Some(handle) = self.supervisor_handle {
            let _ = handle.join();
        }
    }

    /// Worker thread main loop
    fn worker_loop(worker_id: usize, state: Arc<SchedulerState>, config: SchedulerConfig) {
        loop {
            // Check for shutdown
            if *state.shutdown.lock().unwrap() {
                break;
            }

            // Try to get a task from our own queue first
            let task = {
                let queue_mutex = &state.worker_queues[worker_id];
                let condvar = &state.worker_condvars[worker_id];
                let mut queue_guard = queue_mutex.lock().unwrap();
                
                // Wait for tasks if queue is empty
                loop {
                    if queue_guard.len() > 0 {
                        break queue_guard.pop();
                    }
                    
                    if *state.shutdown.lock().unwrap() {
                        return; // Exit the worker
                    }
                    
                    queue_guard = condvar.wait(queue_guard).unwrap();
                }
            };

            if let Some(scheduled_task) = task {
                // Check if this is a poison pill
                if scheduled_task.metadata.id == u64::MAX {
                    break;
                }

                // Execute the task
                (scheduled_task.task)();
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

                    // Execute the stolen task
                    (scheduled_task.task)();
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
                    return Some(task);
                }
            }
        }
        
        None
    }

    /// Supervisor thread main loop for timeout detection
    fn supervisor_loop(state: Arc<SchedulerState>, timeout_duration: Duration) {
        loop {
            // Check for shutdown
            if *state.shutdown.lock().unwrap() {
                break;
            }

            // Sleep before next check
            thread::sleep(Duration::from_secs(1));

            // Check all queues for stale tasks
            let now = Instant::now();
            for (worker_id, queue_mutex) in state.worker_queues.iter().enumerate() {
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