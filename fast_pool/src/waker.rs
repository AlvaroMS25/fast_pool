use crate::{
    task::{AsyncTask, TaskType},
    vtable::Vtable,
};
use std::{
    future::Future,
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr::NonNull,
    task::{RawWaker, RawWakerVTable, Waker},
};
use crate::task::TaskInner;

pub struct WakerRef<'a> {
    waker: ManuallyDrop<Waker>,
    _marker: PhantomData<&'a AsyncTask>,
}

impl<'a> WakerRef<'a> {
    pub fn new(task: &mut AsyncTask) -> Self {
        let waker = unsafe { Waker::from_raw(raw_waker(task.ptr)) };

        Self {
            waker: ManuallyDrop::new(waker),
            _marker: PhantomData,
        }
    }

    pub fn waker(&self) -> &Waker {
        &self.waker
    }
}

unsafe fn clone<T>(ptr: *const ()) -> RawWaker
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    (*(ptr as *const TaskInner<T>)).inc_waker();
    let ptr = NonNull::new_unchecked(ptr as *mut Vtable);
    raw_waker(ptr)
}

unsafe fn wake<T>(ptr: *const ())
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    if ptr.is_null() {
        return;
    }

    let task = &*(ptr as *const TaskInner<T>);

    if task.shared.should_exit() {
        let vtable = NonNull::new_unchecked(ptr as *mut Vtable);
        ((*vtable.as_ptr()).clean)(vtable)
    } else {
        let t = AsyncTask::from_ptr(ptr as *const Vtable);
        task.shared.schedule(TaskType::Async(t));
    }
}

unsafe fn wake_by_ref<T>(ptr: *const ())
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    wake::<T>(ptr);
}

unsafe fn drop<T>(ptr: *const ())
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    if (ptr as *const TaskInner<T>).is_null() {
        return;
    }

    let vtable = NonNull::new_unchecked(ptr as *mut Vtable);

    ((*vtable.as_ptr()).drop)(vtable)
}

fn raw_waker(ptr: NonNull<Vtable>) -> RawWaker {
    let vtable = unsafe { &(*ptr.as_ptr()) }.inner;
    RawWaker::new(ptr.as_ptr() as *const (), vtable)
}

pub fn new_vtable<T>() -> &'static RawWakerVTable
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    &RawWakerVTable::new(
        clone::<T>,
        wake::<T>,
        wake_by_ref::<T>,
        drop::<T>
    )
}
