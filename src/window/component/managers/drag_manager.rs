use crate::window::component::managers::id_manager::IDManager;

pub enum DragRails {
    Vertical,
    Horizontal,
    None,
}

pub struct DragManager {
    select: bool,
    drag: bool,
    drag_element: Option<u32>,
    items: Vec<u32>,
    last_mouse_position: (u16, u16),
}
impl Default for DragManager {
    fn default() -> Self {
        DragManager {
            select: false,
            drag: false,
            drag_element: None,
            items: Vec::new(),
            last_mouse_position: (0, 0),
        }
    }
}
impl DragManager {
    pub fn add(&mut self, item: u32) {
        if !self.items.iter().any(|x| x == &item) {
            self.items.push(item);
        }
    }
    pub fn drag_start(&mut self, mx: u16, my: u16, id_manager: &IDManager) {
        let mut hovered_is_none = self.drag_element.is_none();
        self.select = false;
        self.drag = false;
        let mut item_removed = false;

        if !hovered_is_none {
            if let Some(id) = &self.drag_element {
                if let Some(item) = id_manager.get_upgraded(id) {
                    if let Some(label) = item.borrow_mut().as_dragable_mut() {
                        label.stop_drag();
                        hovered_is_none = true;
                    }
                } else {
                    item_removed = true;
                    hovered_is_none = true;
                }
            }
        }
        if hovered_is_none {
            self.drag_element = None;
            for id in self.items.iter().rev() {
                if let Some(item) = id_manager.get_upgraded(id) {
                    let parent_id = item.borrow().as_base().id_parent;
                    if let Some(parent) = id_manager.get_upgraded(&parent_id) {
                        let rect = item.borrow().as_base().parent_rect.clone();
                        let area = parent
                            .borrow()
                            .as_panel_control()
                            .get_rect_without_offset(&rect);
                        let is_hover_item = item.borrow().hover(mx, my, &area);
                        if is_hover_item {
                            self.drag_element = Some(id.clone());

                            self.select = true;
                            self.last_mouse_position = (mx, my);
                            if let Some(dragable) = item.borrow_mut().as_dragable_mut() {
                                dragable.start_drag();
                            }

                            break;
                        }
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
    pub fn drag(&mut self, mx: u16, my: u16, id_manager: &IDManager) -> bool {
        if self.select {
            if let Some(id) = &self.drag_element {
                if let Some(item) = id_manager.get_upgraded(id) {
                    if let Some(item) = item.borrow_mut().as_dragable_mut() {
                        let x_offset = mx as f32 - self.last_mouse_position.0 as f32;
                        let y_offset = my as f32 - self.last_mouse_position.1 as f32;
                        item.drag(x_offset, y_offset);
                        self.last_mouse_position = (mx, my);
                        return true;
                    }
                } else {
                    self.select = false;
                    self.drag_element = None
                }
            }
        }
        false
    }
    pub fn stop_drag(&mut self, id_manager: &IDManager) {
        let hovered_is_none = self.drag_element.is_none();
        self.select = false;
        self.drag = false;
        self.drag_element = None;
        let mut item_removed = false;

        if !hovered_is_none {
            if let Some(id) = &self.drag_element {
                if let Some(item) = id_manager.get_upgraded(id) {
                    if let Some(label) = item.borrow_mut().as_dragable_mut() {
                        label.stop_drag();
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
