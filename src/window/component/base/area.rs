use std::ops::{Add, Sub};

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
pub struct Rect<T> {
    pub x1: T,
    pub y1: T,
    pub x2: T,
    pub y2: T,
    pub min: Size<T>,
    pub max: Size<T>,
}

impl<T> Default for Rect<T>
where
    T: Copy + Add<Output = T> + Sub<Output = T> + PartialOrd + Default,
{
    fn default() -> Self {
        let min = Size {
            width: T::default(),
            height: T::default(),
        };

        let max = Size {
            width: T::default(),
            height: T::default(),
        };
        Self {
            x1: T::default(),
            y1: T::default(),
            x2: T::default(),
            y2: T::default(),
            min: min,
            max: max,
        }
    }
}

impl<T> Rect<T>
where
    T: Copy + Add<Output = T> + Sub<Output = T> + PartialOrd + Default,
{
    pub fn new(x: T, y: T, w: T, h: T) -> Self {
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
            x2: x + w,
            y2: y + h,
            min: min,
            max: max,
        }
    }

    pub fn get_x_offset(&self) -> T {
        return self.x1 + self.max.get_width();
    }

    pub fn get_y_offset(&self) -> T {
        return self.y1 + self.max.get_height();
    }

    pub fn new_from_coord(first_point: (T, T), second_point: (T, T)) -> Self {
        let w = if second_point.0 > first_point.0 {
            second_point.0 - first_point.0
        } else {
            T::default()
        };
        let h = if second_point.1 > first_point.1 {
            second_point.1 - first_point.1
        } else {
            T::default()
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
            x2: second_point.0,
            y2: second_point.1,
            min: min,
            max: max,
        }
    }

    pub fn set_position(&mut self, x: T, y: T) {
        self.x1 = x;
        self.y1 = y;
        self.x2 = x + self.max.get_width();
        self.y2 = y + self.max.get_height();
    }

    pub fn set_width(&mut self, w: T) {
        self.max.width = w;
        self.min.width = w;
    }

    pub fn change_width(&mut self, w: T) {
        self.x2 = self.x1 + w;
        self.min.width = w
    }

    pub fn change_width_on_coord(&mut self, x2: T) {
        if x2 > self.x1 {
            self.x2 = x2;
            self.min.set_width(x2 - self.x1);
        } else {
            self.x2 = self.x1;
            self.min.set_width(T::default());
        }
    }

    pub fn set_height(&mut self, h: T) {
        self.max.height = h;
        self.min.height = h;
    }

    pub fn change_height(&mut self, h: T) {
        self.y2 = self.y1 + h;
        self.min.height = h
    }

    pub fn change_height_on_coord(&mut self, y2: T) {
        if y2 > self.y1 {
            self.y2 = y2;
            self.min.set_height(y2 - self.y1);
        } else {
            self.y2 = self.y1;
            self.min.set_height(T::default());
        }
    }
    pub fn contains(&self, x: T, y: T) -> bool {
        if x >= self.x1 && x <= self.x2 && y >= self.y1 && y <= self.y2 {
            return true;
        }
        return false;
    }
    pub fn intersection(&self, other: &Self) -> bool {
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
        let x2 = if self.x2 < other.x2 {
            self.x2
        } else {
            other.x2
        };
        let y2 = if self.y2 < other.y2 {
            self.y2
        } else {
            other.y2
        };

        // Если прямоугольники не пересекаются
        if x2 <= x1 || y2 <= y1 {
            return false;
        }

        true
    }
}
