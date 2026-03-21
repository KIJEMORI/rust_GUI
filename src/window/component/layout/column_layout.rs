use crate::window::component::base::area::Rect;
use crate::window::component::interface::layout::Layout;
use crate::window::component::layout::const_base_layout::Direction;

#[allow(dead_code)]
pub struct ColumnLayout {}

#[allow(dead_code)]
impl ColumnLayout {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl Layout for ColumnLayout {
    fn next(&self, area: &Rect<u16>, parent_area: &Rect<u16>, margin: Direction) -> Rect<u16> {
        let offset_x = margin.right;
        let offset_y = margin.up;
        let x1 = area.x2 + offset_x as u16;
        let y1 = parent_area.y1 - offset_y as u16;
        let x2 = parent_area.x2;
        let y2 = parent_area.y2;

        Rect::new_from_coord((x1, y1), (x2, y2))
    }
}
