use wgpu::{Device, util::DeviceExt};

use crate::window::wgpu::{
    draw_args::DrawIndexedIndirectArgs,
    shape_vertex::ShapeVertex,
    wgpu_state::{MAX_INDICES, MAX_VERTICES},
};

pub struct UberResourceManager {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer, // Тот самый статический буфер
    pub indirect_buffer: wgpu::Buffer,
    pub next_free_vertex: usize, // Глобальный указатель на текущий кадр
    pub active_shape_count: u32, // Сколько команд записали шейпы
}

impl UberResourceManager {
    pub const VERTEX_SIZE: u64 = std::mem::size_of::<ShapeVertex>() as u64;
    pub const INDIRECT_SIZE: u64 =
        std::mem::size_of::<wgpu::util::DrawIndexedIndirectArgs>() as u64;

    pub fn start_frame(&mut self) {
        self.next_free_vertex = 0;
        self.active_shape_count = 0;
    }

    pub fn new(device: &Device) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Panel Vertex Buffer"),
            size: MAX_VERTICES * std::mem::size_of::<ShapeVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let mut indices = Vec::with_capacity(MAX_INDICES as usize);
        for i in 0..(MAX_VERTICES / 4) {
            let b = (i * 4) as u32;
            indices.extend_from_slice(&[
                b + 0,
                b + 1,
                b + 2, // Первый треугольник
                b + 2,
                b + 3,
                b + 0, // Второй треугольник (порядок важен для CW/CCW)
            ]);
        }

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Static Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        let indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Draw Vertex Buffer"),
            size: 5000 * std::mem::size_of::<DrawIndexedIndirectArgs>() as u64,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        UberResourceManager {
            vertex_buffer,
            index_buffer,
            indirect_buffer,
            next_free_vertex: 0,
            active_shape_count: 0,
        }
    }

    pub fn ensure_vertex_capacity(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        required_verts: usize,
    ) -> bool {
        let required_size = (required_verts as u64) * Self::VERTEX_SIZE;
        let current_size = self.vertex_buffer.size();

        if required_size > current_size {
            let new_size = required_size.next_power_of_two();
            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Uber Vertex Buffer (Expanded)"),
                size: new_size,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_buffer_to_buffer(&self.vertex_buffer, 0, &new_buffer, 0, current_size);
            queue.submit(std::iter::once(encoder.finish()));

            self.vertex_buffer = new_buffer;
            return true; // Сообщаем, что был ресайз (нужно переподать данные текста)
        }
        false
    }

    /// Проверяет и расширяет Indirect Buffer (Команды)
    pub fn ensure_indirect_capacity(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        required_cmds: u32,
    ) {
        let required_size = (required_cmds as u64) * Self::INDIRECT_SIZE;
        if required_size > self.indirect_buffer.size() {
            let new_size = required_size.next_power_of_two();
            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Uber Indirect Buffer (Expanded)"),
                size: new_size,
                usage: wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_buffer_to_buffer(
                &self.indirect_buffer,
                0,
                &new_buffer,
                0,
                self.indirect_buffer.size(),
            );
            queue.submit(std::iter::once(encoder.finish()));
            self.indirect_buffer = new_buffer;
        }
    }

    /// Проверяет и расширяет Index Buffer (ВАЖНО для больших объемов данных)
    pub fn ensure_index_capacity(&mut self, device: &wgpu::Device, required_verts: usize) {
        let required_indices = (required_verts / 4 * 6) as u64;
        let current_indices_count = self.index_buffer.size() / 4;

        if required_indices > current_indices_count {
            let new_count = required_indices.next_power_of_two();
            // Генерируем новые индексы на CPU
            let mut indices = Vec::with_capacity(new_count as usize);
            for i in 0..(new_count / 6) {
                let b = (i * 4) as u32;
                indices.extend_from_slice(&[b, b + 1, b + 2, b + 2, b + 1, b + 3]);
            }

            self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uber Index Buffer (Expanded)"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        }
    }
}
