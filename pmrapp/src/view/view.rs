use pmrcore::exposure::{ExposureFile, ExposureFileView};
use crate::exposure::api::read_safe_index_html;
use leptos::prelude::*;

// TODO determine if this view module is the default main view for all
// exposure files with regards to documentation for the file itself.
// There may be multiple ways to render this so in effect under this
// pmrplatform, views under a simple name is simply expecting some
// "serialized" data that fits the particular view.  In this case it
// would be some index.html.

#[component]
pub fn View() -> impl IntoView {
    let ef = expect_context::<ExposureFile>();
    let efv = expect_context::<ExposureFileView>();
    let workspace_file_path = ef.workspace_file_path.clone();
    let index_html = Resource::new_blocking(
        move || (),
        move |_| {
            let workspace_file_path = workspace_file_path.clone();
            async move {
                read_safe_index_html(
                    ef.exposure_id,
                    workspace_file_path,
                    efv.id,
                ).await
            }
        }
    );
    let index_view = move || {
        Suspend::new(async move {
            index_html.await.map(|index_html| {
                view! {
                    <div inner_html=index_html></div>
                }
            })
        })
    };
    view! {
        <h1>
            "Exposure "{ef.exposure_id}
            " - ExposureFile "{ef.workspace_file_path}
        </h1>
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            {index_view}
        </Transition>
    }
}
