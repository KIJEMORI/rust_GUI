use crate::window::component::base::area::Rect;
use crate::window::component::interface::layout::Layout;
use crate::window::component::layout::const_base_layout::Direction;

pub struct RowLayout {}

#[allow(dead_code)]
impl RowLayout {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

#[allow(dead_code)]
impl Layout for RowLayout {
    fn next(&self, area: &Rect<u16>, parent_area: &Rect<u16>, margin: Direction) -> Rect<u16> {
        let offset_x = margin.left;
        let offset_y = margin.down;

        let x1 = if parent_area.x1 as i32 > offset_x {
            parent_area.x1 - offset_x as u16
        } else {
            0
        };
        let y1 = area.y2 + offset_y as u16;
        let x2 = parent_area.x2;
        let y2 = parent_area.y2;

        Rect::new_from_coord((x1, y1), (x2, y2))
    }
}
