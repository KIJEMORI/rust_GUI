use std::any::Any;

use crate::window::component::base::area::Rect;
use crate::window::component::base::component_type::ComponentType;
use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::render_context::RenderContext;
use crate::window::component::button::ButtonManager;
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::layout::const_base_layout::Direction;

pub struct InternalAccess(pub(crate) ());

#[allow(dead_code)]
pub trait Drawable: Any {
    fn print(&self, ctx: &mut GpuRenderContext, area: &Rect<u16>);
    fn resize(&mut self, area: &Rect<u16>) -> Rect<u16>;
    fn get_type(&self) -> ComponentType;
    fn click(&self, x: u16, y: u16) -> bool;
    #[doc(hidden)]
    fn get_button_manager(&self, button_manager: &mut ButtonManager, token: &InternalAccess);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn set_padding(&mut self, direction: Direction);
    fn set_margin(&mut self, direction: Direction);
    fn get_padding(&self) -> &Direction;
    fn get_margin(&self) -> &Direction;
    fn set_const_layout(&mut self, const_layout: &dyn ConstLayout);
}

#[macro_export]
macro_rules! add_drawable_control {
    () => {
        fn get_button_manager(
            &self,
            _button_manager: &mut $crate::window::component::button::ButtonManager,
            _token: &$crate::window::component::interface::drawable::InternalAccess,
        ) {
        }
    };
}
