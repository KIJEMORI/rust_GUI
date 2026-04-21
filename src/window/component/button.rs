use crate::add_drawable_control;

use crate::window::component::base::area::Rect;
use crate::window::component::base::base::Base;

use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::ui_command::UiCommand;
use crate::window::component::interface::component_control::{LabelControl, PanelControl};

use crate::window::component::interface::drawable::{
    AnimationDrawable, ClickableDrawable, Drawable, HoverableDrawable, LayoutDrawable,
    ScrollableDrawable, SelectableDrawable,
};
use crate::window::component::label::Label;

use crate::window::component::layout::layout_context::LayoutContext;

pub struct Button {
    label: Label,
}

#[allow(dead_code)]
impl Button {
    pub fn new(text: &str, action: UiCommand) -> Self {
        let mut label = Label::new(text.to_string());
        label.as_clickable_mut().unwrap().set_on_click(action);

        Button { label: label }
    }
}

impl Drawable for Button {
    fn print(
        &mut self,
        ctx: &mut GpuRenderContext,
        area: &Rect<f32, u16>,
        level: u32,
        id_parent: u32,
    ) {
        self.label.print(ctx, area, level, id_parent);
    }
    fn resize(
        &mut self,
        area: &Rect<f32, u16>,
        ctx: &LayoutContext,
        auto_size: bool,
    ) -> Rect<f32, u16> {
        self.label.resize(area, ctx, auto_size)
    }
    fn resize_one(&mut self, ctx: &LayoutContext) {
        self.label.resize_one(ctx);
    }

    add_drawable_control!();

    fn set_default_settings(
        &mut self,
        settings: &super::base::settings::Settings,
    ) -> &mut dyn Drawable {
        self.label.set_default_settings(settings);
        self
    }

    fn hover(&self, mx: u16, my: u16, area: &Rect<f32, u16>) -> bool {
        self.label.hover(mx, my, area)
    }
    fn as_panel_control(&self) -> &dyn PanelControl {
        self.label.as_panel_control()
    }

    fn as_layout_control(&self) -> &dyn LayoutDrawable {
        self.label.as_layout_control()
    }
    fn as_layout_control_mut(&mut self) -> &mut dyn LayoutDrawable {
        self.label.as_layout_control_mut()
    }

    fn as_panel_control_mut(&mut self) -> &mut dyn PanelControl {
        self.label.as_panel_control_mut()
    }
    fn as_label_control_mut(&mut self) -> Option<&mut dyn LabelControl> {
        self.label.as_label_control_mut()
    }
    fn as_base(&self) -> &Base {
        self.label.as_base()
    }
    fn as_base_mut(&mut self) -> &mut Base {
        self.label.as_base_mut()
    }
    fn as_clickable(&self) -> Option<&dyn ClickableDrawable> {
        self.label.as_clickable()
    }
    fn as_clickable_mut(&mut self) -> Option<&mut dyn ClickableDrawable> {
        self.label.as_clickable_mut()
    }

    fn as_hoverable(&self) -> Option<&dyn HoverableDrawable> {
        self.label.as_hoverable()
    }
    fn as_hoverable_mut(&mut self) -> Option<&mut dyn HoverableDrawable> {
        self.label.as_hoverable_mut()
    }

    fn as_selectable(&self) -> Option<&dyn SelectableDrawable> {
        self.label.as_selectable()
    }
    fn as_selectable_mut(&mut self) -> Option<&mut dyn SelectableDrawable> {
        self.label.as_selectable_mut()
    }

    fn as_with_animation(&self) -> Option<&dyn AnimationDrawable> {
        self.label.as_with_animation()
    }
    fn as_with_animation_mut(&mut self) -> Option<&mut dyn AnimationDrawable> {
        self.label.as_with_animation_mut()
    }

    fn as_scrollable(&self) -> Option<&dyn ScrollableDrawable> {
        self.label.as_scrollable()
    }
    fn as_scrollable_mut(&mut self) -> Option<&mut dyn ScrollableDrawable> {
        self.label.as_scrollable_mut()
    }
}
