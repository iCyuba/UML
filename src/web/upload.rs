use crate::app::AppUserEvent;
use web_sys::{
    js_sys::Uint8Array,
    wasm_bindgen::{prelude::Closure, JsCast, JsValue},
    Event, HtmlInputElement,
};
use winit::event_loop::EventLoopProxy;

pub fn setup_file_picker(ev_proxy: EventLoopProxy<AppUserEvent>) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let input = document.create_element("input").unwrap();
    let input = input.dyn_into::<HtmlInputElement>().unwrap();

    input.set_type("file");
    input.set_accept(".bin,application/octet-stream,.json,application/json");

    document.body().unwrap().append_child(&input).unwrap();

    input.set_id("file-input");
    input.set_hidden(true);

    let parse = Closure::wrap(Box::new(move |array_buffer: JsValue| {
        let document = web_sys::window().unwrap().document().unwrap();

        let input = document.get_element_by_id("file-input").unwrap();
        let input = input.dyn_into::<HtmlInputElement>().unwrap();

        let files = input.files().unwrap();
        let file = files.get(0).unwrap();
        let name = file.name();

        input.set_value("");

        let vec = Uint8Array::new(&array_buffer).to_vec();

        ev_proxy
            .send_event(AppUserEvent::FileLoaded(vec, name))
            .unwrap();
    }) as Box<dyn FnMut(_)>);

    let on_change = Closure::wrap(Box::new(move |_: Event| {
        let document = web_sys::window().unwrap().document().unwrap();

        let input = document.get_element_by_id("file-input").unwrap();
        let input = input.dyn_into::<HtmlInputElement>().unwrap();

        let files = input.files().unwrap();
        let file = files.get(0).unwrap();

        _ = file.array_buffer().then(&parse);
    }) as Box<dyn FnMut(_)>);

    input.set_onchange(Some(on_change.as_ref().unchecked_ref()));

    on_change.forget();
}

pub fn open_file_picker() {
    let document = web_sys::window().unwrap().document().unwrap();

    let input = document.get_element_by_id("file-input").unwrap();
    let input = input.dyn_into::<HtmlInputElement>().unwrap();

    input.click();
}
