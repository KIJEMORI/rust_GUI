use crate::window::component::{base::area::Rect, layout::const_base_layout::Direction};
pub trait Layout {
    fn next(&self, area: &Rect<u16>, parent_area: &Rect<u16>, margin: Direction) -> Rect<u16>;
}
