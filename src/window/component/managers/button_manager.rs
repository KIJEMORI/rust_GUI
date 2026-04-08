use std::rc::{Rc, Weak};

use crate::window::component::base::component_type::{SharedDrawable, WeakSharedDrawable};

pub struct ButtonManager {
    items: Vec<WeakSharedDrawable>,
}

impl Default for ButtonManager {
    fn default() -> Self {
        ButtonManager { items: Vec::new() }
    }
}

impl ButtonManager {
    pub fn add(&mut self, item: SharedDrawable) {
        let item = Rc::downgrade(&item);
        if !self.items.iter().any(|x| Weak::ptr_eq(x, &item)) {
            self.items.push(item);
        }
    }
    pub fn click(&mut self, mx: u16, my: u16) {
        for item in self.items.iter().rev() {
            if let Some(rc_item) = item.upgrade() {
                let mut item = rc_item.borrow_mut();
                if item.hover(mx, my) {
                    if let Some(clickable) = item.as_clickable() {
                        clickable.on_click();
                    }
                    break;
                }
            }
        }
        self.items.retain(|x| x.strong_count() > 0);
    }
}
