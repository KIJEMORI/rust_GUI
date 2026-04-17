use winit::keyboard::SmolStr;

use crate::window::component::{
    base::component_type::SharedDrawable,
    interface::{drawable::Drawable, layout::Layout},
    layout::layout_context::LayoutContext,
};

#[allow(dead_code)]
pub trait ComponentControl {
    fn add<T: Drawable + 'static>(&mut self, item: T) -> SharedDrawable;
    fn remove_by_index(&mut self, index: u32) -> Result<(), &'static str>;
    fn remove_item(&mut self, item: SharedDrawable);
    fn set_layout(&mut self, layout: Box<dyn Layout>);
}

#[allow(dead_code)]
pub trait PanelControl {
    fn set_background(&mut self, color: u32) -> &mut dyn PanelControl;
    fn set_position(&mut self, x: f32, y: f32) -> &mut dyn PanelControl;
    fn set_height(&mut self, h: u16) -> &mut dyn PanelControl;
    fn set_width(&mut self, w: u16) -> &mut dyn PanelControl;
}

#[allow(dead_code)]
pub trait LabelControl {
    fn set_font(&mut self, family_name: &'static str);
    fn set_font_color(&mut self, color: u32);
    fn get_font_color(&self) -> u32;
    fn set_text(&mut self, text: String);
    fn get_text(&self) -> &str;
    fn set_text_str(&mut self, text: &str);
    fn set_scale(&mut self, scale: u16);
    fn set_start_caret(&mut self, select_start: (u16, u16), ctx: &LayoutContext);
    fn set_end_caret(&mut self, select_end: (u16, u16), ctx: &LayoutContext) -> bool;
    fn remove_select(&mut self);
}

#[allow(dead_code)]
pub trait EditLabelControl {
    fn is_editable(&self) -> bool;
    fn set_cursor(&mut self);
    fn on_cursor(&mut self);
    fn delete_cursor(&mut self);
    fn move_cursor_right(&mut self, right: bool);
    fn add_text(&mut self, text: &SmolStr);
    fn backspace(&mut self);
    fn delete(&mut self);
    fn sync_indexes(&mut self);
    fn get_byte_offset(&self, char_idx: u32) -> usize;
    fn delete_selection(&mut self);
}

pub trait FullEditControl: LabelControl + EditLabelControl {}
