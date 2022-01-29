mod builder;
mod channel;
mod context;
mod handle;
mod join;
mod shared;
mod task;
mod threadpool;
#[cfg(feature = "async")]
mod vtable;
#[cfg(feature = "async")]
mod waker;
mod worker;

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
    T: Task<R>,
    R: Sized + Send + 'static,
{
    Handle::current().spawn(task)
}

/// Spawns a new task into the pool, but unlike [`spawn`](Self::spawn), doesn't return a
/// handle to retrieve the output of the task, this is useful to avoid the allocation needed
/// to create the channel when the output is not needed.
pub fn spawn_detached<T, R>(task: T)
where
    T: Task<R>,
    R: Sized + Send + 'static,
{
    Handle::current().spawn_detached(task)
}

#[cfg(feature = "async")]
/// Spawns a future into the thread pool, returning a handle which can be `.await`ed or waited
/// synchronously to retrieve the output value.
pub fn spawn_async<Fut, R>(fut: Fut) -> JoinHandle<R>
where
    Fut: std::future::Future<Output = R> + Send + 'static,
    R: Sized + Send + 'static,
{
    Handle::current().spawn_async(fut)
}

#[cfg(feature = "async")]
/// Spawns a future into the thread pool, but instead of returning a handle to retrieve the
/// output, it detaches completely the task, this is useful to avoid the allocation needed to
/// allow to retrieve the output when it's not needed.
pub fn spawn_async_detached<Fut, R>(fut: Fut)
where
    Fut: std::future::Future<Output = R> + Send + 'static,
    R: Sized + Send + 'static,
{
    Handle::current().spawn_async_detached(fut)
}

#[cfg(test)]
mod test;
