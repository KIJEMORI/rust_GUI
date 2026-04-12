pub trait TypeToType<W> {
    fn cast(self) -> W;
}

impl TypeToType<u16> for f32 {
    fn cast(self) -> u16 {
        self as u16
    }
}
