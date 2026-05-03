use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};

use crate::window::component::interface::drawable::Drawable;

pub type SharedDrawable = Rc<RefCell<dyn Drawable>>;

#[allow(dead_code)]
pub trait SharedDrawableExt {
    fn call_as<T: 'static>(&self) -> Option<Ref<'_, T>>;
    fn call_as_mut<T: 'static>(&self) -> Option<RefMut<'_, T>>;
}

pub type WeakSharedDrawable = Weak<RefCell<dyn Drawable>>;

impl SharedDrawableExt for SharedDrawable {
    fn call_as<T: 'static>(&self) -> Option<Ref<'_, T>> {
        let borrow = self.borrow();
        if borrow.as_any().is::<T>() {
            Some(Ref::map(borrow, |b| {
                b.as_any().downcast_ref::<T>().unwrap()
            }))
        } else {
            None
        }
    }

    fn call_as_mut<T: 'static>(&self) -> Option<RefMut<'_, T>> {
        let borrow = self.borrow_mut();
        if borrow.as_any().is::<T>() {
            Some(RefMut::map(borrow, |b| {
                b.as_any_mut().downcast_mut::<T>().unwrap()
            }))
        } else {
            None
        }
    }
}
