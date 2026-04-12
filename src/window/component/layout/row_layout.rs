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

    fn calculate(&self, area: &Rect<f32, u16>, parent_area: &Rect<f32, u16>) -> Rect<f32, u16> {
        self.base.calculate(area, parent_area)
    }
    fn padding_area(&self, area: &Rect<f32, u16>) -> Rect<f32, u16> {
        self.base.padding_area(area)
    }
    fn next(
        &self,
        area: &Rect<f32, u16>,
        parent_area: &Rect<f32, u16>,
        margin: Direction,
    ) -> (Rect<f32, u16>, bool) {
        let offset_x = margin.left;
        let offset_y = margin.down;

        let x1 = if parent_area.x1 > offset_x as f32 {
            parent_area.x1 - offset_x.max(0) as f32
        } else {
            0.0
        };
        let y1 = area.get_y2() + offset_y.max(0) as f32;
        let x2 = parent_area.get_x2();
        let y2 = parent_area.get_y2();

        (Rect::new_from_coord((x1, y1), (x2, y2)), y1 < y2)
    }
    fn decrease(&self, area: &Rect<f32, u16>, parent_area: &Rect<f32, u16>) -> Rect<f32, u16> {
        self.base.decrease(area, parent_area)
    }
}
