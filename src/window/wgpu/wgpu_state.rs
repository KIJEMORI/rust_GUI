use wgpu::{
    Device, Queue, RenderPass, Surface as WgpuSurface, SurfaceConfiguration, TextureView,
    util::DeviceExt,
};

use crate::window::{
    component::base::gpu_render_context::{GpuCommand, GpuRenderContext},
    wgpu::{
        screen_uniform::ScreenUniform, shape_vertex::GPUShapeVertex, text_vertex::GPUTextVertex,
        uber_resourse_manager::UberResourceManager,
    },
};
pub struct WgpuState {
    // База
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    // Vertex Panel
    pub shape_vertex: GPUShapeVertex,
    pub text_vertex: GPUTextVertex,
    pub uniform_buffer: wgpu::Buffer,
    pub depth_stencil_view: TextureView,
    uber_manager: UberResourceManager,
}

pub const MAX_VERTICES: u64 = 36_000;
pub const MAX_INDICES: u64 = MAX_VERTICES * 2;

impl WgpuState {
    pub fn new(
        surface: WgpuSurface<'static>,
        device: Device,
        queue: Queue,
        config: SurfaceConfiguration,
    ) -> Self {
        let screen_uniform = ScreenUniform {
            size: [800.0, 600.0],
            scroll_offset: [0.0, 0.0],
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[screen_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let text_vertex = GPUTextVertex::new(&device);

        let shape_vertex = GPUShapeVertex::new(
            &device,
            &config,
            &uniform_buffer,
            &text_vertex.text_bind_group_layout,
        );

        let depth_stencil_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Stencil"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_stencil_view =
            depth_stencil_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let uber_manager = UberResourceManager::new(&device);

        Self {
            //Base
            surface: surface,
            device: device,
            queue: queue,
            config: config,
            shape_vertex: shape_vertex,
            text_vertex: text_vertex,
            uniform_buffer: uniform_buffer,
            depth_stencil_view: depth_stencil_view,
            uber_manager: uber_manager,
        }
    }

    pub fn prepare_gpu_data(&mut self, gpu_ctx: &mut GpuRenderContext) {
        self.uber_manager.start_frame(); // Обнулили active_shape_count

        self.shape_vertex
            .render(gpu_ctx, &self.device, &self.queue, &mut self.uber_manager);

        let shapes_base_idx = self.uber_manager.active_shape_count;

        let mut shape_counter = 0;
        let mut text_counter = 0;
        for cmd in &mut gpu_ctx.command_sections {
            match cmd {
                GpuCommand::Shape(s) => {
                    s.command_index = shape_counter;
                    shape_counter += 1;
                }
                GpuCommand::Text(s) => {
                    // Текст всегда идет строго после всех шейпов (включая селект)
                    s.command_index = shapes_base_idx + text_counter;
                    text_counter += 1;
                }
                GpuCommand::Unmask(s) => {
                    s.command_index = shape_counter;
                    shape_counter += 1;
                }
            }
        }

        self.text_vertex.render(
            gpu_ctx,
            &self.device,
            &self.queue,
            &mut self.uber_manager,
            shapes_base_idx,
        );
    }
    pub fn render(&mut self, gpu_ctx: &GpuRenderContext, render_pass: &mut RenderPass<'_>) {
        const STRIDE: u64 = 20;
        let use_multi_draw = self
            .device
            .features()
            .contains(wgpu::Features::MULTI_DRAW_INDIRECT);

        render_pass.set_vertex_buffer(0, self.uber_manager.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            self.uber_manager.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.set_bind_group(0, &self.shape_vertex.bind_group, &[]);
        render_pass.set_bind_group(1, &self.text_vertex.text_bind_group, &[]);

        let mut i = 0;
        let mut shape_counter = 0;
        let mut text_counter = 0;
        let shapes_base = gpu_ctx.shape_section_offsets.len() as u32;

        let commands = &gpu_ctx.command_sections;

        let mut current_state: Option<(bool, bool, u32)> = None;

        while i < commands.len() {
            let (level, is_mask, is_text, unmask) = match &commands[i] {
                GpuCommand::Shape(s) => (s.level, s.is_mask, false, false),
                GpuCommand::Text(s) => (s.level, s.is_mask, true, false),
                GpuCommand::Unmask(s) => (s.level, false, false, true),
            };

            let target_stencil = if is_mask {
                level.saturating_sub(1)
            } else {
                level
            };

            let needs_update = match current_state {
                None => true,
                Some((m, u, l)) => m != is_mask || u != unmask || l != target_stencil,
            };

            if needs_update {
                if unmask {
                    render_pass.set_pipeline(&self.shape_vertex.unmask_pipeline);
                    render_pass.set_stencil_reference(target_stencil);
                    current_state = Some((false, true, target_stencil));
                } else if is_mask {
                    render_pass.set_pipeline(&self.shape_vertex.mask_pipeline);
                    render_pass.set_stencil_reference(target_stencil);
                    current_state = Some((true, false, target_stencil));
                } else {
                    render_pass.set_pipeline(&self.shape_vertex.content_pipeline);
                    render_pass.set_stencil_reference(target_stencil);
                    current_state = Some((false, false, target_stencil));
                }
            }

            let mut batch_count = 0;
            let mut j = i;
            while j < commands.len() {
                let matches = match &commands[j] {
                    GpuCommand::Shape(s) => s.level == level && s.is_mask == is_mask && !is_text,
                    GpuCommand::Text(s) => s.level == level && s.is_mask == is_mask && is_text,
                    GpuCommand::Unmask(s) => s.level == level && unmask,
                };

                if matches {
                    batch_count += 1;
                    j += 1;
                } else {
                    break;
                }
            }
            let start_idx = if is_text {
                shapes_base + text_counter
            } else {
                shape_counter
            };
            let offset = start_idx as u64 * STRIDE;

            if use_multi_draw && batch_count > 1 {
                render_pass.multi_draw_indexed_indirect(
                    &self.uber_manager.indirect_buffer,
                    offset,
                    batch_count,
                );
            } else {
                // Если фича выключена или в батче всего 1 элемент, рисуем по старинке
                for k in 0..batch_count {
                    let single_offset = (start_idx + k) as u64 * STRIDE;
                    render_pass
                        .draw_indexed_indirect(&self.uber_manager.indirect_buffer, single_offset);
                }
            }

            // Обновляем глобальные счетчики
            if is_text {
                text_counter += batch_count;
            } else {
                shape_counter += batch_count;
            }
            i = j;
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            let depth_stencil_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Stencil"),
                size: wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let depth_stencil_view =
                depth_stencil_texture.create_view(&wgpu::TextureViewDescriptor::default());

            self.depth_stencil_view = depth_stencil_view
        }
    }
}

#[derive(PartialEq)]
enum PipelineType {
    Shape,
    Text,
}
