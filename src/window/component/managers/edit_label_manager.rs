use std::rc::Rc;

use winit::{
    event::{KeyEvent, Modifiers},
    keyboard::{Key, ModifiersKeyState, NamedKey},
};

use crate::window::component::base::component_type::{SharedDrawable, WeakSharedDrawable};

pub struct EditLabelManager {
    edit_label: Option<WeakSharedDrawable>,
    modifiers: Modifiers,
}

impl Default for EditLabelManager {
    fn default() -> Self {
        Self {
            edit_label: None,
            modifiers: Modifiers::default(),
        }
    }
}

impl EditLabelManager {
    pub fn is_editing(&self) -> bool {
        return self.edit_label.is_some();
    }
    pub fn set_edit_label(&mut self, label: SharedDrawable) {
        let item = Rc::downgrade(&label);

        if let Some(label) = item.upgrade() {
            if let Some(with_animation) = label.borrow_mut().as_with_animation() {
                with_animation.start_animation()
            }
            if let Some(label) = label.borrow_mut().as_edit_label_control_mut() {
                label.set_cursor();
            }
        }
        self.edit_label = Some(item);
    }
    pub fn handle_key(&mut self, event: KeyEvent, needs_layout: &mut bool) {
        if let Some(el) = self.edit_label.as_ref() {
            if let Some(el) = el.upgrade() {
                let mut e = el.borrow_mut();

                let mut user_interacted = false;

                if let Some(label) = e.as_edit_label_control_mut() {
                    if event.state.is_pressed() {
                        match event.logical_key {
                            Key::Named(NamedKey::Backspace) => {
                                label.backspace();
                                user_interacted = true;
                            }
                            Key::Named(NamedKey::Delete) => {
                                label.delete();
                                user_interacted = true;
                            }
                            Key::Named(NamedKey::ArrowRight) => {
                                label.move_cursor_right(true);
                                user_interacted = true;
                            }
                            Key::Named(NamedKey::ArrowLeft) => {
                                label.move_cursor_right(false);
                                user_interacted = true;
                            }
                            Key::Named(NamedKey::Escape) => {
                                if let Some(with_animation) = e.as_with_animation() {
                                    with_animation.stop_loop_animation();
                                }
                                drop(e);
                                self.edit_label = None;
                                *needs_layout = true;
                                return;
                            }
                            _ => {
                                if let Some(txt) = event.text {
                                    if !txt.chars().any(|c| c.is_control()) {
                                        label.add_text(&txt);
                                        user_interacted = true;
                                    }
                                }
                            }
                        }
                    }
                }

                if user_interacted {
                    *needs_layout = true;

                    if let Some(with_animation) = e.as_with_animation() {
                        with_animation.start_animation();
                    }

                    if let Some(label) = e.as_edit_label_control_mut() {
                        label.set_cursor();
                    }
                }
            }
        }
    }
    pub fn stop_edit(&mut self) {
        if let Some(label) = &self.edit_label {
            if let Some(label) = label.upgrade() {
                if let Some(with_animation) = label.borrow_mut().as_with_animation() {
                    with_animation.stop_loop_animation();
                }
                if let Some(label) = label.borrow_mut().as_edit_label_control_mut() {
                    label.delete_cursor();
                }
            }
        }
        self.edit_label = None;
    }
}
