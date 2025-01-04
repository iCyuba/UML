use super::use_super;
use crate::{
    app::{event_target::WheelEvent, AppUserEvent},
    geometry::Vec2,
};
use web_sys::wasm_bindgen::{prelude::Closure, JsCast};
use winit::event_loop::EventLoopProxy;

pub fn setup_scroll_event(ev_proxy: EventLoopProxy<AppUserEvent>) {
    let window = web_sys::window().unwrap();
    let use_super = use_super();

    // Setup a better scroll handler
    let closure = Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
        ev_proxy
            .send_event(AppUserEvent::Scroll(WheelEvent {
                delta: -Vec2 {
                    x: event.delta_x(),
                    y: event.delta_y(),
                },
                zoom: event.ctrl_key() || (use_super && event.meta_key()),
                reverse: false,
                mouse: false,
            }))
            .unwrap();
    }) as Box<dyn FnMut(_)>);

    window
        .add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())
        .unwrap();

    closure.forget();
}
