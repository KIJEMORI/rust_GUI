use winit::keyboard::SmolStr;

use crate::add_drawable_control;
use crate::window::component::base::area::Rect;
use crate::window::component::base::base::Base;
use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::settings::Settings;
use crate::window::component::base::ui_command::UiCommand;
use crate::window::component::interface::component_control::{
    EditLabelControl, FullEditControl, LabelControl, PanelControl,
};
use crate::window::component::interface::drawable::{
    AnimationDrawable, ClickableDrawable, Drawable, HoverableDrawable, LayoutDrawable,
    ScrollableDrawable, SelectableDrawable,
};
use crate::window::component::layout::layout_context::LayoutContext;
use crate::window::component::panel::Panel;

pub struct Label {
    pub text: String,
    pub panel: Panel,
    max_scale: f32,
    pub scale: f32,
    color: u32,
    needs_layout: bool,
    editable: bool,
    change_cursor: bool,
    cursor_need: bool,
    cursor: Rect<f32, u16>,
    cursor_index: u32,
    pub cursor_color: u32,
    select_index_start: u32,
    select_index_end: u32,
    select_color: u32,
    select_rect: Rect<f32, u16>,
}

impl Label {
    pub fn new(text: String) -> Self {
        Label {
            text,
            panel: Panel::default(),
            max_scale: 20.0,
            scale: 20.0,
            color: 0xFF000000,
            needs_layout: true,
            editable: false,
            change_cursor: false,
            cursor_need: false,
            cursor: Rect::default(),
            cursor_index: 0,
            cursor_color: 0xFFFFFF00,
            select_index_start: 0,
            select_index_end: 0,
            select_color: 0xAAFFFFFF,
            select_rect: Rect::default(),
        }
    }

    pub fn from_str(text: &str) -> Self {
        let text = text.to_string();

        Label::new(text)
    }

    pub fn calculate_intrinsic_size(&self, font: &fontdue::Font, scale: f32) -> (u16, u16) {
        let mut width = 0.0;

        for c in self.text.chars() {
            let metrics = font.metrics(c, scale);
            width += metrics.advance_width;
        }

        if let Some(line_metrics) = font.horizontal_line_metrics(scale) {
            let height = line_metrics.ascent - line_metrics.descent;

            (width.ceil() as u16, height.ceil() as u16)
        } else {
            (width.ceil() as u16, scale.ceil() as u16)
        }
    }

    fn set_max_size(&mut self, ctx: &LayoutContext) {
        let (w, h) = self.calculate_intrinsic_size(ctx.font, self.max_scale);
        self.panel.set_width(w);
        self.panel.set_height(h);
    }
    fn get_index(&self, target_x: f32, font: &fontdue::Font, scale: f32) -> u32 {
        let mut current_x = self.panel.base.rect.x1 + self.as_base().parent_rect.x1;

        if let Some(scroll) = &self.panel.scroll {
            current_x += scroll.offset.0;
        }

        for (i, c) in self.text.chars().enumerate() {
            let metrics = font.metrics(c, scale);
            let char_width = metrics.advance_width;

            // Если кликнули в левую половину символа — индекс до него, если в правую — после
            if target_x < current_x + char_width / 2.0 {
                return i as u32;
            }
            current_x += char_width;
        }

        self.text.chars().count() as u32
    }
    fn char_to_byte_idx(&self, char_idx: u32) -> u32 {
        self.text
            .char_indices()
            .map(|(i, _)| i)
            .nth(char_idx as usize)
            .unwrap_or(self.text.len()) as u32
    }
}

impl FullEditControl for Label {}

impl LabelControl for Label {
    fn set_font(&mut self, family_name: &'static str) {}

    #[allow(dead_code)]
    fn set_font_color(&mut self, color: u32) {
        self.color = color;
    }
    #[allow(dead_code)]
    fn get_font_color(&self) -> u32 {
        return self.color;
    }
    fn set_scale(&mut self, scale: u16) {
        self.max_scale = scale as f32;
        self.needs_layout = true;
    }
    fn set_text(&mut self, text: String) {
        self.text = text;
    }
    fn get_text(&self) -> &str {
        &self.text
    }
    fn set_text_str(&mut self, text: &str) {
        self.text = text.to_string();
    }

    fn remove_select(&mut self) {
        self.select_index_start = 0;
        self.select_index_end = 0;
        self.select_rect = Rect::default();

        if let Some(tx) = &self.panel.base.settings.command_tx {
            let _ = tx.send(UiCommand::RequestRedraw());
        }
    }
    fn set_start_caret(&mut self, select_start: (u16, u16), ctx: &LayoutContext) {
        // Вычисляем индекс символа под курсором мыши
        self.select_index_start = self.get_index(select_start.0 as f32, ctx.font, self.scale);
        self.select_index_end = self.select_index_start;
        self.cursor_index = self.select_index_start;
        self.change_cursor = true
    }
    fn set_end_caret(&mut self, select_end: (u16, u16), ctx: &LayoutContext) -> bool {
        let last_end_index = self.select_index_end;
        self.select_index_end = self.get_index(select_end.0 as f32, ctx.font, self.scale);
        self.change_cursor = true;

        if self.select_index_start != self.select_index_end
            && self.select_index_end != last_end_index
        {
            // if let Some(tx) = &self.panel.base.settings.command_tx {
            //     let _ = tx.send(UiCommand::RequestRedraw());
            // }

            self.resize_one(ctx);
            return true;
        }
        false
    }
}

impl EditLabelControl for Label {
    fn is_editable(&self) -> bool {
        self.editable
    }
    fn set_cursor(&mut self) {
        self.cursor_need = true;
        self.cursor_index = self.select_index_end;
    }
    fn on_cursor(&mut self) {
        self.cursor_need = true;
        self.as_with_animation_mut().unwrap().restart_animations();
    }
    fn delete_cursor(&mut self) {
        self.cursor_need = false;
    }
    fn move_cursor_right(&mut self, right: bool) {
        if right {
            self.cursor_index = (self.text.chars().count() as u32).min(self.cursor_index + 1);
            self.select_index_end = self.cursor_index;
            self.select_index_start = self.cursor_index;
        } else if self.cursor_index > 0 {
            self.cursor_index -= 1;
            self.select_index_end = self.cursor_index;
            self.select_index_start = self.cursor_index;
        }
        self.change_cursor = true;
        self.as_with_animation_mut().unwrap().restart_animations();
    }

    fn add_text(&mut self, text: &SmolStr) {
        let byte_idx = self.get_byte_offset(self.cursor_index) as u32;

        if self.select_index_start != self.select_index_end {
            self.delete_selection();
            self.text.insert_str(byte_idx as usize, text);
        } else {
            self.text.insert_str(byte_idx as usize, text);
        }

        self.cursor_index += text.chars().count() as u32;

        self.sync_indexes();
    }
    fn backspace(&mut self) {
        if self.select_index_start != self.select_index_end {
            self.delete_selection();
        } else if self.cursor_index > 0 {
            self.cursor_index -= 1;
            let byte_idx = self.get_byte_offset(self.cursor_index);
            self.text.remove(byte_idx); // Удаляет один char по байтовому смещению
        }
        self.sync_indexes();
    }
    fn delete_selection(&mut self) {
        if self.select_index_start == self.select_index_end {
            return;
        }

        let start_char = self.select_index_start.min(self.select_index_end);
        let end_char = self.select_index_start.max(self.select_index_end);

        // Находим байтовые границы
        let byte_start = self.char_to_byte_idx(start_char) as usize;
        let byte_end = self.char_to_byte_idx(end_char) as usize;

        // Удаляем диапазон байт напрямую из строки (без создания Vec<char>)
        self.text.drain(byte_start..byte_end);

        // Устанавливаем курсор в начало удаленного фрагмента
        self.cursor_index = start_char;
        self.sync_indexes();
    }

    fn delete(&mut self) {
        if self.select_index_start != self.select_index_end {
            self.delete_selection();
        } else {
            let len = self.text.chars().count() as u32;
            if self.cursor_index < len {
                let byte_idx = self.get_byte_offset(self.cursor_index);
                self.text.remove(byte_idx);
                self.sync_indexes();
            }
        }
    }

    fn sync_indexes(&mut self) {
        self.change_cursor = true;
        self.select_index_start = self.cursor_index;
        self.select_index_end = self.cursor_index;
        self.as_with_animation_mut().unwrap().restart_animations();
    }

    fn get_byte_offset(&self, char_idx: u32) -> usize {
        self.text
            .char_indices()
            .map(|(i, _)| i)
            .nth(char_idx as usize)
            .unwrap_or(self.text.len())
    }
}

impl SelectableDrawable for Label {
    fn is_selectable(&self) -> bool {
        true
    }
}

impl Drawable for Label {
    fn print(
        &mut self,
        ctx: &mut GpuRenderContext,
        area: &Rect<f32, u16>,
        level: u32,
        id_parent: u32,
    ) {
        self.as_base_mut().id_parent = id_parent;
        if self.panel.base.visible_on_this_frame {
            self.as_base_mut().set_parent_rect(area.clone());

            let mut rect = self.panel.base.rect.clone();

            let mut x1 = rect.x1 + area.x1;
            let mut y1 = rect.y1 + area.y1;

            rect.set_position(x1, y1);

            let border = &self.panel.border;

            ctx.push_rect_sdf(
                &rect,
                self.panel.base.settings.background_color,
                border,
                level,
                true,
                false,
            );

            if self.select_index_start != self.select_index_end {
                let mut select_rect = self.select_rect.clone();
                select_rect.set_position(select_rect.x1, rect.y1);

                ctx.push_rect_sdf(&select_rect, self.select_color, border, level, false, false);
            }

            if let Some(scroll) = &self.panel.scroll {
                let offset = scroll.get_offset();
                x1 += offset.0;
                y1 += offset.1;
            }

            ctx.push_text(&self.text, x1, y1, self.scale, self.color, level);

            if self.cursor_need {
                let mut cursor_rect = self.cursor.clone();
                cursor_rect.set_position(cursor_rect.x1, rect.y1);
                ctx.push_rect_sdf(&cursor_rect, self.cursor_color, border, level, false, false);
            }

            ctx.push_rect_sdf(
                &rect,
                self.panel.base.settings.background_color,
                border,
                level,
                true,
                true,
            );
        }
    }

    fn resize_one(&mut self, ctx: &LayoutContext) {
        let area = &self.as_base().parent_rect.clone();

        if self.needs_layout {
            self.set_max_size(ctx);

            self.needs_layout = false
        }

        let mut rect = self.panel.base.rect.clone();

        rect.set_position(rect.x1 + area.x1, rect.y1 + area.y1);

        let scale_factor = self.scale / ctx.sdf_base_size;

        let line_metrics = ctx.font.horizontal_line_metrics(ctx.sdf_base_size).unwrap();
        let text_height = (line_metrics.ascent - line_metrics.descent) * scale_factor;

        let mut current_x = 0.0;
        let mut x_start = 0.0;
        let mut x_end = 0.0;
        let mut x_cursor = 0.0;

        for (i, c) in self.text.chars().enumerate() {
            let idx = i as u32;
            if idx == self.select_index_start {
                x_start = current_x;
            }
            if idx == self.select_index_end {
                x_end = current_x;
            }
            if idx == self.cursor_index {
                x_cursor = current_x;
            }

            let metrics = ctx.font.metrics(c, ctx.sdf_base_size);

            current_x += metrics.advance_width * scale_factor;
        }
        if let Some(scroll) = &mut self.panel.scroll {
            scroll.set_height_width(text_height as u16, current_x as u16);
            scroll.set_slider_height_width(rect.min.get_height(), rect.min.get_width());
        }

        let char_count = self.text.chars().count() as u32;
        if self.select_index_start == char_count {
            x_start = current_x;
        }
        if self.select_index_end == char_count {
            x_end = current_x;
        }
        if self.cursor_index == char_count {
            x_cursor = current_x;
        }

        if self.select_index_start != self.select_index_end {
            let mut x1_offset = x_start;
            let mut x2_offset = x_end;

            if let Some(scroll) = &mut self.panel.scroll {
                x1_offset += scroll.offset.0;
                x2_offset += scroll.offset.0;
            }

            x2_offset = x2_offset.min(rect.get_x2());

            let first_point = ((rect.x1 as f32 + x1_offset.min(x2_offset)), rect.y1);
            let second_point = ((rect.x1 as f32 + x1_offset.max(x2_offset)), rect.get_y2());

            self.select_rect = Rect::new_from_coord(first_point, second_point);
        }

        let x_offset = x_cursor;
        let mut x_pos = rect.x1 as f32 + x_offset;

        let x2 = rect.get_x2();

        if let Some(scroll) = &mut self.panel.scroll
            && self.change_cursor
        {
            if x_pos + scroll.offset.0 > x2 {
                let scroll_x_offset = x2 - scroll.offset.0 - x_pos;

                scroll.change_offset_x(scroll_x_offset);
            } else if x_pos < rect.x1 - scroll.offset.0 {
                let scroll_x_offset = rect.x1 - scroll.offset.0 - x_pos;

                scroll.change_offset_x(scroll_x_offset);
            }
            self.change_cursor = false;
        }

        if let Some(scroll) = &mut self.panel.scroll {
            x_pos += scroll.offset.0
        }

        self.cursor = Rect::new_from_coord((x_pos, rect.y1), (x_pos + 1.0, rect.get_y2()));
    }

    fn resize(
        &mut self,
        area: &Rect<f32, u16>,
        ctx: &LayoutContext,
        auto_size: bool,
    ) -> Rect<f32, u16> {
        if self.needs_layout {
            self.set_max_size(ctx);

            self.needs_layout = false
        }

        self.panel.resize(area, ctx, auto_size);

        let rect = &self.panel.base.rect;

        let scale_factor = self.scale / ctx.sdf_base_size;

        let line_metrics = ctx.font.horizontal_line_metrics(ctx.sdf_base_size).unwrap();
        let text_height = (line_metrics.ascent - line_metrics.descent) * scale_factor;

        let mut current_x = 0.0;
        let mut x_start = 0.0;
        let mut x_end = 0.0;
        let mut x_cursor = 0.0;

        for (i, c) in self.text.chars().enumerate() {
            let idx = i as u32;
            if idx == self.select_index_start {
                x_start = current_x;
            }
            if idx == self.select_index_end {
                x_end = current_x;
            }
            if idx == self.cursor_index {
                x_cursor = current_x;
            }

            let metrics = ctx.font.metrics(c, ctx.sdf_base_size);

            current_x += metrics.advance_width * scale_factor;
        }
        if let Some(scroll) = &mut self.panel.scroll {
            scroll.set_height_width(text_height as u16, current_x as u16);
            scroll.set_slider_height_width(rect.min.get_height(), rect.min.get_width());
        }

        let char_count = self.text.chars().count() as u32;
        if self.select_index_start == char_count {
            x_start = current_x;
        }
        if self.select_index_end == char_count {
            x_end = current_x;
        }
        if self.cursor_index == char_count {
            x_cursor = current_x;
        }

        if self.select_index_start != self.select_index_end {
            let mut x1_offset = x_start;
            let mut x2_offset = x_end;

            if let Some(scroll) = &mut self.panel.scroll {
                x1_offset += scroll.offset.0;
                x2_offset += scroll.offset.0;
            }

            x2_offset = x2_offset.min(rect.get_x2());

            let first_point = ((rect.x1 as f32 + x1_offset.min(x2_offset)), rect.y1);
            let second_point = ((rect.x1 as f32 + x1_offset.max(x2_offset)), rect.get_y2());

            self.select_rect = Rect::new_from_coord(first_point, second_point);
        }

        let x_offset = x_cursor;
        let mut x_pos = rect.x1 as f32 + x_offset;

        let x2 = rect.get_x2();

        if let Some(scroll) = &mut self.panel.scroll
            && self.change_cursor
        {
            if x_pos + scroll.offset.0 > x2 {
                let scroll_x_offset = x2 - scroll.offset.0 - x_pos;

                scroll.change_offset_x(scroll_x_offset);
            } else if x_pos < rect.x1 - scroll.offset.0 {
                let scroll_x_offset = rect.x1 - scroll.offset.0 - x_pos;

                scroll.change_offset_x(scroll_x_offset);
            }
            self.change_cursor = false;
        }

        if let Some(scroll) = &mut self.panel.scroll {
            x_pos += scroll.offset.0
        }

        self.cursor = Rect::new_from_coord((x_pos, rect.y1), (x_pos + 1.0, rect.get_y2()));

        self.panel.base.rect.clone()
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

    fn as_label_control_mut(&mut self) -> Option<&mut dyn LabelControl> {
        Some(self)
    }
    fn as_edit_label_control_mut(&mut self) -> Option<&mut dyn FullEditControl> {
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
        Some(self)
    }
    fn as_selectable_mut(&mut self) -> Option<&mut dyn SelectableDrawable> {
        Some(self)
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
}
