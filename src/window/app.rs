use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use crate::window::component::base::component_type::SharedDrawable;
use crate::window::component::base::ui_command::UiCommand;
use crate::window::component::interface::component_control::{
    ComponentControl, ComponentControlExt,
};
use crate::window::component::interface::drawable::Drawable;
use crate::window::component::interface::layout::Layout;
use crate::window::component::panel::Panel;
use crate::window::{app_winit::AppWinit, component::interface::component_control::PanelControl};
use winit::event_loop::{ControlFlow, EventLoop};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub struct App {
    event_loop: Option<EventLoop<()>>,
    app: Option<AppWinit>,
}

impl App {
    pub fn new() -> App {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Wait);
        App {
            event_loop: Some(event_loop),
            app: None,
        }
    }

    pub fn as_drawable(&mut self) -> &SharedDrawable {
        if self.app.is_none() {
            self.app = Some(AppWinit::default());
        }

        return &self.app.as_ref().unwrap().panel;
    }

    pub fn run(&mut self) {
        if self.app.is_none() {
            self.app = Some(AppWinit::default());
        }

        let event_loop = self.event_loop.take().expect("Event loop already taken");
        let app = self.app.as_mut().unwrap();
        let _ = event_loop.run_app(app);
    }

    pub fn get_tx(&mut self) -> Sender<UiCommand> {
        if !self.app.is_none() {
            self.app = Some(AppWinit::default());
        }
        self.app.as_ref().unwrap().get_tx()
    }
}

impl ComponentControl for App {
    fn add_drawable(&mut self, item: SharedDrawable) -> SharedDrawable {
        if self.app.is_none() {
            self.app = Some(AppWinit::default());
        }
        self.app.as_mut().unwrap().add_drawable(item)
    }

    fn remove_by_index(&mut self, index: u32) -> Result<(), &'static str> {
        if self.app.is_some() {
            self.app.as_mut().unwrap().remove_by_index(index)
        } else {
            Err("Window is Empty")
        }
    }

    fn remove_item(&mut self, item: SharedDrawable) {
        if self.app.is_some() {
            self.app.as_mut().unwrap().remove_item(item)
        } else {
            //Err("Window is Empty")
        }
    }

    fn set_layout(&mut self, layout: Box<dyn Layout>) {
        if self.app.is_none() {
            self.app = Some(AppWinit::default());
        }
        self.app.as_mut().unwrap().set_layout(layout);
    }
}

impl ComponentControlExt for App {
    fn add<T: Drawable + 'static>(&mut self, item: T) -> SharedDrawable {
        if self.app.is_none() {
            self.app = Some(AppWinit::default());
        }

        let drawable = Rc::new(RefCell::new(item));
        self.app.as_mut().unwrap().add_drawable(drawable)
    }
}
