use std::any::Any;

use crate::add_drawable_control;
use crate::window::component::base::area::Rect;
use crate::window::component::base::component_type::{ComponentType, SharedDrawable};
use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::render_context::RenderContext;
use crate::window::component::interface::button_manager_control::ButtonManagerControl;
use crate::window::component::interface::component_control::{LabelControl, PanelControl};
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::drawable::Drawable;
use crate::window::component::label::Label;
use crate::window::component::layout::const_base_layout::Direction;

pub struct ButtonManager {
    buttons: Vec<SharedDrawable>,
}

impl Default for ButtonManager {
    fn default() -> Self {
        ButtonManager {
            buttons: Vec::new(),
        }
    }
}

impl ButtonManagerControl for ButtonManager {
    fn add(&mut self, button: SharedDrawable) {
        self.buttons.push(button);
    }
    fn click(&self, x: u16, y: u16) -> bool {
        for button in self.buttons.iter().rev() {
            if button.borrow().click(x, y) {
                return true;
            }
        }
        false
    }
}

pub struct Button {
    label: Label,
    on_click: Box<dyn Fn()>, // Храним «команду» внутри
}

impl Button {
    pub fn new<F>(text: &str, action: F) -> Self
    where
        F: Fn() + 'static,
    {
        let label = Label::new(text.to_string());

        Button {
            label: label,
            on_click: Box::new(action),
        }
    }

    pub fn set_height(&mut self, h: u16) {
        self.label.set_height(h);
    }
    pub fn set_width(&mut self, w: u16) {
        self.label.set_width(w);
    }
}

impl Drawable for Button {
    fn print(&self, ctx: &mut GpuRenderContext, area: &Rect<u16>) {
        self.label.print(ctx, area);
    }
    fn resize(&mut self, area: &Rect<u16>) -> Rect<u16> {
        self.label.resize(area)
    }
    fn get_type(&self) -> ComponentType {
        ComponentType::Button
    }
    fn click(&self, x: u16, y: u16) -> bool {
        if self.label.panel.click(x, y) {
            (self.on_click)();
            return true;
        }
        false
    }
    add_drawable_control!();
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

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
