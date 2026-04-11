use std::{rc::Rc, time::Duration};

use crate::window::component::{
    animation::animation_action::AnimationSequence, base::component_type::SharedDrawable,
};

#[derive(Clone)]
pub enum UiCommand<T = ()> {
    ChangeColor(Option<SharedDrawable>, u32),
    SetText(Option<SharedDrawable>, String),
    Custom(Option<SharedDrawable>, Rc<dyn Fn(SharedDrawable)>),
    SetScale(Option<SharedDrawable>, u16),
    EditLabel(Option<SharedDrawable>),
    RequestRedraw(),
    RequestRedrawWithoutResize(),
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
    Other(T),
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
    pub fn execute_command(&self) {
        match self {
            UiCommand::Batch(commands) => {
                for c in commands {
                    c.execute_command();
                }
            }
            UiCommand::ChangeColor(el, color) => {
                if let Some(el) = el {
                    if let Some(panel) = el.borrow_mut().as_panel_control_mut() {
                        panel.set_background(*color);
                    }
                }
            }
            UiCommand::SetScale(el, scale) => {
                if let Some(el) = el {
                    let mut e = el.borrow_mut();
                    if let Some(ctrl) = e.as_label_control_mut() {
                        ctrl.set_scale(*scale);
                    }
                }
            }
            UiCommand::SetText(el, text) => {
                if let Some(el) = el {
                    let mut e = el.borrow_mut();
                    if let Some(ctrl) = e.as_label_control_mut() {
                        let text = text.clone();
                        ctrl.set_text(text);
                    }
                }
            }
            UiCommand::SetOnClick(el, command) => {
                if let Some(el) = el {
                    if let Some(clickable) = el.borrow_mut().as_clickable() {
                        let cmd = *command.clone();
                        clickable.set_on_click(cmd);
                    }
                }
            }
            UiCommand::SetOnMouseEnter(el, command) => {
                if let Some(el) = el {
                    if let Some(hovearable) = el.borrow_mut().as_hoverable() {
                        let cmd = *command.clone();
                        hovearable.set_on_mouse_enter(cmd);
                    }
                }
            }
            UiCommand::SetOnMouseLeave(el, command) => {
                if let Some(el) = el {
                    if let Some(hovearable) = el.borrow_mut().as_hoverable() {
                        let cmd = *command.clone();
                        hovearable.set_on_mouse_leave(cmd);
                    }
                }
            }
            UiCommand::SetAnimation(el, animation) => {
                if let Some(el) = el {
                    if let Some(with_animation) = el.borrow_mut().as_with_animation() {
                        let anim = (*(*animation).clone()).clone();
                        with_animation.set_animation(anim);
                    }
                }
            }
            UiCommand::AddAnimation(el, animation) => {
                if let Some(el) = el {
                    if let Some(with_animation) = el.borrow_mut().as_with_animation() {
                        let anim = (*(*animation).clone()).clone();
                        with_animation.add_animation(anim);
                    }
                }
            }
            UiCommand::AddAnimationBatch(el, animations) => {
                if let Some(el) = el {
                    if let Some(with_animation) = el.borrow_mut().as_with_animation() {
                        let anim = (*(*animations).clone()).clone();
                        with_animation.add_animation_batch(anim);
                    }
                }
            }
            UiCommand::Custom(el, action) => {
                if let Some(el) = el {
                    (action)(el.clone());
                }
            }
            _ => (),
        }
    }
}
