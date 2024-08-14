use pmrcore::exposure::{ExposureFile, ExposureFileView};
use leptos::prelude::*;
use crate::error::AppError;
use crate::error_template::ErrorTemplate;
use crate::exposure::ViewPath;
use crate::exposure::api::read_blob;

#[component]
pub fn CellMLCodegen() -> impl IntoView {
    let ef = expect_context::<ExposureFile>();
    let efv = expect_context::<ExposureFileView>();
    let view_path = expect_context::<ViewPath>();
    let workspace_file_path = ef.workspace_file_path.clone();
    let code = Resource::new_blocking(
        move || view_path.clone().0,
        move |k| {
            let workspace_file_path = workspace_file_path.clone();
            async move {
                let k = k.map(|k| {
                    match k.as_str() {
                        "C" => Some("code.C.c"),
                        "python" => Some("code.Python.py"),
                        _ => None
                    }
                }).flatten();
                if let Some(k) = k {
                    Some(read_blob(ef.exposure_id, workspace_file_path, efv.id, k.to_string()).await)
                } else {
                    None
                }
            }
        }
    );

    let view_path = expect_context::<ViewPath>().0;
    let code_view = move || {
        let view_path = view_path.clone();
        Suspend::new(async move {
            // FIXME the links only work if route is not with a trailing slash,
            // i.e. cellml_codegen/
            match view_path {
                None => Ok(view! {
                    <ul>
                        <li><a href="cellml_codegen/C">"C"</a></li>
                        <li><a href="cellml_codegen/python">"Python"</a></li>
                    </ul>
                }.into_any()),
                Some(_) => {
                    match code.await {
                        Some(Ok(code)) => {
                            let code = String::from_utf8(code.into_vec())
                                .map_err(|_| AppError::InternalServerError)?;
                            Ok(view! {
                                <div><a href="..">Back</a></div>
                                <code><pre>{code}</pre></code>
                            }.into_any())
                        },
                        _ => Err(AppError::NotFound)
                    }
                },
            }
        })
    };

    let view_path = expect_context::<ViewPath>().0;
    view! {
        <h1>
            "Exposure "{ef.exposure_id}
            " - ExposureFile "{ef.workspace_file_path}
        </h1>
        <h3>"This is "{efv.view_key}", at path "{view_path}</h3>
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|errors| view!{ <ErrorTemplate errors/>}>
                {code_view}
            </ErrorBoundary>
        </Suspense>
    }
}
