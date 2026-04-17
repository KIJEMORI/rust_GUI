use crate::{
    add_drawable_control,
    window::component::{
        base::{
            area::Rect, base::Base, component_type::SharedDrawable,
            gpu_render_context::GpuRenderContext, settings::Settings,
        },
        interface::{
            component_control::{ComponentControl, FullEditControl, LabelControl, PanelControl},
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
    pub horizontal_slider: bool,
    pub vertical_width: u8,
    pub horizontal_height: u8,
}

impl Default for ScrollPanel {
    fn default() -> Self {
        let mut panel = Panel::default();

        panel.as_scrollable_mut().unwrap().set_scrolable();

        Self {
            panel: panel,
            vertical_slider: true,
            horizontal_slider: true,
            vertical_width: 15,
            horizontal_height: 15,
        }
    }
}

impl Drawable for ScrollPanel {
    fn print(&self, ctx: &mut GpuRenderContext, area: &Rect<f32, u16>, level: u32) {
        if self.panel.base.visible && self.panel.base.visible_on_this_frame {
            let rect = &self.panel.base.rect;
            let background_color = self.panel.base.settings.background_color;
            let border = &self.panel.border;

            ctx.push_rect_sdf(rect, background_color, border, level, true, false);
            let current_content_level = level + 1;
            let transient = ((background_color >> 24) & 0xff) as f32;
            if transient > 0.0 {
                ctx.push_rect_sdf(
                    rect,
                    background_color,
                    border,
                    current_content_level,
                    false,
                    false,
                );
            }

            let next_level = level + 1;

            for child in self.panel.childs.iter() {
                child.borrow().print(ctx, rect, next_level);
            }

            if let Some(scroll) = &self.panel.scroll {
                let offset = scroll.get_offset();

                if self.vertical_slider {
                    let track_height = if self.horizontal_slider {
                        rect.min.get_height() as f32 - self.horizontal_height as f32
                    } else {
                        rect.min.get_height() as f32
                    };

                    let proportion = scroll.slider_height as f32 / scroll.height as f32;

                    let slider_height = (proportion * track_height) as u16;

                    let slider_x1 = rect.get_x2() - self.vertical_width as f32;

                    let max_y =
                        rect.get_y2() - self.horizontal_height as f32 - slider_height as f32;
                    let slider_y1 = (offset.1.abs() * proportion + rect.y1).min(max_y);

                    let vertical_scroll_rect = Rect::new(
                        slider_x1,
                        slider_y1,
                        self.vertical_width as u16,
                        slider_height,
                    );
                    let color = 0xFFFF0000;
                    ctx.push_rect_sdf(&vertical_scroll_rect, color, border, level, false, false);
                }
                if self.horizontal_slider {
                    let track_width = if self.vertical_slider {
                        rect.min.get_width() as f32 - self.vertical_width as f32
                    } else {
                        rect.min.get_width() as f32
                    };

                    let proportion = scroll.slider_width as f32 / scroll.width as f32;

                    let slider_width = (proportion * track_width) as u16;

                    let max_x = rect.get_x2() - self.vertical_width as f32 - slider_width as f32;
                    let slider_x1 = (offset.0.abs() * proportion + rect.x1).min(max_x);
                    let slider_y1 = rect.get_y2() - self.horizontal_height as f32;

                    let horizontal_scroll_rect = Rect::new(
                        slider_x1,
                        slider_y1,
                        slider_width,
                        self.horizontal_height as u16,
                    );
                    let color = 0xFF00FF00;
                    ctx.push_rect_sdf(&horizontal_scroll_rect, color, border, level, false, false);
                }
            }

            ctx.push_rect_sdf(rect, background_color, border, level, true, true);
        }
    }

    fn resize(
        &mut self,
        area: &Rect<f32, u16>,
        ctx: &LayoutContext,
        scroll_item: bool,
    ) -> Rect<f32, u16> {
        let rect = self.panel.resize(area, ctx, scroll_item);

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
    }

    fn under(&self, mx: u16, my: u16) -> bool {
        self.panel.under(mx, my)
    }

    fn hover(&self, mx: u16, my: u16) -> bool {
        self.panel.hover(mx, my)
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
        self
    }

    fn as_base(&self) -> &Base {
        self.panel.as_base()
    }
    fn as_base_mut(&mut self) -> &mut Base {
        self.panel.as_base_mut()
    }

    fn as_panel_control_mut(&mut self) -> &mut dyn PanelControl {
        self.panel.as_panel_control_mut()
    }

    fn as_label_control_mut(&mut self) -> Option<&mut dyn LabelControl> {
        None
    }
    fn as_edit_label_control_mut(&mut self) -> Option<&mut dyn FullEditControl> {
        None
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
    fn add<T: Drawable + 'static>(&mut self, item: T) -> SharedDrawable {
        self.panel.add(item)
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
