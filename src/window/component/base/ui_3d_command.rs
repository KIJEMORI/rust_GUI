use std::{cell::Cell, rc::Rc};

use crate::window::component::{
    base::ui_command::{CommandTrait, UiCommand},
    managers::id_manager::get_upgrade_by_id,
};

#[derive(Clone)]
pub enum Ui3DCommand {
    RotateCamera(Cell<Option<u32>>, Cell<f32>, Cell<f32>),
    ChangeDistanceCamera(Cell<Option<u32>>, Cell<f32>, Cell<f32>),
}

impl Ui3DCommand {
    pub fn build(self) -> UiCommand {
        let cmd: Rc<dyn CommandTrait> = Rc::new(self);
        UiCommand::Other(cmd)
    }
    pub fn rotate_camera() -> UiCommand {
        Ui3DCommand::RotateCamera(Cell::new(None), Cell::new(0.0), Cell::new(0.0)).build()
    }
    pub fn change_distance_camera() -> UiCommand {
        Ui3DCommand::ChangeDistanceCamera(Cell::new(None), Cell::new(0.0), Cell::new(0.0)).build()
    }
}

impl CommandTrait for Ui3DCommand {
    fn fill_coord(&self, mx: f32, my: f32) {
        match self {
            Ui3DCommand::RotateCamera(_, x, y) | Ui3DCommand::ChangeDistanceCamera(_, x, y) => {
                x.set(mx);
                y.set(my);
            }
            _ => (),
        }
    }
    fn fill_ref(&self, item: &u32) {
        match self {
            Ui3DCommand::RotateCamera(target, _, _)
            | Ui3DCommand::ChangeDistanceCamera(target, _, _) => {
                if target.get().is_none() {
                    target.set(Some(*item));
                }
            }
            _ => (),
        }
    }
    fn execute_command(
        &self,
        id_manager: &crate::window::component::managers::id_manager::IDManager,
    ) {
        match self {
            Ui3DCommand::RotateCamera(id, _, _) | Ui3DCommand::ChangeDistanceCamera(id, _, _) => {
                let id = &id.get();
                if let Some(el) = get_upgrade_by_id(id, id_manager) {
                    match self {
                        Ui3DCommand::RotateCamera(_, mx, my) => {
                            if let Some(viewport) = el.borrow_mut().as_viewport_control_mut() {
                                viewport.rotate_camera(mx.get(), my.get());
                            }
                        }
                        Ui3DCommand::ChangeDistanceCamera(_, x, y) => {
                            if let Some(viewport) = el.borrow_mut().as_viewport_control_mut() {
                                viewport.change_distance_camera(x.get(), y.get());
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }
}
