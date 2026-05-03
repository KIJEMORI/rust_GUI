use wgpu::{Device, VertexState, util::DeviceExt};

use crate::window::wgpu::{
    block_3d::{camera_uniform::CameraUniform, instance::Instance3DData},
    shape_vertex::ShapeVertex,
};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BakePushConstants {
    pub brick_id: u32,
    pub start_instance: u32, // С какого индекса в instances начинаются команды
    pub count: u32,          // Сколько команд объединить
    pub padding: u32,
}
pub struct GPUBrickRender {
    pub buffer_for_commands: wgpu::Buffer,
    pub buffer_for_brick: wgpu::Buffer,
    pub atlas_texture: wgpu::Texture,
    pub atlas_view: wgpu::TextureView,
    pub atlas_sampler: wgpu::Sampler,
    pub bind_group_layout_for_baking: wgpu::BindGroupLayout,
    pub bind_group_for_baking: wgpu::BindGroup,
    pub bind_group_layout_for_render: wgpu::BindGroupLayout,
    pub bind_group_for_render: wgpu::BindGroup,
    pub baking_pipeline: wgpu::ComputePipeline,
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera_buffer: wgpu::Buffer,
}

impl GPUBrickRender {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        global_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // Шейдеры
        let shader_for_baking =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/high/baking_3d.wgsl"));
        let shader_for_render =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/high/render_3d.wgsl"));

        // Создание Атласа
        let atlas_size = 256; // 32 кирпича по 8 вокселей = 256

        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("3D SDF Volume"),
            size: wgpu::Extent3d {
                width: atlas_size,
                height: atlas_size,
                depth_or_array_layers: atlas_size, // ТЕПЕРЬ ЭТО ГЛУБИНА
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D3), // Вид тоже D3
            ..Default::default()
        });

        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let total_elements = (atlas_size * atlas_size * atlas_size * 4) as usize;

        // 0x4900 — это битовое представление 10.0 в формате f16
        let empty_val: u16 = 0x4900;

        let mut clear_data = vec![0u16; total_elements];
        for i in (0..total_elements).step_by(4) {
            clear_data[i] = empty_val; // Записываем f16 пустоты в канал R
        }

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&clear_data),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(atlas_size * 8),
                rows_per_image: Some(atlas_size),
            },
            wgpu::Extent3d {
                width: atlas_size,
                height: atlas_size,
                depth_or_array_layers: atlas_size,
            },
        );

        // LAYOUT ДЛЯ BAKING (COMPUTE)
        let bind_group_layout_for_baking =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("3D Baking Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba16Float,
                            view_dimension: wgpu::TextureViewDimension::D3,
                        },
                        count: None,
                    },
                ],
            });

        // COMPUTE PIPELINE (Для выпекания)
        let baking_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("3D Baking Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout_for_baking],
                push_constant_ranges: &[],
            });
        let baking_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("3D SDF Baking Pipeline"),
            layout: Some(&baking_pipeline_layout),
            module: &shader_for_baking,
            entry_point: Some("cs_main"),
            compilation_options: Default::default(),
            cache: None,
        });

        // БУФЕРЫ
        let buffer_for_brick = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("3D Brick Task Buffer"),
            size: (std::mem::size_of::<BakePushConstants>() * 1000) as u64, // 1000 задач за раз достаточно
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let buffer_for_commands = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("3D Instance Commands Buffer"),
            size: (std::mem::size_of::<Instance3DData>() * 15000) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_for_baking = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("3D Baking Bind Group"),
            layout: &bind_group_layout_for_baking,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer_for_brick.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffer_for_commands.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
            ],
        });

        let bind_group_layout_for_render =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("3D Render Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D3,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Здесь используем Fullscreen Quad (без буферов вершин)
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("3D Render Pipeline Layout"),
                bind_group_layouts: &[global_bind_group_layout, &bind_group_layout_for_render],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader_for_render,
                entry_point: Some("vs_main"),
                buffers: &[ShapeVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_for_render,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
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

        let camera_uniform = CameraUniform::default();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_for_render = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("3D Scene Bind Group"),
            layout: &bind_group_layout_for_render,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(), // Твой буфер камеры
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
            ],
        });

        Self {
            buffer_for_commands,
            buffer_for_brick,
            atlas_texture,
            atlas_view,
            atlas_sampler,
            bind_group_layout_for_baking,
            bind_group_for_baking,
            bind_group_layout_for_render,
            bind_group_for_render,
            baking_pipeline,
            render_pipeline,
            camera_buffer,
        }
    }

    pub fn recreate_bind_group_for_baking(&mut self, device: &Device) {
        let bind_group_for_baking = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("3D Baking Bind Group"),
            layout: &self.bind_group_layout_for_baking,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.buffer_for_brick.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.buffer_for_commands.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.atlas_view),
                },
            ],
        });

        self.bind_group_for_baking = bind_group_for_baking
    }

    pub fn recreate_bind_group_for_render(&mut self, device: &Device) {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("3D Scene Bind Group"),
            layout: &self.bind_group_layout_for_render,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.atlas_sampler),
                },
            ],
        });

        self.bind_group_for_render = bind_group
    }
}
