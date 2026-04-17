use crate::window::component::managers::id_manager::IDManager;

pub struct ScrollManager {
    items: Vec<u32>,
}

impl Default for ScrollManager {
    fn default() -> Self {
        ScrollManager { items: Vec::new() }
    }
}

impl ScrollManager {
    pub fn add(&mut self, item: u32) {
        if !self.items.iter().any(|x| x == &item) {
            self.items.push(item);
        }
    }
    pub fn scroll(&mut self, mx: u16, my: u16, x: f32, y: f32, id_manager: &IDManager) -> bool {
        let mut item_removed = false;
        for id in self.items.iter().rev() {
            if let Some(rc_item) = id_manager.get_upgraded(id) {
                let mut item = rc_item.borrow_mut();
                if item.hover(mx, my) {
                    if let Some(scrollable) = item.as_scrollable_mut() {
                        if scrollable.scroll(x, y) {
                            return true;
                        }
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
        false
    }
}
