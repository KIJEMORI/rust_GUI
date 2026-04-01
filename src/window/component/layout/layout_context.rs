use wgpu_glyph::ab_glyph::FontArc;

pub struct LayoutContext<'a> {
    pub fonts: &'a [FontArc],
    // тут могут быть еще DPI или настройки интервалов
}
