use std::{cell::Cell, ops::DerefMut, option, rc::Rc, time::Duration};

use crate::window::component::{
    animation::animation_action::AnimationSequence,
    base::component_type::SharedDrawable,
    managers::id_manager::{IDManager, get_upgrade_by_id},
};

pub trait CommandTrait {
    fn fill_coord(&self, mx: f32, my: f32);
    fn fill_ref(&self, item: &u32);
    fn execute_command(&self, id_manager: &IDManager);
}

#[derive(Clone)]
pub enum UiCommand {
    SetPosition(Cell<Option<u32>>, Cell<f32>, Cell<f32>),
    ChangeColor(Cell<Option<u32>>, u32),
    SetText(Cell<Option<u32>>, String),
    Custom(Cell<Option<u32>>, Rc<dyn Fn(SharedDrawable)>),
    SetScale(Cell<Option<u32>>, u16),
    EditLabel(Cell<Option<u32>>),
    ScrollPanel(Cell<Option<u32>>, Cell<f32>, Cell<f32>),
    RequestRedraw(),
    RequestRedrawWithoutResize(),
    RequestRedrawWithTimer(Duration),
    SetOnClick(Cell<Option<u32>>, Box<UiCommand>),
    SetOnMouseEnter(Cell<Option<u32>>, Box<UiCommand>),
    SetOnMouseLeave(Cell<Option<u32>>, Box<UiCommand>),
    SetAnimation(Cell<Option<u32>>, Rc<Vec<AnimationSequence>>),
    AddAnimation(Cell<Option<u32>>, Rc<AnimationSequence>),
    AddAnimationBatch(Cell<Option<u32>>, Rc<Vec<AnimationSequence>>),
    StartAnimation(Cell<Option<u32>>),
    None(),
    Batch(Vec<UiCommand>),
    Other(Rc<dyn CommandTrait>),
    DragItem(Cell<Option<u32>>),
}

impl CommandTrait for UiCommand {
    fn fill_coord(&self, mx: f32, my: f32) {
        match self {
            UiCommand::ScrollPanel(_, x, y) => {
                x.set(-mx * x.get());
                y.set(-my * y.get());
            }
            UiCommand::SetPosition(_, x, y) => {
                x.set(mx);
                y.set(my);
            }
            UiCommand::Other(obj) => {
                obj.fill_coord(mx, my);
            }
            _ => (),
        }
    }
    fn fill_ref(&self, item: &u32) {
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
            | UiCommand::ScrollPanel(target, _, _)
            | UiCommand::SetPosition(target, _, _) => {
                if target.get().is_none() {
                    target.set(Some(*item));
                }
            }
            UiCommand::Other(obj) => {
                obj.fill_ref(item);
            }
            _ => (),
        }
    }
    fn execute_command(&self, id_manager: &IDManager) {
        match self {
            UiCommand::Batch(commands) => {
                for c in commands {
                    c.execute_command(id_manager);
                }
            }
            UiCommand::ChangeColor(id, _)
            | UiCommand::SetScale(id, _)
            | UiCommand::SetText(id, _)
            | UiCommand::SetOnClick(id, _)
            | UiCommand::SetOnMouseEnter(id, _)
            | UiCommand::SetOnMouseLeave(id, _)
            | UiCommand::SetAnimation(id, _)
            | UiCommand::AddAnimation(id, _)
            | UiCommand::AddAnimationBatch(id, _)
            | UiCommand::Custom(id, _)
            | UiCommand::ScrollPanel(id, _, _)
            | UiCommand::SetPosition(id, _, _) => {
                let id = &id.get();
                if let Some(el) = get_upgrade_by_id(id, id_manager) {
                    match self {
                        UiCommand::ChangeColor(_, color) => {
                            el.borrow_mut()
                                .as_panel_control_mut()
                                .set_background(*color);
                        }
                        UiCommand::SetScale(_, scale) => {
                            let mut e = el.borrow_mut();
                            if let Some(ctrl) = e.as_label_control_mut() {
                                ctrl.set_scale(*scale);
                            }
                        }
                        UiCommand::SetText(_, text) => {
                            let mut e = el.borrow_mut();
                            if let Some(ctrl) = e.as_label_control_mut() {
                                let text = text.clone();
                                ctrl.set_text(text);
                            }
                        }
                        UiCommand::SetOnClick(_, command) => {
                            if let Some(clickable) = el.borrow_mut().as_clickable_mut() {
                                let cmd = *command.clone();
                                clickable.set_on_click(cmd);
                            }
                        }
                        UiCommand::SetOnMouseEnter(_, command) => {
                            if let Some(hovearable) = el.borrow_mut().as_hoverable_mut() {
                                let cmd = *command.clone();
                                hovearable.set_on_mouse_enter(cmd);
                            }
                        }
                        UiCommand::SetOnMouseLeave(_, command) => {
                            if let Some(hovearable) = el.borrow_mut().as_hoverable_mut() {
                                let cmd = *command.clone();
                                hovearable.set_on_mouse_leave(cmd);
                            }
                        }
                        UiCommand::SetAnimation(_, animation) => {
                            if let Some(with_animation) = el.borrow_mut().as_with_animation_mut() {
                                let anim = (*(*animation).clone()).clone();
                                with_animation.set_animation(anim);
                            }
                        }
                        UiCommand::AddAnimation(_, animation) => {
                            if let Some(with_animation) = el.borrow_mut().as_with_animation_mut() {
                                let anim = (*(*animation).clone()).clone();
                                with_animation.add_animation(anim);
                            }
                        }

                        UiCommand::AddAnimationBatch(_, animations) => {
                            if let Some(with_animation) = el.borrow_mut().as_with_animation_mut() {
                                let anim = (*(*animations).clone()).clone();
                                with_animation.add_animation_batch(anim);
                            }
                        }
                        UiCommand::Custom(_, action) => {
                            (action)(el.clone());
                        }
                        UiCommand::ScrollPanel(_, mx, my) => {
                            if let Some(scrollable) = el.borrow_mut().as_scrollable_mut() {
                                scrollable.scroll(mx.get(), my.get());
                            }
                        }
                        UiCommand::SetPosition(_, mx, my) => {
                            let mut el = el.borrow_mut();

                            let rect = el.as_panel_control_mut().get_rect();

                            let mx = mx.get() + rect.x1;
                            let my = my.get() + rect.y1;
                            el.as_panel_control_mut().set_position(mx, my);
                        }
                        _ => (),
                    }
                }
            }
            UiCommand::Other(obj) => {
                obj.execute_command(id_manager);
            }
            _ => (),
        }
    }
}
