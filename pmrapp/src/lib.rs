pub mod ac;
pub mod app;
#[cfg(not(feature = "ssr"))]
pub mod client;
pub mod component;
#[cfg(feature = "ssr")]
pub mod conf;
pub mod error;
pub mod error_template;
pub mod exposure;
#[cfg(feature = "ssr")]
pub mod server;
pub mod view;
pub mod workspace;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    let window = web_sys::window()
        .expect("window is missing");
    let performance = window.performance()
        .expect("performance is missing");

    let start = performance.now();
    leptos::logging::log!("starting hydration at {start} ms...");
    leptos::mount::hydrate_body(App);

    let total = performance.now() - start;
    leptos::logging::log!("called hydrate_body in {total} ms.");

    js_sys::Reflect::set(
        &window,
        &wasm_bindgen::JsValue::from_str("_hydrated"),
        &wasm_bindgen::JsValue::TRUE,
    ).expect("error setting hydrated status");
    let event = web_sys::Event::new("_hydrated")
        .expect("error creating hydrated event");
    let document = window.document()
        .expect("document is missing");
    document.dispatch_event(&event)
        .expect("error dispatching hydrated event");

    let end = performance.now();
    leptos::logging::log!("finished hydrate call at {end} ms.");
}
