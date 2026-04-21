use std::{cell::RefCell, rc::Rc};

use crate::{
    add_drawable_control,
    window::component::{
        base::{
            area::Rect, base::Base, component_type::SharedDrawable,
            gpu_render_context::GpuRenderContext, scroll_slider::ScrollSlider, settings::Settings,
            ui_command::UiCommand,
        },
        interface::{
            component_control::{ComponentControl, ComponentControlExt, PanelControl},
            drawable::{
                AnimationDrawable, ClickableDrawable, DragableDrawable, Drawable,
                HoverableDrawable, InternalAccess, LayoutDrawable, ScrollableDrawable,
                SelectableDrawable,
            },
            layout::Layout,
        },
        layout::layout_context::LayoutContext,
        managers::{
            button_manager::ButtonManager, drag_manager::DragManager, hover_manager::HoverManager,
            id_manager::IDManager, scroll_manager::ScrollManager, select_manager::SelectManager,
        },
        panel::Panel,
    },
};

pub struct ScrollPanel {
    pub panel: Panel,
    pub vertical_slider: bool,
    vertical_slider_panel: SharedDrawable,
    pub horizontal_slider: bool,
    horizontal_slider_panel: SharedDrawable,
    pub vertical_width: u8,
    pub horizontal_height: u8,
}

impl Default for ScrollPanel {
    fn default() -> Self {
        let mut panel = Panel::default();

        panel.as_scrollable_mut().unwrap().set_scrolable();

        let mut vertical_slider_panel = ScrollSlider::default();
        vertical_slider_panel
            .as_dragable_mut()
            .unwrap()
            .set_dragable(true)
            .set_rails(super::managers::drag_manager::DragRails::Vertical);

        let vertical_slider_panel: SharedDrawable = Rc::new(RefCell::new(vertical_slider_panel));

        let mut horizontal_slider_panel = ScrollSlider::default();
        horizontal_slider_panel
            .as_dragable_mut()
            .unwrap()
            .set_dragable(true)
            .set_rails(super::managers::drag_manager::DragRails::Horizontal);

        let horizontal_slider_panel: SharedDrawable =
            Rc::new(RefCell::new(horizontal_slider_panel));
        Self {
            panel: panel,
            vertical_slider: true,
            vertical_slider_panel: vertical_slider_panel,
            horizontal_slider: true,
            horizontal_slider_panel: horizontal_slider_panel,
            vertical_width: 15,
            horizontal_height: 15,
        }
    }
}

impl Drawable for ScrollPanel {
    fn print(
        &mut self,
        ctx: &mut GpuRenderContext,
        area: &Rect<f32, u16>,
        level: u32,
        id_parent: u32,
    ) {
        self.panel.base.id_parent = id_parent;
        if self.panel.base.visible && self.panel.base.visible_on_this_frame {
            self.panel.base.set_parent_rect(area.clone());
            let mut rect = self.panel.base.rect.clone();

            let mut x1 = rect.x1 + area.x1;
            let mut y1 = rect.y1 + area.y1;

            rect.set_position(x1, y1);

            {
                let background_color = self.panel.base.settings.background_color;
                let border = &self.panel.border;

                ctx.push_rect_sdf(&rect, background_color, border, level, true, false);
                let current_content_level = level + 1;
                let transient = ((background_color >> 24) & 0xff) as f32;
                if transient > 0.0 {
                    ctx.push_rect_sdf(
                        &rect,
                        background_color,
                        border,
                        current_content_level,
                        false,
                        false,
                    );
                }
            }

            let next_level = level + 1;

            if let Some(scroll) = &self.panel.scroll {
                let offset = scroll.get_offset();
                x1 += offset.0;
                y1 += offset.1;
            }

            let mut rect_child = self.panel.base.rect.clone();
            rect_child.set_position(x1, y1);

            for child in self.panel.childs.iter() {
                child
                    .borrow_mut()
                    .print(ctx, &rect_child, next_level, self.panel.base.id);
            }

            if let Some(scroll) = &mut self.panel.scroll {
                let offset = scroll.get_offset();
                if self.vertical_slider {
                    let mut vertical_slider_panel = self.vertical_slider_panel.borrow_mut();

                    let track_height = if self.horizontal_slider {
                        rect.min.get_height() as f32 - self.horizontal_height as f32
                    } else {
                        rect.min.get_height() as f32
                    };

                    let mut slider_height = scroll.get_vertical_slider_height(track_height) as f32;

                    let min_h = scroll.min_slider_height as f32;
                    if slider_height < min_h {
                        slider_height = min_h;
                    }

                    let free_space = track_height - slider_height;
                    let scroll_progress = scroll.get_vertical_progress();

                    let slider_x1 = rect.get_x2() - self.vertical_width as f32 - rect.x1;

                    let slider_y1 = free_space * scroll_progress;

                    let vertical_scroll_rect = Rect::new(
                        slider_x1,
                        slider_y1,
                        self.vertical_width as u16,
                        slider_height as u16,
                    );

                    vertical_slider_panel.as_base_mut().rect = vertical_scroll_rect;

                    vertical_slider_panel.print(ctx, &rect, level + 1, self.panel.base.id);
                }
                if self.horizontal_slider {
                    let mut horizontal_slider_panel = self.horizontal_slider_panel.borrow_mut();

                    let track_width = if self.vertical_slider {
                        rect.min.get_width() as f32 - self.vertical_width as f32
                    } else {
                        rect.min.get_width() as f32
                    };

                    let mut slider_width = scroll.get_horizontal_slider_width(track_width) as f32;

                    let min_w = scroll.min_slider_width as f32;
                    if slider_width < min_w {
                        slider_width = min_w;
                    }

                    let free_space = track_width - slider_width;

                    let scroll_progress = scroll.get_horizontal_progress();

                    let slider_x1 = (free_space * scroll_progress);

                    let slider_y1 = rect.get_y2() - self.horizontal_height as f32 - rect.y1;

                    let horizontal_scroll_rect = Rect::new(
                        slider_x1,
                        slider_y1,
                        slider_width as u16,
                        self.horizontal_height as u16,
                    );

                    horizontal_slider_panel.as_base_mut().rect = horizontal_scroll_rect;

                    horizontal_slider_panel.print(ctx, &rect, level + 1, self.panel.base.id);
                    //ctx.push_rect_sdf(&horizontal_scroll_rect, color, border, level, false, false);
                }
            }

            let background_color = self.panel.base.settings.background_color;
            let border = &self.panel.border;

            ctx.push_rect_sdf(&rect, background_color, border, level, true, true);
        }
    }

    fn resize(
        &mut self,
        area: &Rect<f32, u16>,
        ctx: &LayoutContext,
        auto_size: bool,
    ) -> Rect<f32, u16> {
        let rect = self.panel.resize(area, ctx, auto_size);

        let id = self.as_base().id;

        if let Some(scroll) = &mut self.panel.scroll {
            let mut height = scroll.height;
            let mut width = scroll.width;

            if self.vertical_slider {
                width += self.vertical_width as u16;
            }
            if self.horizontal_slider {
                height += self.horizontal_height as u16;
            }

            scroll.set_height_width(height, width);

            if self.vertical_slider {
                let mut vertical_slider_panel = self.vertical_slider_panel.borrow_mut();

                vertical_slider_panel.as_base_mut().visible_on_this_frame = true;

                let track_height = if self.horizontal_slider {
                    rect.min.get_height() as f32 - self.horizontal_height as f32
                } else {
                    rect.min.get_height() as f32
                };

                let mut slider_height = scroll.get_vertical_slider_height(track_height) as f32;

                let min_h = scroll.min_slider_height as f32;
                if slider_height < min_h {
                    slider_height = min_h;
                }

                let free_space = track_height - slider_height;

                if let Some(dragable) = vertical_slider_panel.as_dragable_mut() {
                    let content_to_track_ratio =
                        (scroll.height - scroll.slider_height) as f32 / free_space;

                    dragable.set_in_drag(UiCommand::ScrollPanel(
                        Some(id),
                        0.0,
                        content_to_track_ratio, // Множитель для Y
                    ));
                }

                let scroll_progress = scroll.get_vertical_progress();

                let slider_x1 = rect.get_x2() - self.vertical_width as f32 - rect.x1;

                let slider_y1 = free_space * scroll_progress;

                let vertical_scroll_rect = Rect::new(
                    slider_x1,
                    slider_y1,
                    self.vertical_width as u16,
                    slider_height as u16,
                );

                vertical_slider_panel.as_base_mut().rect = vertical_scroll_rect;

                let color = 0xFFFF0000;
                vertical_slider_panel
                    .as_base_mut()
                    .settings
                    .background_color = color;
            }
            if self.horizontal_slider {
                let mut horizontal_slider_panel = self.horizontal_slider_panel.borrow_mut();
                horizontal_slider_panel.as_base_mut().visible_on_this_frame = true;
                let track_width = if self.vertical_slider {
                    rect.min.get_width() as f32 - self.vertical_width as f32
                } else {
                    rect.min.get_width() as f32
                };

                let mut slider_width = scroll.get_horizontal_slider_width(track_width) as f32;

                let min_w = scroll.min_slider_width as f32;
                if slider_width < min_w {
                    slider_width = min_w;
                }

                let free_space = track_width - slider_width;

                if let Some(dragable) = horizontal_slider_panel.as_dragable_mut() {
                    let content_to_track_ratio =
                        (scroll.width - scroll.slider_width) as f32 / free_space;

                    dragable.set_in_drag(UiCommand::ScrollPanel(
                        Some(id),
                        content_to_track_ratio,
                        0.0,
                    ));
                }

                let scroll_progress = scroll.get_horizontal_progress();

                let slider_x1 = (free_space * scroll_progress);

                let slider_y1 = rect.get_y2() - self.horizontal_height as f32 - rect.y1;

                let horizontal_scroll_rect = Rect::new(
                    slider_x1,
                    slider_y1,
                    slider_width as u16,
                    self.horizontal_height as u16,
                );

                horizontal_slider_panel.as_base_mut().rect = horizontal_scroll_rect;

                let color = 0xFFFF0000;
                horizontal_slider_panel
                    .as_base_mut()
                    .settings
                    .background_color = color;

                //ctx.push_rect_sdf(&horizontal_scroll_rect, color, border, level, false, false);
            }
        }

        return rect;
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
        self.panel.get_managers(
            button_manager,
            hover_manager,
            select_manager,
            scroll_manager,
            drag_manager,
            id_manager,
            token,
        );

        let id_vert = id_manager.register(Rc::clone(&self.vertical_slider_panel));
        let id_hori = id_manager.register(Rc::clone(&self.horizontal_slider_panel));

        if let Some(clickable) = self.vertical_slider_panel.borrow().as_clickable() {
            if clickable.is_clickable() {
                button_manager.add(id_vert);
            }
        }
        if let Some(clickable) = self.horizontal_slider_panel.borrow().as_clickable() {
            if clickable.is_clickable() {
                button_manager.add(id_hori);
            }
        }

        if let Some(hoverable) = self.vertical_slider_panel.borrow().as_hoverable() {
            if hoverable.is_hoverable() {
                hover_manager.add(id_vert);
            }
        }
        if let Some(hoverable) = self.horizontal_slider_panel.borrow().as_hoverable() {
            if hoverable.is_hoverable() {
                hover_manager.add(id_hori);
            }
        }

        if let Some(dragable) = self.vertical_slider_panel.borrow().as_dragable() {
            if dragable.is_dragable() {
                drag_manager.add(id_vert);
            }
        }
        if let Some(dragable) = self.horizontal_slider_panel.borrow().as_dragable() {
            if dragable.is_dragable() {
                drag_manager.add(id_hori);
            }
        }
    }

    fn hover(&self, mx: u16, my: u16, area: &Rect<f32, u16>) -> bool {
        self.panel.hover(mx, my, area)
    }

    add_drawable_control!();

    fn as_layout_control(&self) -> &dyn LayoutDrawable {
        self.panel.as_layout_control()
    }
    fn as_layout_control_mut(&mut self) -> &mut dyn LayoutDrawable {
        self.panel.as_layout_control_mut()
    }

    fn set_default_settings(&mut self, settings: &Settings) -> &mut dyn Drawable {
        self.panel.set_default_settings(settings);
        self.horizontal_slider_panel
            .borrow_mut()
            .set_default_settings(settings);
        self.vertical_slider_panel
            .borrow_mut()
            .set_default_settings(settings);
        self
    }

    fn as_base(&self) -> &Base {
        self.panel.as_base()
    }
    fn as_base_mut(&mut self) -> &mut Base {
        self.panel.as_base_mut()
    }

    fn as_panel_control(&self) -> &dyn PanelControl {
        self.panel.as_panel_control()
    }
    fn as_panel_control_mut(&mut self) -> &mut dyn PanelControl {
        self.panel.as_panel_control_mut()
    }

    fn as_component_control_mut(&mut self) -> Option<&mut dyn ComponentControl> {
        Some(self)
    }

    fn as_clickable(&self) -> Option<&dyn ClickableDrawable> {
        self.panel.as_clickable()
    }
    fn as_clickable_mut(&mut self) -> Option<&mut dyn ClickableDrawable> {
        self.panel.as_clickable_mut()
    }

    fn as_hoverable(&self) -> Option<&dyn HoverableDrawable> {
        self.panel.as_hoverable()
    }
    fn as_hoverable_mut(&mut self) -> Option<&mut dyn HoverableDrawable> {
        self.panel.as_hoverable_mut()
    }

    fn as_selectable(&self) -> Option<&dyn SelectableDrawable> {
        None
    }

    fn as_with_animation(&self) -> Option<&dyn AnimationDrawable> {
        self.panel.as_with_animation()
    }
    fn as_with_animation_mut(&mut self) -> Option<&mut dyn AnimationDrawable> {
        self.panel.as_with_animation_mut()
    }

    fn as_scrollable(&self) -> Option<&dyn ScrollableDrawable> {
        self.panel.as_scrollable()
    }
    fn as_scrollable_mut(&mut self) -> Option<&mut dyn ScrollableDrawable> {
        self.panel.as_scrollable_mut()
    }

    fn as_dragable(&self) -> Option<&dyn DragableDrawable> {
        self.panel.as_dragable()
    }
    fn as_dragable_mut(&mut self) -> Option<&mut dyn DragableDrawable> {
        self.panel.as_dragable_mut()
    }
}

impl ComponentControl for ScrollPanel {
    fn add_drawable(&mut self, item: SharedDrawable) -> SharedDrawable {
        self.panel.add_drawable(item)
    }

    fn remove_by_index(&mut self, index: u32) -> Result<(), &'static str> {
        self.panel.remove_by_index(index)
    }

    fn remove_item(&mut self, item: SharedDrawable) {
        self.panel.remove_item(item);
    }

    fn set_layout(&mut self, layout: Box<dyn Layout>) {
        self.panel.set_layout(layout);
    }
}

impl ComponentControlExt for ScrollPanel {
    fn add<T: Drawable + 'static>(&mut self, item: T) -> SharedDrawable {
        self.panel.add(item)
    }
}
