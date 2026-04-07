#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndirectArgs {
    pub vertex_count: u32,   // Кол-во вершин в строке (end - start)
    pub instance_count: u32, // Всегда 1
    pub first_vertex: u32,   // Офсет start в твоем огромном буфере
    pub first_instance: u32, // 0 (или индекс строки, если нужно для шейдера)
}
