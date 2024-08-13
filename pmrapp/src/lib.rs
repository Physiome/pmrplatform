pub mod app;
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
    leptos::mount::hydrate_body(App);
}
