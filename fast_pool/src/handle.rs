use crate::{
    channel::ChannelHalf,
    join::JoinHandle,
    shared::Shared,
    task::{SyncTask, Task, TaskType},
};
use parking_lot::Mutex;
use std::{
    collections::VecDeque,
    sync::{atomic::Ordering, Arc},
    thread::JoinHandle as StdThreadJoinHandle,
};
use std::time::Duration;
use crate::task::PeriodicalTask;

/// A handle used to spawn tasks into the thread pool.
#[derive(Clone)]
pub struct Handle {
    /// The shared data between all workers.
    pub(crate) shared: Arc<Shared>,
    /// The handles of all worker threads.
    handles: Arc<Mutex<VecDeque<StdThreadJoinHandle<()>>>>,
}

impl Handle {
    pub(crate) fn new(shared: Arc<Shared>, handles: VecDeque<StdThreadJoinHandle<()>>) -> Self {
        Self {
            shared,
            handles: Arc::new(Mutex::new(handles)),
        }
    }

    /// Gets the handle of the currently running thread pool.
    pub fn current() -> Self {
        crate::context::get_handle()
    }

    /// Gets the handle of the currently running thread pool if it exists.
    pub fn try_get() -> Option<Self> {
        crate::context::try_get()
    }

    /// Shuts down the thread pool, waiting for all threads to exit.
    pub fn shutdown(self) {
        crate::context::delete_handle();
        self.shared.exit.swap(true, Ordering::Relaxed);
        self.shared.condvar.notify_all();
        let mut lock = self.handles.lock();

        while let Some(handle) = lock.pop_front() {
            handle.join().expect("Failed to join thread");
        }

        self.clean();
    }

    fn clean(&self) {
        let mut lock = self.shared.queue.lock();
        while let Some(task) = lock.pop_front() {
            match task {
                TaskType::Sync(task) => drop(task),
            }
        }

        let mut lock = self.shared.periodical_tasks.lock();
        while let Some(task) = lock.pop_front() {
            drop(task);
        }
    }

    /// Spawns a new task into the thread pool, returning a handle which can be used to retrieve
    /// the output of the task.
    pub fn spawn<T, R>(&self, task: T) -> JoinHandle<R>
    where
        T: Task<Output = R>,
        R: Sized + Send + 'static,
    {
        let (rx, tx) = ChannelHalf::<R>::new_pair();
        self.shared
            .schedule(TaskType::Sync(SyncTask::new(Some(tx), task)));
        JoinHandle::new(rx)
    }

    /// Spawns a new task into the pool, but unlike [`spawn`](Self::spawn), doesn't return a
    /// handle to retrieve the output of the task, this is useful to avoid the allocation needed
    /// to retrieve the output when it's not needed.
    pub fn spawn_detached<T, R>(&self, task: T)
    where
        T: Task<Output = R>,
        R: Sized + Send + 'static,
    {
        self.shared
            .schedule(TaskType::Sync(SyncTask::new(None, task)));
    }

    /// Creates a new periodic task that will be ran every [every](Duration) time and the number
    /// of times given, if the number of times given is [None](None) the task will run until the
    /// thread pool gets closed.
    pub fn periodical<F>(&self, fun: F, every: Duration, times: Option<usize>)
    where
        F: Fn() + Send + 'static
    {
        self.shared.schedule_periodical(PeriodicalTask::new(fun, every, times));
    }
}
