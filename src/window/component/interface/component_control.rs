use crate::window::component::{
    base::component_type::SharedDrawable,
    interface::{drawable::Drawable, layout::Layout},
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
    fn set_background(&mut self, color: u32);
}

#[allow(dead_code)]
pub trait LabelControl {
    fn set_font(&mut self, family_name: &'static str);
    fn set_font_color(&mut self, color: u32);
    fn get_font_color(&self) -> u32;
    fn set_text(&mut self, text: String);

    fn set_text_str(&mut self, text: &str);

    fn set_scale(&mut self, scale: u16);
}
