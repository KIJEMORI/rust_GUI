use std::collections::VecDeque;

use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::window::{
    component::{
        base::gpu_render_context::color_to_gpu,
        block_3d::model::sdf_command::{SDFCommandExt, SDFTrait},
    },
    wgpu::block_3d::{brick_uniform::BakePushConstants, instance::Instance3DData},
};

pub const GRID_RES: u32 = 32;
pub const BRICK_RES: u32 = 8; // Разрешение одного кирпича
pub const ATLAS_WIDTH: u32 = 4096;
pub const ATLAS_HEIGHT: u32 = 4096;

pub struct BrickManager {
    log_book: FxHashMap<u32, Vec<u32>>,
    dirty_bricks: VecDeque<u32>,
    dirty_set: FxHashSet<u32>,
}

impl Default for BrickManager {
    fn default() -> Self {
        BrickManager::new()
    }
}

const MAX_COUNT_COMMANDS: u32 = 1000;

impl BrickManager {
    pub fn new() -> Self {
        BrickManager {
            log_book: FxHashMap::default(),
            dirty_bricks: VecDeque::with_capacity(1024),
            dirty_set: FxHashSet::default(),
        }
    }

    pub fn push_commands(
        &mut self,
        history: &mut Vec<SDFCommandExt>,
        commands: &mut Vec<SDFCommandExt>,
    ) {
        let processed_cmds: Vec<(SDFCommandExt, Vec<u32>)> = commands
            .into_par_iter()
            .map(|cmd| {
                let indices = cmd.get_affected_bricks();
                (cmd.clone(), indices)
            })
            .collect();

        commands.clear();

        for (cmd, indices) in processed_cmds {
            history.push(cmd);
            let index_cmd = (history.len() - 1) as u32;
            for index in indices {
                self.log_book.entry(index).or_default().push(index_cmd);

                if self.dirty_set.insert(index) {
                    self.dirty_bricks.push_back(index);
                }
            }
        }
    }
    pub fn get_commands(
        &mut self,
        history: &[SDFCommandExt],
    ) -> (Vec<BakePushConstants>, Vec<Instance3DData>) {
        let mut total_cmds_in_batch = 0;
        let mut bake_commands = Vec::new();
        let mut instance_3d_data = Vec::new();

        while let Some(&dirty_brick_id) = self.dirty_bricks.front() {
            let cmds_indices = self.log_book.get(&dirty_brick_id);
            let cmds_len = cmds_indices.map(|c| c.len()).unwrap_or(0) as u32;

            if !bake_commands.is_empty() && total_cmds_in_batch + cmds_len > MAX_COUNT_COMMANDS {
                break;
            }

            self.dirty_bricks.pop_front();
            self.dirty_set.remove(&dirty_brick_id);

            bake_commands.push(BakePushConstants {
                brick_id: dirty_brick_id,
                start_instance: instance_3d_data.len() as u32,
                count: cmds_len,
                padding: 0,
            });

            if let Some(indices) = cmds_indices {
                for &idx in indices {
                    if let Some(sdf_cmd) = history.get(idx as usize) {
                        let model_matrix = sdf_cmd.transform.to_matrix(); // Mat4

                        let scale_x = model_matrix.col(0).truncate().length();
                        let scale_y = model_matrix.col(1).truncate().length();
                        let scale_z = model_matrix.col(2).truncate().length();
                        let uniform_scale = (scale_x + scale_y + scale_z) / 3.0; // средний масштаб

                        let inv_matrix = model_matrix.inverse();

                        let mut params = sdf_cmd.params;
                        params[3] = uniform_scale;

                        instance_3d_data.push(Instance3DData {
                            inv_transform: inv_matrix.to_cols_array(), // Передаем честно в column-major
                            params,
                            color: color_to_gpu(sdf_cmd.color),
                            _padding: [0; 3],
                        });
                    }
                }
                total_cmds_in_batch += cmds_len;
            }
        }

        (bake_commands, instance_3d_data)
    }

    fn get_id(x: u32, y: u32, z: u32) -> u32 {
        x + (y * 32) + (z * 1024)
    }
}
