use std::hash::{Hash, Hasher};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub clip: [f32; 4],
    pub section_id: u32,
}

impl TextVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl PartialEq for TextVertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.uv == other.uv
            && self.color == other.color
            && self.clip == other.clip
    }
}

impl Eq for TextVertex {}

impl Hash for TextVertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let fields = [
            &self.position[..],
            &self.uv[..],
            &self.color[..],
            &self.clip[..],
        ];
        for field in fields {
            for &val in field {
                val.to_bits().hash(state);
            }
        }
    }
}

pub fn push_glyph_to_vertices_raw(
    px: glyph_brush::ab_glyph::Rect,
    tex: glyph_brush::ab_glyph::Rect,
    clip: [f32; 4],
    color: [f32; 4],
    section_id: u32,
) -> [TextVertex; 6] {
    let v_tl = TextVertex {
        position: [px.min.x, px.min.y],
        uv: [tex.min.x, tex.min.y],
        color,
        clip,
        section_id,
    };
    let v_tr = TextVertex {
        position: [px.max.x, px.min.y],
        uv: [tex.max.x, tex.min.y],
        color,
        clip,
        section_id,
    };
    let v_bl = TextVertex {
        position: [px.min.x, px.max.y],
        uv: [tex.min.x, tex.max.y],
        color,
        clip,
        section_id,
    };
    let v_br = TextVertex {
        position: [px.max.x, px.max.y],
        uv: [tex.max.x, tex.max.y],
        color,
        clip,
        section_id,
    };

    [v_tl, v_tr, v_bl, v_tr, v_br, v_bl]
}
