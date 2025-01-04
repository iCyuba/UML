#![windows_subsystem="windows"]

mod animations;
mod app;
mod data;
mod elements;
mod geometry;
mod presentation;
#[cfg(target_arch = "wasm32")]
mod web;

use crate::app::{App, AppUserEvent};
use std::error::Error;
use winit::event_loop::EventLoop;

#[cfg(not(target_arch = "wasm32"))]
#[pollster::main]
async fn main() {
    run().await.unwrap();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    wasm_bindgen_futures::spawn_local(async { run().await.unwrap() });
}

async fn run() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::<AppUserEvent>::with_user_event().build()?;

    let mut app = App::new(event_loop.create_proxy()).await;

    #[cfg(target_arch = "wasm32")]
    app.window.init(&event_loop).await?;

    event_loop.run_app(&mut app)?;

    Ok(())
}
