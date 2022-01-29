use crate::{builder::HookFn, shared::Shared, task::TaskType};
use std::sync::Arc;

pub enum WorkerAction {
    Run(TaskType),
    Retry,
    Exit,
}

/// A worker of the thread pool.
pub struct Worker {
    /// The data shared between all workers.
    shared: Arc<Shared>,
    /// The function executed before every task.
    before: Option<Arc<HookFn>>,
    /// The function executed after every task.
    after: Option<Arc<HookFn>>,
    /// The function executed at thread creation.
    on_start: Option<Arc<HookFn>>,
    /// The function executed just before exiting the thread.
    on_stop: Option<Arc<HookFn>>,
}

impl Worker {
    pub fn new(
        shared: Arc<Shared>,
        before: Option<Arc<HookFn>>,
        after: Option<Arc<HookFn>>,
        on_start: Option<Arc<HookFn>>,
        on_stop: Option<Arc<HookFn>>,
    ) -> Self {
        Self {
            shared,
            before,
            after,
            on_start,
            on_stop
        }
    }

    pub fn run(self) {
        if let Some(fun) = self.on_start {
            (fun)();
        }

        loop {
            match self.shared.wait() {
                WorkerAction::Run(task) => {
                    if let Some(before) = &self.before {
                        (before)();
                    }

                    task.run();

                    if let Some(after) = &self.after {
                        (after)();
                    }
                }
                WorkerAction::Retry => (),
                WorkerAction::Exit => break,
            }
        }

        if let Some(fun) = self.on_stop {
            (fun)();
        }
    }
}
