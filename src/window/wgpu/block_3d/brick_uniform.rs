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

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 720;

pub struct GPUBrickRender {
    pub buffer_for_commands: wgpu::Buffer,
    pub buffer_for_brick: wgpu::Buffer,

    pub atlas_view_sdf: wgpu::TextureView,
    pub atlas_view_color: wgpu::TextureView,
    //pub atlas_view_color_after_light: wgpu::TextureView,
    pub bind_group_layout_for_baking: wgpu::BindGroupLayout,
    pub bind_group_for_baking: wgpu::BindGroup,

    // pub bind_group_layout_for_lighting: wgpu::BindGroupLayout,
    // pub bind_group_for_lighting: wgpu::BindGroup,
    pub bind_group_layout_for_render: wgpu::BindGroupLayout,
    pub bind_group_for_render: wgpu::BindGroup,

    pub baking_pipeline: wgpu::ComputePipeline,
    //pub lighting_pipeline: wgpu::ComputePipeline,
    pub render_pipeline: wgpu::RenderPipeline,

    pub camera_buffer: wgpu::Buffer,
    pub atlas_size: u32,
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
        let atlas_size = 4096;

        let atlas_texture_sdf = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("2D SDF Volume Atlas"),
            size: wgpu::Extent3d {
                width: atlas_size,
                height: atlas_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let atlas_view_sdf = atlas_texture_sdf.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        });

        let atlas_texture_color = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("2D Color Volume Atlas"),
            size: wgpu::Extent3d {
                width: atlas_size,
                height: atlas_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let atlas_view_color = atlas_texture_color.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..Default::default()
        });

        // let atlas_texture_color_after_light = device.create_texture(&wgpu::TextureDescriptor {
        //     label: Some("2D Color Volume Atlas"),
        //     size: wgpu::Extent3d {
        //         width: atlas_size,
        //         height: atlas_size,
        //         depth_or_array_layers: 1,
        //     },
        //     mip_level_count: 1,
        //     sample_count: 1,
        //     dimension: wgpu::TextureDimension::D2,
        //     format: wgpu::TextureFormat::Rgba8Unorm,
        //     usage: wgpu::TextureUsages::STORAGE_BINDING
        //         | wgpu::TextureUsages::TEXTURE_BINDING
        //         | wgpu::TextureUsages::COPY_DST,
        //     view_formats: &[],
        // });

        // let atlas_view_color_after_light =
        //     atlas_texture_color_after_light.create_view(&wgpu::TextureViewDescriptor {
        //         dimension: Some(wgpu::TextureViewDimension::D2),
        //         ..Default::default()
        //     });

        let sdf_elements = (atlas_size * atlas_size) as usize;

        let clear_data_sdf = vec![10.0f32; sdf_elements]; // Пишем прямо f32!

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &atlas_texture_sdf,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&clear_data_sdf),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(atlas_size * 4),
                rows_per_image: Some(atlas_size),
            },
            wgpu::Extent3d {
                width: atlas_size,
                height: atlas_size,
                depth_or_array_layers: 1,
            },
        );

        let color_bytes = (atlas_size * atlas_size * 4) as usize;
        let clear_data_color = vec![0u8; color_bytes]; // Заполняем прозрачным черным

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &atlas_texture_color,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &clear_data_color,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                // 4096 пикселей * 4 байта = 16 384 байта (кратно 256, отлично)
                bytes_per_row: Some(atlas_size * 4),
                rows_per_image: Some(atlas_size),
            },
            wgpu::Extent3d {
                width: atlas_size,
                height: atlas_size,
                depth_or_array_layers: 1,
            },
        );

        // queue.write_texture(
        //     wgpu::TexelCopyTextureInfo {
        //         texture: &atlas_texture_color_after_light,
        //         mip_level: 0,
        //         origin: wgpu::Origin3d::ZERO,
        //         aspect: wgpu::TextureAspect::All,
        //     },
        //     &clear_data_color,
        //     wgpu::TexelCopyBufferLayout {
        //         offset: 0,
        //         // 4096 пикселей * 4 байта = 16 384 байта (кратно 256, отлично)
        //         bytes_per_row: Some(atlas_size * 4),
        //         rows_per_image: Some(atlas_size),
        //     },
        //     wgpu::Extent3d {
        //         width: atlas_size,
        //         height: atlas_size,
        //         depth_or_array_layers: 1,
        //     },
        // );

        // COMPUTE PIPELINE (Для выпекания)
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
                            format: wgpu::TextureFormat::R32Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

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
                    resource: wgpu::BindingResource::TextureView(&atlas_view_sdf),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&atlas_view_color),
                },
            ],
        });

        // Lighting Shader

        // let shader_for_lighting =
        //     device.create_shader_module(wgpu::include_wgsl!("../shaders/high/light_3d.wgsl"));

        // let bind_group_layout_for_lighting =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         label: Some("3D Lighting Bind Group Layout"),
        //         entries: &[
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 0,
        //                 visibility: wgpu::ShaderStages::COMPUTE,
        //                 ty: wgpu::BindingType::Buffer {
        //                     ty: wgpu::BufferBindingType::Storage { read_only: true },
        //                     has_dynamic_offset: false,
        //                     min_binding_size: None,
        //                 },
        //                 count: None,
        //             },
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 1,
        //                 visibility: wgpu::ShaderStages::COMPUTE,
        //                 ty: wgpu::BindingType::Buffer {
        //                     ty: wgpu::BufferBindingType::Storage { read_only: true },
        //                     has_dynamic_offset: false,
        //                     min_binding_size: None,
        //                 },
        //                 count: None,
        //             },
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 2,
        //                 visibility: wgpu::ShaderStages::COMPUTE,
        //                 ty: wgpu::BindingType::Texture {
        //                     // Меняем на обычную Texture
        //                     sample_type: wgpu::TextureSampleType::Float { filterable: false },
        //                     view_dimension: wgpu::TextureViewDimension::D2,
        //                     multisampled: false,
        //                 },
        //                 count: None,
        //             },
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 3,
        //                 visibility: wgpu::ShaderStages::COMPUTE,
        //                 ty: wgpu::BindingType::StorageTexture {
        //                     access: wgpu::StorageTextureAccess::ReadOnly,
        //                     format: wgpu::TextureFormat::Rgba8Unorm,
        //                     view_dimension: wgpu::TextureViewDimension::D2,
        //                 },
        //                 count: None,
        //             },
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 4,
        //                 visibility: wgpu::ShaderStages::COMPUTE,
        //                 ty: wgpu::BindingType::StorageTexture {
        //                     access: wgpu::StorageTextureAccess::WriteOnly,
        //                     format: wgpu::TextureFormat::Rgba8Unorm,
        //                     view_dimension: wgpu::TextureViewDimension::D2,
        //                 },
        //                 count: None,
        //             },
        //         ],
        //     });

        // let lighting_pipeline_layout =
        //     device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //         label: Some("3D Lighting Pipeline Layout"),
        //         bind_group_layouts: &[&bind_group_layout_for_lighting],
        //         push_constant_ranges: &[],
        //     });
        // let lighting_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        //     label: Some("3D SDF Lighting Pipeline"),
        //     layout: Some(&lighting_pipeline_layout),
        //     module: &shader_for_lighting,
        //     entry_point: Some("cs_lighting"),
        //     compilation_options: Default::default(),
        //     cache: None,
        // });

        // let bind_group_for_lighting = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("3D Lighting Bind Group"),
        //     layout: &bind_group_layout_for_lighting,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: buffer_for_brick.as_entire_binding(),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: buffer_for_commands.as_entire_binding(),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 2,
        //             resource: wgpu::BindingResource::TextureView(&atlas_view_sdf),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 3,
        //             resource: wgpu::BindingResource::TextureView(&atlas_view_color),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 4,
        //             resource: wgpu::BindingResource::TextureView(&atlas_view_color_after_light),
        //         },
        //     ],
        // });

        // Render Shader

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
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
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
                    resource: wgpu::BindingResource::TextureView(&atlas_view_sdf),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&atlas_view_color),
                },
            ],
        });

        Self {
            buffer_for_commands,
            buffer_for_brick,

            atlas_view_sdf,
            atlas_view_color,
            //atlas_view_color_after_light,
            bind_group_layout_for_baking,
            bind_group_for_baking,

            //bind_group_layout_for_lighting,
            //bind_group_for_lighting,
            bind_group_layout_for_render,
            bind_group_for_render,

            baking_pipeline,
            //lighting_pipeline,
            render_pipeline,

            camera_buffer,
            atlas_size,
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
                    resource: wgpu::BindingResource::TextureView(&self.atlas_view_sdf),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.atlas_view_color),
                },
            ],
        });

        self.bind_group_for_baking = bind_group_for_baking
    }

    // pub fn recreate_bind_group_for_lighting(&mut self, device: &Device) {
    //     let bind_group_for_lighting = device.create_bind_group(&wgpu::BindGroupDescriptor {
    //         label: Some("3D Lighting Bind Group"),
    //         layout: &self.bind_group_layout_for_lighting,
    //         entries: &[
    //             wgpu::BindGroupEntry {
    //                 binding: 0,
    //                 resource: self.buffer_for_brick.as_entire_binding(),
    //             },
    //             wgpu::BindGroupEntry {
    //                 binding: 1,
    //                 resource: self.buffer_for_commands.as_entire_binding(),
    //             },
    //             wgpu::BindGroupEntry {
    //                 binding: 2,
    //                 resource: wgpu::BindingResource::TextureView(&self.atlas_view_sdf),
    //             },
    //             wgpu::BindGroupEntry {
    //                 binding: 3,
    //                 resource: wgpu::BindingResource::TextureView(&self.atlas_view_color),
    //             },
    //             wgpu::BindGroupEntry {
    //                 binding: 4,
    //                 resource: wgpu::BindingResource::TextureView(
    //                     &self.atlas_view_color_after_light,
    //                 ),
    //             },
    //         ],
    //     });

    //     self.bind_group_for_lighting = bind_group_for_lighting
    // }

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
                    resource: wgpu::BindingResource::TextureView(&self.atlas_view_sdf),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.atlas_view_color),
                },
            ],
        });

        self.bind_group_for_render = bind_group
    }
}
