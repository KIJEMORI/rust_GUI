use crate::window::component::interface::const_layout::{ConstLayout, DirectionControl};

#[allow(dead_code)]
#[derive(Clone, Copy, Default)]
pub struct Direction {
    pub up: i32,
    pub down: i32,
    pub right: i32,
    pub left: i32,
}

impl DirectionControl for Direction {
    fn up(&mut self, up: i32) {
        self.up = up;
    }
    fn down(&mut self, down: i32) {
        self.down = down;
    }
    fn right(&mut self, right: i32) {
        self.right = right;
    }
    fn left(&mut self, left: i32) {
        self.left = left;
    }

    fn set_direction(&mut self, up: i32, down: i32, right: i32, left: i32) {
        self.up(up);
        self.down(down);
        self.right(right);
        self.left(left);
    }
}

#[allow(dead_code)]
pub struct ConstBaseLayout {
    margin: Direction,
    padding: Direction,
}

impl ConstBaseLayout {
    pub fn new() -> Self {
        ConstBaseLayout {
            margin: Direction {
                up: 0,
                down: 0,
                right: 0,
                left: 0,
            },
            padding: Direction {
                up: 0,
                down: 0,
                right: 0,
                left: 0,
            },
        }
    }
}

impl ConstLayout for ConstBaseLayout {}
