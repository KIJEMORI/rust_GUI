#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShapeVertex {
    pub position: [f32; 2], // Координаты вершины (куда растеризуем)
    pub color: [f32; 4],
    // Параметры фигуры:
    pub p_a: [f32; 2], // Точка А (центр прямоугольника или старт линии) | UV - координата текста в текстуре
    pub p_b: [f32; 2], // Точка Б (размер прямоугольника или конец линии)
    pub params: [f32; 4], // [радиус/толщина, тип_фигуры, сглаживание, пусто]
    pub border_color: [f32; 4],
}

// Типы фигур для params.y
pub const SHAPE_RECT: f32 = 0.0;
pub const SHAPE_LINE: f32 = 1.0;
pub const SHAPE_TEXT: f32 = 2.0;

impl ShapeVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
            0 => Float32x2, // position
            1 => Float32x4, // color
            2 => Float32x2, // p_a
            3 => Float32x2, // p_b
            4 => Float32x4, // params
            5 => Float32x4, // border_color
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ShapeVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub struct GPUShapeVertex {
    // pub vertex_buffer: wgpu::Buffer,
    // pub vertex_index_buffer: wgpu::Buffer,
    pub mask_pipeline: wgpu::RenderPipeline,
    pub content_pipeline: wgpu::RenderPipeline,
    pub unmask_pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    // Inderected Args Shape
    //pub shape_indirect_buffer: wgpu::Buffer,
    //pub shape_section_offsets: Vec<Range<usize>>, // Офсеты для каждой панели/линии
    pub active_shape_commands_count: u32,
    //pub next_free_vertex: usize,
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

        // let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Panel Vertex Buffer"),
        //     size: MAX_VERTICES * std::mem::size_of::<ShapeVertex>() as u64,
        //     usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, // ОБЯЗАТЕЛЬНО VERTEX
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

        // let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Text Static Index Buffer"),
        //     contents: bytemuck::cast_slice(&indices),
        //     usage: wgpu::BufferUsages::INDEX,
        // });

        // let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Panel Index Buffer"),
        //     size: MAX_INDICES * std::mem::size_of::<u32>() as u64,
        //     usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });

        // let shape_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Draw Vertex Buffer"),
        //     size: MAX_VERTICES * std::mem::size_of::<DrawIndirectArgs>() as u64,
        //     usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });

        Self {
            // vertex_buffer: vertex_buffer,
            // vertex_index_buffer: index_buffer,
            mask_pipeline: mask_pipeline,
            content_pipeline: content_pipeline,
            unmask_pipeline: unmask_pipeline,
            bind_group: bind_group,
            // shape_indirect_buffer: shape_buffer,
            // shape_section_offsets: Vec::new(),
            active_shape_commands_count: 0,
            // next_free_vertex: 0,
        }
    }

    pub fn update_shape_indirect_buffer(
        &mut self,
        offsets: &[Range<usize>],
        device: &Device,
        queue: &Queue,
        manager: &mut UberResourceManager,
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
            //self.active_shape_commands_count = 0;
            return;
        }

        let start_offset_bytes =
            manager.active_shape_count as u64 * UberResourceManager::INDIRECT_SIZE;
        let required_size =
            start_offset_bytes + (commands.len() as u64 * UberResourceManager::INDIRECT_SIZE);

        // Ресайз буфера команд если нужно
        if manager.indirect_buffer.size() < required_size {
            let new_size = required_size.next_power_of_two();
            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Expanded Indirect Buffer"),
                size: new_size,
                usage: wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            // Копируем уже существующие команды (например, test_cmd или предыдущие слои)
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_buffer_to_buffer(
                &manager.indirect_buffer,
                0,
                &new_buffer,
                0,
                manager.indirect_buffer.size(),
            );
            queue.submit(std::iter::once(encoder.finish()));

            manager.indirect_buffer = new_buffer;
        }
        queue.write_buffer(
            &manager.indirect_buffer,
            start_offset_bytes,
            bytemuck::cast_slice(&commands),
        );

        let added_count = commands.len() as u32;
        self.active_shape_commands_count = added_count; // Локальный счетчик этого менеджера
        manager.active_shape_count += added_count;
    }

    pub fn render(
        &mut self,
        gpu_ctx: &mut GpuRenderContext, // Сделай &mut
        device: &Device,
        queue: &Queue,
        manager: &mut UberResourceManager,
    ) -> usize {
        let len = gpu_ctx.shape_vertices.len();

        {
            let required_verts = gpu_ctx.shape_vertices.len();
            let required_cmds = gpu_ctx.shape_section_offsets.len() as u32;

            manager.ensure_vertex_capacity(device, queue, required_verts);
            manager.ensure_index_capacity(device, required_verts);
            manager.ensure_indirect_capacity(device, queue, required_cmds);
        }

        queue.write_buffer(
            &manager.vertex_buffer,
            0,
            bytemuck::cast_slice(&gpu_ctx.shape_vertices),
        );

        let mut shape_idx = 0;
        for cmd in &mut gpu_ctx.command_sections {
            if let GpuCommand::Shape(s) = cmd {
                s.command_index = shape_idx;
                shape_idx += 1;
            }
        }

        self.update_shape_indirect_buffer(&gpu_ctx.shape_section_offsets, device, queue, manager);

        manager.next_free_vertex = len;
        len
    }
}
