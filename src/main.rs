mod fonts;
mod renderer;
mod app;
mod workspace;

use winit::event_loop::EventLoop;

use crate::app::App;

fn main() {
    let mut app = App::new();

    let event_loop = EventLoop::new().unwrap();
    event_loop
        .run_app(&mut app)
        .expect("Couldn't run event loop");
}
