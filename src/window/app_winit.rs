use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};
use wgpu::RenderPassColorAttachment;

use wgpu_glyph::{GlyphCruncher, Text};
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, MouseScrollDelta, TouchPhase, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};

use crate::window::component::base::area::Rect;
use crate::window::component::base::component_type::SharedDrawable;
use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::settings::Settings;
use crate::window::component::base::ui_command::UiCommand;
use crate::window::component::interface::component_control::{ComponentControl, PanelControl};
use crate::window::component::interface::drawable::{Drawable, InternalAccess};
use crate::window::component::interface::layout::Layout;
use crate::window::component::layout::layout_context::LayoutContext;
use crate::window::component::managers::animation_manager::AnimationManager;
use crate::window::component::managers::button_manager::ButtonManager;
use crate::window::component::managers::edit_label_manager::EditLabelManager;
use crate::window::component::managers::hover_manager::HoverManager;
use crate::window::component::managers::scroll_manager::ScrollManager;
use crate::window::component::managers::select_manager::SelectManager;
use crate::window::component::panel::Panel;
use crate::window::wgpu::draw_args::{DrawIndexedIndirectArgs, DrawIndirectArgs};
use crate::window::wgpu::screen_uniform::ScrollData;
use crate::window::wgpu::text_vertex::{TextVertex, push_glyph_to_vertices_raw};
use crate::window::wgpu::wgpu_state::WgpuState;

pub struct AppWinit {
    window: Option<Arc<Window>>,
    state: Option<WgpuState>,
    pub panel: Panel,
    cursor_position: (u16, u16),
    button_manager: ButtonManager,
    hover_manager: HoverManager,
    select_manager: SelectManager,
    edit_label_manager: EditLabelManager,
    animation_manager: AnimationManager,
    scroll_manager: ScrollManager,
    commands_rx: Receiver<UiCommand>,
    commands_tx: Sender<UiCommand>,
    next_redraw: Option<Instant>,
    gpu_ctx: GpuRenderContext,
    last_render: Instant,
}

impl Default for AppWinit {
    fn default() -> Self {
        let mut panel = Panel::default();

        let (tx, rx) = mpsc::channel();
        let mut settings = Settings::default();
        settings.command_tx = Some(tx.clone());
        panel.base.settings = settings;
        Self {
            window: Option::default(),
            state: Option::default(),
            panel: panel,
            button_manager: ButtonManager::default(),
            cursor_position: (u16::default(), u16::default()),
            hover_manager: HoverManager::default(),
            select_manager: SelectManager::default(),
            edit_label_manager: EditLabelManager::default(),
            animation_manager: AnimationManager::default(),
            scroll_manager: ScrollManager::default(),
            commands_tx: tx,
            commands_rx: rx,
            next_redraw: None,
            gpu_ctx: GpuRenderContext {
                shape_vertices: Vec::with_capacity(1024),
                shape_section_offsets: Vec::with_capacity(1024),
                shape_indices: Vec::with_capacity(1024),
                texts: Vec::with_capacity(100),
                text_storage: String::with_capacity(4096),
                offsets: Vec::with_capacity(1024),
                command_sections: Vec::with_capacity(1024),
            },
            last_render: Instant::now(),
        }
    }
}

impl AppWinit {
    fn update_layout(&mut self) {
        if let (Some(window), Some(state)) = (&self.window, &self.state) {
            let window_size = window.inner_size();

            let fonts = state.text_vertex.glyph_brush.fonts();

            let layout_context = LayoutContext { fonts: fonts };

            self.panel.resize(
                &Rect::new(0, 0, window_size.width as i16, window_size.height as i16),
                &layout_context,
                false,
            );
        }
    }
    pub fn process_commands(&mut self) {
        let mut needs_layout = false;
        let mut resize = false;

        while let Ok(cmd) = self.commands_rx.try_recv() {
            cmd.execute_command();
            self.execute_command(cmd, &mut needs_layout, &mut resize);
        }

        if needs_layout {
            if resize {
                self.update_layout();
            } // Пересчитываем геометрию ОДИН раз для всех изменений
            if let Some(w) = &self.window {
                w.request_redraw();
            }
        }
    }
    fn execute_command(&mut self, cmd: UiCommand, needs_layout: &mut bool, resize: &mut bool) {
        match cmd {
            UiCommand::Batch(commands) => {
                for c in commands {
                    self.execute_command(c, needs_layout, resize);
                }
            }
            UiCommand::ChangeColor(_, _) | UiCommand::RequestRedrawWithoutResize() => {
                *needs_layout = true;
            }
            UiCommand::SetScale(_, _)
            | UiCommand::SetText(_, _)
            | UiCommand::RequestRedraw()
            | UiCommand::Custom(_, _) => {
                *needs_layout = true;
                *resize = true
            }

            UiCommand::EditLabel(el) => {
                if let Some(el) = el {
                    self.edit_label_manager.set_edit_label(el);
                    *needs_layout = true;
                    *resize = true
                }
            }
            UiCommand::RequestRedrawWithTimer(time) => {
                let scheduled = std::time::Instant::now() + time;
                self.next_redraw = Some(match self.next_redraw {
                    Some(current) => current.min(scheduled),
                    None => scheduled,
                });
            }
            UiCommand::SetOnClick(el, _) => {
                if let Some(el) = el {
                    self.button_manager.add(el);
                }
            }
            UiCommand::SetOnMouseEnter(el, _) => {
                if let Some(el) = el {
                    self.hover_manager.add(el);
                }
            }
            UiCommand::SetOnMouseLeave(el, _) => {
                if let Some(el) = el {
                    self.hover_manager.add(el);
                }
            }
            UiCommand::StartAnimation(el) => {
                if let Some(el) = el {
                    self.animation_manager.start(el);
                }
            }
            _ => (),
        }
    }

    pub fn get_tx(&self) -> Sender<UiCommand> {
        self.commands_tx.clone()
    }

    fn handle_key(&mut self, event: KeyEvent) {
        let mut needs_layout = false;

        self.edit_label_manager.handle_key(event, &mut needs_layout);

        if needs_layout {
            self.update_layout();
            if let Some(w) = &self.window {
                w.request_redraw();
            }
        }
    }
}

impl ApplicationHandler for AppWinit {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("LOL");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY, // Или DX12
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

        let mut required_features = wgpu::Features::empty();
        if adapter
            .features()
            .contains(wgpu::Features::MULTI_DRAW_INDIRECT)
        {
            required_features |= wgpu::Features::MULTI_DRAW_INDIRECT;
            println!("MultiDrawIndirect — on");
        } else {
            println!("MultiDrawIndirect — off");
        }

        // Создаем логическое устройство и очередь команд
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("My Device"),
            required_features,
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            ..Default::default()
        }))
        .expect("Не удалось создать устройство wgpu");

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
                if let Some(state) = self.state.as_mut() {
                    // Обновляем конфиг wgpu (чтобы не было растягивания картинки и утечек)
                    state.config.width = size.width.max(1);
                    state.config.height = size.height.max(1);
                    state.surface.configure(&state.device, &state.config);

                    // Обновляем размеры корневой панели
                    let width = size.width as u16;
                    let height = size.height as u16;

                    self.panel.set_width(width);
                    self.panel.set_height(height);

                    let fonts = state.text_vertex.glyph_brush.fonts();

                    let layout_context = LayoutContext { fonts: fonts };

                    self.panel.resize(
                        &Rect::new(0, 0, width as i16, height as i16),
                        &layout_context,
                        false,
                    );

                    let screen_uniform = [size.width as f32, size.height as f32, 0.0, 0.0]; // +padding
                    state.queue.write_buffer(
                        &state.uniform_buffer,
                        0,
                        bytemuck::cast_slice(&screen_uniform),
                    );

                    state.resize(size);

                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();

                if now.duration_since(self.last_render).as_millis() < 8 {
                    return;
                }
                self.last_render = now;

                let state = self.state.as_mut().unwrap();

                self.gpu_ctx.clear();

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

                self.panel
                    .print(&mut self.gpu_ctx, &self.panel.base.rect, (0.0, 0.0), 1);

                if !self.gpu_ctx.offsets.is_empty() {
                    let gpu_scrolls: Vec<ScrollData> = self
                        .gpu_ctx
                        .offsets
                        .iter()
                        .map(|(x, y)| ScrollData {
                            offsets: [*x, *y, 0.0, 0.0],
                        })
                        .collect();

                    if !gpu_scrolls.is_empty() {
                        state.queue.write_buffer(
                            &state.scroll_storage_buffer,
                            0,
                            bytemuck::cast_slice(&gpu_scrolls),
                        );
                    }
                }

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
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &state.depth_stencil_view, // Твоя вьюшка, которую ты обновляешь в resize
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0), // Просто очистка, даже если не используем
                                store: wgpu::StoreOp::Discard,
                            }),
                            stencil_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(0), // Очищаем трафарет в 0 каждый кадр
                                store: wgpu::StoreOp::Store, // Сохраняем результат для тестов в этом кадре
                            }),
                        }),
                        ..Default::default()
                    });

                    if !self.gpu_ctx.shape_vertices.is_empty() {
                        // state.queue.write_buffer(
                        //     &state.vertex_buffer,
                        //     0,
                        //     bytemuck::cast_slice(&self.gpu_ctx.shape_vertices),
                        // );

                        // render_pass.set_pipeline(&state.panel_pipeline);
                        // render_pass.set_bind_group(0, &state.bind_group, &[]);
                        // render_pass.set_vertex_buffer(0, state.vertex_buffer.slice(..));

                        // render_pass.draw(0..self.gpu_ctx.shape_vertices.len() as u32, 0..1);

                        state.render_shape(&self.gpu_ctx);
                    }

                    if !self.gpu_ctx.texts.is_empty() {
                        state.render_text(&self.gpu_ctx, self.last_render);
                    }

                    state.render(&self.gpu_ctx, &mut render_pass);
                }

                // Отправляем записанные команды на выполнение в видеокарту
                state.queue.submit(std::iter::once(encoder.finish()));
                output.present();

                self.gpu_ctx.clear();

                // Если надо включить постоянный режим отрисовки
                // if self.edit_label_manager.is_editing() {
                //     self.next_redraw = Some(Instant::now() + Duration::from_millis(500));
                // } else {
                //     self.next_redraw = None;
                // }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = (position.x as u16, position.y as u16);

                self.hover_manager
                    .hover(self.cursor_position.0, self.cursor_position.1);

                if let Some(state) = self.state.as_mut() {
                    let fonts = state.text_vertex.glyph_brush.fonts();

                    let layout_context = LayoutContext { fonts: fonts };
                    if self.select_manager.select(
                        self.cursor_position.0,
                        self.cursor_position.1,
                        &layout_context,
                    ) {
                        self.update_layout();
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left
                    && state == winit::event::ElementState::Pressed
                {
                    let (mx, my) = self.cursor_position;

                    if let Some(state) = self.state.as_ref() {
                        let fonts = state.text_vertex.glyph_brush.fonts();

                        let layout_context = LayoutContext { fonts: fonts };
                        self.select_manager.select_start(
                            self.cursor_position.0,
                            self.cursor_position.1,
                            &layout_context,
                        );
                    }
                    self.edit_label_manager.stop_edit();

                    self.button_manager.click(mx as u16, my as u16);
                } else if button == winit::event::MouseButton::Left
                    && state == winit::event::ElementState::Released
                {
                    self.select_manager.stop_select();
                }
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                let scroll_amount = match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        // y: 1.0 — вверх, -1.0 — вниз.
                        y * 10.5
                    }

                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };

                // Если прокрутка активна (или тачпад в процессе движения)
                if phase == TouchPhase::Moved || phase == TouchPhase::Started {
                    let (mx, my) = self.cursor_position;
                    if self.scroll_manager.scroll(mx, my, 0.0, scroll_amount) {
                        self.update_layout();
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key(event);
            }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let changed = self.animation_manager.update(&self.commands_tx);

        self.process_commands();

        if let Some(state) = self.state.as_mut() {
            if !changed && state.text_vertex.last_defrag_time.elapsed() > Duration::from_secs(10) {
                if state.is_defrag_worth_it() {
                    state.perform_true_defragmentation();
                    state.text_vertex.last_defrag_time = Instant::now();

                    // После дефрагментации нужно один раз перерисовать,
                    // так как Indirect Buffer обновился
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
            }
        }

        let next_event = self.animation_manager.query_next_timeout();

        if changed {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }

        match (next_event, self.next_redraw) {
            (Some(anim_time), Some(redraw_time)) => {
                event_loop.set_control_flow(ControlFlow::WaitUntil(anim_time.min(redraw_time)));
            }
            (Some(time), None) | (None, Some(time)) => {
                event_loop.set_control_flow(ControlFlow::WaitUntil(time));
            }
            (None, None) => {
                event_loop.set_control_flow(ControlFlow::Wait);
            }
        }
    }
}

impl ComponentControl for AppWinit {
    fn add<T: Drawable + 'static>(&mut self, item: T) -> SharedDrawable {
        let shared = self.panel.add(item);

        let weak_self = Rc::downgrade(&shared);

        shared.borrow_mut().as_base_mut().self_ref = Some(weak_self);

        shared
            .borrow_mut()
            .set_default_settings(&self.panel.base.settings);

        self.panel.get_managers(
            &mut self.button_manager,
            &mut self.hover_manager,
            &mut self.select_manager,
            &mut self.scroll_manager,
            &InternalAccess(()),
        );

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
