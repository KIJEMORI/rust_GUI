use crate::{
    add_drawable_control,
    window::component::{
        base::{area::Rect, base::Base, gpu_render_context::GpuRenderContext, settings::Settings},
        interface::{
            component_control::PanelControl,
            drawable::{
                AnimationDrawable, ClickableDrawable, DragableDrawable, Drawable,
                HoverableDrawable, LayoutDrawable,
            },
        },
        layout::layout_context::LayoutContext,
        panel::Panel,
    },
};

pub struct ScrollSlider {
    pub panel: Panel,
}

impl Default for ScrollSlider {
    fn default() -> Self {
        Self {
            panel: Panel::default(),
        }
    }
}

impl Drawable for ScrollSlider {
    fn print(
        &mut self,
        ctx: &mut GpuRenderContext,
        area: &Rect<f32, u16>,
        level: u32,
        id_parent: u32,
    ) {
        // self.panel.base.id_parent = id_parent;
        // if self.panel.base.visible && self.panel.base.visible_on_this_frame {
        //     self.panel.base.set_parent_rect(area.clone());
        self.panel.print(ctx, area, level, id_parent);
        //}
    }

    fn resize(
        &mut self,
        area: &Rect<f32, u16>,
        ctx: &LayoutContext,
        auto_size: bool,
    ) -> Rect<f32, u16> {
        let rect = self.panel.resize(area, ctx, auto_size);

        let id = self.as_base().id;

        // if let Some(scroll) = &mut self.panel.scroll {
        //     let mut height = scroll.height;
        //     let mut width = scroll.width;

        //     if self.vertical_slider {
        //         width += self.vertical_width as u16;
        //     }
        //     if self.horizontal_slider {
        //         height += self.horizontal_height as u16;
        //     }

        //     scroll.set_height_width(height, width);

        //     if self.vertical_slider {
        //         let mut vertical_slider_panel = self.vertical_slider_panel.borrow_mut();

        //         vertical_slider_panel.as_base_mut().visible_on_this_frame = true;

        //         let track_height = if self.horizontal_slider {
        //             rect.min.get_height() as f32 - self.horizontal_height as f32
        //         } else {
        //             rect.min.get_height() as f32
        //         };

        //         let mut slider_height = scroll.get_vertical_slider_height(track_height) as f32;

        //         let min_h = scroll.min_slider_height as f32;
        //         if slider_height < min_h {
        //             slider_height = min_h;
        //         }

        //         let free_space = track_height - slider_height;

        //         if let Some(dragable) = vertical_slider_panel.as_dragable_mut() {
        //             let content_to_track_ratio =
        //                 (scroll.height - scroll.slider_height) as f32 / free_space;

        //             dragable.set_in_drag(UiCommand::ScrollPanel(
        //                 Some(id),
        //                 0.0,
        //                 content_to_track_ratio, // Множитель для Y
        //             ));
        //         }

        //         let scroll_progress = scroll.get_vertical_progress();

        //         let slider_x1 = rect.get_x2() - self.vertical_width as f32 - rect.x1;

        //         let slider_y1 = free_space * scroll_progress;

        //         let vertical_scroll_rect = Rect::new(
        //             slider_x1,
        //             slider_y1,
        //             self.vertical_width as u16,
        //             slider_height as u16,
        //         );

        //         vertical_slider_panel.as_base_mut().rect = vertical_scroll_rect;

        //         let color = 0xFFFF0000;
        //         vertical_slider_panel
        //             .as_base_mut()
        //             .settings
        //             .background_color = color;
        //     }
        //     if self.horizontal_slider {
        //         let mut horizontal_slider_panel = self.horizontal_slider_panel.borrow_mut();
        //         horizontal_slider_panel.as_base_mut().visible_on_this_frame = true;
        //         let track_width = if self.vertical_slider {
        //             rect.min.get_width() as f32 - self.vertical_width as f32
        //         } else {
        //             rect.min.get_width() as f32
        //         };

        //         let mut slider_width = scroll.get_horizontal_slider_width(track_width) as f32;

        //         let min_w = scroll.min_slider_width as f32;
        //         if slider_width < min_w {
        //             slider_width = min_w;
        //         }

        //         let free_space = track_width - slider_width;

        //         if let Some(dragable) = horizontal_slider_panel.as_dragable_mut() {
        //             let content_to_track_ratio =
        //                 (scroll.width - scroll.slider_width) as f32 / free_space;

        //             dragable.set_in_drag(UiCommand::ScrollPanel(
        //                 Some(id),
        //                 content_to_track_ratio,
        //                 0.0,
        //             ));
        //         }

        //         let scroll_progress = scroll.get_horizontal_progress();

        //         let slider_x1 = (free_space * scroll_progress);

        //         let slider_y1 = rect.get_y2() - self.horizontal_height as f32 - rect.y1;

        //         let horizontal_scroll_rect = Rect::new(
        //             slider_x1,
        //             slider_y1,
        //             slider_width as u16,
        //             self.horizontal_height as u16,
        //         );

        //         horizontal_slider_panel.as_base_mut().rect = horizontal_scroll_rect;

        //         let color = 0xFFFF0000;
        //         horizontal_slider_panel
        //             .as_base_mut()
        //             .settings
        //             .background_color = color;

        //         //ctx.push_rect_sdf(&horizontal_scroll_rect, color, border, level, false, false);
        //     }
        // }

        return rect;
    }

    fn hover(&self, mx: u16, my: u16, area: &Rect<f32, u16>) -> bool {
        let mut panel_rect = self.panel.base.rect.clone();
        let parent_rect = &self.panel.base.parent_rect;

        let global_x = parent_rect.x1 + panel_rect.x1;
        let global_y = parent_rect.y1 + panel_rect.y1;
        panel_rect.set_position(global_x, global_y);

        let mx_f = mx as f32;
        let my_f = my as f32;

        let in_panel = mx_f >= panel_rect.x1
            && mx_f <= panel_rect.get_x2()
            && my_f >= panel_rect.y1
            && my_f <= panel_rect.get_y2();

        if !in_panel || !self.panel.base.visible_on_this_frame {
            return false;
        }
        true
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

    fn as_panel_control(&self) -> &dyn PanelControl {
        self.panel.as_panel_control()
    }
    fn as_panel_control_mut(&mut self) -> &mut dyn PanelControl {
        self.panel.as_panel_control_mut()
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

    fn as_with_animation(&self) -> Option<&dyn AnimationDrawable> {
        self.panel.as_with_animation()
    }
    fn as_with_animation_mut(&mut self) -> Option<&mut dyn AnimationDrawable> {
        self.panel.as_with_animation_mut()
    }

    fn as_dragable(&self) -> Option<&dyn DragableDrawable> {
        self.panel.as_dragable()
    }
    fn as_dragable_mut(&mut self) -> Option<&mut dyn DragableDrawable> {
        self.panel.as_dragable_mut()
    }
}
