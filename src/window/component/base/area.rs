use std::ops::{Add, Sub};

use crate::window::component::interface::numeric::TypeToType;

#[derive(Clone, Copy)]
pub struct Size<T> {
    width: T,
    height: T,
}

impl<T> Size<T>
where
    T: Copy,
{
    pub fn new(w: T, h: T) -> Self {
        Self {
            width: w,
            height: h,
        }
    }
    pub fn set_height(&mut self, height: T) {
        self.height = height;
    }
    pub fn set_width(&mut self, width: T) {
        self.width = width;
    }
    pub fn get_height(&self) -> T {
        return self.height;
    }
    pub fn get_width(&self) -> T {
        return self.width;
    }
}
#[derive(Clone)]
pub struct Rect<T, W> {
    pub x1: T,
    pub y1: T,
    pub min: Size<W>,
    pub max: Size<W>,
}

impl<T, W> Default for Rect<T, W>
where
    T: Copy + Default,
    W: Copy + Default,
{
    fn default() -> Self {
        let min = Size {
            width: W::default(),
            height: W::default(),
        };

        let max = Size {
            width: W::default(),
            height: W::default(),
        };
        Self {
            x1: T::default(),
            y1: T::default(),
            min: min,
            max: max,
        }
    }
}

pub trait AreaMath<T, W> {
    fn get_x_offset(&self) -> T;
    fn get_x2(&self) -> T;
    fn get_y_offset(&self) -> T;
    fn get_y2(&self) -> T;
    fn change_width_on_coord(&mut self, x2: T);
    fn change_height_on_coord(&mut self, y2: T);
    fn contains(&self, x: T, y: T) -> bool;
    fn intersection(&self, other: &Self) -> bool;
}

impl<T, W> Rect<T, W>
where
    T: Copy + Add<T, Output = T> + Sub<T, Output = T> + TypeToType<W> + PartialOrd + Default,
    W: Copy + Into<T> + PartialOrd + Default,
{
    pub fn new(x: T, y: T, w: W, h: W) -> Self {
        let min = Size {
            width: w,
            height: h,
        };

        let max = Size {
            width: w,
            height: h,
        };

        Self {
            x1: x,
            y1: y,
            min: min,
            max: max,
        }
    }

    pub fn new_from_coord(first_point: (T, T), second_point: (T, T)) -> Self {
        let w = if second_point.0 > first_point.0 {
            (second_point.0 - first_point.0).cast()
        } else {
            W::default()
        };
        let h = if second_point.1 > first_point.1 {
            (second_point.1 - first_point.1).cast()
        } else {
            W::default()
        };

        let min = Size {
            width: w,
            height: h,
        };

        let max = Size {
            width: w,
            height: h,
        };

        Self {
            x1: first_point.0,
            y1: first_point.1,
            min: min,
            max: max,
        }
    }

    pub fn set_position(&mut self, x: T, y: T) {
        self.x1 = x;
        self.y1 = y;
    }

    pub fn set_width(&mut self, w: W) {
        self.max.width = w;
        self.min.width = w;
    }

    pub fn change_width(&mut self, w: W) {
        self.min.width = w
    }

    pub fn set_height(&mut self, h: W) {
        self.max.height = h;
        self.min.height = h;
    }

    pub fn change_height(&mut self, h: W) {
        self.min.height = h
    }
}

pub type Area = Rect<f32, u16>;

impl AreaMath<f32, u16> for Area {
    fn get_x_offset(&self) -> f32 {
        return self.x1 + self.max.get_width() as f32;
    }

    fn get_x2(&self) -> f32 {
        return self.x1 + self.min.get_width() as f32;
    }

    fn get_y_offset(&self) -> f32 {
        return self.y1 + self.max.get_height() as f32;
    }
    fn get_y2(&self) -> f32 {
        return self.y1 + self.min.get_height() as f32;
    }

    fn change_width_on_coord(&mut self, x2: f32) {
        if x2 > self.x1 {
            self.min.set_width((x2 - self.x1) as u16);
        } else {
            self.min.set_width(u16::default());
        }
    }

    fn change_height_on_coord(&mut self, y2: f32) {
        if y2 > self.y1 {
            self.min.set_height((y2 - self.y1) as u16);
        } else {
            self.min.set_height(u16::default());
        }
    }
    fn contains(&self, x: f32, y: f32) -> bool {
        if x >= self.x1
            && x <= self.x1 + self.min.get_width() as f32
            && y >= self.y1
            && y <= self.y1 + self.min.get_height() as f32
        {
            return true;
        }
        return false;
    }
    fn intersection(&self, other: &Self) -> bool {
        let x1 = if self.x1 > other.x1 {
            self.x1
        } else {
            other.x1
        };
        let y1 = if self.y1 > other.y1 {
            self.y1
        } else {
            other.y1
        };
        let x2 =
            if self.x1 + (self.min.get_width() as f32) < other.x1 + other.min.get_width() as f32 {
                self.x1 + self.min.get_width() as f32
            } else {
                other.x1 + other.min.get_width() as f32
            };
        let y2 = if self.y1 + (self.min.get_height() as f32)
            < other.y1 + other.min.get_height() as f32
        {
            self.y1 + self.min.get_height() as f32
        } else {
            other.y1 + other.min.get_height() as f32
        };

        // Если прямоугольники не пересекаются
        if x2 <= x1 || y2 <= y1 {
            return false;
        }

        true
    }
}
