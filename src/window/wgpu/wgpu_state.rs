use std::{ops::Range, time::Instant};

use wgpu::{Device, Queue, Surface as WgpuSurface, SurfaceConfiguration, util::DeviceExt};
use wgpu_glyph::ab_glyph;

use crate::window::wgpu::{
    draw_args::DrawIndirectArgs, screen_uniform::ScreenUniform, text_vertex::TextVertex,
    vertex::Vertex,
};
pub struct WgpuState {
    // База
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    // Vertex Panel
    pub vertex_buffer: wgpu::Buffer,
    pub text_vertex_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub panel_pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    // Vetex Label
    pub glyph_brush: glyph_brush::GlyphBrush<TextVertex, TextVertex, ab_glyph::FontArc>,
    // Vertex Label Buffers
    pub section_offsets: Vec<Range<usize>>,
    pub section_hashes: Vec<u64>,
    pub section_capacities: Vec<usize>,
    // Pipeline Label
    pub glyph_texture: wgpu::Texture,
    pub text_pipeline: wgpu::RenderPipeline,
    pub text_bind_group: wgpu::BindGroup,
    // Inderected Args
    pub indirect_buffer: wgpu::Buffer,
    pub active_commands_count: u32,
    pub wasted_vertices: usize,
    pub last_defrag_time: Instant,
    next_free_vertex: usize,
}

const MAX_VERTICES: u64 = 36_000;

impl WgpuState {
    pub fn new(
        surface: WgpuSurface<'static>,
        device: Device,
        queue: Queue,
        config: SurfaceConfiguration,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shader1.wgsl"));

        let screen_uniform = ScreenUniform {
            size: [800.0, 600.0],
            _padding: [0.0, 0.0],
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[screen_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // BindGroup — это "вход" для буфера в шейдер
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout], // Сюда передаем наш layout
            push_constant_ranges: &[],
        });

        let panel_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(), // ДОБАВИТЬ
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(), // Рисуем треугольники
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Panel Vertex Buffer"),
            size: MAX_VERTICES * std::mem::size_of::<Vertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, // ОБЯЗАТЕЛЬНО VERTEX
            mapped_at_creation: false,
        });

        // Text Pipeline

        let text_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/label_shader.wgsl"));

        let glyph_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glyph Atlas"),
            size: wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let glyph_texture_view = glyph_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let font = ab_glyph::FontArc::try_from_slice(include_bytes!(
            "../component/base/Fonts/calibri.ttf"
        ))
        .unwrap();
        let glyph_brush =
            glyph_brush::GlyphBrushBuilder::using_font(font).build::<TextVertex, TextVertex>();
        let glyph_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let text_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Vertex Buffer"),
            size: MAX_VERTICES * std::mem::size_of::<TextVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
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
                    resource: wgpu::BindingResource::TextureView(&glyph_texture_view), // Теперь работает!
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&glyph_sampler),
                },
            ],
            label: Some("Text Bind Group"),
        });

        let text_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, &text_bind_group_layout],
            push_constant_ranges: &[],
        });

        let text_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Pipeline"),
            layout: Some(&text_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &text_shader,
                entry_point: Some("vs_main"),
                buffers: &[TextVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &text_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha, // Берем альфу текста
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha, // Оставляем фон за ним
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Max,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let draw_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Draw Vertex Buffer"),
            size: MAX_VERTICES * std::mem::size_of::<DrawIndirectArgs>() as u64,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            surface: surface,
            device: device,
            queue: queue,
            config: config,
            vertex_buffer: vertex_buffer,
            text_vertex_buffer: text_vertex_buffer,
            uniform_buffer: uniform_buffer,
            panel_pipeline: panel_pipeline,
            bind_group: bind_group,
            glyph_brush: glyph_brush,
            section_offsets: Vec::new(),
            section_hashes: Vec::new(),
            section_capacities: Vec::new(),
            glyph_texture: glyph_texture,
            text_pipeline: text_pipeline,
            text_bind_group: text_bind_group,
            indirect_buffer: draw_buffer,
            active_commands_count: 0,
            wasted_vertices: 0,
            last_defrag_time: Instant::now(),
            next_free_vertex: 0,
        }
    }

    // pub fn update_section_in_arena(&mut self, s_idx: usize, new_verts: Vec<TextVertex>) {
    //     let old_range = self.section_offsets.get(s_idx).cloned().unwrap_or(0..0);
    //     let new_len = new_verts.len();
    //     let old_len = old_range.end - old_range.start;

    //     // splice сам раздвинет или сузит вектор, если длины не совпадают
    //     self.last_text_vertices
    //         .splice(old_range.clone(), new_verts.into_iter());

    //     // Если длина изменилась, все последующие офсеты "поплыли" — корректируем их
    //     if new_len != old_len {
    //         let diff = new_len as i32 - old_len as i32;

    //         // Обновляем текущий офсет
    //         self.section_offsets[s_idx] = old_range.start..(old_range.start + new_len);

    //         // Сдвигаем все офсеты, которые идут ПОСЛЕ этой строки
    //         for i in (s_idx + 1)..self.section_offsets.len() {
    //             let r = &self.section_offsets[i];
    //             self.section_offsets[i] =
    //                 (r.start as i32 + diff) as usize..(r.end as i32 + diff) as usize;
    //         }

    //         // ТАК КАК ВЕСЬ БУФЕР СДВИНУЛСЯ — ПЕРЕЗАПИСЫВАЕМ ЕГО В GPU ПОЛНОСТЬЮ
    //         self.write_text_vertices();
    //     } else {
    //         let offset_bytes = (old_range.start * std::mem::size_of::<TextVertex>()) as u64;
    //         self.queue.write_buffer(
    //             &self.text_vertex_buffer,
    //             offset_bytes,
    //             bytemuck::cast_slice(
    //                 &self.last_text_vertices[old_range.start..old_range.start + new_len],
    //             ),
    //         );
    //     }
    // }

    pub fn ensure_gpu_capacity(&mut self, required_count: usize) {
        let required_size = (required_count * std::mem::size_of::<TextVertex>()) as u64;
        let current_capacity = self.text_vertex_buffer.size();

        if required_size > current_capacity {
            let new_size = required_size.next_power_of_two();

            let new_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Dynamic Text Vertex Buffer (Expanded)"),
                size: new_size,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_buffer_to_buffer(
                &self.text_vertex_buffer,
                0,
                &new_buffer,
                0,
                current_capacity,
            );
            self.queue.submit(std::iter::once(encoder.finish()));

            self.text_vertex_buffer.destroy();
            self.text_vertex_buffer = new_buffer;

            println!("GPU Buffer GROW: {} bytes (RAM stays clean)", new_size);
        }
    }

    pub fn update_section_direct_gpu(&mut self, s_idx: usize, new_verts: Vec<TextVertex>) {
        let new_len = new_verts.len();

        // Если секция новая (cap == 0) или текст не влез
        if self.section_capacities[s_idx] == 0 || new_len > self.section_capacities[s_idx] {
            let new_padded_cap = (new_len as f32 * 1.5) as usize + 6; // +6 вершин минимум (1 символ)

            self.ensure_gpu_capacity(self.next_free_vertex + new_padded_cap);

            let offset = (self.next_free_vertex * std::mem::size_of::<TextVertex>()) as u64;

            if !new_verts.is_empty() {
                self.queue.write_buffer(
                    &self.text_vertex_buffer,
                    offset,
                    bytemuck::cast_slice(&new_verts),
                );
            }

            self.section_offsets[s_idx] = self.next_free_vertex..(self.next_free_vertex + new_len);
            self.section_capacities[s_idx] = new_padded_cap;
            self.next_free_vertex += new_padded_cap;
        } else {
            // Влезло в старый слот
            let start_idx = self.section_offsets[s_idx].start;
            let offset = (start_idx * std::mem::size_of::<TextVertex>()) as u64;

            if !new_verts.is_empty() {
                self.queue.write_buffer(
                    &self.text_vertex_buffer,
                    offset,
                    bytemuck::cast_slice(&new_verts),
                );
            }

            self.section_offsets[s_idx] = start_idx..(start_idx + new_len);
        }
    }

    pub fn defragment_if_needed(&mut self) -> bool {
        let total_capacity: usize = self.section_capacities.iter().sum();
        let total_actual: usize = self.section_offsets.iter().map(|r| r.end - r.start).sum();
        //println!("Проверка дефрагментации GPU буфера...");
        if total_capacity > total_actual * 2 && total_capacity > 100_000 {
            println!("Дефрагментация GPU буфера...");

            self.next_free_vertex = 0;
            self.section_offsets.fill(0..0);
            self.section_capacities.fill(0);
            self.section_hashes.fill(0);

            self.wasted_vertices = 0;
            self.update_indirect_buffer(); // Команды отрисовки теперь указывают на новые места
            return true;
        }
        false
    }

    pub fn update_indirect_buffer(&mut self) {
        let commands: Vec<DrawIndirectArgs> = self
            .section_offsets
            .iter()
            .filter_map(|range| {
                let count = (range.end - range.start) as u32;
                if count > 0 {
                    Some(DrawIndirectArgs {
                        vertex_count: count,
                        instance_count: 1,
                        first_vertex: range.start as u32,
                        first_instance: 0,
                    })
                } else {
                    None
                }
            })
            .collect();

        if commands.is_empty() {
            self.active_commands_count = 0;
            return;
        }

        let size = (commands.len() * std::mem::size_of::<DrawIndirectArgs>()) as u64;

        // Если старый буфер мал — пересоздаем, если ок — просто пишем в него
        if self.indirect_buffer.size() < size {
            self.indirect_buffer.destroy();
            self.indirect_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Text Indirect Buffer"),
                        contents: bytemuck::cast_slice(&commands),
                        usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
                    });
        } else {
            self.queue
                .write_buffer(&self.indirect_buffer, 0, bytemuck::cast_slice(&commands));
        }

        self.active_commands_count = commands.len() as u32;
    }
}
