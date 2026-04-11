use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    ops::Range,
    time::Instant,
};

use wgpu_glyph::{Text, ab_glyph};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub scroll_id: u32,
    pub section_id: u32,
    pub version: u32,
}

impl TextVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
            0 => Float32x2, // position
            1 => Float32x2, // uv
            2 => Float32x4, // color
            3 => Uint32,    // scroll_id
            4 => Uint32,    // section_id
            5 => Uint32     // version
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

impl PartialEq for TextVertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.uv == other.uv
            && self.color == other.color
            && self.version == other.version
            && self.scroll_id == other.scroll_id
    }
}

impl Eq for TextVertex {}

impl Hash for TextVertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let fields = [
            &self.position[..],
            &self.uv[..],
            &self.color[..],
            &[self.scroll_id as f32],
        ];
        for field in fields {
            for &val in field {
                val.to_bits().hash(state);
            }
        }
    }
}

pub fn push_glyph_to_vertices_raw(
    px: glyph_brush::ab_glyph::Rect,
    tex: glyph_brush::ab_glyph::Rect,
    color: [f32; 4],
    section_id: u32,
    scroll_id: u32,
) -> [TextVertex; 6] {
    let v_tl = TextVertex {
        position: [px.min.x, px.min.y],
        uv: [tex.min.x, tex.min.y],
        color,
        version: 0,
        section_id,
        scroll_id,
    };
    let v_tr = TextVertex {
        position: [px.max.x, px.min.y],
        uv: [tex.max.x, tex.min.y],
        color,
        version: 0,
        section_id,
        scroll_id,
    };
    let v_bl = TextVertex {
        position: [px.min.x, px.max.y],
        uv: [tex.min.x, tex.max.y],
        color,
        version: 0,
        section_id,
        scroll_id,
    };
    let v_br = TextVertex {
        position: [px.max.x, px.max.y],
        uv: [tex.max.x, tex.max.y],
        color,
        version: 0,
        section_id,
        scroll_id,
    };

    [v_tl, v_tr, v_bl, v_tr, v_br, v_bl]
}

pub struct GPUTextVertex {
    pub glyph_brush: glyph_brush::GlyphBrush<TextVertex, TextVertex, ab_glyph::FontArc>,
    // Vertex Label Buffers
    pub text_vertex_buffer: wgpu::Buffer,
    pub section_offsets: Vec<Range<usize>>,
    pub section_hashes: Vec<u64>,
    pub section_capacities: Vec<usize>,
    // Pipeline Label
    pub glyph_texture: wgpu::Texture,
    pub text_pipeline: wgpu::RenderPipeline,
    pub text_bind_group: wgpu::BindGroup,
    pub bind_group: wgpu::BindGroup,
    // Inderected Args
    pub indirect_buffer: wgpu::Buffer,
    pub active_commands_count: u32,
    pub last_defrag_time: Instant,
    pub next_free_vertex: usize,
}

use wgpu::{Buffer, Device, Queue, RenderPass, SurfaceConfiguration, util::DeviceExt};

use crate::window::{
    component::base::gpu_render_context::GpuRenderContext,
    wgpu::{draw_args::DrawIndirectArgs, wgpu_state::MAX_VERTICES},
};

impl GPUTextVertex {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        uniform_buffer: &Buffer,
        scroll_storage_buffer: &Buffer,
    ) -> Self {
        let text_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/high/label_shader.wgsl"));

        let glyph_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glyph Atlas"),
            size: wgpu::Extent3d {
                width: 2048,
                height: 2048,
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
            "../component/base/Fonts/Roboto-Black.ttf"
        ))
        .unwrap();
        let glyph_brush = glyph_brush::GlyphBrushBuilder::using_font(font)
            .initial_cache_size((2048, 2048))
            .build::<TextVertex, TextVertex>();
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

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX, // Оффсеты нужны только в Vertex шейдере
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: scroll_storage_buffer.as_entire_binding(), // Тот самый буфер оффсетов
                },
            ],
            label: None,
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        pass_op: wgpu::StencilOperation::Keep,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    back: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        pass_op: wgpu::StencilOperation::Keep,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    read_mask: 0xff,
                    write_mask: 0x00,
                },
                bias: wgpu::DepthBiasState::default(),
            }),
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
            text_vertex_buffer: text_vertex_buffer,
            glyph_brush: glyph_brush,
            section_offsets: Vec::new(),
            section_hashes: Vec::new(),
            section_capacities: Vec::new(),
            glyph_texture: glyph_texture,
            text_pipeline: text_pipeline,
            text_bind_group: text_bind_group,
            bind_group: bind_group,
            indirect_buffer: draw_buffer,
            active_commands_count: 0,
            last_defrag_time: Instant::now(),
            next_free_vertex: 0,
        }
    }

    pub fn ensure_gpu_capacity(&mut self, required_count: usize, device: &Device, queue: &Queue) {
        let required_size = (required_count * std::mem::size_of::<TextVertex>()) as u64;
        let current_capacity = self.text_vertex_buffer.size();

        if required_size > current_capacity {
            let new_size = required_size.next_power_of_two();

            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Dynamic Text Vertex Buffer (Expanded)"),
                size: new_size,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_buffer_to_buffer(
                &self.text_vertex_buffer,
                0,
                &new_buffer,
                0,
                current_capacity,
            );
            queue.submit(std::iter::once(encoder.finish()));

            self.text_vertex_buffer.destroy();
            self.text_vertex_buffer = new_buffer;

            println!("GPU Buffer GROW: {} bytes (RAM stays clean)", new_size);
        }
    }

    pub fn update_section_direct_gpu(
        &mut self,
        s_idx: usize,
        new_verts: Vec<TextVertex>,
        device: &Device,
        queue: &Queue,
    ) {
        let new_len = new_verts.len();
        if new_len == 0 {
            let start = self.section_offsets[s_idx].start;
            self.section_offsets[s_idx] = start..start;
            return;
        }

        // Если секция новая (cap == 0) или текст не влез
        if self.section_capacities[s_idx] == 0 || new_len > self.section_capacities[s_idx] {
            let new_padded_cap = (new_len as f32 * 1.5) as usize + 6; // +6 вершин минимум (1 символ)

            self.ensure_gpu_capacity(self.next_free_vertex + new_padded_cap, device, queue);

            let offset = (self.next_free_vertex * std::mem::size_of::<TextVertex>()) as u64;

            if !new_verts.is_empty() {
                queue.write_buffer(
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
                queue.write_buffer(
                    &self.text_vertex_buffer,
                    offset,
                    bytemuck::cast_slice(&new_verts),
                );
            }

            self.section_offsets[s_idx] = start_idx..(start_idx + new_len);
        }
        // let offset = (self.next_free_vertex * std::mem::size_of::<TextVertex>()) as u64;

        // self.queue.write_buffer(
        //     &self.text_vertex_buffer,
        //     offset,
        //     bytemuck::cast_slice(&new_verts),
        // );

        // self.section_offsets[s_idx] = self.next_free_vertex..(self.next_free_vertex + new_len);
        // self.next_free_vertex += new_len;
    }

    // pub fn defragment_if_needed(&mut self) -> bool {
    //     let total_capacity: usize = self.section_capacities.iter().sum();
    //     let total_actual: usize = self.section_offsets.iter().map(|r| r.end - r.start).sum();
    //     //println!("Проверка дефрагментации GPU буфера...");
    //     if total_capacity > total_actual * 2 && total_capacity > 100_000 {
    //         println!("Дефрагментация GPU буфера...");

    //         self.next_free_vertex = 0;
    //         self.section_offsets.fill(0..0);
    //         self.section_capacities.fill(0);
    //         self.section_hashes.fill(0);

    //         self.update_indirect_buffer(); // Команды отрисовки теперь указывают на новые места
    //         return true;
    //     }
    //     false
    // }

    pub fn update_indirect_buffer(&mut self, device: &Device, queue: &Queue) {
        let commands: Vec<DrawIndirectArgs> = self
            .section_offsets
            .iter()
            .map(|range| {
                DrawIndirectArgs {
                    vertex_count: (range.end - range.start) as u32, // Если скрыт, тут будет 0..0 = 0
                    instance_count: 1,
                    first_vertex: range.start as u32,
                    first_instance: 0,
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
            self.indirect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Text Indirect Buffer"),
                contents: bytemuck::cast_slice(&commands),
                usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            });
        } else {
            queue.write_buffer(&self.indirect_buffer, 0, bytemuck::cast_slice(&commands));
        }

        self.active_commands_count = commands.len() as u32;
    }

    pub fn is_defrag_worth_it(&self) -> bool {
        let total_capacity: usize = self.section_capacities.iter().sum();
        let total_actual: usize = self.section_offsets.iter().map(|r| r.end - r.start).sum();
        // Чистим, если "пустоты" больше 30%
        total_capacity > (total_actual as f32 * 1.3) as usize && total_capacity > 50_000
    }

    pub fn perform_true_defragmentation(&mut self, device: &Device, queue: &Queue) {
        let mut current_offset = 0;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Defrag Encoder"),
        });

        for i in 0..self.section_offsets.len() {
            let range = &self.section_offsets[i];
            let len = range.end - range.start;

            if len > 0 {
                let src_offset = (range.start * std::mem::size_of::<TextVertex>()) as u64;
                let dst_offset = (current_offset * std::mem::size_of::<TextVertex>()) as u64;

                // Если сектор уже не на своем идеальном месте (есть дырка перед ним)
                if src_offset != dst_offset {
                    encoder.copy_buffer_to_buffer(
                        &self.text_vertex_buffer,
                        src_offset,
                        &self.text_vertex_buffer,
                        dst_offset,
                        (len * std::mem::size_of::<TextVertex>()) as u64,
                    );
                }

                // Обновляем метаданные: теперь этот текст живет по новому адресу
                self.section_offsets[i] = current_offset..(current_offset + len);
                self.section_capacities[i] = len; // Ужимаем до реального размера
                current_offset += len;
            }
        }

        self.next_free_vertex = current_offset;
        queue.submit(std::iter::once(encoder.finish()));

        // ВАЖНО: Хеши НЕ сбрасываем! GlyphBrush даже не заметит, что мы что-то двигали.
        self.update_indirect_buffer(device, queue);
    }

    pub fn render(
        &mut self,
        gpu_ctx: &GpuRenderContext,
        last_render: Instant,
        device: &Device,
        queue: &Queue,
    ) {
        let texts_count = gpu_ctx.texts.len();
        let mut force_update = false;

        if self.section_offsets.len() != texts_count {
            self.section_offsets.resize(texts_count, 0..0);
            //state.section_hashes.clear(); // Полностью сбрасываем хеши
            self.section_hashes.resize(texts_count, 0);
            self.section_capacities.resize(texts_count, 0);
            //force_update = true;
        }
        for (idx, data) in gpu_ctx.texts.iter().enumerate() {
            let content = &gpu_ctx.text_storage[data.range.clone()];

            let mut item_needs_update = force_update;

            if self.section_offsets[idx].start == self.section_offsets[idx].end
                && !content.is_empty()
            {
                // Если офсет пустой, но текст есть — значит, элемент только что "проснулся"
                self.section_hashes[idx] = 0; // Сбрасываем хеш, чтобы форсировать Draw
                item_needs_update = true;
            }
            if content.is_empty() {
                if self.section_offsets[idx].end != self.section_offsets[idx].start {
                    self.update_section_direct_gpu(idx, Vec::new(), device, queue);
                    self.section_hashes[idx] = 0;

                    // state.update_indirect_buffer();
                }
                continue; // Пропускаем glyph_brush для пустой строки
            }

            let mut hasher = DefaultHasher::new();
            content.hash(&mut hasher); // Текст
            data.x.to_bits().hash(&mut hasher); // Позиция X
            data.y.to_bits().hash(&mut hasher); // Позиция Y
            data.size.to_bits().hash(&mut hasher); // Масштаб

            let current_hash = hasher.finish();

            if self.section_hashes[idx] != current_hash {
                self.section_hashes[idx] = current_hash; // Запоминаем новый хеш
                item_needs_update = true;
            }

            let extra = TextVertex {
                position: [0.0, 0.0],
                uv: [0.0, 0.0],
                version: if item_needs_update {
                    Instant::now().duration_since(last_render).as_micros() as u32
                } else {
                    0
                },
                color: data.color,
                section_id: idx as u32,
                scroll_id: data.scroll_id,
            };
            let text_fragment = Text::<TextVertex>::new(content)
                .with_scale(data.size)
                .with_extra(extra);

            // Если скролл двигается или ресайзится - заставляем браш выдать новые вершины
            let x_offset = if item_needs_update {
                0.0002 * (idx as f32 + 1.0) // Микро-сдвиг заставляет brush пересчитать геометрию
            } else {
                0.0
            };

            let section = glyph_brush::Section {
                screen_position: (data.x + x_offset, data.y),
                bounds: (f32::INFINITY, f32::INFINITY),
                layout: glyph_brush::Layout::default(),
                text: vec![text_fragment],
            };

            self.glyph_brush.queue(section);
        }

        // АРЕНА: временное хранилище для пересчитанных строк (только грязных)
        let mut dirty_section = Vec::new();

        let ref_dirty_sections = RefCell::new(&mut dirty_section);

        let action = self
            .glyph_brush
            .process_queued(
                |rect, data| {
                    queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &self.glyph_texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: rect.min[0],
                                y: rect.min[1],
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        data,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(rect.width()),
                            rows_per_image: Some(rect.height()),
                        },
                        wgpu::Extent3d {
                            width: rect.width(),
                            height: rect.height(),
                            depth_or_array_layers: 1,
                        },
                    );
                },
                |glyph| {
                    let s_idx = glyph.extra.section_id as usize;

                    // Генерируем 6 вершин (треугольники)
                    let vertices = push_glyph_to_vertices_raw(
                        glyph.pixel_coords,
                        glyph.tex_coords,
                        glyph.extra.color,
                        glyph.extra.section_id,
                        glyph.extra.scroll_id,
                    );
                    let extra = glyph.extra.clone();
                    ref_dirty_sections
                        .borrow_mut()
                        .push((s_idx, Vec::from(vertices)));

                    extra
                },
            )
            .expect("Ошибка обработки очереди текста");

        match action {
            glyph_brush::BrushAction::Draw(_) => {
                let mut section_map: HashMap<usize, Vec<TextVertex>> = HashMap::new();
                for (id, verts) in dirty_section {
                    section_map.entry(id).or_default().extend_from_slice(&verts);
                }

                for (s_idx, new_verts) in section_map {
                    self.update_section_direct_gpu(s_idx, new_verts, device, queue);
                }

                self.update_indirect_buffer(device, queue);
            }
            glyph_brush::BrushAction::ReDraw => {
                self.update_indirect_buffer(device, queue);
            }
        }
    }
}
