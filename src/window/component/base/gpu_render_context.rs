use crate::window::{component::base::area::Rect, wgpu::vertex::Vertex};

pub struct GpuRenderContext {
    pub vertices: Vec<Vertex>,
    pub texts: Vec<TextData>,
}

pub struct TextData {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub color: [f32; 4],
}

impl GpuRenderContext {
    pub fn push_rect(&mut self, rect: &Rect<i16>, color: u32) {
        let x1 = rect.x1 as f32;
        let y1 = rect.y1 as f32;
        let x2 = rect.x2 as f32;
        let y2 = rect.y2 as f32;

        let color = u32_to_rgba(color);

        // Два треугольника (6 вершин)
        let v1 = Vertex {
            position: [x1, y1],
            color,
        };
        let v2 = Vertex {
            position: [x2, y1],
            color,
        };
        let v3 = Vertex {
            position: [x1, y2],
            color,
        };
        let v4 = Vertex {
            position: [x2, y2],
            color,
        };

        self.vertices.extend_from_slice(&[v1, v2, v3, v3, v2, v4]);
    }

    pub fn push_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: u32) {
        self.texts.push(TextData {
            text: text.to_string(),
            x,
            y,
            size,
            color: u32_to_rgba(color),
        });
    }
}

fn u32_to_rgba(color: u32) -> [f32; 4] {
    let a = ((color >> 24) & 0xFF) as f32 / 255.0;
    let r = ((color >> 16) & 0xFF) as f32 / 255.0;
    let g = ((color >> 8) & 0xFF) as f32 / 255.0;
    let b = (color & 0xFF) as f32 / 255.0;
    [r, g, b, a]
}
