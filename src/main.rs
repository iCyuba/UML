mod animations;
mod app;
mod colors;
mod fonts;
mod renderer;
mod workspace;

use crate::app::App;
use winit::event_loop::EventLoop;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let mut app = App::new();
    let event_loop = EventLoop::new().unwrap();

    event_loop
        .run_app(&mut app)
        .expect("Couldn't run event loop");
}

#[cfg(target_arch = "wasm32")]
fn main() {
    wasm_bindgen_futures::spawn_local(async {
        let mut app = App::new();
        let event_loop = EventLoop::new().unwrap();

        app.renderer.init(&event_loop).await;

        event_loop
            .run_app(&mut app)
            .expect("Couldn't run event loop");
    });
}
