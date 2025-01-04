use super::download;
use winit::{platform::web::WindowExtWebSys, window::Window};

pub fn screenshot(window: &Window) {
    let canvas = window.canvas().unwrap();

    _ = canvas.to_data_url().map(|data_url| {
        download(&data_url, "screenshot.png");
    });
}
