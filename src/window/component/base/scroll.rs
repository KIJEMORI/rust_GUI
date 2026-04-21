pub struct Scroll {
    pub offset: (f32, f32),
    pub height: u16,
    pub width: u16,
    pub slider_height: u16,
    pub slider_width: u16,
    pub min_slider_height: u16,
    pub min_slider_width: u16,
}

impl Scroll {
    pub fn new() -> Self {
        Self {
            offset: (0.0, 0.0),
            height: 0,
            width: 0,
            slider_height: 0,
            slider_width: 0,
            min_slider_height: 25,
            min_slider_width: 25,
        }
    }
    pub fn resize(&mut self) {
        self.height = 0;
        self.width = 0;
    }

    pub fn set_slider_height_width(&mut self, height: u16, width: u16) {
        self.slider_height = height;
        self.slider_width = width;
        if height as u16 > self.height {
            self.height = height as u16;
        }
        if width as u16 > self.width {
            self.width = width as u16;
        }
    }
    pub fn set_height_width(&mut self, height: u16, width: u16) {
        self.height = height as u16;
        self.width = width as u16;
    }
    pub fn get_offset(&self) -> (f32, f32) {
        self.offset
    }
    pub fn set_offset(&mut self, x: f32, y: f32) {
        self.offset = (x, y);
    }
    pub fn can_offset_x(&self, x: f32) -> bool {
        if self.offset.0 == -((self.width - self.slider_width) as f32) && x < 0.0
            || self.offset.0 == 0.0 && x > 0.0
        {
            return false;
        }
        true
    }

    pub fn change_offset_x(&mut self, x: f32) -> bool {
        if !self.can_offset_x(x) {
            return false;
        }

        let offset_x = self.offset.0 + x;
        //self.offset.0 = offset_x.max(0.0).min((self.width) as f32);
        self.offset.0 = offset_x
            .max(-((self.width - self.slider_width) as f32))
            .min(0.0);

        true
    }

    pub fn can_offset_y(&self, y: f32) -> bool {
        if self.offset.1 == -((self.height - self.slider_height) as f32) && y < 0.0
            || self.offset.1 == 0.0 && y > 0.0
        {
            return false;
        }
        true
    }
    pub fn change_offset_y(&mut self, y: f32) -> bool {
        if !self.can_offset_y(y) {
            return false;
        }

        let offset_y = self.offset.1 + y;
        self.offset.1 = offset_y
            .max(-((self.height - self.slider_height) as f32))
            .min(0.0);

        true
    }

    pub fn get_proportion_height(&self) -> f32 {
        self.slider_height as f32 / self.height as f32
    }
    pub fn get_proportion_width(&self) -> f32 {
        self.slider_width as f32 / self.width as f32
    }
    pub fn get_vertical_slider_height(&self, height: f32) -> u16 {
        (height * self.get_proportion_height()) as u16
    }
    pub fn get_horizontal_slider_width(&self, width: f32) -> u16 {
        (width * self.get_proportion_width()) as u16
    }

    pub fn get_vertical_progress(&self) -> f32 {
        let max_scroll = (self.height - self.slider_height) as f32;
        if max_scroll == 0.0 {
            return 0.0;
        }
        (self.offset.1.abs() / max_scroll).min(1.0)
    }
    pub fn get_horizontal_progress(&self) -> f32 {
        let max_scroll = (self.width - self.slider_width) as f32;
        if max_scroll == 0.0 {
            return 0.0;
        }
        (self.offset.0.abs() / max_scroll).min(1.0)
    }
}
