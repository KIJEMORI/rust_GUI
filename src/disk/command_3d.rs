#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawCommand {
    pub pos: [f32; 3],      // 12 байт
    pub rotation: [f32; 3], // 12 байт
    pub params: [f32; 4],   // 16 байт (выровнено по 16)
    pub color: u32,         // 4 байта
    pub _padding: u32,     // 4 байта
                            // 48 байт в сумме на комманду
}
