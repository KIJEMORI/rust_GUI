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
use crate::window::component::interface::component_control::{
    ComponentControl, ComponentControlExt, PanelControl,
};
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::drawable::{
    AnimationDrawable, ClickableDrawable, DragableDrawable, Drawable, HoverableDrawable,
    InternalAccess, LayoutDrawable, ScrollableDrawable,
};
use crate::window::component::interface::layout::Layout;
use crate::window::component::layout::base_layout::{Align, BaseLayout};
use crate::window::component::layout::const_base_layout::{ConstBaseLayout, Direction};
use crate::window::component::layout::layout_context::LayoutContext;
use crate::window::component::managers::button_manager::ButtonManager;
use crate::window::component::managers::drag_manager::{DragManager, DragRails};
use crate::window::component::managers::hover_manager::HoverManager;
use crate::window::component::managers::id_manager::IDManager;
use crate::window::component::managers::scroll_manager::ScrollManager;
use crate::window::component::managers::select_manager::SelectManager;
use crate::window::component::theme::border::Border;

#[allow(dead_code)]
pub struct Panel {
    pub childs: Vec<SharedDrawable>,
    pub base: Base,
    pub layout: Box<dyn Layout>,
    pub scroll: Option<Box<Scroll>>,
    dragable: bool,
    on_click: Option<UiCommand>,
    on_mouse_enter: Option<UiCommand>,
    on_mouse_leave: Option<UiCommand>,
    on_drag: Option<UiCommand>,
    in_drag: Option<UiCommand>,
    on_drop: Option<UiCommand>,
    drag_rails: DragRails,
    animation: Vec<AnimationSequence>,
    pub border: Border,
}

#[allow(dead_code)]
impl Panel {
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
        let base = Base::new(Rect::new(0.0, 0.0, 0, 0));

        Panel {
            childs: Vec::new(),
            base: base,
            layout: BaseLayout::new(),
            scroll: None,
            dragable: false,
            on_click: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
            on_drag: None,
            in_drag: None,
            on_drop: None,
            drag_rails: DragRails::None,
            animation: Vec::new(),
            border: Border::default(),
        }
    }
}

impl ClickableDrawable for Panel {
    fn is_clickable(&self) -> bool {
        self.on_click.is_some()
    }

    fn remove_clickable(&mut self) -> &mut dyn ClickableDrawable {
        self.on_click = None;
        self
    }

    fn set_on_click(&mut self, action: UiCommand) -> &mut dyn ClickableDrawable {
        self.on_click = Some(action);
        self
    }

    fn on_click(&self) {
        if let Some(cmd) = &self.on_click {
            let mut command_to_send = cmd.clone();

            command_to_send.fill_ref(&self.base.id);

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

            command_to_send.fill_ref(&self.base.id);

            if let Some(tx) = &self.base.settings.command_tx {
                let _ = tx.send(command_to_send);
            }
        }
    }
    fn on_mouse_leave(&self) {
        if let Some(cmd) = &self.on_mouse_leave {
            let mut command_to_send = cmd.clone();

            command_to_send.fill_ref(&self.base.id);

            if let Some(tx) = &self.base.settings.command_tx {
                let _ = tx.send(command_to_send);
            }
        }
    }
    fn set_on_mouse_enter(&mut self, action: UiCommand) -> &mut dyn HoverableDrawable {
        self.on_mouse_enter = Some(action);
        self
    }
    fn set_on_mouse_leave(&mut self, action: UiCommand) -> &mut dyn HoverableDrawable {
        self.on_mouse_leave = Some(action);
        self
    }
}

impl AnimationDrawable for Panel {
    fn have_animation(&self) -> bool {
        !self.animation.is_empty()
    }
    fn set_animation(&mut self, animation: Vec<AnimationSequence>) -> &mut dyn AnimationDrawable {
        self.animation = animation;
        self
    }
    fn add_animation(&mut self, animation: AnimationSequence) -> &mut dyn AnimationDrawable {
        self.animation.push(animation);
        self
    }
    fn add_animation_batch(
        &mut self,
        animations: Vec<AnimationSequence>,
    ) -> &mut dyn AnimationDrawable {
        self.animation.extend(animations);
        self
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
            command_to_send.fill_ref(&self.base.id);

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
                steps.action.fill_ref(&self.base.id);
            }
        }
    }
}

impl ScrollableDrawable for Panel {
    fn is_scrollable(&self) -> bool {
        self.scroll.is_some()
    }
    fn set_scrolable(&mut self) -> &mut dyn ScrollableDrawable {
        self.scroll = Some(Box::new(Scroll::new()));
        self
    }
    fn remove_scrolable(&mut self) -> &mut dyn ScrollableDrawable {
        self.scroll = None;
        return self;
    }
    fn set_offset(&mut self, x: f32, y: f32, area: &Rect<f32, u16>) {
        let rect = &self.base.rect;
        let y1 = rect.y1 + y;

        self.base.rect.set_position(rect.x1, y1);

        let y1 = self.base.rect.y1;
        let y2 = self.base.rect.get_y2();

        if y1 > area.get_y2() || y2 < area.y1 {
            self.base.visible_on_this_frame = false;
        } else {
            self.base.visible_on_this_frame = true;
        }
    }

    fn scroll(&mut self, x: f32, y: f32) -> bool {
        if let Some(scroll) = &mut self.scroll {
            let x_change = scroll.change_offset_x(x);
            let y_change = scroll.change_offset_y(y);
            if x_change && y_change {
                //println!("y: {} h: {}", scroll.offset.1, scroll.height);
                // for child in &self.childs {
                //     if let Some(scrollable) = child.borrow_mut().as_scrollable() {
                //         scrollable.set_offset(x, y, &self.base.rect);
                //     }
                // }

                let mut offset = scroll.get_offset();
                for child in self.childs.iter() {
                    let item_top = offset.1;

                    let mut child = child.borrow_mut();

                    let child_rect = child.as_panel_control_mut().get_rect();

                    let item_height = child_rect.max.get_height() as f32;
                    let item_bottom = item_top + item_height;

                    let panel_view_height = self.base.rect.min.get_height() as f32;
                    let buffer = 0.0;

                    if item_bottom < -buffer || item_top > panel_view_height + buffer {
                        child.as_base_mut().visible_on_this_frame = false;
                    } else {
                        child.as_base_mut().visible_on_this_frame = true;
                    }

                    let margin = child.as_layout_control_mut().get_margin();

                    offset.1 += item_height + margin.up as f32 + margin.down as f32;
                }

                return true;
            }
        }
        false
    }
}

impl LayoutDrawable for Panel {
    fn set_padding(&mut self, direction: Direction) -> &mut dyn LayoutDrawable {
        self.layout.set_padding(direction);
        self
    }
    fn set_margin(&mut self, direction: Direction) -> &mut dyn LayoutDrawable {
        self.layout.set_margin(direction);
        self
    }
    fn set_const_layout(
        &mut self,
        const_layout: Option<Box<dyn ConstLayout>>,
    ) -> &mut dyn LayoutDrawable {
        self.layout.set_const_layout(const_layout);
        self
    }

    fn get_margin(&self) -> &Direction {
        &self.layout.get_margin()
    }
    fn get_padding(&self) -> &Direction {
        &self.layout.get_padding()
    }
}

impl DragableDrawable for Panel {
    fn is_dragable(&self) -> bool {
        self.dragable
    }
    fn set_dragable(&mut self, tumbler: bool) -> &mut dyn DragableDrawable {
        self.dragable = tumbler;
        self
    }
    fn start_drag(&mut self) {
        self.layout.as_mut().set_align(Align::FreeRun);
        if let Some(cmd) = &self.on_drag {
            let mut command_to_send = cmd.clone();

            command_to_send.fill_ref(&self.base.id);

            if let Some(tx) = &self.base.settings.command_tx {
                let _ = tx.send(command_to_send);
            }
        }
    }
    fn drag(&mut self, mx_offset: f32, my_offset: f32) {
        let rect = &self.base.rect;
        let mut x1 = rect.x1;
        let mut y1 = rect.y1;

        let mut mx_offset = mx_offset;
        let mut my_offset = my_offset;

        match &self.drag_rails {
            DragRails::Horizontal => {
                x1 += mx_offset;
                my_offset = 0.0;
            }
            DragRails::Vertical => {
                y1 += my_offset;
                mx_offset = 0.0
            }
            DragRails::None => {
                x1 += mx_offset;
                y1 += my_offset;
            }
        }

        if let Some(cmd) = &self.in_drag {
            let mut command_to_send = cmd.clone();

            command_to_send.fill_ref(&self.base.id);
            command_to_send.fill_coord(mx_offset, my_offset);

            if let Some(tx) = &self.base.settings.command_tx {
                let _ = tx.send(command_to_send);
            }
        }

        self.set_position(x1, y1);
    }
    fn stop_drag(&mut self) {
        if let Some(cmd) = &self.on_drop {
            let mut command_to_send = cmd.clone();

            command_to_send.fill_ref(&self.base.id);

            if let Some(tx) = &self.base.settings.command_tx {
                let _ = tx.send(command_to_send);
            }
        }
    }
    fn set_rails(&mut self, rails: DragRails) -> &mut dyn DragableDrawable {
        self.drag_rails = rails;
        self
    }
    fn set_on_drag(&mut self, command: UiCommand) -> &mut dyn DragableDrawable {
        self.on_drag = Some(command);
        self
    }
    fn set_in_drag(&mut self, command: UiCommand) -> &mut dyn DragableDrawable {
        self.in_drag = Some(command);
        self
    }
    fn set_on_drop(&mut self, command: UiCommand) -> &mut dyn DragableDrawable {
        self.on_drop = Some(command);
        self
    }
}

impl Drawable for Panel {
    fn print(
        &mut self,
        ctx: &mut GpuRenderContext,
        area: &Rect<f32, u16>,
        level: u32,
        id_parent: u32,
    ) {
        self.base.id_parent = id_parent;
        if self.base.visible && self.base.visible_on_this_frame {
            self.base.set_parent_rect(area.clone());
            let mut rect = self.base.rect.clone();

            let mut x1 = rect.x1 + area.x1;
            let mut y1 = rect.y1 + area.y1;

            rect.set_position(x1, y1);

            ctx.push_rect_sdf(
                &rect,
                self.base.settings.background_color,
                &self.border,
                level,
                true,
                false,
            );
            let current_content_level = level + 1;
            let transient = ((self.base.settings.background_color >> 24) & 0xff) as f32;
            if transient > 0.0 {
                ctx.push_rect_sdf(
                    &rect,
                    self.base.settings.background_color,
                    &self.border,
                    current_content_level,
                    false,
                    false,
                );
            }

            let next_level = level + 1;

            if let Some(scroll) = &self.scroll {
                let offset = scroll.get_offset();
                x1 += offset.0;
                y1 += offset.1;
            }

            let mut rect_child = self.base.rect.clone();
            rect_child.set_position(x1, y1);

            for child in self.childs.iter() {
                child
                    .borrow_mut()
                    .print(ctx, &rect_child, next_level, self.base.id);
            }

            ctx.push_rect_sdf(
                &rect,
                self.base.settings.background_color,
                &self.border,
                level,
                true,
                true,
            );
        }
    }

    fn resize(
        &mut self,
        area: &Rect<f32, u16>,
        ctx: &LayoutContext,
        auto_size: bool,
    ) -> Rect<f32, u16> {
        self.layout.set_auto_scale(auto_size);

        let mut rect = self.layout.calculate(&self.base.rect, area);

        if self.layout.is_auto_scale() {
            rect = self.layout.decrease(&rect, area);
        }

        self.base.rect = rect.clone();

        if rect.intersection(area) {
            self.base.visible_on_this_frame = true;

            let padding_rect = self.layout.padding_area(&rect);

            let mut layout = padding_rect;
            let mut offset = (0.0, 0.0);
            let mut width = self.base.rect.min.get_width();
            let mut height = 0;

            let scrollabe = self.is_scrollable();

            for child in self.childs.iter_mut() {
                let mut current_free_zone = layout.clone();

                let mut child = child.borrow_mut();

                // Получаем маржин ребенка
                let margin = child.as_layout_control().get_margin().clone();

                let (x, y) = (
                    (offset.0) + current_free_zone.x1,
                    (offset.1) + current_free_zone.y1,
                );

                // if let Some(scroll) = &self.scroll {
                //     let offset = scroll.get_offset();
                // }
                // if x > rect.get_x2() && y > rect.get_y2() {
                //     continue;
                // }

                current_free_zone.set_position(x, y);

                let child_rect = child.resize(&current_free_zone, ctx, !scrollabe);

                if !scrollabe {
                    let (remainder, need_more) =
                        self.layout.next(&child_rect, &current_free_zone, margin);

                    if !need_more {
                        break;
                    }

                    layout = remainder;
                }

                if let Some(scroll) = &mut self.scroll {
                    let item_top = offset.1 + scroll.get_offset().1;

                    let item_height = child_rect.max.get_height() as f32;
                    let item_bottom = item_top + item_height;

                    let panel_view_height = self.base.rect.min.get_height() as f32;
                    let buffer = 0.0;

                    if item_bottom < -buffer || item_top > panel_view_height + buffer {
                        child.as_base_mut().visible_on_this_frame = false;
                    } else {
                        child.as_base_mut().visible_on_this_frame = true;
                    }

                    // Обновляем offset для следующего элемента
                    offset.1 += item_height + margin.up as f32 + margin.down as f32;
                    height += (item_height as i16 + margin.up + margin.down) as u16;
                    width = width.max(child_rect.min.get_width());

                    scroll.set_height_width(height, width);
                    scroll.set_slider_height_width(
                        self.base.rect.min.get_height(),
                        self.base.rect.min.get_width(),
                    );
                }
            }
        } else {
            self.base.visible_on_this_frame = false;
        }

        return self.base.rect.clone();
    }
    fn get_managers<'a>(
        &'a self,
        button_manager: &mut ButtonManager,
        hover_manager: &mut HoverManager,
        select_manager: &mut SelectManager,
        scroll_manager: &mut ScrollManager,
        drag_manager: &mut DragManager,
        id_manager: &mut IDManager,
        token: &InternalAccess,
    ) {
        for child in self.childs.iter() {
            let id = id_manager.register(Rc::clone(&child));

            if let Some(clickable) = child.borrow_mut().as_clickable() {
                if clickable.is_clickable() {
                    button_manager.add(id);
                }
            }
            if let Some(hoverable) = child.borrow_mut().as_hoverable() {
                if hoverable.is_hoverable() {
                    hover_manager.add(id);
                }
            }
            if let Some(selectable) = child.borrow_mut().as_selectable() {
                if selectable.is_selectable() {
                    select_manager.add(id);
                }
            }
            if let Some(scrollable) = child.borrow_mut().as_scrollable() {
                if scrollable.is_scrollable() {
                    scroll_manager.add(id);
                }
            }
            if let Some(dragable) = child.borrow_mut().as_dragable() {
                if dragable.is_dragable() {
                    drag_manager.add(id);
                }
            }
            child.borrow().get_managers(
                button_manager,
                hover_manager,
                select_manager,
                scroll_manager,
                drag_manager,
                id_manager,
                token,
            );
        }
    }

    add_drawable_control!();

    fn set_default_settings(&mut self, settings: &Settings) -> &mut dyn Drawable {
        if let Some(tx) = &settings.command_tx {
            self.base.settings.command_tx = Some(tx.clone());
        }
        for child in self.childs.iter_mut() {
            child.borrow_mut().set_default_settings(settings);
        }
        self
    }

    fn hover(&self, mx: u16, my: u16, area: &Rect<f32, u16>) -> bool {
        let mut panel_rect = self.base.rect.clone();
        let parent_rect = &self.base.parent_rect;

        let global_x = parent_rect.x1 + panel_rect.x1;
        let global_y = parent_rect.y1 + panel_rect.y1;
        panel_rect.set_position(global_x, global_y);

        let mx_f = mx as f32;
        let my_f = my as f32;

        let in_panel = mx_f >= panel_rect.x1.max(area.x1)
            && mx_f <= panel_rect.get_x2().min(area.get_x2())
            && my_f >= panel_rect.y1.max(area.y1)
            && my_f <= panel_rect.get_y2().min(area.get_y2());

        if !in_panel || !self.base.visible_on_this_frame {
            return false;
        }
        true
    }

    fn as_base(&self) -> &Base {
        &self.base
    }
    fn as_base_mut(&mut self) -> &mut Base {
        &mut self.base
    }
    fn as_panel_control(&self) -> &dyn PanelControl {
        self
    }
    fn as_panel_control_mut(&mut self) -> &mut dyn PanelControl {
        self
    }
    fn as_layout_control(&self) -> &dyn LayoutDrawable {
        self
    }
    fn as_layout_control_mut(&mut self) -> &mut dyn LayoutDrawable {
        self
    }

    fn as_component_control_mut(&mut self) -> Option<&mut dyn ComponentControl> {
        Some(self)
    }

    fn as_clickable(&self) -> Option<&dyn ClickableDrawable> {
        Some(self)
    }
    fn as_clickable_mut(&mut self) -> Option<&mut dyn ClickableDrawable> {
        Some(self)
    }

    fn as_hoverable(&self) -> Option<&dyn HoverableDrawable> {
        Some(self)
    }
    fn as_hoverable_mut(&mut self) -> Option<&mut dyn HoverableDrawable> {
        Some(self)
    }

    fn as_with_animation(&self) -> Option<&dyn AnimationDrawable> {
        Some(self)
    }
    fn as_with_animation_mut(&mut self) -> Option<&mut dyn AnimationDrawable> {
        Some(self)
    }

    fn as_scrollable(&self) -> Option<&dyn ScrollableDrawable> {
        Some(self)
    }
    fn as_scrollable_mut(&mut self) -> Option<&mut dyn ScrollableDrawable> {
        Some(self)
    }

    fn as_dragable(&self) -> Option<&dyn DragableDrawable> {
        Some(self)
    }
    fn as_dragable_mut(&mut self) -> Option<&mut dyn DragableDrawable> {
        Some(self)
    }
}

impl ComponentControl for Panel {
    // fn add<T: Drawable + 'static>(&mut self, item: T) -> SharedDrawable {
    //     let shared: SharedDrawable = Rc::new(RefCell::new(item));
    //     let weak_self = Rc::downgrade(&shared);
    //     shared.borrow_mut().as_base_mut().self_ref = Some(weak_self);
    //     shared
    //         .borrow_mut()
    //         .set_default_settings(&self.base.settings);
    //     self.childs.push(Rc::clone(&shared));
    //     return shared;
    // }

    fn add_drawable(&mut self, item: SharedDrawable) -> SharedDrawable {
        let shared: SharedDrawable = Rc::clone(&item);
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

impl ComponentControlExt for Panel {
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
}

impl PanelControl for Panel {
    fn set_background(&mut self, color: u32) -> &mut dyn PanelControl {
        self.base.settings.background_color = color;
        self
    }
    fn set_position(&mut self, x: f32, y: f32) -> &mut dyn PanelControl {
        self.base.set_position(x, y);
        self
    }
    fn set_height(&mut self, h: u16) -> &mut dyn PanelControl {
        self.base.set_height(h);
        self
    }
    fn set_width(&mut self, w: u16) -> &mut dyn PanelControl {
        self.base.set_width(w);
        self
    }
    fn get_rect(&self) -> &Rect<f32, u16> {
        &self.base.rect
    }
    fn get_rect_without_offset(&self, rect: &Rect<f32, u16>) -> Rect<f32, u16> {
        let mut rect = rect.clone();
        if let Some(scroll) = self.scroll.as_ref() {
            let offset = scroll.get_offset();
            let x1 = rect.x1 - offset.0;
            let y1 = rect.y1 - offset.1;
            rect.set_position(x1, y1);
        }
        rect
    }
}
