use crate::threadpool::ThreadPool;
use std::sync::Arc;

pub(crate) type HookFn = dyn Fn() + Send + Sync + 'static;
pub(crate) type NameFn = dyn Fn() -> String + Send + Sync + 'static;

/// A builder which allows to configure the thread pool before building it.
pub struct ThreadPoolBuilder {
    pub(crate) on_start: Option<Arc<HookFn>>,
    pub(crate) on_stop: Option<Arc<HookFn>>,
    pub(crate) before: Option<Arc<HookFn>>,
    pub(crate) after: Option<Arc<HookFn>>,
    pub(crate) name: Arc<NameFn>,
    pub(crate) thread_number: usize,
    pub(crate) stack_size: Option<usize>,
}

impl ThreadPoolBuilder {
    /// Creates a new [builder](self::ThreadPoolBuilder)
    pub fn new() -> Self {
        Self {
            on_start: None,
            on_stop: None,
            before: None,
            after: None,
            name: Arc::new(|| String::from("fast_pool-worker")),
            thread_number: num_cpus::get() * 2,
            stack_size: None,
        }
    }

    /// Sets a function to execute before every task.
    pub fn before<F>(mut self, fun: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.before = Some(Arc::new(fun));
        self
    }

    /// Sets a function to execute after every task.
    pub fn after<F>(mut self, fun: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.after = Some(Arc::new(fun));
        self
    }

    /// Sets a function to execute at thread creation, before start doing any work.
    pub fn on_start<F>(mut self, fun: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_start = Some(Arc::new(fun));
        self
    }

    /// Sets a function to execute at thread stop, this will mainly be called when the thread pool
    /// shuts down just before exiting the thread.
    pub fn on_stop<F>(mut self, fun: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_stop = Some(Arc::new(fun));
        self
    }

    /// Sets the name for the threads of the pool.
    pub fn thread_name(mut self, name: impl ToString) -> Self {
        let name = name.to_string();
        self.name = Arc::new(move || name.clone());
        self
    }

    /// Sets a function to determine the name for the threads of the pool.
    pub fn thread_name_fn<F>(mut self, fun: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.name = Arc::new(fun);
        self
    }

    /// Sets the stack size for the threads of the pool.
    pub fn thread_stack_size(mut self, size: usize) -> Self {
        self.stack_size = Some(size);
        self
    }

    /// Sets the number of threads of the pool.
    pub fn thread_number(mut self, threads: usize) -> Self {
        self.thread_number = threads;
        self
    }

    /// Builds into a [ThreadPool](ThreadPool) and starts it.
    pub fn build(self) -> std::io::Result<ThreadPool> {
        ThreadPool::start(self)
    }
}
