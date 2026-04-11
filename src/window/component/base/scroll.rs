#[derive(Default)]
pub struct Scroll {
    pub offset: (f32, f32),
    pub height: u16,
    pub width: u16,
    slider_height: u16,
    slider_width: u16,
}

impl Scroll {
    pub fn new() -> Self {
        Self {
            offset: (0.0, 0.0),
            height: 0,
            width: 0,
            slider_height: 0,
            slider_width: 0,
        }
    }
    pub fn resize(&mut self) {
        self.height = 0;
        self.width = 0;
    }
    pub fn set_slider_height_width(&mut self, height: i16, width: i16) {
        self.slider_height = height as u16;
        self.slider_width = width as u16;
    }
    pub fn set_height_width(&mut self, height: i16, width: i16) {
        self.height = self.height.max(height as u16);
        self.width = self.width.max(width as u16);
    }
    pub fn get_offset(&self) -> (f32, f32) {
        self.offset
    }
    pub fn set_offset(&mut self, x: f32, y: f32) {
        self.offset = (x, y);
    }
    pub fn change_offset_x(&mut self, x: f32) -> bool {
        // if self.offset.0 == 0.0 && x < 0.0 || self.offset.0 == self.width as f32 && x > 0.0 {
        //     return false;
        // }
        if self.offset.0 == -(self.width as f32) && x < 0.0 || self.offset.0 == 0.0 && x > 0.0 {
            return false;
        }

        let offset_x = self.offset.0 + x;
        //self.offset.0 = offset_x.max(0.0).min((self.width) as f32);
        self.offset.0 = offset_x
            .max(-((self.width - self.slider_width) as f32))
            .min(0.0);

        true
    }
    pub fn change_offset_y(&mut self, y: f32) -> bool {
        if self.offset.1 == -(self.height as f32) && y < 0.0 || self.offset.1 == 0.0 && y > 0.0 {
            return false;
        }

        let offset_y = self.offset.1 + y;
        self.offset.1 = offset_y
            .max(-((self.height - self.slider_height) as f32))
            .min(0.0);

        true
    }
}
