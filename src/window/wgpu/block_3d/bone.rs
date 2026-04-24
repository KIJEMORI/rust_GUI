#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BoneInstance {
    pub transform: glam::Mat4, // Мировая матрица кости
    pub params: [f32; 4], // x: тип (текстура/капсула), y: радиус, z: ID текстуры, w: сила слияния
    pub color: u32,
    pub parent_id: i32, // Для иерархии (опционально)
    pub _padding: [u32; 2],
}
