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
                if item.hover(mx, my) {
                    if let Some(clickable) = item.as_clickable() {
                        clickable.on_click();
                    }
                    break;
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
