use std::ops::Range;

use crate::window::{
    component::{base::area::Rect, theme::border::Border},
    wgpu::shape_vertex::{SHAPE_LINE, SHAPE_RECT, ShapeVertex},
};

pub struct GpuRenderContext {
    pub texts: Vec<TextData>,
    pub shape_vertices: Vec<ShapeVertex>,
    pub shape_indices: Vec<u32>,
    pub text_storage: String,
    pub shape_section_offsets: Vec<Range<usize>>,
    pub command_sections: Vec<GpuCommand>,
}

pub enum GpuCommand {
    Shape(Section),
    Text(Section),
    Unmask(Section),
}

pub struct Section {
    pub level: u32,
    pub command_index: u32,
    pub command_count: u32,
    pub is_mask: bool,
}

pub struct TextData {
    pub range: std::ops::Range<usize>,
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub color: u32, //[f32; 4],
}

impl GpuRenderContext {
    pub fn push_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: u32, level: u32) {
        let start = self.text_storage.len();
        self.text_storage.push_str(text);
        let end = self.text_storage.len();

        let final_x = x;
        let final_y = y;

        self.texts.push(TextData {
            range: start..end,
            x: final_x,
            y: final_y,
            size,
            color: color_to_gpu(color), //u32_to_rgba(color),
        });

        self.command_sections.push(GpuCommand::Text(Section {
            level,
            command_index: 0,
            command_count: 1,
            is_mask: false,
        }));
    }

    pub fn push_shape(
        &mut self,
        min_p: [f32; 2],   // Левый верхний угол Bounding Box
        max_p: [f32; 2],   // Правый нижний угол Bounding Box
        p_a: [f32; 2],     // Данные для SDF (Центр или Старт)
        p_b: [f32; 2],     // Данные для SDF (Размер или Конец)
        color: u32,        //[f32; 4],
        params: [f32; 4],  // [радиус_толщина, тип, сглаживание, 0.0]
        border_color: u32, //[f32; 4],
        level: u32,
        is_clip: bool,
        un_mask: bool,
    ) {
        let aa_padding = 2.0; // Запас для сглаживания
        let final_min = [min_p[0] - aa_padding, min_p[1] - aa_padding];
        let final_max = [max_p[0] + aa_padding, max_p[1] + aa_padding];

        let corners = [
            [final_min[0], final_min[1]], // TL
            [final_max[0], final_min[1]], // TR
            [final_min[0], final_max[1]], // BL
            [final_max[0], final_max[1]], // BR
        ];

        let v = corners.map(|pos| ShapeVertex {
            position: pos,
            color: color_to_gpu(color),
            p_a,
            p_b,
            params,
            border_color: color_to_gpu(border_color),
        });

        let start_vertex = self.shape_vertices.len();

        self.shape_vertices
            .extend_from_slice(&[v[0], v[1], v[2], v[3]]);
        let end_vertex = self.shape_vertices.len();

        self.shape_section_offsets.push(start_vertex..end_vertex);

        if un_mask {
            self.command_sections.push(GpuCommand::Unmask(Section {
                level: level,
                command_index: 0,
                command_count: 1,
                is_mask: is_clip,
            }));
        } else {
            self.command_sections.push(GpuCommand::Shape(Section {
                level: level,
                command_index: 0,
                command_count: 1,
                is_mask: is_clip,
            }));
        }
    }

    pub fn push_rect_sdf(
        &mut self,
        rect: &Rect<f32, u16>,
        color: u32,
        border: &Border,
        level: u32,
        is_clip: bool,
        un_mask: bool,
    ) {
        let x1 = rect.x1;
        let y1 = rect.y1;
        let x2 = rect.get_x2();
        let y2 = rect.get_y2();

        // Параметры для SDF шейдера
        let width = x2 - x1;
        let height = y2 - y1;
        let center = [x1 + width * 0.5, y1 + height * 0.5];
        let size = [width, height];

        self.push_shape(
            [x1, y1],
            [x2, y2],
            center,
            size,
            color, // color_rgba,
            [border.radius, SHAPE_RECT, 1.0, border.width],
            border.color,
            level,
            is_clip,
            un_mask,
        );
    }

    // Рисует линию графика
    pub fn push_line(
        &mut self,
        start_p: [f32; 2],
        end_p: [f32; 2],
        thickness: f32,
        color: u32,
        border: &Border,
        level: u32,
        is_clip: bool,
    ) {
        let pad = thickness + 2.0;

        let x1 = start_p[0].min(end_p[0]) - pad;
        let y1 = start_p[1].min(end_p[1]) - pad;
        let x2 = start_p[0].max(end_p[0]) + pad;
        let y2 = start_p[1].max(end_p[1]) + pad;

        // params: [половина толщины, тип: 1.0 (LINE), сглаживание: 1.0, 0.0]
        self.push_shape(
            [x1, y1], // min_p
            [x2, y2], // max_p
            start_p,  // p_a
            end_p,    // p_b
            color,    // color_rgba,
            [thickness * 0.5, SHAPE_LINE, 1.0, border.width],
            border.color,
            level,
            is_clip,
            false,
        );
    }

    pub fn clear(&mut self) {
        self.shape_vertices.clear();
        self.shape_indices.clear();
        self.texts.clear();
        self.text_storage.clear();
        self.shape_section_offsets.clear();
        self.command_sections.clear();
    }
}

fn color_to_gpu(color: u32) -> u32 {
    let a = (color >> 24) & 0xFF;
    let r = (color >> 16) & 0xFF;
    let g = (color >> 8) & 0xFF;
    let b = color & 0xFF;

    // Собираем в RGBA (порядок байтов для Unorm8x4 в wgpu)
    (a << 24) | (b << 16) | (g << 8) | r
}
