use crate::window::component::block_3d::type_union::{self, get_type_union};

pub struct Pencil3D {
    pub type_union: f32,
    pub color: u32,
}

impl Default for Pencil3D {
    fn default() -> Self {
        Pencil3D {
            type_union: type_union::UNION,
            color: 0xFF555555,
        }
    }
}

impl Pencil3D {
    pub fn new(type_union: f32, color: u32) -> Self {
        Pencil3D {
            type_union: get_type_union(type_union),
            color: color,
        }
    }

    pub fn set_color_pencil(&mut self, color: u32) {
        self.color = color;
    }

    pub fn set_union_pencil(&mut self, type_union: f32) {
        self.type_union = get_type_union(type_union);
    }
}
