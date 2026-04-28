use std::any::Any;

use crate::window::component::animation::animation_action::AnimationSequence;
use crate::window::component::base::area::Area;
use crate::window::component::base::base::Base;

use crate::window::component::base::component_type::SharedDrawable;
use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::scroll::Scroll;
use crate::window::component::base::settings::Settings;
use crate::window::component::base::ui_command::UiCommand;
use crate::window::component::interface::drawable_3d::ViewportControl;
use crate::window::component::managers::atlas_manager::AtlasManager;
use crate::window::component::managers::button_manager::ButtonManager;
use crate::window::component::managers::drag_manager::{DragManager, DragRails};
use crate::window::component::managers::hover_manager::HoverManager;
use crate::window::component::managers::id_manager::IDManager;
use crate::window::component::managers::scroll_manager::ScrollManager;
use crate::window::component::managers::select_manager::SelectManager;

use crate::window::component::interface::component_control::{
    ComponentControl, FullEditControl, LabelControl, PanelControl,
};
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::layout::const_base_layout::Direction;
use crate::window::component::layout::layout_context::LayoutContext;

pub struct InternalAccess(pub(crate) ());

pub trait ClickableDrawable {
    fn is_clickable(&self) -> bool;
    fn remove_clickable(&mut self) -> &mut dyn ClickableDrawable;
    fn set_on_click(&mut self, action: UiCommand) -> &mut dyn ClickableDrawable;
    fn on_click(&self);
}

pub trait HoverableDrawable {
    fn is_hoverable(&self) -> bool;
    fn set_on_mouse_enter(&mut self, action: UiCommand) -> &mut dyn HoverableDrawable;
    fn set_on_mouse_leave(&mut self, action: UiCommand) -> &mut dyn HoverableDrawable;
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
    fn set_on_scroll(&mut self, cmd: UiCommand) -> &mut dyn ScrollableDrawable;
    fn set_scrolable(&mut self, tumbler: bool) -> &mut dyn ScrollableDrawable;
    fn set_offset(&mut self, x: f32, y: f32, area: &Area);
    fn scroll(&mut self, x: f32, y: f32) -> bool;
}

pub trait AnimationDrawable {
    fn have_animation(&self) -> bool;
    fn set_animation(&mut self, animation: Vec<AnimationSequence>) -> &mut dyn AnimationDrawable;
    fn add_animation(&mut self, animation: AnimationSequence) -> &mut dyn AnimationDrawable;
    fn add_animation_batch(
        &mut self,
        animations: Vec<AnimationSequence>,
    ) -> &mut dyn AnimationDrawable;
    fn start_animation(&mut self);
    fn get_animations(&self) -> &[AnimationSequence];
    fn stop_animations(&mut self);
    fn restart_animations(&mut self);
    fn need_animate(&self) -> bool;
    fn stop_loop_animation(&mut self);
    fn need_animate_loop(&self) -> bool;
    fn fill_ref(&mut self);
}

pub trait LayoutDrawable {
    fn set_padding(&mut self, direction: Direction) -> &mut dyn LayoutDrawable;
    fn set_margin(&mut self, direction: Direction) -> &mut dyn LayoutDrawable;
    fn get_padding(&self) -> &Direction;
    fn get_margin(&self) -> &Direction;
    fn set_const_layout(
        &mut self,
        const_layout: Option<Box<dyn ConstLayout>>,
    ) -> &mut dyn LayoutDrawable;
}

pub trait DragableDrawable {
    fn is_dragable(&self) -> bool;
    fn set_dragable(&mut self, tumbler: bool) -> &mut dyn DragableDrawable;
    fn start_drag(&mut self);
    fn drag(&mut self, mx_offset: f32, my_offset: f32);
    fn stop_drag(&mut self);
    fn set_rails(&mut self, rails: DragRails) -> &mut dyn DragableDrawable;
    fn set_on_drag(&mut self, command: UiCommand) -> &mut dyn DragableDrawable;
    fn set_in_drag(&mut self, command: UiCommand) -> &mut dyn DragableDrawable;
    fn set_on_drop(&mut self, command: UiCommand) -> &mut dyn DragableDrawable;
}

#[allow(dead_code)]
pub trait Drawable: Any {
    fn print(
        &mut self,
        ctx: &mut GpuRenderContext,
        area: &Area,
        level: u32,
        id_parent: u32,
        atlas: &mut AtlasManager,
    );

    fn resize(&mut self, area: &Area, ctx: &LayoutContext, auto_size: bool) -> Area;

    fn resize_one(&mut self, ctx: &LayoutContext) {}

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn set_default_settings(&mut self, settings: &Settings) -> &mut dyn Drawable;

    fn hover(&self, mx: u16, my: u16, area: &Area) -> bool;
    #[allow(unused_variables)]
    fn get_managers<'a>(
        &'a self,
        button_manager: &mut ButtonManager,
        hover_manager: &mut HoverManager,
        select_manager: &mut SelectManager,
        scroll_manager: &mut ScrollManager,
        drag_manager: &mut DragManager,
        id_manager: &mut IDManager,
        token: &InternalAccess,
    ) {
    }

    fn as_base(&self) -> &Base;
    fn as_base_mut(&mut self) -> &mut Base;

    fn as_panel_control(&self) -> &dyn PanelControl;
    fn as_panel_control_mut(&mut self) -> &mut dyn PanelControl;

    // Layout control
    fn as_layout_control(&self) -> &dyn LayoutDrawable;
    fn as_layout_control_mut(&mut self) -> &mut dyn LayoutDrawable;

    fn as_component_control_mut(&mut self) -> Option<&mut dyn ComponentControl> {
        None
    }

    // Label control
    fn as_label_control(&self) -> Option<&dyn LabelControl> {
        None
    }
    fn as_label_control_mut(&mut self) -> Option<&mut dyn LabelControl> {
        None
    }

    // Только mut
    fn as_edit_label_control_mut(&mut self) -> Option<&mut dyn FullEditControl> {
        None
    }

    fn as_clickable(&self) -> Option<&dyn ClickableDrawable> {
        None
    }
    fn as_clickable_mut(&mut self) -> Option<&mut dyn ClickableDrawable> {
        None
    }

    fn as_hoverable(&self) -> Option<&dyn HoverableDrawable> {
        None
    }
    fn as_hoverable_mut(&mut self) -> Option<&mut dyn HoverableDrawable> {
        None
    }

    fn as_selectable(&self) -> Option<&dyn SelectableDrawable> {
        None
    }
    fn as_selectable_mut(&mut self) -> Option<&mut dyn SelectableDrawable> {
        None
    }

    fn as_with_animation(&self) -> Option<&dyn AnimationDrawable> {
        None
    }
    fn as_with_animation_mut(&mut self) -> Option<&mut dyn AnimationDrawable> {
        None
    }

    fn as_scrollable(&self) -> Option<&dyn ScrollableDrawable> {
        None
    }
    fn as_scrollable_mut(&mut self) -> Option<&mut dyn ScrollableDrawable> {
        None
    }

    fn as_dragable(&self) -> Option<&dyn DragableDrawable> {
        None
    }
    fn as_dragable_mut(&mut self) -> Option<&mut dyn DragableDrawable> {
        None
    }

    #[cfg(feature = "3d_render")]
    fn as_viewport_control(&self) -> Option<&dyn ViewportControl> {
        None
    }
    #[cfg(feature = "3d_render")]
    fn as_viewport_control_mut(&mut self) -> Option<&mut dyn ViewportControl> {
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
