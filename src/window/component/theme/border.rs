pub struct Border {
    pub color: u32,
    pub width: f32,
    pub radius: f32,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            color: 0xFF000000,
            width: 0.0,
            radius: 0.0,
        }
    }
}

impl Border {
    pub fn set_color(&mut self, color: u32) {
        self.color = color;
    }
    pub fn set_width(&mut self, width: f32) {
        self.width = width;
    }
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }
}
