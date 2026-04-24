#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShapeVertex {
    pub position: [f32; 2], // Координаты вершины (куда растеризуем)
    pub color: u32,         //[f32; 4],
    // Параметры фигуры:
    pub p_a: [f32; 2], // Точка А (центр прямоугольника или старт линии) | UV - координата текста в текстуре
    pub p_b: [f32; 2], // Точка Б (размер прямоугольника или конец линии)
    pub params: [f32; 4], // [радиус/толщина, тип_фигуры, сглаживание, пусто]
    pub border_color: u32, //[f32; 4],
}

// Типы фигур для params.y
pub const SHAPE_RECT: f32 = 0.0;
pub const SHAPE_LINE: f32 = 1.0;
pub const SHAPE_TEXT: f32 = 2.0;

impl ShapeVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
            0 => Float32x2, // position
            1 => Unorm8x4, // color
            2 => Float32x2, // p_a
            3 => Float32x2, // p_b
            4 => Float32x4, // params
            5 => Unorm8x4, // border_color
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ShapeVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub struct GPUShapeVertex {
    pub mask_pipeline: wgpu::RenderPipeline,
    pub content_pipeline: wgpu::RenderPipeline,
    pub unmask_pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

use std::ops::Range;

use wgpu::{BindGroupLayout, Buffer, Device, Queue, SurfaceConfiguration};

use crate::window::{
    component::base::gpu_render_context::{GpuCommand, GpuRenderContext},
    wgpu::{draw_args::DrawIndexedIndirectArgs, uber_resourse_manager::UberResourceManager},
};

impl GPUShapeVertex {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        uniform_buffer: &Buffer,
        text_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/high/shape_shader.wgsl"));

        // BindGroup — это "вход" для буфера в шейдер
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
            bind_group_layouts: &[&bind_group_layout, text_bind_group_layout], // Сюда передаем наш layout
            push_constant_ranges: &[],
        });

        let mask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Mask Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[ShapeVertex::desc()],
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always, // Игнорируем глубину
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        pass_op: wgpu::StencilOperation::IncrementClamp,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    back: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        pass_op: wgpu::StencilOperation::IncrementClamp,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    read_mask: 0xFF,
                    write_mask: 0xFF,
                },
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let content_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Content Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[ShapeVertex::desc()],
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

        let unmask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Unmask Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[ShapeVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(), // ДОБАВИТЬ
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::empty(),
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(), // Рисуем треугольники
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always, // Игнорируем глубину
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        pass_op: wgpu::StencilOperation::DecrementClamp,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    back: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        pass_op: wgpu::StencilOperation::DecrementClamp,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    read_mask: 0xFF,
                    write_mask: 0xFF,
                },
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            mask_pipeline: mask_pipeline,
            content_pipeline: content_pipeline,
            unmask_pipeline: unmask_pipeline,
            bind_group_layout: bind_group_layout,
            bind_group: bind_group,
        }
    }
}
