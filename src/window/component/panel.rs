use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use crate::window::component::base::area::Rect;
use crate::window::component::base::base::Base;
use crate::window::component::base::component_type::SharedDrawable;
use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::hover_manager::HoverManager;
use crate::window::component::base::select_manager::SelectManager;
use crate::window::component::base::settings::Settings;
use crate::window::component::base::ui_command::UiCommand;
use crate::window::component::button::ButtonManager;
use crate::window::component::interface::component_control::{ComponentControl, PanelControl};
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::drawable::{Drawable, InternalAccess};
use crate::window::component::interface::layout::Layout;
use crate::window::component::layout::base_layout::BaseLayout;
use crate::window::component::layout::const_base_layout::Direction;
use crate::window::component::layout::layout_context::LayoutContext;

#[allow(dead_code)]
pub struct Panel {
    childs: Vec<SharedDrawable>,
    pub base: Base,
    layout: Box<dyn Layout>,
    clickable: bool,
    hoverable: bool,
    on_click: Option<UiCommand>,
    on_mouse_enter: Option<UiCommand>,
    on_mouse_leave: Option<UiCommand>,
}

#[allow(dead_code)]
impl Panel {
    pub fn set_position(&mut self, x: i16, y: i16) {
        self.base.set_position(x, y);
    }
    pub fn set_height(&mut self, h: u16) {
        self.base.set_height(h);
    }
    pub fn set_width(&mut self, w: u16) {
        self.base.set_width(w);
    }
    pub fn get_command_click(&self) -> Option<UiCommand> {
        if let Some(cmd) = &self.on_click {
            return Some(cmd.clone());
        }
        None
    }
    pub fn get_command_on_mouse_enter(&self) -> Option<UiCommand> {
        if let Some(cmd) = &self.on_mouse_enter {
            return Some(cmd.clone());
        }
        None
    }
    pub fn get_command_on_mouse_leave(&self) -> Option<UiCommand> {
        if let Some(cmd) = &self.on_mouse_leave {
            return Some(cmd.clone());
        }
        None
    }
}

impl Default for Panel {
    fn default() -> Panel {
        let base = Base::new("Panel".to_string(), Rect::new(0, 0, 0, 0));

        Panel {
            childs: Vec::new(),
            base: base,
            layout: BaseLayout::new(),
            clickable: false,
            on_click: None,
            hoverable: false,
            on_mouse_enter: None,
            on_mouse_leave: None,
        }
    }
}

impl Drawable for Panel {
    fn print(&self, ctx: &mut GpuRenderContext) {
        if self.base.visible {
            let transient = ((self.base.settings.background_color >> 24) & 0xff) as f32;
            if transient > 0.0 {
                let rect = &self.base.rect;

                ctx.push_rect(rect, self.base.settings.background_color);
            }

            for child in self.childs.iter() {
                child.borrow().print(ctx);
            }
        }
    }

    fn resize(&mut self, area: &Rect<i16>, ctx: &LayoutContext) -> Rect<i16> {
        // Выделяем прямугольник текущей структуры
        let rect = self.layout.calculate(&self.base.rect, area);
        self.base.rect = rect.clone();

        let padding_rect = self.layout.padding_area(&self.base.rect);
        // Список размещений потомков структуры
        let mut layout = padding_rect;
        for child in self.childs.iter_mut() {
            // Берем текущую свободную область
            let current_free_zone = layout.clone();

            // Получаем маржин ребенка
            let margin = child.borrow().get_margin().clone();

            // Вызываем resize
            let child_rect = child.borrow_mut().resize(&current_free_zone, ctx);

            // Рассчитываем, что осталось после размещения ребенка
            let remainder = self.layout.next(&child_rect, &current_free_zone, margin);

            layout = remainder;
        }

        return self.base.rect.clone();
    }
    fn get_managers<'a>(
        &'a self,
        button_manager: &mut ButtonManager,
        hover_manager: &mut HoverManager,
        select_manager: &mut SelectManager,
        token: &InternalAccess,
    ) {
        for child in self.childs.iter() {
            if child.borrow_mut().is_clickable() {
                button_manager.add(Rc::clone(&child));
            }
            if child.borrow_mut().is_hoverable() {
                hover_manager.add(Rc::clone(&child));
            }
            if child.borrow().is_selectable() {
                select_manager.add(Rc::clone(&child));
            }
            child
                .borrow()
                .get_managers(button_manager, hover_manager, select_manager, token);
        }
    }

    fn set_on_click(&mut self, action: UiCommand) {
        self.clickable = true;
        self.on_click = Some(action)
    }

    fn on_click(&self) {
        if let Some(cmd) = &self.on_click {
            let command_to_send = cmd.clone();

            if let Some(tx) = &self.base.settings.command_tx {
                let _ = tx.send(command_to_send);
            }
        }
    }

    // fn get_button_manager<'a>(
    //     &'a self,
    //     button_manager: &mut ButtonManager,
    //     token: &InternalAccess,
    // ) {
    //     for child in self.childs.iter() {
    //         if child.borrow_mut().is_clickable() {
    //             button_manager.add(Rc::clone(&child));
    //         }
    //         child.borrow().get_button_manager(button_manager, token);
    //     }
    // }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn set_padding(&mut self, direction: Direction) {
        self.layout.set_padding(direction);
    }
    fn set_margin(&mut self, direction: Direction) {
        self.layout.set_margin(direction);
    }
    fn set_const_layout(&mut self, const_layout: Option<Box<dyn ConstLayout>>) {
        self.layout.set_const_layout(const_layout);
    }

    fn get_margin(&self) -> &Direction {
        &self.layout.get_margin()
    }
    fn get_padding(&self) -> &Direction {
        &self.layout.get_padding()
    }
    fn set_default_settings(&mut self, settings: &Settings) {
        if let Some(tx) = &settings.command_tx {
            self.base.settings.command_tx = Some(tx.clone());
        }
        for child in self.childs.iter_mut() {
            child.borrow_mut().set_default_settings(settings);
        }
    }
    fn is_clickable(&mut self) -> bool {
        self.clickable
    }
    fn is_hoverable(&mut self) -> bool {
        self.hoverable
    }
    fn hover(&self, mx: u16, my: u16) -> bool {
        let rect = &self.base.rect;

        if (mx >= rect.x1.max(0) as u16 && mx <= rect.x2.max(0) as u16)
            && (my >= rect.y1.max(0) as u16 && my <= rect.y2.max(0) as u16)
        {
            return true;
        }
        false
    }
    // fn get_hover_manager<'a>(&'a self, hover_manager: &mut HoverManager, token: &InternalAccess) {
    //     for child in self.childs.iter() {
    //         if child.borrow_mut().is_hoverable() {
    //             hover_manager.add(Rc::clone(&child));
    //         } else {
    //             child.borrow().get_hover_manager(hover_manager, token);
    //         }
    //     }
    // }
    fn on_mouse_enter(&self) {
        if let Some(cmd) = &self.on_mouse_enter {
            let command_to_send = cmd.clone();

            if let Some(tx) = &self.base.settings.command_tx {
                let _ = tx.send(command_to_send);
            }
        }
    }
    fn on_mouse_leave(&self) {
        if let Some(cmd) = &self.on_mouse_leave {
            let command_to_send = cmd.clone();

            if let Some(tx) = &self.base.settings.command_tx {
                let _ = tx.send(command_to_send);
            }
        }
    }
    fn set_on_mouse_enter(&mut self, action: UiCommand) {
        self.hoverable = true;
        self.on_mouse_enter = Some(action)
    }
    fn set_on_mouse_leave(&mut self, action: UiCommand) {
        self.hoverable = true;
        self.on_mouse_leave = Some(action)
    }
    fn as_base(&self) -> &Base {
        &self.base
    }
    fn as_base_mut(&mut self) -> &mut Base {
        &mut self.base
    }
}

impl ComponentControl for Panel {
    fn add<T: Drawable + 'static>(&mut self, item: T) -> SharedDrawable {
        let shared: SharedDrawable = Rc::new(RefCell::new(item));
        let weak_self = Rc::downgrade(&shared);
        shared.borrow_mut().as_base_mut().self_ref = Some(weak_self);
        shared
            .borrow_mut()
            .set_default_settings(&self.base.settings);
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
        self.base.settings.background_color = color;
    }
}
