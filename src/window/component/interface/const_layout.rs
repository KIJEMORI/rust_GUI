#[allow(dead_code)]
pub trait DirectionControl {
    fn up(&mut self, up: i32);
    fn down(&mut self, down: i32);
    fn right(&mut self, right: i32);
    fn left(&mut self, left: i32);
    fn set_direction(&mut self, up: i32, down: i32, right: i32, left: i32);
}
#[allow(dead_code)]
pub trait ConstLayout {}
