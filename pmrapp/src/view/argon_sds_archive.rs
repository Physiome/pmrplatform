use pmrcore::exposure::{ExposureFile, ExposureFileView};
use crate::exposure::{
    api::read_safe_index_html,
    ViewPath,
};
use leptos::prelude::*;
use leptos_meta::{Link, Script};

// TODO determine if this view module is the default main view for all
// exposure files with regards to documentation for the file itself.
// There may be multiple ways to render this so in effect under this
// pmrplatform, views under a simple name is simply expecting some
// "serialized" data that fits the particular view.  In this case it
// would be some index.html.

#[component]
pub fn ArgonSdsArchive() -> impl IntoView {
    let ef = expect_context::<ExposureFile>();
    let efv = expect_context::<ExposureFileView>();
    let view_path = expect_context::<ViewPath>();

    let url = format!(
        "/data/exposure/{}/{}/{}/{}",
        ef.exposure_id,
        ef.id,
        efv.view_key.as_ref().expect("this is inside a resolved view"),
        "derivative/Scaffold/scaffold_metadata.json",
    );

    view! {
        <Script id="scaffoldvuer" src="https://unpkg.com/vue@2.6.10"/>
        // <Script id="scaffoldvuer" src="/pkg/scaffoldvuer-wc.umd.min.js"/>
        <Script id="scaffoldvuer" type_="module" src="https://unpkg.com/@abi-software/scaffoldvuer@1.6.2-wc/dist/scaffoldvuer-wc.js"/>
        <Link id="scaffoldvuer_stylesheet" href="https://unpkg.com/@abi-software/scaffoldvuer@1.6.2-wc/dist/style.css" rel="stylesheet"/>
        <h1>
            "Exposure "{ef.exposure_id}
            " - ExposureFile "{ef.workspace_file_path}
        </h1>
        <div id="ArgonSdsArchive">
            <div>
                <scaffoldvuer-wc url=url/>
            </div>
        </div>
    }
}
