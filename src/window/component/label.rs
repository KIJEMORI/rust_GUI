use std::any::Any;
use std::cell::RefCell;

use crate::add_drawable_control;
use crate::window::component::base::area::Rect;
use crate::window::component::base::base::Base;
use crate::window::component::base::component_type::ComponentType;
use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::render_context::RenderContext;
use crate::window::component::interface::component_control::{LabelControl, PanelControl};
use crate::window::component::interface::const_layout::ConstLayout;
use crate::window::component::interface::drawable::Drawable;
use crate::window::component::layout::const_base_layout::Direction;
use crate::window::component::panel::Panel;

pub struct Label {
    pub text: String,
    pub panel: Panel,
    pub scale: f32,
    color: u32,
}

impl Label {
    pub fn new(text: String) -> Self {
        Label {
            text,
            panel: Panel::default(),
            scale: 20.0,
            color: 0xFF000000,
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

    // fn update_size(&mut self) {
    //     let rect = &self.panel.base.rect;
    //     let max_w = rect.min.get_width() as f32;
    //     let max_h = rect.min.get_height() as f32;

    //     if max_w <= 0.0 || max_h <= 0.0 {
    //         return;
    //     }

    //     // 1. Бинарный поиск оптимального Scale [14.0 ... max_scale]
    //     let mut low = 14.0;
    //     let mut high = self.max_scale.x;
    //     let mut best_scale = low;

    //     for _ in 0..10 {
    //         // 10 итераций дают точность ~0.1 для диапазона 100
    //         let mid = (low + high) / 2.0;
    //         if self.check_fits(mid, max_w, max_h) {
    //             best_scale = mid;
    //             low = mid;
    //         } else {
    //             high = mid;
    //         }
    //     }

    //     self.scale = Scale::uniform(best_scale);

    //     // 2. Финальная генерация глифов с переносами
    //     self.glyphs = self.layout_with_wrap(self.scale, max_w);
    // }
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
        self.scale = scale as f32;
    }
    fn set_text(&mut self, text: String) {
        self.text = text;
    }
    fn set_text_str(&mut self, text: &str) {
        self.text = text.to_string();
    }
}

impl Drawable for Label {
    fn print(&self, ctx: &mut GpuRenderContext, _area: &Rect<u16>) {
        self.panel.print(ctx, _area);

        let rect = &self.panel.base.rect;
        ctx.push_text(
            &self.text,
            rect.x1 as f32,
            rect.y2 as f32,
            self.scale,
            self.color,
        );
        // let window_width = ctx.window_size.get_width();
        // let window_height = ctx.window_size.get_height();

        // for glyph in self.glyphs.iter() {
        //     if let Some(bb) = glyph.pixel_bounding_box() {
        //         glyph.draw(|gx, gy, v| {
        //             if v < 0.01 {
        //                 return;
        //             }
        //             // bb.min.x/y уже учитывают позицию каретки
        //             let x = bb.min.x + gx as i32 + rect.x1 as i32;
        //             let y = bb.min.y + gy as i32 + rect.y1 as i32;

        //             if x >= 0
        //                 && x < rect.x2.min(window_width) as i32
        //                 && y >= 0
        //                 && y < rect.y2.min(window_height) as i32
        //             {
        //                 let index = y as usize * window_width as usize + x as usize;
        //                 if let Some(pixel) = ctx.buffer.get_mut(index) {
        //                     *pixel = blend_colors(*pixel, self.color, v);
        //                 }
        //             }
        //         });
        //     }
        // }
    }

    fn resize(&mut self, area: &Rect<u16>) -> Rect<u16> {
        self.panel.resize(area);

        return self.panel.base.rect.clone();
    }
    fn get_type(&self) -> ComponentType {
        ComponentType::Label
    }
    fn click(&self, x: u16, y: u16) -> bool {
        self.panel.click(x, y)
    }

    add_drawable_control!();
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

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
}

#[inline(always)]
fn blend_colors(bg: u32, fg: u32, alpha: f32) -> u32 {
    if alpha <= 0.0 {
        return bg;
    }
    if alpha >= 1.0 {
        return fg;
    }

    let bg_r = ((bg >> 16) & 0xff) as f32;
    let bg_g = ((bg >> 8) & 0xff) as f32;
    let bg_b = (bg & 0xff) as f32;

    let fg_r = ((fg >> 16) & 0xff) as f32;
    let fg_g = ((fg >> 8) & 0xff) as f32;
    let fg_b = (fg & 0xff) as f32;

    let inv_alpha = 1.0 - alpha;

    let r = (fg_r * alpha + bg_r * inv_alpha) as u32;
    let g = (fg_g * alpha + bg_g * inv_alpha) as u32;
    let b = (fg_b * alpha + bg_b * inv_alpha) as u32;

    (r << 16) | (g << 8) | b
}

impl PanelControl for Label {
    fn set_background(&mut self, color: u32) {
        self.panel.set_background(color);
    }
}
