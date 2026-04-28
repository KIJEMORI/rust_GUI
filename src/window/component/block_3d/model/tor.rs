use crate::window::component::block_3d::{
    model::model::{Model, SHAPE_TORUS},
    transform::Transform,
};

pub struct Tor {}

impl Tor {
    pub fn new(size1: f32, size2: f32, position: [f32; 3]) -> Model {
        let transform = Transform {
            position: glam::Vec3::from_array(position),
            rotation: glam::Vec3::from_array([0.0, 0.0, 0.0]),
            scale: glam::Vec3::ONE,
        };
        Model {
            is_dirty: true,
            id_model: None,
            transform: transform,
            params: [SHAPE_TORUS, size1, size2, 0.0],
            color: 0xFFFF0000,
        }
    }
}
