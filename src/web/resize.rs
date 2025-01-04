use std::sync::Arc;
use web_sys::wasm_bindgen::{closure::Closure, JsCast};
use winit::{dpi::PhysicalSize, window::Window};

pub fn get_size() -> PhysicalSize<u32> {
    let window = web_sys::window().unwrap();
    let width = window.inner_width().unwrap().as_f64().unwrap();
    let height = window.inner_height().unwrap().as_f64().unwrap();
    let device_pixel_ratio = window.device_pixel_ratio();

    PhysicalSize::from_logical::<_, f64>((width, height), device_pixel_ratio)
}

pub fn resize(window: &Window) {
    let _ = window.request_inner_size(get_size());
}

pub fn setup_resize_event(window: Arc<Window>) {
    let cloned_window = window.clone();

    let resize_cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
        resize(&cloned_window);
    }) as Box<dyn FnMut(_)>);

    let web_window = web_sys::window().unwrap();
    web_window
        .add_event_listener_with_callback("resize", resize_cb.as_ref().unchecked_ref())
        .unwrap();

    resize_cb.forget();
}
