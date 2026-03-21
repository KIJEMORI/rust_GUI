use std::cell::RefCell;
use std::rc::Rc;

use crate::window::component::interface::drawable::Drawable;

#[derive(PartialEq, Eq)]
pub enum ComponentType {
    Panel,
    Button,
    Label,
    _Custom(String),
}

pub type SharedDrawable = Rc<RefCell<dyn Drawable>>;

pub trait SharedDrawableExt {
    fn call_as<T: 'static>(&self, f: impl FnMut(&mut T));
}

impl SharedDrawableExt for SharedDrawable {
    fn call_as<T: 'static>(&self, mut f: impl FnMut(&mut T)) {
        let mut borrow = self.borrow_mut();
        if let Some(concrete) = borrow.as_any_mut().downcast_mut::<T>() {
            f(concrete);
        }
    }
}
