use crossbeam_utils::sync::{Parker, Unparker};
#[cfg(feature = "async")]
use std::task::{Context, Poll, Waker};
use std::{cell::UnsafeCell, sync::Arc};

type BoxedError = Box<dyn std::any::Any + Send + 'static>;

struct ChannelInner<T> {
    unparker: Option<Unparker>,
    data: Option<Result<T, BoxedError>>,
    #[cfg(feature = "async")]
    waker: Option<Waker>,
}

impl<T: Send> ChannelInner<T> {
    fn new() -> Self {
        Self {
            unparker: None,
            data: None,
            #[cfg(feature = "async")]
            waker: None,
        }
    }
}

pub struct ChannelHalf<T> {
    inner: Arc<UnsafeCell<ChannelInner<T>>>,
}

unsafe impl<T> Send for ChannelHalf<T> {}
unsafe impl<T> Sync for ChannelHalf<T> {}

impl<T: Send + Sized + 'static> ChannelHalf<T> {
    fn new(cell: Arc<UnsafeCell<ChannelInner<T>>>) -> Self {
        Self { inner: cell }
    }

    pub fn new_pair() -> (Self, Self) {
        let inner = Arc::new(UnsafeCell::new(ChannelInner::new()));
        (Self::new(Arc::clone(&inner)), Self::new(inner))
    }

    #[cfg(feature = "async")]
    pub fn set_waker(&self, waker: Waker) {
        // SAFETY: Only the worker thread which is executing the task uses this method, so there is
        // no risk of any data races.
        let inner = unsafe { &mut *self.inner.get() };
        inner.waker = Some(waker);
    }

    pub fn try_get(&self) -> Option<Result<T, BoxedError>> {
        // SAFETY: Only one thread is meant to use this method, so there are no race conditions.
        let inner = unsafe { &mut *self.inner.get() };
        inner.data.take()
    }

    pub fn set(self, value: Result<T, BoxedError>) {
        // SAFETY: Only a worker thread is able to use this method.
        let inner = unsafe { &mut *self.inner.get() };
        inner.data = Some(value);

        if let Some(unparker) = inner.unparker.take() {
            unparker.unpark();
            return;
        }

        #[cfg(feature = "async")]
        if let Some(waker) = inner.waker.take() {
            waker.wake();
        }
    }

    #[cfg(feature = "async")]
    pub fn wait_async(&mut self, cx: &mut Context) -> Poll<Result<T, BoxedError>> {
        if let Some(value) = self.try_get() {
            Poll::Ready(value)
        } else {
            self.set_waker(cx.waker().clone());
            Poll::Pending
        }
    }

    pub fn wait(self) -> Result<T, BoxedError> {
        if let Some(value) = self.try_get() {
            value
        } else {
            // SAFETY: This method can only be called once as it consumes `self`.
            let inner = unsafe { &mut *self.inner.get() };
            let parker = Parker::new();
            inner.unparker = Some(parker.unparker().clone());

            // Park the thread so no work is done while waiting.
            parker.park();
            // Now wait until the thread is unparked.
            self.try_get().unwrap()
        }
    }
}
