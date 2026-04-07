use std::any::Any;

use wgpu_glyph::ab_glyph::{Font, FontArc, PxScaleFont, ScaleFont};
use wgpu_glyph::{FontId, GlyphPositioner, Layout, SectionGeometry, SectionText};

use crate::add_drawable_control;
use crate::window::component::base::area::Rect;
use crate::window::component::base::base::Base;
use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::settings::Settings;
use crate::window::component::base::ui_command::UiCommand;
use crate::window::component::interface::component_control::{LabelControl, PanelControl};
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::drawable::Drawable;
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
    select_index_start: usize,
    select_index_end: usize,
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
    fn get_index(&self, target_x: f32, scaled_font: PxScaleFont<&FontArc>) -> usize {
        let rect = &self.panel.base.rect;
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
                return i;
            }
            current_x += advance;
            last_glyph_id = Some(glyph_id);
        }
        self.text.chars().count() // Если кликнули правее всего текста
    }
}

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
    fn get_text(&self) -> String {
        self.text.clone()
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

        self.select_index_start = self.get_index(select_start.0 as f32, scaled_font)
    }
    fn set_end_caret(&mut self, select_end: (u16, u16), ctx: &LayoutContext) {
        let font = &ctx.fonts[self.panel.base.settings.font_id.0];
        let scaled_font = font.as_scaled(self.scale);

        let last_end_index = self.select_index_end;

        self.select_index_end = self.get_index(select_end.0 as f32, scaled_font);

        if self.select_index_start != self.select_index_end
            && self.select_index_end != last_end_index
        {
            if let Some(tx) = &self.panel.base.settings.command_tx {
                let _ = tx.send(UiCommand::RequestRedraw());
            }
        }
    }
}

impl Drawable for Label {
    fn print(&self, ctx: &mut GpuRenderContext) {
        self.panel.print(ctx);

        let rect = &self.panel.base.rect;

        if self.select_index_start != self.select_index_end {
            ctx.push_rect(&self.select_rect, self.select_color);
        }
        ctx.push_text(
            &self.text,
            rect.x1 as f32,
            rect.y1 as f32,
            self.scale,
            self.color,
        );
    }

    fn resize(&mut self, area: &Rect<i16>, ctx: &LayoutContext) -> Rect<i16> {
        if self.needs_layout {
            self.set_max_size(ctx);

            self.needs_layout = false
        }

        self.panel.resize(area, ctx);

        let rect = &self.panel.base.rect;

        let target_w = rect.min.get_width() as f32;
        let target_h = rect.min.get_height() as f32;

        let font = &ctx.fonts[self.panel.base.settings.font_id.0];

        if self.select_index_start != self.select_index_end {
            let scaled_font = font.as_scaled(self.scale);

            let get_x = |index: usize| -> f32 {
                let mut x = 0.0;
                let mut last_id = None;
                for (i, c) in self.text.chars().enumerate() {
                    if i >= index {
                        break;
                    }
                    let gid = scaled_font.glyph_id(c);
                    if let Some(l) = last_id {
                        x += scaled_font.kern(l, gid);
                    }
                    x += scaled_font.h_advance(gid);
                    last_id = Some(gid);
                }
                x
            };

            let x1_offset = get_x(self.select_index_start);
            let x2_offset = get_x(self.select_index_end);

            let first_point = ((rect.x1 as f32 + x1_offset.min(x2_offset)) as i16, rect.y1);
            let second_point = ((rect.x1 as f32 + x1_offset.max(x2_offset)) as i16, rect.y2);

            self.select_rect = Rect::new_from_coord(first_point, second_point);
        }

        if target_w > 0.0 && target_h > 0.0 {
            // Считаем размер текста при маленьком базовом масштабе
            let base_scale = 10.0;
            let (w_base, h_base) = self.calculate_intrinsic_size(&font, base_scale);

            if w_base > 0 && h_base > 0 {
                // Вычисляем, во сколько раз нам нужно увеличить масштаб,
                // чтобы заполнить target_w или target_h
                let ratio_w = target_w / w_base as f32;
                let ratio_h = target_h / h_base as f32;

                // Выбираем меньший коэффициент (чтобы влезло и по ширине, и по высоте)
                let optimal_scale = base_scale * ratio_w.min(ratio_h);

                // Ограничиваем максимальным значением
                self.scale = optimal_scale.min(self.max_scale);
            }
        }

        self.panel.base.rect.clone()
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
    fn is_clickable(&mut self) -> bool {
        if self.panel.is_clickable() {
            if let Some(cmd) = &mut self.panel.get_command_click() {
                cmd.fill_ref(&self.panel.base.get_shared());
                let action = cmd.clone();
                self.set_on_click(action);
            }
            return true;
        }
        false
    }
    fn set_on_click(&mut self, action: UiCommand) {
        self.panel.set_on_click(action);
    }
    fn on_click(&self) {
        self.panel.on_click();
    }
    fn is_selectable(&self) -> bool {
        true
    }
    fn is_hoverable(&mut self) -> bool {
        if self.panel.is_clickable() {
            if let Some(cmd) = &mut self.panel.get_command_on_mouse_enter() {
                cmd.fill_ref(&self.panel.base.get_shared());
                let action = cmd.clone();
                self.set_on_mouse_enter(action);
            }
            if let Some(cmd) = &mut self.panel.get_command_on_mouse_leave() {
                cmd.fill_ref(&self.panel.base.get_shared());
                let action = cmd.clone();
                self.set_on_mouse_leave(action);
            }
            return true;
        }
        false
    }
    fn hover(&self, mx: u16, my: u16) -> bool {
        self.panel.hover(mx, my)
    }
    fn set_on_mouse_enter(&mut self, action: UiCommand) {
        self.panel.set_on_mouse_enter(action);
    }
    fn set_on_mouse_leave(&mut self, action: UiCommand) {
        self.panel.set_on_mouse_leave(action);
    }
    fn on_mouse_enter(&self) {
        self.panel.on_mouse_enter();
    }
    fn on_mouse_leave(&self) {
        self.panel.on_mouse_leave();
    }
    fn as_label_control_mut(&mut self) -> Option<&mut dyn LabelControl> {
        Some(self)
    }
    fn as_base(&self) -> &Base {
        self.panel.as_base()
    }
    fn as_base_mut(&mut self) -> &mut Base {
        self.panel.as_base_mut()
    }
}

// #[inline(always)]
// fn blend_colors(bg: u32, fg: u32, alpha: f32) -> u32 {
//     if alpha <= 0.0 {
//         return bg;
//     }
//     if alpha >= 1.0 {
//         return fg;
//     }

//     let bg_r = ((bg >> 16) & 0xff) as f32;
//     let bg_g = ((bg >> 8) & 0xff) as f32;
//     let bg_b = (bg & 0xff) as f32;

//     let fg_r = ((fg >> 16) & 0xff) as f32;
//     let fg_g = ((fg >> 8) & 0xff) as f32;
//     let fg_b = (fg & 0xff) as f32;

//     let inv_alpha = 1.0 - alpha;

//     let r = (fg_r * alpha + bg_r * inv_alpha) as u32;
//     let g = (fg_g * alpha + bg_g * inv_alpha) as u32;
//     let b = (fg_b * alpha + bg_b * inv_alpha) as u32;

//     (r << 16) | (g << 8) | b
// }

impl PanelControl for Label {
    fn set_background(&mut self, color: u32) {
        self.panel.set_background(color);
    }
}
