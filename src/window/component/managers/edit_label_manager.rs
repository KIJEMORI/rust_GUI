use winit::{
    event::{KeyEvent, Modifiers},
    keyboard::{Key, ModifiersKeyState, NamedKey},
};

use crate::window::component::{
    layout::layout_context::LayoutContext, managers::id_manager::IDManager,
};

pub struct EditLabelManager {
    edit_label: Option<u32>,
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
    pub fn set_edit_label(&mut self, id: &u32, id_manager: &IDManager, ctx: &LayoutContext) {
        if let Some(label) = id_manager.get_upgraded(id) {
            let mut label = label.borrow_mut();
            if let Some(with_animation) = label.as_with_animation_mut() {
                with_animation.start_animation()
            }
            if let Some(label) = label.as_edit_label_control_mut() {
                label.set_cursor();
            }

            label.resize_one(ctx);
        }
        self.edit_label = Some(id.clone());
    }
    pub fn handle_key(
        &mut self,
        event: KeyEvent,
        needs_layout: &mut bool,
        id_manager: &IDManager,
        ctx: &LayoutContext,
    ) {
        if let Some(id) = self.edit_label.as_ref() {
            if let Some(el) = id_manager.get_upgraded(id) {
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
                                if let Some(with_animation) = e.as_with_animation_mut() {
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
                    e.resize_one(ctx);

                    *needs_layout = true;

                    if let Some(with_animation) = e.as_with_animation_mut() {
                        with_animation.start_animation();
                    }

                    if let Some(label) = e.as_edit_label_control_mut() {
                        label.set_cursor();
                    }
                }
            } else {
                self.edit_label = None;
            }
        }
    }
    pub fn stop_edit(&mut self, id_manager: &IDManager) {
        if let Some(id) = &self.edit_label {
            if let Some(label) = id_manager.get_upgraded(id) {
                if let Some(with_animation) = label.borrow_mut().as_with_animation_mut() {
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
