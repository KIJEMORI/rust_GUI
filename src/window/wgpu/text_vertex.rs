use std::time::Instant;

fn push_glyph_vertices(
    vertices: &mut Vec<ShapeVertex>,
    pos: [f32; 4], // x1, y1, x2, y2
    uv: [f32; 4],  // u1, v1, u2, v2
    color: u32,    //[f32; 4],
) {
    let [x1, y1, x2, y2] = pos;
    let [u1, v1, u2, v2] = uv;

    // Порядок: Top-Left, Top-Right, Bottom-Left, Bottom-Right
    let positions = [[x1, y1], [x2, y1], [x1, y2], [x2, y2]];
    let uvs = [[u1, v1], [u2, v1], [u1, v2], [u2, v2]];

    for i in 0..4 {
        vertices.push(ShapeVertex {
            position: positions[i],
            p_a: uvs[i],
            p_b: [0.0, 0.0],
            color,
            params: [0.0, SHAPE_TEXT, 0.0, 0.0],
            border_color: 0,
        });
    }
}

pub struct GPUTextVertex {
    // Vertex Label Buffers
    // pub text_vertex_buffer: wgpu::Buffer,
    // pub text_index_buffer: wgpu::Buffer,
    // pub section_offsets: Vec<Range<usize>>,
    pub section_hashes: Vec<u64>,
    // pub section_capacities: Vec<usize>,
    // Pipeline Label
    // pub text_pipeline: wgpu::RenderPipeline,
    // pub bind_group: wgpu::BindGroup,
    pub text_bind_group_layout: wgpu::BindGroupLayout,
    pub text_bind_group: wgpu::BindGroup,
    // Inderected Args
    // pub indirect_buffer: wgpu::Buffer,
    // pub active_commands_count: u32,
    pub last_defrag_time: Instant,
    // pub next_free_vertex: usize,
    pub atlas: AtlasManager,
    pub temp_verts: Vec<ShapeVertex>,
    pub last_base_idx: u32,
}

use wgpu::Device;

use crate::window::{
    component::{
        base::gpu_render_context::{GpuRenderContext, TextData},
        managers::atlas_manager::AtlasManager,
    },
    wgpu::{
        shape_vertex::{SHAPE_TEXT, ShapeVertex},
        uber_resourse_manager::UberResourceManager,
    },
};

impl GPUTextVertex {
    pub fn new(device: &Device) -> Self {
        let font = include_bytes!("../component/base/Fonts/Roboto-Black.ttf");

        let atlas_manager = AtlasManager::new(&device, font, 2048);

        let glyph_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let text_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // Текстура атласа (@binding(0))
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // Самплер (@binding(1))
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("text_bind_group_layout"),
            });

        let text_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &text_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_manager.view), // Теперь работает!
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&glyph_sampler),
                },
            ],
            label: Some("Text Bind Group"),
        });

        // let draw_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Draw Vertex Buffer"),
        //     size: MAX_VERTICES * std::mem::size_of::<DrawIndirectArgs>() as u64,
        //     usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });

        // let mut indices = Vec::with_capacity(MAX_INDICES as usize);
        // for i in (0..(MAX_VERTICES as u32)).step_by(4) {
        //     indices.extend_from_slice(&[
        //         i + 0,
        //         i + 1,
        //         i + 2, // Первый треугольник
        //         i + 2,
        //         i + 1,
        //         i + 3, // Второй треугольник
        //     ]);
        // }

        // let text_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Text Static Index Buffer"),
        //     contents: bytemuck::cast_slice(&indices),
        //     usage: wgpu::BufferUsages::INDEX,
        // });

        Self {
            // text_vertex_buffer: text_vertex_buffer,
            atlas: atlas_manager,
            //section_offsets: Vec::with_capacity(1024),
            section_hashes: Vec::with_capacity(1024),
            //section_capacities: Vec::with_capacity(1024),
            // text_pipeline: text_pipeline,
            text_bind_group_layout: text_bind_group_layout,
            text_bind_group: text_bind_group,
            // bind_group: bind_group,
            // indirect_buffer: draw_buffer,
            // active_commands_count: 0,
            last_defrag_time: Instant::now(),
            // next_free_vertex: 0,
            // text_index_buffer: text_index_buffer,
            temp_verts: Vec::with_capacity(1024),
            last_base_idx: 0,
        }
    }

    // pub fn is_defrag_worth_it(&self) -> bool {
    //     let total_capacity: usize = self.section_capacities.iter().sum();
    //     let total_actual: usize = self.section_offsets.iter().map(|r| r.end - r.start).sum();
    //     // Чистим, если "пустоты" больше 30%
    //     total_capacity > (total_actual as f32 * 1.3) as usize && total_capacity > 50_000
    // }

    // pub fn perform_true_defragmentation(
    //     &mut self,
    //     device: &Device,
    //     queue: &Queue,
    //     manager: &mut UberResourceManager,
    //     shapes_end_vertex: usize, // Где кончаются прямоугольники в этом кадре
    // ) {
    //     let mut current_offset = shapes_end_vertex;
    //     let mut encoder =
    //         device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    //     for i in 0..self.section_offsets.len() {
    //         let len = self.section_offsets[i].end - self.section_offsets[i].start;
    //         if len > 0 {
    //             let src_offset =
    //                 (self.section_offsets[i].start as u64) * UberResourceManager::VERTEX_SIZE;
    //             let dst_offset = (current_offset as u64) * UberResourceManager::VERTEX_SIZE;

    //             encoder.copy_buffer_to_buffer(
    //                 &manager.vertex_buffer,
    //                 src_offset,
    //                 &manager.vertex_buffer,
    //                 dst_offset,
    //                 (len as u64) * UberResourceManager::VERTEX_SIZE,
    //             );

    //             self.section_offsets[i] = current_offset..(current_offset + len);
    //             current_offset += len;
    //         }
    //     }
    //     manager.next_free_vertex = current_offset;
    //     queue.submit(std::iter::once(encoder.finish()));
    // }

    pub fn render(
        &mut self,
        gpu_ctx: &GpuRenderContext,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        manager: &mut UberResourceManager,
        base_command_idx: u32, // Это shape_cmds из prepare_gpu_data
    ) {
        if gpu_ctx.texts.is_empty() {
            return;
        }
        {
            let total_verts = manager.next_free_vertex + (gpu_ctx.texts.len() * 4); // Примерно
            let total_cmds = base_command_idx + gpu_ctx.texts.len() as u32;

            // Если Vertex Buffer расширился, нужно форсировать перезапись ВСЕХ текстов
            // let was_resized = manager.ensure_vertex_capacity(device, queue, total_verts);
            manager.ensure_index_capacity(device, total_verts);
            manager.ensure_indirect_capacity(device, queue, total_cmds);
        }

        let mut temp_verts = std::mem::take(&mut self.temp_verts);
        temp_verts.clear();

        let mut text_commands = Vec::with_capacity(gpu_ctx.texts.len());

        // Текущий указатель на свободную вершину в глобальном буфере
        let start_vertex_all_text = manager.next_free_vertex;
        let mut current_v_cursor = start_vertex_all_text;

        self.last_base_idx = base_command_idx;

        for data in &gpu_ctx.texts {
            let content = &gpu_ctx.text_storage[data.range.clone()];

            // Запоминаем, сколько вершин БЫЛО в temp_verts до генерации этой строки
            let verts_before = temp_verts.len();

            // Генерируем вершины прямо в общий temp_verts
            self.generate_vertices_to(&mut temp_verts, content, data);

            // Считаем, сколько РЕАЛЬНО добавилось для ЭТОЙ строки
            let added_verts = temp_verts.len() - verts_before;

            text_commands.push(wgpu::util::DrawIndexedIndirectArgs {
                index_count: (added_verts as u32 / 4) * 6,
                instance_count: 1,
                // Смещение в индексном буфере:
                // Берем текущий глобальный курсор, делим на 4 (получаем номер квада) и умножаем на 6
                first_index: (current_v_cursor as u32 / 4) * 6,
                base_vertex: 0,
                first_instance: 0,
            });

            // Двигаем курсор ровно на столько, сколько добавили
            current_v_cursor += added_verts;
        }

        let vertex_write_offset =
            (start_vertex_all_text as u64) * std::mem::size_of::<ShapeVertex>() as u64;
        queue.write_buffer(
            &manager.vertex_buffer,
            vertex_write_offset,
            bytemuck::cast_slice(&temp_verts),
        );

        let indirect_write_offset = (base_command_idx as u64) * UberResourceManager::INDIRECT_SIZE;
        queue.write_buffer(
            &manager.indirect_buffer,
            indirect_write_offset,
            bytemuck::cast_slice(&text_commands),
        );

        manager.next_free_vertex = current_v_cursor;
        manager.active_shape_count = base_command_idx + text_commands.len() as u32;

        // Возвращаем буфер для переиспользования
        self.temp_verts = temp_verts;

        // Обновляем текстуру атласа, если добавились новые глифы
        self.atlas.update_atlas(queue);
    }

    pub fn generate_vertices_to(
        &mut self,
        temp_verts: &mut Vec<ShapeVertex>,
        content: &str,
        data: &TextData,
    ) {
        let scale_factor = data.size / 64.0;
        let metrics = self.atlas.font.horizontal_line_metrics(64.0).unwrap();
        // Рассчитываем базовую линию один раз на строку
        let baseline = (data.y + metrics.ascent * scale_factor).round();
        let mut x_cursor = data.x;

        for c in content.chars() {
            let glyph = self.atlas.get_glyph(c);

            let x1 = x_cursor + glyph.x_offset * scale_factor;
            let y1 = baseline - (glyph.y_offset + glyph.height) * scale_factor;
            let x2 = x1 + glyph.width * scale_factor;
            let y2 = y1 + glyph.height * scale_factor;

            // Используем твою существующую функцию
            push_glyph_vertices(
                temp_verts,
                [x1, y1, x2, y2],
                [
                    glyph.uv_min[0],
                    glyph.uv_min[1],
                    glyph.uv_max[0],
                    glyph.uv_max[1],
                ],
                data.color,
            );

            x_cursor += glyph.advance * scale_factor;
        }
    }
}
