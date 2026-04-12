use std::sync::mpsc::Sender;
//use crate::window::component::base::font::Font;

use crate::window::component::base::ui_command::UiCommand;

pub struct Settings {
    pub background_color: u32,
    pub font_id: u32,
    pub command_tx: Option<Sender<UiCommand>>,
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            background_color: 0xFFFFFFFF,
            font_id: 0,
            command_tx: None,
        }
    }
    pub fn set_font(&mut self, font_id: u32) {
        self.font_id = font_id;
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_settings() -> Settings {
    Settings::new()
}
