pub const UNION: f32 = 0.0;
pub const INTERSECTION: f32 = 1.0;
pub const SUBTRACTION: f32 = 2.0;
pub const SMOOTH: f32 = 3.0;
pub const COLOR_DRAWING: f32 = 4.0;

pub fn get_type_union(type_union: f32) -> f32 {
    match type_union {
        INTERSECTION | SUBTRACTION | SMOOTH | COLOR_DRAWING => type_union,
        _ => UNION,
    }
}
