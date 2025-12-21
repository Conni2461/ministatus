use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

pub struct Shared<T> {
    v: Rc<RefCell<T>>,
}

impl<T> Shared<T> {
    pub fn new(t: T) -> Self {
        Self {
            v: Rc::new(RefCell::new(t)),
        }
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.v.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.v.borrow_mut()
    }

    pub fn as_ptr(&self) -> *mut T {
        self.v.as_ptr()
    }

    pub fn clone_rc(&self) -> Self {
        Self {
            v: Rc::clone(&self.v),
        }
    }
}
