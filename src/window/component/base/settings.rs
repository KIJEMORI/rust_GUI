use std::sync::mpsc::Sender;
//use crate::window::component::base::font::Font;

use crate::window::component::base::ui_command::UiCommand;
use wgpu_glyph::FontId;

pub struct Settings {
    pub background_color: u32,
    pub font_id: FontId,
    pub command_tx: Option<Sender<UiCommand>>,
}

impl Settings {
    pub fn new(path: Option<String>) -> Self {
        Settings {
            background_color: 0xFFFFFFFF,
            font_id: FontId::default(),
            command_tx: None,
        }
    }
    pub fn set_font(&mut self, font_id: FontId) {
        self.font_id = font_id;
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new(Some(r"C:\Users\KOCH\Documents\GameEngine\visual\src\window\component\base\Fonts\calibri.ttf".to_string()))
    }
}

pub fn get_settings() -> Settings {
    Settings::new(None)
}
