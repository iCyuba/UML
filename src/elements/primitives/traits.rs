use vello::Scene;

pub trait Draw {
    fn draw(&self, canvas: &mut Scene);
}
