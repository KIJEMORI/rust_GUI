use std::any::Any;

use crate::window::component::animation::animation_action::AnimationSequence;
use crate::window::component::base::area::Rect;
use crate::window::component::base::base::Base;

use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::scroll::Scroll;
use crate::window::component::base::settings::Settings;
use crate::window::component::base::ui_command::UiCommand;
use crate::window::component::managers::button_manager::ButtonManager;
use crate::window::component::managers::hover_manager::HoverManager;
use crate::window::component::managers::scroll_manager::ScrollManager;
use crate::window::component::managers::select_manager::SelectManager;

use crate::window::component::interface::component_control::{
    FullEditControl, LabelControl, PanelControl,
};
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::layout::const_base_layout::Direction;
use crate::window::component::layout::layout_context::LayoutContext;

pub struct InternalAccess(pub(crate) ());

pub trait ClickableDrawable {
    fn is_clickable(&self) -> bool;
    fn remove_clickable(&mut self);
    fn set_on_click(&mut self, action: UiCommand);
    fn on_click(&self);
}

pub trait HoverableDrawable {
    fn is_hoverable(&self) -> bool;
    fn set_on_mouse_enter(&mut self, action: UiCommand);
    fn set_on_mouse_leave(&mut self, action: UiCommand);
    fn on_mouse_enter(&self);
    fn on_mouse_leave(&self);
}

pub trait SelectableDrawable {
    fn is_selectable(&self) -> bool {
        false
    }
}

pub trait ScrollableDrawable {
    fn is_scrollable(&self) -> bool;
    fn set_scrolable(&mut self);
    fn set_offset(&mut self, x: f32, y: f32, area: &Rect<f32, u16>);
    fn remove_scrolable(&mut self);
    fn scroll(&mut self, x: f32, y: f32);
}

pub trait AnimationDrawable {
    fn have_animation(&self) -> bool;
    fn set_animation(&mut self, animation: Vec<AnimationSequence>);
    fn add_animation(&mut self, animation: AnimationSequence);
    fn add_animation_batch(&mut self, animations: Vec<AnimationSequence>);
    fn start_animation(&mut self);
    fn get_animations(&self) -> &[AnimationSequence];
    fn stop_animations(&mut self);
    fn restart_animations(&mut self);
    fn need_animate(&self) -> bool;
    fn stop_loop_animation(&mut self);
    fn need_animate_loop(&self) -> bool;
    fn fill_ref(&mut self);
}

#[allow(dead_code)]
pub trait Drawable: Any {
    fn print(&self, ctx: &mut GpuRenderContext, area: &Rect<f32, u16>, level: u32);
    fn resize(
        &mut self,
        area: &Rect<f32, u16>,
        ctx: &LayoutContext,
        scroll_item: bool,
    ) -> Rect<f32, u16>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn set_padding(&mut self, direction: Direction);
    fn set_margin(&mut self, direction: Direction);
    fn get_padding(&self) -> &Direction;
    fn get_margin(&self) -> &Direction;
    fn set_const_layout(&mut self, const_layout: Option<Box<dyn ConstLayout>>);

    fn set_default_settings(&mut self, settings: &Settings);

    fn under(&self, mx: u16, my: u16) -> bool;
    fn hover(&self, mx: u16, my: u16) -> bool;
    #[allow(unused_variables)]
    fn get_managers<'a>(
        &'a self,
        button_manager: &mut ButtonManager,
        hover_manager: &mut HoverManager,
        select_manager: &mut SelectManager,
        scroll_manager: &mut ScrollManager,
        token: &InternalAccess,
    ) {
    }

    fn as_base(&self) -> &Base;
    fn as_base_mut(&mut self) -> &mut Base;
    fn as_panel_control_mut(&mut self) -> Option<&mut dyn PanelControl> {
        None
    }

    fn as_label_control_mut(&mut self) -> Option<&mut dyn LabelControl> {
        None
    }
    fn as_edit_label_control_mut(&mut self) -> Option<&mut dyn FullEditControl> {
        None
    }
    fn as_clickable(&mut self) -> Option<&mut dyn ClickableDrawable> {
        None
    }
    fn as_hoverable(&mut self) -> Option<&mut dyn HoverableDrawable> {
        None
    }
    fn as_selectable(&mut self) -> Option<&mut dyn SelectableDrawable> {
        None
    }
    fn as_with_animation(&mut self) -> Option<&mut dyn AnimationDrawable> {
        None
    }
    fn as_scrollable(&mut self) -> Option<&mut dyn ScrollableDrawable> {
        None
    }
}

#[macro_export]
macro_rules! add_drawable_control {
    () => {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    };
}
