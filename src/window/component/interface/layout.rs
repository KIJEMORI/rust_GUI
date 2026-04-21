use crate::window::component::{
    base::area::Rect,
    interface::const_layout::ConstLayout,
    layout::{base_layout::Align, const_base_layout::Direction},
};

#[allow(dead_code)]
pub trait Layout {
    fn calculate(&self, area: &Rect<f32, u16>, parent_area: &Rect<f32, u16>) -> Rect<f32, u16>;
    fn padding_area(&self, area: &Rect<f32, u16>) -> Rect<f32, u16>;
    fn next(
        &self,
        area: &Rect<f32, u16>,
        parent_area: &Rect<f32, u16>,
        margin: Direction,
    ) -> (Rect<f32, u16>, bool);
    fn decrease(&self, area: &Rect<f32, u16>, parent_area: &Rect<f32, u16>) -> Rect<f32, u16>;
    fn set_padding(&mut self, direction: Direction);
    fn set_margin(&mut self, direction: Direction);
    fn get_padding(&self) -> &Direction;
    fn get_margin(&self) -> &Direction;
    fn set_const_layout(&mut self, const_layout: Option<Box<dyn ConstLayout>>);
    fn set_align(&mut self, align: Align);
    fn set_auto_scale(&mut self, tumbler: bool);
    fn is_auto_scale(&self) -> bool;
}
