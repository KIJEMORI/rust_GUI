use crate::window::component::managers::id_manager::IDManager;

pub struct ButtonManager {
    items: Vec<u32>,
}

impl Default for ButtonManager {
    fn default() -> Self {
        ButtonManager { items: Vec::new() }
    }
}

impl ButtonManager {
    pub fn add(&mut self, item: u32) {
        if !self.items.iter().any(|x| x == &item) {
            self.items.push(item);
        }
    }
    pub fn click(&mut self, mx: u16, my: u16, id_manager: &IDManager) {
        let mut item_removed = false;
        for id in self.items.iter().rev() {
            if let Some(rc_item) = id_manager.get_upgraded(id) {
                let item = rc_item.borrow();
                let parent_id = item.as_base().id_parent;
                if let Some(parent) = id_manager.get_upgraded(&parent_id) {
                    let rect = item.as_base().parent_rect.clone();
                    let area = parent
                        .borrow()
                        .as_panel_control()
                        .get_rect_without_offset(&rect);
                    if item.hover(mx, my, &area) {
                        if let Some(clickable) = item.as_clickable() {
                            clickable.on_click();
                        }
                        break;
                    }
                }
            } else {
                item_removed = true;
            }
        }
        if item_removed {
            self.items
                .retain(|id| id_manager.get_upgraded(id).is_some());
        }
    }
}
