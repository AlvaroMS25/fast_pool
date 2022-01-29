use crate::channel::ChannelHalf;
#[cfg(feature = "async")]
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// A handle used to wait for output values of tasks.
///
/// This is returned by [spawn](crate::handle::Handle::spawn) and
/// [spawn_async](crate::Handle::spawn_async)
pub struct JoinHandle<T>(ChannelHalf<T>);

impl<T: Send + Sized + 'static> JoinHandle<T> {
    pub(crate) fn new(half: ChannelHalf<T>) -> Self {
        Self(half)
    }

    /// Waits synchronously for the output of this task.
    pub fn wait(self) -> Result<T, Box<dyn std::any::Any + Send + 'static>> {
        self.0.wait()
    }
}

#[cfg(feature = "async")]
impl<T: Send + Sized + 'static> Future for JoinHandle<T> {
    type Output = Result<T, Box<dyn std::any::Any + Send + 'static>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_mut().0.wait_async(cx)
    }
}
