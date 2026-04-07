use std::rc::Rc;

use crate::window::component::base::component_type::SharedDrawable;

#[derive(Clone)]
pub enum UiCommand {
    ChangeColor(Option<SharedDrawable>, u32),
    SetText(Option<SharedDrawable>, String),
    Custom(Rc<dyn Fn()>),
    SetScale(Option<SharedDrawable>, u16),
    EditLabel(Option<SharedDrawable>),
    RequestRedraw(),
    Batch(Vec<UiCommand>),
}

impl UiCommand {
    pub fn fill_ref(&mut self, item: &SharedDrawable) {
        match self {
            UiCommand::Batch(cmds) => {
                for c in cmds {
                    c.fill_ref(item);
                }
            }
            UiCommand::ChangeColor(target, _)
            | UiCommand::SetText(target, _)
            | UiCommand::SetScale(target, _)
            | UiCommand::EditLabel(target) => {
                if target.is_none() {
                    *target = Some(Rc::clone(item));
                }
            }
            UiCommand::Custom(..) => (),
            _ => (),
        }
    }
}
