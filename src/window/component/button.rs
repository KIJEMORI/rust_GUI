use std::any::Any;

use crate::add_drawable_control;
use crate::window::component::base::area::Rect;
use crate::window::component::base::base::Base;
use crate::window::component::base::component_type::SharedDrawable;
use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::ui_command::UiCommand;
use crate::window::component::interface::component_control::{LabelControl, PanelControl};
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::drawable::Drawable;
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
    pub fn add(&mut self, button: SharedDrawable) {
        self.items.push(button);
    }
    pub fn click(&self, mx: u16, my: u16) {
        for item in self.items.iter().rev() {
            let is_hover_item = item.borrow().hover(mx, my);
            if is_hover_item {
                item.borrow_mut().on_click();
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

    fn fill_self_ref(&self, cmd: &mut UiCommand) {
        match cmd {
            UiCommand::Batch(cmds) => {
                for c in cmds {
                    self.fill_self_ref(c);
                }
            }
            UiCommand::ChangeColor(target, _)
            | UiCommand::SetText(target, _)
            | UiCommand::SetScale(target, _) => {
                if target.is_none() {
                    *target = Some(self.label.panel.base.get_shared());
                }
            }
            UiCommand::Custom(..) => (),
        }
    }
}

impl Drawable for Button {
    fn print(&self, ctx: &mut GpuRenderContext) {
        self.label.print(ctx);
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
    fn set_const_layout(&mut self, const_layout: &dyn ConstLayout) {
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
    fn is_clickable(&mut self) -> bool {
        self.label.is_clickable()
    }
    fn on_click(&self) {
        self.label.on_click();
    }
    fn set_on_click(&mut self, action: UiCommand) {
        self.label.set_on_click(action);
    }
    fn is_hoverable(&mut self) -> bool {
        self.label.is_hoverable()
    }
    fn hover(&self, mx: u16, my: u16) -> bool {
        self.label.hover(mx, my)
    }
    fn set_on_mouse_enter(&mut self, action: UiCommand) {
        self.label.set_on_mouse_enter(action);
    }
    fn set_on_mouse_leave(&mut self, action: UiCommand) {
        self.label.set_on_mouse_leave(action);
    }
    fn on_mouse_enter(&self) {
        self.label.on_mouse_enter();
    }
    fn on_mouse_leave(&self) {
        self.label.on_mouse_leave();
    }
    fn as_label_control_mut(&mut self) -> Option<&mut dyn LabelControl> {
        Some(self)
    }
    fn as_base(&self) -> &Base {
        self.label.as_base()
    }
    fn as_base_mut(&mut self) -> &mut Base {
        self.label.as_base_mut()
    }
}
impl PanelControl for Button {
    fn set_background(&mut self, color: u32) {
        self.label.set_background(color);
    }
}
impl LabelControl for Button {
    fn set_font(&mut self, family_name: &'static str) {
        self.label.set_font(family_name);
    }
    fn get_font_color(&self) -> u32 {
        self.label.get_font_color()
    }
    fn set_font_color(&mut self, color: u32) {
        self.label.set_font_color(color);
    }
    fn set_text(&mut self, text: String) {
        self.label.set_text(text);
    }
    fn set_text_str(&mut self, text: &str) {
        self.label.set_text_str(&text);
    }
    fn set_scale(&mut self, scale: u16) {
        self.label.set_scale(scale);
    }
}
