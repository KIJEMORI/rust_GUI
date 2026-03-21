use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
//use crate::window::component::base::font::Font;

use std::sync::{Arc, Mutex};

pub struct Settings {
    // Храним только атрибуты (название, стиль), так как сами байты уже в FontSystem
    pub panel_background_color: u32,
}

static BACKGROUND_COLOR: OnceLock<Mutex<u32>> = OnceLock::new();

pub fn get_background_color() -> &'static Mutex<u32> {
    BACKGROUND_COLOR.get_or_init(|| Mutex::new(0xFFFFFFFF))
}
