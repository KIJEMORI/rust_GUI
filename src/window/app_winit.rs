use crate::window::component::base::area::Area;
use crate::window::component::base::component_type::SharedDrawable;
use crate::window::component::base::gpu_render_context::GpuRenderContext;
use crate::window::component::base::settings::Settings;
use crate::window::component::base::ui_command::{CommandTrait, UiCommand};
use crate::window::component::interface::component_control::ComponentControl;
use crate::window::component::interface::drawable::InternalAccess;
use crate::window::component::interface::layout::Layout;
use crate::window::component::layout::layout_context::LayoutContext;
use crate::window::component::managers::animation_manager::AnimationManager;
use crate::window::component::managers::button_manager::ButtonManager;
use crate::window::component::managers::drag_manager::DragManager;
use crate::window::component::managers::edit_label_manager::EditLabelManager;
use crate::window::component::managers::hover_manager::HoverManager;
use crate::window::component::managers::id_manager::{IDManager, get_upgrade_by_id};
use crate::window::component::managers::scroll_manager::ScrollManager;
use crate::window::component::managers::select_manager::SelectManager;
use crate::window::component::panel::Panel;
use crate::window::wgpu::wgpu_state::WgpuState;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};
use wgpu::RenderPassColorAttachment;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, Modifiers, MouseScrollDelta, TouchPhase, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};

pub struct AppWinit {
    window: Option<Arc<Window>>,
    state: Option<WgpuState>,
    pub panel: SharedDrawable,
    cursor_position: (u16, u16),
    button_manager: ButtonManager,
    hover_manager: HoverManager,
    select_manager: SelectManager,
    edit_label_manager: EditLabelManager,
    animation_manager: AnimationManager,
    scroll_manager: ScrollManager,
    drag_manager: DragManager,
    id_manager: IDManager,
    commands_rx: Receiver<UiCommand>,
    commands_tx: Sender<UiCommand>,
    next_redraw: Option<Instant>,
    gpu_ctx: GpuRenderContext,
    last_render: Instant,
    modifiers: Modifiers,
}

impl Default for AppWinit {
    fn default() -> Self {
        let mut panel = Panel::default();

        let (tx, rx) = mpsc::channel();
        let mut settings = Settings::default();
        settings.command_tx = Some(tx.clone());
        panel.base.settings = settings;

        let panel: SharedDrawable = Rc::new(RefCell::new(panel));

        let mut id_manager = IDManager::default();

        id_manager.register(Rc::clone(&panel));

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
            drag_manager: DragManager::default(),
            id_manager: id_manager,
            commands_tx: tx,
            commands_rx: rx,
            next_redraw: None,
            gpu_ctx: GpuRenderContext::new(),
            last_render: Instant::now(),
            modifiers: Modifiers::default(),
        }
    }
}

impl AppWinit {
    fn update_layout(&mut self) {
        if let (Some(window), Some(state)) = (&self.window, &mut self.state) {
            let now = Instant::now();

            let window_size = window.inner_size();

            let width = window_size.width as u16;
            let height = window_size.height as u16;

            self.panel
                .borrow_mut()
                .as_panel_control_mut()
                .set_width(width)
                .set_height(height);

            let layout_context = LayoutContext {
                font: &state.text_vertex.atlas.font,
                sdf_base_size: 64.0,
            };

            self.panel.borrow_mut().resize(
                &Area::new(0.0, 0.0, width, height),
                &layout_context,
                false,
            );

            let screen_uniform = [
                window_size.width as f32,
                window_size.height as f32,
                0.0,
                0.0,
            ]; // +padding
            state.queue.write_buffer(
                &state.uniform_buffer,
                0,
                bytemuck::cast_slice(&screen_uniform),
            );

            let duration = now.elapsed(); // Получаем длительность
            println!("Время Пересчёта: {:?}", duration);
        }
    }
    fn print(&mut self) {
        let now = Instant::now();

        if now.duration_since(self.last_render).as_millis() < 8 {
            return;
        }
        self.last_render = now;

        let state = self.state.as_mut().unwrap();

        self.gpu_ctx.clear();

        let rect = &self.panel.borrow().as_base().rect.clone();

        self.panel
            .borrow_mut()
            .print(&mut self.gpu_ctx, rect, 1, 0, &mut state.text_vertex.atlas);

        state.prepare_gpu_data(&mut self.gpu_ctx);

        let output = state.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Создаем "записчик" команд

        state.render(&self.gpu_ctx, &view);

        let duration = now.elapsed(); // Получаем длительность
        println!("Время кадра: {:?}", duration);

        // Отправляем записанные команды на выполнение в видеокарту

        output.present();

        self.gpu_ctx.clear();

        // Если надо включить постоянный режим отрисовки
        // if self.edit_label_manager.is_editing() {
        //     self.next_redraw = Some(Instant::now() + Duration::from_millis(500));
        // } else {
        //     self.next_redraw = None;
        // }

        // let duration = now.elapsed(); // Получаем длительность
        // println!("Время кадра: {:?}", duration);
    }

    pub fn process_commands(&mut self) {
        let mut needs_layout = false;
        let mut resize = false;

        while let Ok(cmd) = self.commands_rx.try_recv() {
            cmd.execute_command(&self.id_manager);
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
            | UiCommand::Custom(_, _)
            | UiCommand::ScrollPanel(_, _, _)
            | UiCommand::SetPosition(_, _, _)
            | UiCommand::Other(_) => {
                *needs_layout = true;
            }

            UiCommand::EditLabel(el) => {
                let el = el.get();
                if let Some(el) = el {
                    if let Some(state) = &self.state {
                        let layout_context = LayoutContext {
                            font: &state.text_vertex.atlas.font,
                            sdf_base_size: 64.0,
                        };
                        self.edit_label_manager.set_edit_label(
                            &el,
                            &self.id_manager,
                            &layout_context,
                        );
                        *needs_layout = true;
                    }
                }
            }
            UiCommand::RequestRedrawWithTimer(time) => {
                let scheduled = std::time::Instant::now() + time;
                self.next_redraw = Some(match self.next_redraw {
                    Some(current) => current.min(scheduled),
                    None => scheduled,
                });
            }
            UiCommand::SetOnClick(id, _) => {
                let id = &id.get();
                if let Some(el) = get_upgrade_by_id(id, &self.id_manager) {
                    self.button_manager.add(el.borrow().as_base().id);
                }
            }
            UiCommand::SetOnMouseEnter(id, _) => {
                let id = &id.get();
                if let Some(id) = id {
                    self.hover_manager.add(*id);
                }
            }
            UiCommand::SetOnMouseLeave(id, _) => {
                let id = &id.get();
                if let Some(id) = id {
                    self.hover_manager.add(*id);
                }
            }
            UiCommand::StartAnimation(id) => {
                let id = &id.get();
                if let Some(id) = id {
                    self.animation_manager.start(&id, &self.id_manager);
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

        if let Some(state) = self.state.as_ref() {
            let layout_context = LayoutContext {
                font: &state.text_vertex.atlas.font,
                sdf_base_size: 64.0,
            };

            self.edit_label_manager.handle_key(
                event,
                &mut needs_layout,
                &self.id_manager,
                &layout_context,
            );

            if needs_layout {
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
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
        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        let caps = surface.get_capabilities(&adapter);
        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else if caps.present_modes.contains(&wgpu::PresentMode::Immediate) {
            wgpu::PresentMode::Immediate
        } else {
            wgpu::PresentMode::Fifo // Дефолтный вариант
        };

        //let present_mode = wgpu::PresentMode::Fifo;

        config.present_mode = present_mode;

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
                    state.resize(size);

                    self.update_layout();
                    self.print();
                }
            }
            WindowEvent::RedrawRequested => {
                self.print();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = (position.x as u16, position.y as u16);

                let mut need_redraw = false;

                self.hover_manager.hover(
                    self.cursor_position.0,
                    self.cursor_position.1,
                    &self.id_manager,
                );

                if let Some(state) = self.state.as_mut() {
                    let layout_context = LayoutContext {
                        font: &state.text_vertex.atlas.font,
                        sdf_base_size: 64.0,
                    };
                    self.select_manager.select(
                        self.cursor_position.0,
                        self.cursor_position.1,
                        &layout_context,
                        &self.id_manager,
                    );
                    need_redraw = self.select_manager.in_run();
                }

                if !need_redraw {
                    let (mx, my) = self.cursor_position;

                    need_redraw = self.drag_manager.drag(mx, my, &self.id_manager);
                }

                if need_redraw {
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left
                    && state == winit::event::ElementState::Pressed
                {
                    let (mx, my) = self.cursor_position;

                    if let Some(state) = self.state.as_ref() {
                        let layout_context = LayoutContext {
                            font: &state.text_vertex.atlas.font,
                            sdf_base_size: 64.0,
                        };
                        self.select_manager.select_start(
                            self.cursor_position.0,
                            self.cursor_position.1,
                            &layout_context,
                            &self.id_manager,
                        );
                    }
                    self.edit_label_manager.stop_edit(&self.id_manager);

                    self.drag_manager.drag_start(mx, my, &self.id_manager);

                    self.button_manager
                        .click(mx as u16, my as u16, &self.id_manager);
                } else if button == winit::event::MouseButton::Left
                    && state == winit::event::ElementState::Released
                {
                    self.select_manager.stop_select();
                    self.drag_manager.stop_drag(&self.id_manager);
                }
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                let (scroll_amount_x, scroll_amount_y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        let (mut dx, mut dy) = (x * 20.0, y * 20.0);

                        if self.modifiers.state().shift_key() && dy != 0.0 && dx == 0.0 {
                            dx = dy;
                            dy = 0.0;
                        }
                        (dx, dy)
                    }

                    MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };

                // Если прокрутка активна (или тачпад в процессе движения)
                if phase == TouchPhase::Moved || phase == TouchPhase::Started {
                    let (mx, my) = self.cursor_position;
                    if self.scroll_manager.scroll(
                        mx,
                        my,
                        scroll_amount_x,
                        scroll_amount_y,
                        &self.id_manager,
                    ) {
                        // if let Some(state) = self.state.as_mut() {
                        //     state.text_vertex.section_hashes.fill(0);
                        // }
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key(event);
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = new_modifiers;
            }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let changed = self
            .animation_manager
            .update(&self.commands_tx, &self.id_manager);

        self.process_commands();

        if let Some(state) = self.state.as_mut() {
            if !changed && state.text_vertex.last_defrag_time.elapsed() > Duration::from_secs(10) {
                // if state.is_defrag_worth_it() {
                //     state.perform_true_defragmentation();
                //     state.text_vertex.last_defrag_time = Instant::now();

                //     // После дефрагментации нужно один раз перерисовать,
                //     // так как Indirect Buffer обновился
                //     if let Some(window) = self.window.as_ref() {
                //         window.request_redraw();
                //     }
                // }
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
    fn add_drawable(&mut self, item: SharedDrawable) -> SharedDrawable {
        let shared = self
            .panel
            .borrow_mut()
            .as_component_control_mut()
            .unwrap()
            .add_drawable(item);

        let weak_self = Rc::downgrade(&shared);

        shared.borrow_mut().as_base_mut().self_ref = Some(weak_self);

        shared
            .borrow_mut()
            .set_default_settings(&self.panel.borrow().as_base().settings);

        self.panel.borrow().get_managers(
            &mut self.button_manager,
            &mut self.hover_manager,
            &mut self.select_manager,
            &mut self.scroll_manager,
            &mut self.drag_manager,
            &mut self.id_manager,
            &InternalAccess(()),
        );

        return shared;
    }

    fn remove_by_index(&mut self, index: u32) -> Result<(), &'static str> {
        self.panel
            .borrow_mut()
            .as_component_control_mut()
            .unwrap()
            .remove_by_index(index)
    }

    fn remove_item(&mut self, item: SharedDrawable) {
        self.panel
            .borrow_mut()
            .as_component_control_mut()
            .unwrap()
            .remove_item(item)
    }

    fn set_layout(&mut self, layout: Box<dyn Layout>) {
        self.panel
            .borrow_mut()
            .as_component_control_mut()
            .unwrap()
            .set_layout(layout);
    }
}
