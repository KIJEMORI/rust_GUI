use crate::window::component::base::area::Rect;
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::layout::Layout;
use crate::window::component::layout::base_layout::BaseLayout;
use crate::window::component::layout::const_base_layout::Direction;

pub struct RowLayout {
    base: BaseLayout,
}

#[allow(dead_code)]
impl RowLayout {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            base: BaseLayout::default(),
        })
    }
}

#[allow(dead_code)]
impl Layout for RowLayout {
    fn get_margin(&self) -> &Direction {
        self.base.get_margin()
    }
    fn get_padding(&self) -> &Direction {
        self.base.get_margin()
    }
    fn set_margin(&mut self, direction: Direction) {
        self.base.set_margin(direction);
    }
    fn set_padding(&mut self, direction: Direction) {
        self.base.set_padding(direction);
    }
    fn set_const_layout(&mut self, const_layout: Option<Box<dyn ConstLayout>>) {
        self.base.set_const_layout(const_layout);
    }

    fn calculate(&self, area: &Rect<i16>, parent_area: &Rect<i16>) -> Rect<i16> {
        self.base.calculate(area, parent_area)
    }
    fn padding_area(&self, area: &Rect<i16>) -> Rect<i16> {
        self.base.padding_area(area)
    }
    fn next(&self, area: &Rect<i16>, parent_area: &Rect<i16>, margin: Direction) -> Rect<i16> {
        let offset_x = margin.left;
        let offset_y = margin.down;

        let x1 = if parent_area.x1 > offset_x {
            parent_area.x1 - offset_x.max(0)
        } else {
            0
        };
        let y1 = area.y2 + offset_y.max(0);
        let x2 = parent_area.x2;
        let y2 = parent_area.y2;

        Rect::new_from_coord((x1, y1), (x2, y2))
    }
}
