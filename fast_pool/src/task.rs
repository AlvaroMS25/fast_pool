use crate::channel::ChannelHalf;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[cfg(feature = "async")]
use crate::{vtable::Vtable, waker::WakerRef, shared::Shared};
#[cfg(feature = "async")]
use std::{future::Future, ptr::NonNull, task::Context, sync::Arc};

/// A synchronous task, any type implementing this trait can be ran inside the thread pool.
pub trait Task<R>: Send + 'static
where
    R: Sized + Send + 'static,
{
    fn run(self) -> R;
}

impl<F, R> Task<R> for F
where
    F: FnOnce() -> R + Send + 'static,
    R: Sized + Send + 'static,
{
    fn run(self) -> R {
        self()
    }
}

pub enum TaskType {
    Sync(SyncTask),

    #[cfg(feature = "async")]
    Async(AsyncTask),
}

impl TaskType {
    pub fn run(self) {
        match self {
            Self::Sync(task) => (task.fun)(),
            #[cfg(feature = "async")]
            Self::Async(mut task) => {
                let waker = WakerRef::new(&mut task);
                let context = &mut Context::from_waker(waker.waker());
                unsafe {
                    task.poll(context);
                }
            }
        }
    }
}

pub struct SyncTask {
    fun: Box<dyn FnOnce() + Send + 'static>,
}

impl SyncTask {
    pub fn new<R>(channel: Option<ChannelHalf<R>>, fun: impl Task<R>) -> Self
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

#[cfg(feature = "async")]
pub enum TaskState<T: Future + Send + 'static> {
    Incomplete(T),
    Completed,
}

#[cfg(feature = "async")]
#[repr(C)]
pub struct TaskInner<T>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    vtable: Vtable,
    pub state: TaskState<T>,
    pub channel: Option<ChannelHalf<T::Output>>,
    pub shared: Arc<Shared>
}

#[cfg(feature = "async")]
pub struct AsyncTask {
    pub ptr: NonNull<Vtable>,
}

#[cfg(feature = "async")]
unsafe impl Send for AsyncTask {}

#[cfg(feature = "async")]
impl AsyncTask {
    pub fn new<T>(shared: Arc<Shared>, channel: Option<ChannelHalf<T::Output>>, fut: T) -> Self
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        let ptr = Box::into_raw(Box::new(TaskInner {
            vtable: Vtable::new::<T>(),
            state: TaskState::Incomplete(fut),
            channel,
            shared
        })) as *mut Vtable;

        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
        }
    }

    pub unsafe fn from_ptr(ptr: *const Vtable) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr as *mut Vtable),
        }
    }

    pub unsafe fn poll(&mut self, cx: &mut Context) {
        let vtable = &*self.ptr.as_ptr();
        (vtable.poll)(self.ptr, cx)
    }
}
