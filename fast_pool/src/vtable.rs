use crate::task::{TaskInner, TaskState};
use std::{
    future::Future,
    panic::{catch_unwind, AssertUnwindSafe},
    pin::Pin,
    ptr::NonNull,
    task::{Context, Poll, RawWakerVTable},
};

unsafe fn poll<T>(ptr: NonNull<Vtable>, cx: &mut Context)
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    let inner = &mut *ptr.cast::<TaskInner<T>>().as_ptr();

    if let TaskState::Incomplete(fut) = &mut inner.state {
        let res = catch_unwind(AssertUnwindSafe(|| Pin::new_unchecked(fut).poll(cx)));

        match res {
            Err(why) => {
                if let Some(channel) = inner.channel.take() {
                    channel.set(Err(why));
                }
            }
            Ok(output) => {
                if let Poll::Ready(value) = output {
                    if let Some(channel) = inner.channel.take() {
                        channel.set(Ok(value));
                    }
                } else {
                    return;
                }
            }
        }

        inner.state = TaskState::Completed;
    }
}

unsafe fn drop_if_needed<T>(ptr: NonNull<Vtable>)
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    let inner = &mut *ptr.cast::<TaskInner<T>>().as_ptr();

    if let TaskState::Completed = inner.state {
        drop(Box::from_raw(ptr.as_ptr() as *mut TaskInner<T>));
    }
}

unsafe fn clean<T>(ptr: NonNull<Vtable>)
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    let inner = &mut *ptr.cast::<TaskInner<T>>().as_ptr();

    let state = std::mem::replace(&mut inner.state, TaskState::Completed);

    match state {
        TaskState::Completed => (),
        TaskState::Incomplete(fut) => drop(fut),
    };

    if let Some(channel) = inner.channel.take() {
        channel.set(Err(Box::new("ThreadPool dropped")))
    }

    drop_if_needed::<T>(ptr);
}

pub struct Vtable {
    pub inner: &'static RawWakerVTable,
    pub poll: unsafe fn(NonNull<Vtable>, &mut Context),
    pub drop: unsafe fn(NonNull<Vtable>),
    pub clean: unsafe fn(NonNull<Vtable>),
}

impl Vtable {
    pub fn new<T>() -> Self
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        Self {
            inner: crate::waker::new_vtable::<T>(),
            poll: poll::<T>,
            drop: drop_if_needed::<T>,
            clean: clean::<T>,
        }
    }
}
