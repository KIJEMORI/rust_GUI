use std::any::Any;

use wgpu_glyph::ab_glyph::{Font, FontArc, ScaleFont};
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
    fn set_text_str(&mut self, text: &str) {
        self.text = text.to_string();
    }
}

impl Drawable for Label {
    fn print(&self, ctx: &mut GpuRenderContext) {
        self.panel.print(ctx);

        let rect = &self.panel.base.rect;
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
    fn set_const_layout(&mut self, const_layout: &dyn ConstLayout) {
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
