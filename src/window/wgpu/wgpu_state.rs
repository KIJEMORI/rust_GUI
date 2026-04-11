use std::{ops::Range, time::Instant};

use wgpu::{
    Device, Queue, RenderPass, Surface as WgpuSurface, SurfaceConfiguration, TextureView,
    util::DeviceExt,
};

use crate::window::{
    component::base::gpu_render_context::{GpuCommand, GpuRenderContext},
    wgpu::{
        draw_args::{DrawIndexedIndirectArgs, DrawIndirectArgs},
        screen_uniform::{ScreenUniform, ScrollData},
        shape_vertex::GPUShapeVertex,
        text_vertex::{GPUTextVertex, TextVertex},
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
    pub scroll_storage_buffer: wgpu::Buffer,
    pub depth_stencil_view: TextureView,
}

pub const MAX_VERTICES: u64 = 36_000;
pub const MAX_INDICES: u64 = 36_000;

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

        let initial_offsets = vec![
            ScrollData {
                offsets: [0.0, 0.0, 0.0, 0.0]
            };
            100
        ];

        let scroll_storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scroll Storage Buffer"),
            contents: bytemuck::cast_slice(&initial_offsets),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let shape_vertex =
            GPUShapeVertex::new(&device, &config, &uniform_buffer, &scroll_storage_buffer);

        let text_vertex =
            GPUTextVertex::new(&device, &config, &uniform_buffer, &scroll_storage_buffer);

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

        Self {
            //Base
            surface: surface,
            device: device,
            queue: queue,
            config: config,
            shape_vertex: shape_vertex,
            text_vertex: text_vertex,
            uniform_buffer: uniform_buffer,
            scroll_storage_buffer: scroll_storage_buffer,
            depth_stencil_view: depth_stencil_view,
        }
    }

    // pub fn update_section_in_arena(&mut self, s_idx: usize, new_verts: Vec<TextVertex>) {
    //     let old_range = self.section_offsets.get(s_idx).cloned().unwrap_or(0..0);
    //     let new_len = new_verts.len();
    //     let old_len = old_range.end - old_range.start;

    //     // splice сам раздвинет или сузит вектор, если длины не совпадают
    //     self.last_text_vertices
    //         .splice(old_range.clone(), new_verts.into_iter());

    //     // Если длина изменилась, все последующие офсеты "поплыли" — корректируем их
    //     if new_len != old_len {
    //         let diff = new_len as i32 - old_len as i32;

    //         // Обновляем текущий офсет
    //         self.section_offsets[s_idx] = old_range.start..(old_range.start + new_len);

    //         // Сдвигаем все офсеты, которые идут ПОСЛЕ этой строки
    //         for i in (s_idx + 1)..self.section_offsets.len() {
    //             let r = &self.section_offsets[i];
    //             self.section_offsets[i] =
    //                 (r.start as i32 + diff) as usize..(r.end as i32 + diff) as usize;
    //         }

    //         // ТАК КАК ВЕСЬ БУФЕР СДВИНУЛСЯ — ПЕРЕЗАПИСЫВАЕМ ЕГО В GPU ПОЛНОСТЬЮ
    //         self.write_text_vertices();
    //     } else {
    //         let offset_bytes = (old_range.start * std::mem::size_of::<TextVertex>()) as u64;
    //         self.queue.write_buffer(
    //             &self.text_vertex_buffer,
    //             offset_bytes,
    //             bytemuck::cast_slice(
    //                 &self.last_text_vertices[old_range.start..old_range.start + new_len],
    //             ),
    //         );
    //     }
    // }

    pub fn update_shape_indirect_buffer(&mut self, offsets: &[Range<usize>]) {
        self.shape_vertex
            .update_shape_indirect_buffer(offsets, &self.device, &self.queue);
    }

    pub fn ensure_gpu_capacity(&mut self, required_count: usize) {
        self.text_vertex
            .ensure_gpu_capacity(required_count, &self.device, &self.queue);
    }

    pub fn update_section_direct_gpu(&mut self, s_idx: usize, new_verts: Vec<TextVertex>) {
        self.text_vertex
            .update_section_direct_gpu(s_idx, new_verts, &self.device, &self.queue);
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

    //         self.wasted_vertices = 0;
    //         self.update_indirect_buffer(); // Команды отрисовки теперь указывают на новые места
    //         return true;
    //     }
    //     false
    // }

    pub fn update_indirect_buffer(&mut self) {
        self.text_vertex
            .update_indirect_buffer(&self.device, &self.queue);
    }

    pub fn is_defrag_worth_it(&self) -> bool {
        self.text_vertex.is_defrag_worth_it()
    }

    pub fn perform_true_defragmentation(&mut self) {
        self.text_vertex
            .perform_true_defragmentation(&self.device, &self.queue);
    }

    pub fn render_shape(&mut self, gpu_ctx: &GpuRenderContext) {
        self.shape_vertex.render(gpu_ctx, &self.device, &self.queue);
    }

    pub fn render_text(&mut self, gpu_ctx: &GpuRenderContext, last_render: Instant) {
        self.text_vertex
            .render(gpu_ctx, last_render, &self.device, &self.queue);
    }

    pub fn render(&mut self, gpu_ctx: &GpuRenderContext, render_pass: &mut RenderPass<'_>) {
        render_pass.set_index_buffer(
            self.shape_vertex.vertex_index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );

        let shape_stride = std::mem::size_of::<DrawIndexedIndirectArgs>() as u64;
        let text_stride = std::mem::size_of::<DrawIndirectArgs>() as u64;

        let mut last_pipeline_is_mask: Option<bool> = None;

        render_pass.set_pipeline(&self.shape_vertex.mask_pipeline);

        for cmd in &gpu_ctx.command_sections {
            match cmd {
                GpuCommand::Shape(section) => {
                    render_pass.set_vertex_buffer(0, self.shape_vertex.vertex_buffer.slice(..));
                    render_pass.set_bind_group(0, &self.shape_vertex.bind_group, &[]);

                    if section.is_mask {
                        if last_pipeline_is_mask != Some(true) {
                            render_pass.set_pipeline(&self.shape_vertex.mask_pipeline);
                        }
                        render_pass.set_stencil_reference(section.level - 1);
                        // Заполняем трафарет
                    } else {
                        if last_pipeline_is_mask != Some(false) {
                            render_pass.set_pipeline(&self.shape_vertex.content_pipeline);
                        }
                        render_pass.set_stencil_reference(section.level);

                        // Рисуем контент (он автоматически обрежется)
                    }
                    last_pipeline_is_mask = Some(section.is_mask);
                    // Меняем уровень трафарета ОДИН раз для всей группы

                    if self
                        .device
                        .features()
                        .contains(wgpu::Features::MULTI_DRAW_INDIRECT)
                    {
                        // Рисуем всю группу команд одним вызовом
                        render_pass.multi_draw_indexed_indirect(
                            &self.shape_vertex.shape_indirect_buffer,
                            section.command_index as u64 * shape_stride,
                            section.command_count,
                        );
                    } else {
                        // Fallback: рисуем каждую команду в группе по отдельности
                        for i in 0..section.command_count {
                            let offset = (section.command_index + i) as u64 * shape_stride;
                            render_pass.draw_indexed_indirect(
                                &self.shape_vertex.shape_indirect_buffer,
                                offset,
                            );
                        }
                    }
                }
                GpuCommand::Text(section) => {
                    last_pipeline_is_mask = None;
                    if self.text_vertex.active_commands_count > 0 {
                        render_pass.set_pipeline(&self.text_vertex.text_pipeline);
                        render_pass.set_bind_group(0, &self.text_vertex.bind_group, &[]);
                        render_pass.set_bind_group(1, &self.text_vertex.text_bind_group, &[]);
                        render_pass
                            .set_vertex_buffer(0, self.text_vertex.text_vertex_buffer.slice(..));

                        // Идем по группам текста, разделенным по уровням вложенности
                        // Устанавливаем уровень трафарета для этого блока текста
                        render_pass.set_stencil_reference(section.level);

                        if self
                            .device
                            .features()
                            .contains(wgpu::Features::MULTI_DRAW_INDIRECT)
                        {
                            render_pass.multi_draw_indirect(
                                &self.text_vertex.indirect_buffer,
                                section.command_index as u64 * text_stride,
                                section.command_count,
                            );
                        } else {
                            for i in 0..section.command_count {
                                let offset = (section.command_index + i) as u64 * text_stride;
                                render_pass
                                    .draw_indirect(&self.text_vertex.indirect_buffer, offset);
                            }
                        }
                    }
                }
            }
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

            // ОБНОВЛЯЕМ ТРАФАРЕТ ТУТ
            self.depth_stencil_view = depth_stencil_view
        }
    }
}
