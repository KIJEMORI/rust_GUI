use std::sync::Arc;
use wgpu::{Instance, RenderPassColorAttachment};
use wgpu_glyph::Section;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::window::component::base::component_type::SharedDrawable;
use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::button::ButtonManager;
use crate::window::component::interface::button_manager_control::ButtonManagerControl;
use crate::window::component::interface::component_control::{ComponentControl, PanelControl};
use crate::window::component::interface::drawable::{Drawable, InternalAccess};
use crate::window::component::interface::layout::Layout;
use crate::window::component::panel::Panel;
use crate::window::wgpu::wgpu_state::WgpuState;

#[derive(Default)]
pub struct AppWinit {
    window: Option<Arc<Window>>,
    state: Option<WgpuState>,
    panel: Panel,
    button_manager: ButtonManager,
    cursor_position: (u16, u16),
}

impl ApplicationHandler for AppWinit {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("LOL");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN, // Или DX12, проверь что ест меньше
            flags: wgpu::InstanceFlags::empty(),
            ..Default::default()
        });

        // Создаем Surface (холст окна) - требует 'static lifetime или Arc
        let surface = instance.create_surface(window.clone()).unwrap();

        // Запрашиваем видеокарту (Адаптер)
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("Не удалось найти видеокарту");

        // Создаем логическое устройство и очередь команд
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();

        // Конфигурация поверхности под размер окна
        let size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        self.state = Some(WgpuState::new(surface, device, queue, config));
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                // if let (Some(surface), Some(_)) = (self.surface.as_mut(), self.window.as_ref()) {
                //     if let (Some(w), Some(h)) =
                //         (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                //     {
                //         surface.resize(w, h).unwrap();

                //         if let Some(window) = self.window.as_ref() {
                //             let size = window.inner_size();

                //             self.panel.set_height(size.height as u16);
                //             self.panel.set_width(size.width as u16);

                //             let area = Rect::new(0, 0, size.width as u16, size.height as u16);

                //             self.panel.resize(&area);

                //             self.window.as_ref().unwrap().request_redraw();
                //         }
                //     }
                // }
                //
                if let Some(state) = self.state.as_mut() {
                    // 1. Обновляем конфиг wgpu (чтобы не было растягивания картинки и утечек)
                    state.config.width = size.width.max(1);
                    state.config.height = size.height.max(1);
                    state.surface.configure(&state.device, &state.config);

                    // Обновляем размеры корневой панели
                    // (Размеры в UI у вас u16, а winit дает u32)
                    let width = size.width as u16;
                    let height = size.height as u16;

                    self.panel.set_width(width);
                    self.panel.set_height(height);

                    // Пересчитываем положение всех дочерних элементов (Layout)
                    // Создаем Rect на весь экран
                    let screen_area =
                        crate::window::component::base::area::Rect::new(0, 0, width, height);
                    self.panel.resize(&screen_area);

                    // Обновляем Uniform буфер (размер окна для шейдера)
                    // Это заставит шейдер правильно пересчитать координаты (-1..1)
                    let screen_uniform = [size.width as f32, size.height as f32, 0.0, 0.0]; // +padding
                    state.queue.write_buffer(
                        &state.uniform_buffer,
                        0,
                        bytemuck::cast_slice(&screen_uniform),
                    );

                    // Просим окно перерисоваться
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let state = self.state.as_mut().unwrap();

                // Получаем текущий кадр из видеопамяти
                let output = state.surface.get_current_texture().unwrap();
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Создаем "записчик" команд
                let mut encoder =
                    state
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Main Render Encoder"),
                        });

                let mut gpu_ctx = GpuRenderContext {
                    vertices: Vec::new(),
                    texts: Vec::new(),
                };
                self.panel.print(&mut gpu_ctx, &self.panel.base.rect);

                {
                    // Начинаем проход отрисовки (Render Pass)
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view,
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
                        ..Default::default()
                    });

                    render_pass.set_pipeline(&state.panel_pipeline);
                    render_pass.set_bind_group(0, &state.bind_group, &[]);

                    if !gpu_ctx.vertices.is_empty() {
                        // Создаем буфер точно под текущее количество вершин

                        state.queue.write_buffer(
                            &state.vertex_buffer,
                            0,
                            bytemuck::cast_slice(&gpu_ctx.vertices),
                        );

                        render_pass.set_vertex_buffer(0, state.vertex_buffer.slice(..));

                        render_pass.draw(0..gpu_ctx.vertices.len() as u32, 0..1);
                        gpu_ctx.vertices.clear()
                    }
                }

                for text_data in &gpu_ctx.texts {
                    state.glyph_brush.queue(Section {
                        screen_position: (text_data.x, text_data.y),
                        bounds: (state.config.width as f32, state.config.height as f32),
                        text: vec![
                            wgpu_glyph::Text::new(&text_data.text)
                                .with_color(text_data.color) // Попробуйте чисто белый для теста
                                .with_scale(text_data.size),
                        ],
                        ..Section::default()
                    });
                }

                if !gpu_ctx.texts.is_empty() {
                    state
                        .glyph_brush
                        .draw_queued(
                            &state.device,
                            &mut state.staging_belt, // Рекомендуется использовать пояс для скорости
                            &mut encoder,
                            &view,
                            state.config.width,
                            state.config.height,
                        )
                        .unwrap();
                }

                // Отправляем записанные команды на выполнение в видеокарту
                state.staging_belt.finish();
                state.queue.submit(std::iter::once(encoder.finish()));
                state.staging_belt.recall();
                output.present();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = (position.x as u16, position.y as u16);

                //println!("{} {}", self.cursor_position.0, self.cursor_position.1);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left
                    && state == winit::event::ElementState::Pressed
                {
                    let (mx, my) = self.cursor_position;
                    if self.button_manager.click(mx as u16, my as u16) {
                        if let Some(window) = self.window.as_ref() {
                            window.request_redraw();
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

impl ComponentControl for AppWinit {
    fn add<T: Drawable + 'static>(&mut self, item: T) -> SharedDrawable {
        let shared = self.panel.add(item);
        self.panel
            .get_button_manager(&mut self.button_manager, &InternalAccess(()));

        return shared;
    }

    fn remove_by_index(&mut self, index: u32) -> Result<(), &'static str> {
        self.panel.remove_by_index(index)
    }

    fn remove_item(&mut self, item: SharedDrawable) {
        self.panel.remove_item(item)
    }

    fn set_layout(&mut self, layout: Box<dyn Layout>) {
        self.panel.set_layout(layout);
    }
}

impl PanelControl for AppWinit {
    fn set_background(&mut self, color: u32) {
        self.panel.set_background(color);
    }
}
