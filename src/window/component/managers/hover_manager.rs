use crate::window::component::managers::id_manager::IDManager;

pub struct HoverManager {
    hovered_element: Option<u32>,
    items: Vec<u32>,
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
    pub fn add(&mut self, item: u32) {
        if !self.items.iter().any(|x| x == &item) {
            self.items.push(item);
        }
    }
    pub fn hover(&mut self, mx: u16, my: u16, id_manager: &IDManager) {
        let mut hovered_is_none = self.hovered_element.is_none();

        let mut item_removed = false;

        if !hovered_is_none {
            if let Some(id) = &self.hovered_element {
                if let Some(item) = id_manager.get_upgraded(id) {
                    let item = item.borrow();
                    if item.hover(mx, my) {
                        return;
                    } else {
                        if let Some(hoverable) = item.as_hoverable() {
                            hoverable.on_mouse_leave();
                        }
                        hovered_is_none = true;
                    }
                } else {
                    hovered_is_none = true;
                    item_removed = true;
                }
            }
        }
        if hovered_is_none {
            self.hovered_element = None;
            for id in self.items.iter().rev() {
                if let Some(item) = id_manager.get_upgraded(id) {
                    let item = item.borrow();
                    if item.hover(mx, my) {
                        self.hovered_element = Some(id.clone());
                        if let Some(hoverable) = item.as_hoverable() {
                            hoverable.on_mouse_enter();
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
}
