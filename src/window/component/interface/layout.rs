use crate::window::component::{base::area::Rect, layout::const_base_layout::Direction};
pub trait Layout {
    fn calculate(&self, area: &Rect<i16>, parent_area: &Rect<i16>) -> Rect<i16>;
    fn padding_area(&self, area: &Rect<i16>) -> Rect<i16>;
    fn next(&self, area: &Rect<i16>, parent_area: &Rect<i16>, margin: Direction) -> Rect<i16>;
    fn set_padding(&mut self, direction: Direction);
    fn set_margin(&mut self, direction: Direction);
    fn get_padding(&self) -> &Direction;
    fn get_margin(&self) -> &Direction;
}
