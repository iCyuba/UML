pub fn use_super() -> bool {
    let window = web_sys::window().unwrap();

    window
        .navigator()
        .user_agent()
        .unwrap()
        .to_lowercase()
        .contains("mac")
}
