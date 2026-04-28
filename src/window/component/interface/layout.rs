use crate::window::component::{
    base::area::Area,
    interface::const_layout::ConstLayout,
    layout::{base_layout::Align, const_base_layout::Direction},
};

#[allow(dead_code)]
pub trait Layout {
    fn calculate(&self, area: &Area, parent_area: &Area) -> Area;
    fn padding_area(&self, area: &Area) -> Area;
    fn next(&self, area: &Area, parent_area: &Area, margin: Direction) -> (Area, bool);
    fn decrease(&self, area: &Area, parent_area: &Area) -> Area;
    fn set_padding(&mut self, direction: Direction);
    fn set_margin(&mut self, direction: Direction);
    fn get_padding(&self) -> &Direction;
    fn get_margin(&self) -> &Direction;
    fn set_const_layout(&mut self, const_layout: Option<Box<dyn ConstLayout>>);
    fn set_align(&mut self, align: Align);
    fn set_auto_scale(&mut self, tumbler: bool);
    fn is_auto_scale(&self) -> bool;
}
