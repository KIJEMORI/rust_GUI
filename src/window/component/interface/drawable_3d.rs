pub trait ViewportControl {
    fn rotate_camera(&mut self, mx_offset: f32, my_offset: f32);
}
