use crate::window::{component::base::area::Rect, wgpu::vertex::Vertex};

pub struct GpuRenderContext {
    pub vertices: Vec<Vertex>,
    pub texts: Vec<TextData>,
    pub text_storage: String,
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
    pub fn push_rect(&mut self, rect: &Rect<i16>, parent_rect: Option<&Rect<i16>>, color: u32) {
        let x1 = rect.x1 as f32;
        let y1 = rect.y1 as f32;
        let x2 = rect.x2 as f32;
        let y2 = rect.y2 as f32;

        let color = u32_to_rgba(color);
        let mut clip = [x1, y1, x2, y2];
        if let Some(parent) = parent_rect {
            let cx1 = (rect.x1 as f32).max(parent.x1 as f32);
            let cy1 = (rect.y1 as f32).max(parent.y1 as f32);
            let cx2 = (rect.x2 as f32).min(parent.x2 as f32);
            let cy2 = (rect.y2 as f32).min(parent.y2 as f32);

            clip = [cx1, cy1, cx2, cy2];
        }

        // Два треугольника (6 вершин)
        let v1 = Vertex {
            position: [x1, y1],
            color,
            clip,
        };
        let v2 = Vertex {
            position: [x2, y1],
            color,
            clip,
        };
        let v3 = Vertex {
            position: [x1, y2],
            color,
            clip,
        };
        let v4 = Vertex {
            position: [x2, y2],
            color,
            clip,
        };
        self.vertices.reserve(6);
        self.vertices.push(v1);
        self.vertices.push(v2);
        self.vertices.push(v3);
        self.vertices.push(v3);
        self.vertices.push(v2);
        self.vertices.push(v4);
    }

    pub fn push_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: u32,
        parent_rect: Option<&Rect<i16>>,
    ) {
        let start = self.text_storage.len();
        self.text_storage.push_str(text);
        let end = self.text_storage.len();

        let clip = parent_rect
            .map(|r| [r.x1 as f32, r.y1 as f32, r.x2 as f32, r.y2 as f32])
            .unwrap_or([-1e6, -1e6, 1e6, 1e6]);

        self.texts.push(TextData {
            range: start..end,
            x,
            y,
            size,
            color: u32_to_rgba(color),
            clip,
        });
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.texts.clear();
        self.text_storage.clear();
    }
}

fn u32_to_rgba(color: u32) -> [f32; 4] {
    let a = ((color >> 24) & 0xFF) as f32 / 255.0;
    let r = ((color >> 16) & 0xFF) as f32 / 255.0;
    let g = ((color >> 8) & 0xFF) as f32 / 255.0;
    let b = (color & 0xFF) as f32 / 255.0;
    [r, g, b, a]
}
