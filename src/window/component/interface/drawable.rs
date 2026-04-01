use std::any::Any;

use crate::window::component::base::area::Rect;
use crate::window::component::base::base::Base;
use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::hover_manager::HoverManager;
use crate::window::component::base::settings::Settings;
use crate::window::component::base::ui_command::UiCommand;
use crate::window::component::button::ButtonManager;
use crate::window::component::interface::component_control::LabelControl;
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::layout::const_base_layout::Direction;
use crate::window::component::layout::layout_context::LayoutContext;

pub struct InternalAccess(pub(crate) ());

#[allow(dead_code)]
pub trait Drawable: Any {
    fn print(&self, ctx: &mut GpuRenderContext);
    fn resize(&mut self, area: &Rect<i16>, ctx: &LayoutContext) -> Rect<i16>;
    #[doc(hidden)]
    fn get_button_manager(&self, button_manager: &mut ButtonManager, token: &InternalAccess);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn set_padding(&mut self, direction: Direction);
    fn set_margin(&mut self, direction: Direction);
    fn get_padding(&self) -> &Direction;
    fn get_margin(&self) -> &Direction;
    fn set_const_layout(&mut self, const_layout: &dyn ConstLayout);
    fn set_default_settings(&mut self, settings: &Settings);
    fn is_clickable(&mut self) -> bool;
    fn is_hoverable(&mut self) -> bool;
    fn hover(&self, mx: u16, my: u16) -> bool;
    fn get_hover_manager<'a>(&'a self, hover_manager: &mut HoverManager, token: &InternalAccess);
    fn set_on_click(&mut self, action: UiCommand);
    fn set_on_mouse_enter(&mut self, action: UiCommand);
    fn set_on_mouse_leave(&mut self, action: UiCommand);
    fn on_click(&self);
    fn on_mouse_enter(&self);
    fn on_mouse_leave(&self);
    fn as_label_control_mut(&mut self) -> Option<&mut dyn LabelControl> {
        None
    }
    fn as_base(&self) -> &Base;
    fn as_base_mut(&mut self) -> &mut Base;
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
        fn get_hover_manager(
            &self,
            _hover_manager: &mut $crate::window::component::base::hover_manager::HoverManager,
            _token: &$crate::window::component::interface::drawable::InternalAccess,
        ) {
        }
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    };
}
