use gloo_utils::format::JsValueSerdeExt;
use js_sys::{
    Object,
    Reflect::{get, set},
};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen(module = "/dist/pmrapp-bundle.js")]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = hljs, js_name = highlight)]
    fn highlight_lang(code: String, options: Object) -> Result<Object, JsValue>;
}

pub fn highlight(code: String, lang: String) -> Result<String, JsValue> {
    let options = js_sys::Object::new();
    set(&options, &"language".into(), &lang.into())
        .expect("failed to assign lang to options");
    highlight_lang(code, options)
        .map(|result| {
            let value = get(&result, &"value".into())
                .expect("HighlightResult failed to contain the value key");
            value.into_serde().expect("Value should have been a string")
        })
}
