use std::{rc::Rc, time::Duration};

use crate::window::component::{
    animation::animation_action::AnimationSequence, base::component_type::SharedDrawable,
};

#[derive(Clone)]
pub enum UiCommand {
    ChangeColor(Option<SharedDrawable>, u32),
    SetText(Option<SharedDrawable>, String),
    Custom(Option<SharedDrawable>, Rc<dyn Fn(SharedDrawable)>),
    SetScale(Option<SharedDrawable>, u16),
    EditLabel(Option<SharedDrawable>),
    RequestRedraw(),
    RequestRedrawWithTimer(Duration),
    SetOnClick(Option<SharedDrawable>, Box<UiCommand>),
    SetOnMouseEnter(Option<SharedDrawable>, Box<UiCommand>),
    SetOnMouseLeave(Option<SharedDrawable>, Box<UiCommand>),
    SetAnimation(Option<SharedDrawable>, Rc<Vec<AnimationSequence>>),
    AddAnimation(Option<SharedDrawable>, Rc<AnimationSequence>),
    AddAnimationBatch(Option<SharedDrawable>, Rc<Vec<AnimationSequence>>),
    StartAnimation(Option<SharedDrawable>),
    None(),
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
            | UiCommand::EditLabel(target)
            | UiCommand::SetOnClick(target, _)
            | UiCommand::SetOnMouseEnter(target, _)
            | UiCommand::SetOnMouseLeave(target, _)
            | UiCommand::SetAnimation(target, _)
            | UiCommand::StartAnimation(target)
            | UiCommand::Custom(target, _) => {
                if target.is_none() {
                    *target = Some(Rc::clone(item));
                }
            }
            _ => (),
        }
    }
}
