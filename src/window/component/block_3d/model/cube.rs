use crate::window::component::block_3d::{
    model::model::{Model, SHAPE_BOX},
    transform::Transform,
};

pub struct Cube {}

impl Cube {
    pub fn new(size: f32, position: [f32; 3]) -> Model {
        let transform = Transform {
            position: glam::Vec3::from_array(position),
            rotation: glam::Vec3::from_array([0.0, 0.0, 0.0]),
            scale: glam::Vec3::ONE,
        };
        Model {
            is_dirty: true,
            id_model: None,
            transform: transform,
            params: [SHAPE_BOX, size, 0.0, 0.0],
            color: 0xFFFF0000,
        }
    }
}
