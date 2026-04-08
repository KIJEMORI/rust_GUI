use std::ops::Range;

use crate::window::{
    component::base::area::Rect,
    wgpu::{shape_vertex::ShapeVertex, vertex::Vertex},
};

pub struct GpuRenderContext {
    //pub vertices: Vec<Vertex>,
    pub texts: Vec<TextData>,
    pub shape_vertices: Vec<ShapeVertex>,
    pub text_storage: String,
    pub shape_section_offsets: Vec<Range<usize>>,
}

pub struct TextData {
    pub range: std::ops::Range<usize>,
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub color: [f32; 4],
    pub clip: [f32; 4],
}

impl GpuRenderContext {
    // pub fn push_rect(
    //     &mut self,
    //     rect: &Rect<i16>,
    //     parent_rect: Option<&Rect<i16>>,
    //     color: u32,
    //     offset: (f32, f32),
    // ) {
    //     let x1 = rect.x1 as f32 + offset.0;
    //     let y1 = rect.y1 as f32 + offset.1;
    //     let x2 = rect.x2 as f32 + offset.0;
    //     let y2 = rect.y2 as f32 + offset.1;

    //     let color = u32_to_rgba(color);
    //     let mut clip = [x1, y1, x2, y2];
    //     if let Some(parent) = parent_rect {
    //         let cx1 = (x1).max(parent.x1 as f32);
    //         let cy1 = (y1).max(parent.y1 as f32);
    //         let cx2 = (x2).min(parent.x2 as f32);
    //         let cy2 = (y2).min(parent.y2 as f32);

    //         clip = [cx1, cy1, cx2, cy2];
    //     }

    //     // Два треугольника (6 вершин)
    //     let v1 = Vertex {
    //         position: [x1, y1],
    //         color,
    //         clip,
    //     };
    //     let v2 = Vertex {
    //         position: [x2, y1],
    //         color,
    //         clip,
    //     };
    //     let v3 = Vertex {
    //         position: [x1, y2],
    //         color,
    //         clip,
    //     };
    //     let v4 = Vertex {
    //         position: [x2, y2],
    //         color,
    //         clip,
    //     };
    //     self.vertices.reserve(6);
    //     self.vertices.push(v1);
    //     self.vertices.push(v2);
    //     self.vertices.push(v3);
    //     self.vertices.push(v3);
    //     self.vertices.push(v2);
    //     self.vertices.push(v4);
    // }

    pub fn push_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: u32,
        rect: &Rect<i16>,
        parent_rect: Option<&Rect<i16>>,
        offset: (f32, f32),
    ) {
        let start = self.text_storage.len();
        self.text_storage.push_str(text);
        let end = self.text_storage.len();

        let final_x = x + offset.0;
        let final_y = y + offset.1;

        let x1 = rect.x1 as f32 + offset.0;
        let y1 = rect.y1 as f32 + offset.1;
        let x2 = rect.x2 as f32 + offset.0;
        let y2 = rect.y2 as f32 + offset.1;

        let mut clip = [x1, y1, x2, y2];
        if let Some(parent) = parent_rect {
            let cx1 = (x1).max(parent.x1 as f32);
            let cy1 = (y1).max(parent.y1 as f32);
            let cx2 = (x2).min(parent.x2 as f32);
            let cy2 = (y2).min(parent.y2 as f32);

            clip = [cx1, cy1, cx2, cy2];
        }

        self.texts.push(TextData {
            range: start..end,
            x: final_x,
            y: final_y,
            size,
            color: u32_to_rgba(color),
            clip,
        });
    }

    pub fn push_shape(
        &mut self,
        min_p: [f32; 2], // Левый верхний угол Bounding Box
        max_p: [f32; 2], // Правый нижний угол Bounding Box
        p_a: [f32; 2],   // Данные для SDF (Центр или Старт)
        p_b: [f32; 2],   // Данные для SDF (Размер или Конец)
        color: [f32; 4],
        clip: [f32; 4],
        params: [f32; 4], // [радиус_толщина, тип, сглаживание, 0.0]
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
            color,
            clip,
            p_a,
            p_b,
            params,
        });

        let start_vertex = self.shape_vertices.len();

        self.shape_vertices
            .extend_from_slice(&[v[0], v[1], v[2], v[2], v[1], v[3]]);
        let end_vertex = self.shape_vertices.len();

        self.shape_section_offsets.push(start_vertex..end_vertex);
    }

    pub fn push_rect_sdf(
        &mut self,
        rect: &Rect<i16>,
        parent_rect: Option<&Rect<i16>>,
        color: u32,
        offset: (f32, f32),
        radius: f32,
    ) {
        let x1 = rect.x1 as f32 + offset.0;
        let y1 = rect.y1 as f32 + offset.1;
        let x2 = rect.x2 as f32 + offset.0;
        let y2 = rect.y2 as f32 + offset.1;

        let color_rgba = u32_to_rgba(color);

        // Параметры для SDF шейдера
        let width = x2 - x1;
        let height = y2 - y1;
        let center = [x1 + width * 0.5, y1 + height * 0.5];
        let size = [width, height];

        let mut clip = [x1, y1, x2, y2];
        if let Some(parent) = parent_rect {
            clip = [
                x1.max(parent.x1 as f32),
                y1.max(parent.y1 as f32),
                x2.min(parent.x2 as f32),
                y2.min(parent.y2 as f32),
            ];
        }

        self.push_shape(
            [x1, y1],
            [x2, y2],
            center,
            size,
            color_rgba,
            clip,
            [radius, 0.0, 1.0, 0.0],
        );
    }

    // Рисует линию графика
    pub fn push_line(
        &mut self,
        start_p: [f32; 2],
        end_p: [f32; 2],
        thickness: f32,
        color: u32,
        clip_rect: &Rect<i16>,
    ) {
        let color_rgba = u32_to_rgba(color);

        let pad = thickness + 2.0;

        let x1 = start_p[0].min(end_p[0]) - pad;
        let y1 = start_p[1].min(end_p[1]) - pad;
        let x2 = start_p[0].max(end_p[0]) + pad;
        let y2 = start_p[1].max(end_p[1]) + pad;

        let clip = [
            clip_rect.x1 as f32,
            clip_rect.y1 as f32,
            clip_rect.x2 as f32,
            clip_rect.y2 as f32,
        ];

        // params: [половина толщины, тип: 1.0 (LINE), сглаживание: 1.0, 0.0]
        self.push_shape(
            [x1, y1], // min_p
            [x2, y2], // max_p
            start_p,  // p_a
            end_p,    // p_b
            color_rgba,
            clip,
            [thickness * 0.5, 1.0, 1.0, 0.0],
        );
    }

    pub fn clear(&mut self) {
        self.shape_vertices.clear();
        self.texts.clear();
        self.text_storage.clear();
        self.shape_section_offsets.clear();
    }
}

fn u32_to_rgba(color: u32) -> [f32; 4] {
    let a = ((color >> 24) & 0xFF) as f32 / 255.0;
    let r = ((color >> 16) & 0xFF) as f32 / 255.0;
    let g = ((color >> 8) & 0xFF) as f32 / 255.0;
    let b = (color & 0xFF) as f32 / 255.0;
    [r, g, b, a]
}
