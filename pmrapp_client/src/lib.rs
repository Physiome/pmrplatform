pub use app::{App, Msg};
use sauron::prelude::*;
pub use sauron;

mod api;
mod app;
mod error;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub async fn main(serialized_state: String) {
    #[cfg(feature = "wasm-bindgen")]
    {
        console_log::init_with_level(log::Level::Trace).ok();
        console_error_panic_hook::set_once();
    }
    if serialized_state == "" {
        return;
    }

    let app = match serde_json::from_str::<App>(&serialized_state) {
        Ok(app_state) => app_state,
        Err(e) => {
            log::warn!("error: {}", e);
            App::default()
        }
    };
    Program::replace_mount(app, &document().query_selector("body").unwrap().unwrap());
}
