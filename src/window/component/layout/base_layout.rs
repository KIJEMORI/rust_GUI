use crate::window::component::base::area::Rect;
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::layout::Layout;
use crate::window::component::layout::const_base_layout::Direction;

pub struct BaseLayout {
    margin: Direction,
    padding: Direction,
    const_layout: Option<Box<dyn ConstLayout>>,
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
    fn calculate(&self, area: &Rect<i16>, parent_area: &Rect<i16>) -> Rect<i16> {
        let mut area = area.clone();

        area.set_position(
            parent_area.x1 + self.margin.left,
            parent_area.y1 + self.margin.up,
        );

        if let Some(const_layout) = &self.const_layout {
            let width = const_layout.as_ref().get_width(
                area.max.get_width() as u16,
                parent_area.min.get_width() as u16,
            );
            area.set_width(width as i16);

            let height = const_layout.as_ref().get_height(
                area.max.get_height() as u16,
                parent_area.min.get_height() as u16,
            );
            area.set_height(height as i16);
        }

        return area;
    }

    fn decrease(&self, area: &Rect<i16>, parent_area: &Rect<i16>) -> Rect<i16> {
        let mut area = area.clone();

        let x_offset = area.get_x_offset() + self.margin.right;
        let parent_x_offset = parent_area.x2;

        // Если смещение больше чем данная область отрисовки уменьшаем размер отрисовки текущей структуры
        if x_offset > parent_x_offset {
            area.change_width_on_coord(parent_x_offset - self.margin.right);
        } else {
            area.change_width(area.max.get_width().min(parent_area.min.get_width()));
        }

        // Всё то же самое но для высоты
        let y_offset = area.get_y_offset();
        let parent_y_offset = parent_area.y2;

        if y_offset > parent_y_offset {
            area.change_height_on_coord(parent_y_offset);
        } else {
            area.change_height(area.max.get_height().min(parent_area.min.get_height()));
        }

        return area;
    }
    fn padding_area(&self, area: &Rect<i16>) -> Rect<i16> {
        let x1 = area.x1 + self.padding.left;
        let y1 = area.y1 + self.padding.up;
        let x2 = area.x2 - self.padding.right;
        let y2 = area.y2 - self.padding.down;

        Rect::new_from_coord((x1, y1), (x2, y2))
    }
    fn next(
        &self,
        _area: &Rect<i16>,
        parent_area: &Rect<i16>,
        _margin: Direction,
    ) -> (Rect<i16>, bool) {
        (
            Rect::new(
                parent_area.x1,
                parent_area.y1,
                parent_area.min.get_width(),
                parent_area.min.get_height(),
            ),
            true,
        )
    }
}
