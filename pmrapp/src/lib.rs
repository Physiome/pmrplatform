pub mod app;
#[cfg(feature = "ssr")]
pub mod conf;
pub mod error_template;
pub mod exposure;
#[cfg(feature = "ssr")]
pub mod fileserv;
pub mod workspace;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
