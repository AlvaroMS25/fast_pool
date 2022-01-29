use crate::handle::Handle;
use std::cell::RefCell;

struct HandleWrapper(pub RefCell<Option<Handle>>);
unsafe impl Sync for HandleWrapper {}

static HANDLE: HandleWrapper = HandleWrapper(RefCell::new(None));
const NOT_INITIALIZED: &str = "Thread pool not initialized";

pub fn get_handle() -> Handle {
    try_get().expect(NOT_INITIALIZED).clone()
}

pub fn try_get() -> Option<Handle> {
    HANDLE.0.borrow().clone()
}

pub fn set_handle(handle: Handle) {
    *HANDLE.0.borrow_mut() = Some(handle);
}

pub fn delete_handle() {
    *HANDLE.0.borrow_mut() = None;
}
