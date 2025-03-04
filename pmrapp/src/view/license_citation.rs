use pmrcore::exposure::{ExposureFile, ExposureFileView};
use leptos::prelude::*;
use crate::error::AppError;
use crate::error_template::ErrorTemplate;
use crate::exposure::api::read_blob;

#[component]
pub fn LicenseCitation() -> impl IntoView {
    let ef = expect_context::<ExposureFile>();
    let efv = expect_context::<ExposureFileView>();
    let workspace_file_path = ef.workspace_file_path.clone();
    let license = Resource::new_blocking(
        move || (),
        move |_| {
            let workspace_file_path = workspace_file_path.clone();
            async move {
                read_blob(ef.exposure_id, workspace_file_path, efv.id, "license.txt".to_string()).await
            }
        }
    );

    let view = move || {
        Suspend::new(async move {
            match license.await {
                Ok(license) => {
                    let license = std::str::from_utf8(&license)
                        .map_err(|_| AppError::InternalServerError)?
                        .to_string();
                    Ok(view! {
                        <h3>"License"</h3>
                        <p>"This work is licensed under "{license}"."</p>
                    })
                },
                _ => Err(AppError::NotFound)
            }
        })
    };

    view! {
        <h1>
            "Exposure "{ef.exposure_id}
            " - ExposureFile "{ef.workspace_file_path}
            " - License Citation"
        </h1>
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                {view}
            </ErrorBoundary>
        </Transition>
    }
}
