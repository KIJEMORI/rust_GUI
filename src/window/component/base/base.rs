use crate::window::component::base::area::Rect;
pub struct Base {
    pub id: String,
    pub rect: Rect<u16>,
    pub visible: bool,
}

impl Base {
    pub fn new(id: String, rect: Rect<u16>) -> Base {
        Base {
            id: id,
            rect: rect,
            visible: true,
        }
    }

    #[allow(dead_code)]
    pub fn set_position(&mut self, x: u16, y: u16) {
        self.rect.set_position(x, y);
    }

    pub fn set_height(&mut self, h: u16) {
        self.rect.set_height(h);
    }

    pub fn set_width(&mut self, w: u16) {
        self.rect.set_width(w);
    }
}
