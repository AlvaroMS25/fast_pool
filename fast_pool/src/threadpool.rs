use crate::{builder::ThreadPoolBuilder, handle::Handle, shared::Shared, worker::Worker};
use std::{collections::VecDeque, sync::Arc};

/// The thread pool used to execute tasks.
pub struct ThreadPool {
    /// A handle to allow managing the pool.
    handle: Handle,
}

impl ThreadPool {
    /// Creates a new pool with default values.
    pub fn default() -> std::io::Result<Self> {
        ThreadPoolBuilder::new().build()
    }

    pub(crate) fn start(builder: ThreadPoolBuilder) -> std::io::Result<Self> {
        let shared = Shared::new();
        let mut handles = VecDeque::new();

        use std::thread::Builder;
        for _ in 0..builder.thread_number {
            let worker = Worker::new(
                Arc::clone(&shared),
                builder.before.as_ref().map(Arc::clone),
                builder.after.as_ref().map(Arc::clone),
                builder.on_start.as_ref().map(Arc::clone),
                builder.on_stop.as_ref().map(Arc::clone)
            );

            let mut thread_builder = Builder::new();

            if let Some(size) = &builder.stack_size {
                thread_builder = thread_builder.stack_size(*size);
            }

            let handle = thread_builder
                .name((builder.name)())
                .spawn(move || worker.run())?;

            handles.push_back(handle);
        }

        let handle = Handle::new(shared, handles);
        crate::context::set_handle(handle.clone());

        Ok(Self { handle })
    }
    /// Returns a reference to the current [handle](Handle).
    pub fn handle_ref(&self) -> &Handle {
        &self.handle
    }

    /// Unlike [handle_ref](Self::handle_ref), returns an owned [handle](Handle).
    pub fn handle(&self) -> Handle {
        self.handle_ref().clone()
    }

    /// Shuts down the thread pool, waiting for all threads to exit.
    pub fn shutdown(self) {
        self.handle.shutdown()
    }
}

impl std::ops::Deref for ThreadPool {
    type Target = Handle;

    fn deref(&self) -> &Self::Target {
        self.handle_ref()
    }
}
