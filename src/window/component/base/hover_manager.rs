use std::rc::Rc;

use crate::window::component::base::component_type::SharedDrawable;

pub struct HoverManager {
    hovered_element: Option<SharedDrawable>,
    items: Vec<SharedDrawable>,
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
    pub fn add(&mut self, button: SharedDrawable) {
        self.items.push(button);
    }
    pub fn hover(&mut self, mx: u16, my: u16) {
        let mut hovered_is_none = self.hovered_element.is_none();

        if !hovered_is_none {
            if let Some(item) = &self.hovered_element {
                let is_still_over = item.borrow().hover(mx, my);
                if is_still_over {
                    return;
                } else {
                    item.borrow_mut().on_mouse_leave();
                    hovered_is_none = true;
                }
            }
        }
        if hovered_is_none {
            self.hovered_element = None;
            for item in self.items.iter().rev() {
                let is_hover_item = item.borrow().hover(mx, my);
                if is_hover_item {
                    self.hovered_element = Some(Rc::clone(item));
                    item.borrow_mut().on_mouse_enter();
                    break;
                }
            }
        }
    }
}
