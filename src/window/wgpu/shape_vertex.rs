#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShapeVertex {
    pub position: [f32; 2], // Координаты вершины (куда растеризуем)
    pub color: [f32; 4],
    pub clip: [f32; 4],
    // Параметры фигуры:
    pub p_a: [f32; 2],    // Точка А (центр прямоугольника или старт линии)
    pub p_b: [f32; 2],    // Точка Б (размер прямоугольника или конец линии)
    pub params: [f32; 4], // [радиус/толщина, тип_фигуры, сглаживание, пусто]
}

// Типы фигур для params.y
const SHAPE_RECT: f32 = 0.0;
const SHAPE_LINE: f32 = 1.0;

impl ShapeVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
            0 => Float32x2, // position
            1 => Float32x4, // color
            2 => Float32x4, // clip
            3 => Float32x2, // p_a
            4 => Float32x2, // p_b
            5 => Float32x4, // params
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ShapeVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}
