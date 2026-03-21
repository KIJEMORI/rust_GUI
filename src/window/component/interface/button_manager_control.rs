use crate::window::component::base::component_type::SharedDrawable;

pub trait ButtonManagerControl {
    fn add(&mut self, button: SharedDrawable);
    fn click(&self, x: u16, y: u16) -> bool;
}
