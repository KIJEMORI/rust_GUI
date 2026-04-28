use crate::window::component::block_3d::transform::Transform;

pub struct Model {
    pub is_dirty: bool,
    pub id_model: Option<u32>,
    pub transform: Transform,
    pub params: [f32; 4],
    pub color: u32,
}

pub const SHAPE_SPHERE: f32 = 1.0;
pub const SHAPE_BOX: f32 = 2.0;
pub const SHAPE_TORUS: f32 = 3.0;
pub const SHAPE_CILINDER: f32 = 4.0;
pub const SHAPE_CAPSULE: f32 = 5.0;

pub trait ModelTrait {
    fn get_transform(&self) -> &Transform;
    fn get_id(&self) -> Option<u32>;
    fn is_dirty(&self) -> bool;
    fn set_dirty(&mut self, tumbler: bool);
    fn get_params(&self) -> [f32; 4];
}

impl ModelTrait for Model {
    fn get_transform(&self) -> &Transform {
        &self.transform
    }
    fn get_id(&self) -> Option<u32> {
        self.id_model
    }
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn set_dirty(&mut self, tumbler: bool) {
        self.is_dirty = tumbler;
    }
    fn get_params(&self) -> [f32; 4] {
        self.params
    }
}
