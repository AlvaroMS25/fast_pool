use crate::{task::{TaskType, PeriodicTask}, worker::WorkerAction};
use parking_lot::{Condvar, Mutex, MutexGuard};
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

/// The shared data for all workers in the thread pool.
pub struct Shared {
    /// Queue of tasks.
    pub queue: Mutex<VecDeque<TaskType>>,
    /// The variable used by worker threads to wait for notifications.
    pub condvar: Condvar,
    /// The lock used along with the upper condvar.
    lock: Mutex<()>,
    /// Whether the workers should stop and exit.
    pub exit: AtomicBool,
    /// Queue of tasks that run periodically.
    pub periodic_tasks: Mutex<VecDeque<PeriodicTask>>
}

impl Shared {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            queue: Mutex::new(VecDeque::new()),
            condvar: Condvar::new(),
            lock: Mutex::new(()),
            exit: AtomicBool::new(false),
            periodic_tasks: Mutex::new(VecDeque::new())
        })
    }

    fn try_get(&self) -> Option<TaskType> {
        self.queue.try_lock().map(|mut q| q.pop_front()).flatten()
    }

    fn get_periodic(
        &self,
        lock: &mut MutexGuard<VecDeque<PeriodicTask>>
    ) -> Option<PeriodicTask>
    {
        while let Some(task) = lock.pop_front() {
            if task.can_run() {
                return Some(task)
            } else {
                lock.push_back(task);
            }
        }

        None
    }

    pub fn should_exit(&self) -> bool {
        self.exit.load(Ordering::Relaxed)
    }

    pub fn run_periodical(&self) {
        if let Some(mut lock) = self.periodic_tasks.try_lock() {
            println!("Locked");
            if let Some(mut task) = self.get_periodic(&mut lock) {
                if task.run() {
                    lock.push_back(task);
                    if lock.len() > 0 {
                        //self.condvar.notify_one();
                    }
                }
            }
        }
    }

    pub fn wait(&self) -> WorkerAction {
        if self.should_exit() {
            WorkerAction::Exit
        } else {
            if let Some(task) = self.try_get() {
                WorkerAction::Run(task)
            } else {
                // If there are no tasks available, just try to run periodical ones.
                self.run_periodical();

                let mut lock = self.lock.lock();
                self.condvar.wait(&mut lock);
                if self.should_exit() {
                    WorkerAction::Exit
                } else {
                    match self.queue.lock().pop_front() {
                        Some(task) => WorkerAction::Run(task),
                        None => {
                            // If we got here, we got unparked and the task was not stored yet, so
                            // just notify another thread and wait again.
                            self.condvar.notify_one();
                            WorkerAction::Retry
                        }
                    }
                }
            }
        }
    }

    pub fn schedule(&self, task: TaskType) {
        if self.should_exit() {
            panic!("Cannot spawn a task, thread pool exited.");
        }

        self.queue.lock().push_back(task);
        self.condvar.notify_one();
    }

    pub fn schedule_periodic(&self, task: PeriodicTask) {
        if self.should_exit() {
            panic!("Cannot spawn a task, thread pool exited.");
        }

        self.periodic_tasks.lock().push_back(task);
        self.condvar.notify_one();
    }
}
