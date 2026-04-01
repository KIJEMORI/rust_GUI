use wgpu::{
    Device, Queue, Surface as WgpuSurface, SurfaceConfiguration,
    util::{DeviceExt, StagingBelt},
};
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, ab_glyph};

use crate::window::wgpu::{screen_uniform::ScreenUniform, vertex::Vertex};
pub struct WgpuState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub vertex_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub panel_pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub glyph_brush: GlyphBrush<()>,
    pub staging_belt: StagingBelt,
}

const MAX_VERTICES: u64 = 10_000;

impl WgpuState {
    pub fn new(
        surface: WgpuSurface<'static>,
        device: Device,
        queue: Queue,
        config: SurfaceConfiguration,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shader1.wgsl"));

        // В WgpuState добавьте:
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

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    blend: Some(wgpu::BlendState::REPLACE),
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

        let font = ab_glyph::FontArc::try_from_slice(include_bytes!(
            "../component/base/Fonts/calibri.ttf"
        ))
        .unwrap();
        let glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, config.format);
        let staging_belt = StagingBelt::new(1024);

        Self {
            surface: surface,
            device: device,
            queue: queue,
            config: config,
            vertex_buffer: vertex_buffer,
            uniform_buffer: uniform_buffer,
            panel_pipeline: render_pipeline,
            bind_group: bind_group,
            glyph_brush: glyph_brush,
            staging_belt: staging_belt,
        }
    }
}
