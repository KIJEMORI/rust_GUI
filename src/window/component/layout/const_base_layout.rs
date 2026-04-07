use crate::window::component::interface::const_layout::{ConstLayout, DirectionControl};

#[allow(dead_code)]
#[derive(Clone, Copy, Default)]
pub struct Direction {
    pub up: i16,
    pub down: i16,
    pub right: i16,
    pub left: i16,
}

impl DirectionControl for Direction {
    fn up(&mut self, up: i16) {
        self.up = up;
    }
    fn down(&mut self, down: i16) {
        self.down = down;
    }
    fn right(&mut self, right: i16) {
        self.right = right;
    }
    fn left(&mut self, left: i16) {
        self.left = left;
    }

    fn set_direction(&mut self, up: i16, down: i16, right: i16, left: i16) {
        self.up(up);
        self.down(down);
        self.right(right);
        self.left(left);
    }
}

#[allow(dead_code)]
pub struct ConstBaseLayout {
    relative_width: u8,
    relative_height: u8,
}

impl ConstBaseLayout {
    pub fn new() -> Self {
        ConstBaseLayout {
            relative_width: 101,
            relative_height: 101,
        }
    }
}

impl ConstLayout for ConstBaseLayout {
    fn set_relative_width(&mut self, width: u8) {
        self.relative_width = width.min(100);
    }
    fn set_normal_width(&mut self) {
        self.relative_width = 101;
    }

    fn set_relative_height(&mut self, height: u8) {
        self.relative_height = height.min(100);
    }
    fn set_normal_height(&mut self) {
        self.relative_height = 101;
    }

    fn get_width(&self, width: u16, parent_width: u16) -> u16 {
        if self.relative_width != 101 {
            return ((parent_width as u32 * (self.relative_width as u32)) / 100) as u16;
        } else {
            return width;
        }
    }
    fn get_height(&self, height: u16, parent_height: u16) -> u16 {
        if self.relative_height != 101 {
            return ((parent_height as u32 * (self.relative_height as u32)) / 100) as u16;
        } else {
            return height;
        }
    }
}
