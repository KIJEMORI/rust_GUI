use crate::window::component::{
    base::ui_command::UiCommand, interface::drawable::Drawable, label::Label,
};

#[allow(dead_code)]
pub struct EditLabel {
    label: Label,
}

impl EditLabel {
    pub fn new(text: &str) -> Self {
        let mut label = Label::new(text.to_string());

        label.set_on_click(UiCommand::EditLabel(None));

        EditLabel { label: label }
    }
}

impl Default for EditLabel {
    fn default() -> Self {
        EditLabel::new("")
    }
}
