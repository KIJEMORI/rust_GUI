use crate::window::{
    component::{
        block_3d::model::sdf_command::SDFCommandExt, managers::brick_3d_manager::BrickManager,
    },
    wgpu::block_3d::{brick_uniform::BakePushConstants, instance::Instance3DData},
};

pub struct Model {
    sdf_cmds: Vec<SDFCommandExt>,
    history_sdf_cmds: Vec<SDFCommandExt>,
    brick_manager: BrickManager,
}

pub const SHAPE_SPHERE: f32 = 1.0;
pub const SHAPE_BOX: f32 = 2.0;
pub const SHAPE_TORUS: f32 = 3.0;
pub const SHAPE_CILINDER: f32 = 4.0;
pub const SHAPE_CAPSULE: f32 = 5.0;

impl Default for Model {
    fn default() -> Self {
        Model::new()
    }
}

impl Model {
    pub fn new() -> Self {
        Model {
            sdf_cmds: Vec::with_capacity(1024),
            history_sdf_cmds: Vec::with_capacity(1024),
            brick_manager: BrickManager::default(),
        }
    }

    pub fn push(&mut self, sdf_command: SDFCommandExt) {
        self.sdf_cmds.push(sdf_command)
    }

    pub fn render(&mut self) -> (Vec<BakePushConstants>, Vec<Instance3DData>) {
        self.brick_manager
            .push_commands(&mut self.history_sdf_cmds, &mut self.sdf_cmds);

        self.brick_manager.get_commands(&self.history_sdf_cmds)
    }
}
