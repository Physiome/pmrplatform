use pmrcore::exposure::{ExposureFile, ExposureFileView};
use leptos::prelude::*;

#[component]
pub fn CellMLCodegen() -> impl IntoView {
    let ef = expect_context::<ExposureFile>();
    let efv = expect_context::<ExposureFileView>();
    view! {
        <h1>
            "Exposure "{ef.exposure_id}
            " - ExposureFile "{ef.workspace_file_path}
        </h1>
        <h3>"This is cellml_codegen"</h3>
    }
}
