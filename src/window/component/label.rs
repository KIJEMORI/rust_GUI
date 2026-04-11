use wgpu_glyph::ab_glyph::{Font, FontArc, PxScaleFont, ScaleFont};
use wgpu_glyph::{FontId, GlyphPositioner, Layout, SectionGeometry, SectionText};
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
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::drawable::{
    AnimationDrawable, ClickableDrawable, Drawable, HoverableDrawable, SelectableDrawable,
};
use crate::window::component::layout::const_base_layout::Direction;
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
    cursor_need: bool,
    cursor: Rect<i16>,
    cursor_index: u32,
    pub cursor_color: u32,
    select_index_start: u32,
    select_index_end: u32,
    select_color: u32,
    select_rect: Rect<i16>,
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

    pub fn set_height(&mut self, h: u16) {
        self.panel.set_height(h);
    }
    pub fn set_width(&mut self, w: u16) {
        self.panel.set_width(w);
    }

    pub fn calculate_intrinsic_size(&self, font: &FontArc, scale: f32) -> (u16, u16) {
        use wgpu_glyph::ab_glyph::{Font, ScaleFont};

        let scaled_font = font.as_scaled(scale);
        let mut width = 0.0;
        let mut last_glyph_id = None;

        for c in self.text.chars() {
            let glyph_id = scaled_font.glyph_id(c);
            if let Some(last_id) = last_glyph_id {
                // Кернинг можно убрать для еще большего ускорения
                width += scaled_font.kern(last_id, glyph_id);
            }
            width += scaled_font.h_advance(glyph_id);
            last_glyph_id = Some(glyph_id);
        }

        let height = scaled_font.ascent() - scaled_font.descent();
        (width.ceil() as u16, height.ceil() as u16)
    }

    pub fn calculate_wrapped_size(&self, font: &FontArc, scale: f32, max_width: f32) -> (u16, u16) {
        let layout = Layout::default();

        let text = SectionText {
            text: &self.text,
            scale: scale.into(),
            font_id: FontId(0),
        };

        let fonts = &[font];
        let glyphs = layout.calculate_glyphs(
            fonts,
            &SectionGeometry {
                screen_position: (0.0, 0.0),
                bounds: (max_width, f32::INFINITY),
            },
            &[text],
        );

        let mut min_x = 0.0;
        let mut max_x = 0.0;
        let mut min_y = 0.0;
        let mut max_y = 0.0;

        for glyph in glyphs {
            let pos = glyph.glyph.position;

            max_x = (max_x as f32).max(pos.x as f32);
            max_y = (max_y as f32).max(pos.y as f32);
        }

        let scaled_font = font.as_scaled(scale);
        let line_height = scaled_font.ascent() - scaled_font.descent();

        (max_x.ceil() as u16, (max_y + line_height).ceil() as u16)
    }

    fn set_max_size(&mut self, ctx: &LayoutContext) {
        let font = &ctx.fonts[self.panel.base.settings.font_id.0];

        let (w, h) = self.calculate_intrinsic_size(&font, self.max_scale);
        self.panel.set_width(w);
        self.panel.set_height(h);
    }
    fn get_index(&self, target_x: f32, scaled_font: PxScaleFont<&FontArc>) -> u32 {
        let rect = &self.panel.base.rect;
        let offset = self.panel.scroll.get_offset();
        let mut current_x = 0.0;
        let mut last_glyph_id = None;
        let local_x = target_x - rect.x1 as f32;

        for (i, c) in self.text.chars().enumerate() {
            let glyph_id = scaled_font.glyph_id(c);
            if let Some(last_id) = last_glyph_id {
                current_x += scaled_font.kern(last_id, glyph_id);
            }
            let advance = scaled_font.h_advance(glyph_id);

            if local_x < current_x + advance / 2.0 {
                return i as u32;
            }
            current_x += advance;
            last_glyph_id = Some(glyph_id);
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
        let font = &ctx.fonts[self.panel.base.settings.font_id.0];
        let scaled_font = font.as_scaled(self.scale);

        self.select_index_start = self.get_index(select_start.0 as f32, scaled_font);
        self.select_index_end = self.select_index_start;
    }
    fn set_end_caret(&mut self, select_end: (u16, u16), ctx: &LayoutContext) -> bool {
        let font = &ctx.fonts[self.panel.base.settings.font_id.0];
        let scaled_font = font.as_scaled(self.scale);

        let last_end_index = self.select_index_end;

        self.select_index_end = self.get_index(select_end.0 as f32, scaled_font);

        // if let Some(edit) = &mut self.as_edit_label_control_mut() {
        //     if edit.is_editable() {
        //         edit.set_cursor();
        //         edit.on_cursor();
        //     }
        // }

        if self.select_index_start != self.select_index_end
            && self.select_index_end != last_end_index
        {
            if let Some(tx) = &self.panel.base.settings.command_tx {
                let _ = tx.send(UiCommand::RequestRedraw());
            }
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
        self.as_with_animation().unwrap().restart_animations();
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
        self.as_with_animation().unwrap().restart_animations();
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
        self.select_index_start = self.cursor_index;
        self.select_index_end = self.cursor_index;
        self.as_with_animation().unwrap().restart_animations();
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
    fn print(&self, ctx: &mut GpuRenderContext, area: &Rect<i16>, offset: (f32, f32), level: u32) {
        if self.panel.base.visible_on_this_frame {
            let scroll_val = self.panel.scroll.get_offset();
            let content_offset = (scroll_val.0 + offset.0, scroll_val.1 + offset.1);

            self.panel.print(ctx, area, offset, level);
            //let level = level - 1;

            let rect = &self.panel.base.rect;
            // ctx.push_rect_sdf(
            //     rect,
            //     self.panel.base.settings.background_color,
            //     offset,
            //     0.0,
            //     self.panel.border,
            //     level,
            //     false,
            // );

            if self.select_index_start != self.select_index_end {
                ctx.push_rect_sdf(
                    &self.select_rect,
                    self.select_color,
                    offset,
                    5.0,
                    (0, 0.0),
                    level,
                    false,
                );
            }

            ctx.push_text(
                &self.text,
                rect.x1 as f32,
                rect.y1 as f32,
                self.scale,
                self.color,
                content_offset,
                level,
            );
            if self.cursor_need {
                ctx.push_rect_sdf(
                    &self.cursor,
                    self.cursor_color,
                    offset,
                    1.0,
                    (0, 0.0),
                    level,
                    false,
                );
            }
        }
    }

    fn resize(&mut self, area: &Rect<i16>, ctx: &LayoutContext, scroll_item: bool) -> Rect<i16> {
        if self.needs_layout {
            self.set_max_size(ctx);

            self.needs_layout = false
        }

        self.panel.resize(area, ctx, scroll_item);

        let rect = &self.panel.base.rect;

        let font = &ctx.fonts[self.panel.base.settings.font_id.0];

        let scaled_font = font.as_scaled(self.scale);

        let mut last_id = None;
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

            let gid = scaled_font.glyph_id(c);
            if let Some(l) = last_id {
                current_x += scaled_font.kern(l, gid);
            }
            current_x += scaled_font.h_advance(gid);
            last_id = Some(gid);
        }
        self.panel
            .scroll
            .set_height_width(rect.max.get_height(), current_x as i16);
        self.panel
            .scroll
            .set_slider_height_width(rect.min.get_height(), rect.min.get_width());

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
            let x1_offset = x_start;
            let x2_offset = x_end;

            let first_point = ((rect.x1 as f32 + x1_offset.min(x2_offset)) as i16, rect.y1);
            let second_point = ((rect.x1 as f32 + x1_offset.max(x2_offset)) as i16, rect.y2);

            self.select_rect = Rect::new_from_coord(first_point, second_point);
        }

        {
            let offset = self.panel.scroll.get_offset();
            self.panel
                .scroll
                .change_offset_x(rect.min.get_width() as f32 - x_cursor - offset.0);

            let x_offset = x_cursor.min(rect.min.get_width() as f32);
            let x_pos = (rect.x1 as f32 + x_offset) as i16;

            self.cursor =
                Rect::new_from_coord((x_pos, rect.y1 as i16), (x_pos + 1, rect.y2 as i16));
        }

        self.panel.base.rect.clone()

        // let target_w = rect.min.get_width() as f32;
        // let target_h = rect.min.get_height() as f32;

        // if target_w > 0.0 && target_h > 0.0 {
        //     // Считаем размер текста при маленьком базовом масштабе
        //     let base_scale = 10.0;
        //     let (w_base, h_base) = self.calculate_intrinsic_size(&font, base_scale);

        //     if w_base > 0 && h_base > 0 {
        //         let ratio_w = target_w / w_base as f32;
        //         let ratio_h = target_h / h_base as f32;

        //         // Выбираем меньший коэффициент (чтобы влезло и по ширине, и по высоте)
        //         let optimal_scale = base_scale * ratio_w.min(ratio_h);

        //         self.scale = optimal_scale.min(self.max_scale);
        //     }
        // }
    }
    fn under(&self, mx: u16, my: u16) -> bool {
        self.panel.under(mx, my)
    }

    fn hover(&self, mx: u16, my: u16) -> bool {
        self.panel.hover(mx, my)
    }

    add_drawable_control!();

    fn set_padding(&mut self, direction: Direction) {
        self.panel.set_padding(direction);
    }
    fn set_margin(&mut self, direction: Direction) {
        self.panel.set_margin(direction);
    }
    fn set_const_layout(&mut self, const_layout: Option<Box<dyn ConstLayout>>) {
        self.panel.set_const_layout(const_layout);
    }
    fn get_margin(&self) -> &Direction {
        self.panel.get_margin()
    }
    fn get_padding(&self) -> &Direction {
        self.panel.get_padding()
    }
    fn set_default_settings(&mut self, settings: &Settings) {
        self.panel.set_default_settings(settings);
    }

    fn as_base(&self) -> &Base {
        self.panel.as_base()
    }
    fn as_base_mut(&mut self) -> &mut Base {
        self.panel.as_base_mut()
    }

    fn as_panel_control_mut(&mut self) -> Option<&mut dyn PanelControl> {
        self.panel.as_panel_control_mut()
    }

    fn as_label_control_mut(&mut self) -> Option<&mut dyn LabelControl> {
        Some(self)
    }
    fn as_edit_label_control_mut(&mut self) -> Option<&mut dyn FullEditControl> {
        Some(self)
    }

    fn as_clickable(&mut self) -> Option<&mut dyn ClickableDrawable> {
        self.panel.as_clickable()
    }
    fn as_hoverable(&mut self) -> Option<&mut dyn HoverableDrawable> {
        self.panel.as_hoverable()
    }
    fn as_selectable(&mut self) -> Option<&mut dyn SelectableDrawable> {
        Some(self)
    }
    fn as_with_animation(&mut self) -> Option<&mut dyn AnimationDrawable> {
        self.panel.as_with_animation()
    }
    fn as_scrollable(&mut self) -> Option<&mut dyn super::interface::drawable::ScrollableDrawable> {
        self.panel.as_scrollable()
    }
}

impl PanelControl for Label {
    fn set_background(&mut self, color: u32) {
        self.panel.set_background(color);
    }
}
