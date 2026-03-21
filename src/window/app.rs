use crate::window::component::base::component_type::SharedDrawable;
use crate::window::component::interface::component_control::ComponentControl;
use crate::window::component::interface::drawable::Drawable;
use crate::window::component::interface::layout::Layout;
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

    pub fn run(&mut self) {
        if !self.app.is_some() {
            self.app = Some(AppWinit::default());
        }

        let mut event_loop = self.event_loop.take().expect("Event loop already taken");
        let app = self.app.as_mut().unwrap();
        let _ = event_loop.run_app(app);
    }
}

impl ComponentControl for App {
    fn add<T: Drawable + 'static>(&mut self, item: T) -> SharedDrawable {
        if self.app.is_some() {
            self.app.as_mut().unwrap().add(item)
        } else {
            self.app = Some(AppWinit::default());
            self.app.as_mut().unwrap().add(item)
        }
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
        if self.app.is_some() {
            self.app.as_mut().unwrap().set_layout(layout);
        } else {
            self.app = Some(AppWinit::default());
            self.app.as_mut().unwrap().set_layout(layout);
        }
    }
}

impl PanelControl for App {
    fn set_background(&mut self, color: u32) {
        if self.app.is_some() {
            self.app.as_mut().unwrap().set_background(color);
        } else {
            self.app = Some(AppWinit::default());
            self.app.as_mut().unwrap().set_background(color);
        }
    }
}
