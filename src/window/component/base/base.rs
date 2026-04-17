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
    pub id: u32,
    pub rect: Rect<f32, u16>,
    pub visible: bool,
    pub settings: Settings,
    pub self_ref: Option<Weak<RefCell<dyn Drawable>>>,
    pub run_loop_animation: bool,
    pub run_base_animation: bool,
    pub visible_on_this_frame: bool,
}

#[allow(dead_code)]
impl Base {
    pub fn new(rect: Rect<f32, u16>) -> Base {
        Base {
            id: 0,
            rect: rect,
            visible: true,
            settings: get_settings(),
            self_ref: None,
            run_loop_animation: false,
            run_base_animation: false,
            visible_on_this_frame: false,
        }
    }

    #[allow(dead_code)]
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.rect.set_position(x, y);
    }

    pub fn set_height(&mut self, h: u16) {
        let safe_h = h.min(u16::MAX);
        self.rect.set_height(safe_h);
    }

    pub fn set_width(&mut self, w: u16) {
        let safe_w = w.min(u16::MAX);
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
    // pub fn handle(&mut self) {
    //     self.last_interaction = Instant::now();
    // }
}
