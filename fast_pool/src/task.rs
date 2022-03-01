use crate::channel::ChannelHalf;
use std::panic::{catch_unwind, AssertUnwindSafe};

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
}

impl TaskType {
    pub fn run(self) {
        match self {
            Self::Sync(task) => (task.fun)(),
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
