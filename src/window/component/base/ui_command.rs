use std::{option, rc::Rc, time::Duration};

use crate::window::component::{
    animation::animation_action::AnimationSequence,
    base::component_type::SharedDrawable,
    managers::id_manager::{IDManager, get_upgrade_by_id},
};

#[derive(Clone)]
pub enum UiCommand<T = ()> {
    ChangeColor(Option<u32>, u32),
    SetText(Option<u32>, String),
    Custom(Option<u32>, Rc<dyn Fn(SharedDrawable)>),
    SetScale(Option<u32>, u16),
    EditLabel(Option<u32>),
    ScrollPanel(Option<u32>, f32, f32),
    RequestRedraw(),
    RequestRedrawWithoutResize(),
    RequestRedrawWithTimer(Duration),
    SetOnClick(Option<u32>, Box<UiCommand>),
    SetOnMouseEnter(Option<u32>, Box<UiCommand>),
    SetOnMouseLeave(Option<u32>, Box<UiCommand>),
    SetAnimation(Option<u32>, Rc<Vec<AnimationSequence>>),
    AddAnimation(Option<u32>, Rc<AnimationSequence>),
    AddAnimationBatch(Option<u32>, Rc<Vec<AnimationSequence>>),
    StartAnimation(Option<u32>),
    None(),
    Batch(Vec<UiCommand>),
    Other(T),
    DragItem(Option<u32>),
}

impl UiCommand {
    pub fn fill_coord(&mut self, mx: f32, my: f32) {
        match self {
            UiCommand::ScrollPanel(_, x, y) => {
                *x = -mx * x.clone();
                *y = -my * y.clone();
            }
            _ => (),
        }
    }
    pub fn fill_ref(&mut self, item: &u32) {
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
            | UiCommand::Custom(target, _)
            | UiCommand::DragItem(target)
            | UiCommand::ScrollPanel(target, _, _) => {
                if target.is_none() {
                    *target = Some(item.clone());
                }
            }
            _ => (),
        }
    }
    pub fn execute_command(&self, id_manager: &IDManager) {
        match self {
            UiCommand::Batch(commands) => {
                for c in commands {
                    c.execute_command(id_manager);
                }
            }
            UiCommand::ChangeColor(id, color) => {
                if let Some(el) = get_upgrade_by_id(id, id_manager) {
                    el.borrow_mut()
                        .as_panel_control_mut()
                        .set_background(*color);
                }
            }
            UiCommand::SetScale(id, scale) => {
                if let Some(el) = get_upgrade_by_id(id, id_manager) {
                    let mut e = el.borrow_mut();
                    if let Some(ctrl) = e.as_label_control_mut() {
                        ctrl.set_scale(*scale);
                    }
                }
            }
            UiCommand::SetText(id, text) => {
                if let Some(el) = get_upgrade_by_id(id, id_manager) {
                    let mut e = el.borrow_mut();
                    if let Some(ctrl) = e.as_label_control_mut() {
                        let text = text.clone();
                        ctrl.set_text(text);
                    }
                }
            }
            UiCommand::SetOnClick(id, command) => {
                if let Some(el) = get_upgrade_by_id(id, id_manager) {
                    if let Some(clickable) = el.borrow_mut().as_clickable_mut() {
                        let cmd = *command.clone();
                        clickable.set_on_click(cmd);
                    }
                }
            }
            UiCommand::SetOnMouseEnter(id, command) => {
                if let Some(el) = get_upgrade_by_id(id, id_manager) {
                    if let Some(hovearable) = el.borrow_mut().as_hoverable_mut() {
                        let cmd = *command.clone();
                        hovearable.set_on_mouse_enter(cmd);
                    }
                }
            }
            UiCommand::SetOnMouseLeave(id, command) => {
                if let Some(el) = get_upgrade_by_id(id, id_manager) {
                    if let Some(hovearable) = el.borrow_mut().as_hoverable_mut() {
                        let cmd = *command.clone();
                        hovearable.set_on_mouse_leave(cmd);
                    }
                }
            }
            UiCommand::SetAnimation(id, animation) => {
                if let Some(el) = get_upgrade_by_id(id, id_manager) {
                    if let Some(with_animation) = el.borrow_mut().as_with_animation_mut() {
                        let anim = (*(*animation).clone()).clone();
                        with_animation.set_animation(anim);
                    }
                }
            }
            UiCommand::AddAnimation(id, animation) => {
                if let Some(el) = get_upgrade_by_id(id, id_manager) {
                    if let Some(with_animation) = el.borrow_mut().as_with_animation_mut() {
                        let anim = (*(*animation).clone()).clone();
                        with_animation.add_animation(anim);
                    }
                }
            }
            UiCommand::AddAnimationBatch(id, animations) => {
                if let Some(el) = get_upgrade_by_id(id, id_manager) {
                    if let Some(with_animation) = el.borrow_mut().as_with_animation_mut() {
                        let anim = (*(*animations).clone()).clone();
                        with_animation.add_animation_batch(anim);
                    }
                }
            }
            UiCommand::Custom(id, action) => {
                if let Some(el) = get_upgrade_by_id(id, id_manager) {
                    (action)(el.clone());
                }
            }
            UiCommand::ScrollPanel(id, mx, my) => {
                if let Some(el) = get_upgrade_by_id(id, id_manager) {
                    if let Some(scrollable) = el.borrow_mut().as_scrollable_mut() {
                        let mx = mx.clone();
                        let my = my.clone();

                        scrollable.scroll(mx, my);
                    }
                }
            }
            _ => (),
        }
    }
}
