use std::rc::Rc;

use crate::window::component::{
    base::component_type::SharedDrawable, layout::layout_context::LayoutContext,
};

pub struct SelectManager {
    select: bool,
    selected_element: Option<SharedDrawable>,
    items: Vec<SharedDrawable>,
}
impl Default for SelectManager {
    fn default() -> Self {
        SelectManager {
            select: false,
            selected_element: None,
            items: Vec::new(),
        }
    }
}
impl SelectManager {
    pub fn add(&mut self, button: SharedDrawable) {
        self.items.push(button);
    }
    pub fn select_start(&mut self, mx: u16, my: u16, ctx: &LayoutContext) {
        let mut hovered_is_none = self.selected_element.is_none();
        self.select = false;

        if !hovered_is_none {
            if let Some(item) = &self.selected_element {
                if let Some(label) = item.borrow_mut().as_label_control_mut() {
                    label.remove_select();
                    hovered_is_none = true;
                }
            }
        }
        if hovered_is_none {
            self.selected_element = None;
            for item in self.items.iter().rev() {
                let is_hover_item = item.borrow().hover(mx, my);
                if is_hover_item {
                    self.selected_element = Some(Rc::clone(item));
                    if let Some(label) = &self.selected_element {
                        if let Some(label) = label.borrow_mut().as_label_control_mut() {
                            label.set_start_caret((mx, my), ctx);
                            self.select = true;
                        }
                    }
                    break;
                }
            }
        }
    }
    pub fn select(&mut self, mx: u16, my: u16, ctx: &LayoutContext) {
        if self.select {
            if let Some(item) = &self.selected_element {
                let is_still_over = item.borrow().hover(mx, my);
                if is_still_over {
                    if let Some(label) = &self.selected_element {
                        if let Some(label) = label.borrow_mut().as_label_control_mut() {
                            label.set_end_caret((mx, my), ctx);
                        }
                    }
                }
            }
        }
    }
    pub fn stop_select(&mut self) {
        self.select = false;
    }
}
