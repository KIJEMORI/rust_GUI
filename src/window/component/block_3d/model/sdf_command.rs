use glam::Vec3;

use crate::window::component::{
    block_3d::{
        model::model::{SHAPE_BOX, SHAPE_CAPSULE, SHAPE_SPHERE},
        transform::Transform,
    },
    managers::brick_3d_manager::{BRICK_RES, GRID_RES},
};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SDFCommandRaw {
    pub pos: [f32; 3],      // 12 байт
    pub rotation: [f32; 3], // 12 байт
    pub params: [f32; 4],   // 16 байт (выровнено по 16)
    pub color: u32,         // 4 байта
    pub _padding: u32,      // 4 байта
                            // 48 байт в сумме на комманду
}

#[derive(Copy, Clone)]
pub struct SDFCommandExt {
    pub transform: Transform,
    pub params: [f32; 4],
    pub color: u32,
}
pub trait SDFTrait {
    fn get_transform(&self) -> &Transform;
    fn get_params(&self) -> [f32; 4];
    fn get_affected_bricks(&self) -> Vec<u32>;
    fn get_aabb(&self) -> (Vec3, Vec3);
}

impl SDFTrait for SDFCommandExt {
    fn get_transform(&self) -> &Transform {
        &self.transform
    }
    fn get_params(&self) -> [f32; 4] {
        self.params
    }

    fn get_aabb(&self) -> (Vec3, Vec3) {
        let pos = self.get_transform().position;
        let p = self.get_params();
        let tag = p[0];
        let size = p[1];
        let extra = p[2]; // например, высота для цилиндра
        let k = 1.0; // запас на smin

        // Логика выбора бокса в зависимости от типа функции
        match tag {
            SHAPE_SPHERE => (pos - Vec3::splat(size + k), pos + Vec3::splat(size + k)),
            SHAPE_BOX => {
                // Для куба учитываем, что size — это половина стороны
                (pos - Vec3::splat(size + k), pos + Vec3::splat(size + k))
            }
            SHAPE_CAPSULE => {
                // Капсула вытянута по оси (например, Y), учитываем 'extra' как высоту
                let r = size + k;
                let h = extra;
                (pos - Vec3::new(r, h + r, r), pos + Vec3::new(r, h + r, r))
            }
            _ => (pos - Vec3::splat(size + k), pos + Vec3::splat(size + k)),
        }
    }

    fn get_affected_bricks(&self) -> Vec<u32> {
        let (min_p, max_p) = self.get_aabb();
        let offset = 16.0;
        let margin = 1.0;

        // Переводим в координаты сетки и сразу ограничиваем [0..31]
        // Используем малые отступы (epsilon), чтобы избежать дрожания на границах
        let start = (min_p + offset - margin).clamp(Vec3::ZERO, Vec3::splat(31.0));
        let end = (max_p + offset + margin).clamp(Vec3::ZERO, Vec3::splat(31.0));

        let mut affected = Vec::with_capacity(
            ((end.x - start.x + 1.0) * (end.y - start.y + 1.0) * (end.z - start.z + 1.0)) as usize,
        );

        for z in (start.z as u32)..=(end.z as u32) {
            let z_off = z * 1024;
            for y in (start.y as u32)..=(end.y as u32) {
                let y_off = y * 32;
                for x in (start.x as u32)..=(end.x as u32) {
                    affected.push(x + y_off + z_off);
                }
            }
        }
        affected
    }
}
