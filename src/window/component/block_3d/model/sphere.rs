use crate::window::component::block_3d::{
    model::{model::SHAPE_SPHERE, sdf_command::SDFCommandExt},
    transform::Transform,
    type_union,
};

pub struct Sphere {}

impl Sphere {
    pub fn new(radius: f32, position: [f32; 3]) -> SDFCommandExt {
        let transform = Transform {
            position: glam::Vec3::from_array(position),
            rotation: glam::Vec3::from_array([0.0, 0.0, 0.0]),
            scale: glam::Vec3::from_array([1.0, 1.0, 1.0]),
        };
        SDFCommandExt {
            transform: transform,
            params: [SHAPE_SPHERE, radius, 0.0, 0.0],
            color: 0xFF555555,
            material_id: 0,
            type_union: type_union::SMOOTH,
        }
    }
    pub fn new_with_color(radius: f32, position: [f32; 3], color: u32) -> SDFCommandExt {
        let transform = Transform {
            position: glam::Vec3::from_array(position),
            rotation: glam::Vec3::from_array([0.0, 0.0, 0.0]),
            scale: glam::Vec3::from_array([1.0, 1.0, 1.0]),
        };
        SDFCommandExt {
            transform: transform,
            params: [SHAPE_SPHERE, radius, 0.0, 0.0],
            color: color,
            material_id: 0,
            type_union: type_union::SMOOTH,
        }
    }
}
