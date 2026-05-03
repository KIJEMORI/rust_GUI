#[cfg(feature = "3d_render")]
use crate::window::wgpu::block_3d::{brick_uniform::GPUBrickRender, instance::GPUInstance3DData};

use crate::window::{
    component::base::gpu_render_context::{GpuCommand, GpuRenderContext},
    wgpu::{
        screen_uniform::ScreenUniform, shape_vertex::GPUShapeVertex, text_vertex::GPUTextVertex,
        uber_resourse_manager::UberResourceManager,
    },
};
use wgpu::{
    Device, Queue, RenderPass, RenderPassColorAttachment, Surface as WgpuSurface,
    SurfaceConfiguration, TextureView, util::DeviceExt,
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
    #[cfg(feature = "3d_render")]
    //instance_3d_manager: GPUInstance3DData,
    brick_manager: GPUBrickRender,
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

        #[cfg(feature = "3d_render")]
        // let gpu_istance_manager =
        //     GPUInstance3DData::new(&device, &config, &shape_vertex.bind_group_layout);
        let brick_manager =
            GPUBrickRender::new(&device, &queue, &config, &shape_vertex.bind_group_layout);

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
            #[cfg(feature = "3d_render")]
            brick_manager: brick_manager, //instance_3d_manager: gpu_istance_manager,
        }
    }

    pub fn prepare_gpu_data(&mut self, gpu_ctx: &mut GpuRenderContext) {
        self.uber_manager.start_frame();

        // Обновляем текстуру атласа (SDF пиксели)
        self.text_vertex.atlas.update_atlas(&self.queue);

        // Проверяем, хватает ли места в Uber-буферах
        // Если произойдет ресайз индекса, Uber сам обновит свой буфер
        self.uber_manager.ensure_vertex_capacity(
            &self.device,
            &self.queue,
            gpu_ctx.shape_vertices.len(),
        );
        self.uber_manager
            .ensure_index_capacity(&self.device, gpu_ctx.shape_vertices.len());
        self.uber_manager.ensure_indirect_capacity(
            &self.device,
            &self.queue,
            gpu_ctx.indirect_cmd.len() as u32,
        );

        // Пишем вершины (теперь они содержат и фигуры, и текст)
        self.queue.write_buffer(
            &self.uber_manager.vertex_buffer,
            0,
            bytemuck::cast_slice(&gpu_ctx.shape_vertices),
        );

        self.queue.write_buffer(
            &self.uber_manager.indirect_buffer,
            0,
            bytemuck::cast_slice(&gpu_ctx.indirect_cmd),
        );

        #[cfg(feature = "3d_render")]
        {
            self.queue.write_buffer(
                &self.brick_manager.camera_buffer,
                0,
                bytemuck::cast_slice(&[gpu_ctx.camera_data]),
            );

            if !gpu_ctx.bake_cmds.is_empty() {
                // self.queue.write_buffer(
                //     &self.instance_3d_manager.camera_buffer,
                //     0,
                //     bytemuck::cast_slice(&[gpu_ctx.camera_data]),
                // );

                //let storage_buffer = &mut self.instance_3d_manager.buffer;

                let was_resized_baking = write_to_gpu_buffer(
                    &self.device,
                    &self.queue,
                    &mut self.brick_manager.buffer_for_brick,
                    &gpu_ctx.bake_cmds,
                    "3D Bake Storage Buffer",
                    wgpu::BufferUsages::STORAGE,
                );

                let was_resized_instance = write_to_gpu_buffer(
                    &self.device,
                    &self.queue,
                    &mut self.brick_manager.buffer_for_commands,
                    &gpu_ctx.instances_3d,
                    "3D Instance Storage Buffer",
                    wgpu::BufferUsages::STORAGE,
                );

                if was_resized_baking || was_resized_instance {
                    // Если любой из буферов изменился, обновляем всё, где они участвуют
                    self.brick_manager
                        .recreate_bind_group_for_baking(&self.device);
                    self.brick_manager
                        .recreate_bind_group_for_render(&self.device);
                }
            }
        }
    }

    pub fn render(&mut self, gpu_ctx: &GpuRenderContext, view: &TextureView) {
        const STRIDE: u64 = 20; // Размер DrawIndexedIndirectArgs (5 * u32)
        let use_multi_draw = self
            .device
            .features()
            .contains(wgpu::Features::MULTI_DRAW_INDIRECT);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        if gpu_ctx.bake_cmds.len() > 0 {
            #[cfg(feature = "3d_render")]
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                cpass.set_pipeline(&self.brick_manager.baking_pipeline);
                cpass.set_bind_group(0, &self.brick_manager.bind_group_for_baking, &[]);
                // Запускаем ровно столько групп, сколько у нас кирпичей на запекание
                println!("Bake commands count: {}", gpu_ctx.bake_cmds.len());
                cpass.dispatch_workgroups(gpu_ctx.bake_cmds.len() as u32, 1, 1);
            }
        }

        {
            // Начинаем проход отрисовки (Render Pass)
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }), // Очистка фона
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_stencil_view, // Твоя вьюшка, которую ты обновляешь в resize
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0), // Просто очистка, даже если не используем
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0), // Очищаем трафарет в 0 каждый кадр
                        store: wgpu::StoreOp::Store,  // Сохраняем результат для тестов в этом кадре
                    }),
                }),
                ..Default::default()
            });

            render_pass.set_vertex_buffer(0, self.uber_manager.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.uber_manager.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );

            // Устанавливаем стандартные бинд-группы для 2D
            render_pass.set_bind_group(0, &self.shape_vertex.bind_group, &[]); // ScreenUniform
            render_pass.set_bind_group(1, &self.text_vertex.text_bind_group, &[]); // Texture + Sampler

            let mut i = 0;
            let mut was_3d_active = false;
            let commands = &gpu_ctx.command_sections;

            while i < commands.len() {
                // Извлекаем тип и параметры первой команды в текущем батче
                let (level, is_mask, is_text, is_instance, is_unmask, start_idx) = match &commands
                    [i]
                {
                    GpuCommand::Shape(s) => {
                        (s.level, s.is_mask, false, false, false, s.command_index)
                    }
                    GpuCommand::Text(s) => {
                        (s.level, s.is_mask, true, false, false, s.command_index)
                    }
                    GpuCommand::Unmask(s) => (s.level, false, false, false, true, s.command_index),
                    GpuCommand::Instance(s) => {
                        (s.level, false, false, true, false, s.command_index)
                    }
                };

                // --- ПЕРЕКЛЮЧЕНИЕ ПАЙПЛАЙНОВ И БИНД-ГРУПП ---

                if is_instance {
                    // Переключаемся на 3D
                    #[cfg(feature = "3d_render")]
                    render_pass.set_pipeline(&self.brick_manager.render_pipeline);
                    // Переопределяем Group 1 (вместо текстур ставим Камеру + Инстансы)
                    #[cfg(feature = "3d_render")]
                    render_pass.set_bind_group(1, &self.brick_manager.bind_group_for_render, &[]);
                    was_3d_active = true;
                } else {
                    // Если вернулись к 2D (Shape или Text)
                    if was_3d_active {
                        render_pass.set_bind_group(1, &self.text_vertex.text_bind_group, &[]);
                        was_3d_active = false;
                    }

                    if is_unmask {
                        render_pass.set_pipeline(&self.shape_vertex.unmask_pipeline);
                    } else if is_mask {
                        render_pass.set_pipeline(&self.shape_vertex.mask_pipeline);
                    } else {
                        render_pass.set_pipeline(&self.shape_vertex.content_pipeline);
                    }
                }

                // Установка трафарета (Stencil)
                let target_stencil = if is_mask {
                    level.saturating_sub(1)
                } else {
                    level
                };
                render_pass.set_stencil_reference(target_stencil);

                let mut batch_count = 0;
                let mut j = i;
                while j < commands.len() {
                    let matches = match &commands[j] {
                        GpuCommand::Shape(s) => {
                            !is_instance
                                && !is_unmask
                                && !is_text
                                && s.level == level
                                && s.is_mask == is_mask
                        }
                        GpuCommand::Text(s) => {
                            !is_instance
                                && !is_unmask
                                && is_text
                                && s.level == level
                                && s.is_mask == is_mask
                        }
                        GpuCommand::Unmask(s) => is_unmask && s.level == level,
                        GpuCommand::Instance(s) => is_instance && s.level == level,
                    };

                    if matches {
                        batch_count += 1;
                        j += 1;
                    } else {
                        break;
                    }
                }

                let offset = start_idx as u64 * STRIDE;

                // Выбираем правильный косвенный буфер (Indirect Buffer)
                let buffer = &self.uber_manager.indirect_buffer;

                if use_multi_draw && batch_count > 1 {
                    render_pass.multi_draw_indexed_indirect(buffer, offset, batch_count);
                } else {
                    for k in 0..batch_count {
                        let current_cmd_idx = match &commands[i + k as usize] {
                            GpuCommand::Shape(s) => s.command_index,
                            GpuCommand::Text(s) => s.command_index,
                            GpuCommand::Unmask(s) => s.command_index,
                            GpuCommand::Instance(s) => s.command_index,
                        };
                        let single_offset = current_cmd_idx as u64 * STRIDE;
                        render_pass.draw_indexed_indirect(buffer, single_offset);
                    }
                }

                i = j;
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
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

fn write_to_gpu_buffer<T: bytemuck::Pod>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    gpu_buffer: &mut wgpu::Buffer,
    data: &[T],
    label: &str,
    usage: wgpu::BufferUsages,
) -> bool {
    let size = (data.len() * std::mem::size_of::<T>()) as u64;

    let mut resize = false;

    // Если данных больше, чем размер текущего буфера на GPU
    if size > gpu_buffer.size() {
        // Реаллокация (как у Vec): берем с запасом (x1.5 или x2)
        *gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: size * 2,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        println!("Buffer {} resized to {} bytes", label, size * 2);
        resize = true;
    }

    queue.write_buffer(gpu_buffer, 0, bytemuck::cast_slice(data));
    resize
}
