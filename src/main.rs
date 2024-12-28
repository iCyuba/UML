mod animations;
mod app;
mod data;
mod elements;
mod geometry;
mod presentation;
mod sample;

use crate::app::{App, AppUserEvent};
use winit::event_loop::EventLoop;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let event_loop = EventLoop::<AppUserEvent>::with_user_event()
        .build()
        .unwrap();

    let mut app = App::new(event_loop.create_proxy());

    event_loop
        .run_app(&mut app)
        .expect("Couldn't run event loop");
}

#[cfg(target_arch = "wasm32")]
fn main() {
    wasm_bindgen_futures::spawn_local(async {
        let event_loop = EventLoop::<AppUserEvent>::with_user_event()
            .build()
            .unwrap();

        let mut app = App::new(event_loop.create_proxy());

        app.renderer.init(&event_loop).await;

        event_loop
            .run_app(&mut app)
            .expect("Couldn't run event loop");
    });
}
