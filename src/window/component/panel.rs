use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use crate::window::component::base::area::Rect;
use crate::window::component::base::base::Base;
use crate::window::component::base::component_type::{ComponentType, SharedDrawable};
use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::settings::get_background_color;
use crate::window::component::button::ButtonManager;
use crate::window::component::interface::button_manager_control::ButtonManagerControl;
use crate::window::component::interface::component_control::{ComponentControl, PanelControl};
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::drawable::{Drawable, InternalAccess};
use crate::window::component::interface::layout::Layout;
use crate::window::component::layout::base_layout::BaseLayout;
use crate::window::component::layout::const_base_layout::Direction;

pub struct Panel {
    background_color: u32,
    childs: Vec<SharedDrawable>,
    margin: Direction,
    padding: Direction,
    const_layout: Option<&'static dyn ConstLayout>,
    pub base: Base,
    layout: Box<dyn Layout>,
}

#[allow(dead_code)]
impl Panel {
    pub fn set_position(&mut self, x: u16, y: u16) {
        self.base.set_position(x, y);
    }
    pub fn set_height(&mut self, h: u16) {
        self.base.set_height(h);
    }
    pub fn set_width(&mut self, w: u16) {
        self.base.set_width(w);
    }
}

impl Default for Panel {
    fn default() -> Panel {
        let base = Base::new("Panel".to_string(), Rect::new(0, 0, 0, 0));

        Panel {
            background_color: *get_background_color().lock().unwrap(),
            childs: Vec::new(),
            margin: Direction::default(),
            padding: Direction::default(),
            const_layout: None,
            base: base,
            layout: Box::new(BaseLayout {}),
        }
    }
}

impl Drawable for Panel {
    fn print(&self, ctx: &mut GpuRenderContext, _area: &Rect<u16>) {
        if self.base.visible {
            let transient = ((self.background_color >> 24) & 0xff) as f32;
            if transient > 0.0 {
                let rect = &self.base.rect;

                ctx.push_rect(rect, self.background_color);
            }

            for child in self.childs.iter() {
                child.borrow().print(ctx, &self.base.rect);
            }
        }
    }

    fn resize(&mut self, area: &Rect<u16>) -> Rect<u16> {
        // Выделяем прямугольник текущей структуры
        let rect = &mut self.base.rect;
        // Ставим начало в координаты прямогульника поданного на вход
        rect.set_position(
            area.x1 + self.margin.left as u16,
            area.y1 + self.margin.up as u16,
        );

        // Считаем смещение полной области по ширине относительно 0
        let x_offset = rect.get_x_offset();
        let parent_x_offset = area.x2;

        // Если смещение больше чем данная область отрисовки уменьшаем размер отрисовки текущей структуры
        if x_offset > parent_x_offset {
            rect.change_width_on_coord(parent_x_offset);
        } else {
            rect.change_width(rect.max.get_width().min(area.min.get_width()));
        }

        // Всё то же самое но для высоты
        let y_offset = rect.get_y_offset();
        let parent_y_offset = area.y2;

        if y_offset > parent_y_offset {
            rect.change_height_on_coord(parent_y_offset);
        } else {
            rect.change_height(rect.max.get_height().min(area.min.get_height()));
        }

        let x1 = (rect.x1 as i32 + self.padding.left) as u16;
        let y1 = (rect.y1 as i32 + self.padding.up) as u16;
        let x2 = (rect.x2 as i32 - self.padding.right) as u16;
        let y2 = (rect.y2 as i32 - self.padding.down) as u16;

        let padding_rect = Rect::new_from_coord((x1, y1), (x2, y2));
        // Список размещений потомков структуры
        let mut layout = Vec::new();
        layout.push(padding_rect);
        for child in self.childs.iter_mut() {
            let size = layout.len();
            // Берем текущую свободную область
            let current_free_zone = layout[size - 1].clone();

            // Получаем маржин ребенка
            let margin = child.borrow().get_margin().clone();

            // Вызываем resize
            let child_rect = child.borrow_mut().resize(&current_free_zone);

            // Рассчитываем, что осталось после размещения ребенка
            let remainder = self.layout.next(&child_rect, &current_free_zone, margin);

            layout.push(remainder);
        }

        return self.base.rect.clone();
    }

    fn get_type(&self) -> ComponentType {
        ComponentType::Panel
    }

    fn click(&self, x: u16, y: u16) -> bool {
        let rect = &self.base.rect;

        if (x >= rect.x1 && x <= rect.x2) && (y >= rect.y1 && y <= rect.y2) {
            return true;
        }
        false
    }

    fn get_button_manager<'a>(
        &'a self,
        button_manager: &mut ButtonManager,
        token: &InternalAccess,
    ) {
        for child in self.childs.iter() {
            if child.borrow().get_type() == ComponentType::Button {
                button_manager.add(Rc::clone(&child));
            } else {
                child.borrow().get_button_manager(button_manager, token);
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn set_padding(&mut self, direction: Direction) {
        self.padding = direction;
    }
    fn set_margin(&mut self, direction: Direction) {
        self.margin = direction;
    }
    fn set_const_layout(&mut self, const_layout: &dyn ConstLayout) {}

    fn get_margin(&self) -> &Direction {
        &self.margin
    }
    fn get_padding(&self) -> &Direction {
        &self.padding
    }
}

impl ComponentControl for Panel {
    fn add<T: Drawable + 'static>(&mut self, item: T) -> SharedDrawable {
        let shared: SharedDrawable = Rc::new(RefCell::new(item));
        self.childs.push(Rc::clone(&shared));
        return shared;
    }

    fn remove_by_index(&mut self, index: u32) -> Result<(), &'static str> {
        if index > self.childs.len() as u32 {
            self.childs.remove(index as usize);
            Ok(())
        } else {
            Err("Index element out of range")
        }
    }

    fn remove_item(&mut self, _item: SharedDrawable) {
        // if self.childs.contains(&item){

        // }
    }

    fn set_layout(&mut self, layout: Box<dyn Layout>) {
        self.layout = layout;
    }
}

impl PanelControl for Panel {
    fn set_background(&mut self, color: u32) {
        self.background_color = color;
    }
}
