use glam::{UVec2, Vec3};
use std::collections::VecDeque;

pub const BRICK_RES: u32 = 16; // Разрешение одного кирпича
pub const ATLAS_WIDTH: u32 = 2048;
pub const ATLAS_HEIGHT: u32 = 4096;

pub struct BrickManager {
    free_slots: VecDeque<u32>, // Список ID свободных ячеек в атласе
    bricks_per_row: u32,
    total_slots: u32,
}

impl BrickManager {
    pub fn new() -> Self {
        let bricks_per_row = ATLAS_WIDTH / BRICK_RES;
        let bricks_per_col = ATLAS_HEIGHT / (BRICK_RES * BRICK_RES); // Z-слои идут вертикально
        let total_slots = bricks_per_row * bricks_per_col;

        let mut free_slots = VecDeque::new();
        for i in 0..total_slots {
            free_slots.push_back(i);
        }

        Self {
            free_slots,
            bricks_per_row,
            total_slots,
        }
    }

    /// Выделяет место в атласе и возвращает UV-координаты [min, max]
    pub fn alloc_brick(&mut self) -> Option<([f32; 2], [f32; 2], u32)> {
        let id = self.free_slots.pop_front()?;

        let x = (id % self.bricks_per_row) * BRICK_RES;
        let y = (id / self.bricks_per_row) * (BRICK_RES * BRICK_RES);

        let uv_min = [
            x as f32 / ATLAS_WIDTH as f32,
            y as f32 / ATLAS_HEIGHT as f32,
        ];
        let uv_max = [
            (x + BRICK_RES) as f32 / ATLAS_WIDTH as f32,
            (y + BRICK_RES * BRICK_RES) as f32 / ATLAS_HEIGHT as f32,
        ];

        Some((uv_min, uv_max, id))
    }

    pub fn free_brick(&mut self, id: u32) {
        self.free_slots.push_back(id);
    }
}
