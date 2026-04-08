use std::cell::RefCell;
use std::rc::Rc;

use crate::add_drawable_control;
use crate::window::component::animation::animation_action::AnimationSequence;
use crate::window::component::base::area::Rect;
use crate::window::component::base::base::Base;
use crate::window::component::base::component_type::SharedDrawable;
use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::scroll::Scroll;
use crate::window::component::base::settings::Settings;
use crate::window::component::base::ui_command::UiCommand;
use crate::window::component::interface::component_control::{ComponentControl, PanelControl};
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::drawable::{
    AnimationDrawable, ClickableDrawable, Drawable, HoverableDrawable, InternalAccess,
    ScrollableDrawable,
};
use crate::window::component::interface::layout::Layout;
use crate::window::component::layout::base_layout::BaseLayout;
use crate::window::component::layout::const_base_layout::Direction;
use crate::window::component::layout::layout_context::LayoutContext;
use crate::window::component::managers::button_manager::ButtonManager;
use crate::window::component::managers::hover_manager::HoverManager;
use crate::window::component::managers::scroll_manager::ScrollManager;
use crate::window::component::managers::select_manager::SelectManager;

#[allow(dead_code)]
pub struct Panel {
    childs: Vec<SharedDrawable>,
    pub base: Base,
    layout: Box<dyn Layout>,
    scrollable: bool,
    pub scroll: Scroll,
    on_click: Option<UiCommand>,
    on_mouse_enter: Option<UiCommand>,
    on_mouse_leave: Option<UiCommand>,
    animation: Vec<AnimationSequence>,
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
            scrollable: false,
            scroll: Scroll::default(),
            on_click: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
            animation: Vec::new(),
        }
    }
}

impl ClickableDrawable for Panel {
    fn is_clickable(&self) -> bool {
        self.on_click.is_some()
    }

    fn remove_clickable(&mut self) {
        self.on_click = None;
    }

    fn set_on_click(&mut self, action: UiCommand) {
        self.on_click = Some(action)
    }

    fn on_click(&self) {
        if let Some(cmd) = &self.on_click {
            let mut command_to_send = cmd.clone();

            command_to_send.fill_ref(&self.base.get_shared());

            if let Some(tx) = &self.base.settings.command_tx {
                let _ = tx.send(command_to_send);
            }
        }
    }
}

impl HoverableDrawable for Panel {
    fn is_hoverable(&self) -> bool {
        self.on_mouse_enter.is_some() || self.on_mouse_leave.is_some()
    }
    fn on_mouse_enter(&self) {
        if let Some(cmd) = &self.on_mouse_enter {
            let mut command_to_send = cmd.clone();

            command_to_send.fill_ref(&self.base.get_shared());

            if let Some(tx) = &self.base.settings.command_tx {
                let _ = tx.send(command_to_send);
            }
        }
    }
    fn on_mouse_leave(&self) {
        if let Some(cmd) = &self.on_mouse_leave {
            let mut command_to_send = cmd.clone();

            command_to_send.fill_ref(&self.base.get_shared());

            if let Some(tx) = &self.base.settings.command_tx {
                let _ = tx.send(command_to_send);
            }
        }
    }
    fn set_on_mouse_enter(&mut self, action: UiCommand) {
        self.on_mouse_enter = Some(action)
    }
    fn set_on_mouse_leave(&mut self, action: UiCommand) {
        self.on_mouse_leave = Some(action)
    }
}

impl AnimationDrawable for Panel {
    fn have_animation(&self) -> bool {
        !self.animation.is_empty()
    }
    fn set_animation(&mut self, animation: Vec<AnimationSequence>) {
        self.animation = animation
    }
    fn add_animation(&mut self, animation: AnimationSequence) {
        self.animation.push(animation)
    }
    fn add_animation_batch(&mut self, animations: Vec<AnimationSequence>) {
        self.animation.extend(animations);
    }
    fn start_animation(&mut self) {
        self.fill_ref();
        if self.base.run_base_animation && self.base.run_loop_animation {
            return;
        }
        if !self.animation.is_empty() {
            self.base.run_base_animation = true;
            self.base.run_loop_animation = true;
            let mut command_to_send = UiCommand::StartAnimation(None);
            command_to_send.fill_ref(&self.base.get_shared());

            if let Some(tx) = &self.base.settings.command_tx {
                let _ = tx.send(command_to_send);
            }
        }
    }
    fn get_animations(&self) -> &[AnimationSequence] {
        &self.animation
    }
    fn stop_animations(&mut self) {
        self.base.run_base_animation = false;
    }
    fn stop_loop_animation(&mut self) {
        self.base.run_loop_animation = false;
    }
    fn restart_animations(&mut self) {
        self.stop_animations();
        self.stop_loop_animation();

        self.start_animation();
    }
    fn need_animate(&self) -> bool {
        self.base.run_base_animation
    }
    fn need_animate_loop(&self) -> bool {
        self.base.run_loop_animation
    }

    fn fill_ref(&mut self) {
        for anim in &mut self.animation {
            for steps in &mut anim.steps {
                steps.action.fill_ref(&self.base.get_shared());
            }
        }
    }
}

impl ScrollableDrawable for Panel {
    fn is_scrollable(&self) -> bool {
        self.scrollable
    }
    fn set_scrolable(&mut self) {
        self.scrollable = true;
    }
    fn remove_scrolable(&mut self) {
        self.scrollable = false;
    }
    fn set_offset(&mut self, x: f32, y: f32) {
        self.scroll.set_offset(x, y);
    }
    fn scroll(&mut self, x: f32, y: f32) -> bool {
        let change_y = self.scroll.change_offset_y(y);
        let change_x = self.scroll.change_offset_x(x);
        change_y || change_x
    }
}

impl Drawable for Panel {
    fn print(&self, ctx: &mut GpuRenderContext, area: &Rect<i16>, offset: (f32, f32)) {
        let mut new_offset = self.scroll.get_offset();
        new_offset.0 += offset.0;
        new_offset.1 += offset.1;

        if self.scrollable {
            new_offset = (0.0, 0.0)
        }

        if self.base.visible && self.base.visible_on_this_frame {
            let transient = ((self.base.settings.background_color >> 24) & 0xff) as f32;
            if transient > 0.0 {
                let rect = &self.base.rect;

                ctx.push_rect_sdf(
                    rect,
                    Some(area),
                    self.base.settings.background_color,
                    new_offset,
                    0.0,
                );
            }

            for child in self.childs.iter() {
                child
                    .borrow()
                    .print(ctx, &self.base.rect, self.scroll.get_offset());
            }
        }
    }

    fn resize(&mut self, area: &Rect<i16>, ctx: &LayoutContext, scroll_item: bool) -> Rect<i16> {
        let rect = self.layout.calculate(&self.base.rect, area);
        self.base.rect = rect.clone();

        if scroll_item {
            let padding_rect = self.layout.padding_area(&self.base.rect);

            let mut layout = padding_rect;
            for child in self.childs.iter_mut() {
                let current_free_zone = layout.clone();

                // Получаем маржин ребенка
                let margin = child.borrow().get_margin().clone();

                // Вызываем resize
                let child_rect = child.borrow_mut().resize(&current_free_zone, ctx, false);

                let (remainder, need_more) =
                    self.layout.next(&child_rect, &current_free_zone, margin);

                if !need_more {
                    break;
                }

                layout = remainder;
            }
        } else {
            let rect = self.layout.decrease(&self.base.rect, area);
            self.base.rect = rect.clone();

            if rect.intersection(area) {
                self.base.visible_on_this_frame = true;

                let padding_rect = self.layout.padding_area(&self.base.rect);

                let mut layout = padding_rect;
                let mut offset = (0.0, 0.0);
                let mut width = 0;
                for child in self.childs.iter_mut() {
                    let current_free_zone = layout.clone();

                    // Получаем маржин ребенка
                    let margin = child.borrow().get_margin().clone();

                    // Вызываем resize
                    let child_rect =
                        child
                            .borrow_mut()
                            .resize(&current_free_zone, ctx, self.scrollable);

                    if !self.scrollable {
                        let (remainder, need_more) =
                            self.layout.next(&child_rect, &current_free_zone, margin);

                        if !need_more {
                            break;
                        }

                        layout = remainder;
                    } else {
                        child
                            .borrow_mut()
                            .as_scrollable()
                            .unwrap()
                            .set_offset(offset.0, offset.1);

                        width += child_rect.max.get_width();

                        let scroll_offset = self.scroll.get_offset();
                        //child.borrow_mut().as_base_mut().visible_on_this_frame = true;
                        let buffer = 100.0; // Запас видимости
                        if offset.1 + scroll_offset.1
                            > (self.base.rect.min.get_height() as f32 + buffer)
                        {
                            child.borrow_mut().as_base_mut().visible_on_this_frame = false;
                        } else if offset.1 + scroll_offset.1 < -buffer {
                            // Если ушло вверх
                            child.borrow_mut().as_base_mut().visible_on_this_frame = false;
                        } else {
                            child.borrow_mut().as_base_mut().visible_on_this_frame = true;
                        }
                        offset.1 += child_rect.max.get_height() as f32
                            + margin.up as f32
                            + margin.down as f32;
                    }
                }
                self.scroll.set_height_width(offset.1 as i16, width);
                self.scroll.set_slider_height_width(
                    self.base.rect.min.get_height(),
                    self.base.rect.min.get_width(),
                );
            } else {
                self.base.visible_on_this_frame = false;
            }
        }

        return self.base.rect.clone();
    }
    fn get_managers<'a>(
        &'a self,
        button_manager: &mut ButtonManager,
        hover_manager: &mut HoverManager,
        select_manager: &mut SelectManager,
        scroll_manager: &mut ScrollManager,
        token: &InternalAccess,
    ) {
        for child in self.childs.iter() {
            if let Some(clickable) = child.borrow_mut().as_clickable() {
                if clickable.is_clickable() {
                    button_manager.add(Rc::clone(&child));
                }
            }
            if let Some(hoverable) = child.borrow_mut().as_hoverable() {
                if hoverable.is_hoverable() {
                    hover_manager.add(Rc::clone(&child));
                }
            }
            if let Some(selectable) = child.borrow_mut().as_selectable() {
                if selectable.is_selectable() {
                    select_manager.add(Rc::clone(&child));
                }
            }
            if let Some(scrollable) = child.borrow_mut().as_scrollable() {
                if scrollable.is_scrollable() {
                    scroll_manager.add(Rc::clone(&child));
                }
            }
            child.borrow().get_managers(
                button_manager,
                hover_manager,
                select_manager,
                scroll_manager,
                token,
            );
        }
    }

    add_drawable_control!();

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

    fn hover(&self, mx: u16, my: u16) -> bool {
        let rect = &self.base.rect;

        mx as i32 >= rect.x1 as i32
            && mx as i32 <= rect.x2 as i32
            && my as i32 >= rect.y1 as i32
            && my as i32 <= rect.y2 as i32
    }

    fn as_base(&self) -> &Base {
        &self.base
    }
    fn as_base_mut(&mut self) -> &mut Base {
        &mut self.base
    }
    fn as_panel_control_mut(&mut self) -> Option<&mut dyn PanelControl> {
        Some(self)
    }

    fn as_clickable(&mut self) -> Option<&mut dyn ClickableDrawable> {
        Some(self)
    }
    fn as_hoverable(&mut self) -> Option<&mut dyn HoverableDrawable> {
        Some(self)
    }
    fn as_with_animation(&mut self) -> Option<&mut dyn AnimationDrawable> {
        Some(self)
    }
    fn as_scrollable(&mut self) -> Option<&mut dyn ScrollableDrawable> {
        Some(self)
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
