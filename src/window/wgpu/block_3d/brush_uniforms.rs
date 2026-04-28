use glam::Vec3;

use crate::window::component::managers::brick_3d_manager::{ATLAS_WIDTH, BRICK_RES};

// Структуры для Uniform-буферов шейдера
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BrushUniform {
    pos: [f32; 3], // Локальная позиция в кирпиче (0..16)
    radius: f32,
    op_type: f32,    // 1.0 = Add, 2.0 = Sub
    smoothness: f32, // k для smin
    _padding: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BrickAreaUniform {
    uv_min_px: [u32; 2], // Координаты в пикселях для textureStore
    _padding: [u32; 2],
}

pub fn sculpt_brick(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    compute_pipeline: &wgpu::ComputePipeline,
    atlas_bind_group: &wgpu::BindGroup,
    brick_id: u32,
    brush_pos: Vec3, // Позиция кисти относительно центра кирпича
    radius: f32,
) {
    let bricks_per_row = ATLAS_WIDTH / BRICK_RES;
    let x_px = (brick_id % bricks_per_row) * BRICK_RES;
    let y_px = (brick_id / bricks_per_row) * (BRICK_RES * BRICK_RES);

    // 1. Обновляем Uniform буферы (создаются заранее)
    let brush_data = BrushUniform {
        pos: brush_pos.to_array(),
        radius,
        op_type: 1.0,
        smoothness: 0.5,
        _padding: [0.0; 2],
    };

    let area_data = BrickAreaUniform {
        uv_min_px: [x_px, y_px],
        _padding: [0; 2],
    };

    // 2. Кодируем команды
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cpass.set_pipeline(compute_pipeline);
        cpass.set_bind_group(0, atlas_bind_group, &[]);

        // Передаем данные кисти (через отдельный буфер или push constants)
        // queue.write_buffer(...) вызывается перед encoder

        // Запускаем вычисления: 1 кирпич = 16x16x16 вокселей
        // Workgroup size в шейдере (8, 8, 1), значит нужно (2, 2, 16) групп
        cpass.dispatch_workgroups(2, 2, 16);
    }
    queue.submit(Some(encoder.finish()));
}
