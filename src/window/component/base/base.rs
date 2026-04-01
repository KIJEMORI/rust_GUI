use std::{cell::RefCell, rc::Weak};

use crate::window::component::{
    base::{
        area::Rect,
        component_type::SharedDrawable,
        settings::{Settings, get_settings},
    },
    interface::drawable::Drawable,
};
pub struct Base {
    pub id: String,
    pub rect: Rect<i16>,
    pub visible: bool,
    pub settings: Settings,
    pub self_ref: Option<Weak<RefCell<dyn Drawable>>>,
}

impl Base {
    pub fn new(id: String, rect: Rect<i16>) -> Base {
        Base {
            id: id,
            rect: rect,
            visible: true,
            settings: get_settings(),
            self_ref: None,
        }
    }

    #[allow(dead_code)]
    pub fn set_position(&mut self, x: i16, y: i16) {
        self.rect.set_position(x, y);
    }

    pub fn set_height(&mut self, h: u16) {
        let safe_h = h.min(i16::MAX as u16) as i16;
        self.rect.set_height(safe_h);
    }

    pub fn set_width(&mut self, w: u16) {
        let safe_w = w.min(i16::MAX as u16) as i16;
        self.rect.set_width(safe_w);
    }

    pub fn set_settings(&mut self, settings: Settings) {
        self.settings = settings;
    }

    pub fn get_shared(&self) -> SharedDrawable {
        self.self_ref
                .as_ref()
                .and_then(|w| w.upgrade())
                .expect("Ошибка: Попытка использовать self_ref до того, как компонент был добавлен в систему (app.add)")
    }
}
