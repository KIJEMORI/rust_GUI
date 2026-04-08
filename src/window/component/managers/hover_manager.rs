use std::rc::{Rc, Weak};

use crate::window::component::base::component_type::{SharedDrawable, WeakSharedDrawable};

pub struct HoverManager {
    hovered_element: Option<WeakSharedDrawable>,
    items: Vec<WeakSharedDrawable>,
}

impl Default for HoverManager {
    fn default() -> Self {
        HoverManager {
            hovered_element: None,
            items: Vec::new(),
        }
    }
}

impl HoverManager {
    pub fn add(&mut self, item: SharedDrawable) {
        let item = Rc::downgrade(&item);
        if !self.items.iter().any(|x| Weak::ptr_eq(x, &item)) {
            self.items.push(item);
        }
    }
    pub fn hover(&mut self, mx: u16, my: u16) {
        let mut hovered_is_none = self.hovered_element.is_none();

        if !hovered_is_none {
            if let Some(item) = &self.hovered_element {
                if let Some(item) = item.upgrade() {
                    if item.borrow().hover(mx, my) {
                        return;
                    } else {
                        if let Some(hoverable) = item.borrow_mut().as_hoverable() {
                            hoverable.on_mouse_leave();
                        }
                        hovered_is_none = true;
                    }
                }
            }
        }
        if hovered_is_none {
            self.hovered_element = None;
            for item in self.items.iter().rev() {
                if let Some(item) = item.upgrade() {
                    if item.borrow().hover(mx, my) {
                        self.hovered_element = Some(Rc::downgrade(&item));
                        if let Some(hoverable) = item.borrow_mut().as_hoverable() {
                            hoverable.on_mouse_enter();
                        }
                        break;
                    }
                }
            }
        }

        self.items.retain(|x| x.strong_count() > 0);
    }
}
