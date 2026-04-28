use crate::window::component::base::area::{Area, AreaMath};
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

    fn calculate(&self, area: &Area, parent_area: &Area) -> Area {
        self.base.calculate(area, parent_area)
    }
    fn padding_area(&self, area: &Area) -> Area {
        self.base.padding_area(area)
    }
    fn next(&self, area: &Area, parent_area: &Area, margin: Direction) -> (Area, bool) {
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

        (Area::new_from_coord((x1, y1), (x2, y2)), y1 < y2)
    }
    fn decrease(&self, area: &Area, parent_area: &Area) -> Area {
        self.base.decrease(area, parent_area)
    }
    fn set_align(&mut self, align: super::base_layout::Align) {
        self.base.set_align(align);
    }
    fn set_auto_scale(&mut self, tumbler: bool) {
        self.base.set_auto_scale(tumbler);
    }
    fn is_auto_scale(&self) -> bool {
        self.base.is_auto_scale()
    }
}
