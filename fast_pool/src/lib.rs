#![feature(drain_filter)]

mod builder;
mod channel;
mod context;
mod handle;
mod join;
mod shared;
mod task;
mod threadpool;
mod timer;
mod worker;

use std::time::Duration;
pub use builder::ThreadPoolBuilder;
pub use handle::Handle;
pub use join::JoinHandle;
pub use task::Task;
pub use threadpool::ThreadPool;

#[cfg(feature = "macros")]
pub use fast_pool_macros::init;

/// Spawns a new task into the thread pool, returning a handle which can be used to retrieve
/// the output of the task.
pub fn spawn<T, R>(task: T) -> JoinHandle<R>
where
    T: Task<Output = R>,
    R: Sized + Send + 'static,
{
    Handle::current().spawn(task)
}

/// Spawns a new task into the pool, but unlike [`spawn`](self::spawn), doesn't return a
/// handle to retrieve the output of the task, this is useful to avoid the allocation needed
/// to create the channel when the output is not needed.
pub fn spawn_detached<T, R>(task: T)
where
    T: Task<Output = R>,
    R: Sized + Send + 'static,
{
    Handle::current().spawn_detached(task)
}

/// Creates a new periodic task that will be ran every [every](Duration) time and the number
/// of times given, if the number of times given is [None](None) the task will run until the
/// thread pool gets closed.
pub fn periodic<F>(fun: F, every: Duration, times: Option<usize>)
where
    F: Fn() + Send + 'static
{
    Handle::current().periodic(fun, every, times)
}

#[cfg(test)]
mod test;
