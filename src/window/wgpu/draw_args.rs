#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndirectArgs {
    pub vertex_count: u32,   // Кол-во вершин в строке (end - start)
    pub instance_count: u32, // Всегда 1
    pub first_vertex: u32,   // Офсет start в твоем огромном буфере
    pub first_instance: u32, // 0 (или индекс строки, если нужно для шейдера)
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndexedIndirectArgs {
    pub index_count: u32,    // 6 для прямоугольника
    pub instance_count: u32, // 1
    pub first_index: u32,    // Смещение в Index Buffer
    pub base_vertex: i32,    // Смещение в Vertex Buffer
    pub first_instance: u32, // 0
}
