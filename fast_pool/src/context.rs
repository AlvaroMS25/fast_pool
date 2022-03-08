use crate::handle::Handle;
use std::cell::RefCell;
use crate::timer::TimerHandle;

struct HandleWrapper {
    pub threadpool: RefCell<Option<Handle>>,
    pub timer: RefCell<Option<TimerHandle>>
}
unsafe impl Sync for HandleWrapper {}

static HANDLE: HandleWrapper = HandleWrapper {
    threadpool: RefCell::new(None),
    timer: RefCell::new(None)
};
const NOT_INITIALIZED: &str = "Thread pool not initialized";

pub fn get_handle() -> Handle {
    try_get().expect(NOT_INITIALIZED).clone()
}

pub fn try_get() -> Option<Handle> {
    HANDLE.threadpool.borrow().clone()
}

pub fn set_handle(handle: Handle) {
    *HANDLE.threadpool.borrow_mut() = Some(handle);
}

pub fn delete_handle() {
    *HANDLE.threadpool.borrow_mut() = None;
}

pub fn get_timer() -> TimerHandle {
    let option = HANDLE.timer.borrow().clone();

    if option.is_none() {
        *HANDLE.timer.borrow_mut() = Some(TimerHandle::new().expect("Failed to spawn timer"));
    };

    option.or_else(|| {
        HANDLE.timer.borrow().clone()
    }).unwrap()
}

pub fn get_timer_optional() -> Option<TimerHandle> {
    HANDLE.timer.borrow().clone()
}

pub fn delete_timer() {
    *HANDLE.timer.borrow_mut() = None;
}
