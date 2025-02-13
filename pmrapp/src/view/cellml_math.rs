use pmrcore::exposure::{ExposureFile, ExposureFileView};
use leptos::prelude::*;
use crate::error::AppError;
use crate::error_template::ErrorTemplate;
use crate::exposure::api::read_blob;

#[component]
pub fn CellMLMath() -> impl IntoView {
    let ef = expect_context::<ExposureFile>();
    let efv = expect_context::<ExposureFileView>();
    let workspace_file_path = ef.workspace_file_path.clone();
    let math = Resource::new_blocking(
        move || (),
        move |_| {
            let workspace_file_path = workspace_file_path.clone();
            async move {
                read_blob(ef.exposure_id, workspace_file_path, efv.id, "math.json".to_string()).await
            }
        }
    );

    let math_view = move || {
        Suspend::new(async move {
            match math.await {
                Ok(math) => {
                    let math: Vec<(String, Vec<String>)> = serde_json::from_slice(&math)
                        .map_err(|_| AppError::InternalServerError)?;
                    Ok(math.into_iter()
                        .map(|(name, maths)| view! {
                            <div>
                                <h3>"Component: "{name}</h3>
                                {
                                    maths.into_iter()
                                        .map(|math| view! {
                                            <div inner_html=math></div>
                                        })
                                        .collect_view()
                                }
                            </div>
                        })
                        .collect_view())
                },
                _ => Err(AppError::NotFound)
            }
        })
    };

    view! {
        <h1>
            "Exposure "{ef.exposure_id}
            " - ExposureFile "{ef.workspace_file_path}
            " - Mathematics"
        </h1>
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                {math_view}
            </ErrorBoundary>
        </Transition>
    }
}
