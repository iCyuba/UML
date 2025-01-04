use web_sys::{
    js_sys::{Array, Uint8Array},
    wasm_bindgen::JsCast,
    Blob, HtmlElement, Url,
};

pub fn download(url: &str, name: &str) {
    let document = web_sys::window().unwrap().document().unwrap();

    let a = document.create_element("a").unwrap();

    a.set_attribute("href", url).unwrap();
    a.set_attribute("download", name).unwrap();

    a.dyn_ref::<HtmlElement>().unwrap().click();
}

pub fn download_bytes(bytes: &[u8], name: &str) {
    let uint8_array = Uint8Array::from(bytes);
    let blob = Blob::new_with_u8_array_sequence(&Array::of1(&uint8_array)).unwrap();

    let url = Url::create_object_url_with_blob(&blob).unwrap();

    download(&url, name);

    Url::revoke_object_url(&url).unwrap();
}
