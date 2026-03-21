use crate::window::component::base::area::Rect;
use crate::window::component::interface::layout::Layout;
use crate::window::component::layout::const_base_layout::Direction;

pub struct BaseLayout {}

#[allow(dead_code)]
impl BaseLayout {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl Layout for BaseLayout {
    fn next(&self, _area: &Rect<u16>, parent_area: &Rect<u16>, _margin: Direction) -> Rect<u16> {
        Rect::new(
            parent_area.x1,
            parent_area.y1,
            parent_area.min.get_width(),
            parent_area.min.get_height(),
        )
    }
}
