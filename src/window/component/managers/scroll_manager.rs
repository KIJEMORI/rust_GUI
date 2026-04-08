use std::rc::{Rc, Weak};

use crate::window::component::base::component_type::{SharedDrawable, WeakSharedDrawable};

pub struct ScrollManager {
    items: Vec<WeakSharedDrawable>,
}

impl Default for ScrollManager {
    fn default() -> Self {
        ScrollManager { items: Vec::new() }
    }
}

impl ScrollManager {
    pub fn add(&mut self, item: SharedDrawable) {
        let item = Rc::downgrade(&item);
        if !self.items.iter().any(|x| Weak::ptr_eq(x, &item)) {
            self.items.push(item);
        }
    }
    pub fn scroll(&mut self, mx: u16, my: u16, x: f32, y: f32) -> bool {
        for item in self.items.iter().rev() {
            if let Some(rc_item) = item.upgrade() {
                let mut item = rc_item.borrow_mut();
                if item.hover(mx, my) {
                    if let Some(scrollable) = item.as_scrollable() {
                        return scrollable.scroll(x, y);
                    }
                    break;
                }
            }
        }

        self.items.retain(|x| x.strong_count() > 0);
        false
    }
}
