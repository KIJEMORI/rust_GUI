pub struct LayoutContext<'a> {
    pub font: &'a fontdue::Font,
    // Базовый размер, при котором генерировался SDF (например, 64.0)
    pub sdf_base_size: f32,
}
