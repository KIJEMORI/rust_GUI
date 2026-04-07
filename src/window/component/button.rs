use std::any::Any;
use std::rc::Rc;

use crate::add_drawable_control;
use crate::window::component::animation::animation_action::AnimationSequence;
use crate::window::component::base::area::Rect;
use crate::window::component::base::base::Base;
use crate::window::component::base::component_type::SharedDrawable;
use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::ui_command::UiCommand;
use crate::window::component::interface::component_control::{LabelControl, PanelControl};
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::drawable::{
    AnimationDrawable, ClickableDrawable, Drawable,
};
use crate::window::component::label::Label;
use crate::window::component::layout::const_base_layout::Direction;
use crate::window::component::layout::layout_context::LayoutContext;

pub struct ButtonManager {
    items: Vec<SharedDrawable>,
}

impl Default for ButtonManager {
    fn default() -> Self {
        ButtonManager { items: Vec::new() }
    }
}

impl ButtonManager {
    pub fn add(&mut self, item: SharedDrawable) {
        if !self.items.iter().any(|x| Rc::ptr_eq(x, &item)) {
            self.items.push(item);
        }
    }
    pub fn click(&self, mx: u16, my: u16) {
        for item in self.items.iter().rev() {
            let is_hover_item = item.borrow().hover(mx, my);
            if is_hover_item {
                if let Some(clickable) = item.borrow_mut().as_clickable() {
                    clickable.on_click();
                }
                break;
            }
        }
    }
}

pub struct Button {
    label: Label,
}

#[allow(dead_code)]
impl Button {
    pub fn new(text: &str, action: UiCommand) -> Self {
        let mut label = Label::new(text.to_string());
        label.set_on_click(action);

        Button { label: label }
    }

    pub fn set_height(&mut self, h: u16) {
        self.label.set_height(h);
    }
    pub fn set_width(&mut self, w: u16) {
        self.label.set_width(w);
    }
}

impl Drawable for Button {
    fn print(&self, ctx: &mut GpuRenderContext, area: &Rect<i16>) {
        self.label.print(ctx, area);
    }
    fn resize(&mut self, area: &Rect<i16>, ctx: &LayoutContext) -> Rect<i16> {
        self.label.resize(area, ctx)
    }

    add_drawable_control!();

    fn set_padding(&mut self, direction: Direction) {
        self.label.set_padding(direction);
    }
    fn set_margin(&mut self, direction: Direction) {
        self.label.set_margin(direction);
    }
    fn set_const_layout(&mut self, const_layout: Option<Box<dyn ConstLayout>>) {
        self.label.set_const_layout(const_layout);
    }
    fn get_margin(&self) -> &Direction {
        self.label.get_margin()
    }
    fn get_padding(&self) -> &Direction {
        self.label.get_padding()
    }
    fn set_default_settings(&mut self, settings: &super::base::settings::Settings) {
        self.label.set_default_settings(settings);
    }
    fn hover(&self, mx: u16, my: u16) -> bool {
        self.label.hover(mx, my)
    }
    fn as_panel_control_mut(&mut self) -> Option<&mut dyn PanelControl> {
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
    fn as_clickable(&mut self) -> Option<&mut dyn super::interface::drawable::ClickableDrawable> {
        self.label.as_clickable()
    }
    fn as_hoverable(&mut self) -> Option<&mut dyn super::interface::drawable::HoverableDrawable> {
        self.label.as_hoverable()
    }
    fn as_selectable(&mut self) -> Option<&mut dyn super::interface::drawable::SelectableDrawable> {
        self.label.as_selectable()
    }
    fn as_with_animation(&mut self) -> Option<&mut dyn AnimationDrawable> {
        self.label.as_with_animation()
    }
}
