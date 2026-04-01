use std::cell::RefCell;
use std::rc::Rc;

use crate::window::component::interface::drawable::Drawable;

pub type SharedDrawable = Rc<RefCell<dyn Drawable>>;

pub trait SharedDrawableExt {
    fn call_as<T: 'static>(&self, f: impl FnMut(&T));
    fn call_as_mut<T: 'static>(&self, f: impl FnMut(&mut T));
}

impl SharedDrawableExt for SharedDrawable {
    fn call_as<T: 'static>(&self, mut f: impl FnMut(&T)) {
        let borrow = self.borrow();
        if let Some(concrete) = borrow.as_any().downcast_ref::<T>() {
            f(concrete);
        }
    }
    fn call_as_mut<T: 'static>(&self, mut f: impl FnMut(&mut T)) {
        let mut borrow = self.borrow_mut();
        if let Some(concrete) = borrow.as_any_mut().downcast_mut::<T>() {
            f(concrete);
        }
    }
}
