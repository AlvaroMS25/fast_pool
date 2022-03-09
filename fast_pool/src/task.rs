use crate::channel::ChannelHalf;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::shared::Shared;

/// A synchronous task, any type implementing this trait can be ran inside the thread pool.
pub trait Task: Send + 'static
{
    type Output: Sized + Send + 'static;

    fn run(self) -> Self::Output;
}

impl<F, R> Task for F
where
    F: FnOnce() -> R + Send + 'static,
    R: Sized + Send + 'static,
{
    type Output = R;

    fn run(self) -> Self::Output {
        self()
    }
}

pub enum TaskType {
    Sync(SyncTask),
    Periodic(PeriodicTask)
}

impl TaskType {
    pub fn run(self) {
        match self {
            Self::Sync(task) => (task.fun)(),
            Self::Periodic(task) => task.run()
        }
    }
}

pub struct SyncTask {
    fun: Box<dyn FnOnce() + Send + 'static>,
}

impl SyncTask {
    pub fn new<R>(channel: Option<ChannelHalf<R>>, fun: impl Task<Output = R>) -> Self
    where
        R: Sized + Send + 'static,
    {
        Self {
            fun: Box::new(move || {
                let value = catch_unwind(AssertUnwindSafe(move || fun.run()));
                if let Some(channel) = channel {
                    channel.set(value)
                }
            }),
        }
    }
}

pub struct PeriodicTask {
    shared: Arc<Shared>,
    fun: Box<dyn Fn() + Send + 'static>,
    every: Duration,
    next: Instant,
    times: Option<usize>,
}

impl PeriodicTask {
    pub fn new<F>(shared: Arc<Shared>, fun: F, every: Duration, times: Option<usize>) -> Self
    where
        F: Fn() + Send + 'static
    {
        let next = Instant::now() + every;

        Self {
            shared,
            fun: Box::new(move || {
                let _ = catch_unwind(AssertUnwindSafe(|| (fun)()));
            }),
            every,
            next,
            times
        }
    }

    pub fn run(mut self) {
        (self.fun)();
        self.times.as_mut().map(|t| *t = *t-1);

        if self.times.is_none() || self.times.as_ref().map(|t| *t >= 1).unwrap() {
            self.next = Instant::now() + self.every;
            self.reschedule();
        }
    }

    pub fn schedule(self) {
        if self.shared.should_exit() {
            drop(self);
        } else {
            // SAFETY: As the task is holding the Arc we ensure the pointer is valid, also as we're
            // not modifying its contents, we avoid any possible data races. This way of scheduling
            // the task is just a workaround to avoid cloning the Arc every time.
            unsafe { (&*Arc::as_ptr(&self.shared)).schedule(TaskType::Periodic(self)); }
        }
    }

    pub fn reschedule(self) {
        crate::context::get_timer()
            .schedule(self);
    }

    pub fn can_run(&self) -> bool {
        Instant::now() > self.next
    }
}
