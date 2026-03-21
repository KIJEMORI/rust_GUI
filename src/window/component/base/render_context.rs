use crate::window::component::base::area::Size;

pub struct RenderContext<'a> {
    pub buffer: &'a mut [u32],
    pub window_size: Size<u16>,
}
