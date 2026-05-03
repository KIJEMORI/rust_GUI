use crate::window::component::block_3d::{
    model::{model::SHAPE_BOX, sdf_command::SDFCommandExt},
    transform::Transform,
};

pub struct Cube {}

impl Cube {
    pub fn new(size: f32, position: [f32; 3]) -> SDFCommandExt {
        let transform = Transform {
            position: glam::Vec3::from_array(position),
            rotation: glam::Vec3::from_array([0.0, 0.0, 0.0]),
            scale: glam::Vec3::ONE,
        };
        SDFCommandExt {
            transform: transform,
            params: [SHAPE_BOX, size, 0.0, 0.0],
            color: 0xFFFF0000,
        }
    }
}
