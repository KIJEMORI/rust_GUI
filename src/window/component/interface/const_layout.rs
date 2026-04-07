#[allow(dead_code)]
pub trait DirectionControl {
    fn up(&mut self, up: i16);
    fn down(&mut self, down: i16);
    fn right(&mut self, right: i16);
    fn left(&mut self, left: i16);
    fn set_direction(&mut self, up: i16, down: i16, right: i16, left: i16);
}
#[allow(dead_code)]
pub trait ConstLayout {
    fn set_relative_width(&mut self, width: u8);
    fn set_normal_width(&mut self);
    fn set_relative_height(&mut self, height: u8);
    fn set_normal_height(&mut self);
    fn get_width(&self, width: u16, parent_width: u16) -> u16;
    fn get_height(&self, height: u16, parent_height: u16) -> u16;
}
