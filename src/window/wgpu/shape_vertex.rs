#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShapeVertex {
    pub position: [f32; 2], // Координаты вершины (куда растеризуем)
    pub color: [f32; 4],
    // Параметры фигуры:
    pub p_a: [f32; 2],    // Точка А (центр прямоугольника или старт линии)
    pub p_b: [f32; 2],    // Точка Б (размер прямоугольника или конец линии)
    pub params: [f32; 4], // [радиус/толщина, тип_фигуры, сглаживание, пусто]
    pub border_color: [f32; 4],
    pub scroll_id: u32,
}

// Типы фигур для params.y
const SHAPE_RECT: f32 = 0.0;
const SHAPE_LINE: f32 = 1.0;

impl ShapeVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
            0 => Float32x2, // position
            1 => Float32x4, // color
            2 => Float32x2, // p_a
            3 => Float32x2, // p_b
            4 => Float32x4, // params
            5 => Float32x4, // border_color
            6 => Uint32,
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ShapeVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub struct GPUShapeVertex {
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_index_buffer: wgpu::Buffer,
    pub mask_pipeline: wgpu::RenderPipeline,
    pub content_pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    // Inderected Args Shape
    pub shape_indirect_buffer: wgpu::Buffer,
    pub shape_section_offsets: Vec<Range<usize>>, // Офсеты для каждой панели/линии
    pub active_shape_commands_count: u32,
}

use std::ops::Range;

use wgpu::{
    Buffer, Device, Queue, RenderPass, StencilOperation, SurfaceConfiguration, util::DeviceExt,
};

use crate::window::{
    component::base::gpu_render_context::GpuRenderContext,
    wgpu::{
        draw_args::{DrawIndexedIndirectArgs, DrawIndirectArgs},
        wgpu_state::{MAX_INDICES, MAX_VERTICES},
    },
};

impl GPUShapeVertex {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        uniform_buffer: &Buffer,
        scroll_storage_buffer: &Buffer,
    ) -> Self {
        let shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/high/shape_shader.wgsl"));

        // BindGroup — это "вход" для буфера в шейдер
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout], // Сюда передаем наш layout
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

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Panel Vertex Buffer"),
            size: MAX_VERTICES * std::mem::size_of::<ShapeVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, // ОБЯЗАТЕЛЬНО VERTEX
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Panel Index Buffer"),
            size: MAX_INDICES * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shape_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Draw Vertex Buffer"),
            size: MAX_VERTICES * std::mem::size_of::<DrawIndirectArgs>() as u64,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer: vertex_buffer,
            vertex_index_buffer: index_buffer,
            mask_pipeline: mask_pipeline,
            content_pipeline: content_pipeline,
            bind_group: bind_group,
            shape_indirect_buffer: shape_buffer,
            shape_section_offsets: Vec::new(),
            active_shape_commands_count: 0,
        }
    }

    pub fn update_shape_indirect_buffer(
        &mut self,
        offsets: &[Range<usize>],
        device: &Device,
        queue: &Queue,
    ) {
        let commands: Vec<DrawIndexedIndirectArgs> = offsets
            .iter()
            .map(|range| {
                let vertex_count = (range.end - range.start) as u32;
                let quad_count = vertex_count / 4;
                DrawIndexedIndirectArgs {
                    index_count: quad_count * 6,
                    instance_count: 1,
                    // Смещение в буфере индексов (каждые 4 вершины — это 6 индексов)
                    first_index: (range.start / 4 * 6) as u32,
                    base_vertex: 0,
                    first_instance: 0,
                }
            })
            .collect();

        if commands.is_empty() {
            self.active_shape_commands_count = 0;
            return;
        }

        let size = (commands.len() * std::mem::size_of::<DrawIndexedIndirectArgs>()) as u64;

        // Ресайз буфера команд если нужно
        if self.shape_indirect_buffer.size() < size {
            self.shape_indirect_buffer.destroy();
            self.shape_indirect_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Shape Indirect Buffer"),
                    contents: bytemuck::cast_slice(&commands),
                    usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
                });
        } else {
            queue.write_buffer(
                &self.shape_indirect_buffer,
                0,
                bytemuck::cast_slice(&commands),
            );
        }

        self.active_shape_commands_count = commands.len() as u32;
    }

    pub fn render(&mut self, gpu_ctx: &GpuRenderContext, device: &Device, queue: &Queue) {
        queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&gpu_ctx.shape_vertices),
        );

        queue.write_buffer(
            &self.vertex_index_buffer,
            0,
            bytemuck::cast_slice(&gpu_ctx.shape_indices),
        );

        self.update_shape_indirect_buffer(&gpu_ctx.shape_section_offsets, device, queue);
    }
}
