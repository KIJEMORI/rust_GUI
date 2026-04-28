pub trait ViewportControl {
    fn rotate_camera(&mut self, mx_offset: f32, my_offset: f32);
    fn change_distance_camera(&mut self, x_offset: f32, y_offset: f32);
}
