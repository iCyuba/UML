use crate::app::renderer::Canvas;

pub trait Draw {
    fn draw(&self, canvas: &mut Canvas);
}
