use std::rc::{Rc, Weak};

use rustc_hash::FxHashMap;

use crate::window::component::base::component_type::{SharedDrawable, WeakSharedDrawable};

pub struct IDManager {
    pub id_map: FxHashMap<u32, WeakSharedDrawable>,
    pub last_id: u32,
}

impl Default for IDManager {
    fn default() -> Self {
        Self {
            id_map: FxHashMap::default(),
            last_id: 0,
        }
    }
}

impl IDManager {
    pub fn register(&mut self, item: SharedDrawable) -> u32 {
        let id = self.last_id;
        item.borrow_mut().as_base_mut().id = id;
        let item = Rc::downgrade(&item);
        self.id_map.entry(id).insert_entry(Weak::clone(&item));
        self.last_id += 1;
        id
    }

    pub fn get_by_id(&self, id: &u32) -> Option<&WeakSharedDrawable> {
        self.id_map.get(&id)
    }

    pub fn get_upgraded(&self, id: &u32) -> Option<SharedDrawable> {
        self.id_map.get(&id).and_then(|weak| weak.upgrade())
    }

    pub fn cleanup(&mut self) {
        self.id_map.retain(|_, weak| weak.strong_count() > 0);
    }
}

pub fn get_upgrade_by_id(id: &Option<u32>, id_manager: &IDManager) -> Option<SharedDrawable> {
    if let Some(id) = id {
        if let Some(el) = id_manager.get_upgraded(id) {
            return Some(el);
        }
    }
    None
}
