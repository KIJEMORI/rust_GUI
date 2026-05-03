use wgpu::{BindGroupLayout, Device, SurfaceConfiguration, VertexState, util::DeviceExt};

use crate::window::wgpu::{block_3d::camera_uniform::CameraUniform, shape_vertex::ShapeVertex};

#[repr(C, align(16))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance3DData {
    pub inv_transform: [f32; 16], // 64 байта (выровнено по 16)
    pub params: [f32; 4],         // 16 байт (выровнено по 16)
    pub color: u32,               // 4 байта
    pub _padding: [u32; 3],       // 12 байт (4*3)
}

pub struct GPUInstance3DData {
    pub buffer: wgpu::Buffer,
    pub camera_buffer: wgpu::Buffer,
    pub instance_pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}
impl GPUInstance3DData {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        global_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/high/3d_shader.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("3D Scene Bind Group Layout"),
            entries: &[
                // Binding 0: Camera Uniform
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
                // Binding 1: Instances Storage Buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("3D Render Pipeline Layout"),
                bind_group_layouts: &[
                    global_bind_group_layout, // Группа 0
                    &bind_group_layout,       // Группа 1 (твои инстансы)
                ],
                push_constant_ranges: &[],
            });

        let instance_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Instance Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[ShapeVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("3D Instance Storage Buffer"),
            size: (std::mem::size_of::<Instance3DData>() * 15000) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_uniform = CameraUniform::default();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("3D Scene Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(), // Твой буфер камеры
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffer.as_entire_binding(), // Твой буфер моделей
                },
            ],
        });

        Self {
            buffer,
            camera_buffer,
            instance_pipeline,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn recreate_bind_group(&mut self, device: &Device) {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("3D Scene Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.camera_buffer.as_entire_binding(), // Твой буфер камеры
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.buffer.as_entire_binding(), // Твой буфер моделей
                },
            ],
        });

        self.bind_group = bind_group
    }
}
