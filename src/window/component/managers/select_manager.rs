use crate::window::component::{
    layout::layout_context::LayoutContext, managers::id_manager::IDManager,
};

pub struct SelectManager {
    select: bool,
    selected_element: Option<u32>,
    items: Vec<u32>,
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
    pub fn add(&mut self, item: u32) {
        if !self.items.iter().any(|x| x == &item) {
            self.items.push(item);
        }
    }
    pub fn select_start(&mut self, mx: u16, my: u16, ctx: &LayoutContext, id_manager: &IDManager) {
        let mut hovered_is_none = self.selected_element.is_none();
        self.select = false;
        let mut item_removed = false;

        if !hovered_is_none {
            if let Some(id) = &self.selected_element {
                if let Some(item) = id_manager.get_upgraded(id) {
                    if let Some(label) = item.borrow_mut().as_label_control_mut() {
                        label.remove_select();
                        hovered_is_none = true;
                    }
                } else {
                    item_removed = true;
                    hovered_is_none = true;
                }
            }
        }
        if hovered_is_none {
            self.selected_element = None;
            for id in self.items.iter().rev() {
                if let Some(item) = id_manager.get_upgraded(id) {
                    let is_hover_item = item.borrow().hover(mx, my);
                    if is_hover_item {
                        self.selected_element = Some(id.clone());

                        if let Some(label) = item.borrow_mut().as_label_control_mut() {
                            label.set_start_caret((mx, my), ctx);
                            self.select = true;
                        }

                        break;
                    }
                } else {
                    item_removed = true;
                }
            }
        }
        if item_removed {
            self.items
                .retain(|id| id_manager.get_upgraded(id).is_some());
        }
    }
    pub fn select(
        &mut self,
        mx: u16,
        my: u16,
        ctx: &LayoutContext,
        id_manager: &IDManager,
    ) -> bool {
        if self.select {
            if let Some(id) = &self.selected_element {
                if let Some(item) = id_manager.get_upgraded(id) {
                    let is_still_over = item.borrow().hover(mx, my);
                    if is_still_over {
                        if let Some(label) = item.borrow_mut().as_label_control_mut() {
                            return label.set_end_caret((mx, my), ctx);
                        }
                    }
                } else {
                    self.selected_element = None
                }
            }
        }
        false
    }
    pub fn stop_select(&mut self) {
        self.select = false;
    }
}
