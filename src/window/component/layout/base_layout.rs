use crate::window::component::base::area::{Area, AreaMath};
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::layout::Layout;
use crate::window::component::layout::const_base_layout::Direction;

pub enum Align {
    FreeRun,
    Base,
}

pub struct BaseLayout {
    margin: Direction,
    padding: Direction,
    const_layout: Option<Box<dyn ConstLayout>>,
    pub align: Align,
    pub auto_scale: bool,
}

impl BaseLayout {
    pub fn new() -> Box<Self> {
        Box::new(Self::default())
    }
}

impl Default for BaseLayout {
    fn default() -> Self {
        Self {
            margin: Direction::default(),
            padding: Direction::default(),
            const_layout: None,
            align: Align::Base,
            auto_scale: true,
        }
    }
}

impl Layout for BaseLayout {
    fn set_margin(&mut self, direction: Direction) {
        self.margin = direction
    }
    fn set_padding(&mut self, direction: Direction) {
        self.padding = direction
    }
    fn get_margin(&self) -> &Direction {
        &self.margin
    }
    fn get_padding(&self) -> &Direction {
        &self.padding
    }
    fn set_const_layout(&mut self, const_layout: Option<Box<dyn ConstLayout>>) {
        self.const_layout = const_layout
    }
    fn calculate(&self, area: &Area, parent_area: &Area) -> Area {
        let mut area = area.clone();

        match self.align {
            Align::Base => {
                area.set_position(
                    (parent_area.x1 + self.margin.left as f32).round(),
                    (parent_area.y1 + self.margin.up as f32).round(),
                );
            }
            Align::FreeRun => {}
        }

        if let Some(const_layout) = &self.const_layout {
            let width = const_layout.as_ref().get_width(
                area.max.get_width() as u16,
                parent_area.min.get_width() as u16,
            );
            area.set_width(width);

            let height = const_layout.as_ref().get_height(
                area.max.get_height() as u16,
                parent_area.min.get_height() as u16,
            );
            area.set_height(height);
        }

        return area;
    }

    fn decrease(&self, area: &Area, parent_area: &Area) -> Area {
        let mut area = area.clone();

        let x_offset = area.get_x_offset() + self.margin.right as f32;
        let parent_x_offset = parent_area.get_x2();

        // Если смещение больше чем данная область отрисовки уменьшаем размер отрисовки текущей структуры
        if x_offset > parent_x_offset {
            area.change_width_on_coord(parent_x_offset - self.margin.right as f32);
        } else {
            area.change_width(area.max.get_width().min(parent_area.min.get_width()));
        }

        // Всё то же самое но для высоты
        let y_offset = area.get_y_offset();
        let parent_y_offset = parent_area.get_y2();

        if y_offset > parent_y_offset {
            area.change_height_on_coord(parent_y_offset);
        } else {
            area.change_height(area.max.get_height().min(parent_area.min.get_height()));
        }

        return area;
    }
    fn padding_area(&self, area: &Area) -> Area {
        let x1 = self.padding.left as f32;
        let y1 = self.padding.up as f32;
        let x2 = area.min.get_width() as f32 - self.padding.right as f32;
        let y2 = area.min.get_height() as f32 - self.padding.down as f32;

        Area::new_from_coord((x1, y1), (x2, y2))
    }
    fn next(&self, _area: &Area, parent_area: &Area, _margin: Direction) -> (Area, bool) {
        (
            Area::new(
                parent_area.x1,
                parent_area.y1,
                parent_area.min.get_width(),
                parent_area.min.get_height(),
            ),
            true,
        )
    }
    fn set_align(&mut self, align: Align) {
        self.align = align;
    }
    fn set_auto_scale(&mut self, tumbler: bool) {
        self.auto_scale = tumbler
    }
    fn is_auto_scale(&self) -> bool {
        self.auto_scale
    }
}
